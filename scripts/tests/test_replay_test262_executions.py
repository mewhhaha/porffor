"""Failure-only replay must preserve exact identities and reject incomplete reports."""

import importlib.util
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "replay-test262-executions.py"
SPEC = importlib.util.spec_from_file_location("execution_replay", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def report(outcome="Success"):
    return "total: 1\noutcomes:\n" + "".join(
        f"  {name}: {int(name == outcome)}\n" for name in MODULE.OUTCOMES
    )


class ExecutionReplayTests(unittest.TestCase):
    def test_accepts_every_native_outcome_without_counting_failures_as_passes(self):
        for outcome in MODULE.OUTCOMES:
            with self.subTest(outcome=outcome):
                self.assertEqual(MODULE.native_outcome(report(outcome), int(outcome != "Success")), outcome)

    def test_rejects_partial_duplicate_or_inconsistent_reports_and_exit_codes(self):
        valid = report()
        for transcript, exit_code in [
            ("", 0), (valid.replace("total: 1", "total: 0"), 0),
            (valid + "total: 1\n", 0), (valid + "  Success: 1\n", 0),
            (valid.replace("  Crash: 0\n", ""), 0),
            (valid.replace("  Crash: 0", "  Crash: 1"), 0),
            (valid, 1), (valid, -9), (report("Bug"), 0), (report("Bug"), 2),
        ]:
            with self.subTest(transcript=transcript, exit_code=exit_code), self.assertRaises(ValueError):
                MODULE.native_outcome(transcript, exit_code)

    def test_execution_list_keeps_modes_distinct_and_rejects_empty_duplicates_and_filters(self):
        identities = [f"{mode}:built-ins/Array/case.js" for mode in MODULE.EXECUTION_MODES]
        self.assertEqual(MODULE.parse_executions("# frozen cohort\n\n" + "\n".join(identities)), identities)
        for invalid in ["# empty\n", identities[0] + "\n" + identities[0],
                        "built-ins/Array", "invalid:case.js", "module:two paths.js"]:
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                MODULE.parse_executions(invalid)

    def test_existing_evidence_directory_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executions = root / "executions"
            executions.write_text("sloppy-script:case.js\n")
            evidence = root / "evidence"
            evidence.mkdir()
            marker = evidence / "original.log"
            marker.write_text("original result\n")
            result = subprocess.run(
                [sys.executable, str(SCRIPT), str(executions), "--binary", sys.executable,
                 "--suite-root", str(root), "--output-dir", str(evidence)],
                capture_output=True, text=True, timeout=10,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(marker.read_text(), "original result\n")
            self.assertEqual(list(evidence.iterdir()), [marker])

    def test_replay_freezes_its_inputs_and_forces_case_isolation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executions = root / "executions"
            execution_list = "sloppy-script:first.js\nstrict-script:second.js\n"
            executions.write_text(execution_list)
            binary = root / "fake-compiler"
            program = (
                f"#!{sys.executable}\n"
                "import os\nfrom pathlib import Path\n"
                "assert os.environ['LILA_TEST262_FORCE_CASE_RUNNER'] == '1'\n"
                "assert 'LILA_TEST262_DISABLE_CASE_RUNNER' not in os.environ\n"
                f"Path({str(binary)!r}).write_text('replacement compiler')\n"
                f"Path({str(executions)!r}).write_text('replacement executions')\n"
                f"print({report()!r})\n"
            )
            binary.write_text(program)
            binary.chmod(0o755)
            evidence = root / "evidence"
            environment = os.environ.copy()
            environment["LILA_TEST262_DISABLE_CASE_RUNNER"] = "1"
            result = subprocess.run(
                [sys.executable, str(SCRIPT), str(executions), "--binary", str(binary),
                 "--suite-root", str(root), "--output-dir", str(evidence), "--workers", "1"],
                capture_output=True, text=True, timeout=10, env=environment,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            summary = json.loads((evidence / "summary.json").read_text())
            self.assertEqual(summary["outcomes"]["Success"], 2)
            self.assertEqual(summary["binary_source"], str(binary))
            self.assertEqual(summary["binary_sha256"], hashlib.sha256(program.encode()).hexdigest())
            self.assertEqual(Path(summary["binary"]).read_text(), program)
            self.assertEqual(binary.read_text(), "replacement compiler")
            self.assertEqual(summary["execution_list_source"], str(executions))
            self.assertEqual(Path(summary["execution_list"]).read_text(), execution_list)
            self.assertEqual(summary["execution_list_sha256"], hashlib.sha256(execution_list.encode()).hexdigest())


if __name__ == "__main__":
    unittest.main()
