"""Real child accounting, retained-evidence admission and V8 memory observations."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import benchmark_metrics as metrics

SPEC = importlib.util.spec_from_file_location('benchmark_v8', SCRIPTS / 'benchmark-v8.py')
BENCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BENCH)
FIXTURES = SCRIPTS.parent / 'crates/scriptw-cli/tests/fixtures/v8_baseline'


class AccountingTests(unittest.TestCase):
    def run_child(self, code, timeout=10):
        with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
            return metrics.measure_process([sys.executable, '-c', code], cwd=SCRIPTS,
                env=os.environ.copy(), stdout=out, stderr=err, timeout=timeout)

    def test_peak_is_this_child_not_previous_larger_child(self):
        large = self.run_child('x = bytearray(64 * 1024 * 1024)')
        small = self.run_child('pass')
        self.assertEqual(large['exit_code'], 0)
        self.assertGreater(large['peak_rss_bytes'], small['peak_rss_bytes'] + 32 * 1024**2)
        self.assertGreater(small['elapsed_ns'], 0)
        self.assertFalse(small['timed_out'])
        self.assertEqual(small['memory_scope'], 'child-process-lifetime-high-water-rss')

    def test_failure_and_timeout_remain_failed_measurements(self):
        failed = self.run_child('raise SystemExit(7)')
        self.assertEqual(failed['exit_code'], 7)
        stopped = self.run_child('import time; time.sleep(30)', timeout=.05)
        self.assertTrue(stopped['timed_out'])
        self.assertEqual(stopped['exit_code'], -9)
        self.assertLess(stopped['elapsed_ns'], 5_000_000_000)

    def test_summary_retains_outliers_and_uses_nearest_rank(self):
        result = metrics.summary([1, 100, 3, 2])
        self.assertEqual(result['samples'], [1, 100, 3, 2])
        self.assertEqual(result['median'], 2.5)
        self.assertEqual(result['p95'], 100)
        for values in ([1, 2], [True, 2, 3], [-1, 2, 3]):
            with self.assertRaises(ValueError):
                metrics.summary(values)


class EvidenceTests(unittest.TestCase):
    def make_report(self, root):
        for directory in ('logs', 'workload', 'wasm'):
            (root / directory).mkdir()
        source = root / 'workload' / 'case.js'
        source.write_text('print(42);')
        startup = root / 'workload' / 'startup.js'
        startup.write_text('print("start")')
        plan = dict(schema='scriptw-v8-workloads-v1', startup=dict(file='startup.js', stdout='start\n'),
                    execution=[dict(file='case.js', stdout='42\n')])
        (root / 'workload' / 'manifest.json').write_text(json.dumps(plan))
        wasm = root / 'wasm' / 'case.js.wasm'
        wasm.write_bytes(b'\x00asm\x01\x00\x00\x00')
        observations = [dict(warmup=i == 0, execution_ns=10 + i, stdout='42\n', module_cache='Hit',
            phases_ns=dict(engine=1, module=2, store_and_linker=3, instantiate=4, export_lookup=1, total=1000),
            memory=dict(gc_capacity_before_bytes=0, gc_capacity_after_bytes=4096,
                process_rss=dict(status='sampled', scope='runtime-invocation-process-rss', interval_ns=10_000_000, elapsed_ns=100,
                    sample_count=2, initial_rss_bytes=1024, final_rss_bytes=2048, maximum_rss_bytes=2048),
                unavailable={**{k: 'WasmtimeHasNoPublicCounter' for k in ('live_gc_bytes', 'allocation_count',
                    'allocation_bytes', 'collection_count', 'gc_pause_time')}, 'peak_gc_heap_bytes': 'NoPerInvocationPeakSampler'}))
            for i in range(4)]
        case = dict(file='case.js', source_sha256=metrics.digest(source), observations=observations,
                    wasm_sha256=metrics.digest(wasm), wasm_bytes=wasm.stat().st_size,
                    compilation_ns=dict(preparation=1, lowering=2, emission=3, validation=4))
        raw = dict(schema='scriptw-wasmtime-baseline-v2', samples=3, warmups=1, cases=[case])
        stdout = root / 'logs' / 'execution-wasmtime-case.js.stdout'
        stdout.write_text(json.dumps(raw))
        stderr = root / 'logs' / 'execution-wasmtime-case.js.stderr'
        stderr.write_text('')
        command = dict(label='execution-wasmtime-case.js', state='closed', exit_code=0, timed_out=False,
                       elapsed_ns=123, peak_rss_bytes=4096, user_cpu_ns=10, system_cpu_ns=2,
                       stdout_sha256=metrics.digest(stdout), stderr_sha256=metrics.digest(stderr))
        memory = dict(rss=4096, heapUsed=1024, heapTotal=2048, external=0, arrayBuffers=0)
        v8case = dict(file='case.js', source_sha256=metrics.digest(source), compilation_ns=1,
                      observations=[dict(warmup=i == 0, execution_ns=10+i, stdout='42\n', memory=dict(
                          scope='process-wide-endpoints;natural-gc;not-peak-or-live-workload-bytes',
                          before_context=memory, before=memory, after=memory)) for i in range(4)])
        v8raw = dict(schema='scriptw-v8-baseline-v2', samples=3, warmups=1, cases=[v8case])
        v8stdout = root / 'logs' / 'execution-v8-case.js.stdout'
        v8stdout.write_text(json.dumps(v8raw))
        v8stderr = root / 'logs' / 'execution-v8-case.js.stderr';v8stderr.write_text('')
        v8command = dict(command, label='execution-v8-case.js', stdout_sha256=metrics.digest(v8stdout), stderr_sha256=metrics.digest(v8stderr))
        identity_stdout = root / 'logs' / 'compiler-identity.stdout';identity_stdout.write_text('{}')
        identity_stderr = root / 'logs' / 'compiler-identity.stderr';identity_stderr.write_text('')
        identity_command = dict(command, label='compiler-identity', stdout_sha256=metrics.digest(identity_stdout), stderr_sha256=metrics.digest(identity_stderr))
        report = dict(schema=metrics.SCHEMA, complete=True, source_commit='old', machine={'cpu': 'test'},
            configuration=dict(startup=False, warmups=1, timeout_seconds=900, native_jobs=1),
            policy={'version': 2}, profile='release', samples=3, node_versions={'v8': 'test'},
            wasmtime_version='47', harness_sha256={'harness': 'hash'}, process_samples=[], commands=[identity_command, command, v8command],
            workloads=[dict(plan['startup'], role='startup', sha256=metrics.digest(startup)),
                       dict(plan['execution'][0], role='execution', sha256=metrics.digest(source))],
            execution={'wasmtime': {'cases': [dict(case, command_label=command['label'])]},
                       'v8': {'cases': [dict(v8case, command_label=v8command['label'])]}},
            retained_files=[dict(path=str(p.relative_to(root)), sha256=metrics.digest(p)) for p in root.rglob('*') if p.is_file()])
        report['summaries'] = metrics.summarize(report)
        path = root / 'report.json'
        path.write_text(json.dumps(report))
        return path, report

    def test_complete_report_recomputes_summaries_and_binds_retained_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path, report = self.make_report(root)
            self.assertEqual(metrics.load_report(path), report)
            (root / 'wasm' / 'case.js.wasm').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'retained benchmark file'):
                metrics.load_report(path)

    def test_incomplete_forged_summary_and_changed_raw_observations_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path, report = self.make_report(Path(directory))
            incomplete = dict(report, complete=False)
            forged = copy.deepcopy(report)
            forged['summaries']['execution']['wasmtime']['case.js']['execution_ns']['median'] += 1
            changed = copy.deepcopy(report)
            changed['execution']['wasmtime']['cases'][0]['observations'][1]['stdout'] = 'wrong\n'
            for candidate in (incomplete, forged, changed):
                path.write_text(json.dumps(candidate))
                with self.assertRaises(ValueError):
                    metrics.load_report(path)

    def test_comparison_allows_compiler_changes_but_requires_same_work_and_machine(self):
        with tempfile.TemporaryDirectory() as directory:
            _, before = self.make_report(Path(directory))
            after = copy.deepcopy(before)
            after['source_commit'] = 'new'
            self.assertTrue(all(row['ratio'] in (None, 1) for row in metrics.compare(before, after)['changes']))
            for key, value in [('configuration', {'warmups': 2}), ('machine', {'cpu': 'other'}),
                               ('workloads', []), ('harness_sha256', {}), ('profile', 'debug'),
                               ('node_versions', {'v8': 'different'})]:
                with self.subTest(key=key), self.assertRaisesRegex(ValueError, 'incompatible'):
                    metrics.compare(before, dict(after, **{key: value}))

    def test_missing_memory_never_becomes_a_zero_rss_summary(self):
        with tempfile.TemporaryDirectory() as directory:
            _, report = self.make_report(Path(directory))
            report['execution']['wasmtime']['cases'][0]['observations'][2]['memory']['process_rss'] = dict(status='unavailable', reason='read-failed')
            self.assertIsNone(metrics.summarize(report)['execution']['wasmtime']['case.js']['sampled_invocation_max_rss_bytes'])
            report['summaries'] = metrics.summarize(report)
            before = copy.deepcopy(report)
            before['summaries']['execution']['wasmtime']['case.js']['sampled_invocation_max_rss_bytes'] = metrics.summary([1, 2, 3])
            comparison = metrics.compare(before, report)
            row = next(p for p in comparison['changes'] if p['metric'].endswith('sampled_invocation_max_rss_bytes'))
            self.assertTrue(row['availability_changed'])
            self.assertIsNone(row['ratio'])

    def test_complete_report_requires_both_runtimes_and_exact_commands(self):
        with tempfile.TemporaryDirectory() as directory:
            path, report = self.make_report(Path(directory))
            missing_runtime = copy.deepcopy(report)
            del missing_runtime['execution']['v8']
            missing_runtime['summaries'] = metrics.summarize(missing_runtime)
            missing_command = copy.deepcopy(report)
            missing_command['commands'] = [p for p in missing_command['commands'] if p['label'] != 'execution-v8-case.js']
            bad_label = copy.deepcopy(report)
            bad_label['execution']['v8']['cases'][0]['command_label'] = 'execution-wasmtime-case.js'
            for candidate in (missing_runtime, missing_command, bad_label):
                path.write_text(json.dumps(candidate))
                with self.assertRaises(ValueError):
                    metrics.load_report(path)

    def test_workload_selection_retains_manifest_order_and_rejects_unknown_cases(self):
        names = [c['file'] for c in BENCH.workload_plan(FIXTURES, ['retained-objects.js', 'integer.js'])['execution']]
        self.assertEqual(names, ['integer.js', 'retained-objects.js'])
        for selected in (['missing.js'], ['integer.js', 'integer.js']):
            with self.assertRaises(ValueError):
                BENCH.workload_plan(FIXTURES, selected)

    def test_workload_manifest_rejects_escaping_or_duplicate_sources(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'safe.js').write_text('print(42)')
            plan = dict(schema='scriptw-v8-workloads-v1', startup=dict(file='safe.js', stdout='42\n'),
                        execution=[dict(file='../escape.js', stdout='42\n')])
            for name in ('../escape.js', 'safe.js'):
                plan['execution'][0]['file'] = name
                (root / 'manifest.json').write_text(json.dumps(plan))
                with self.assertRaises(ValueError):
                    BENCH.workload_plan(root, None)

    @unittest.skipUnless(shutil.which('node'), 'Node required for reference harness control')
    def test_real_v8_outputs_and_memory_endpoints_for_every_workload(self):
        plan = json.loads((FIXTURES / 'manifest.json').read_text())
        for case in plan['execution']:
            result = json.loads(subprocess.check_output(['node', str(FIXTURES / 'node-runtime.cjs'),
                str(FIXTURES), '3', '1', case['file']], text=True))
            observations = result['cases'][0]['observations']
            self.assertEqual(len(observations), 4)
            self.assertEqual([p['warmup'] for p in observations], [True, False, False, False])
            for sample in observations:
                self.assertEqual(sample['stdout'], case['stdout'])
                self.assertGreater(sample['execution_ns'], 0)
                self.assertGreater(sample['memory']['after']['rss'], 0)
                self.assertGreater(sample['memory']['after']['heapUsed'], 0)


if __name__ == '__main__':
    unittest.main()
