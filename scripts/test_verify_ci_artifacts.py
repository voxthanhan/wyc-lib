"""Local negative controls for the artifact verifier; no network or credentials."""
import hashlib
import io
import json
import unittest
import zipfile
from verify_ci_artifacts import audit

SHA = 'a' * 40
RUN = '123'


def fixture(change=None, extra=None):
    row = dict(samples_ms=[1., 2., 3., 4., 5.], min_ms=1., median_ms=3., max_ms=5.)
    raw = dict(profile='release', rows=[row] * 12)
    baseline = dict(raw, commit=SHA, clean=True, process=dict(cpu_seconds=1., peak_rss_bytes=1000))
    summary = dict(commit=SHA, passed=True, clean_before=True, clean_after=True,
                   github_run_url='https://github.com/example/repo/actions/runs/' + RUN,
                   checks=[dict(name=n, exit_code=0) for n in ['format', 'clippy', 'build',
                       'test', 'example', 'baseline-build', 'baseline', 'oracle', 'workspace']])
    trace = dict(prefixes=1008, traces=[dict(batch_state_equal=True, sha256='b' * 64)] * 1008)
    files = {'summary.json': summary, 'baseline.json': baseline,
             'baseline.raw.json': raw, 'replay-traces.json': trace}
    if change:
        change(files)
    members = {n: json.dumps(v).encode() for n, v in files.items()}
    members.update({'test.txt': b'test result: ok. 88 passed; 0 failed; 0 ignored',
                    'oracle.txt': b'PASS: SHA-256 manifest', 'workspace.txt': b'PASS:',
                    'example.txt': b'{}'})
    if extra:
        members.update(extra)
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, 'w') as archive:
        for name, data in members.items():
            archive.writestr(name, data)
    payload = stream.getvalue()
    return payload, 'sha256:' + hashlib.sha256(payload).hexdigest()


class ArtifactControls(unittest.TestCase):
    def test_valid_payload(self):
        payload, digest = fixture()
        _, report = audit(payload, digest, SHA, RUN, 88)
        self.assertEqual(report['integration_tests'], 88)
        self.assertEqual(report['replay_prefixes'], 1008)

    def test_wrong_digest_before_parsing_zip(self):
        with self.assertRaisesRegex(AssertionError, 'digest'):
            audit(b'not a ZIP', 'sha256:' + '0' * 64, SHA, RUN, 88)

    def test_wrong_commit_and_dirty_checkout(self):
        for change in [lambda f: f['summary.json'].update(commit='c' * 40),
                       lambda f: f['summary.json'].update(clean_after=False)]:
            payload, digest = fixture(change)
            with self.assertRaises(AssertionError):
                audit(payload, digest, SHA, RUN, 88)

    def test_failed_check_missing_trace_and_bad_baseline(self):
        for change in [lambda f: f['summary.json']['checks'][0].update(exit_code=1),
                       lambda f: f['replay-traces.json'].update(prefixes=999),
                       lambda f: f['baseline.json']['rows'][0].update(median_ms=999)]:
            payload, digest = fixture(change)
            with self.assertRaises(AssertionError):
                audit(payload, digest, SHA, RUN, 88)

    def test_zip_path_escape_is_rejected(self):
        for path in ['../escape.txt', '/absolute.txt', 'C:/escape.txt', '..\\escape.txt']:
            payload, digest = fixture(extra={path: b'unsafe'})
            with self.assertRaisesRegex(AssertionError, 'Unsafe ZIP path'):
                audit(payload, digest, SHA, RUN, 88)


if __name__ == '__main__':
    unittest.main()
