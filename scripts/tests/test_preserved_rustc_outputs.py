"""Fail-closed output ownership checks, independent of product code generation."""
import hashlib
import json
import os
import runpy
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

WRAPPER = Path(__file__).resolve().parents[1] / 'test-shared-runtime-images/preserve-output-rustc'
guard = runpy.run_path(str(WRAPPER))
protect = guard['protect_outputs']

class OutputProtection(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.target = self.root / 'target'
        self.deps = self.target / 'release/deps'
        self.deps.mkdir(parents=True)
        self.store = self.root / 'history'
        self.store.mkdir()
        self.inventory = self.root / 'inventory.json'
        self.log = self.root / 'actions.jsonl'
        self.records = []
        self.args = ['--crate-name', 'chosen', '--out-dir', str(self.deps), '-C', 'extra-filename=-123',
                     '--emit=dep-info,metadata,link']

    def link(self, name, data=b'preserved bytes', directory=None):
        path = (directory or self.deps) / name
        storage = self.store / name
        storage.write_bytes(data)
        path.symlink_to(storage)
        record = dict(original=str(path), storage=str(storage), sha256=hashlib.sha256(data).hexdigest())
        self.records.append(record)
        return path, storage, record

    def run_guard(self, arguments=None):
        self.inventory.write_text(json.dumps(dict(cargo_target_dir=str(self.target), files=self.records)))
        return protect(self.args if arguments is None else arguments, self.inventory, self.log)

    def test_dirty_unit_detaches_only_its_owned_aliases_and_keeps_bytes(self):
        chosen, payload, _ = self.link('libchosen-123.rlib')
        fresh, other, _ = self.link('libother-456.rlib')
        variant, variant_data, _ = self.link('libchosen-999.rlib')
        rows = self.run_guard()
        self.assertEqual([r['original'] for r in rows], [str(chosen)])
        self.assertFalse(chosen.is_symlink())
        self.assertEqual(payload.read_bytes(), b'preserved bytes')
        self.assertTrue(fresh.is_symlink() and variant.is_symlink())
        self.assertEqual(other.read_bytes(), variant_data.read_bytes())
        self.assertEqual(json.loads(self.log.read_text())['state'], 'detach-intent')

    def test_all_aliases_validate_before_any_detachment(self):
        first, _, _ = self.link('libchosen-123.rlib')
        bad, _, record = self.link('libchosen-123.rmeta')
        record['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'bytes changed'):
            self.run_guard()
        self.assertTrue(first.is_symlink() and bad.is_symlink())
        self.assertFalse(self.log.exists())

    def test_metadata_only_unit_keeps_its_unwritten_link_output(self):
        linked, _, _ = self.link('libchosen-123.rlib')
        metadata, payload, _ = self.link('libchosen-123.rmeta')
        arguments = [x for x in self.args if not x.startswith('--emit=')] + ['--emit=metadata']
        rows = self.run_guard(arguments)
        self.assertEqual([r['original'] for r in rows], [str(metadata)])
        self.assertTrue(linked.is_symlink())
        self.assertFalse(metadata.is_symlink())
        self.assertEqual(payload.read_bytes(), b'preserved bytes')

    def test_changed_link_owner_is_rejected(self):
        path, _, _ = self.link('libchosen-123.rlib')
        intruder = self.store / 'intruder'
        intruder.write_bytes(b'preserved bytes')
        path.unlink()
        path.symlink_to(intruder)
        with self.assertRaisesRegex(ValueError, 'ownership'):
            self.run_guard()
        self.assertTrue(path.is_symlink())
        self.assertFalse(self.log.exists())

    def test_regular_output_is_never_deleted(self):
        path, payload, _ = self.link('libchosen-123.rlib')
        path.unlink()
        path.write_bytes(b'new producer output')
        self.assertEqual(self.run_guard(), [])
        self.assertEqual(path.read_bytes(), b'new producer output')
        self.assertEqual(payload.read_bytes(), b'preserved bytes')

    def test_queries_never_read_or_mutate_inventory(self):
        for args in [self.args + ['--print=cfg'], self.args + ['--print', 'file-names'], ['-vV']]:
            self.assertEqual(protect(args, self.inventory, self.log), [])
        self.assertFalse(self.log.exists())

    def test_outside_inventory_cannot_remove_external_link(self):
        path, _, _ = self.link('chosen-123', directory=self.root)
        with self.assertRaisesRegex(ValueError, 'outside'):
            self.run_guard()
        self.assertTrue(path.is_symlink())

    def test_canonical_executable_and_hashed_alias_preserve_same_payload(self):
        hashed, payload, record = self.link('chosen-123')
        canonical = self.deps.parent / 'chosen'
        canonical.symlink_to(payload)
        self.records.append(dict(record, original=str(canonical)))
        self.assertEqual(len(self.run_guard()), 2)
        self.assertFalse(hashed.is_symlink() or canonical.is_symlink())
        self.assertEqual(payload.read_bytes(), b'preserved bytes')

    def test_unwritable_journal_fails_before_detachment(self):
        path, _, _ = self.link('libchosen-123.rlib')
        self.log.mkdir()
        with self.assertRaises(OSError):
            self.run_guard()
        self.assertTrue(path.is_symlink())

    def test_actual_wrapper_preserves_even_test_and_incremental_arguments(self):
        compiler = self.root / 'compiler.py'
        compiler.write_text('import json,sys; print(json.dumps(sys.argv[1:]))\n')
        arguments = ['--crate-name', 'chosen', '--test', '-Cincremental=unchanged', '--print=cfg']
        environment = dict(os.environ, SCRIPTW_TEST_NO_INCREMENTAL='1')
        for name in ['SCRIPTW_TEST_CODEGEN_JOBS', 'SCRIPTW_PRESERVED_OUTPUT_INVENTORY',
                     'SCRIPTW_OUTPUT_PROTECTION_LOG']:
            environment.pop(name, None)
        result = subprocess.run([sys.executable, str(WRAPPER), sys.executable,
                                 str(compiler), *arguments], env=environment,
                                check=True, capture_output=True, text=True)
        self.assertEqual(json.loads(result.stdout), arguments)

    def test_native_wrapper_composes_protection_with_its_existing_test_policy(self):
        path, payload, _ = self.link('libchosen-123.rlib')
        self.inventory.write_text(json.dumps(dict(cargo_target_dir=str(self.target), files=self.records)))
        compiler = self.root / 'compiler.py'
        compiler.write_text('import json,sys; print(json.dumps(sys.argv[1:]))\n')
        arguments = self.args + ['--test', '-Cincremental=existing-test-scratch']
        environment = dict(os.environ, SCRIPTW_PRESERVED_OUTPUT_INVENTORY=str(self.inventory),
                           SCRIPTW_OUTPUT_PROTECTION_LOG=str(self.log))
        environment.pop('SCRIPTW_TEST_CODEGEN_JOBS', None)
        result = subprocess.run([sys.executable, str(WRAPPER.with_name('test-rustc')),
                                 sys.executable, str(compiler), *arguments],
                                env=environment, check=True, capture_output=True, text=True)
        self.assertEqual(json.loads(result.stdout), self.args + ['--test'])
        self.assertFalse(path.is_symlink())
        self.assertEqual(payload.read_bytes(), b'preserved bytes')

    def test_both_wrappers_reject_incomplete_protection_environment(self):
        path, _, _ = self.link('libchosen-123.rlib')
        for wrapper in [WRAPPER, WRAPPER.with_name('test-rustc')]:
            environment = dict(os.environ, SCRIPTW_PRESERVED_OUTPUT_INVENTORY=str(self.inventory))
            for name in ['SCRIPTW_OUTPUT_PROTECTION_LOG', 'SCRIPTW_TEST_CODEGEN_JOBS']:
                environment.pop(name, None)
            result = subprocess.run([sys.executable, str(wrapper), sys.executable, '-c',
                                     'raise RuntimeError("must not execute compiler")', *self.args],
                                    env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn('both inventory and log paths', result.stderr)
            self.assertTrue(path.is_symlink())

if __name__ == '__main__':
    unittest.main()
