"""Read-only, independent P0/P1 oracle verification using the standard library.

No Rust import, no market feed, no expected-output generation or rewriting.
The ramp example uses exact fractions and an independent decimal square root.
"""
import hashlib
import json
import math
import struct
from decimal import Decimal, localcontext
from fractions import Fraction as F
from pathlib import Path
from basic_oracle import evaluate as evaluate_basic
from advanced_oracle import evaluate as evaluate_advanced

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def verify_manifest():
    manifest = read("tests/fixtures/manifest.json")
    for fixture in manifest["fixtures"]:
        for item in fixture["files"]:
            actual = hashlib.sha256((ROOT / item["path"]).read_bytes()).hexdigest()
            check(actual == item["sha256"], f"checksum: {item['path']}")


def verify_temporal():
    data = read("tests/fixtures/temporal.json")
    golden = read("tests/golden/temporal.json")
    actual = [sum(closed and t <= as_of and available <= as_of
                  for t, available, closed in zip(data["timestamp"],
                                                  data["available_at"],
                                                  data["is_closed"]))
              for as_of in golden["as_of"]]
    check(actual == golden["available_lengths"], "temporal availability")
    pivot = golden["delayed_pivot"]
    check(data["high"][1] > max(data["high"][0], data["high"][2]), "strict pivot")
    check(pivot["observed_at"] == data["timestamp"][1], "pivot observed_at")
    check(pivot["known_at"] == data["available_at"][2] == 3500, "pivot delay")


def verify_ramp():
    data = read("tests/fixtures/p1-arithmetic.json")
    golden = read("tests/golden/p1-arithmetic.json")
    x = list(map(F, data["close"]))
    check(x == list(map(F, range(1, 7))), "ramp provenance")
    check(data["open"] == data["close"], "ramp open")
    check(data["high"] == [int(y + 1) for y in x], "ramp high")
    check(data["low"] == [int(y - 1) for y in x], "ramp low")
    check(data["volume"] == list(range(0, 60, 10)), "ramp volume")
    n = golden["config"]["indicators"]["period"]
    check(n == 3, "oracle example period")
    check(golden["config"]["indicators"]["macd_fast"] == 2, "fast")
    check(golden["config"]["indicators"]["macd_slow"] == 3, "slow")
    check(golden["config"]["indicators"]["macd_signal"] == 2, "signal")
    check(golden["config"]["indicators"]["bollinger_multiplier"] == 2, "bands")
    mean = sum(x[:n]) / n
    variance = sum((y - mean) ** 2 for y in x[:n]) / n
    with localcontext() as ctx:
        ctx.prec = 50
        std = (Decimal(variance.numerator) / Decimal(variance.denominator)).sqrt()
        upper, lower = Decimal(2) + 2 * std, Decimal(2) - 2 * std
    # EMA_2 seed=1.5, next=2.5, EMA_3 seed=2: line=.5.
    fast_seed = (x[0] + x[1]) / 2
    fast_at_2 = fast_seed + F(2, 3) * (x[2] - fast_seed)
    line = fast_at_2 - mean
    at_2 = {
        "sum": sum(x[:n]), "sma": mean, "ema": mean,
        "wma": sum(F(i + 1) * y for i, y in enumerate(x[:n])) / 6,
        "min": min(x[:n]), "max": max(x[:n]), "variance": variance,
        "std": std, "tr": F(2), "atr": F(2), "volume_sma": F(10),
        "rvol": F(20, 10), "macd_line": line,
        "bollinger_upper": upper, "bollinger_lower": lower,
    }
    # All transitions have gain=1, loss=0, +DM=1, -DM=0, TR=2.
    plus_di, minus_di = 100 * F(1, 2), F(0)
    dx = 100 * abs(plus_di - minus_di) / (plus_di + minus_di)
    at_3 = {
        "sma": sum(x[1:4]) / 3, "ema": mean + F(1, 2) * (x[3] - mean),
        "momentum": x[3] - x[0], "roc": 100 * (x[3] / x[0] - 1),
        "rsi": F(100), "macd_line": line, "macd_signal": (line + line) / 2,
        "macd_histogram": F(0), "plus_di": plus_di, "minus_di": minus_di,
        "dx": dx, "rvol": F(30, 20),
    }
    expected_first = {"sma": n-1, "ema": n-1, "wma": n-1, "atr": n-1,
                      "rsi": n, "macd_line": 2, "macd_signal": 3,
                      "adx": 2*n-1, "roc": n}
    check(golden["first_valid_index"] == expected_first, "first-valid masks")
    for group, independent in [("scalars_at_index_2", at_2),
                               ("scalars_at_index_3", at_3),
                               ("scalars_at_index_5", {"adx": (dx+dx+dx)/3})]:
        check(golden[group].keys() == independent.keys(), f"{group} schema")
        for name, expected in independent.items():
            category = ("variance" if name == "variance" else
                        "percent" if name in {"roc", "rsi", "plus_di", "minus_di", "dx", "adx"}
                        else "ratio" if name == "rvol" else "price")
            budget = golden["tolerances"][category]
            actual, expected = golden[group][name], float(expected)
            check(math.isfinite(actual) and
                  abs(actual-expected) <= budget["atol"] + budget["rtol"] * max(abs(actual), abs(expected)),
                  f"{group}.{name}: {actual} != independent {expected}")


