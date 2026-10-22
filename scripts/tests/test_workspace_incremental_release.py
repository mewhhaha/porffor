"""Cleanup may overlap builds only when it touches no caches."""
import ast
import fcntl
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[1] / 'run_workspace_tests.py'

class IncrementalReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.profile=Path(self.temp.name)
        tree=ast.parse(SOURCE.read_text())
        node=next(n for n in ast.walk(tree) if isinstance(n,ast.FunctionDef) and n.name=='release_incremental')
        namespace=dict(profile_root=self.profile,fcntl=fcntl,shutil=shutil)
        exec(compile(ast.Module(body=[node],type_ignores=[]),str(SOURCE),'exec'),namespace)
        self.release=namespace['release_incremental']

    def test_missing_or_empty_cache_never_waits_for_cargo(self):
        for create in (False,True):
            if create:(self.profile/'incremental').mkdir()
            with patch.object(fcntl,'flock',side_effect=AssertionError('unnecessary Cargo lock')):
                row=dict(target='aot-language',kind='test');self.release(row)
            self.assertEqual(row['released_incremental'],[])
            self.assertFalse((self.profile/'.cargo-lock').exists())

    def test_library_namespace_is_preserved_without_lock(self):
        cache=self.profile/'incremental/scriptw_engine-library';cache.mkdir(parents=True)
        with patch.object(fcntl,'flock',side_effect=AssertionError('library cleanup is forbidden')):
            row=dict(target='scriptw_engine',kind='lib');self.release(row)
        self.assertTrue(cache.is_dir());self.assertEqual(row['released_incremental'],[])

    def test_real_cleanup_holds_exclusive_lock_and_preserves_other_targets(self):
        selected=self.profile/'incremental/aot_language-selected';selected.mkdir(parents=True)
        other=self.profile/'incremental/other-target';other.mkdir()
        original=shutil.rmtree;held=False
        def lock(fd,mode):
            nonlocal held
            self.assertEqual(mode,fcntl.LOCK_EX);held=True
        def remove(path):
            self.assertTrue(held);original(path)
        with patch.object(fcntl,'flock',side_effect=lock) as acquire, patch.object(shutil,'rmtree',side_effect=remove):
            row=dict(target='aot-language',kind='test');self.release(row)
        self.assertEqual(acquire.call_count,1)
        self.assertEqual(row['released_incremental'],['aot_language-selected'])
        self.assertFalse(selected.exists());self.assertTrue(other.is_dir())

    def test_candidates_are_discovered_again_after_acquiring_lock(self):
        first=self.profile/'incremental/aot_language-first';first.mkdir(parents=True)
        second=first.with_name('aot_language-created-while-waiting')
        def lock(fd,mode):second.mkdir()
        with patch.object(fcntl,'flock',side_effect=lock):
            row=dict(target='aot_language',kind='test');self.release(row)
        self.assertEqual(set(row['released_incremental']),{first.name,second.name})
        self.assertFalse(first.exists());self.assertFalse(second.exists())

if __name__=='__main__':unittest.main()
