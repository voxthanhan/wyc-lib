"""Measurement evidence controls; no assertions on runner speed."""
import copy
import unittest
from pathlib import Path
from p2_baseline import SIZES, OPERATIONS, audit, inputs


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


if __name__ == '__main__':
    unittest.main()