def verify_identity():
    data = read("tests/fixtures/p1-identity.json")
    golden = read("tests/golden/p1-identity.json")
    config = data["config"]
    indicators = config["indicators"]
    # IEEE-754 big-endian bytes give the same 16 hex digits as the specified bits.
    bits = struct.pack(">d", indicators["bollinger_multiplier"]).hex()
    canonical = "\n".join([
        f"config_version={config['config_version']}",
        *(f"{key}={indicators[key]}" for key in
          ["period", "macd_fast", "macd_slow", "macd_signal"]),
        f"bollinger_multiplier_bits={bits}",
        f"left={config['structure']['left']}",
        f"right={config['structure']['right']}", ""])
    check(canonical == golden["canonical_config"], "canonical config bytes")
    config_hash = "sha256:config-v1:" + hashlib.sha256(canonical.encode("ascii")).hexdigest()
    check(config_hash == golden["config_hash"], "config hash")
    check(len(data["events"]) == len(golden["event_preimages"]) == len(golden["event_ids"]) == 4,
          "identity vector count")
    for i, event in enumerate(data["events"]):
        fields = [event[k] for k in ["engine_version", "algorithm_version", "symbol",
                                    "timeframe", "data_revision"]] + [config_hash]
        fields += [event[k] for k in ["source_module", "rule_id", "event_type",
                                     "observed_at", "known_at", "ordinal"]]
        preimage = b"".join(str(len(str(x).encode("utf-8"))).encode("ascii") + b":" +
                            str(x).encode("utf-8") for x in fields)
        check(preimage == golden["event_preimages"][i].encode("utf-8"), f"event preimage {i}")
        event_id = "sha256:event-v1:" + hashlib.sha256(preimage).hexdigest()
        check(event_id == golden["event_ids"][i], f"event ID {i}")
    check(golden["event_ids"][2] != golden["event_ids"][3], "length-prefix ambiguity")


def verify_rolling():
    data = read("tests/fixtures/p1-rolling.json")
    golden = read("tests/golden/p1-rolling.json")
    check(data["fixture_id"] == golden["fixture_id"] == "p1-rolling-v1", "rolling identity")
    check(data["algorithm_version"] == golden["algorithm_version"] == "p1-v1", "rolling version")
    check(len(data["cases"]) == len(golden["cases"]) == 8, "rolling case count")
    for case, expected in zip(data["cases"], golden["cases"]):
        check(case["id"] == expected["id"], "rolling case order")
        # Exact rational arithmetic is independent of compensated floating-point summation.
        values = [F(str(value)) for value in case["values"]]
        n = case["period"]
        check(1 <= n <= 1_000_000, "rolling period")
        names = ["sum", "mean", "min", "max", "variance", "std"]
        check(set(expected) == {"id", *names}, "rolling golden schema")
        check(all(len(expected[name]) == len(values) for name in names), "rolling output length")
        for i in range(len(values)):
            if i + 1 < n:
                check(all(expected[name][i] is None for name in names), "rolling warmup mask")
                continue
            window = values[i+1-n:i+1]
            mean = sum(window) / n
            variance = sum((value - mean) ** 2 for value in window) / n
            with localcontext() as ctx:
                ctx.prec = 80
                std = (Decimal(variance.numerator) / Decimal(variance.denominator)).sqrt()
            independent = {"sum": sum(window), "mean": mean, "min": min(window),
                           "max": max(window), "variance": variance, "std": std}
            for name, value in independent.items():
                actual, oracle = expected[name][i], float(value)
                check(actual is not None and math.isfinite(actual), "rolling finite golden")
                rtol = 1e-11 if name == "variance" else 1e-12
                atol = 0.0 if case["id"] == "tiny" else 1e-12
                equal = (actual == oracle if name in {"min", "max"} else
                         abs(actual-oracle) <= atol + rtol * max(abs(actual), abs(oracle)))
                check(equal, f"rolling {case['id']}.{name}[{i}]: {actual} != {oracle}")


