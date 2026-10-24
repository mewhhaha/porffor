#!/usr/bin/env python3
"""Task-owned same-binary R-retention ABBA diagnostic; never changes global caps.

Preparation only until explicitly launched in an idle existing 4-GiB/one-CPU
verification cgroup. The old runtime_baseline executable reports P cache hits,
not R cache outcomes or cache-owner occupancy; those remain unavailable.
"""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil

from benchmark_metrics import digest, measure_process, summary
from limited_verification import kernel_group, limited_environment, require_kernel_budget


def require(condition, reason):
    if not condition:
        raise ValueError(reason)



def manifest_stdout(manifest, case_name):
    """Bind the checksum to the retained fixture inventory, including newlines."""
    require(isinstance(manifest, dict) and manifest.get('schema') == 'scriptw-v8-workloads-v1',
            'unsupported frozen fixture manifest')
    inventory = manifest.get('execution')
    require(isinstance(inventory, list), 'fixture execution inventory is missing')
    require(all(isinstance(case, dict) for case in inventory), 'invalid fixture execution entry')
    matches = [case for case in inventory if case.get('file') == case_name]
    require(len(matches) == 1, 'fixture must select exactly one manifest entry')
    expected = matches[0].get('stdout')
    require(isinstance(expected, str), 'fixture expected stdout must be a string')
    return expected


def _sha256(value):
    return (isinstance(value, str) and len(value) == 64
            and all(character in '0123456789abcdef' for character in value))


def _nonnegative_integer(value):
    return type(value) is int and value >= 0


def validate_baseline_payload(raw, *, case_name, fixture_sha256, executable_sha256,
                              source_commit, expected_stdout, samples, warmups, identity=None):
    """Admit a complete ordinary frozen-SDK payload before summarizing samples.

    The frozen SDK checks Normal Wasm-AOT completion before publishing its JSON;
    baseline-v2 has no independent completion field. P module hits below are
    strictly P observations, never evidence of R hits or cache-owner occupancy.
    Nested memory diagnostics remain raw telemetry, not allocation/live counters.
    """
    require(type(samples) is int and 3 <= samples <= 100
            and type(warmups) is int and 1 <= warmups <= 100, 'invalid sample plan')
    require(_sha256(fixture_sha256) and _sha256(executable_sha256), 'invalid expected SHA256')
    require(isinstance(expected_stdout, str), 'expected fixture stdout must be a string')
    require(isinstance(raw, dict) and raw.get('schema') == 'scriptw-wasmtime-baseline-v2',
            'only ordinary frozen SDK cohorts are admitted')
    require(raw.get('executable_sha256') == executable_sha256, 'loaded executable identity differs')
    require(raw.get('source_commit') == source_commit, 'payload is not the expected frozen source commit')
    require(_sha256(raw.get('source_fingerprint')), 'payload compiler fingerprint is missing or invalid')
    require('requested_native_policy' in raw and raw['requested_native_policy'] is None,
            'retention cohort requires ordinary requested native policy')
    require(type(raw.get('samples')) is int and raw['samples'] == samples
            and type(raw.get('warmups')) is int and raw['warmups'] == warmups,
            'payload sample plan differs')
    cases = raw.get('cases')
    require(isinstance(cases, list) and len(cases) == 1 and isinstance(cases[0], dict),
            'payload must contain exactly one complete case')
    case = cases[0]
    require(case.get('file') == case_name and case.get('source_sha256') == fixture_sha256,
            'payload source differs')
    require(case.get('source_compilation_profile') == 'portable', 'payload source profile differs')
    require(_sha256(case.get('wasm_sha256')) and _nonnegative_integer(case.get('wasm_bytes'))
            and case['wasm_bytes'] > 0, 'P artifact identity is missing or invalid')
    runtime = case.get('linked_runtime')
    require(isinstance(runtime, dict) and set(runtime) == {'key', 'wasm_sha256', 'wasm_bytes'}
            and _sha256(runtime.get('key')) and _sha256(runtime.get('wasm_sha256'))
            and _nonnegative_integer(runtime.get('wasm_bytes')) and runtime['wasm_bytes'] > 0,
            'linked R artifact identity is missing or invalid')
    observations = case.get('observations')
    require(isinstance(observations, list) and len(observations) == warmups + samples,
            'incomplete all-observation inventory')
    runtime_policy = None
    phases = ('admission', 'engine', 'module', 'store_and_linker',
              'host_import_registration', 'instantiate', 'export_lookup', 'total')
    for index, observation in enumerate(observations):
        require(isinstance(observation, dict), f'observation {index} is not an object')
        require(type(observation.get('warmup')) is bool
                and observation['warmup'] is (index < warmups),
                f'observation {index} warmup order differs')
        require(observation.get('stdout') == expected_stdout,
                f'observation {index} stdout differs from fixture manifest')
        require('actual_native_policy' not in observation,
                f'observation {index} contains explicit native policy')
        policy = observation.get('runtime_policy')
        require(isinstance(policy, str) and bool(policy), f'observation {index} runtime policy is missing')
        if runtime_policy is None:
            runtime_policy = policy
        require(policy == runtime_policy, f'observation {index} runtime policy differs')
        require(observation.get('module_cache') in ('Hit', 'Miss'),
                f'observation {index} P module cache outcome is invalid')
        if index > 0:
            require(observation['module_cache'] == 'Hit', f'observation {index} warm P module cache miss')
        require(_nonnegative_integer(observation.get('execution_ns')),
                f'observation {index} execution timing is invalid')
        timings = observation.get('phases_ns')
        require(isinstance(timings, dict)
                and all(_nonnegative_integer(timings.get(phase)) for phase in phases),
                f'observation {index} phase timings are incomplete or invalid')
        require(timings['total'] >= observation['execution_ns']
                and all(timings['total'] >= timings[phase] for phase in phases),
                f'observation {index} total timing is inconsistent')
    current = dict(source_sha256=case['source_sha256'], wasm_sha256=case['wasm_sha256'],
                   wasm_bytes=case['wasm_bytes'], linked_runtime=dict(runtime),
                   source_fingerprint=raw['source_fingerprint'], runtime_policy=runtime_policy)
    require(identity is None or current == identity, 'P/R or compiler/policy identity differs across retention cohorts')
    return case, observations[warmups:], current


