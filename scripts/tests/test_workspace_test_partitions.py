"""Pure Python controls; no Cargo or native test body is executed."""
import ast
from collections import Counter
import importlib.util
from pathlib import Path
import re
import unittest

SOURCE = Path(__file__).resolve().parents[1] / 'workspace_test_partitions.py'
TREE = ast.parse(SOURCE.read_text())
FUNCTIONS = [node for node in TREE.body if isinstance(node, ast.FunctionDef)
             and node.name in ('names', 'outcomes', 'weight', 'partition_groups')]
NAMESPACE = {'re': re}
exec(compile(ast.Module(body=FUNCTIONS, type_ignores=[]), str(SOURCE), 'exec'), NAMESPACE)


class NativePartitionProofTests(unittest.TestCase):
    def test_four_way_partition_is_an_exact_disjoint_union_of_293_qualified_ids(self):
        original = [f'aot_intl::control_{index:03}' for index in range(293)]
        groups = NAMESPACE['partition_groups'](original, 4)
        flattened = [name for group in groups for name in group]
        self.assertEqual(Counter(flattened), Counter(original))
        self.assertTrue(all(groups))
        self.assertEqual(sorted(map(len, groups)), [73, 73, 73, 74])

    def test_weight_only_changes_scheduling_never_membership_or_original_order(self):
        original = ['frontend_test262_subset::slow', 'intl::collator',
                    'frontend::inspect', 'known_failures::one', 'aot_intl::fast']
        saved = list(original)
        groups = NAMESPACE['partition_groups'](original, 4)
        self.assertEqual(original, saved)
        self.assertEqual(Counter(name for group in groups for name in group), Counter(original))
        self.assertEqual(groups, NAMESPACE['partition_groups'](list(reversed(original)), 4))

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

    def test_driver_rejects_an_attempt_to_raise_non_cli_stall_to_cli_allowance(self):
        checks = []
        for node in TREE.body:
            if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name)
                    and target.id == 'root' for target in node.targets):
                break
            if isinstance(node, (ast.Assign, ast.Assert)):
                checks.append(node)
        # Execute the actual argument-to-stock-policy binding only, without I/O.
        checks = [node for node in checks if 'args.qualified_target' in ast.unparse(node)
                  or 'stock_stall' in ast.unparse(node)
                  or 'separator' in ast.unparse(node)]
        for qualified, requested, expected in [('lila-cli:cli', None, 3600),
                                                ('lila-engine:aot_intl', None, 900),
                                                ('lila-engine:aot_intl', 900, 900)]:
            args = type('Arguments', (), {'qualified_target': qualified,
                                         'stall_seconds': requested})()
            scope = {'args': args, 'native_stall_seconds': lambda package: 3600 if package == 'lila-cli' else 900}
            exec(compile(ast.Module(body=checks, type_ignores=[]), str(SOURCE), 'exec'), scope)
            self.assertEqual(args.stall_seconds, expected)
        args = type('Arguments', (), {'qualified_target': 'lila-engine:aot_intl',
                                     'stall_seconds': 3600})()
        with self.assertRaises(AssertionError):
            exec(compile(ast.Module(body=checks, type_ignores=[]), str(SOURCE), 'exec'),
                 {'args': args, 'native_stall_seconds': lambda package: 900})


if __name__ == '__main__':
    unittest.main()