def verify_basic():
    data = read("tests/fixtures/p1-basic.json")
    golden = read("tests/golden/p1-basic.json")
    check(data["fixture_id"] == golden["fixture_id"] == "p1-basic-v1", "basic identity")
    check(data["algorithm_version"] == golden["algorithm_version"] == "p1-v1", "basic version")
    check(len(data["cases"]) == len(golden["cases"]) == 6, "basic cases")
    for case, expected in zip(data["cases"], golden["cases"]):
        independent = evaluate_basic(case)
        check(independent.keys() == expected.keys(), "basic golden schema")
        check(independent["id"] == expected["id"], "basic case order")
        for name, values in independent.items():
            if name == "id":
                continue
            check(len(values) == len(expected[name]), "basic output length")
            for i, (oracle, actual) in enumerate(zip(values, expected[name])):
                if oracle is None or isinstance(oracle, str):
                    check(actual == oracle, f"basic {case['id']}.{name}[{i}] mask")
                else:
                    atol = 0.0 if case["id"] == "tiny" else 1e-10 if name == "roc" else 1e-12
                    check(isinstance(actual, (float, int)) and math.isfinite(actual) and
                          abs(actual-oracle) <= atol + 1e-12 * max(abs(actual), abs(oracle)),
                          f"basic {case['id']}.{name}[{i}] number")


def verify_advanced():
    data = read("tests/fixtures/p1-advanced.json")
    golden = read("tests/golden/p1-advanced.json")
    check(data["fixture_id"] == golden["fixture_id"] == "p1-advanced-v1", "advanced identity")
    check(data["algorithm_version"] == golden["algorithm_version"] == "p1-v1", "advanced version")
    check(len(data["cases"]) == len(golden["cases"]) == 8, "advanced cases")
    for case, expected in zip(data["cases"], golden["cases"]):
        independent = evaluate_advanced(case)
        check(independent.keys() == expected.keys(), "advanced schema")
        check(independent["id"] == expected["id"], "advanced case order")
        for name, values in independent.items():
            if name == "id":
                continue
            check(len(values) == len(expected[name]), "advanced length")
            for i, (oracle, actual) in enumerate(zip(values, expected[name])):
                if oracle is None:
                    check(actual is None, f"advanced {case['id']}.{name}[{i}] warmup")
                else:
                    percent = name in {"rsi", "plus_di", "minus_di", "dx", "adx"}
                    atol = 1e-10 if percent else 0.0 if case["id"] == "tiny" else 1e-12
                    check(isinstance(actual, (int, float)) and math.isfinite(actual) and
                          abs(actual-oracle) <= atol + 1e-12 * max(abs(actual), abs(oracle)),
                          f"advanced {case['id']}.{name}[{i}] number")


if __name__ == "__main__":
    verify_manifest()
    verify_temporal()
    verify_ramp()
    verify_identity()
    verify_rolling()
    verify_basic()
    verify_advanced()
    print("PASS: SHA-256 manifest, temporal, Fraction/Decimal ramp, 8 rolling cases, 6 basic cases, 8 advanced cases (independent closed-form smoothing/variance), config/event vectors")
