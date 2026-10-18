"""Libtest scratch controls preserve normal builds and actual compiler verdicts."""

import importlib.machinery
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

SOURCE = Path(__file__).resolve().parents[1] / "test-shared-runtime-images" / "test-rustc"
loader = importlib.machinery.SourceFileLoader("test_rustc_cache", str(SOURCE))
spec = importlib.util.spec_from_loader(loader.name, loader)
wrapper = importlib.util.module_from_spec(spec)
loader.exec_module(wrapper)


class WorkspaceRustcCacheTests(unittest.TestCase):
    def test_default_environment_is_literal_and_never_allocates_tokens(self):
        environment = {"CARGO_MAKEFLAGS": "original", "MAKEFLAGS": "kept",
                       "MFLAGS": "also kept", "RUSTFLAGS": "unchanged"}
        with mock.patch.object(wrapper, "codegen_capacity") as capacity:
            actual, descriptors = wrapper.compiler_environment(
                ["/rustc", "--crate-name", "sample"], environment)
        self.assertEqual(actual, environment)
        self.assertEqual(descriptors, ())
        capacity.assert_not_called()

    def test_codegen_queries_preserve_the_original_environment(self):
        environment = {"LILA_TEST_CODEGEN_JOBS": "auto", "CARGO_MAKEFLAGS": "original"}
        for arguments in [["/rustc", "-vV"],
                          ["/rustc", "--crate-name", "query", "--print=file-names"],
                          ["/rustc", "--crate-name", "query", "--print", "cfg"]]:
            with mock.patch.object(wrapper, "codegen_capacity") as capacity:
                self.assertEqual(wrapper.compiler_environment(arguments, environment),
                                 (environment, ()))
            capacity.assert_not_called()

    def test_codegen_requires_an_explicit_valid_option_and_serial_cargo(self):
        arguments = ["/rustc", "--crate-name", "sample"]
        for value in ("", "0", "3", "unbounded"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                wrapper.compiler_environment(arguments, {"LILA_TEST_CODEGEN_JOBS": value})
        for value in (None, "2", "auto"):
            environment = {"LILA_TEST_CODEGEN_JOBS": "auto"}
            if value is not None:
                environment["CARGO_BUILD_JOBS"] = value
            with self.subTest(cargo_jobs=value), self.assertRaises(ValueError):
                wrapper.compiler_environment(arguments, environment)

    def test_one_cpu_keeps_the_existing_jobserver_without_allocating_fds(self):
        environment = {"LILA_TEST_CODEGEN_JOBS": "auto", "CARGO_BUILD_JOBS": "1",
                       "CARGO_MAKEFLAGS": "original"}
        with mock.patch.object(wrapper, "codegen_capacity", return_value=1), \
                mock.patch.object(wrapper.os, "pipe") as pipe:
            self.assertEqual(wrapper.compiler_environment(
                ["/rustc", "--crate-name", "sample"], environment), (environment, ()))
        pipe.assert_not_called()

    def test_private_jobserver_has_exactly_one_token_per_additional_cpu(self):
        environment = {"LILA_TEST_CODEGEN_JOBS": "auto", "CARGO_BUILD_JOBS": "1",
                       "CARGO_MAKEFLAGS": "original", "MAKEFLAGS": "old",
                       "MFLAGS": "old", "RUSTFLAGS": "literal"}
        with mock.patch.object(wrapper, "codegen_capacity", return_value=3):
            actual, descriptors = wrapper.compiler_environment(
                ["/rustc", "--crate-name", "sample"], environment)
        try:
            reader, writer = descriptors
            self.assertTrue(all(os.get_inheritable(fd) for fd in descriptors))
            self.assertEqual(actual["CARGO_MAKEFLAGS"],
                             f"-j --jobserver-fds={reader},{writer} --jobserver-auth={reader},{writer}")
            self.assertNotIn("MAKEFLAGS", actual)
            self.assertNotIn("MFLAGS", actual)
            self.assertEqual(actual["RUSTFLAGS"], "literal")
            self.assertEqual(environment["CARGO_MAKEFLAGS"], "original")
            os.set_blocking(reader, False)
            self.assertEqual(os.read(reader, 100), b"++")
            with self.assertRaises(BlockingIOError):
                os.read(reader, 1)
        finally:
            for descriptor in descriptors:
                os.close(descriptor)

    def test_budget_failure_and_invalid_cpu_capacity_never_allocate_tokens(self):
        environment = {"LILA_TEST_CODEGEN_JOBS": "auto", "CARGO_BUILD_JOBS": "1"}
        with mock.patch.object(wrapper, "codegen_capacity", side_effect=ValueError("budget")), \
                mock.patch.object(wrapper.os, "pipe") as pipe:
            with self.assertRaisesRegex(ValueError, "budget"):
                wrapper.compiler_environment(["/rustc", "--crate-name", "sample"], environment)
        pipe.assert_not_called()
        with mock.patch.object(wrapper, "codegen_capacity", return_value=0):
            with self.assertRaises(ValueError):
                wrapper.compiler_environment(["/rustc", "--crate-name", "sample"], environment)

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

    def test_intl_retention_never_retains_libtest_or_other_crate_scratch(self):
        environment = {"LILA_TEST_NO_INCREMENTAL": "1", "LILA_TEST_KEEP_INTL_INCREMENTAL": "1"}
        for crate in ("lila_intl", "lila_intl_collator_data", "lila_ir", "build_script_build"):
            for test in (False, True):
                arguments = ["/rustc", "--crate-name", crate, "file.rs", "-C",
                             "incremental=/cache", "-Copt-level=2"] + (["--test"] if test else [])
                actual = wrapper.command_from_environment(arguments, environment)
                expected = arguments if crate == "lila_intl" and not test else [
                    word for word in arguments if word not in ("-C", "incremental=/cache")]
                self.assertEqual(actual, expected, (crate, test))
        for query in (["/rustc", "-vV"], ["/rustc", "--print", "cfg"]):
            self.assertEqual(wrapper.command_from_environment(query, environment), query)

    def test_intl_retention_rejects_ambiguous_environment_values(self):
        for value in ("", "0", "true", "lila_intl", "all"):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "must be 1"):
                wrapper.command_from_environment(["/rustc"], {"LILA_TEST_KEEP_INTL_INCREMENTAL": value})

    def test_executed_intl_retention_preserves_compiler_arguments_and_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            compiler = Path(directory) / "compiler"
            compiler.write_text("#!/usr/bin/env python3\nimport json,sys\n"
                                "print(json.dumps(sys.argv[1:]))\nsys.exit(37)\n")
            compiler.chmod(0o755)
            environment = dict(os.environ, LILA_TEST_NO_INCREMENTAL="1",
                               LILA_TEST_KEEP_INTL_INCREMENTAL="1")
            environment.pop("LILA_TEST_CODEGEN_JOBS", None)
            for test in (False, True):
                arguments = ["--crate-name", "lila_intl", "-Cincremental=/cache",
                             "-Copt-level=2"] + (["--test"] if test else [])
                result = subprocess.run([str(SOURCE), str(compiler), *arguments],
                                        capture_output=True, text=True, env=environment)
                self.assertEqual(result.returncode, 37)
                expected = arguments if not test else [w for w in arguments if not w.startswith("-Cincremental=")]
                self.assertEqual(json.loads(result.stdout), expected)

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
