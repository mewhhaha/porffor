"""Pure Python controls; no Cargo or native test body is executed."""
import ast
from collections import Counter
import hashlib
import importlib.util
import os
from pathlib import Path
import re
import tempfile
import unittest

import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

SOURCE = Path(__file__).resolve().parents[1] / 'workspace_test_partitions.py'
TREE = ast.parse(SOURCE.read_text())
FUNCTIONS = [node for node in TREE.body if isinstance(node, ast.FunctionDef)
             and node.name in ('names', 'outcomes', 'weight', 'partition_groups', 'partition_label')]
NAMESPACE = {'re': re, 'hashlib': hashlib, 'os': os, 'Path': Path}
from native_partition_costs import measured_weight
from native_test_protocol import test_names, summary_counts
NAMESPACE.update(measured_weight=measured_weight, test_names=test_names,
                 summary_counts=summary_counts)
exec(compile(ast.Module(body=FUNCTIONS, type_ignores=[]), str(SOURCE), 'exec'), NAMESPACE)

ENGINE_UTF16_OUTLIER = (
    'runtime_regexp_compiler_tests::raw_utf16::'
    'literal_prepared_and_static_constructor_patterns_share_actual_utf16_units')


class NativePartitionProofTests(unittest.TestCase):
    def test_four_way_partition_is_an_exact_disjoint_union_of_293_qualified_ids(self):
        original = [f'aot_intl::control_{index:03}' for index in range(293)]
        groups = NAMESPACE['partition_groups'](original, 4)
        flattened = [name for group in groups for name in group]
        self.assertEqual(Counter(flattened), Counter(original))
        self.assertTrue(all(groups))
        self.assertEqual(sorted(map(len, groups)), [73, 73, 73, 74])

    def test_engine_outlier_runs_alone_in_a_complete_893_id_partition(self):
        # Match the current Engine inventory size without depending on build artifacts.
        original = [ENGINE_UTF16_OUTLIER] + [f'engine::control_{index:03}' for index in range(892)]
        saved = list(original)
        groups = NAMESPACE['partition_groups'](original, 4)
        flattened = [name for group in groups for name in group]
        self.assertEqual(Counter(flattened), Counter(original))
        self.assertEqual(original, saved)
        self.assertTrue(all(groups))
        outlier_group = next(group for group in groups if ENGINE_UTF16_OUTLIER in group)
        self.assertEqual(outlier_group, [ENGINE_UTF16_OUTLIER])
        other_counts = [len(group) for group in groups if group is not outlier_group]
        self.assertEqual(sum(other_counts), 892)
        self.assertLessEqual(max(other_counts) - min(other_counts), 1)
        self.assertEqual(groups, NAMESPACE['partition_groups'](list(reversed(original)), 4))

    def test_engine_outlier_weight_is_exact_and_other_cost_classes_are_unchanged(self):
        self.assertGreater(NAMESPACE['weight'](ENGINE_UTF16_OUTLIER), 3)
        for name, expected in [
            (ENGINE_UTF16_OUTLIER + '_other', 3),
            ('other::' + ENGINE_UTF16_OUTLIER, 3),
            ('runtime_regexp_compiler_tests::raw_utf16::other', 3),
            ('frontend_test262_subset::slow', 800),
            ('intl::collator', 25),
            ('frontend::inspect', 1),
            ('known_failures::one', 1),
            ('aot_intl::control', 3),
        ]:
            self.assertEqual(NAMESPACE['weight'](name), expected, name)

    def test_weight_only_changes_scheduling_never_membership_or_original_order(self):
        original = ['frontend_test262_subset::slow', 'intl::collator',
                    'frontend::inspect', 'known_failures::one', 'aot_intl::fast']
        saved = list(original)
        groups = NAMESPACE['partition_groups'](original, 4)
        self.assertEqual(original, saved)
        self.assertEqual(Counter(name for group in groups for name in group), Counter(original))
        self.assertEqual(groups, NAMESPACE['partition_groups'](list(reversed(original)), 4))

    def test_measured_cli_outliers_balance_without_dropping_any_of_855_ids(self):
        measured = __import__('json').loads(SOURCE.with_name('native-partition-costs.json').read_text())['observations'][0]['seconds']
        original = list(measured) + [f'array::ordinary_{index:03}'
                                    for index in range(855 - len(measured))]
        saved = list(original)
        groups = NAMESPACE['partition_groups'](original, 3)
        self.assertEqual(Counter(name for group in groups for name in group), Counter(original))
        self.assertEqual(original, saved)
        self.assertEqual(groups, NAMESPACE['partition_groups'](list(reversed(original)), 3))
        # Scheduling must account for the projection and matrix tests that
        # previously accumulated on one partition behind a cheap prefix weight.
        costs = [sum(NAMESPACE['weight'](name) for name in group) for group in groups]
        self.assertLessEqual(max(costs) - min(costs), 3)
        largest = sorted(measured, key=measured.get, reverse=True)[:3]
        owners = [next(index for index, group in enumerate(groups) if name in group)
                  for name in largest]
        self.assertEqual(len(set(owners)), 3)

    def test_cli_cost_estimates_match_exact_ids_and_never_change_unknown_tests(self):
        name = 'intl::datetime_cli_projection_matches_sdk_and_runs_six_selected_components'
        self.assertGreater(NAMESPACE['weight'](name), NAMESPACE['weight']('intl::ordinary'))
        self.assertEqual(NAMESPACE['weight'](name + '_new'), 25)
        self.assertEqual(NAMESPACE['weight']('other::' + name), 3)
        self.assertEqual(NAMESPACE['weight']('frontend::unmeasured'), 1)
        for estimate in __import__('json').loads(SOURCE.with_name('native-partition-costs.json').read_text())['observations'][0]['seconds'].values():
            self.assertIs(type(estimate), int)
            self.assertGreaterEqual(estimate, 30)

    def test_duplicate_empty_and_oversubscribed_inventories_are_rejected(self):
        for names, count in [([], 1), (['one', 'one'], 1), (['one'], 2),
                             (['one'], 0), ([str(i) for i in range(10)], 5)]:
            with self.assertRaises(AssertionError):
                NAMESPACE['partition_groups'](names, count)

    def test_inventory_parser_keeps_exact_qualified_ids_and_excludes_benchmarks(self):
        self.assertEqual(NAMESPACE['names']('a::b: test\nc::d: test\ne::f: benchmark\n2 tests\n'),
                         ['a::b', 'c::d'])

    def test_native_success_and_failure_are_retained_with_summary_agreement(self):
        text = ('test a::one ... ok\ntest b::two ... FAILED\n'
                'test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;\n')
        actual, counts = NAMESPACE['outcomes'](text)
        self.assertEqual(actual, {'a::one': 'ok', 'b::two': 'FAILED'})
        self.assertEqual(counts, {'passed': 1, 'failed': 1, 'ignored': 0})

    def test_bad_summary_ignored_duplicate_and_missing_native_results_are_rejected(self):
        cases = [
            'test a ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored;\n',
            'test a ... ignored\ntest result: ok. 0 passed; 0 failed; 1 ignored;\n',
            'test a ... ok\ntest a ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored;\n',
            'test a ... unfinished\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n',
            'test a ... ok\n',
        ]
        for text in cases:
            with self.assertRaises(AssertionError):
                NAMESPACE['outcomes'](text)

    def test_each_actual_child_launch_uses_one_cpu_one_thread_and_stock_stall(self):
        loop = next(node for node in TREE.body if isinstance(node, ast.For)
                    and isinstance(node.target, ast.Name) and node.target.id == 'row'
                    and isinstance(node.iter, ast.Subscript))
        launch = next(node for node in loop.body if isinstance(node, ast.Assign)
                      and any(isinstance(target, ast.Name) and target.id == 'launch'
                              for target in node.targets))
        for stall in [900, 3600]:
            scope = {'args': type('Arguments', (), {'stall_seconds': stall})(),
                     'row': {'cpu': 3, 'command': ['/native', '--exact', 'a::one',
                                                  '--test-threads=1', '--include-ignored']},
                     'label': 'proof'}
            exec(compile(ast.Module(body=[launch], type_ignores=[]), str(SOURCE), 'exec'), scope)
            command = scope['launch']
            self.assertEqual(command[:3], ['taskset', '-c', '3'])
            self.assertEqual(command[command.index('--stall') + 1], str(stall))
            self.assertEqual(command[-3:], ['a::one', '--test-threads=1', '--include-ignored'])
            self.assertIn('--cloud', command)
            self.assertNotIn('--cloud-cpus', command)

    def test_watched_logs_keep_the_resolved_run_owner_and_partition_identity(self):
        loop = next(node for node in TREE.body if isinstance(node, ast.For)
                    and isinstance(node.target, ast.Name) and node.target.id == 'row'
                    and isinstance(node.iter, ast.Subscript))
        assignments = [node for node in loop.body if isinstance(node, ast.Assign)
                       and any((isinstance(target, ast.Name) and target.id in ('label', 'launch'))
                               or (isinstance(target, ast.Subscript)
                                   and ast.unparse(target) == "row['transcript_log']")
                               for target in node.targets)]
        actual = compile(ast.Module(body=assignments, type_ignores=[]), str(SOURCE), 'exec')
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            paths = []
            labels = {}
            for parent in ('default-run', 'all-features-run'):
                output = base / parent / '0049-scriptw-cli-cli-partitions'
                for index in range(4):
                    scope = dict(NAMESPACE, args=type('Arguments', (), {
                        'output': output, 'stall_seconds': 3600,
                    })(), row={'index': index, 'cpu': index,
                               'command': ['/native', '--exact', 'a::one',
                                           '--test-threads=1', '--include-ignored']})
                    exec(actual, scope)
                    launch = scope['launch']
                    label = launch[launch.index('--label') + 1]
                    transcript = Path(scope['row']['transcript_log'])
                    self.assertEqual(transcript, Path('target/watched') / f'{label}.log')
                    paths.append(transcript)
                    labels[(parent, index)] = label
            self.assertEqual(len(set(paths)), 8, 'Fresh run owners must not overwrite prior transcripts')
            output = base / 'default-run' / '0049-scriptw-cli-cli-partitions'
            alias = output / '..' / output.name
            self.assertEqual(NAMESPACE['partition_label'](alias, 0), labels[('default-run', 0)])
            self.assertEqual(NAMESPACE['partition_label'](output, 0), labels[('default-run', 0)])
            long = NAMESPACE['partition_label'](base / ('long owner ' * 18), 3)
            self.assertRegex(long, r'^[A-Za-z0-9_.-]+$')
            self.assertLessEqual(len(os.fsencode(long + '.log')), 255)

    def test_driver_rejects_an_attempt_to_raise_non_cli_stall_to_cli_allowance(self):
        checks = []
        for node in TREE.body:
            if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name)
                    and target.id == 'root' for target in node.targets):
                break
            if isinstance(node, (ast.Assign, ast.If)):
                checks.append(node)
        # Execute the actual argument-to-stock-policy binding only, without I/O.
        checks = [node for node in checks if 'args.qualified_target' in ast.unparse(node)
                  or 'stock_stall' in ast.unparse(node)
                  or 'separator' in ast.unparse(node)]
        for qualified, requested, expected in [('scriptw-cli:cli', None, 3600),
                                                ('scriptw-engine:aot_intl', None, 900),
                                                ('scriptw-engine:aot_intl', 900, 900)]:
            args = type('Arguments', (), {'qualified_target': qualified,
                                         'stall_seconds': requested})()
            scope = {'args': args, 'native_stall_seconds': lambda package: 3600 if package == 'scriptw-cli' else 900}
            exec(compile(ast.Module(body=checks, type_ignores=[]), str(SOURCE), 'exec'), scope)
            self.assertEqual(args.stall_seconds, expected)
        args = type('Arguments', (), {'qualified_target': 'scriptw-engine:aot_intl',
                                     'stall_seconds': 3600})()
        with self.assertRaises(AssertionError):
            exec(compile(ast.Module(body=checks, type_ignores=[]), str(SOURCE), 'exec'),
                 {'args': args, 'native_stall_seconds': lambda package: 900})


if __name__ == '__main__':
    unittest.main()
