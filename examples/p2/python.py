"""Run with the interpreter from a fresh venv containing the built wheel."""
import json
from pathlib import Path
import wyc_lib as api

request = json.loads(Path(__file__).with_name('request.json').read_text(encoding='utf-8'))
with api.AnalysisState(request) as state:
    print(json.dumps(dict(
        batch=api.analyze_batch(request),
        indicators=api.calculate_indicators(request['data'], request['config'], request['as_of']),
        structure=api.analyze_structure(request['data'], request['config'], request['as_of']),
        state=state.result()), allow_nan=False))
