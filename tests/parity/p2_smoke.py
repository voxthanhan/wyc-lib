"""Run with the consumer venv; only the installed wheel may provide wyc_lib."""
import gc
import json
import sys
from pathlib import Path
import wyc_lib as api

actions = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
states, records = {}, []
for action in actions:
    try:
        value = None
        state = states.get(action.get("id", "a"))
        op = action["op"]
        if op == "analyze":
            value = json.loads(api.analyze_json(action["input"]))
        elif op == "new":
            states[action.get("id", "a")] = api.State(action["input"])
        elif op == "result":
            value = json.loads(state.result_json())
        elif op == "identity":
            value = json.loads(state.identity_json())
        elif op == "reset":
            value = json.loads(state.reset_json(action["input"]))
        elif op == "update":
            payload = action["input"]
            if action.get("echo"):
                update = json.loads(payload)
                update["identity"] = json.loads(state.identity_json())
                if action.get("wrong_identity"):
                    update["identity"]["symbol"] = "different"
                payload = json.dumps(update)
            value = json.loads(state.update_json(payload))
        elif op == "mutate":
            output = json.loads(state.result_json())
            output["metadata"]["symbol"] = "caller-mutation"
            value = json.loads(state.result_json())
        elif op == "close":
            state.close()
        elif op == "gc":
            for _ in range(100):
                api.State(action["input"])
            gc.collect()
            value = "gc-no-crash"
        else:
            raise AssertionError("unknown smoke operation")
        records.append(dict(name=action["name"], value=value))
    except Exception as error:
        kind = "state" if isinstance(error, api.WycStateError) else (
            "engine" if isinstance(error, api.WycEngineError) else (
                "type" if isinstance(error, TypeError) else (
                    "range" if isinstance(error, ValueError) else "unexpected")))
        records.append(dict(name=action["name"], error=dict(
            kind=kind, code=getattr(error, "code", None), path=getattr(error, "path", None))))
print(json.dumps(records, ensure_ascii=False, allow_nan=False))
