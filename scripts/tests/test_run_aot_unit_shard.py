"""Anti-vacuity and partition checks for the AOT CI runner."""

import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "run_aot_unit_shard", Path(__file__).parents[1] / "run_aot_unit_shard.py"
)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class InventoryTests(unittest.TestCase):
    def test_inventory_uses_declared_total_and_sorts_names(self):
        self.assertEqual(runner.parse_inventory("z::a: test\na::b: test\n\n2 tests, 0 benchmarks\n"), ["a::b", "z::a"])

    def test_rejects_empty_duplicate_truncated_and_benchmark_inventories(self):
        for text in ["0 tests, 0 benchmarks", "a: test\na: test\n2 tests, 0 benchmarks",
                     "a: test\n2 tests, 0 benchmarks", "a: test", "a: test\n1 test, 1 benchmark"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                runner.parse_inventory(text)

    def test_every_test_is_assigned_exactly_once(self):
        for total in [1, 27, 392, 393]:
            names = [f"test_{i:04}" for i in range(total)]
            for count in range(1, min(total, 8) + 1):
                shards = [runner.select_shard(names, i, count) for i in range(count)]
                self.assertTrue(all(shards))
                self.assertEqual(sorted(name for shard in shards for name in shard), names)

    def test_invalid_shards_fail_instead_of_selecting_nothing(self):
        for index, count in [(-1, 1), (0, 0), (1, 1), (0, 2)]:
            with self.subTest(index=index, count=count), self.assertRaises(ValueError):
                runner.select_shard(["a"], index, count)

    def test_result_must_execute_one_test_without_ignores(self):
        good = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 391 filtered out; finished in 0.01s\n"
        self.assertTrue(runner.passed_exactly_one(good, 392))
        for bad in ["", good.replace("1 passed", "0 passed"), good.replace("0 ignored", "1 ignored"),
                    good.replace("391 filtered out", "390 filtered out"), good + good]:
            with self.subTest(output=bad):
                self.assertFalse(runner.passed_exactly_one(bad, 392))


if __name__ == "__main__":
    unittest.main()
