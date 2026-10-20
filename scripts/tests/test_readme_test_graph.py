import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('test_graph', ROOT / 'scripts/generate-test-graph.py')
GRAPH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GRAPH)


class RecordedTestDiagram(unittest.TestCase):
    def setUp(self):
        self.inventory = json.loads((ROOT / 'docs/rust-rewrite/current-inventory-20261020.json').read_bytes())
        self.publication = json.loads(GRAPH.read_committed_or_local(ROOT, 'test262/snapshots/published-status-wasm-aot.json'))

    def test_unrun_tests_never_become_not_implemented(self):
        rendered = GRAPH.render(self.inventory, self.publication)
        self.assertIn('Not run yet<br/>2,094', rendered)
        self.assertIn('NotImplemented<br/>1,125', rendered)
        self.assertIn('Historical Test262 publication', rendered)
        self.assertIn('before the scriptw rename', rendered)

    def test_inconsistent_totals_and_unrecognized_outcomes_are_rejected(self):
        self.inventory['current_af']['remaining_uncompleted_native_tests'] += 1
        with self.assertRaisesRegex(ValueError, 'native totals disagree'):
            GRAPH.render(self.inventory, self.publication)
        self.inventory['current_af']['remaining_uncompleted_native_tests'] -= 1
        self.publication['real_suite']['counts_per_outcome'].append({'label': 'Unclassified', 'count': 1})
        with self.assertRaisesRegex(ValueError, 'unknown published outcomes'):
            GRAPH.render(self.inventory, self.publication)
