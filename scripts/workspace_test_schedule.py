"""Schedule complete native scopes without changing inventory ownership."""


def is_partitioned_cli(case, enabled):
    return (enabled and case['package'] == 'scriptw-cli'
            and case['target'] == 'cli' and case['kind'] == 'test')


def native_stall_seconds(package):
    """Keep the original whole-target watcher allowance in each child."""
    return 3600 if package == 'scriptw-cli' else 900


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


def validate_native_cpu_targets(inventory, requested, native_filters,
                                include_ignored, partition_targets=()):
    """Admit complete idle scopes without partitioning publication or timing."""
    selected = sorted(set(requested))
    if selected and not include_ignored:
        raise ValueError('Complete native CPU targets require --include-ignored')
    for name in selected:
        matches = [case for case in inventory
                   if case['package'] + ':' + case['target'] == name]
        if len(matches) != 1:
            raise ValueError(f'Native CPU target must name one available '
                             f'Cargo metadata scope: {name}')
        scope = matches[0]
        # Publication already runs its fixed case workers in one idle scope.
        # Let them share the inherited CPU budget without changing their CLI
        # jobs/worker counts, assertions or termination bound. Timing gates
        # retain their separate original measurement policy.
        publication = (scope['package'] == 'scriptw-cli'
                       and scope['target'] == 'fake_suite_publication'
                       and scope['kind'] == 'test')
        if (scope['kind'] not in ('lib', 'bin', 'test')
                or (is_barrier(scope) and not publication)):
            raise ValueError(f'Native CPU targets exclude documentation, timing '
                             f'and campaign barriers: {name}')
        if name in native_filters:
            raise ValueError(f'Native CPU targets require an unfiltered whole '
                             f'target: {name}')
    if set(selected) & set(partition_targets):
        raise ValueError('A native target cannot own compiler CPUs and partitions')
    return selected


def native_cpu_target(case, targets):
    return (case['kind'] in ('lib', 'bin', 'test')
            and case['package'] + ':' + case['target'] in targets)


def is_barrier(case):
    return (case['kind'] == 'doc'
            or (case['package'] == 'scriptw-cli' and case['target'] in (
                'perf', 'fake_suite_publication', 'differential_corpus',
                'differential_robustness', 'differential_scenario_pair',
                'differential_worker'))
            or (case['package'] == 'scriptw-test262' and case['target'] in (
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
            "native_jobs": int(os.environ["SCRIPTW_JOBS"]),
            "rayon_threads": int(os.environ["RAYON_NUM_THREADS"]),
            "cargo_jobs": int(os.environ["CARGO_BUILD_JOBS"]),
            "libtest_threads": int(os.environ["RUST_TEST_THREADS"]),
            "command": sys.argv[2:]}
Path(sys.argv[1]).write_text(json.dumps(resource, sort_keys=True) + "\\n")
os.execvpe(sys.argv[2], sys.argv[2:], os.environ)
'''


def validate_native_resources(resource, command, inherited_cpus):
    """Check the actual child policy, including unchanged serial test/builds."""
    cpus = resource['cpus']
    if (not cpus or cpus != sorted(set(cpus))
            or not set(cpus) <= set(inherited_cpus)
            or resource['command'] != command
            or resource['libtest_threads'] != 1 or resource['cargo_jobs'] != 1
            or not 1 <= resource['native_jobs'] <= len(cpus)
            or not 1 <= resource['rayon_threads'] <= len(cpus)):
        raise ValueError('Native execution changed its command or bounded CPU/job policy')


def execution_order(cases, partition_cli, native_targets=(), cpu_targets=()):
    """Keep original indices and objects; drain independent work before barriers.

    Sorting is stable within both groups. Receipt inventories and continuation
    selection retain their original order; only dispatch order changes.
    """
    return sorted(enumerate(cases), key=lambda indexed: (
        is_barrier(indexed[1])
        or is_partitioned_native(indexed[1], partition_cli, native_targets)
        or native_cpu_target(indexed[1], cpu_targets)))
