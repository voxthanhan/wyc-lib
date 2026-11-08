"""Download public CI ZIPs without credentials and independently audit payloads.

The optional public nightly.link redirect only transports bytes. Trust is the
SHA-256 published by GitHub for the exact artifact/run/SHA, checked before unzip.
Never log redirected signed URLs, read credentials, execute ZIP contents or
install/authorize an app. All extraction stays under the explicitly given root.
"""
import argparse
import hashlib
import io
import json
import math
import re
import stat
import urllib.request
import zipfile
from pathlib import Path


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def audit(payload, expected_digest, sha, run_id, minimum_tests):
    digest = 'sha256:' + hashlib.sha256(payload).hexdigest()
    check(digest == expected_digest, 'ZIP digest differs from GitHub published digest')
    archive = zipfile.ZipFile(io.BytesIO(payload))
    check(sum(i.file_size for i in archive.infolist()) < 30_000_000, 'Unexpected ZIP size')
    check(archive.testzip() is None, 'ZIP CRC failed')
    files = {}
    for item in archive.infolist():
        name = item.filename.replace('\\', '/')
        check(not name.startswith('/') and ':' not in name and '..' not in Path(name).parts, 'Unsafe ZIP path')
        check(not stat.S_ISLNK(item.external_attr >> 16), 'ZIP symlink refused')
        if not item.is_dir():
            check(name not in files, 'Duplicate ZIP member')
            files[name] = archive.read(item)
    required = {'summary.json', 'test.txt', 'oracle.txt', 'workspace.txt',
                'replay-traces.json', 'baseline.json', 'baseline.raw.json', 'example.txt'}
    check(required <= files.keys(), 'Required payload files missing')
    read_json = lambda name: json.loads(files[name].decode('utf-8-sig'))
    summary = read_json('summary.json')
    check(summary['commit'] == sha and summary['passed'], 'Wrong SHA or failing summary')
    check(summary['clean_before'] and summary['clean_after'], 'CI checkout was not clean')
    check(summary['github_run_url'].endswith('/' + str(run_id)), 'Wrong run URL in payload')
    checks = {c['name']: c['exit_code'] for c in summary['checks']}
    check(all(code == 0 for code in checks.values()), 'Nonzero check exit code')
    check({'format', 'clippy', 'build', 'test', 'example', 'baseline-build',
           'baseline', 'oracle', 'workspace'} <= checks.keys(), 'Missing runner checks')
    tests = files['test.txt'].decode('utf-8-sig')
    counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored', tests)
    check(counts and not any(int(f) or int(i) for _, f, i in counts), 'Failing or ignored tests')
    passed = sum(int(p) for p, _, _ in counts)
    check(passed >= minimum_tests, 'Too few tests in CI payload')
    check('PASS: SHA-256 manifest' in files['oracle.txt'].decode('utf-8-sig'), 'Oracle success missing')
    check('PASS:' in files['workspace.txt'].decode('utf-8-sig'), 'Workspace audit missing')
    trace = read_json('replay-traces.json')
    check(trace['prefixes'] == len(trace['traces']) == 1008, 'Incomplete replay traces')
    check(all(t['batch_state_equal'] and re.fullmatch('[0-9a-f]{64}', t['sha256'])
              for t in trace['traces']), 'Invalid replay result')
    baseline = read_json('baseline.json')
    check(baseline['commit'] == sha and baseline['clean'], 'Wrong benchmark SHA/dirty checkout')
    check(baseline['profile'] == 'release' and len(baseline['rows']) == 12, 'Baseline incomplete')
    for row in baseline['rows']:
        samples = row['samples_ms']
        check(len(samples) == 5 and all(math.isfinite(v) and v > 0 for v in samples), 'Bad timing samples')
        check(row['min_ms'] == min(samples) and row['max_ms'] == max(samples)
              and row['median_ms'] == sorted(samples)[2], 'Baseline summary mismatch')
    raw = read_json('baseline.raw.json')
    check(all(baseline.get(k) == v for k, v in raw.items()), 'Raw baseline differs from report')
    process = baseline['process']
    check(process['cpu_seconds'] > 0 and process['peak_rss_bytes'] > 0, 'CPU/RSS measurement missing')
    return files, dict(digest_verified=True, crc_verified=True, commit=sha,
        checkout_clean=True, runner_checks=checks, integration_tests=passed,
        replay_prefixes=trace['prefixes'], baseline_rows=12,
        process=process, files_sha256={n: hashlib.sha256(v).hexdigest() for n, v in files.items()})


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('evidence', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--minimum-tests', type=int, default=88)
    parser.add_argument('--local-trace', type=Path)
    args = parser.parse_args()
    record = json.loads(args.evidence.read_text(encoding='utf-8-sig'))
    sha, run_id = record['head_sha'], record['run']['id']
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    results = []
    for artifact in record['artifacts']:
        name = artifact['name']
        check(name.endswith(sha) and re.fullmatch('[A-Za-z0-9_.-]+', name), 'Unexpected artifact name')
        expected = artifact.get('published_digest', artifact.get('digest'))
        url = f'https://nightly.link/voxthanhan/wyc-lib/actions/runs/{run_id}/{name}.zip'
        local_zip = output / name / 'artifact.zip'
        if local_zip.exists():
            payload = local_zip.read_bytes()
        else:
            with urllib.request.urlopen(url, timeout=45) as response:
                payload = response.read(5_000_001)
            check(len(payload) <= 5_000_000, 'Unexpected download size')
        files, result = audit(payload, expected, sha, run_id, args.minimum_tests)
        target = output / name
        target.mkdir(parents=True, exist_ok=True)
        local_zip.write_bytes(payload)
        for filename, data in files.items():
            path = (target / 'payload' / filename).resolve()
            check(path.is_relative_to(target.resolve()), 'ZIP destination escaped output root')
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        if args.local_trace:
            local = json.loads(args.local_trace.read_text(encoding='utf-8-sig'))
            remote = json.loads(files['replay-traces.json'].decode('utf-8-sig'))
            check(local == remote, 'CI prefix traces differ from local traces')
            result['all_local_trace_digests_equal'] = True
        result.update(name=name, run_id=run_id, published_digest=expected,
                      bytes=len(payload), transport_url=url, payload_downloaded=True)
        results.append(result)
        print(f'PASS: {name}: ZIP digest/CRC, clean SHA, {result["integration_tests"]} tests, 1008 traces, baseline CPU/RSS')
    report = dict(evidence_version='1', head_sha=sha, run_id=run_id, artifacts=results,
                  authentication='none; no credentials read; public HTTP redirect, verified against GitHub digest')
    (output / 'verification.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')


if __name__ == '__main__':
    main()
