"""P2.6 reproducible inputs, measurements and fail-closed evidence audit."""
import copy
import hashlib
import json
import math
import shutil
import statistics
from pathlib import Path
from p2_parity import compare

SIZES = (64, 256, 1024)
COMMON = {'deepcopy_proxy', 'host_snapshot_encode', 'output_decode',
          'native_json_transport', 'typed_batch', 'typed_two_calls', 'array_batch'}
OPERATIONS = dict(rust={'direct_batch', 'direct_two_calls', 'input_clone',
                       'input_decode', 'output_encode', 'json_transport'},
                  node=COMMON, python=COMMON | {'pandas_conversion', 'polars_conversion'})


def inputs(root):
    template = json.loads((root / 'examples/p2/request.json').read_text(encoding='utf-8'))
    requests = []
    for n in SIZES:
        r = copy.deepcopy(template)
        d = r['data']
        d['symbol'] = 'BASELINE'
        d['metadata'].update(source='synthetic-triangle-v1', data_revision='baseline-r1')
        d['timestamp'] = [(i + 1) * 1000 for i in range(n)]
        d['available_at'] = [t + 200 for t in d['timestamp']]
        d['close'] = [100 + min(i % 32, 32 - i % 32) for i in range(n)]
        d['open'] = d['close'][:]
        d['high'] = [v + 1 for v in d['close']]
        d['low'] = [v - 1 for v in d['close']]
        d['volume'] = [(i % 17) * 100 for i in range(n)]
        d['is_closed'] = [True] * n
        r['as_of'] = d['available_at'][-1]
        r['config']['indicators'].update(period=14, macd_fast=12, macd_slow=26, macd_signal=9)
        r['config']['structure'].update(left=2, right=2)
        requests.append(r)
    return requests


def audit(reports, requests):
    if set(reports) != set(OPERATIONS):
        raise AssertionError('missing runtime')
    for runtime, report in reports.items():
        if (report['runtime'], report['warmups'], report['samples'], report['iterations'], report['profile']) != (
                runtime, 2, 5, 3, 'release'):
            raise AssertionError('measurement protocol differs')
        expected = {(n, op) for n in SIZES for op in OPERATIONS[runtime]}
        actual = [(r['bars'], r['operation']) for r in report['rows']]
        if len(actual) != len(expected) or set(actual) != expected:
            raise AssertionError('operation coverage differs')
        for row in report['rows']:
            values = row['ns_per_op']
            if len(values) != 5 or any(type(v) not in (float, int) or not math.isfinite(v) or v <= 0 for v in values):
                raise AssertionError('invalid timing samples')
            row.update(median_ns=statistics.median(values), min_ns=min(values), max_ns=max(values))
            row['bars_per_second'] = row['bars'] * 1e9 / row['median_ns']
        if len(report['outputs']) != len(SIZES):
            raise AssertionError('output coverage differs')
    for i, request in enumerate(requests):
        expected = reports['rust']['outputs'][i]
        for runtime in ('node', 'python'):
            output = reports[runtime]['outputs'][i]
            for kind in ('batch', 'raw', 'array_batch'):
                compare(output[kind], expected)
            for kind in ('indicators', 'structure'):
                compare(output[kind], dict(metadata=expected['metadata'], output=expected[kind]))
            if runtime == 'python':
                for kind in ('pandas', 'polars'):
                    compare(output['converted'][kind], request['data'])
    return {runtime: {str(n): next(r['median_ns'] for r in report['rows'] if r['bars'] == n and r['operation'] in (
                'typed_two_calls', 'direct_two_calls')) / next(r['median_ns'] for r in report['rows']
                if r['bars'] == n and r['operation'] in ('typed_batch', 'direct_batch')) for n in SIZES}
            for runtime, report in reports.items()}


def baseline(root, consumer, python, target, evidence, run):
    requests = inputs(root)
    encoded = json.dumps(requests, separators=(',', ':'), allow_nan=False)
    input_path = consumer / 'baseline-input.json'
    input_path.write_text(encoded, encoding='utf-8')
    (evidence / 'baseline-input.json').write_text(encoded, encoding='utf-8')
    run('baseline-controls', [shutil.which('python') or python, root / 'scripts/test_p2_baseline.py'])
    run('baseline-build', ['cargo', 'build', '--release', '--locked', '--package', 'wyc-binding-contract',
                           '--bin', 'p2_baseline', '--target-dir', target])
    for name in ('p2_node.cjs', 'p2_python.py'):
        shutil.copy2(root / 'benchmarks' / name, consumer / name)
    commands = dict(rust=[target / 'release' / ('p2_baseline.exe' if __import__('os').name == 'nt' else 'p2_baseline')],
                    node=[Path(shutil.which('node')), consumer / 'p2_node.cjs'],
                    python=[python, consumer / 'p2_python.py'])
    reports = {}
    for runtime, command in commands.items():
        output = evidence / f'baseline-{runtime}.json'
        run(f'baseline-{runtime}', [python, root / 'scripts/measure_baseline.py', command[0], output,
                                   *command[1:], input_path], cwd=consumer)
        reports[runtime] = json.loads(output.read_text(encoding='utf-8'))
    ratios = audit(reports, requests)
    for runtime, report in reports.items():
        # Full outputs remain in raw JSON, keeping summary reasonably small.
        del report['outputs']
        (evidence / f'baseline-{runtime}.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    report = dict(protocol='p2-baseline-v1', input_sha256=hashlib.sha256(encoded.encode()).hexdigest(),
                  sizes=list(SIZES), warmups=2, samples=5, iterations=3, profile='release',
                  two_calls_over_batch=ratios, reports=reports, outputs_verified=True,
                  scope='layer timings overlap; process CPU/RSS includes setup, not heap/leak evidence')
    (evidence / 'baseline-report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    return {k: v for k, v in report.items() if k != 'reports'}
