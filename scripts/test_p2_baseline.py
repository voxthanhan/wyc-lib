"""Measurement evidence controls; no assertions on runner speed."""
import copy
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from p2_baseline import SIZES, OPERATIONS, audit, inputs
from measure_baseline import launch_command, attach_process_metrics
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'benchmarks'))
from p2_process import worker_metrics


class Controls(unittest.TestCase):
    def setUp(self):
        self.requests = inputs(Path(__file__).resolve().parents[1])
        expected = dict(metadata={'as_of': 1200}, indicators={'value': 1.0}, structure={'status': 'confirmed'})
        self.reports = {runtime: dict(runtime=runtime, warmups=2, samples=5, iterations=3, profile='release',
            rows=[dict(bars=n, operation=op, ns_per_op=[1, 2, 3, 4, 5]) for n in SIZES for op in sorted(ops)],
            outputs=[copy.deepcopy(expected) if runtime == 'rust' else dict(
                batch=copy.deepcopy(expected), raw=copy.deepcopy(expected), array_batch=copy.deepcopy(expected),
                indicators=dict(metadata=expected['metadata'], output=expected['indicators']),
                structure=dict(metadata=expected['metadata'], output=expected['structure']),
                converted={k: r['data'] for k in ('pandas', 'polars')}) for r in self.requests])
            for runtime, ops in OPERATIONS.items()}

    def test_valid(self):
        self.assertEqual(audit(self.reports, self.requests)['node']['64'], 1)
        self.assertEqual([len(r['data']['close']) for r in self.requests], list(SIZES))
        self.assertEqual(self.requests[0]['data']['available_at'][0], 1200)

    def test_missing_or_duplicate_operation(self):
        self.reports['node']['rows'][-1] = self.reports['node']['rows'][0]
        with self.assertRaises(AssertionError):
            audit(self.reports, self.requests)

    def test_nonfinite_or_zero_or_short_samples(self):
        for values in ([1, 2], [0]*5, [float('nan')]*5, [float('inf')]*5):
            report = copy.deepcopy(self.reports)
            report['rust']['rows'][0]['ns_per_op'] = values
            with self.assertRaises(AssertionError):
                audit(report, self.requests)

    def test_wrong_output(self):
        self.reports['python']['outputs'][0]['array_batch']['structure']['status'] = 'candidate'
        with self.assertRaises(AssertionError):
            audit(self.reports, self.requests)

    def test_missing_runtime_or_wrong_protocol(self):
        self.reports['python']['profile'] = 'debug'
        with self.assertRaises(AssertionError):
            audit(self.reports, self.requests)
        del self.reports['python']
        with self.assertRaises(AssertionError):
            audit(self.reports, self.requests)

    def test_module_wrapper_required(self):
        self.reports['node']['outputs'][0]['indicators'] = {'value': 1.0}
        with self.assertRaises(AssertionError):
            audit(self.reports, self.requests)

    @unittest.skipUnless(sys.platform == 'linux', 'Linux symlink interpreter regression')
    def test_venv_python_symlink_preserves_environment(self):
        with tempfile.TemporaryDirectory(prefix='wyc-baseline-venv-') as scratch:
            venv = Path(scratch) / 'venv'
            subprocess.run([sys.executable, '-m', 'venv', '--without-pip', '--symlinks', venv], check=True)
            binary = venv / 'bin/python'
            self.assertTrue(binary.is_symlink())
            prefix = subprocess.check_output(launch_command(binary, ['-c', 'import sys; print(sys.prefix)']), text=True).strip()
            self.assertEqual(Path(prefix).resolve(), venv.resolve())

    def test_worker_metrics_override_launcher_and_missing_metrics_fail(self):
        worker = dict(pid=200, cpu_seconds=1.5, peak_rss_bytes=60000000)
        launcher = dict(pid=100, cpu_seconds=0.01, peak_rss_bytes=4000000, wall_seconds=2.0)
        report = dict(runtime='python', worker_process=worker)
        attach_process_metrics(report, launcher)
        self.assertEqual(report['process'], dict(worker, wall_seconds=2.0))
        self.assertEqual(report['parent_process_observation']['pid'], launcher['pid'])
        self.assertEqual(report['parent_process_observation']['cpu_seconds'], launcher['cpu_seconds'])
        self.assertIn('redirector', report['parent_process_observation']['scope'])
        with self.assertRaises(ValueError):
            attach_process_metrics(dict(runtime='python'), launcher)

    def test_sampler_reports_current_worker_pid_and_live_resources(self):
        metrics = worker_metrics()
        self.assertEqual(metrics['pid'], os.getpid())
        self.assertGreaterEqual(metrics['cpu_seconds'], 0)
        self.assertGreater(metrics['peak_rss_bytes'], 0)
        self.assertIn('worker', metrics['memory_method'])


if __name__ == '__main__':
    unittest.main()
