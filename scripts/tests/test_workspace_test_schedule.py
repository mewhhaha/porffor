"""Protect exact scope ownership while avoiding an early CLI drain."""
import ast
import importlib.util
from types import SimpleNamespace
from pathlib import Path
import unittest
import os
import subprocess
import sys
import tempfile
import json

SOURCE = Path(__file__).resolve().parents[1] / 'workspace_test_schedule.py'
spec = importlib.util.spec_from_file_location('workspace_test_schedule', SOURCE)
schedule = importlib.util.module_from_spec(spec)
spec.loader.exec_module(schedule)


def case(package, target, kind='test'):
    return {'package': package, 'target': target, 'kind': kind,
            'selector': ['--' + kind, target]}


class WorkspaceScheduleTests(unittest.TestCase):
    def test_actual_plan_reports_compiler_cpu_scopes_after_independent_work(self):
        runner = SOURCE.with_name('run_workspace_tests.py')
        tree = ast.parse(runner.read_text())
        planning = next(node for node in tree.body if isinstance(node, ast.If)
                        and ast.unparse(node.test) == 'args.plan_only')
        assignment = next(node for node in planning.body if isinstance(node, ast.Assign)
                          and any(ast.unparse(target) == "result['command_plan']"
                                  for target in node.targets))
        inventory = [case('lila-engine', 'aot_intl'),
                     case('lila-engine', 'runtime_cache')]
        original = [dict(scope, selector=list(scope['selector'])) for scope in inventory]
        scope = dict(cases=inventory, native_cpu_targets=['lila-engine:aot_intl'],
                     partition_native_targets=[],
                     args=SimpleNamespace(partition_cli=False, include_ignored=True),
                     cargo_test_base=lambda _: ['cargo', 'test'],
                     group_key=lambda _: ('selected-native', '*'),
                     **{name: getattr(schedule, name) for name in (
                         'execution_order', 'is_barrier', 'is_partitioned_native',
                         'is_partitioned_cli', 'native_cpu_target')})
        expression = ast.Expression(body=assignment.value)
        rows = eval(compile(expression, str(runner), 'eval'), scope)
        self.assertEqual([row['target'] for row in rows], ['runtime_cache', 'aot_intl'])
        self.assertEqual([row['barrier'] for row in rows], [False, True])
        self.assertEqual(inventory, original)
        self.assertTrue(all(row['execution_arguments'] ==
                            ['--test-threads=1', '--include-ignored'] for row in rows))

    def test_native_compiler_targets_remain_complete_and_exclusive(self):
        inventory = self.inventory()
        validate = schedule.validate_native_cpu_targets
        selected = validate(inventory, ['lila-engine:aot_intl'] * 2, {}, True)
        self.assertEqual(selected, ['lila-engine:aot_intl'])
        ordered = schedule.execution_order(inventory, False, (), selected)
        self.assertEqual([i for i, _ in ordered], [1, 2, 3, 4, 0, 5, 6])
        for index, scope in ordered:
            self.assertIs(scope, inventory[index])
        for names, filters, ignored, partitions in [
                (selected, {}, True, selected), (selected, {}, False, []),
                (selected, {selected[0]: 'partial'}, True, []),
                (['lila-cli:perf'], {}, True, []),
                (['lila-front:doc'], {}, True, []),
                (['lila-engine:aot_*'], {}, True, [])]:
            with self.subTest(names=names, filters=filters, partitions=partitions):
                with self.assertRaises(ValueError):
                    validate(inventory, names, filters, ignored, partitions)

    def test_native_resource_receipt_rejects_changed_command_or_parallel_tests(self):
        command = ['/actual/test', '--test-threads=1', '--include-ignored']
        valid = dict(cpus=[2, 3], native_jobs=2, rayon_threads=2,
                     cargo_jobs=1, libtest_threads=1, command=command)
        schedule.validate_native_resources(valid, command, [1, 2, 3])
        for change in [dict(cpus=[]), dict(cpus=[2, 2]), dict(cpus=[0]),
                       dict(native_jobs=3), dict(rayon_threads=3),
                       dict(cargo_jobs=2), dict(libtest_threads=2),
                       dict(command=['/actual/test', 'filtered'])]:
            with self.subTest(change=change):
                with self.assertRaises(ValueError):
                    schedule.validate_native_resources(dict(valid, **change), command, [1, 2, 3])

    def test_publication_cpu_budget_keeps_one_complete_idle_scope(self):
        publication = case('lila-cli', 'fake_suite_publication')
        name = 'lila-cli:fake_suite_publication'
        inventory = self.inventory() + [publication]
        selected = schedule.validate_native_cpu_targets(
            inventory, [name, name], {}, True)
        self.assertEqual(selected, [name])
        self.assertTrue(schedule.native_cpu_target(publication, selected))
        self.assertTrue(schedule.is_barrier(publication))
        self._assert_actual_runner_drains([], selected, publication)
        for filters, ignored, partitions in [
                ({name: 'one'}, True, []), ({}, False, []), ({}, True, [name])]:
            with self.subTest(filters=filters, ignored=ignored, partitions=partitions):
                with self.assertRaises(ValueError):
                    schedule.validate_native_cpu_targets(
                        inventory, [name], filters, ignored, partitions)
        with self.assertRaises(ValueError):
            schedule.validate_partition_native_targets(inventory, [name], {}, True)
        for invalid in [case('lila-cli', 'fake_suite_publication', 'lib'),
                        case('lila-cli', 'perf'),
                        case('lila-cli', 'differential_worker')]:
            with self.subTest(invalid=invalid):
                with self.assertRaises(ValueError):
                    schedule.validate_native_cpu_targets(
                        [invalid], [invalid['package'] + ':' + invalid['target']], {}, True)

    def test_native_cpu_opt_in_excludes_performance_docs_and_independent_scopes(self):
        for scope in [case('lila-test262','differential_generated_campaign'),
                      case('lila-cli','differential_worker')]:
            self.assertTrue(schedule.differential_native_compilation(scope,'auto'))
            self.assertFalse(schedule.differential_native_compilation(scope,None))
        for scope in self.inventory()+[case('lila-engine','differential_corpus'),
                                      case('lila-cli','perf'),
                                      case('lila-cli','doc','doc')]:
            self.assertFalse(schedule.differential_native_compilation(scope,'auto'))

    def test_resource_prelude_records_actual_policy_and_preserves_exit_status(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'resource.json'
            command=[sys.executable,'-c','raise SystemExit(7)']
            result=subprocess.run([sys.executable,'-c',schedule.NATIVE_RESOURCE_EXEC,str(path),*command],
                                  env=dict(os.environ,LILA_JOBS='1',RUST_TEST_THREADS='1',
                                           RAYON_NUM_THREADS='1',CARGO_BUILD_JOBS='1'),
                                  capture_output=True)
            self.assertEqual(result.returncode,7,result.stderr)
            resource=json.loads(path.read_text())
            self.assertEqual(resource['command'],command)
            self.assertEqual(resource['cpus'],sorted(os.sched_getaffinity(0)))
            self.assertEqual(resource['native_jobs'],1)
            self.assertEqual(resource['libtest_threads'],1)
            self.assertEqual(resource['rayon_threads'],1)
            self.assertEqual(resource['cargo_jobs'],1)
            schedule.validate_native_resources(resource,command,os.sched_getaffinity(0))

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



    def test_actual_runner_drains_before_partition_or_compiler_cpu_admission(self):
        for partitions, compiler in [(['lila-engine:aot_intl'], []),
                                     ([], ['lila-engine:aot_intl'])]:
            with self.subTest(partitions=partitions, compiler=compiler):
                self._assert_actual_runner_drains(partitions,compiler)

    def _assert_actual_runner_drains(self,partitions,compiler,selected=None):
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
        scope = {'active': active, 'slots': [0, 1, 2, 3], 'reap': reap, 'case': selected or case('lila-engine', 'aot_intl'),
                 'args': SimpleNamespace(partition_cli=True, continuation_phase='all',
                                         test_processes=4),
                 'partition_native_targets': partitions,
                 'native_cpu_targets': compiler, 'native_cpu_target': schedule.native_cpu_target,
                 'is_barrier': schedule.is_barrier,
                 'is_partitioned_native': schedule.is_partitioned_native}
        # The prefix contains a doc-only continue, so retain its actual loop context.
        harness = ast.For(target=loop.target, iter=ast.List(elts=[ast.Tuple(
            elts=[ast.Constant(value=0), ast.Name(id='case', ctx=ast.Load())], ctx=ast.Load())],
            ctx=ast.Load()), body=prefix, orelse=[])
        module = ast.fix_missing_locations(ast.Module(body=[harness], type_ignores=[]))
        exec(compile(module, str(runner), 'exec'), scope)
        self.assertEqual(scope['partitioned'],bool(partitions))
        self.assertTrue(scope['barrier'])
        self.assertEqual(active, {})
        self.assertEqual(calls, [False, True, True])

    def test_idle_builds_drain_only_uncached_work_before_storage_admission(self):
        runner = SOURCE.with_name('run_workspace_tests.py')
        tree = ast.parse(runner.read_text())
        loop = next(node for node in tree.body if isinstance(node, ast.For)
                    and isinstance(node.target, ast.Tuple)
                    and ast.unparse(node.target) == '(i, case)')
        prefix = []
        for node in loop.body:
            if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name)
                    and target.id == 'start' for target in node.targets):
                break
            prefix.append(node)
        for requested, cached, remaining in [(True, False, 0), (True, True, 2),
                                              (False, False, 2)]:
            with self.subTest(idle_builds=requested, cached=cached):
                active = {0: object(), 1: object()}
                calls = []
                admissions = []
                def reap(wait=False):
                    calls.append(wait)
                    if wait:
                        active.pop(next(iter(active)))
                def admit(*arguments, **options):
                    admissions.append((len(active), options['cached_native']))
                    return False  # Stop before any real Cargo or filesystem work.
                selected = case('lila-front', 'independent')
                scope = {'active': active, 'slots': [0, 1, 2, 3], 'reap': reap,
                         'case': selected, 'artifact_cohorts': {'group': {}} if cached else {},
                         'args': SimpleNamespace(partition_cli=False, continuation_phase='all',
                                                 idle_builds=requested),
                         'partition_native_targets': [], 'native_cpu_targets': [],
                         'native_cpu_target': schedule.native_cpu_target,
                         'is_barrier': schedule.is_barrier,
                         'is_partitioned_native': schedule.is_partitioned_native,
                         'cargo_test_base': lambda selected: ['cargo', 'test'],
                         'group_key': lambda selected: 'group', 'admit_build_space': admit,
                         'out': Path('/unused'), 'time': __import__('time')}
                harness = ast.For(target=loop.target, iter=ast.List(elts=[ast.Tuple(
                    elts=[ast.Constant(value=0), ast.Name(id='case', ctx=ast.Load())],
                    ctx=ast.Load())], ctx=ast.Load()), body=prefix, orelse=[])
                module = ast.fix_missing_locations(ast.Module(body=[harness], type_ignores=[]))
                exec(compile(module, str(runner), 'exec'), scope)
                self.assertEqual(admissions, [(remaining, cached)])
                self.assertEqual(len(active), remaining)
                self.assertEqual(calls, [False, True, True] if remaining == 0 else [False])
                self.assertEqual(selected['selector'], ['--test', 'independent'])
                self.assertEqual('idle_build_wait' in selected, remaining == 0)


if __name__ == '__main__':
    unittest.main()
