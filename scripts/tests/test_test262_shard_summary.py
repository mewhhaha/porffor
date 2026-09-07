"""Failure controls for CLI summary checking; Rust owns exact snapshot ID validation."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "check-test262-shard-summary.py"
SPEC = importlib.util.spec_from_file_location("shard_summary", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)

# Actual output shape from map shard 4/4, run 34095866697, job 101660503669.
# This fixture is a report contract, not a saved conformance verdict.
INVENTORY = "count: 429\nsloppy-script:built-ins/Array/prototype/map/first.js\n... 379 more\n"
REPORT = """test262 checkpoint: 10/107 cases
execution_backend: wasm-aot
shard: 4/4
total: 107
passed: 107
failed: 0
Parser: 0
EarlyError: 0
Lowering: 0
Runtime: 0
WasmBackend: 0
HostHarness: 0
Unsupported: 0
"""


class SummaryTests(unittest.TestCase):
    def test_real_report_shape_is_accepted_without_claiming_displayed_ids_are_complete(self):
        self.assertEqual(MODULE.validate_summary(REPORT, INVENTORY, 4, 4), 107)

    def test_shard_counts_cover_the_entire_inventory_for_even_and_uneven_partitions(self):
        for total in (4, 5, 7, 8, 429, 500):
            counts = [MODULE.selected_count(f"count: {total}\n", index, 4) for index in range(1, 5)]
            self.assertEqual(sum(counts), total)
            self.assertLessEqual(max(counts) - min(counts), 1)
            self.assertEqual(counts, [len(range(index, total, 4)) for index in range(4)])

    def test_empty_and_out_of_range_shards_fail(self):
        for inventory, index, shards in (("count: 0\n", 1, 1), ("count: 1\n", 4, 4),
                                         (INVENTORY, 0, 4), (INVENTORY, 5, 4), (INVENTORY, 1, 0)):
            with self.subTest(index=index, shards=shards), self.assertRaises(ValueError):
                MODULE.selected_count(inventory, index, shards)

    def test_missing_and_duplicate_inventory_counts_fail(self):
        for inventory in ("", "429\n", INVENTORY + "count: 429\n", "notice\n" + INVENTORY):
            with self.subTest(inventory=inventory), self.assertRaises(ValueError):
                MODULE.validate_summary(REPORT, inventory, 4, 4)

    def test_every_required_field_must_be_present_exactly_once(self):
        for key in MODULE.FIELDS:
            line = next(line for line in REPORT.splitlines() if line.startswith(key + ": "))
            for changed in (REPORT.replace(line + "\n", ""), REPORT + line + "\n"):
                with self.subTest(key=key), self.assertRaises(ValueError):
                    MODULE.validate_summary(changed, INVENTORY, 4, 4)

    def test_every_non_success_bucket_is_rejected(self):
        for key in ("failed", *MODULE.KINDS):
            with self.subTest(key=key), self.assertRaises(ValueError):
                MODULE.validate_summary(REPORT.replace(f"{key}: 0", f"{key}: 1"), INVENTORY, 4, 4)

    def test_partial_or_inconsistent_totals_fail(self):
        for changed in (REPORT.replace("total: 107", "total: 106"),
                        REPORT.replace("passed: 107", "passed: 106"),
                        REPORT.replace("total: 107", "total: 108")):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                MODULE.validate_summary(changed, INVENTORY, 4, 4)

    def test_wrong_backend_or_shard_is_rejected(self):
        for changed in (REPORT.replace("wasm-aot", "spec-exec"), REPORT.replace("shard: 4/4", "shard: 3/4")):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                MODULE.validate_summary(changed, INVENTORY, 4, 4)

    def test_unknown_diagnostic_output_does_not_turn_green(self):
        for extra in ("Timeout: 1", "Unknown: 0", "failure: a testcase", "unrecognized output"):
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                MODULE.validate_summary(REPORT + extra + "\n", INVENTORY, 4, 4)

    def test_progress_denominator_order_range_and_position_are_checked(self):
        for changed in (REPORT.replace("10/107", "10/106"), REPORT.replace("10/107", "108/107"),
                        REPORT.replace("10/107", "0/107"),
                        "test262 checkpoint: 20/107 cases\n" + REPORT,
                        REPORT + "test262 checkpoint: 20/107 cases\n"):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                MODULE.validate_summary(changed, INVENTORY, 4, 4)

    def test_reports_without_periodic_checkpoints_are_valid(self):
        self.assertEqual(MODULE.validate_summary(REPORT.split("\n", 1)[1], INVENTORY, 4, 4), 107)

    def test_noncanonical_counts_are_rejected(self):
        for value in ("-1", "+107", "1e2", "107.0", "0107", "107 ", "\u0661\u0660\u0667", "9" * 20):
            with self.subTest(value=value), self.assertRaises(ValueError):
                MODULE.validate_summary(REPORT.replace("passed: 107", "passed: " + value), INVENTORY, 4, 4)

    def test_concatenated_runs_cannot_pass(self):
        with self.assertRaises(ValueError):
            MODULE.validate_summary(REPORT + REPORT, INVENTORY, 4, 4)

    def test_cli_success_and_failure_exit_codes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report, inventory = root / "report.txt", root / "inventory.txt"
            report.write_text(REPORT, encoding="utf-8")
            inventory.write_text(INVENTORY, encoding="utf-8")
            args = [sys.executable, str(SCRIPT), "--report", str(report), "--inventory", str(inventory),
                    "--shard-index", "4", "--shard-count", "4"]
            good = subprocess.run(args, capture_output=True, text=True)
            self.assertEqual(good.returncode, 0, good.stderr)
            report.write_text(REPORT.replace("Unsupported: 0", "Unsupported: 1"), encoding="utf-8")
            bad = subprocess.run(args, capture_output=True, text=True)
            self.assertEqual(bad.returncode, 1)
            report.unlink()
            missing = subprocess.run(args, capture_output=True, text=True)
            self.assertEqual(missing.returncode, 1)


if __name__ == "__main__":
    unittest.main()
