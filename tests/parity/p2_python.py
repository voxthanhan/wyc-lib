"""Installed-wheel native P1 API tests, independent direct-Core references."""
import copy
import gc
import json
import sys
from pathlib import Path
import wyc_lib as api

bundle = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))
actions, references = bundle['actions'], bundle['references']
full = json.loads(next(x['input'] for x in actions if x['name'] == 'batch-full'))
columns = ('timestamp', 'available_at', 'open', 'high', 'low', 'close', 'volume', 'is_closed')
assertions = 0


def equal(actual, expected):
    global assertions
    assert actual == expected
    assertions += 1


def error(call, cls, code, path=None):
    global assertions
    try:
        call()
    except cls as exc:
        assert exc.code == code and (path is None or exc.path == path), (exc.code, exc.path)
    else:
        raise AssertionError(f'Expected {cls.__name__}: {code}')
    assertions += 1


def prefix(n):
    request = copy.deepcopy(full)
    for k in columns:
        request['data'][k] = request['data'][k][:n]
    request['as_of'] = n * 1000
    return request


def main():
    equal('numpy' in sys.modules, False)  # Plain API must not import extras.
    equal(Path(api.__file__).parent.joinpath('py.typed').is_file(), True)
    equal(Path(api.__file__).with_suffix('.pyi').is_file(), True)
    for name in ('batch-full', 'batch-empty', 'unicode', 'zero-volume'):
        request = json.loads(next(x['input'] for x in actions if x['name'] == name))
        before = copy.deepcopy(request); expected = references[name]
        equal(api.analyze_batch(request), expected); equal(request, before)
        equal(api.calculate_indicators(request['data'], request['config'], request['as_of']),
              dict(metadata=expected['metadata'], output=expected['indicators']))
        equal(api.analyze_structure(request['data'], request['config'], request['as_of']),
              dict(metadata=expected['metadata'], output=expected['structure']))
    for module, absent in [('indicators', 'structure'), ('market_structure', 'indicators')]:
        request = copy.deepcopy(full); request['modules'] = (module,)
        equal(api.analyze_batch(request)[absent], None)
    for bad in (None, '{}', [], 1, True):
        error(lambda: api.analyze_batch(bad), TypeError, 'invalid_type', 'request')
    for value in (float('nan'), float('inf'), -float('inf')):
        request = copy.deepcopy(full); request['data']['close'][0] = value
        error(lambda: api.analyze_batch(request), ValueError, 'non_finite', 'request.data.close[0]')
    for value in (True, 6000.0, '6000'):
        request = copy.deepcopy(full); request['as_of'] = value
        error(lambda: api.analyze_batch(request), TypeError, 'invalid_type', 'request.as_of')
    request = copy.deepcopy(full); request['as_of'] = 9007199254740992
    error(lambda: api.analyze_batch(request), ValueError, 'invalid_timestamp', 'request.as_of')
    for key, value, cls, code, path in [
        ('extra', 1, TypeError, 'invalid_type', 'request'),
        ('modules', ['unknown'], TypeError, 'invalid_type', 'request'),
        ('modules', ['price_zones'], ValueError, 'unsupported_module', 'modules')]:
        request = copy.deepcopy(full); request[key] = value
        error(lambda: api.analyze_batch(request), cls, code, path)
    for key, value, cls, code, path in [
        ('close', [None] * 6, TypeError, 'missing_data', 'close[0]'),
        ('close', [], ValueError, 'length_mismatch', 'close'),
        ('volume', [-1] * 6, ValueError, 'negative_volume', 'volume[0]'),
        ('timestamp', [1000] * 6, ValueError, 'duplicate_timestamp', 'timestamp[1]')]:
        request = copy.deepcopy(full); request['data'][key] = value
        error(lambda: api.analyze_batch(request), cls, code, path)
    request = copy.deepcopy(full); request['data']['close'][0] = request['data']['high'][0] = 1e308
    error(lambda: api.analyze_batch(request), api.WycEngineError, 'numeric_overflow')
    original = prefix(3); state = api.AnalysisState(original); other = api.AnalysisState(original)
    original['data']['close'][0] = 999
    equal(state.result(), references['initial-result'])
    saved = state.result(); saved['metadata']['symbol'] = 'mutated'
    identity = state.identity(); identity['symbol'] = 'mutated'
    equal(state.result(), references['initial-result'])
    for n in range(3, 6):
        request = prefix(n + 1)
        update = dict(identity=state.identity(), metadata=request['data']['metadata'],
                      bar={k: request['data'][k][n] for k in columns}, as_of=request['as_of'])
        before, cursor = state.result(), state.identity()
        bad = copy.deepcopy(update); bad['bar']['close'] = float('nan')
        error(lambda: state.update_closed_bar(bad), ValueError, 'non_finite', 'update.bar.close')
        bad = copy.deepcopy(update); bad['identity']['symbol'] = 'wrong'
        error(lambda: state.update_closed_bar(bad), api.WycStateError, 'state_mismatch', 'state.identity')
        equal(state.result(), before); equal(state.identity(), cursor)
        equal(state.update_closed_bar(update), references[f'append-{n}'])
        equal(state.result(), api.analyze_batch(request))
    bad = copy.deepcopy(full); bad['config']['indicators']['period'] = 0
    error(lambda: state.reset(bad), ValueError, 'invalid_config', 'config.indicators.period')
    equal(state.result(), references['batch-full'])
    revised = json.loads(next(x['input'] for x in actions if x['name'] == 'revision-reset'))
    equal(state.reset(revised), references['revision-reset'])
    equal(other.result(), references['initial-result'])
    state.close(); state.close(); other.close()
    for call in (state.result, state.identity, lambda: state.reset(None), lambda: state.update_closed_bar(None), state.__enter__):
        error(call, api.WycStateError, 'state_closed', 'state')
    try:
        with api.AnalysisState(prefix(1)) as managed:
            equal(managed.result(), api.analyze_batch(prefix(1)))
            raise LookupError('test')
    except LookupError:
        pass
    error(managed.result, api.WycStateError, 'state_closed', 'state')
    for _ in range(100):
        api.AnalysisState(prefix(1))
    gc.collect()
    if '--extras' in sys.argv:
        import numpy as np
        import pandas as pd
        import polars as pl
        request = copy.deepcopy(full)
        for key in columns:
            dtype = np.bool_ if key == 'is_closed' else np.int64 if key in columns[:2] else np.float64
            request['data'][key] = np.array(full['data'][key], dtype=dtype)
        equal(api.analyze_batch(request), references['batch-full'])
        with api.AnalysisState(request) as owned:
            request['data']['close'][0] = 999
            equal(owned.result(), references['batch-full'])
        for backend in ('pandas', 'polars'):
            frame = pd.DataFrame({k: full['data'][k] for k in columns}) if backend == 'pandas' else pl.DataFrame({k: full['data'][k] for k in columns})
            convert = getattr(api, f'market_data_from_{backend}')
            data = convert(frame, symbol=full['data']['symbol'], timeframe=full['data']['timeframe'], metadata=full['data']['metadata'])
            equal(data, full['data'])
            equal(api.analyze_batch(dict(full, data=data)), references['batch-full'])
        nulls = pl.DataFrame({k: full['data'][k] for k in columns}).with_columns(pl.lit(None, dtype=pl.Float64).alias('close'))
        data = api.market_data_from_polars(nulls, symbol=full['data']['symbol'], timeframe=full['data']['timeframe'], metadata=full['data']['metadata'])
        error(lambda: api.analyze_batch(dict(full, data=data)), TypeError, 'missing_data', 'close[0]')
    print(json.dumps(dict(passed=True, assertions=assertions, extras='--extras' in sys.argv,
                          gc='100 abandoned handles, no crash; leak/RSS not measured')))


if __name__ == '__main__':
    main()
