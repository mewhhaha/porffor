"""Protect exact scope ownership while avoiding an early CLI drain."""
import importlib.util
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


if __name__ == '__main__':
    unittest.main()
