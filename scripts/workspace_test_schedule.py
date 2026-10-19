"""Schedule complete native scopes without changing inventory ownership."""


def is_partitioned_cli(case, enabled):
    return (enabled and case['package'] == 'lila-cli'
            and case['target'] == 'cli' and case['kind'] == 'test')


def native_stall_seconds(package):
    """Keep the original whole-target watcher allowance in each child."""
    return 3600 if package == 'lila-cli' else 900


def is_partitioned_native(case, partition_cli, native_targets=()):
    return (is_partitioned_cli(case, partition_cli)
            or (case['kind'] in ('lib', 'bin', 'test')
                and case['package'] + ':' + case['target'] in native_targets))


def validate_partition_native_targets(inventory, requested, native_filters,
                                      include_ignored):
    """Admit only exact available metadata scopes with complete inventories."""
    selected = sorted(set(requested))
    if selected and not include_ignored:
        raise ValueError('Complete native partitions require --include-ignored')
    for name in selected:
        matches = [case for case in inventory
                   if case['package'] + ':' + case['target'] == name]
        if len(matches) != 1:
            raise ValueError(f'Native partition target must name one available '
                             f'Cargo metadata scope: {name}')
        if matches[0]['kind'] not in ('lib', 'bin', 'test') or is_barrier(matches[0]):
            raise ValueError(f'Native partitions exclude documentation, timing '
                             f'and campaign barriers: {name}')
        if name in native_filters:
            raise ValueError(f'Native partitions require an unfiltered whole '
                             f'target: {name}')
    return selected


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


def differential_native_compilation(case, setting):
    """Only existing idle-exclusive differential scopes can opt into more CPUs."""
    return (setting == 'auto' and case['kind'] == 'test'
            and case['target'].startswith('differential_') and is_barrier(case))


NATIVE_RESOURCE_EXEC = '''import json, os, sys
from pathlib import Path
resource = {"cpus": sorted(os.sched_getaffinity(0)),
            "native_jobs": int(os.environ["LILA_JOBS"]),
            "libtest_threads": int(os.environ["RUST_TEST_THREADS"]),
            "command": sys.argv[2:]}
Path(sys.argv[1]).write_text(json.dumps(resource, sort_keys=True) + "\\n")
os.execvpe(sys.argv[2], sys.argv[2:], os.environ)
'''


def execution_order(cases, partition_cli, native_targets=()):
    """Keep original indices and objects; drain independent work before barriers.

    Sorting is stable within both groups. Receipt inventories and continuation
    selection retain their original order; only dispatch order changes.
    """
    return sorted(enumerate(cases), key=lambda indexed: (
        is_barrier(indexed[1])
        or is_partitioned_native(indexed[1], partition_cli, native_targets)))
