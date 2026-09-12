"""Replay audits reconcile retained native evidence without running a compiler."""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "audit-test262-replay.py"
OUTCOMES = ("Success", "NotImplemented", "Crash", "Bug")
MODES = ("sloppy-script", "strict-script", "raw-script", "module", "raw-module")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path, document):
    path.write_text(json.dumps(document))


class ReplayAuditTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.suite = self.root / "checkout" / "vendor" / "suite"
        (self.suite / "test").mkdir(parents=True)
        (self.suite / "test" / "case.js").write_text("/*--- flags: [] ---*/\ntrue;\n")
        self.repository = self.root / "checkout"
        self.git("init", "--quiet")
        self.git("add", ".")
        self.git("-c", "user.email=audit@example.invalid", "-c", "user.name=Audit",
                 "commit", "--quiet", "-m", "Create synthetic suite")
        self.pin = self.git("rev-parse", "HEAD:vendor/suite")
        self.evidence = self.root / "replay"
        (self.evidence / "snapshots").mkdir(parents=True)
        self.compiler = self.root / "compiler.json"
        self.output = self.root / "audit.json"
        (self.evidence / "compiler").write_bytes(b"frozen compiler, never executed")
        write_json(self.compiler, {"binary_sha256": digest(self.evidence / "compiler")})
        self.identities = [f"{mode}:case.js" for mode in MODES]
        (self.evidence / "executions").write_text("\n".join(self.identities) + "\n")
        self.run = {
            "binary": str(self.evidence / "compiler"), "binary_source": "/unavailable/old/compiler",
            "binary_sha256": digest(self.evidence / "compiler"), "suite_root": str(self.suite),
            "execution_list": str(self.evidence / "executions"), "execution_list_source": "/old/list",
            "execution_list_sha256": digest(self.evidence / "executions"),
        }
        write_json(self.evidence / "run.json", self.run)
        self.rows = []
        for index, execution in enumerate(self.identities):
            outcome = ("Success", "Success", "NotImplemented", "Crash", "Bug")[index]
            key = hashlib.sha256(execution.encode()).hexdigest()
            row = {"execution_id": execution, "outcome": outcome, "exit_code": int(outcome != "Success"),
                   "transcript": f"{key}.log"}
            self.rows.append(row)
            write_json(self.evidence / f"{key}.json", row)
            (self.evidence / f"{key}.log").write_text("total: 1\noutcomes:\n" + "".join(
                f"  {name}: {int(name == outcome)}\n" for name in OUTCOMES))
            failure = {"test_id": execution, "test_path": "case.js", "outcome": outcome, "kind": "Runtime"}
            snapshot = {
                "producer": "lila", "execution_backend": "wasm-aot", "run_kind": "full",
                "pinned_revisions": {"test262": self.pin}, "manifest_hash": index + 1,
                "total": 1, "passed": int(outcome == "Success"), "completed_test_ids": [execution],
                "counts_per_outcome": {name: int(name == outcome) for name in OUTCOMES},
                "counts_per_kind": {"Runtime": int(outcome != "Success")},
                "failures": [] if outcome == "Success" else [failure],
                "timeout_test_ids": [execution] if outcome == "Crash" else [],
            }
            write_json(self.evidence / "snapshots" / f"{key}-{index + 1}.json", snapshot)
        self.summary = {**self.run, "total": len(self.rows), "results": self.rows,
                        "outcomes": {name: sum(row["outcome"] == name for row in self.rows) for name in OUTCOMES}}
        self.save_summary()

    def git(self, *arguments):
        result = subprocess.run(["git", "-C", str(self.repository), *arguments],
                                text=True, capture_output=True, check=True)
        return result.stdout.strip()

    def save_summary(self):
        write_json(self.evidence / "summary.json", self.summary)

    def audit(self, *arguments):
        return subprocess.run([sys.executable, str(SCRIPT), str(self.evidence),
                               "--compiler", str(self.compiler), "--output", str(self.output), *arguments],
                              cwd=self.root, text=True, capture_output=True, timeout=15)

    def rejected(self, reason):
        result = self.audit()
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn(reason, result.stderr)
        self.assertNotIn("Traceback", result.stderr)
        self.assertFalse(self.output.exists())

    def snapshot(self, index=0):
        key = hashlib.sha256(self.identities[index].encode()).hexdigest()
        return next((self.evidence / "snapshots").glob(f"{key}-*.json"))

    def test_accepts_all_modes_and_outcomes_and_retains_timeout_truth(self):
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(self.output.read_text())
        self.assertEqual(report["total"], 5)
        self.assertEqual(report["outcomes"], {"Success": 2, "NotImplemented": 1, "Crash": 1, "Bug": 1})
        self.assertEqual(report["timeouts"], 1)
        self.assertEqual(report["test262_tree_pin"], self.pin)
        self.assertEqual(report["suite_root"], str(self.suite))
        self.assertEqual({row["execution_id"] for row in report["rows"]}, set(self.identities))
        self.assertTrue(all(row["source_sha256"] == digest(self.suite / "test" / "case.js") for row in report["rows"]))
        self.assertEqual(report["compiler_metadata_sha256"], digest(self.compiler))

    def test_compares_exact_origin_ids_and_sources_without_hiding_retained_failures(self):
        origin = self.root / "origin.json"
        write_json(origin, {"test262_tree_pin": self.pin, "total": 2, "outcomes": {"Bug": 1, "Success": 1},
                            "rows": [{"execution_id": self.identities[i], "outcome": outcome,
                                      "timeout": False,
                                      "source_sha256": digest(self.suite / "test" / "case.js")}
                                     for i, outcome in enumerate(["Bug", "Success"])]})
        result = self.audit("--origin", str(origin))
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(self.output.read_text())
        self.assertEqual(report["repaired_origin_failures"], 1)
        self.assertEqual(report["retained_origin_successes"], 1)
        self.assertEqual(report["origin_overlap"], 2)
        self.assertEqual(report["outcomes"]["Bug"], 1)

    def test_classifies_every_origin_repair_and_separates_timeout_rechecks(self):
        origin = self.root / "origin.json"
        for outcome, timeout in [("Bug", False), ("NotImplemented", False), ("Crash", False), ("Crash", True)]:
            with self.subTest(outcome=outcome, timeout=timeout):
                write_json(origin, {"test262_tree_pin": self.pin, "total": 1, "outcomes": {outcome: 1},
                                    "rows": [{"execution_id": self.identities[0], "outcome": outcome,
                                              "timeout": timeout,
                                              "source_sha256": digest(self.suite / "test" / "case.js")}]})
                result = self.audit("--origin", str(origin))
                self.assertEqual(result.returncode, 0, result.stderr)
                report = json.loads(self.output.read_text())
                self.assertEqual(report["repaired_origin_failures"], 1)
                self.assertEqual(report["repaired_origin_timeout_rechecks"], int(timeout))
                self.assertEqual(report["repaired_origin_non_timeout_failures"], int(not timeout))
                self.assertEqual(report["origin_transitions"], {f"{outcome} -> Success": 1})
                row = next(row for row in report["rows"] if row["execution_id"] == self.identities[0])
                self.assertEqual(row["origin_timeout"], timeout)

    def test_accepts_equivalent_historical_suite_commit_and_unrelated_dirty_files(self):
        path = self.snapshot()
        snapshot = json.loads(path.read_text())
        snapshot["pinned_revisions"]["test262"] = self.git("rev-parse", "HEAD")
        write_json(path, snapshot)
        (self.repository / "unrelated-source.rs").write_text("uncommitted product change")
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_accepts_retained_interrupted_attempt_without_counting_it(self):
        key = Path(self.rows[0]["transcript"]).stem
        (self.evidence / f"{key}.interrupted-123.log").write_text("partial attempt")
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(self.output.read_text())["total"], 5)

    def test_accepts_standalone_suite_checkout_commit_pin(self):
        self.repository = self.suite
        self.git("init", "--quiet")
        self.git("add", ".")
        self.git("-c", "user.email=audit@example.invalid", "-c", "user.name=Audit",
                 "commit", "--quiet", "-m", "Standalone suite")
        pin = self.git("rev-parse", "HEAD")
        for path in (self.evidence / "snapshots").glob("*.json"):
            snapshot = json.loads(path.read_text())
            snapshot["pinned_revisions"]["test262"] = pin
            write_json(path, snapshot)
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(self.output.read_text())["recorded_test262_pins"], [pin])

    def test_rejects_changed_frozen_binary(self):
        (self.evidence / "compiler").write_bytes(b"replacement")
        self.rejected("compiler binary SHA-256 mismatch")

    def test_rejects_compiler_metadata_mismatch(self):
        write_json(self.compiler, {"binary_sha256": "0" * 64})
        self.rejected("compiler binary SHA-256 mismatch")

    def test_rejects_changed_execution_list(self):
        (self.evidence / "executions").write_text(self.identities[0] + "\n")
        self.rejected("execution list SHA-256 mismatch")

    def test_rejects_duplicate_execution_list_even_when_its_hash_matches(self):
        (self.evidence / "executions").write_text("\n".join(self.identities + [self.identities[0]]))
        self.run["execution_list_sha256"] = digest(self.evidence / "executions")
        write_json(self.evidence / "run.json", self.run)
        self.summary.update(self.run)
        self.save_summary()
        self.rejected("duplicate identities")

    def test_rejects_duplicate_summary_rows(self):
        self.summary["results"].append(self.rows[0])
        self.save_summary()
        self.rejected("duplicate execution")

    def test_rejects_missing_mode_from_summary(self):
        self.summary["results"].pop()
        self.save_summary()
        self.rejected("summary execution membership: missing")

    def test_rejects_summary_metadata_mismatch(self):
        self.summary["suite_root"] = str(self.root)
        self.save_summary()
        self.rejected("summary metadata differs")

    def test_rejects_saved_result_mismatch(self):
        path = self.evidence / Path(self.rows[0]["transcript"]).with_suffix(".json")
        row = json.loads(path.read_text())
        row["outcome"] = "Bug"
        write_json(path, row)
        self.rejected("saved result differs")

    def test_rejects_transcript_with_inconsistent_exit_status(self):
        self.rows[0]["exit_code"] = 1
        write_json(self.evidence / Path(self.rows[0]["transcript"]).with_suffix(".json"), self.rows[0])
        self.save_summary()
        self.rejected("native outcome and command exit status disagree")

    def test_rejects_saved_outcome_different_from_native_transcript(self):
        self.rows[0]["outcome"] = "Bug"
        write_json(self.evidence / Path(self.rows[0]["transcript"]).with_suffix(".json"), self.rows[0])
        self.save_summary()
        self.rejected("outcome differs from native transcript")

    def test_rejects_partial_or_duplicated_native_report(self):
        path = self.evidence / self.rows[0]["transcript"]
        path.write_text(path.read_text() + "total: 1\n")
        self.rejected("native runner did not report exactly one execution")

    def test_rejects_missing_transcript(self):
        (self.evidence / self.rows[0]["transcript"]).unlink()
        self.rejected("native transcripts: missing")

    def test_rejects_extra_result_and_orphan_mode(self):
        key = hashlib.sha256(b"module:unexpected.js").hexdigest()
        write_json(self.evidence / f"{key}.json", self.rows[0])
        self.rejected("saved results: missing [], extra")

    def test_rejects_extra_native_transcript(self):
        (self.evidence / ("f" * 64 + ".log")).write_text("orphan")
        self.rejected("native transcripts: missing [], extra")

    def test_rejects_unrecognized_result_filename(self):
        write_json(self.evidence / "unidentified.json", self.rows[0])
        self.rejected("saved results: missing [], extra")

    def test_rejects_origin_source_mismatch(self):
        origin = self.root / "origin.json"
        write_json(origin, {"test262_tree_pin": self.pin, "total": 1, "outcomes": {"Bug": 1},
                            "rows": [{"execution_id": self.identities[0], "outcome": "Bug",
                                      "timeout": False,
                                      "source_sha256": "0" * 64}]})
        result = self.audit("--origin", str(origin))
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("origin source SHA-256 mismatch", result.stderr)
        self.assertFalse(self.output.exists())

    def test_rejects_empty_origin_report(self):
        origin = self.root / "origin.json"
        write_json(origin, {})
        result = self.audit("--origin", str(origin))
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("origin report: expected a row array", result.stderr)
        self.assertFalse(self.output.exists())

    def test_rejects_output_that_would_replace_retained_evidence(self):
        original = (self.evidence / "summary.json").read_bytes()
        self.output = self.evidence / "summary.json"
        result = self.audit()
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("audit output must be outside", result.stderr)
        self.assertEqual(self.output.read_bytes(), original)

    def test_rejects_missing_snapshot(self):
        self.snapshot().unlink()
        self.rejected("expected one snapshot, found 0")

    def test_rejects_duplicate_snapshot(self):
        path = self.snapshot()
        shutil.copyfile(path, path.with_name(path.stem + "-duplicate.json"))
        self.rejected("expected one snapshot, found 2")

    def test_rejects_extra_snapshot(self):
        shutil.copyfile(self.snapshot(), self.evidence / "snapshots" / "extra.json")
        self.rejected("snapshot inventory: missing [], extra")

    def test_rejects_snapshot_manifest_filename_mismatch(self):
        path = self.snapshot()
        snapshot = json.loads(path.read_text())
        snapshot["manifest_hash"] += 1
        write_json(path, snapshot)
        self.rejected("manifest filename mismatch")

    def test_rejects_snapshot_execution_mode_mismatch(self):
        path = self.snapshot()
        snapshot = json.loads(path.read_text())
        snapshot["completed_test_ids"] = [self.identities[1]]
        write_json(path, snapshot)
        self.rejected("completed execution identity mismatch")

    def test_rejects_snapshot_outcome_counts_mismatch(self):
        path = self.snapshot()
        snapshot = json.loads(path.read_text())
        snapshot["counts_per_outcome"]["Bug"] = 1
        write_json(path, snapshot)
        self.rejected("native outcome counts mismatch")

    def test_rejects_non_integer_native_counts(self):
        path = self.snapshot()
        snapshot = json.loads(path.read_text())
        snapshot["counts_per_outcome"]["Success"] = True
        write_json(path, snapshot)
        self.rejected("native outcome counts mismatch")

    def test_rejects_snapshot_failure_identity_mismatch(self):
        path = self.snapshot(4)
        snapshot = json.loads(path.read_text())
        snapshot["failures"][0]["test_id"] = self.identities[0]
        write_json(path, snapshot)
        self.rejected("failure identity or outcome mismatch")

    def test_rejects_timeout_on_success(self):
        path = self.snapshot()
        snapshot = json.loads(path.read_text())
        snapshot["timeout_test_ids"] = [self.identities[0]]
        write_json(path, snapshot)
        self.rejected("timeout identity or outcome mismatch")

    def test_rejects_snapshot_pin_from_another_content_tree(self):
        path = self.snapshot()
        snapshot = json.loads(path.read_text())
        snapshot["pinned_revisions"]["test262"] = self.git("rev-parse", "HEAD^{tree}")
        write_json(path, snapshot)
        self.rejected("Test262 pin mismatch")

    def test_rejects_changed_suite_source(self):
        (self.suite / "test" / "case.js").write_text("changed source")
        self.rejected("suite contents are dirty")

    def test_rejects_duplicate_json_fields(self):
        self.compiler.write_text('{"binary_sha256":"first","binary_sha256":"second"}')
        self.rejected("duplicate JSON field")

    def test_rejects_summary_outcome_totals_mismatch(self):
        self.summary["outcomes"]["Success"] += 1
        self.save_summary()
        self.rejected("summary outcome counts mismatch")


if __name__ == "__main__":
    unittest.main()
