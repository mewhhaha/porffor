"""Libtest scratch controls preserve normal builds and actual compiler verdicts."""

import importlib.machinery
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

SOURCE = Path(__file__).resolve().parents[1] / "test-shared-runtime-images" / "test-rustc"
loader = importlib.machinery.SourceFileLoader("test_rustc_cache", str(SOURCE))
spec = importlib.util.spec_from_loader(loader.name, loader)
wrapper = importlib.util.module_from_spec(spec)
loader.exec_module(wrapper)


class WorkspaceRustcCacheTests(unittest.TestCase):
    def test_normal_library_and_queries_retain_exact_arguments(self):
        for arguments in [["/rustc", "--crate-type", "rlib", "-C", "incremental=/cache"],
                          ["/rustc", "-vV"], ["/rustc", "--print", "cfg"]]:
            self.assertEqual(wrapper.compiler_command(arguments), arguments)

    def test_libtest_removes_only_incremental_storage_in_both_rustc_forms(self):
        arguments = ["/rustc", "--test", "file.rs", "-C", "incremental=/cache with spaces",
                     "-Copt-level=1", "-Cincremental=/other", "-C", "debuginfo=1",
                     "--cfg", "feature=\"kept\"", "-C"]
        self.assertEqual(wrapper.compiler_command(arguments),
                         ["/rustc", "--test", "file.rs", "-Copt-level=1", "-C",
                          "debuginfo=1", "--cfg", "feature=\"kept\"", "-C"])

    def test_empty_compiler_command_is_rejected(self):
        with self.assertRaises(ValueError):
            wrapper.compiler_command([])

    def test_opt_in_normal_units_omit_only_scratch_and_queries_stay_literal(self):
        arguments = ["/rustc", "--crate-name", "lila_ir", "file.rs", "--crate-type", "rlib",
                     "-C", "incremental=/cache", "-Copt-level=0", "--cfg", "feature=\"modules\""]
        self.assertEqual(wrapper.compiler_command(arguments, omit_normal_incremental=True),
                         ["/rustc", "--crate-name", "lila_ir", "file.rs", "--crate-type", "rlib",
                          "-Copt-level=0", "--cfg", "feature=\"modules\""])
        for query in [["/rustc", "-vV"], ["/rustc", "--print", "cfg"]]:
            self.assertEqual(wrapper.compiler_command(query, omit_normal_incremental=True), query)

    def test_executed_compiler_receives_literal_arguments_and_keeps_failure_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            compiler = Path(directory) / "compiler"
            compiler.write_text("#!/usr/bin/env python3\nimport json,sys\n"
                                "print(json.dumps(sys.argv[1:]))\nsys.exit(37)\n")
            compiler.chmod(0o755)
            result = subprocess.run([str(SOURCE), str(compiler), "--test", "-C",
                                     "incremental=/scratch", "literal $(kept)", "-Copt-level=1"],
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 37)
            self.assertEqual(result.stdout, '["--test", "literal $(kept)", "-Copt-level=1"]\n')


if __name__ == "__main__":
    unittest.main()
