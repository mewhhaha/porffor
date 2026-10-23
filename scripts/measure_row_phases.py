#!/usr/bin/env python3
"""Measure row phases and diagnostic overhead using an identified existing runner.

Run through limited_verification.py on an otherwise idle machine. This performs
no builds, creates a new evidence directory, and retains failed attempts. It
does not update the ordinary V8 table or claim an optimization speedup.
"""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import platform
import shutil
from statistics import median
import subprocess

from benchmark_metrics import digest, measure_process

ROOT = Path(__file__).resolve().parents[1]
PHASES = ('construction_ns', 'traversal_ns', 'combined_ns', 'middle_callback_ns')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate(raw, entry, runner_hash, samples, warmups, observed, region):
    schema = 'scriptw-wasmtime-row-phases-v1' if observed else 'scriptw-wasmtime-baseline-v2'
    require(raw['schema'] == schema and raw['samples'] == samples
            and raw['warmups'] == warmups and raw['executable_sha256'] == runner_hash,
            'runner identity or sample plan differs')
    require(len(raw['cases']) == 1, 'expected one program per child')
    case = raw['cases'][0]
    require(case['file'] == entry['file'] and case['source_sha256'] == entry['source_sha256'],
            'fixture identity differs')
    rows = case['observations']
    require(len(rows) == warmups + samples
            and [row['warmup'] for row in rows] == [True] * warmups + [False] * samples,
            'warmup order differs')
    for row in rows:
        require(row['stdout'] == entry['stdout'] and row['execution_ns'] > 0,
                'checksum or execution time differs')
        require(row['runtime_policy'] == rows[0]['runtime_policy'], 'runtime policy changed')
        if not row['warmup']:
            require(row['module_cache'] == 'Hit', 'measured module-cache miss')
        require(('row_phases' in row) == observed, 'unexpected phase observer state')
        if observed:
            phase = row['row_phases']
            require(all(type(phase[key]) is int and phase[key] >= 0 for key in PHASES),
                    'invalid phase duration')
            require(phase['combined_ns'] == phase['construction_ns'] + phase['traversal_ns']
                    + phase['middle_callback_ns'], 'phase boundaries do not add up')
            sampled = region is not None and not row['warmup']
            require(phase['perf_region'] == region and phase['perf_sampled'] == sampled
                    and [span['enable'] for span in phase['perf_control_spans']]
                    == ([True, False] if sampled else []), 'perf window differs')
    return dict(source_commit=raw['source_commit'], source_fingerprint=raw['source_fingerprint'],
                wasm_sha256=case['wasm_sha256'], linked_runtime=case['linked_runtime'],
                runtime_policy=rows[0]['runtime_policy'])


