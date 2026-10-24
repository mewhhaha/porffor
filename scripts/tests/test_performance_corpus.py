"""Generated benchmark checksums are independently executed by normal V8."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import performance_corpus as corpus


class CorpusTests(unittest.TestCase):
    def test_same_realm_tuning_and_holdout_checksums_match_v8(self):
        node = shutil.which('node')
        self.assertIsNotNone(node, 'normal V8 is required for batched corpus validation')
        entries = [entry for batches in (1, 9, 32) for entry in corpus.same_realm_workloads(batches)]
        self.assertEqual({entry['partition'] for entry in entries}, {'tuning', 'holdout'})
        self.assertEqual(len({entry['file'] for entry in entries}), len(entries))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'batches.cjs'
            path.write_text("const vm=require('node:vm');const cases=" + json.dumps(entries) + ";\n"
                "for(const e of cases){let out='';vm.runInNewContext(e.source,{print:x=>out+=String(x)+'\\n'});"
                "if(out!==e.stdout)throw new Error(e.file+': '+JSON.stringify(out)+' != '+JSON.stringify(e.stdout));}\n")
            result = subprocess.run([node, path], capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_same_realm_batch_bounds_and_phase_separation(self):
        for batches in (0, -1, 257, True, 1.5):
            with self.subTest(batches=batches), self.assertRaises(ValueError):
                corpus.same_realm_workloads(batches)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'corpus'
            with self.assertRaisesRegex(ValueError, 'separate corpora'):
                corpus.write_corpus(path, (8,), (2,), (1,), row_phases=True, sustained_batches=8)
            self.assertFalse(path.exists())
            plan = corpus.write_corpus(path, (8,), (2,), (1,), sustained_batches=8)
            self.assertEqual(len(plan['execution']), 10)
            self.assertTrue(all(entry['parameters']['batches'] == 8 for entry in plan['execution']))

    def test_row_phase_checksums_and_identity_guard_match_v8(self):
        node = shutil.which('node')
        self.assertIsNotNone(node, 'normal V8 is required for phase corpus validation')
        entries = corpus.row_phase_workloads((8, 17), (1, 4), (0, 1, 3))
        changed = dict(entries[-1], expected_identity_failure=True)
        end = 'print("scriptw:rows:traversal:end");'
        changed['source'] = changed['source'].replace(
            end, 'first.values = first.values.slice();\n' + end, 1)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'phases.cjs'
            path.write_text("const vm=require('node:vm');const cases=" + json.dumps(entries + [changed]) + ";\n"
                "for(const e of cases){let out='',failure;try{vm.runInNewContext(e.source,{print:x=>out+=String(x)+'\\n'});}"
                "catch(error){failure=error;}if(e.expected_identity_failure){"
                "if(!failure||failure.message!=='retained row identity changed during traversal')throw new Error('identity guard missed changed array');"
                "}else if(failure||out!==e.stdout)throw new Error(e.file+': '+String(failure)+' '+JSON.stringify(out));}\n")
            result = subprocess.run([node, path], capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_all_family_checksums_match_v8(self):
        node = shutil.which('node')
        self.assertIsNotNone(node, 'normal V8 is required for performance corpus validation')
        entries = corpus.workloads(rows=(8, 17), widths=(1, 4), passes=(0, 1, 3))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'checks.cjs'
            # One independent context per source, matching the ordinary V8 driver.
            path.write_text("const vm=require('node:vm');const cases=" + json.dumps(entries) + ";\n"
                "for(const e of cases){let out='';vm.runInNewContext(e.source,{print:x=>out+=String(x)+'\\n'});"
                "if(out!==e.stdout)throw new Error(e.file+': '+JSON.stringify(out)+' != '+JSON.stringify(e.stdout));}\n")
            result = subprocess.run([node, path], capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_written_corpus_is_deterministic_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as directory:
            first, second = Path(directory) / 'a', Path(directory) / 'b'
            a = corpus.write_corpus(first, (8,), (2,), (0, 1))
            b = corpus.write_corpus(second, (8,), (2,), (0, 1))
            self.assertEqual(a, b)
            self.assertEqual({p.name: p.read_bytes() for p in first.iterdir()},
                             {p.name: p.read_bytes() for p in second.iterdir()})
            with self.assertRaises(FileExistsError):
                corpus.write_corpus(first, (8,), (2,), (0, 1))
            self.assertTrue(all('source' not in entry for entry in a['execution']))

    def test_invalid_scale_plans_are_rejected(self):
        for value in ('1,1', '-1', '999999', ''):
            with self.subTest(value=value), self.assertRaises(ValueError):
                corpus.bounded_list(value, 16384)
