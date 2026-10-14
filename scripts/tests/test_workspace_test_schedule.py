"""Protect exact scope ownership while avoiding an early CLI drain."""
import ast
import importlib.util
from types import SimpleNamespace
from pathlib import Path
import unittest

SOURCE = Path(__file__).resolve().parents[1] / 'workspace_test_schedule.py'
spec = importlib.util.spec_from_file_location('workspace_test_schedule', SOURCE)
schedule = importlib.util.module_from_spec(spec)
spec.loader.exec_module(schedule)


def case(package, target, kind='test'):
    return {'package': package, 'target': target, 'kind': kind,
            'selector': ['--' + kind, target]}


class WorkspaceScheduleTests(unittest.TestCase):
    def inventory(self):
        return [case('lila-engine', 'aot_intl'),
                case('lila-engine', 'aot_regexp'),
                case('lila-cli', 'cli'),
                case('lila-runtime', 'embedded_module_graph'),
                case('lila-intl', 'lila_intl', 'lib'),
                case('lila-cli', 'perf'),
                case('lila-front', 'doc', 'doc')]

    def test_later_independent_targets_dispatch_before_the_cli_drain(self):
        inventory = self.inventory()
        original = [dict(c, selector=list(c['selector'])) for c in inventory]
        ordered = schedule.execution_order(inventory, partition_cli=True)
        self.assertEqual([i for i, _ in ordered], [0, 1, 3, 4, 2, 5, 6])
        self.assertEqual(inventory, original)
        for index, scope in ordered:
            self.assertIs(scope, inventory[index])
        ordered[2][1]['state'] = 'passed'
        self.assertEqual(inventory[3]['state'], 'passed')
        self.assertNotIn('state', inventory[2])

    def test_unpartitioned_cli_keeps_independent_inventory_order(self):
        ordered = schedule.execution_order(self.inventory(), partition_cli=False)
        self.assertEqual([i for i, _ in ordered], list(range(7)))

    def test_partitioning_requires_the_exact_package_target_and_kind(self):
        self.assertTrue(schedule.is_partitioned_cli(case('lila-cli', 'cli'), True))
        for scope in [case('lila-engine', 'cli'), case('lila-cli', 'cache'),
                      case('lila-cli', 'cli', 'bin'), case('lila-cli', 'cli', 'lib')]:
            self.assertFalse(schedule.is_partitioned_cli(scope, True))
        self.assertFalse(schedule.is_partitioned_cli(case('lila-cli', 'cli'), False))

    def test_continuation_membership_keeps_its_existing_barrier_contract(self):
        cli = case('lila-cli', 'cli')
        self.assertFalse(schedule.is_barrier(cli))
        self.assertTrue(schedule.is_partitioned_cli(cli, True))
        for scope in [case('lila-cli', 'perf'), case('lila-cli', 'fake_suite_publication'),
                      case('lila-test262', 'differential_generated_campaign'),
                      case('lila-intl', 'doc', 'doc')]:
            self.assertTrue(schedule.is_barrier(scope))
        self.assertFalse(schedule.is_barrier(case('lila-engine', 'differential_corpus')))


    def test_opt_in_intl_runs_after_independent_scopes_as_an_idle_barrier(self):
        inventory = self.inventory()
        ordered = schedule.execution_order(inventory, True, ['lila-engine:aot_intl'])
        self.assertEqual([i for i, _ in ordered], [1, 3, 4, 0, 2, 5, 6])
        self.assertTrue(schedule.is_partitioned_native(inventory[0], False,
                                                      ['lila-engine:aot_intl']))
        self.assertFalse(schedule.is_barrier(inventory[0]))
        self.assertEqual(len(ordered), len(inventory))
        for index, scope in ordered:
            self.assertIs(scope, inventory[index])

    def test_exact_metadata_scope_is_required_and_duplicate_options_are_one_scope(self):
        inventory = self.inventory()
        validate = schedule.validate_partition_native_targets
        self.assertEqual(validate(inventory, ['lila-engine:aot_intl'] * 2, {}, True),
                         ['lila-engine:aot_intl'])
        for name in ['aot_intl', 'lila-engine:aot_intl*', 'lila-engine:absent',
                     'lila-cli:aot_intl']:
            with self.assertRaises(ValueError):
                validate(inventory, [name], {}, True)
        with self.assertRaises(ValueError):
            validate(inventory + [dict(inventory[0])], ['lila-engine:aot_intl'], {}, True)

    def test_complete_partitions_reject_filters_and_require_ignored_tests(self):
        validate = schedule.validate_partition_native_targets
        for include_ignored, filters in [(False, {}), (True, {'lila-engine:aot_intl': 'one'})]:
            with self.assertRaises(ValueError):
                validate(self.inventory(), ['lila-engine:aot_intl'], filters, include_ignored)
        self.assertEqual(validate(self.inventory(), [], {}, False), [])

    def test_rustdoc_timing_and_campaign_barriers_cannot_be_partitioned(self):
        inventory = [case('lila-engine', 'doc', 'doc'),
                     case('lila-cli', 'perf'), case('lila-cli', 'fake_suite_publication'),
                     case('lila-cli', 'differential_worker'),
                     case('lila-test262', 'differential_generated_campaign'),
                     case('lila-test262', 'differential_worker_execution'),
                     case('lila-test262', 'differential_rooted_snapshot_execution'),
                     case('lila-test262', 'example', 'example')]
        for scope in inventory:
            with self.assertRaises(ValueError):
                schedule.validate_partition_native_targets(inventory,
                    [scope['package'] + ':' + scope['target']], {}, True)

    def test_native_watcher_keeps_the_original_cli_and_native_allowances(self):
        self.assertEqual(schedule.native_stall_seconds('lila-cli'), 3600)
        for package in ['lila-engine', 'lila-intl', 'lila-test262']:
            self.assertEqual(schedule.native_stall_seconds(package), 900)



    def test_actual_runner_drains_existing_native_processes_before_partition_admission(self):
        runner = SOURCE.with_name('run_workspace_tests.py')
        tree = ast.parse(runner.read_text())
        loop = next(node for node in tree.body if isinstance(node, ast.For)
                    and isinstance(node.target, ast.Tuple)
                    and ast.unparse(node.target) == '(i, case)')
        # Execute the actual scheduling prefix, stopping before any label/build/I/O.
        prefix = []
        for node in loop.body:
            if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name)
                    and target.id == 'label' for target in node.targets):
                break
            prefix.append(node)
        active = {0: object(), 1: object()}
        calls = []
        def reap(wait=False):
            calls.append(wait)
            if wait:
                active.pop(next(iter(active)))
        scope = {'active': active, 'slots': [0, 1, 2, 3], 'reap': reap, 'case': case('lila-engine', 'aot_intl'),
                 'args': SimpleNamespace(partition_cli=True, continuation_phase='all',
                                         test_processes=4),
                 'partition_native_targets': ['lila-engine:aot_intl'],
                 'is_barrier': schedule.is_barrier,
                 'is_partitioned_native': schedule.is_partitioned_native}
        # The prefix contains a doc-only continue, so retain its actual loop context.
        harness = ast.For(target=loop.target, iter=ast.List(elts=[ast.Tuple(
            elts=[ast.Constant(value=0), ast.Name(id='case', ctx=ast.Load())], ctx=ast.Load())],
            ctx=ast.Load()), body=prefix, orelse=[])
        module = ast.fix_missing_locations(ast.Module(body=[harness], type_ignores=[]))
        exec(compile(module, str(runner), 'exec'), scope)
        self.assertTrue(scope['partitioned'])
        self.assertTrue(scope['barrier'])
        self.assertEqual(active, {})
        self.assertEqual(calls, [False, True, True])


if __name__ == '__main__':
    unittest.main()
