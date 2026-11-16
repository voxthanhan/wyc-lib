"""Measure installed owned-copy APIs; no analytical formulas."""
import copy
import json
import sys
import time
from pathlib import Path
import numpy as np
import pandas as pd
import polars as pl
import wyc_lib as lib
from wyc_lib import _native, _conversion
from p2_process import worker_metrics


requests = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))
rows, outputs = [], []


def measure(bars, operation, call):
    for _ in range(2):
        call()
    samples = []
    for _ in range(5):
        start = time.perf_counter_ns()
        for _ in range(3):
            call()
        samples.append((time.perf_counter_ns() - start) / 3)
    rows.append(dict(bars=bars, operation=operation, ns_per_op=samples))


for r in requests:
    before = json.dumps(r, sort_keys=True)
    bars = len(r['data']['timestamp'])
    encoded = _conversion.encode(r, 'request')
    response = _native.analyze_json(encoded)
    if not json.loads(response)['ok']:
        raise RuntimeError('native baseline failed')
    typed = copy.deepcopy(r)
    columns = ['timestamp', 'available_at', 'open', 'high', 'low', 'close', 'volume', 'is_closed']
    for key in columns:
        dtype = np.int64 if key in ('timestamp', 'available_at') else np.bool_ if key == 'is_closed' else np.float64
        typed['data'][key] = np.array(r['data'][key], dtype=dtype)
    frames = {kind: constructor({k: r['data'][k] for k in columns})
              for kind, constructor in [('pandas', pd.DataFrame), ('polars', pl.DataFrame)]}
    kwargs = {k: r['data'][k] for k in ('symbol', 'timeframe', 'metadata', 'schema_version')}
    converted = {}
    for kind, frame in frames.items():
        convert = getattr(lib, 'market_data_from_' + kind)
        converted[kind] = convert(frame, **kwargs)
        measure(bars, kind + '_conversion', lambda: convert(frame, **kwargs))
    outputs.append(dict(batch=lib.analyze_batch(r),
        indicators=lib.calculate_indicators(r['data'], r['config'], r['as_of']),
        structure=lib.analyze_structure(r['data'], r['config'], r['as_of']),
        raw=json.loads(response)['value'], array_batch=lib.analyze_batch(typed), converted=converted))
    measure(bars, 'deepcopy_proxy', lambda: copy.deepcopy(r))
    measure(bars, 'host_snapshot_encode', lambda: _conversion.encode(r, 'request'))
    measure(bars, 'output_decode', lambda: json.loads(response))
    measure(bars, 'native_json_transport', lambda: _native.analyze_json(encoded))
    measure(bars, 'typed_batch', lambda: lib.analyze_batch(r))
    measure(bars, 'typed_two_calls', lambda: (
        lib.calculate_indicators(r['data'], r['config'], r['as_of']),
        lib.analyze_structure(r['data'], r['config'], r['as_of'])))
    measure(bars, 'array_batch', lambda: lib.analyze_batch(typed))
    if before != json.dumps(r, sort_keys=True):
        raise RuntimeError('input mutated')
    for key in columns:
        if typed['data'][key].tolist() != r['data'][key]:
            raise RuntimeError('NumPy input mutated')
print(json.dumps(dict(runtime='python', version=sys.version, rows=rows, outputs=outputs,
    warmups=2, samples=5, iterations=3, profile='release',
    worker_process=worker_metrics(),
    extras=dict(numpy=np.__version__, pandas=pd.__version__, polars=pl.__version__)), allow_nan=False))
