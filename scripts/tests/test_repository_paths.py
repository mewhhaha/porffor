"""Portable unit tests plus a real Git-index integration test for the path gate."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "check-repository-paths.py"
SPEC = importlib.util.spec_from_file_location("repository_paths", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class PathTests(unittest.TestCase):
    def test_valid_names_and_repeated_parent_directories(self):
        self.assertEqual(MODULE.check_paths([
            "docs/early-error-taxonomy.md", "docs/with spaces.md",
            "src/caf\u00e9.rs", "src/\U0001f600.rs", ".github/workflows/ci.yaml",
        ]), [])

    def test_rejects_windows_invalid_characters(self):
        for char in '<>:"\\|?*\0\n\x1f':
            with self.subTest(char=repr(char)):
                self.assertTrue(MODULE.check_paths([f"docs/bad{char}name.md"]))

    def test_rejects_reserved_names_with_extensions_and_case_changes(self):
        for name in ["CON", "aux.txt", "NuL.tar.gz", "PRN", "COM1.js", "LPT9",
                     "com\u00b9.txt", "lpt\u00b2", "CONIN$", "conout$.txt", "NUL .txt"]:
            with self.subTest(name=name):
                self.assertTrue(MODULE.check_paths([f"src/{name}/file"]))
        self.assertEqual(MODULE.check_paths(["com0.txt", "lpt10", "console.rs"]), [])

    def test_rejects_trailing_spaces_and_periods(self):
        for path in ["dir /file", "dir./file", "file. ", "file."]:
            with self.subTest(path=path):
                self.assertTrue(MODULE.check_paths([path]))

    def test_rejects_case_collisions_in_files_and_directories(self):
        for paths in [["Foo.rs", "foo.rs"], ["SRC/a.rs", "src/b.rs"]]:
            with self.subTest(paths=paths):
                self.assertTrue(MODULE.check_paths(paths))

    def test_rejects_unicode_normalization_collisions(self):
        self.assertTrue(MODULE.check_paths(["caf\u00e9.rs", "cafe\u0301.rs"]))

    def test_rejects_empty_and_relative_components(self):
        for path in ["/absolute", "a//b", "../x", "a/./b", "a/"]:
            with self.subTest(path=path):
                self.assertTrue(MODULE.check_paths([path]))

    def test_rejects_invalid_unicode_without_crashing(self):
        self.assertTrue(MODULE.check_paths(["bad\udcffname"]))

    def test_checks_component_length_in_utf16_units(self):
        self.assertEqual(MODULE.check_paths(["x" * 255]), [])
        self.assertTrue(MODULE.check_paths(["x" * 256]))
        self.assertTrue(MODULE.check_paths(["\U0001f600" * 128]))

    def test_duplicate_index_entries_do_not_create_false_collisions(self):
        self.assertEqual(MODULE.check_paths(["src/a.rs", "src/a.rs"]), [])

    def test_reads_index_instead_of_worktree_and_preserves_spaces(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            tracked = root / "with spaces.txt"
            tracked.write_text("fixture\n", encoding="utf-8")
            subprocess.run(["git", "-C", str(root), "add", "--", tracked.name], check=True)
            tracked.unlink()
            (root / "untracked.txt").write_text("not in the index", encoding="utf-8")
            self.assertEqual(MODULE.tracked_paths(root), ["with spaces.txt"])
            run = subprocess.run([sys.executable, str(SCRIPT), "--root", str(root)],
                                 capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stderr)
            self.assertIn("1 tracked files checked", run.stdout)

    def test_empty_index_is_not_vacuously_green(self):
        with tempfile.TemporaryDirectory() as directory:
            subprocess.run(["git", "init", "-q", directory], check=True)
            run = subprocess.run([sys.executable, str(SCRIPT), "--root", directory],
                                 capture_output=True, text=True)
            self.assertEqual(run.returncode, 1)
            self.assertIn("empty tracked-path inventory", run.stderr)


if __name__ == "__main__":
    unittest.main()
