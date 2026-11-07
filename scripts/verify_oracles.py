"""Read-only, independent P0 oracle verification using the standard library.

No Rust import, no market feed, no expected-output generation or rewriting.
The ramp example uses exact fractions and an independent decimal square root.
"""
import hashlib
import json
import math
from decimal import Decimal, localcontext
from fractions import Fraction as F
from pathlib import Path

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


if __name__ == "__main__":
    verify_manifest()
    verify_temporal()
    verify_ramp()
    print("PASS: SHA-256 manifest, temporal hand oracle, Fraction/Decimal P1 ramp oracle")
