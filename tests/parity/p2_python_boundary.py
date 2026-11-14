"""Host conversion only. The injected native module never computes analysis."""
import importlib.util
import json
import sys
import types
from pathlib import Path

root = Path(__file__).resolve().parents[2]
sys.modules['wyc_lib._native'] = types.ModuleType('wyc_lib._native')
spec = importlib.util.spec_from_file_location('wyc_lib', root / 'bindings/python/python/wyc_lib/__init__.py')
api = importlib.util.module_from_spec(spec)
sys.modules['wyc_lib'] = api
spec.loader.exec_module(api)
from wyc_lib._conversion import snapshot
import numpy as np
import pandas as pd
import polars as pl

assertions = 0


def equal(actual, expected):
    global assertions
    assert actual == expected, (actual, expected)
    assertions += 1


def error(call, cls, code, path):
    global assertions
    try:
        call()
    except cls as exc:
        assert exc.code == code and exc.path == path, (exc.code, exc.path, code, path)
    else:
        raise AssertionError(f'Expected {cls.__name__} {code} at {path}')
    assertions += 1


def main():
    import copy
    base = json.loads((root / 'tests/fixtures/p1-arithmetic.json').read_text(encoding='utf-8'))
    columns = ('timestamp', 'available_at', 'open', 'high', 'low', 'close', 'volume', 'is_closed')
    dtypes = {k: np.bool_ if k == 'is_closed' else np.int64 if k in columns[:2] else np.float64 for k in columns}
    equal(snapshot(base, 'data'), base)
    arrays = copy.deepcopy(base)
    for k in columns:
        arrays[k] = np.array(base[k], dtype=dtypes[k])
        arrays[k].flags.writeable = False
    equal(snapshot(arrays, 'data'), base)
    for k in columns:
        strided = np.repeat(arrays[k], 2)[::2]
        data = copy.deepcopy(base)
        data[k] = strided
        equal(snapshot(data, 'data'), base)
    for k in ('open', 'high', 'low', 'close', 'volume'):
        data = copy.deepcopy(base)
        data[k] = np.array(base[k], dtype=np.int64)
        equal(snapshot(data, 'data'), base)
    data = copy.deepcopy(base)
    data['close'] = np.array(base['close'], dtype='>f8')
    equal(snapshot(data, 'data'), base)
    for bad in (np.array([[1.0]]), np.array([1], dtype=np.float32), np.array([1], dtype=object),
                np.array([1], dtype=np.uint64), np.array([1j]), np.ma.array([1.0]), np.array(1.0)):
        error(lambda: snapshot({'close': bad}, 'data'), TypeError, 'invalid_type', 'data.close')
    for bad in (np.array([1000.0]), np.array([1000], dtype=np.int32)):
        error(lambda: snapshot({'timestamp': bad}, 'data'), TypeError, 'invalid_type', 'data.timestamp')
    error(lambda: snapshot({'is_closed': np.array([1], dtype=np.int64)}, 'data'), TypeError, 'invalid_type', 'data.is_closed')
    for bad in (float('nan'), float('inf'), -float('inf')):
        error(lambda: snapshot({'close': [bad]}, 'data'), ValueError, 'non_finite', 'data.close[0]')
        error(lambda: snapshot({'as_of': bad}, 'request'), ValueError, 'non_finite', 'request.as_of')
    error(lambda: snapshot({'symbol': '\ud800'}, 'data'), TypeError, 'invalid_type', 'data.symbol')
    error(lambda: snapshot({'\ud800': 1}, 'data'), TypeError, 'invalid_type', 'data')
    for bad in (True, 1000.0, '1000', np.int64(1000)):
        error(lambda: snapshot({'timestamp': [bad]}, 'data'), TypeError, 'invalid_type', 'data.timestamp[0]')
    for bad in (9007199254740992, -9007199254740992):
        error(lambda: snapshot({'timestamp': [bad]}, 'data'), ValueError, 'invalid_timestamp', 'data.timestamp[0]')
        error(lambda: snapshot({'close': [bad]}, 'data'), ValueError, 'invalid_type', 'data.close[0]')
    equal(snapshot({'timestamp': [-9007199254740991, 9007199254740991]}, 'data'),
          {'timestamp': [-9007199254740991, 9007199254740991]})
    cycle = {}; cycle['self'] = cycle
    error(lambda: snapshot(cycle, 'data'), TypeError, 'invalid_type', 'data.self')
    error(lambda: snapshot({1: 'bad'}, 'data'), TypeError, 'invalid_type', 'data')
    class Custom(dict):
        def items(self):
            raise AssertionError('Custom code must not run')
    error(lambda: snapshot(Custom(), 'data'), TypeError, 'invalid_type', 'data')
    data = copy.deepcopy(base); data['close'] = tuple(data['close'])
    equal(snapshot(data, 'data'), base)
    data = copy.deepcopy(base); data['close'][0] = None
    equal(snapshot(data, 'data')['close'][0], None)
    args = dict(symbol=base['symbol'], timeframe=base['timeframe'], metadata=base['metadata'])
    saved_base = copy.deepcopy(base)
    for backend in ('pandas', 'polars'):
        frame = pd.DataFrame({k: arrays[k] for k in columns}) if backend == 'pandas' else pl.DataFrame({k: arrays[k] for k in columns})
        convert = getattr(api, f'market_data_from_{backend}')
        equal(convert(frame, **args), base)
        if backend == 'pandas':
            frame.index = list(reversed(range(len(frame))))
        equal(convert(frame, **args), base)
        converted = convert(frame, **args)
        converted['metadata']['source'] = 'caller change'
        equal(base, saved_base)
        for wrong in ({}, frame.drop(columns=['close']) if backend == 'pandas' else frame.drop('close')):
            error(lambda: convert(wrong, **args), TypeError, 'invalid_type', 'frame' if type(wrong) is dict else 'frame.columns')
        extra = frame.assign(extra=1) if backend == 'pandas' else frame.with_columns(pl.lit(1).alias('extra'))
        error(lambda: convert(extra, **args), TypeError, 'invalid_type', 'frame.columns')
        bad = frame.copy() if backend == 'pandas' else frame.clone()
        if backend == 'pandas':
            bad['timestamp'] = bad['timestamp'].astype('float64')
        else:
            bad = bad.with_columns(pl.col('timestamp').cast(pl.Float64))
        error(lambda: convert(bad, **args), TypeError, 'invalid_type', 'data.timestamp')
    duplicate = pd.DataFrame(np.ones((6, 8)), columns=list(columns[:-1]) + ['close'])
    error(lambda: api.market_data_from_pandas(duplicate, **args), TypeError, 'invalid_type', 'frame.columns')
    nullable = pd.DataFrame({k: arrays[k] for k in columns}); nullable['volume'] = nullable['volume'].astype('Int64')
    error(lambda: api.market_data_from_pandas(nullable, **args), TypeError, 'invalid_type', 'data.volume')
    null_frame = pl.DataFrame({k: arrays[k] for k in columns}).with_columns(pl.lit(None, dtype=pl.Float64).alias('close'))
    equal(api.market_data_from_polars(null_frame, **args)['close'], [None] * 6)
    for convert, frame in [(api.market_data_from_pandas, pd.DataFrame({k: arrays[k] for k in columns})),
                           (api.market_data_from_polars, pl.DataFrame({k: arrays[k] for k in columns}))]:
        if type(frame) is pd.DataFrame:
            frame.loc[0, 'close'] = float('nan')
        else:
            frame = frame.with_columns(pl.lit(float('nan')).alias('close'))
        error(lambda: convert(frame, **args), ValueError, 'non_finite', 'data.close[0]')
    print(json.dumps(dict(passed=True, assertions=assertions, scope='host conversion only; native module is inert',
                          versions=dict(numpy=np.__version__, pandas=pd.__version__, polars=pl.__version__))))


if __name__ == '__main__':
    main()
