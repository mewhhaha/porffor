"""Independent output and accepted-bytecode controls for the new boundaries."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from boundary_corpus import supported_react_fixture, workloads

ROOT = Path(__file__).resolve().parents[2]


class BoundaryCorpusTests(unittest.TestCase):
    def test_all_tuning_and_holdout_function_outputs_match_normal_v8(self):
        node = shutil.which('node')
        self.assertIsNotNone(node, 'normal V8 is required for independent fixture checks')
        cases = workloads(7) + workloads(9) + [supported_react_fixture(ROOT, 7)]
        self.assertEqual({case['partition'] for case in cases}, {'tuning', 'holdout'})
        program = "const vm=require('node:vm');const cases=" + json.dumps(cases) + ";\n"
        program += "for(const c of cases)for(const mode of ['fresh','calls']){let out='';"
        program += "vm.runInNewContext(c[mode+'_source'],{print:x=>out+=String(x)+'\\n'},{timeout:30000});"
        program += "if(out!==c[mode+'_stdout'])throw Error(c.name+' '+mode+': '+JSON.stringify(out));}\n"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'controls.cjs'
            path.write_text(program)
            result = subprocess.run([node, path], capture_output=True, text=True, timeout=60)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_real_node_cached_data_is_accepted_and_corruption_cannot_run_source_fallback(self):
        node = shutil.which('node')
        self.assertIsNotNone(node)
        fixture = next(case for case in workloads(7) if case['name'] == 'closures')
        helper = ROOT / 'scripts/boundary_node.cjs'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, cache, expected = root / 'calls.js', root / 'calls.cache', root / 'expected'
            source.write_text(fixture['calls_source'])
            expected.write_text(fixture['calls_stdout'])
            def invoke(mode, extra=()):
                return subprocess.run([node, helper, mode, source, cache, expected, *extra],
                    capture_output=True, text=True, timeout=30)
            prepared = invoke('prepare')
            self.assertEqual(prepared.returncode, 0, prepared.stderr)
            source_result = subprocess.run([node, helper, 'source', source, expected],
                capture_output=True, text=True, timeout=30)
            self.assertEqual(source_result.returncode, 0, source_result.stderr)
            source_report = json.loads(source_result.stdout)
            self.assertFalse(source_report['cached_data_supplied'])
            self.assertIsNone(source_report['cached_data_rejected'])
            self.assertEqual(source_report['stdout'], fixture['calls_stdout'])
            measured = invoke('calls', ('7',))
            self.assertEqual(measured.returncode, 0, measured.stderr)
            report = json.loads(measured.stdout)
            self.assertIs(report['cached_data_rejected'], False)
            self.assertEqual(report['stdout'], fixture['calls_stdout'])
            self.assertEqual(len(report['calls']['samples_ns']), 7)
            cache.write_bytes(b'not valid V8 cachedData')
            rejected = invoke('fresh')
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn('source fallback is not a valid observation', rejected.stderr)

    def test_restored_supported_react_keeps_exact_html_and_pinned_sources_for_isolated_calls(self):
        fixture = supported_react_fixture(ROOT)
        self.assertIsInstance(fixture['calls_source'], str)
        self.assertEqual(fixture['guest_warmups'], 10)
        self.assertEqual(fixture['call_samples'], 7)
        self.assertIn('react/LICENSE', fixture['restored_assets'])
        self.assertIn('react-dom/LICENSE', fixture['restored_assets'])
        lines = [json.loads(line) for line in fixture['fresh_stdout'].splitlines()]
        self.assertEqual([line['name'] for line in lines],
            ['element', 'component-state-list-escaping', 'context-style-fragment'])
        self.assertEqual(lines[0]['html'], '<h1 id="hello">Hello Wasm</h1>')
        node = shutil.which('node')
        self.assertIsNotNone(node)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'react.cjs'
            path.write_text("const vm=require('node:vm');let out='';vm.runInNewContext(" +
                json.dumps(fixture['fresh_source']) + ",{print:x=>out+=String(x)+'\\n'});process.stdout.write(out);")
            result = subprocess.run([node, path], capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, fixture['fresh_stdout'])

    def test_sample_limits_are_checked(self):
        for samples in (0, 6, 101, True, 7.5):
            with self.subTest(samples=samples), self.assertRaises(ValueError):
                workloads(samples)