def check_map(path, case):
    lines = path.read_text().splitlines()
    pid = int(path.name.removeprefix('perf-').removesuffix('.map.identity'))
    require(lines[0] == f'scriptw-perf-map-v1 pid={pid} boundary=serial-single-program-snapshot',
            'map process or boundary differs')
    modules = {}
    for line in lines[1:3]:
        label, *fields = line.split()
        modules[label] = dict(field.split('=', 1) for field in fields)
    require(set(modules) == {'R', 'P'} and len(lines) == 4, 'expected two mapped modules')
    require(modules['R']['wasm_sha256'] == case['linked_runtime']['wasm_sha256']
            and modules['P']['wasm_sha256'] == case['wasm_sha256'], 'mapped artifact differs')
    mapping = path.with_suffix('')
    require(lines[3] == 'map_sha256=' + digest(mapping), 'loaded-code map checksum differs')
    for module in modules.values():
        require(len(module['native_text_sha256']) == 64 and int(module['bytes']) > 0,
                'missing loaded native-text identity')
    return dict(pid=pid, modules=modules, map_sha256=digest(mapping))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runner', required=True, type=Path)
    parser.add_argument('--corpus', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--cache', required=True, type=Path)
    parser.add_argument('--perf', required=True, type=Path)
    parser.add_argument('--perf-library-path')
    parser.add_argument('--profile-case', default='rows-n8192-w4-p4-phases.js')
    parser.add_argument('--samples', type=int, default=7)
    parser.add_argument('--warmups', type=int, default=10)
    parser.add_argument('--frequency', type=int, default=499)
    args = parser.parse_args()
    require(3 <= args.samples <= 100 and 1 <= args.warmups <= 100
            and 1 <= args.frequency <= 999, 'invalid sample plan')
    runner, corpus, output, perf = (path.resolve() for path in
                                    (args.runner, args.corpus, args.output, args.perf))
    plan = json.loads((corpus / 'manifest.json').read_text())
    entries = plan['execution']
    names = [entry['file'] for entry in entries]
    require(len(names) == len(set(names)) and args.profile_case in names
            and 'row-phases-empty.js' in names, 'invalid case inventory')
    require(all(Path(name).name == name and name.endswith('.js') for name in names),
            'cases must be basenames')
    output.mkdir(parents=True, exist_ok=False)
    shutil.copytree(corpus, output / 'corpus')
    for entry in entries:
        entry['source_sha256'] = digest(output / 'corpus' / entry['file'])
    cpu = min(os.sched_getaffinity(0))
    os.sched_setaffinity(0, {cpu})
    env = dict(os.environ, SCRIPTW_CACHE_DIR=str(args.cache.resolve()))
    for key in ('SCRIPTW_WASM_TRACE', 'SCRIPTW_PERF_MAP_DIR', 'LD_LIBRARY_PATH',
                'SCRIPTW_ROW_PERF_CONTROL', 'SCRIPTW_ROW_PERF_ACK', 'SCRIPTW_ROW_PERF_REGION'):
        env.pop(key, None)
    perf_env = dict(env)
    if args.perf_library_path:
        perf_env['LD_LIBRARY_PATH'] = args.perf_library_path
    version = subprocess.check_output([perf, '--version'], env=perf_env, text=True).strip()
    report = dict(schema='scriptw-row-phase-measurement-v1', complete=False,
                  started=datetime.now(timezone.utc).isoformat(),
                  harness_sha256=digest(Path(__file__)),
                  runner=dict(path=str(runner), sha256=digest(runner)),
                  perf=dict(path=str(perf), sha256=digest(perf), version=version),
                  configuration=dict(samples=args.samples, warmups=args.warmups,
                                     frequency=args.frequency, profile_case=args.profile_case),
                  machine=dict(cpu=cpu, platform=platform.platform(),
                               cpu_model=next(line.split(':', 1)[1].strip() for line in
                                   Path('/proc/cpuinfo').read_text().splitlines()
                                   if line.startswith('model name'))),
                  corpus=entries, observations=[])

    def save():
        pending = output / 'receipt.pending'
        pending.write_text(json.dumps(report, indent=2) + '\n')
        pending.replace(output / 'receipt.json')

    save()
    # Execute each exact source independently in normal V8 as a checksum oracle.
    for entry in entries:
        oracle = subprocess.run(['node', '-e', 'globalThis.print=console.log;require(process.argv[1]);',
                                 str(output / 'corpus' / entry['file'])],
                                capture_output=True, text=True, check=True, timeout=30)
        require(oracle.stdout == entry['stdout'], 'Node oracle checksum differs')
    report['node_oracle'] = dict(version=subprocess.check_output(['node', '--version'], text=True).strip(),
                                 checked_sources=len(entries))
    save()
    identities = {}

    def invoke(entry, group, index, observed, region=None):
        label = f'{entry["file"]}-{group}-{index}'
        folder = output / label
        folder.mkdir()
        (folder / 'wasm').mkdir()
        command = [str(runner), str(output / 'corpus'), str(args.samples), str(args.warmups),
                   entry['file'], str(folder / 'wasm')]
        if observed:
            command.append('--row-phases')
        environment = env
        maps_before = set(Path('/tmp').glob('perf-*.map.identity'))
        if region:
            for name in ('control', 'ack'):
                os.mkfifo(folder / name)
            environment = dict(perf_env, SCRIPTW_PERF_MAP_DIR='/tmp',
                               SCRIPTW_ROW_PERF_CONTROL=str(folder / 'control'),
                               SCRIPTW_ROW_PERF_ACK=str(folder / 'ack'), SCRIPTW_ROW_PERF_REGION=region)
            command = [str(perf), 'record', '-D', '-1', '--control',
                       f'fifo:{folder / "control"},{folder / "ack"}', '-e', 'cpu-clock:u',
                       '-F', str(args.frequency), '-g', '-o', str(folder / 'perf.data'), '--',
                       'env', '-u', 'LD_LIBRARY_PATH', *command]
        row = dict(label=label, file=entry['file'], group=group, index=index,
                   observer=observed, region=region, command=command, complete=False)
        report['observations'].append(row)
        save()
        with (folder / 'stdout.json').open('xb') as stdout, (folder / 'stderr.log').open('xb') as stderr:
            row['process'] = measure_process(command, cwd=ROOT, env=environment,
                                             stdout=stdout, stderr=stderr, timeout=600)
        save()
        require(row['process']['exit_code'] == 0 and not row['process']['timed_out'],
                'child failed; inspect ' + str(folder))
        raw = json.loads((folder / 'stdout.json').read_text())
        identity = validate(raw, entry, report['runner']['sha256'], args.samples,
                            args.warmups, observed, region)
        require(identities.setdefault(entry['file'], identity) == identity, 'case identity changed')
        require(digest(folder / 'wasm' / (entry['file'] + '.wasm')) == identity['wasm_sha256'],
                'emitted artifact checksum differs')
        if region:
            maps = set(Path('/tmp').glob('perf-*.map.identity')) - maps_before
            require(len(maps) == 1, 'expected exactly one fresh serial loaded-code map')
            path = maps.pop()
            row['map'] = check_map(path, raw['cases'][0])
            shutil.copy2(path, folder / path.name)
            shutil.copy2(path.with_suffix(''), folder / path.with_suffix('').name)
            row['perf_data_sha256'] = digest(folder / 'perf.data')
        measured = raw['cases'][0]['observations'][args.warmups:]
        row.update(complete=True, identity=identity, raw_sha256=digest(folder / 'stdout.json'),
                   execution_median_ns=median(item['execution_ns'] for item in measured))
        if observed:
            row['phase_medians_ns'] = {key: median(item['row_phases'][key] for item in measured)
                                        for key in PHASES}
        save()
        print(label, 'execution ms', round(row['execution_median_ns'] / 1e6, 3),
              'phase ms', {key: round(value / 1e6, 3) for key, value in
                           row.get('phase_medians_ns', {}).items()}, flush=True)

    # Same marked JS and runner, observer off/on/on/off: no compiler change.
    for entry in entries:
        for index, observed in enumerate((False, True, True, False)):
            invoke(entry, 'observer-abba', index, observed)
    selected = next(entry for entry in entries if entry['file'] == args.profile_case)
    empty = next(entry for entry in entries if entry['file'] == 'row-phases-empty.js')
    # Keep the observer on; separately estimate total profiler/control overhead.
    for entry in (empty, selected):
        for index, region in enumerate((None, 'combined', 'combined', None)):
            invoke(entry, 'profiler-abba', index, True, region)
    for region in ('construction', 'traversal'):
        for index in range(2):
            invoke(selected, region, index, True, region)
    require(digest(runner) == report['runner']['sha256']
            and digest(perf) == report['perf']['sha256'], 'executable changed during measurement')
    report.update(complete=True, finished=datetime.now(timezone.utc).isoformat())
    save()
    print('Complete:', output / 'receipt.json', flush=True)


if __name__ == '__main__':
    main()
