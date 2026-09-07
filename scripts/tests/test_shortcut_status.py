"""Failure controls for the derived, non-conformance shortcut status report."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'generate-shortcut-status.py'
SPEC = importlib.util.spec_from_file_location('shortcut_status', SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def row(number=1, kind='semantic-shortcut', owner='T13', removal='T13', reason='dynamic-source-substitution'):
    return '\t'.join([f'source-text-predicate/example/{number:03}', 'a' * 64, kind, owner, removal, reason])


def inventory(legitimate=0, diagnostic=0, semantic=1):
    return ('# Inventory\n\n## Classification summary\n\n'
            '| Classification | Count |\n| --- | ---: |\n'
            f'| `legitimate-harness-adaptation` | {legitimate} |\n'
            f'| `diagnostic-instrumentation` | {diagnostic} |\n'
            f'| `semantic-shortcut` | {semantic} |\n\n## Other section\n')


class ShortcutStatusTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for path, text in ((MODULE.LEDGER, row() + '\n'),
                           (MODULE.INVENTORY, inventory()),
                           (MODULE.SOURCE, '// source identity\n')):
            destination = self.root / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(text, encoding='utf-8')

    def test_deterministic_report_explains_its_denominator(self):
        report = MODULE.render(self.root)
        self.assertEqual(report, MODULE.render(self.root))
        self.assertIn('| **Total** | **1** |', report)
        self.assertIn('| T13 | 1 |', report)
        self.assertIn('not a Test262 pass rate', report)

    def test_all_classifications_count_but_only_semantic_tasks_group(self):
        (self.root / MODULE.LEDGER).write_text('\n'.join([
            row(), row(2, 'legitimate-harness-adaptation', 'T03', 'T03', 'suite-selection'),
            row(3, 'diagnostic-instrumentation', 'T17', 'T17', 'artifact-routing'),
        ]))
        (self.root / MODULE.INVENTORY).write_text(inventory(1, 1, 1))
        report = MODULE.render(self.root)
        self.assertIn('| **Total** | **3** |', report)
        section = report.split('## Semantic shortcuts by removal task')[1].split('## Input identity')[0]
        self.assertNotIn('| T03 |', section)
        self.assertNotIn('| T17 |', section)

    def test_rejects_empty_or_duplicate_ledger(self):
        for text in ('', '# header\n', row() + '\n' + row()):
            with self.subTest(text=text), self.assertRaises(ValueError):
                MODULE.ledger_rows(text)

    def test_rejects_invalid_closed_domains_and_extra_columns(self):
        invalid = [row() + '\textra', row().replace('a' * 64, 'bad'),
                   row(kind='unknown'), row(owner='T26-unclassified'), row(removal='T99'),
                   row(reason='suite-selection'), row().replace('/001', '/1')]
        for text in invalid:
            with self.subTest(text=text), self.assertRaises(ValueError):
                MODULE.ledger_rows(text)

    def test_rejects_missing_duplicate_unknown_or_malformed_inventory_rows(self):
        good = inventory()
        bad = [good.replace('| `semantic-shortcut` | 1 |\n', ''),
               good.replace('| `semantic-shortcut` | 1 |', '| `semantic-shortcut` | 1 |\n| `semantic-shortcut` | 1 |'),
               good.replace('`semantic-shortcut`', '`unknown`'),
               good.replace('| 1 |', '| 01 |'), good + '## Classification summary\n',
               good.replace('| Count |', '| Value |')]
        for text in bad:
            with self.subTest(text=text), self.assertRaises(ValueError):
                MODULE.inventory_counts(text)

    def test_rejects_ledger_inventory_disagreement(self):
        (self.root / MODULE.INVENTORY).write_text(inventory(semantic=2))
        with self.assertRaisesRegex(ValueError, 'counts differ'):
            MODULE.render(self.root)

    def test_source_bytes_change_input_identity(self):
        before = MODULE.render(self.root)
        (self.root / MODULE.SOURCE).write_text('// changed source\n')
        self.assertNotEqual(before, MODULE.render(self.root))

    def test_missing_source_is_not_treated_as_zero_observations(self):
        (self.root / MODULE.SOURCE).unlink()
        with self.assertRaises(OSError):
            MODULE.render(self.root)

    def test_cli_generation_check_and_stale_report_failure(self):
        def run(*arguments):
            return subprocess.run([sys.executable, str(SCRIPT), '--root', str(self.root), *arguments],
                                  capture_output=True, text=True, timeout=10)
        self.assertNotEqual(run('--check').returncode, 0)
        self.assertEqual(run().returncode, 0)
        self.assertEqual(run('--check').returncode, 0)
        report = self.root / MODULE.OUTPUT
        original = report.read_bytes()
        report.write_text('manually edited counts\n')
        failed = run('--check')
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn('stale', failed.stderr)
        self.assertEqual(report.read_text(), 'manually edited counts\n')
        self.assertEqual(run().returncode, 0)
        self.assertEqual(report.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
