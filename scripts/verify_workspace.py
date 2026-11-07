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
    check(set(packages) == {"contracts", "indicators", "wyc-lib"}, "P1.4 workspace membership changed")
    check(all(p["publish"] == [] for p in packages.values()), "packages must remain non-publishable")
    for target in ["foundation", "p1_foundation", "p1_rolling", "p1_basic", "p1_advanced"]:
        check(any(t["name"] == target and "test" in t["kind"]
                  for p in packages.values() for t in p["targets"]), f"{target} test target missing")
    edges = {name: {d["name"] for d in p["dependencies"] if d["name"] in packages}
             for name, p in packages.items()}
    check(edges["contracts"] == set(), "contracts must not depend on orchestration")
    check(edges["indicators"] == {"contracts"}, "indicators must depend only on contracts")
    check(edges["wyc-lib"] == {"contracts", "indicators"}, "unexpected orchestration dependency")
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
    print(f"PASS: P1.4 package boundaries, acyclic dependency graph, foundation/rolling/basic/advanced targets, {checked} local links")


if __name__ == "__main__":
    main()
