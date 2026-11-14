"""Build/copy/package/install the P2.1 native pilot and audit both consumers.

Network is needed only to fetch build dependencies. No credentials, publication,
source execution from wheels/tarballs outside normal install, or market feed.
Consumer directories are fresh temporary directories outside the repository.
"""
import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from p2_scenarios import scenarios, verify


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence", type=Path, default=Path("ci-evidence/p2"))
    parser.add_argument("--bootstrap-native-locks", action="store_true",
                        help="Initial dependency resolution only; commit resulting native Cargo.lock files before acceptance")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix="wyc-p2-consumer-"))
    assert root != scratch and root not in scratch.parents, "Consumer must be outside source tree"
    checks = []
    summary = dict(started_at_utc=datetime.now(timezone.utc).isoformat(),
                   platform=platform.platform(), python=sys.version, checks=checks, passed=False,
                   native_locks_bootstrapped=args.bootstrap_native_locks, consumer_outside_source=True,
                   locked_source_acceptance=False)

    def run(name, command, cwd=root, input_text=None):
        result = subprocess.run([str(c) for c in command], cwd=cwd, input=input_text,
                                text=True, encoding="utf-8", stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        (evidence / f"{name}.txt").write_text(result.stdout + result.stderr, encoding="utf-8")
        checks.append(dict(name=name, command=[str(c) for c in command], exit_code=result.returncode))
        if result.returncode:
            raise RuntimeError(f"{name} failed with exit code {result.returncode}; see {evidence / (name + '.txt')}")
        return result.stdout

    try:
        summary["commit"] = run("commit", ["git", "rev-parse", "HEAD"]).strip()
        summary["clean_before"] = not run("status-before", ["git", "status", "--porcelain"]).strip()
        run("node-version", ["node", "--version"])
        npm = shutil.which("npm.cmd" if os.name == "nt" else "npm")
        if not npm:
            raise RuntimeError("npm is required")
        run("npm-version", [npm, "--version"])
        run("maturin-version", [sys.executable, "-m", "maturin", "--version"])
        for adapter in ("node", "python"):
            manifest = root / "bindings" / adapter / "Cargo.toml"
            if args.bootstrap_native_locks:
                run(f"{adapter}-lock-bootstrap", ["cargo", "generate-lockfile", "--manifest-path", manifest])
            lock = manifest.with_name("Cargo.lock")
            if not lock.exists():
                raise RuntimeError(f"Missing {adapter} Cargo.lock: bootstrap once on a connected host and review/commit it")
            shutil.copy2(lock, evidence / f"{adapter}-Cargo.lock")
            run(f"{adapter}-format", ["cargo", "fmt", "--manifest-path", manifest, "--", "--check"])
            run(f"{adapter}-clippy", ["cargo", "clippy", "--manifest-path", manifest, "--locked", "--all-targets", "--", "-D", "warnings"])
        target = scratch / "target"
        run("node-build", ["cargo", "build", "--manifest-path", root / "bindings/node/Cargo.toml",
                            "--locked", "--release", "--target-dir", target])
        stage = scratch / "node-stage"
        stage.mkdir()
        for name in ("package.json", "index.cjs", "index.d.ts"):
            shutil.copy2(root / "bindings/node" / name, stage / name)
        library = "wyc_node_pilot.dll" if os.name == "nt" else "libwyc_node_pilot.so"
        shutil.copy2(target / "release" / library, stage / "wyc_lib.node")
        pack = json.loads(run("npm-pack", [npm, "pack", "--json", "--ignore-scripts", "--pack-destination", scratch], cwd=stage))
        tarball = scratch / pack[0]["filename"]
        consumer = scratch / "consumer"
        consumer.mkdir()
        (consumer / "package.json").write_text('{"name":"wyc-p2-consumer","private":true}', encoding="utf-8")
        run("npm-install", [npm, "install", "--ignore-scripts", "--no-audit", "--no-fund", tarball], cwd=consumer)
        wheel_dir = scratch / "wheels"
        command = [sys.executable, "-m", "maturin", "build", "--manifest-path", root / "bindings/python/Cargo.toml",
                   "--locked", "--release", "--interpreter", sys.executable, "--out", wheel_dir]
        if sys.platform == "linux":
            command += ["--compatibility", "linux"]
        run("wheel-build", command)
        wheels = list(wheel_dir.glob("*.whl"))
        if len(wheels) != 1:
            raise RuntimeError("Expected exactly one CPython-specific wheel")
        venv = scratch / "consumer-venv"
        run("venv", [sys.executable, "-m", "venv", venv])
        python = venv / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
        run("wheel-install", [python, "-m", "pip", "install", "--no-index", "--no-deps", wheels[0]], cwd=consumer)
        run("reference-build", ["cargo", "build", "--locked", "--package", "wyc-binding-contract",
                                "--bin", "p2_reference", "--target-dir", target])
        reference = target / "debug" / ("p2_reference.exe" if os.name == "nt" else "p2_reference")
        actions, requests, errors, golden = scenarios(root)
        actions_path = consumer / "actions.json"
        actions_path.write_text(json.dumps(actions, ensure_ascii=False), encoding="utf-8")
        shutil.copy2(root / "tests/parity/p2_smoke.cjs", consumer / "smoke.cjs")
        shutil.copy2(root / "tests/parity/p2_smoke.py", consumer / "smoke.py")
        references = {name: json.loads(run(f"reference-{name}", [reference], input_text=json.dumps(r)))
                      for name, r in requests.items()}
        node_records = json.loads(run("node-smoke", ["node", "--expose-gc", "smoke.cjs", actions_path], cwd=consumer))
        python_records = json.loads(run("python-smoke", [python, "smoke.py", actions_path], cwd=consumer))
        assert [r["name"] for r in node_records] == [a["name"] for a in actions], "Node action coverage"
        assert [r["name"] for r in python_records] == [a["name"] for a in actions], "Python action coverage"
        summary["node_smoke"] = verify(node_records, references, errors, golden)
        summary["python_smoke"] = verify(python_records, references, errors, golden)
        assert node_records == python_records, "Node/Python transport or error mapping differs"
        summary["packages"] = [{"name": p.name, "sha256": hashlib.sha256(p.read_bytes()).hexdigest(),
                                 "size": p.stat().st_size} for p in (tarball, wheels[0])]
        for p in (tarball, wheels[0]):
            shutil.copy2(p, evidence / p.name)
        summary["clean_after"] = not run("status-after", ["git", "status", "--porcelain"]).strip()
        summary["passed"] = True
        # Bootstrap success still does not satisfy locked-source acceptance.
        summary["locked_source_acceptance"] = (not args.bootstrap_native_locks and
                                                summary["clean_before"] and summary["clean_after"])
    finally:
        summary["finished_at_utc"] = datetime.now(timezone.utc).isoformat()
        (evidence / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
        # Public, source-only evidence for review through decoded Actions logs.
        # No credentials, redirected download URLs or caller payloads are emitted.
        print("P2_EVIDENCE " + json.dumps(summary, separators=(",", ":")), flush=True)
        if args.bootstrap_native_locks:
            for adapter in ("node", "python"):
                lock = root / "bindings" / adapter / "Cargo.lock"
                if lock.exists():
                    content = lock.read_text(encoding="utf-8")
                    print("P2_LOCK " + json.dumps(dict(adapter=adapter,
                        sha256=hashlib.sha256(content.encode()).hexdigest(), content=content),
                        separators=(",", ":")), flush=True)
    print(f"PASS: native pilot smoke; evidence: {evidence}")


if __name__ == "__main__":
    main()
