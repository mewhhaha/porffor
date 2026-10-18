"""Reject graph drift and count contamination in narrowed Cargo verification."""

import copy
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[1] / 'workspace_feature_roots.py'
spec = importlib.util.spec_from_file_location('workspace_feature_roots', SOURCE)
roots = importlib.util.module_from_spec(spec)
spec.loader.exec_module(roots)


def case(package, target, kind='test'):
    return {'package': package, 'target': target, 'kind': kind,
            'selector': ['--lib'] if kind == 'lib' else ['--test', target]}


def graph():
    return {'roots': [0], 'units': [
        {'pkg_id': 'engine', 'mode': 'test', 'target': {'name': 'entry', 'kind': ['test']},
         'features': [], 'profile': {'opt_level': '0'},
         'dependencies': [{'index': 1, 'extern_crate_name': 'parser', 'public': False}]},
        {'pkg_id': 'parser', 'mode': 'build', 'target': {'name': 'parser', 'kind': ['lib']},
         'features': ['annex-b'], 'profile': {'opt_level': '2'}, 'dependencies': []},
    ]}


class FeatureRootTests(unittest.TestCase):
    def test_selected_owner_avoids_unrequested_libtest_siblings(self):
        selected = [case('ir', 'ir', 'lib'), case('engine', 'entry')]
        command = roots.command(selected[1], selected, ['intl'], True, [])
        self.assertNotIn('--lib', command)
        self.assertNotIn('--workspace', command)
        self.assertEqual([command[i+1] for i,v in enumerate(command) if v == '-p'],
                         ['engine', roots.PACKAGE])
        self.assertNotEqual(roots.build_key(selected[0], ['intl'], True),
                            roots.build_key(selected[1], ['intl'], True))
        self.assertIsNone(roots.build_key(case('intl', 'intl', 'lib'), ['intl'], True))

    def test_recursive_identity_detects_transitive_features_profiles_and_edges(self):
        original = graph(); expected = roots.closure_signature(original, 0)
        for mutate in (
            lambda g: g['units'][1]['features'].append('rc'),
            lambda g: g['units'][1]['profile'].update(opt_level='0'),
            lambda g: g['units'][0]['dependencies'][0].update(public=True),
            lambda g: g['units'][1].update(pkg_id='different-parser'),
        ):
            changed = copy.deepcopy(original); mutate(changed)
            self.assertNotEqual(roots.closure_signature(changed, 0), expected)

    def test_graph_array_positions_do_not_change_recursive_identity(self):
        original = graph(); changed = graph()
        changed['units'].reverse(); changed['roots'] = [1]
        changed['units'][1]['dependencies'][0]['index'] = 0
        self.assertEqual(roots.closure_signature(original, 0), roots.closure_signature(changed, 1))
        self.assertEqual(roots.selected_root(changed, 'engine', 'entry', 'test'), 1)
        with self.assertRaises(ValueError):roots.selected_root(changed, 'parser', 'entry', 'test')

    def test_cycles_and_ambiguous_roots_fail_closed(self):
        cyclic = graph(); cyclic['units'][1]['dependencies'] = [{'index': 0}]
        with self.assertRaises(ValueError):roots.closure_signature(cyclic, 0)
        duplicate = graph(); duplicate['roots'].append(0)
        with self.assertRaises(ValueError):roots.selected_root(duplicate, 'engine', 'entry', 'test')

    def auxiliary(self, inventory, execution):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / 'aux'; binary.write_bytes(b'exact-selected-Cargo-output')
            replies = [SimpleNamespace(returncode=0, stdout=text.encode(), stderr=b'')
                       for text in (inventory, execution)]
            with patch.object(roots.subprocess, 'run', side_effect=replies):
                return roots.verify_auxiliary(binary, Path(directory) / 'proof', {})

    def test_empty_auxiliary_is_recorded_outside_the_suite_denominator(self):
        result = self.auxiliary('0 tests, 0 benchmarks\n',
            'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n')
        self.assertFalse(result['included_in_suite_denominator'])
        self.assertEqual(result['native_ids'], 0)

    def test_auxiliary_cannot_hide_a_test_benchmark_ignore_or_execution(self):
        empty = 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n'
        for inventory in ('hidden: test\n1 test, 0 benchmarks\n', '0 tests, 1 benchmark\n'):
            with self.assertRaises(ValueError):self.auxiliary(inventory, empty)
        for bad in ('1 passed', '1 ignored', '1 measured', '1 filtered out'):
            field = bad.split(' ', 1)[1]
            with self.assertRaises(ValueError):self.auxiliary('0 tests, 0 benchmarks\n', empty.replace('0 '+field, bad))
        with self.assertRaises(ValueError):
            self.auxiliary('0 tests, 0 benchmarks\n', empty + empty.replace('0 passed', '1 passed'))


if __name__ == '__main__':
    unittest.main()
