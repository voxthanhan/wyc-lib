"""Independent declarative P1.5 oracle; no Rust calls or file writes.

Enumerate windows first, then derive each historical trend from the complete
eligible pivot set. This differs from the engine's sequential references.
Strength uses exact Fraction arithmetic, IDs use standard-library hashlib.
"""
import hashlib
import struct
from fractions import Fraction


def config_hash(config):
    p = config["indicators"]
    s = config["structure"]
    bits = struct.pack(">d", p["bollinger_multiplier"]).hex()
    text = (f"config_version={config['config_version']}\nperiod={p['period']}\n"
            f"macd_fast={p['macd_fast']}\nmacd_slow={p['macd_slow']}\n"
            f"macd_signal={p['macd_signal']}\nbollinger_multiplier_bits={bits}\n"
            f"left={s['left']}\nright={s['right']}\n")
    return "sha256:config-v1:" + hashlib.sha256(text.encode()).hexdigest()


def evaluate(case):
    data, config = case["data"], case["config"]
    as_of = case["as_of"]
    revision = data["metadata"]["revision_available_at"]
    assert revision <= as_of
    n = sum(c and a <= as_of and t <= as_of for c, a, t in
            zip(data["is_closed"], data["available_at"], data["timestamp"]))
    left, right = config["structure"]["left"], config["structure"]["right"]
    metadata = dict(schema_version="0.1.0", engine_version="0.1.0",
                    algorithm_version="p1-v1", symbol=data["symbol"],
                    timeframe=data["timeframe"], data_revision=data["metadata"]["data_revision"],
                    config_hash=config_hash(config), as_of=as_of)

    def known(i):
        return max(revision, data["available_at"][i])

    def identity(rule, event, observed, available, ordinal):
        fields = [metadata[k] for k in ["engine_version", "algorithm_version", "symbol",
                  "timeframe", "data_revision", "config_hash"]]
        fields += ["market_structure", rule, event, str(observed), str(available), str(ordinal)]
        preimage = b"".join(str(len(f.encode())).encode() + b":" + f.encode() for f in fields)
        return "sha256:event-v1:" + hashlib.sha256(preimage).hexdigest()

    pivots = []
    for j in range(left, n-right):
        neighbors = list(range(j-left, j)) + list(range(j+1, j+right+1))
        for kind, column in [("high", "high"), ("low", "low")]:
            level = data[column][j]
            values = [data[column][k] for k in neighbors]
            if not (level > max(values) if kind == "high" else level < min(values)):
                continue
            previous = [p for p, _ in pivots if p["kind"] == kind]
            label = "unclassified"
            if previous:
                last = previous[-1]["level"]
                label = (("hh" if level > last else "lh" if level < last else "eh")
                         if kind == "high" else
                         ("hl" if level > last else "ll" if level < last else "el"))
            observed, available = data["timestamp"][j], known(j+right)
            event_id = identity("structure-pivot-strict-v1", "swing_"+kind,
                                observed, available, 0 if kind == "high" else 1)
            pivots.append((dict(pivot_id=event_id, kind=kind, label=label, level=level,
                                observed_at=observed, known_at=available), j+right))

    def trend_at(i):
        pairs = [[p for p, confirmation in pivots if confirmation <= i and p["kind"] == kind][-2:]
                 for kind in ["high", "low"]]
        if any(len(pair) < 2 for pair in pairs):
            return "unknown"
        if all(pair[1]["level"] > pair[0]["level"] for pair in pairs):
            return "uptrend"
        if all(pair[1]["level"] < pair[0]["level"] for pair in pairs):
            return "downtrend"
        return "sideways"

    trends = [dict(observed_at=data["timestamp"][i], known_at=known(i), trend=trend_at(i))
              for i in range(n)]
    breaks, evidence, used = [], [], set()
    for pivot, _ in pivots:
        evidence.append(dict(evidence_id=pivot["pivot_id"], event_type="swing_"+pivot["kind"],
                             observed_at=pivot["observed_at"], known_at=pivot["known_at"],
                             source_module="market_structure", measured_values={"level": pivot["level"]},
                             rule_id="structure-pivot-strict-v1", status="confirmed", layer="observation"))
    for i in range(n):
        trend = trend_at(i-1)
        if trend == "unknown":
            continue
        close = data["close"][i]
        references = [[p for p, confirmation in pivots if confirmation < i and p["kind"] == kind][-1:]
                      for kind in ["high", "low"]]
        triggers = [(pair[0], direction) for pair, direction in zip(references, ["up", "down"])
                    if pair and pair[0]["pivot_id"] not in used and
                    (close > pair[0]["level"] if direction == "up" else close < pair[0]["level"])]
        if len(triggers) > 1:
            raise ValueError(f"structure.breaks[{i}].references")
        for p, direction in triggers:
            kind = "choch" if (trend, direction) in [("downtrend", "up"), ("uptrend", "down")] else "bos"
            event = kind+"_"+direction
            observed, available = data["timestamp"][i], known(i)
            event_id = identity("structure-break-close-v1", event, observed, available, 0)
            spread = Fraction(str(data["high"][i]))-Fraction(str(data["low"][i]))
            strength = (dict(status="ready", value=float(abs(Fraction(str(close))-Fraction(str(p["level"]))) / spread))
                        if spread else dict(status="missing", reason="zero_denominator"))
            breaks.append(dict(event_id=event_id, reference_pivot_id=p["pivot_id"], kind=kind,
                               direction=direction, level=p["level"], close=close, strength=strength,
                               observed_at=observed, known_at=available))
            measured = dict(level=p["level"], close=close)
            if strength["status"] == "ready":
                measured["strength"] = strength["value"]
            evidence.append(dict(evidence_id=event_id, event_type=event, observed_at=observed,
                                 known_at=available, source_module="market_structure", measured_values=measured,
                                 rule_id="structure-break-close-v1", status="confirmed", layer="confirmation"))
            used.add(p["pivot_id"])
    evidence.sort(key=lambda e: tuple(e[k] for k in ["observed_at", "known_at", "source_module", "rule_id", "evidence_id"]))
    return dict(metadata=metadata, output=dict(pivots=[p for p, _ in pivots], trends=trends,
                                             breaks=breaks, evidence=evidence))
