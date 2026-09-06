#!/usr/bin/env python3
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from run_engine_regression_inventory import run_inventory

INVENTORY = 'a: test\nb: test\n\n2 tests, 0 benchmarks\n'
PASS = 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s\n'


def completed(output, code=0):
    return subprocess.CompletedProcess([], code, stdout=output)


class EngineInventoryTests(unittest.TestCase):
    def execute(self, responses):
        with tempfile.TemporaryDirectory() as directory, patch(
                'run_engine_regression_inventory.subprocess.run', side_effect=responses) as run:
            result = run_inventory(Path('/test-binary'), Path(directory), 3)
            self.assertEqual(result, json.loads((Path(directory) / 'summary.json').read_text()))
            self.assertEqual([call.args[0][1] for call in run.call_args_list[1:]], ['a', 'b'])
            for call in run.call_args_list[1:]:
                self.assertIn('--exact', call.args[0])
                self.assertEqual(call.kwargs['timeout'], 3)
            return result

    def test_complete_inventory_runs_once_per_test(self):
        result = self.execute([completed(INVENTORY), completed(PASS), completed(PASS)])
        self.assertEqual((result['total'], result['passed']), (2, 2))

    def test_ignored_and_empty_selections_fail(self):
        ignored = PASS.replace('1 passed; 0 failed; 0 ignored', '0 passed; 0 failed; 1 ignored')
        empty = PASS.replace('1 passed;', '0 passed;').replace('1 filtered out;', '2 filtered out;')
        result = self.execute([completed(INVENTORY), completed(ignored), completed(empty)])
        self.assertEqual(result['passed'], 0)

    def test_nonzero_exit_cannot_be_overridden_by_success_text(self):
        result = self.execute([completed(INVENTORY), completed(PASS, 1), completed(PASS)])
        self.assertEqual(result['passed'], 1)

    def test_timeout_is_recorded_and_remaining_inventory_still_runs(self):
        timeout = subprocess.TimeoutExpired(['/test-binary'], 3, output=b'partial output\n')
        result = self.execute([completed(INVENTORY), timeout, completed(PASS)])
        self.assertEqual([r['status'] for r in result['results']], ['timeout', 'passed'])

    def test_incomplete_or_empty_inventory_is_rejected_before_execution(self):
        for inventory in ['', '0 tests, 0 benchmarks\n', 'a: test\n2 tests, 0 benchmarks\n']:
            with tempfile.TemporaryDirectory() as directory, patch(
                    'run_engine_regression_inventory.subprocess.run', return_value=completed(inventory)) as run:
                with self.assertRaises(ValueError):
                    run_inventory(Path('/test-binary'), Path(directory), 3)
                self.assertEqual(run.call_count, 1)


if __name__ == '__main__':
    unittest.main()
