"""Negative controls for the acceptance comparator, independent of native builds."""
import unittest
from pathlib import Path
from p2_parity import compare, matrix


class ParityControls(unittest.TestCase):
    def test_numeric_tolerance_is_scoped(self):
        compare(1.0 + 1e-13, 1.0, ("series", "ema", 0, "result", "value"))
        compare(1e6 + 5e-6, 1e6, ("series", "variance", 0, "result", "value"))
        with self.assertRaises(AssertionError):
            compare(1.01, 1.0, ("series", "ema", 0, "result", "value"))
        for key in ("min", "max"):
            with self.assertRaises(AssertionError):
                compare(1.0 + 1e-13, 1.0, ("series", key, 0, "result", "value"))

    def test_discrete_fields_are_exact(self):
        for key in ("known_at", "observed_at", "as_of", "last_timestamp", "available", "required"):
            with self.assertRaises(AssertionError):
                compare(9007199254740990, 9007199254740991, (key,))
        for actual, expected in ((True, 1), (1, True), ("ready", "warmup"), (None, 0), ("id-a", "id-b")):
            with self.assertRaises(AssertionError):
                compare(actual, expected, ("status",))

    def test_shape_and_order_are_exact(self):
        for actual, expected in (({"a": 1}, {"b": 1}), ([1], [1, 2]), ([2, 1], [1, 2])):
            with self.assertRaises(AssertionError):
                compare(actual, expected)

    def test_nonfinite_never_matches(self):
        for value in (float("nan"), float("inf"), -float("inf")):
            with self.assertRaises(AssertionError):
                compare(value, value, ("value",))

    def test_matrix_is_deterministic_and_covers_public_operations(self):
        root = Path(__file__).resolve().parents[1]
        first = matrix(root)
        self.assertEqual(first, matrix(root))
        self.assertEqual(len(first["actions"]), len({a["name"] for a in first["actions"]}))
        self.assertEqual({a["op"] for a in first["actions"]},
                         {"batch", "indicators", "structure", "new", "update", "reset", "result", "identity"})
        self.assertEqual(first["fixture_cases"], 39)
        self.assertGreater(len(first["equalities"]), 1008)
        self.assertGreater(len(first["expected_errors"]), 50)


if __name__ == "__main__":
    unittest.main()
