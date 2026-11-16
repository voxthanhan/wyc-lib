"""Run copied examples and real production HTTP consumers of installed artifacts."""
import json
import os
import shutil
import socket
import subprocess
import time
import urllib.error
import urllib.request


def http_smoke(name, command, cwd, request, expected, evidence):
    with socket.socket() as reservation:
        reservation.bind(('127.0.0.1', 0))
        port = reservation.getsockname()[1]
    env = dict(os.environ, PORT=str(port), NEXT_TELEMETRY_DISABLED='1', NODE_ENV='production')
    url = f'http://127.0.0.1:{port}/api/analyze' if name == 'next' else f'http://127.0.0.1:{port}/analyze'
    command = [str(x).replace('{port}', str(port)) for x in command]
    # Bypass proxies for this loopback-only integration test.
    client = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    checks = []
    with (evidence / f'{name}-server.txt').open('w', encoding='utf-8') as log:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 60
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f'{name} server exited: {process.returncode}')
                try:
                    client.open(url, timeout=1).close()
                    break
                except urllib.error.HTTPError:
                    break  # GET is deliberately unsupported; the server is listening.
                except urllib.error.URLError:
                    if time.monotonic() >= deadline:
                        raise RuntimeError(f'{name} server readiness timeout')
                    time.sleep(0.2)

            cases = [('batch', request, 200, expected),
                     ('invalid-shape', {}, 400, dict(error=dict(code='invalid_type', path='request'))),
                     ('malformed-json', '{', 400, dict(error=dict(code='invalid_json', path='request')))]
            missing = json.loads(json.dumps(request))
            missing['data']['close'][0] = None
            cases.append(('missing-data', missing, 400, dict(error=dict(code='missing_data', path='close[0]'))))
            negative = json.loads(json.dumps(request))
            negative['data']['volume'][0] = -1
            cases.append(('negative-volume', negative, 400, dict(error=dict(code='negative_volume', path='volume[0]'))))
            cases.append(('recovery', request, 200, expected))
            for label, body, status, result in cases:
                data = (body if isinstance(body, str) else json.dumps(body)).encode()
                req = urllib.request.Request(url, data=data, headers={'Content-Type': 'application/json'})
                try:
                    response = client.open(req, timeout=15)
                except urllib.error.HTTPError as error:
                    response = error
                with response:
                    actual = json.loads(response.read())
                    if response.status != status or actual != result:
                        raise AssertionError(f'{name}/{label}: HTTP status/body differs from Core reference')
                    checks.append(dict(name=label, status=response.status))
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
    return dict(passed=True, checks=checks, production=True, host='127.0.0.1')


def distribution(root, scratch, consumer, python, npm, tarball, expected, run, evidence):
    examples = root / 'examples/p2'
    for name in ('request.json', 'node.cjs', 'python.py', 'node-server.cjs'):
        shutil.copy2(examples / name, consumer / name)
    request = json.loads((consumer / 'request.json').read_text(encoding='utf-8'))
    results = {}
    for runtime, command in (('node', ['node', 'node.cjs']), ('python', [python, '-I', 'python.py'])):
        output = json.loads(run(f'{runtime}-example', command, cwd=consumer))
        wanted = dict(batch=expected, state=expected,
                      indicators=dict(metadata=expected['metadata'], output=expected['indicators']),
                      structure=dict(metadata=expected['metadata'], output=expected['structure']))
        if output != wanted:
            raise AssertionError(f'{runtime} example differs from direct Core reference')
        results[runtime + '_example'] = dict(passed=True, public_apis=4)
    results['node_server'] = http_smoke('node', ['node', 'node-server.cjs'], consumer,
                                        request, expected, evidence)
    next_dir = scratch / 'next-consumer'
    shutil.copytree(examples / 'next-server', next_dir, ignore=shutil.ignore_patterns('node_modules', '.next'))
    run('next-install', [npm, 'ci', '--ignore-scripts', '--no-audit', '--no-fund'], cwd=next_dir)
    run('next-package-install', [npm, 'install', '--no-save', '--engine-strict', '--ignore-scripts',
                                  '--no-audit', '--no-fund', tarball], cwd=next_dir)
    entry = next_dir / 'node_modules/next/dist/bin/next'
    run('next-build', ['node', entry, 'build', '--webpack'], cwd=next_dir)
    results['next_server'] = http_smoke('next', ['node', entry, 'start', '--hostname', '127.0.0.1',
                                               '--port', '{port}'], next_dir, request, expected, evidence)
    results['next_server']['versions'] = {name: json.loads((next_dir / f'node_modules/{name}/package.json').read_text())['version']
                                         for name in ('next', 'react', 'react-dom')}
    return results
