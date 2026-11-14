"""Installed-wheel P2.4 driver; calculations and state remain in Rust."""
import copy
import json
import sys
from pathlib import Path
import wyc_lib as api

actions = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
state, records = None, []
for action in actions:
    before = copy.deepcopy(action.get("input"))
    try:
        op, request = action["op"], action.get("input")
        if op == "batch":
            value = api.analyze_batch(request)
        elif op == "indicators":
            value = api.calculate_indicators(request["data"], request["config"], request["as_of"])
        elif op == "structure":
            value = api.analyze_structure(request["data"], request["config"], request["as_of"])
        elif op == "new":
            if state is not None:
                state.close()
            state = api.AnalysisState(request)
            value = state.result()
        elif op == "reset":
            value = state.reset(request)
        elif op == "update":
            update = copy.deepcopy(request)
            update["identity"] = {**state.identity(), **action.get("identity_patch", {})}
            value = state.update_closed_bar(update)
        elif op == "result":
            value = state.result()
        elif op == "identity":
            value = state.identity()
        else:
            raise AssertionError("unknown parity operation")
        records.append(dict(name=action["name"], value=value))
    except Exception as error:
        kind = "state" if isinstance(error, api.WycStateError) else (
            "engine" if isinstance(error, api.WycEngineError) else (
                "type" if isinstance(error, TypeError) else (
                    "range" if isinstance(error, ValueError) else "unexpected")))
        records.append(dict(name=action["name"], error=dict(code=getattr(error, "code", None),
                       path=getattr(error, "path", None)), exception_kind=kind))
    assert action.get("input") == before, f"input changed: {action['name']}"
if state is not None:
    state.close()
print(json.dumps(records, ensure_ascii=False, allow_nan=False))
