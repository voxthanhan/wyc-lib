"""P2.4 deterministic parity matrix and strict schema/numeric comparator.

References are direct Rust Core outputs, not independent numeric goldens.
P1 oracle/golden regression remains a separate mandatory CI gate.
"""
import copy
import hashlib
import json
import math
from collections import Counter

COLUMNS = ("timestamp", "available_at", "open", "high", "low", "close", "volume", "is_closed")
MODULES = (["indicators"], ["market_structure"], ["market_structure", "indicators"])
PERCENT = {"roc", "rsi", "plus_di", "minus_di", "dx", "adx"}


def compare(actual, expected, path=()):
    """Exact shape/order/discrete values; P1 tolerances only on quantitative leaves."""
    label = ".".join(map(str, path))
    if isinstance(expected, dict):
        assert isinstance(actual, dict) and actual.keys() == expected.keys(), (label, "keys")
        for key in expected:
            compare(actual[key], expected[key], path + (key,))
    elif isinstance(expected, list):
        assert isinstance(actual, list) and len(actual) == len(expected), (label, "shape")
        for i, item in enumerate(expected):
            compare(actual[i], item, path + (i,))
    elif isinstance(expected, (int, float)) and not isinstance(expected, bool):
        assert isinstance(actual, (int, float)) and not isinstance(actual, bool), (label, "number type")
        assert math.isfinite(actual) and math.isfinite(expected), (label, "nonfinite")
        quantitative = path[-1] in ("value", "strength") or "measured_values" in path
        exact = not quantitative or "min" in path or "max" in path
        if exact:
            assert actual == expected, (label, actual, expected)
        else:
            atol = 1e-10 if any(key in PERCENT for key in path) else 1e-12
            rtol = 1e-11 if "variance" in path else 1e-12
            assert abs(actual - expected) <= atol + rtol * max(abs(actual), abs(expected)), (label, actual, expected)
    else:
        assert type(actual) is type(expected) and actual == expected, (label, actual, expected)


