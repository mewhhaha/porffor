"""Native-replay reconciliation and the conservative standalone-Wasm scope split."""
import importlib.util
import json
from pathlib import Path
import sys
import tarfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
import test262_chart_evidence as evidence


class ChartReconciliationTests(unittest.TestCase):
    def test_only_reviewed_primary_targets_enter_scope(self):
        self.assertEqual(evidence.classify('language/example.js', '/*---\nesid: sec-performeval\n---*/\neval(source);'),
                         ('sec-performeval', 'dynamic-eval-primary-target'))
        self.assertEqual(evidence.classify('other.js', '/*---\nesid: sec-performeval-rules-in-initializer\n---*/'),
                         ('sec-performeval-rules-in-initializer', 'dynamic-eval-primary-target'))
        for source in ('/*---\nesid: sec-eval-x\n---*/\nverifyProperty(eval, "name", {});',
                       '/*---\nesid: sec-function.prototype.bind\n---*/\nFunction("return 1");',
                       '/*---\nesid: sec-createdynamicfunction\n---*/\n// constructor realm setup',
                       '// eval(x); new Function(x);'):
            self.assertIsNone(evidence.classify('unrelated.js', source))
        for name in evidence.EVAL_NON_SOURCE_CASES:
            self.assertIsNone(evidence.classify(name, '/*---\nesid: sec-performeval\n---*/'))
        path = 'built-ins/Function/private-identifiers-not-empty.js'
        self.assertEqual(evidence.classify(path, '/*---\nesid: sec-createdynamicfunction\n---*/'),
                         ('sec-createdynamicfunction', 'dynamic-function-source-parser'))
        with self.assertRaisesRegex(ValueError, 'specification target'):
            evidence.classify(path, '/*---\nesid: sec-function.prototype.bind\n---*/')

    def snapshot(self):
        return dict(pinned_revisions={'test262': 'pin'}, execution_backend='wasm-aot',
            completed_test_ids=['sloppy-script:a.js', 'strict-script:a.js'], total=2, passed=1,
            failures=[dict(test_id='strict-script:a.js', outcome='Bug')],
            counts_per_outcome={'Success': 1, 'Bug': 1, 'Crash': 0, 'NotImplemented': 0})

    def test_replay_must_match_every_original_failure_and_original_pin(self):
        ids = ['sloppy-script:a.js', 'strict-script:a.js', 'sloppy-script:passing.js']
        failures = dict.fromkeys(ids[:2], 'Bug')
        result = evidence.checked_replay(ids, failures, self.snapshot(), {'test262': 'pin'})
        self.assertEqual(result, {'sloppy-script:a.js': 'Success', 'strict-script:a.js': 'Bug'})
        for change in (dict(completed_test_ids=ids[:1], total=1),
                       dict(completed_test_ids=[ids[0], ids[0]]),
                       dict(pinned_revisions={'test262': 'worktree-errata'}),
                       dict(passed=2), dict(execution_backend='spec-exec')):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evidence.checked_replay(ids, failures, dict(self.snapshot(), **change), {'test262': 'pin'})

    def test_scope_partitions_successes_preserves_denominator_and_cannot_hide_failures(self):
        outcomes = {'a': 'Success', 'b': 'Success', 'c': 'Bug', 'd': 'NotImplemented'}
        result = evidence.projection(outcomes, {'a'})
        self.assertEqual(result, dict(Success=1, Bug=1, Crash=0, NotImplemented=1, OutOfScope=1))
        self.assertEqual(sum(result.values()), len(outcomes))
        for excluded in ({'c'}, {'d'}, {'absent'}):
            with self.assertRaises(ValueError):
                evidence.projection(outcomes, excluded)

    def test_actual_archived_evidence_has_exact_reviewed_scope_inventory(self):
        # Keep this historical fixture independent of the next full publication.
        archive_path = 'test262/snapshots/current-test262-20261020.tar.gz'
        with tarfile.open(ROOT / archive_path, 'r:gz') as archive:
            publication = json.load(archive.extractfile('snapshots/published-status-wasm-aot.json'))
        with patch.object(evidence, 'load_publication', return_value=(publication, archive_path)):
            record = evidence.build(ROOT)
        self.assertEqual(record['scope_source_paths'], 221)
        self.assertEqual(record['scope_execution_count'], 429)
        self.assertEqual(record['scoped_chart_counts'], dict(Success=102561, Bug=4, Crash=0, NotImplemented=0, OutOfScope=429))
        self.assertFalse(record['full_current_source_rerun'])
        self.assertFalse(evidence.EVAL_NON_SOURCE_CASES & {r['path'] for r in record['scope_rows']})

    def test_reconciled_svg_has_numeric_blue_bar_and_clear_mixed_checkpoint_caption(self):
        spec = importlib.util.spec_from_file_location('chart', ROOT / 'scripts/generate-test-graph.py')
        graph = importlib.util.module_from_spec(spec);spec.loader.exec_module(graph)
        publication = dict(real_suite=dict(total=10, passed=6, refresh_date='2026-10-20',
            counts_per_outcome=[dict(label='Success', count=6),dict(label='Bug', count=4),
                                dict(label='Crash', count=0),dict(label='NotImplemented', count=0)],
            pinned_revisions={'test262': 'a'*40}, compiler_identity={'source_revision': {'kind':'git-commit','commit':'b'*40}}))
        reconciliation = dict(date='2026-10-21', scoped_chart_counts=dict(Success=6, Bug=1, Crash=0, NotImplemented=0, OutOfScope=3))
        self.assertEqual([row[2] for row in graph.categories(publication, reconciliation)], [6,1,0,3])
        self.assertIn('not had a fresh full rerun', graph.render(publication, reconciliation=reconciliation))
        svg=graph.render_svg(publication, reconciliation)
        self.assertIn('id="out-of-scope" data-count="3"', svg)
        self.assertIn('not a fresh full rerun', svg)
        self.assertNotIn('count unknown', svg)
        reconciliation['scoped_chart_counts']['Success'] += 1
        with self.assertRaisesRegex(ValueError, 'denominator'):
            graph.categories(publication, reconciliation)


if __name__ == '__main__':
    unittest.main()
