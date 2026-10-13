"""Schedule complete native scopes without changing inventory ownership."""


def is_partitioned_cli(case, enabled):
    return (enabled and case['package'] == 'lila-cli'
            and case['target'] == 'cli' and case['kind'] == 'test')


def is_barrier(case):
    return (case['kind'] == 'doc'
            or (case['package'] == 'lila-cli' and case['target'] in (
                'perf', 'fake_suite_publication', 'differential_corpus',
                'differential_robustness', 'differential_scenario_pair',
                'differential_worker'))
            or (case['package'] == 'lila-test262' and case['target'] in (
                'differential_worker_execution', 'differential_generated_campaign',
                'differential_corpus', 'differential_robustness',
                'differential_rooted_snapshot_execution')))


def execution_order(cases, partition_cli):
    """Keep original indices and objects; drain independent work before barriers.

    Sorting is stable within both groups. Receipt inventories and continuation
    selection retain their original order; only dispatch order changes.
    """
    return sorted(enumerate(cases), key=lambda indexed: (
        is_barrier(indexed[1]) or is_partitioned_cli(indexed[1], partition_cli)))