def run(args):
    require_kernel_budget(4 * 1024 ** 3)
    require(len(os.sched_getaffinity(0)) == 1, 'one inherited CPU is required')
    runner, fixtures, output = args.runner.resolve(), args.fixtures.resolve(), args.output_dir.resolve()
    require(runner.is_file(), 'an existing frozen runtime_baseline executable is required')
    require((fixtures / 'manifest.json').is_file(), 'existing frozen fixtures are required')
    time_binary = Path(args.time or shutil.which('gtime') or shutil.which('time') or '/missing-time').resolve()
    require(time_binary.is_file(), 'GNU time is required for child CPU/RSS accounting')
    env = limited_environment()
    for name in ('SCRIPTW_COMPILER_CACHE_IMAGE_HASH', 'SCRIPTW_WASM_TRACE', 'SCRIPTW_WASM_TRACE_DUMP'):
        require(not env.get(name), f'unset unrelated experiment {name}')
    output.mkdir(parents=True, exist_ok=False)
    image_hash = digest(runner)
    fixture_hash = digest(fixtures / args.case)
    manifest_hash = digest(fixtures / 'manifest.json')
    expected_stdout = manifest_stdout(json.loads((fixtures / 'manifest.json').read_text()), args.case)
    report = dict(schema='scriptw-same-image-runtime-retention-abba-v1', complete=False,
        started_at=datetime.now(timezone.utc).isoformat(), runner=str(runner), runner_sha256=image_hash,
        expected_source_commit=args.source_commit,
        case=args.case, source_sha256=fixture_hash, manifest_sha256=manifest_hash,
        expected_stdout=expected_stdout, samples=args.samples, warmups=args.warmups,
        configuration=dict(order=['A', 'B', 'B', 'A'],
            A=dict(entry_limit=1, image_byte_limit=64 * 1024 ** 2),
            B=dict(entry_limit=2, image_byte_limit=128 * 1024 ** 2),
            scope='Payload-only positive cache limits override the limiter defaults; inherited cgroup and CPU affinity remain unchanged. Each old SDK invocation creates fresh Stores and Wasm Realms.',
            limitations='P-only cache Hit is not R Hit. Old executable exposes no cache-owner occupancy or R cache outcome. No claim of exact R mapping bytes from serialized native-image size.'),
        resources=dict(cgroup=str(kernel_group()), affinity=sorted(os.sched_getaffinity(0)),
            memory_max=(kernel_group() / 'memory.max').read_text().strip(),
            memory_swap_max=(kernel_group() / 'memory.swap.max').read_text().strip()), runs=[])

    def save():
        pending = output / 'receipt.pending'
        pending.write_text(json.dumps(report, indent=2) + '\n')
        pending.replace(output / 'receipt.json')

    save()
    identity = None
    try:
        for index, cohort in enumerate(report['configuration']['order']):
            limits = report['configuration'][cohort]
            label = f'{index:02}-{cohort}'
            artifact_dir = output / label
            artifact_dir.mkdir()
            # This is the measured payload's environment, after the existing
            # limiter has already bounded its whole process group. No global
            # limits, source tree, cache owner or mutable Realm is replaced.
            payload = dict(env, SCRIPTW_CACHE_DIR=str(output / 'native-cache'),
                SCRIPTW_MODULE_MEMORY_CACHE_ENTRIES=str(limits['entry_limit']),
                SCRIPTW_MODULE_MEMORY_CACHE_LIMIT_BYTES=str(limits['image_byte_limit']))
            command = [str(runner), str(fixtures), str(args.samples), str(args.warmups), args.case, str(artifact_dir)]
            row = dict(cohort=cohort, command=command, state='running',
                payload_cache_environment={key: payload[key] for key in (
                    'SCRIPTW_MODULE_MEMORY_CACHE_ENTRIES', 'SCRIPTW_MODULE_MEMORY_CACHE_LIMIT_BYTES')},
                runtime_module_cache=dict(status='unavailable', reason='frozen SDK reports P only'),
                retained_cache_entries=dict(status='unavailable', reason='frozen SDK has no owner observation'),
                retained_cache_image_bytes=dict(status='unavailable', reason='frozen SDK has no owner observation'))
            report['runs'].append(row)
            save()
            require(digest(runner) == image_hash and digest(fixtures / args.case) == fixture_hash
                    and digest(fixtures / 'manifest.json') == manifest_hash,
                    'frozen executable, source or fixture manifest changed')
            with (artifact_dir / 'stdout.json').open('xb') as stdout, (artifact_dir / 'stderr.txt').open('xb') as stderr:
                row['accounting'] = measure_process(command, cwd=Path(__file__).resolve().parents[1],
                    env=payload, stdout=stdout, stderr=stderr, timeout=args.timeout, time_binary=time_binary)
            require(row['accounting']['exit_code'] == 0 and not row['accounting']['timed_out'], 'payload failed')
            raw = json.loads((artifact_dir / 'stdout.json').read_text())
            case, observations, identity = validate_baseline_payload(raw,
                case_name=args.case, fixture_sha256=fixture_hash, executable_sha256=image_hash,
                source_commit=args.source_commit, expected_stdout=expected_stdout,
                samples=args.samples, warmups=args.warmups, identity=identity)
            row.update(state='closed', raw=raw,
                module_ns=summary([value['phases_ns']['module'] for value in observations]),
                execution_ns=summary([value['execution_ns'] for value in observations]),
                total_ns=summary([value['phases_ns']['total'] for value in observations]),
                stdout_sha256=digest(artifact_dir / 'stdout.json'), stderr_sha256=digest(artifact_dir / 'stderr.txt'))
            save()
        report.update(complete=True, finished_at=datetime.now(timezone.utc).isoformat())
    except BaseException as error:
        report['failure'] = str(error)
        raise
    finally:
        save()
    return output / 'receipt.json'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runner', type=Path, required=True)
    parser.add_argument('--fixtures', type=Path, required=True)
    parser.add_argument('--output-dir', type=Path, required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--case', default='objects.js', choices=('integer.js', 'objects.js', 'retained-objects.js'))
    parser.add_argument('--samples', type=int, default=7, choices=range(3, 101))
    parser.add_argument('--warmups', type=int, default=10, choices=range(1, 101))
    parser.add_argument('--timeout', type=int, default=180, choices=range(1, 3601))
    parser.add_argument('--time')
    print(run(parser.parse_args()))


if __name__ == '__main__':
    main()
