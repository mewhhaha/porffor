#!/usr/bin/env python3
"""Opt-in local release/Wasmtime vs Node/V8 baseline with retained raw samples."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def summary(samples):
    if len(samples) < 3 or any(not isinstance(n, int) or n <= 0 for n in samples):
        raise ValueError('need at least three positive measured samples')
    return dict(samples_ns=samples, median_ns=statistics.median(samples),
                minimum_ns=min(samples), maximum_ns=max(samples))


def machine():
    cpu = next((line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines()
                if line.startswith('model name')), platform.processor())
    cgroups = []
    membership = next(line[3:] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    group = Path('/sys/fs/cgroup') / membership.lstrip('/')
    if '..' in group.parts:
        raise ValueError('invalid cgroup membership')
    while True:
        row = {}
        for name in ['cpu.max', 'memory.max', 'memory.swap.max']:
            path = group / name
            if path.exists():
                row[name] = path.read_text().strip()
        cgroups.append(row)
        if group == Path('/sys/fs/cgroup'):
            break
        group = group.parent
    memory_limits = [int(row['memory.max']) for row in cgroups if row.get('memory.max', 'max') != 'max']
    cpu_limits = [int(row['cpu.max'].split()[0]) / int(row['cpu.max'].split()[1]) for row in cgroups
                  if row.get('cpu.max', 'max').split()[0] != 'max']
    return dict(system=platform.platform(), architecture=platform.machine(), cpu_model=cpu,
                host_logical_cpus=os.cpu_count(), affinity_before_pin=sorted(os.sched_getaffinity(0)),
                cgroups=cgroups, effective_memory_bytes=min(memory_limits) if memory_limits else None,
                effective_cpu_quota=min(cpu_limits) if cpu_limits else None)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scriptw', type=Path, required=True)
    parser.add_argument('--runtime-runner', type=Path, required=True)
    parser.add_argument('--build-log', type=Path, required=True)
    parser.add_argument('--node', default='node')
    parser.add_argument('--output-dir', type=Path, required=True)
    parser.add_argument('--samples', type=int, default=7, choices=range(3, 101))
    parser.add_argument('--idle-machine', action='store_true', required=True)
    args = parser.parse_args()
    cli, runner = args.scriptw.resolve(), args.runtime_runner.resolve()
    if subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT):
        raise ValueError('commit the coherent source batch before measuring')
    producer = {}
    for line in args.build_log.read_text().splitlines():
        row = json.loads(line)
        if row.get('reason') == 'compiler-artifact' and row.get('executable'):
            producer[Path(row['executable']).resolve()] = row
    for path in [cli, runner]:
        row = producer.get(path)
        if not row or row['profile']['opt_level'] != '3' or row['profile']['test'] or row['features']:
            raise ValueError('both executables need actual default-feature release Cargo producers')
        expected_name, expected_kind = ('scriptw', 'bin') if path == cli else ('runtime_baseline', 'example')
        if row['target']['name'] != expected_name or expected_kind not in row['target']['kind'] or Path(row['manifest_path']).resolve() != ROOT / 'crates/scriptw-cli/Cargo.toml':
            raise ValueError('Cargo producer target or package differs')
    fixtures = ROOT / 'crates/scriptw-cli/tests/fixtures/v8_baseline'
    plan = json.loads((fixtures / 'manifest.json').read_text())
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=False)
    (output / 'logs').mkdir()
    spec = machine()
    # Both engines share one inherited allowed CPU. No concurrent native owner.
    selected_cpu = min(os.sched_getaffinity(0))
    os.sched_setaffinity(0, {selected_cpu})
    spec['measurement_affinity'] = [selected_cpu]
    env = dict(os.environ)
    if env.get('NODE_OPTIONS'):
        raise ValueError('NODE_OPTIONS must be unset for the normal V8 baseline')
    env.pop('NODE_COMPILE_CACHE', None)
    env['NODE_DISABLE_COMPILE_CACHE'] = '1'
    report = dict(schema='scriptw-v8-machine-baseline-v1', complete=False,
                  started_at=datetime.now(timezone.utc).isoformat(), machine=spec,
                  source_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  cli_sha256=digest(cli), runtime_runner_sha256=digest(runner),
                  cargo_build_log_sha256=digest(args.build_log), profile='release; default features; opt-level 3',
                  samples=args.samples, process_samples=[], execution={},
                  policy=dict(startup='new process; parent monotonic wall time; startup.js print/checksum; OS page cache not flushed',
                              cold='unique empty scriptw application cache per sample; normal Node with no persistent compile cache',
                              hot='prewarmed scriptw persistent cache in a new process; normal Node repeated new process, no equivalent persistent code cache',
                              execution='source compiled once; warmed code; fresh Wasmtime Store or V8 vm Context; host monotonic execution span including output/completion; excludes source compile/context creation',
                              wasmtime_compilation='product default Fast: Cranelift OptLevel::None; no custom faster backend',
                              limitations='single cloud VM, one pinned CPU, no OS-cache flush/DVFS control/statistical significance or conformance acceptance'))
    report['harness_sha256'] = {str(path.relative_to(ROOT)): digest(path) for path in
        [Path(__file__).resolve(), fixtures / 'manifest.json', fixtures / 'node-runtime.cjs',
         ROOT / 'crates/scriptw-cli/examples/runtime_baseline.rs']}

    def save():
        temp = output / 'report.pending'
        temp.write_text(json.dumps(report, indent=2) + '\n')
        temp.replace(output / 'report.json')

    def invoke(label, command, environment, timeout=300):
        begin = time.perf_counter_ns()
        event = dict(label=label, command=[str(c) for c in command], state='running', timeout_seconds=timeout)
        report.setdefault('commands', []).append(event)
        save()
        with (output / 'logs' / (label + '.stdout')).open('xb') as out, (output / 'logs' / (label + '.stderr')).open('xb') as err:
            try:
                result = subprocess.run(command, cwd=ROOT, env=environment, stdout=out, stderr=err, timeout=timeout)
            except (OSError, subprocess.TimeoutExpired) as error:
                event.update(state='failed', failure=str(error), elapsed_ns=time.perf_counter_ns() - begin)
                report['failure'] = dict(label=label, reason=str(error))
                save()
                raise
        elapsed = time.perf_counter_ns() - begin
        event.update(state='closed', elapsed_ns=elapsed,
                     exit_code=result.returncode,
                     stdout_sha256=digest(output / 'logs' / (label + '.stdout')),
                     stderr_sha256=digest(output / 'logs' / (label + '.stderr')))
        save()
        if result.returncode:
            report['failure'] = dict(label=label, reason=f'exit {result.returncode}')
            save()
            raise RuntimeError(f'{label} failed: retained logs and incomplete report in {output}')
        return event, (output / 'logs' / (label + '.stdout')).read_text()

    save()
    node_versions = subprocess.check_output([args.node, '-p', 'JSON.stringify(process.versions)'], env=env, text=True)
    report['node_versions'] = json.loads(node_versions)
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    report['wasmtime_version'] = next(p['version'] for p in lock['package'] if p['name'] == 'wasmtime')
    event, text = invoke('compiler-identity', [cli, 'compiler-identity'], env)
    report['compiler_identity'] = json.loads(text)
    identity = report['compiler_identity']
    if identity['source_revision'] != dict(kind='git-commit', commit=report['source_commit']) or identity['executable_sha256'] != report['cli_sha256']:
        raise ValueError('loaded compiler source/executable identity differs from the measured image')
    report['workloads'] = [dict(case, sha256=digest(fixtures / case['file']))
                           for case in [plan['startup'], *plan['execution']]]
    startup = fixtures / plan['startup']['file']
    suffix = 'run outcome: RunOutcome { backend_used: WasmAot, note: "wasm-aot completion: undefined(undefined)" }\n'
    node_command = [args.node, '-e', 'globalThis.print = console.log; require(process.argv[1]);', str(startup)]
    scriptw_command = [cli, '--jobs', '1', 'run', '--execution-backend', 'wasm', str(startup)]
    warm_cache = output / 'hot-cache'
    warm_env = dict(env, SCRIPTW_CACHE_DIR=str(warm_cache))
    _, text = invoke('scriptw-prime-hot-cache', scriptw_command, warm_env)
    if text != plan['startup']['stdout'] + suffix:
        raise ValueError('warmup did not produce the expected Wasm-AOT outcome')
    for mode in ['cold', 'hot']:
        for index in range(args.samples):
            for runtime in (['scriptw', 'v8'] if index % 2 == 0 else ['v8', 'scriptw']):
                label = f'{mode}-{index:02}-{runtime}'
                runtime_env = warm_env if mode == 'hot' else dict(env, SCRIPTW_CACHE_DIR=str(output / f'cold-cache-{index:02}'))
                event, text = invoke(label, scriptw_command if runtime == 'scriptw' else node_command, runtime_env)
                expected = plan['startup']['stdout'] + (suffix if runtime == 'scriptw' else '')
                if text != expected:
                    raise ValueError(f'{label}: output mismatch; original retained')
                report['process_samples'].append(dict(mode=mode, runtime=runtime, elapsed_ns=event['elapsed_ns'], label=label))
                save()
    for runtime, command in [
        ('wasmtime', [runner, fixtures, str(args.samples), '3']),
        ('v8', [args.node, str(fixtures / 'node-runtime.cjs'), str(fixtures), str(args.samples), '50'])]:
        _, text = invoke('execution-' + runtime, command, dict(env, SCRIPTW_CACHE_DIR=str(output / 'execution-cache')), timeout=900)
        report['execution'][runtime] = json.loads(text)
        save()
    native = report['execution']['wasmtime']
    if native['source_commit'] != report['source_commit'] or native['source_fingerprint'] != identity['source_fingerprint'] or native['executable_sha256'] != report['runtime_runner_sha256']:
        raise ValueError('runtime runner is not bound to the same compiler source and actual executable')
    for runtime, result in report['execution'].items():
        expected_schema = 'scriptw-wasmtime-baseline-v1' if runtime == 'wasmtime' else 'scriptw-v8-baseline-v1'
        warmups = 3 if runtime == 'wasmtime' else 50
        if result['schema'] != expected_schema or result['samples'] != args.samples or result['warmups'] != warmups:
            raise ValueError('runtime sample plan differs')
        if [c['file'] for c in result['cases']] != [c['file'] for c in plan['execution']]:
            raise ValueError('runtime case inventory differs')
        for case, expected in zip(result['cases'], plan['execution'], strict=True):
            if case['source_sha256'] != digest(fixtures / expected['file']) or len(case['observations']) != warmups + args.samples:
                raise ValueError('runtime source or sample count differs')
            for index, sample in enumerate(case['observations']):
                if sample['warmup'] != (index < warmups) or sample['stdout'] != expected['stdout']:
                    raise ValueError('runtime output or warmup inventory differs')
                if runtime == 'wasmtime' and index >= warmups and sample['module_cache'] != 'Hit':
                    raise ValueError('execution sample was not actually warm in the native-module cache')
    for left, right in zip(native['cases'], report['execution']['v8']['cases'], strict=True):
        if left['file'] != right['file'] or left['source_sha256'] != right['source_sha256']:
            raise ValueError('execution workloads differ')
    report['summaries'] = dict(startup={}, execution={})
    for mode in ['cold', 'hot']:
        report['summaries']['startup'][mode] = {runtime: summary([p['elapsed_ns'] for p in report['process_samples']
              if p['mode'] == mode and p['runtime'] == runtime]) for runtime in ['scriptw', 'v8']}
    for runtime in ['wasmtime', 'v8']:
        report['summaries']['execution'][runtime] = {case['file']: summary([p['execution_ns'] for p in case['observations'] if not p['warmup']])
              for case in report['execution'][runtime]['cases']}
    report.update(complete=True, finished_at=datetime.now(timezone.utc).isoformat())
    save()
    print(json.dumps(dict(report=str(output / 'report.json'), summaries=report['summaries']), indent=2))


if __name__ == '__main__':
    main()
