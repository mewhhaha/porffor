"""T25: probe observations must not become fabricated conformance evidence."""

import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location("feature_probe", Path(__file__).resolve().parents[1] / "feature-probe.py")
probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probe)
CASE = ("example", 10, "print(42);", False, "42")


class FeatureProbeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.out = self.root / "out"
        self.out.mkdir()
        self.lila = self.root / "lila"
        self.lila.touch()
        for name, value in (("OUT", self.out), ("LILA", self.lila)):
            patcher = mock.patch.object(probe, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def result(self, stdout="", stderr="", status=0):
        completed = subprocess.CompletedProcess([], status, stdout, stderr)
        with mock.patch.object(probe.subprocess, "run", return_value=completed) as run:
            result = probe.run(CASE)
        self.assertEqual(run.call_args.args[0][1:4], ["run", "--execution-backend", "wasm"])
        return result

    def test_stdout_answer_is_required_and_stderr_cannot_substitute(self):
        result = self.result(stderr="42\n")
        self.assertFalse(result[2])
        self.assertIn("missing stdout", result[3])

    def test_expected_stdout_succeeds_without_reading_stderr_as_output(self):
        self.assertEqual(self.result("\nrun outcome: Normal\n42\n", "warning\n"),
                         ("example", 10, True, ""))
        self.assertEqual((self.out / "example.js").read_text(), "print(42);\n")

    def test_nonzero_status_keeps_diagnostics_even_when_answer_matches(self):
        result = self.result("42\n", "compiler failed\n", 23)
        self.assertFalse(result[2])
        self.assertIn("exit 23", result[3])
        self.assertIn("compiler failed", result[3])

    def test_wrong_answer_and_empty_output_fail_with_details(self):
        for stdout in ("41\n", "", " \n"):
            with self.subTest(stdout=stdout):
                result = self.result(stdout)
                self.assertFalse(result[2])
                self.assertTrue(result[3])
                self.assertIn("42", result[3])

    def test_timeouts_and_launch_errors_are_visible_failures(self):
        for error, detail in ((subprocess.TimeoutExpired("lila", 120), "TIMEOUT"),
                              (PermissionError("not executable"), "launch error")):
            with self.subTest(error=error), mock.patch.object(probe.subprocess, "run", side_effect=error):
                result = probe.run(CASE)
                self.assertFalse(result[2])
                self.assertIn(detail, result[3])

    def test_diagnostics_are_bounded(self):
        result = self.result(stderr="x" * 10000, status=1)
        self.assertFalse(result[2])
        self.assertLessEqual(len(result[3]), 70)
        self.assertTrue(result[3].startswith("exit 1:"))

    def test_invalid_jobs_fail_before_creating_output_or_starting_workers(self):
        self.out.rmdir()
        for jobs in ("0", "-2"):
            with self.subTest(jobs=jobs), mock.patch.object(sys, "argv", ["feature-probe.py", "--jobs", jobs]), \
                    mock.patch.object(probe.concurrent.futures, "ThreadPoolExecutor") as pool, \
                    contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    probe.main()
                self.assertEqual(error.exception.code, 2)
                pool.assert_not_called()
                self.assertFalse(self.out.exists())

    def test_summary_does_not_call_all_failures_unsupported_or_counts_conformance(self):
        output = io.StringIO()
        with mock.patch.object(sys, "argv", ["feature-probe.py", "--jobs", "1"]), \
                mock.patch.object(probe, "PROBES", [CASE]), \
                mock.patch.object(probe, "run", return_value=("example", 10, False, "TIMEOUT")), \
                contextlib.redirect_stdout(output):
            self.assertEqual(probe.main(), 0)  # Diagnostic survey, not a release gate.
        text = output.getvalue()
        self.assertIn("failed:", text)
        self.assertIn("TIMEOUT", text)
        self.assertNotIn("unsupported:", text)
        self.assertIn("reference tag-count sum", text)
        self.assertIn("not Test262 conformance counts", text)

    def test_real_stderr_only_executable_cannot_pass(self):
        self.lila.write_text(f"#!{sys.executable} -S\nimport sys\nprint('42', file=sys.stderr)\n")
        self.lila.chmod(0o755)
        self.assertFalse(probe.run(CASE)[2])


if __name__ == "__main__":
    unittest.main()