def matrix(root):
    def read(name):
        return json.loads((root / name).read_text(encoding="utf-8"))

    manifest = read("tests/fixtures/manifest.json")
    fingerprints = {}
    for fixture in manifest["fixtures"]:
        for item in fixture["files"]:
            content = (root / item["path"]).read_bytes().replace(b"\r\n", b"\n")
            digest = hashlib.sha256(content).hexdigest()
            assert digest == item["sha256"], item["path"]
            fingerprints[item["path"]] = digest
    template = read("tests/fixtures/p1-arithmetic.json")
    config = read("tests/golden/p1-arithmetic.json")["config"]

    def request(data, cfg=None, as_of=None):
        return dict(schema_version="0.1.0", data=copy.deepcopy(data), config=copy.deepcopy(cfg or config),
                    modules=list(MODULES[2]), as_of=as_of if as_of is not None else max(data["available_at"], default=0))

    cases = [("arithmetic", request(template)), ("temporal", request(read("tests/fixtures/temporal.json"), as_of=9000))]
    for family in ("rolling", "basic", "advanced"):
        for case in read(f"tests/fixtures/p1-{family}.json")["cases"]:
            data, cfg = copy.deepcopy(template), copy.deepcopy(config)
            # Quantitative fixture columns use the shared f64 contract. Python
            # arbitrary-precision int rejection is covered by host boundary tests.
            values = [float(v) for v in case.get("close", case.get("values"))]
            n = len(values)
            for key in ("timestamp", "available_at"):
                data[key] = [(i + 1) * 1000 for i in range(n)]
            data.update(open=values, close=values, high=[float(v) for v in case.get("high", values)],
                        low=[float(v) for v in case.get("low", values)],
                        volume=[float(v) for v in case.get("volume", [10] * n)], is_closed=[True] * n)
            data["symbol"] = f"P2:{family}:{case['id']}"
            cfg["indicators"].update(period=case["period"], macd_fast=case.get("fast", 1),
                                      macd_slow=case.get("slow", 2), macd_signal=case.get("signal", 2),
                                      bollinger_multiplier=case.get("multiplier", 2))
            cases.append((f"{family}/{case['id']}", request(data, cfg)))
    structures = []
    for case in read("tests/fixtures/p1-structure.json")["cases"]:
        item = (f"structure/{case['id']}", request(case["data"], case["config"], case["as_of"]))
        structures.append(item)
        cases.append(item)

    actions, equalities, expected_errors = [], [], {}

    def add(name, op, payload=None, **extra):
        action = dict(name=name, op=op, **extra)
        if payload is not None:
            action["input"] = copy.deepcopy(payload)
        actions.append(action)
        return name

    def prefix(full, n, as_of):
        r = copy.deepcopy(full)
        r["as_of"] = as_of
        for key in COLUMNS:
            r["data"][key] = r["data"][key][:n]
        return r

    def eligible(full):
        d = full["data"]
        return sum(closed and t <= full["as_of"] and a <= full["as_of"]
                   for t, a, closed in zip(d["timestamp"], d["available_at"], d["is_closed"]))

    def update(full, i, as_of):
        return dict(metadata=full["data"]["metadata"], identity={}, as_of=as_of,
                    bar={key: full["data"][key][i] for key in COLUMNS})

    for name, full in cases:
        for op in ("batch", "indicators", "structure"):
            add(f"{name}/{op}", op, full)
        # Every prefix for every numeric/structure fixture. Structure additionally
        # covers all module selections and grouped availability, matching G1 replay.
        selections = MODULES if name.startswith("structure/") else [MODULES[2]]
        schedules = (False, True) if name.startswith("structure/") else (False,)
        for grouped in schedules:
            for selection in selections:
                r = prefix(full, eligible(full), full["as_of"])
                r["modules"] = selection
                d, n = r["data"], len(r["data"]["timestamp"])
                if grouped:
                    base_availability = max(d["available_at"][-1], d["timestamp"][-1]) if n else 0
                    for i in range(n):
                        d["available_at"][i] = base_availability + (i // 2 + 1) * 1000
                label = f"{name}/{'grouped' if grouped else 'raw'}/{'-'.join(selection)}"
                initial_time = d["metadata"]["revision_available_at"]
                initial = prefix(r, 0, initial_time)
                a = add(label + "/new", "new", initial)
                b = add(label + "/empty", "batch", initial)
                equalities.append((a, b))
                for i in range(n):
                    as_of = max(d["available_at"][i], initial_time)
                    a = add(f"{label}/update-{i}", "update", update(r, i, as_of))
                    b = add(f"{label}/batch-{i}", "batch", prefix(r, i + 1, as_of))
                    equalities.append((a, b))
                    if i + 1 == n or d["available_at"][i + 1] > as_of:
                        future = copy.deepcopy(r)
                        future["as_of"] = as_of
                        f = add(f"{label}/future-{i}", "batch", future)
                        equalities.append((b, f))
                add(label + "/identity", "identity")
        # Revised history must reset, then continue from half a snapshot.
        r = prefix(full, eligible(full), full["as_of"] + 100_000)
        d, n = r["data"], eligible(full)
        d["metadata"].update(data_revision=d["metadata"]["data_revision"] + "-p2-r2",
                             snapshot_kind="revised_history", revision_available_at=r["as_of"])
        for key in ("open", "high", "low", "close"):
            d[key] = [v * 2 for v in d[key]]
        half = prefix(r, n // 2, r["as_of"])
        a = add(name + "/revision-reset", "reset", half)
        b = add(name + "/revision-batch", "batch", half)
        equalities.append((a, b))
        for i in range(n // 2, n):
            a = add(f"{name}/revision-update-{i}", "update", update(r, i, r["as_of"]))
            b = add(f"{name}/revision-prefix-{i}", "batch", prefix(r, i + 1, r["as_of"]))
            equalities.append((a, b))

    # Domain failures are well-typed DTOs so all three runtimes reach Core.
    full = request(template)
    defects = [
        ("shape", ("data", "close"), [], "length_mismatch"),
        ("missing", ("data", "close", 0), None, "missing_data"),
        ("negative-volume", ("data", "volume", 0), -1, "negative_volume"),
        ("duplicate-time", ("data", "timestamp", 1), 1000, "duplicate_timestamp"),
        ("unordered-time", ("data", "timestamp", 1), 0, "unordered_timestamp"),
        ("availability", ("data", "available_at", 0), 999, "invalid_availability"),
        ("ohlc", ("data", "high", 0), -1, "invalid_ohlc"),
        ("period", ("config", "indicators", "period"), 0, "invalid_config"),
        ("metadata", ("data", "symbol"), "", "invalid_metadata"),
        ("version", ("schema_version",), "99", "unsupported_version"),
        ("duplicate-module", ("modules",), ["indicators", "indicators"], "duplicate_module"),
        ("unsupported", ("modules",), ["price_zones"], "unsupported_module"),
        ("revision-unavailable", ("data", "metadata", "revision_available_at"), 7000, "revision_not_available"),
        ("gaps", ("data", "metadata", "gaps"), [dict(after_timestamp=1000, before_timestamp=3000)], "declared_gap"),
    ]
    for name, path, value, code in defects:
        bad = copy.deepcopy(full)
        target = bad
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        for op in ("batch", "new", "reset"):
            if op != "batch":
                add(f"error/{name}/{op}/seed", "new", prefix(full, 3, 3000))
            key = add(f"error/{name}/{op}", op, bad)
            expected_errors[key] = code
            if op == "reset":
                a = add(key + "/atomic-result", "result")
                b = add(key + "/atomic-batch", "batch", prefix(full, 3, 3000))
                equalities.append((a, b))
                add(key + "/atomic-identity", "identity")

    overflow = copy.deepcopy(full)
    for column in ("open", "high", "close"):
        overflow["data"][column][0] = 1e308
    for op in ("batch", "new", "reset"):
        add(f"error/overflow/{op}/seed", "new", prefix(full, 3, 3000))
        expected_errors[add(f"error/overflow/{op}", op, overflow)] = "numeric_overflow"
        if op == "reset":
            a = add("error/overflow/reset/atomic-result", "result")
            b = add("error/overflow/reset/atomic-batch", "batch", prefix(full, 3, 3000))
            equalities.append((a, b))

    add("state/seed", "new", prefix(full, 3, 3000))
    good = update(full, 3, 4000)
    state_defects = [("symbol", {"symbol": "wrong"}), ("revision", {"data_revision": "wrong"}),
                     ("config", {"config_hash": "wrong"}), ("cursor", {"last_timestamp": 2000}),
                     ("modules", {"modules": ["indicators"]})]
    for name, patch in state_defects:
        key = add("state/" + name, "update", good, identity_patch=patch)
        expected_errors[key] = "state_mismatch"
    for name, change in [("duplicate", {"timestamp": 3000}), ("open", {"is_closed": False}),
                         ("unavailable", {"available_at": 5000}), ("missing", {"close": None})]:
        bad = copy.deepcopy(good)
        bad["bar"].update(change)
        key = add("state/" + name, "update", bad)
        expected_errors[key] = None
    bad = copy.deepcopy(good)
    bad["metadata"]["gaps"] = [dict(after_timestamp=3000, before_timestamp=4000)]
    expected_errors[add("state/gap", "update", bad)] = "reset_required"
    bad = copy.deepcopy(good)
    bad["metadata"]["source"] = "different"
    expected_errors[add("state/metadata", "update", bad)] = "state_mismatch"
    bad = copy.deepcopy(good)
    bad["bar"].update(open=1e308, high=1e308, close=1e308)
    expected_errors[add("state/overflow", "update", bad)] = "numeric_overflow"
    a = add("state/atomic-result", "result")
    b = add("state/atomic-batch", "batch", prefix(full, 3, 3000))
    equalities.append((a, b))
    add("state/atomic-identity", "identity")
    a = add("state/recovery", "update", good)
    b = add("state/recovery-batch", "batch", prefix(full, 4, 4000))
    equalities.append((a, b))
    # Reset also changes config, modules and stream identity, then goes backward.
    changed = copy.deepcopy(full)
    changed["config"]["indicators"]["period"] = 1
    changed["modules"] = ["market_structure"]
    changed["data"]["symbol"] = "Mã:Việt Nam"
    a = add("state/reconfigure", "reset", changed)
    b = add("state/reconfigure-batch", "batch", changed)
    equalities.append((a, b))
    a = add("state/reset-past", "reset", prefix(full, 1, 1000))
    b = add("state/reset-past-batch", "batch", prefix(full, 1, 1000))
    equalities.append((a, b))
    near_max = copy.deepcopy(full)
    for column in ("timestamp", "available_at"):
        near_max["data"][column] = [9007199254740991 - 6000 + t for t in full["data"][column]]
    near_max["as_of"] = 9007199254740991
    add("timestamp/max-safe", "batch", near_max)
    for as_of in (2000, 2500, 3499, 3500, 9000):
        temporal = request(read("tests/fixtures/temporal.json"), as_of=as_of)
        add(f"availability/{as_of}", "batch", temporal)
    return dict(actions=actions, equalities=equalities, expected_errors=expected_errors,
                fixture_cases=len(cases), fixture_sha256=fingerprints)


def verify(records, references, bundle):
    names = [a["name"] for a in bundle["actions"]]
    assert [r["name"] for r in records] == names, "runtime action coverage/order"
    assert [r["name"] for r in references] == names, "Rust action coverage/order"
    for actual, expected in zip(records, references):
        actual = copy.deepcopy(actual)
        kind = actual.pop("exception_kind", None)
        if "error" in expected:
            code = expected["error"]["code"]
            wanted = "type" if code == "missing_data" else (
                "state" if code in ("state_mismatch", "reset_required") else (
                    "engine" if code in ("numeric_overflow", "invalid_result") else "range"))
            assert kind == wanted, (actual["name"], kind, wanted)
        else:
            assert kind is None, actual["name"]
        compare(actual, expected, (actual["name"],))
    return audit(records, bundle)


def audit(records, bundle):
    """Check replay/atomicity and hand-authored invariants also in Core-only runs."""
    assert [r["name"] for r in records] == [a["name"] for a in bundle["actions"]]
    mapped = {r["name"]: r for r in records}
    for name, code in bundle["expected_errors"].items():
        assert "error" in mapped[name], (name, "expected controlled failure")
        if code is not None:
            assert mapped[name]["error"]["code"] == code, (name, mapped[name])
    for left, right in bundle["equalities"]:
        assert "value" in mapped[left] and "value" in mapped[right], (left, right, "unexpected error")
        compare(mapped[left]["value"], mapped[right]["value"], (left, right))
    # Independent hand-authored temporal truth table: open/future bars excluded.
    for as_of, count in ((2000, 1), (2500, 2), (3499, 2), (3500, 3), (9000, 3)):
        value = mapped[f"availability/{as_of}"]["value"]
        assert len(value["indicators"]["series"]["sma"]) == count
        assert all(e["known_at"] <= as_of for e in value["structure"]["evidence"])
    errors = Counter(r["error"]["code"] for r in records if "error" in r)
    assert errors, "error coverage missing"
    return dict(passed=True, actions=len(records), fixture_cases=bundle["fixture_cases"],
                equivalences=len(bundle["equalities"]), errors=dict(sorted(errors.items())),
                operations=dict(Counter(a["op"] for a in bundle["actions"])),
                fixture_sha256=bundle["fixture_sha256"], reference="direct-public-Rust-Core",
                numeric_policy="docs/algorithms/P1.md; discrete fields and min/max exact")


if __name__ == "__main__":
    import argparse
    import subprocess
    from pathlib import Path
    parser = argparse.ArgumentParser(description="Audit direct Core parity matrix without native consumers")
    parser.add_argument("--core-reference", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, default=Path("ci-evidence/p2-parity-core.json"))
    options = parser.parse_args()
    bundle = matrix(Path(__file__).resolve().parents[1])
    payload = json.dumps(bundle["actions"], ensure_ascii=False, allow_nan=False)
    result = subprocess.run([str(options.core_reference.resolve())], input=payload, text=True,
                            encoding="utf-8", stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
    report = audit(json.loads(result.stdout), bundle)
    report.update(runtimes=["Rust"], actions_sha256=hashlib.sha256(payload.encode()).hexdigest())
    options.evidence.parent.mkdir(parents=True, exist_ok=True)
    options.evidence.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False))
