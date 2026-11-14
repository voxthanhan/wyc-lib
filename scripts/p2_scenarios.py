"""Contract smoke actions; arithmetic expectations come from immutable P1 golden."""
import copy
import json
import math

COLUMNS = ("timestamp", "available_at", "open", "high", "low", "close", "volume", "is_closed")


def scenarios(root):
    data = json.loads((root / "tests/fixtures/p1-arithmetic.json").read_text(encoding="utf-8"))
    golden = json.loads((root / "tests/golden/p1-arithmetic.json").read_text(encoding="utf-8"))
    full = dict(schema_version="0.1.0", data=data, config=golden["config"],
                modules=["indicators", "market_structure"], as_of=6000)

    def prefix(n):
        r = copy.deepcopy(full)
        for key in COLUMNS:
            r["data"][key] = r["data"][key][:n]
        r["as_of"] = n * 1000
        return r

    actions, expected_requests, errors = [], {}, {}

    def add(name, op, payload=None, expected=None, error=None, **extra):
        action = dict(name=name, op=op, **extra)
        if payload is not None:
            action["input"] = json.dumps(payload, ensure_ascii=False) if isinstance(payload, dict) else payload
        actions.append(action)
        if expected is not None:
            expected_requests[name] = expected
        if error:
            errors[name] = dict(zip(("kind", "code", "path"), error))

    add("batch-full", "analyze", full, full)
    add("batch-empty", "analyze", prefix(0), prefix(0))
    unicode_request = copy.deepcopy(full)
    unicode_request["data"]["symbol"] = "Mã:Việt Nam"
    add("unicode", "analyze", unicode_request, unicode_request)
    zero = copy.deepcopy(full)
    zero["data"]["volume"] = [0] * 6
    add("zero-volume", "analyze", zero, zero)
    for name, payload, code in [("syntax", "{", "invalid_json"),
                                ("nonfinite", "NaN", "invalid_json"),
                                ("overflow-token", "1e999", "invalid_json"),
                                ("trailing", "{} {}", "invalid_json"),
                                ("shape", "{}", "invalid_type")]:
        add(name, "analyze", payload, error=("type", code, "request"))
    add("host-type", "analyze", 123, error=("type", "invalid_type", "request"))
    for name, path, value, err in [
        ("timestamp-range", ("as_of",), 9007199254740992, ("range", "invalid_timestamp", "as_of")),
        ("fractional-time", ("as_of",), 6000.5, ("type", "invalid_type", "request")),
        ("bool-time", ("as_of",), True, ("type", "invalid_type", "request")),
        ("unknown-field", ("unexpected",), 1, ("type", "invalid_type", "request")),
        ("missing-value", ("data", "close", 0), None, ("type", "missing_data", "close[0]")),
        ("period-range", ("config", "indicators", "period"), 0,
         ("range", "invalid_config", "config.indicators.period")),
    ]:
        r = copy.deepcopy(full)
        location = r
        for key in path[:-1]:
            location = location[key]
        location[path[-1]] = value
        add(name, "analyze", r, error=err)
    add("new", "new", prefix(3))
    add("initial-result", "result", expected=prefix(3))
    add("initial-identity", "identity")
    add("bad-update", "update", "{", error=("type", "invalid_json", "update"))
    add("after-bad-update", "result", expected=prefix(3))
    invalid = copy.deepcopy(full)
    invalid["config"]["indicators"]["period"] = 0
    add("bad-reset", "reset", invalid, error=("range", "invalid_config", "config.indicators.period"))
    add("after-bad-reset", "result", expected=prefix(3))
    for i in range(3, 6):
        bar = {k: data[k][i] for k in COLUMNS}
        update = dict(identity={}, metadata=data["metadata"], bar=bar, as_of=(i + 1) * 1000)
        if i == 3:
            add("bad-cursor", "update", update, error=("type", "invalid_type", "update"))
            add("wrong-identity", "update", update, echo=True, wrong_identity=True,
                error=("state", "state_mismatch", "state.identity"))
        add(f"append-{i}", "update", update, prefix(i + 1), echo=True)
    add("owned-result", "mutate", expected=full)
    add("independent-new", "new", prefix(3), id="b")
    add("independent-result", "result", expected=prefix(3), id="b")
    revised = copy.deepcopy(full)
    revised["data"]["metadata"].update(data_revision="ramp-r2", snapshot_kind="revised_history",
                                       revision_available_at=6000)
    revised["data"]["close"][-1] = 5
    add("revision-reset", "reset", revised, revised)
    add("old-handle-unchanged", "result", expected=prefix(3), id="b")
    add("close", "close")
    add("close-again", "close")
    for op in ("result", "identity", "update", "reset"):
        add(f"closed-{op}", op, "bad" if op in ("update", "reset") else None,
            error=("state", "state_closed", "state"))
    add("gc", "gc", prefix(3))
    return actions, expected_requests, errors, golden


def verify(records, references, errors, golden):
    by_name = {r["name"]: r for r in records}
    for name, expected in errors.items():
        assert by_name[name].get("error") == expected, (name, by_name[name], expected)
    for name, record in by_name.items():
        if name not in errors:
            assert "error" not in record, (name, record)
    for name, expected in references.items():
        assert by_name[name]["value"] == expected, f"Rust transport parity failed: {name}"
    output = by_name["batch-full"]["value"]
    series = output["indicators"]["series"]
    assert len(series) == 25
    for i in (2, 3, 5):
        for name, value in golden[f"scalars_at_index_{i}"].items():
            actual = series[name][i]["result"]
            assert actual["status"] == "ready"
            assert math.isclose(actual["value"], value, rel_tol=1e-12, abs_tol=1e-12), (name, i)
    assert series["sma"][0]["result"] == dict(status="warmup", required=3, available=1)
    assert series["macd_histogram"][3]["result"] == dict(status="ready", value=0)
    missing = by_name["zero-volume"]["value"]["indicators"]["series"]["rvol"][2]["result"]
    assert missing == dict(status="missing", reason="zero_denominator")
    assert output["metadata"]["as_of"] == 6000
    assert series["sma"][2]["observed_at"] == series["sma"][2]["known_at"] == 3000
    assert by_name["initial-identity"]["value"]["last_timestamp"] == 3000
    assert by_name["gc"]["value"] == "gc-no-crash"
    return dict(actions=len(records), rust_references=len(references), error_cases=len(errors),
                independent_golden="p1-arithmetic-v1", tolerance=dict(atol=1e-12, rtol=1e-12),
                gc="100 unclosed handles reclaimed without crash; leak/RSS not measured")
