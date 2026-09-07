"""Exact-denominator and non-success failure controls for real shard reports."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'check-test262-shard-report.py'
SPEC = importlib.util.spec_from_file_location('shard_report', SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def inventory(total=9):
    lines = [f'count: {total}']
    lines.extend(f'sloppy-script:built-ins/Array/prototype/map/case-{index}.js'
                 for index in range(min(total, 50)))
    if total > 50:
        lines.append(f'... {total - 50} more')
    return '\n'.join(lines) + '\n'


def report(total=3, selector='1/4'):
    lines = ['execution_backend: wasm-aot', f'shard: {selector}',
             f'total: {total}', f'passed: {total}', 'failed: 0']
    lines.extend(kind + ': 0' for kind in MODULE.FAILURE_KINDS)
    return '\n'.join(lines) + '\n'


class ShardReportTests(unittest.TestCase):
    def test_accepts_exact_native_summary_and_prior_progress(self):
        self.assertEqual(MODULE.verify(inventory(), 'progress 3/3\n' + report(), '1/4'), 3)
        self.assertEqual(MODULE.verify(inventory(), report(2, '4/4'), '4/4'), 2)

    def test_cli_fails_closed_without_rewriting_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            inventory_path, report_path = root / 'inventory.txt', root / 'report.txt'
            inventory_path.write_text(inventory(), encoding='utf-8')
            report_path.write_text(report(), encoding='utf-8')
            command = [sys.executable, str(SCRIPT), '--inventory', str(inventory_path),
                       '--report', str(report_path), '--shard', '1/4']
            good = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(good.returncode, 0, good.stderr)
            self.assertIn('verified 3/3 Wasm-AOT executions for 1/4', good.stdout)
            corrupted = report().replace('passed: 3', 'passed: 2')
            report_path.write_text(corrupted, encoding='utf-8')
            bad = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertNotEqual(bad.returncode, 0)
            self.assertEqual(report_path.read_text(encoding='utf-8'), corrupted)
            report_path.unlink()
            missing = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertNotEqual(missing.returncode, 0)
            self.assertFalse(report_path.exists())

    def test_round_robin_sizes_cover_every_execution_once(self):
        for count in range(1, 12):
            for total in range(count, 80):
                expected = [sum(index % count == shard for index in range(total))
                            for shard in range(count)]
                observed = [MODULE.expected_shard(total, f'{index}/{count}')
                            for index in range(1, count + 1)]
                self.assertEqual(observed, expected)
                self.assertEqual(sum(observed), total)

    def test_rejects_empty_out_of_range_and_noncanonical_selections(self):
        for total, selector in [(0, '1/4'), (1, '4/4'), (9, '0/4'), (9, '5/4'),
                                (9, '1/0'), (9, '01/4'), (9, '1/04'), (9, '1'),
                                (9, '1/4/5'), (9, '-1/4')]:
            with self.subTest(total=total, selector=selector), self.assertRaises(ValueError):
                MODULE.expected_shard(total, selector)

    def test_native_inventory_modes_and_truncation_footer(self):
        for total in (1, 49, 50, 51, 1000):
            self.assertEqual(MODULE.inventory_total(inventory(total)), total)
        for mode in ('sloppy-script', 'strict-script', 'raw-script', 'module', 'raw-module'):
            self.assertEqual(MODULE.inventory_total(f'count: 1\n{mode}:path:with-colon.js\n'), 1)

    def test_rejects_bad_empty_duplicate_or_truncated_inventory(self):
        good = inventory()
        bad = ['', 'count: 0\n', good.replace('count: 9', 'count: 09'),
               good.replace('case-1.js', 'case-0.js'), good.replace('sloppy-script:', 'invalid:'),
               '\n'.join(good.splitlines()[:-1]) + '\n', good + 'extra\n',
               inventory(51).replace('... 1 more', '... 2 more'), 'count: 1\nmodule:\n']
        for text in bad:
            with self.subTest(text=text), self.assertRaises(ValueError):
                MODULE.inventory_total(text)

    def test_rejects_each_nonzero_failure_bucket(self):
        for kind in ('failed', *MODULE.FAILURE_KINDS):
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                MODULE.verify(inventory(), report().replace(kind + ': 0', kind + ': 1'), '1/4')

    def test_rejects_wrong_backend_shard_missing_duplicate_or_unknown_fields(self):
        good = report()
        bad = [good.replace('wasm-aot', 'spec-exec'), good.replace('shard: 1/4', 'shard: 2/4'),
               good.replace('Runtime: 0\n', ''), good + 'Runtime: 0\n',
               good + 'Unknown: 0\n', good + '\n', 'total: 3\n' + good,
               good + good, good.replace('WasmBackend: 0', 'Backend: 0')]
        for text in bad:
            with self.subTest(text=text), self.assertRaises(ValueError):
                MODULE.verify(inventory(), text, '1/4')

    def test_rejects_partial_extra_or_inconsistent_pass_totals(self):
        for text in [report(2), report(4), report(0), report().replace('passed: 3', 'passed: 2')]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                MODULE.verify(inventory(), text, '1/4')

    def test_rejects_noncanonical_and_overflowing_counts(self):
        for value in ('+3', '03', '3 ', '-1', 'nan', str(2**64), '9' * 100):
            with self.subTest(value=value), self.assertRaises(ValueError):
                MODULE.verify(inventory(), report().replace('total: 3', 'total: ' + value), '1/4')


if __name__ == '__main__':
    unittest.main()
