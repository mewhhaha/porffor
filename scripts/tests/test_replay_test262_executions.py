"""Failure-only replay must preserve exact identities and reject incomplete reports."""

import importlib.util
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
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "executions"
            identities = [f"{mode}:built-ins/Array/case.js" for mode in MODULE.EXECUTION_MODES]
            path.write_text("# frozen cohort\n\n" + "\n".join(identities) + "\n")
            self.assertEqual(MODULE.read_executions(path), identities)
            for invalid in ["# empty\n", identities[0] + "\n" + identities[0],
                            "built-ins/Array", "invalid:case.js", "module:two paths.js"]:
                path.write_text(invalid)
                with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                    MODULE.read_executions(path)

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


if __name__ == "__main__":
    unittest.main()
