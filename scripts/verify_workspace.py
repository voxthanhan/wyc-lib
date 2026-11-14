"""Audit the committed workspace boundary and local documentation links."""
import json
import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import unquote


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def main():
    root = Path(__file__).resolve().parents[1]
    metadata = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8-sig"))
    packages = {p["name"]: p for p in metadata["packages"]}
    check(set(packages) == {"contracts", "indicators", "market_structure", "wyc-lib", "wyc-binding-contract"}, "P2.1 workspace membership changed")
    check(all(p["publish"] == [] for p in packages.values()), "packages must remain non-publishable")
    for target in ["foundation", "p1_foundation", "p1_rolling", "p1_basic", "p1_advanced", "p1_structure", "p1_orchestration", "p1_replay", "p1_incremental", "p2_transport"]:
        check(any(t["name"] == target and "test" in t["kind"]
                  for p in packages.values() for t in p["targets"]), f"{target} test target missing")
    for name in ['p1_walkthrough', 'p1_baseline']:
        check(any(t['name'] == name and 'example' in t['kind']
                  for t in packages['wyc-lib']['targets']), f'{name} executable missing')
    edges = {name: {d["name"] for d in p["dependencies"] if d["name"] in packages}
             for name, p in packages.items()}
    check(edges["contracts"] == set(), "contracts must not depend on orchestration")
    check(edges["indicators"] == {"contracts"}, "indicators must depend only on contracts")
    check(edges["market_structure"] == {"contracts"}, "structure must depend only on contracts")
    check(edges["wyc-lib"] == {"contracts", "indicators", "market_structure"}, "unexpected orchestration dependency")
    check(edges["wyc-binding-contract"] == {"wyc-lib"}, "FFI must call only the public Core API")
    check(any(t['name'] == 'p2_reference' and 'bin' in t['kind']
              for t in packages['wyc-binding-contract']['targets']), 'Rust FFI reference executable missing')
    check(any(t['name'] == 'p2_parity_reference' and 'bin' in t['kind']
              for t in packages['wyc-binding-contract']['targets']), 'Direct Core parity executable missing')
    active, visited = set(), set()

    def visit(name):
        check(name not in active, f"workspace dependency cycle at {name}")
        if name in visited:
            return
        active.add(name)
        for dependency in edges[name]:
            visit(dependency)
        active.remove(name)
        visited.add(name)

    for name in packages:
        visit(name)
    files = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard",
                            "-z", "--", "*.md"], cwd=root,
                           check=True, stdout=subprocess.PIPE).stdout.decode("utf-8").split("\0")
    checked = 0
    for filename in filter(None, files):
        path = root / filename
        for target in re.findall(r"\[[^\]]*\]\(([^)]+)\)", path.read_text(encoding="utf-8")):
            if "://" in target or target.startswith("#"):
                continue
            target = unquote(target.split("#", 1)[0])
            if target:
                check((path.parent / target).exists(), f"broken link: {filename} -> {target}")
                checked += 1
    print(f"PASS: P2 five package boundaries, acyclic dependency graph, ten integration targets, two examples, two Core references, {checked} local links")


if __name__ == "__main__":
    main()
