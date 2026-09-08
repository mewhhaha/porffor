#!/usr/bin/env python3
"""Publication control-flow regressions; these are NOT ECMAScript/Test262 results."""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(os.environ.get("PUBLICATION_SOURCE_ROOT", Path(__file__).resolve().parents[2]))
SPEC = importlib.util.spec_from_file_location("publication_session", ROOT / "scripts/publication-session.py")
assert SPEC is not None and SPEC.loader is not None
session = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = session
SPEC.loader.exec_module(session)


class ProgressParsingTests(unittest.TestCase):
    def test_valid_observation_with_unrelated_status_lines(self):
        value = session.parse_matrix_progress(
            "backend: wasm-aot\nmatrix_nodes_total: 9\nmatrix_nodes_completed: 3\n"
        )
        self.assertEqual(value, session.MatrixProgress(3, 9))

    def test_zero_completed_and_largest_supported_total(self):
        self.assertEqual(session.parse_matrix_progress(
            "matrix_nodes_completed: 0\nmatrix_nodes_total: 999999999999999999"
        ).completed, 0)

    def test_rejects_missing_and_duplicate_fields(self):
        for value in ("", "matrix_nodes_completed: 0", "matrix_nodes_total: 1",
                      "matrix_nodes_completed: 0\nmatrix_nodes_total: 1\nmatrix_nodes_total: 1",
                      "matrix_nodes_completed: 0\nmatrix_nodes_total: 1\nmatrix_nodes_completed: 0"):
            with self.subTest(value=value), self.assertRaises(session.ProvenanceError):
                session.parse_matrix_progress(value)

    def test_rejects_noncanonical_decimal_spellings(self):
        for value in ("-1", "+1", "01", "1.0", "1e2", " 1", "1 ", "١", "1: 2",
                      "1000000000000000000", ""):
            with self.subTest(value=value), self.assertRaises(session.ProvenanceError):
                session.parse_matrix_progress(f"matrix_nodes_completed: {value}\nmatrix_nodes_total: 9")

    def test_constructor_rejects_impossible_and_mistyped_states(self):
        for completed, total in ((True, 2), (1, True), (1.0, 2), ("1", 2), (0, 0),
                                 (-1, 1), (2, 1), (0, 10**18), (10**18, 10**18)):
            with self.subTest(completed=completed, total=total), self.assertRaises(session.ProvenanceError):
                session.MatrixProgress(completed, total)

    def test_json_progress_requires_exact_fields(self):
        for value in ([], 1, "0:1", {}, {"completed": 0},
                      {"completed": 0, "total": 1, "extra": 0}):
            with self.subTest(value=value), self.assertRaises(session.ProvenanceError):
                session.MatrixProgress.from_dict(value)


class ManifestProgressTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.snapshots = Path(self.temporary.name)
        self.identity = dict.fromkeys(session.IDENTITY_KEYS, "fixture")
        self.identity.update(snapshot_directory=str(self.snapshots), snapshot_name="run α with spaces")
        self.manifest, _ = session.manifest_paths(self.identity)
        self.manifest.parent.mkdir()
        session.claim_manifest(self.manifest, self.identity)

    def record(self, completed, total=3, after_report=False):
        session.record_matrix_progress(self.manifest, session.MatrixProgress(completed, total),
                                       after_report=after_report)

    def document(self):
        return json.loads(self.manifest.read_text())

    def test_new_manifest_has_no_observation(self):
        self.assertIsNone(self.document()["progress"])
        session.require_fresh_matrix(self.manifest)

    def test_records_progress_without_changing_identity(self):
        self.record(1)
        self.assertEqual(self.document()["progress"], {"completed": 1, "total": 3})
        self.assertEqual(session.read_manifest(self.manifest), self.identity)
        session.require_identity(self.manifest, self.identity)

    def test_equal_observation_is_idempotent_before_report(self):
        self.record(1)
        with mock.patch.object(session.os, "replace", side_effect=AssertionError("unnecessary rewrite")):
            self.record(1)

    def test_advancing_after_report_is_accepted(self):
        self.record(1)
        self.record(2, after_report=True)
        self.assertEqual(self.document()["progress"]["completed"], 2)

    def test_regression_preserves_last_good_manifest(self):
        self.record(2)
        before = self.manifest.read_bytes()
        with self.assertRaisesRegex(session.ProvenanceError, "regressed"):
            self.record(1)
        self.assertEqual(self.manifest.read_bytes(), before)

    def test_changed_total_preserves_last_good_manifest(self):
        self.record(1)
        before = self.manifest.read_bytes()
        with self.assertRaisesRegex(session.ProvenanceError, "total changed"):
            self.record(2, total=4)
        self.assertEqual(self.manifest.read_bytes(), before)

    def test_successful_report_must_advance(self):
        self.record(1)
        with self.assertRaisesRegex(session.ProvenanceError, "did not advance"):
            self.record(1, after_report=True)

    def test_first_successful_report_must_complete_a_node(self):
        with self.assertRaisesRegex(session.ProvenanceError, "did not advance"):
            self.record(0, after_report=True)
        self.assertIsNone(self.document()["progress"])

    def test_zero_observation_still_prevents_bootstrapping(self):
        self.record(0)
        with self.assertRaisesRegex(session.ProvenanceError, "previously recorded"):
            session.require_fresh_matrix(self.manifest)

    def test_results_without_observation_prevent_bootstrapping(self):
        for suffix in (".json", "-node.json", "-matrix.txt", "-executions.jsonl"):
            result = self.snapshots / (self.identity["snapshot_name"] + suffix)
            result.write_text("fixture")
            with self.subTest(suffix=suffix), self.assertRaisesRegex(session.ProvenanceError, "existing results"):
                session.require_fresh_matrix(self.manifest)
            result.unlink()

    def test_unrelated_family_does_not_prevent_bootstrapping(self):
        (self.snapshots / "unrelated-matrix.json").write_text("fixture")
        session.require_fresh_matrix(self.manifest)

    def test_exact_snapshot_name_cannot_be_adopted_without_manifest(self):
        self.manifest.unlink()
        (self.snapshots / (self.identity["snapshot_name"] + ".json")).write_text("fixture")
        with self.assertRaisesRegex(session.ProvenanceError, "no publication provenance"):
            session.claim_manifest(self.manifest, self.identity)

    def test_duplicate_json_fields_are_rejected(self):
        self.manifest.write_text(self.manifest.read_text().replace(
            '"progress": null', '"progress": null, "progress": null'))
        with self.assertRaisesRegex(session.ProvenanceError, "duplicate"):
            session.read_manifest(self.manifest)

    def test_schema_one_is_not_silently_upgraded(self):
        document = self.document()
        document["schema_version"] = 1
        document.pop("progress")
        self.manifest.write_text(json.dumps(document))
        with self.assertRaisesRegex(session.ProvenanceError, "unsupported"):
            session.read_manifest(self.manifest)

    def test_missing_progress_field_is_not_treated_as_fresh(self):
        document = self.document()
        del document["progress"]
        self.manifest.write_text(json.dumps(document))
        with self.assertRaises(session.ProvenanceError):
            session.require_fresh_matrix(self.manifest)

    def test_boolean_checkpoint_count_is_rejected(self):
        document = self.document()
        document["progress"] = {"completed": True, "total": 3}
        self.manifest.write_text(json.dumps(document))
        with self.assertRaises(session.ProvenanceError):
            session.read_manifest(self.manifest)

    def test_failed_atomic_replace_keeps_previous_observation(self):
        self.record(1)
        before = self.manifest.read_bytes()
        with mock.patch.object(session.os, "replace", side_effect=OSError("disk fault")):
            with self.assertRaisesRegex(OSError, "disk fault"):
                self.record(2)
        self.assertEqual(self.manifest.read_bytes(), before)
        self.assertEqual(list(self.manifest.parent.glob(".manifest-*")), [])

    def test_successful_transition_syncs_file_and_parent_directory(self):
        with mock.patch.object(session.os, "fsync", wraps=session.os.fsync) as sync:
            self.record(1)
        self.assertEqual(sync.call_count, 2)

    def test_manifest_symlink_is_not_followed(self):
        target = self.manifest.with_name("target.json")
        self.manifest.rename(target)
        self.manifest.symlink_to(target)
        before = target.read_bytes()
        with self.assertRaisesRegex(session.ProvenanceError, "symlink"):
            self.record(1)
        self.assertEqual(target.read_bytes(), before)

    def test_observation_lock_serializes_updates(self):
        with session.family_lock(self.manifest.with_suffix(".progress.lock")):
            with self.assertRaisesRegex(session.ProvenanceError, "locked"):
                self.record(1)
        self.record(1)


FAKE_CLI = r'''#!/usr/bin/env python3
import json
import os
from pathlib import Path
import sys

root = Path(__file__).resolve().parent
config = json.loads((root / "scenario.json").read_text())
command = sys.argv[2]
args = sys.argv[3:]
def option(name):
    return args[args.index(name) + 1]
snapshots = Path(option("--snapshot-dir"))
matrix = snapshots / (option("--snapshot-name") + "-matrix.json")
with (root / "commands.jsonl").open("a") as log:
    log.write(json.dumps({"command": command, "args": args}) + "\n")
if command == "progress-status":
    if config.get("progress_error"):
        sys.exit(17)
    if "progress_text" in config:
        print(config["progress_text"])
        sys.exit(0)
    if not matrix.exists():
        sys.exit(4)
    value = json.loads(matrix.read_text())
    print("matrix_nodes_completed:", value["completed"])
    print("matrix_nodes_total:", value["total"])
    if config.get("mutate_source_on_progress"):
        (root / "crates/fixture.rs").write_text("changed source")
elif command == "report-all":
    counter = root / "report-counter"
    count = int(counter.read_text()) + 1 if counter.exists() else 1
    counter.write_text(str(count))
    fail = config.get("fail_report") == count
    if fail and not config.get("fail_after_write"):
        sys.exit(23)
    value = json.loads(matrix.read_text()) if matrix.exists() else {"completed": 0, "total": config.get("total", 3)}
    if config.get("change_total_at") == count:
        value["total"] += 1
    value["completed"] = min(value["total"], value["completed"] + config.get("step", 1))
    snapshots.mkdir(parents=True, exist_ok=True)
    matrix.write_text(json.dumps(value))
    if fail:
        sys.exit(23)
elif command == "publish-status":
    if "--readme-path" in args:
        Path(option("--readme-path")).write_text("publication test double; not conformance evidence\n")
    sys.exit(config.get("publish_exit", 0))
else:
    sys.exit(99)
'''


class DriverTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="publication driver ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        for relative in ("scripts/publication-session.py", "scripts/publish-real-status-low-ram.sh",
                         "scripts/lib/publish-real-status-driver.sh"):
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, destination)
        (self.root / "crates").mkdir()
        (self.root / "crates/fixture.rs").write_text("// source identity fixture\n")
        self.suite = self.root / "test262/vendor/test262"
        self.suite.mkdir(parents=True)
        (self.suite / "case.js").write_text("// suite identity fixture; not a conformance run\n")
        self.snapshots = self.root / "test262/snapshots"
        self.binary = self.root / "fake-lila"
        self.binary.write_text(FAKE_CLI)
        self.binary.chmod(0o755)
        self.configure()
        self.git("init", "-q")
        self.git("add", "scripts", "crates")
        self.git("-c", "user.name=Publication Test", "-c", "user.email=test@example.invalid",
                 "commit", "-qm", "fixture")
        self.name = "resume α with spaces"
        self.environment = dict(os.environ, REPO_ROOT=str(self.root), LILA_BIN=str(self.binary),
                                SUITE_ROOT=str(self.suite), SNAPSHOT_DIR=str(self.snapshots),
                                SNAPSHOT_NAME=self.name, THREADS="1", JOBS="1", ISOLATE_CASES="1",
                                MAX_MATRIX_NODES="1", README_PATH=str(self.root / "README fixture.md"),
                                LILA_TEST262_FORCE_CASE_RUNNER="1")
        self.environment.pop("LILA_TEST262_DISABLE_CASE_RUNNER", None)
        self.matrix = self.snapshots / (self.name + "-matrix.json")

    def git(self, *args):
        subprocess.run(["git", "-C", str(self.root), *args], check=True,
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5)

    def configure(self, **options):
        (self.root / "scenario.json").write_text(json.dumps(options))

    def run_driver(self, backend="wasm-aot"):
        return subprocess.run(["bash", str(self.root / "scripts/publish-real-status-low-ram.sh"),
                               backend, self.name], cwd=self.root, env=self.environment,
                              capture_output=True, text=True, timeout=15)

    def commands(self):
        log = self.root / "commands.jsonl"
        return [json.loads(line)["command"] for line in log.read_text().splitlines()] if log.exists() else []

    def manifest(self):
        return next((self.snapshots / ".publication-provenance").glob("*.json"))

    def pause_after_one_observation(self):
        self.configure(fail_report=2)
        result = self.run_driver()
        self.assertEqual(result.returncode, 23, result.stdout + result.stderr)
        self.assertNotIn("publish-status", self.commands())
        self.configure()

    def assert_stops_before_report_or_publish(self, result, since, message):
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(message, result.stderr)
        self.assertNotIn("report-all", self.commands()[since:])
        self.assertNotIn("publish-status", self.commands()[since:])

    def test_fresh_run_reaches_publication_only_after_complete_matrix(self):
        result = self.run_driver()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.commands().count("report-all"), 3)
        self.assertEqual(self.commands()[-1], "publish-status")
        self.assertEqual(json.loads(self.manifest().read_text())["progress"], {"completed": 3, "total": 3})
        self.assertIn("test double", (self.root / "README fixture.md").read_text())

    def test_interrupted_run_resumes_same_observation_then_advances(self):
        self.pause_after_one_observation()
        result = self.run_driver()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.commands().count("publish-status"), 1)

    def test_resume_rejects_rollback(self):
        self.pause_after_one_observation()
        self.matrix.write_text(json.dumps({"completed": 0, "total": 3}))
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "regressed")

    def test_resume_rejects_changed_total(self):
        self.pause_after_one_observation()
        self.matrix.write_text(json.dumps({"completed": 1, "total": 4}))
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "total changed")

    def test_resume_rejects_missing_checkpoint(self):
        self.pause_after_one_observation()
        self.matrix.unlink()
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "previously recorded progress")

    def test_unreadable_unobserved_results_are_not_a_fresh_family(self):
        self.configure(fail_report=1, fail_after_write=True)
        result = self.run_driver()
        self.assertEqual(result.returncode, 23, result.stderr)
        self.configure(progress_error=True)
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "existing results")

    def test_interrupt_before_any_results_can_still_bootstrap(self):
        self.configure(fail_report=1)
        self.assertEqual(self.run_driver().returncode, 23)
        self.configure()
        result = self.run_driver()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_partial_checkpoint_after_failed_report_can_resume(self):
        self.configure(fail_report=1, fail_after_write=True)
        self.assertEqual(self.run_driver().returncode, 23)
        self.configure()
        result = self.run_driver()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_stalled_successful_report_is_an_error(self):
        self.configure(step=0)
        result = self.run_driver()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("did not advance", result.stderr)
        self.assertEqual(self.commands().count("report-all"), 1)
        self.assertNotIn("publish-status", self.commands())

    def test_total_cannot_change_within_one_session(self):
        self.configure(change_total_at=2)
        result = self.run_driver()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("total changed", result.stderr)
        self.assertNotIn("publish-status", self.commands())

    def test_duplicate_status_fields_never_trigger_a_report(self):
        self.configure(progress_text="matrix_nodes_completed: 0\nmatrix_nodes_total: 3\nmatrix_nodes_total: 3")
        self.assert_stops_before_report_or_publish(self.run_driver(), 0, "duplicate")

    def test_corrupt_manifest_prevents_any_cli_call(self):
        self.pause_after_one_observation()
        self.manifest().write_text("not json")
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "cannot read")
        self.assertEqual(len(self.commands()), since)

    def test_lost_manifest_does_not_adopt_existing_results(self):
        self.pause_after_one_observation()
        self.manifest().unlink()
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "no publication provenance")

    def test_schema_one_family_requires_a_new_name(self):
        self.pause_after_one_observation()
        path = self.manifest()
        document = json.loads(path.read_text())
        document["schema_version"] = 1
        document.pop("progress")
        path.write_text(json.dumps(document))
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "unsupported")

    def test_source_change_on_resume_is_still_rejected(self):
        self.pause_after_one_observation()
        (self.root / "crates/fixture.rs").write_text("// changed\n")
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "source_inputs_sha256")

    def test_source_change_during_progress_query_is_rejected(self):
        self.pause_after_one_observation()
        self.configure(mutate_source_on_progress=True)
        since = len(self.commands())
        self.assert_stops_before_report_or_publish(self.run_driver(), since, "source_inputs_sha256")

    def test_oracle_backend_cannot_publish(self):
        result = self.run_driver("spec-exec")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(self.commands(), [])

    def test_publisher_failure_is_not_reported_as_success(self):
        self.configure(publish_exit=29)
        result = self.run_driver()
        self.assertEqual(result.returncode, 29, result.stdout + result.stderr)
        self.assertEqual(self.commands()[-1], "publish-status")

    def test_completed_family_can_be_republished_without_running_nodes(self):
        self.assertEqual(self.run_driver().returncode, 0)
        since = len(self.commands())
        result = self.run_driver()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.commands()[since:], ["progress-status", "publish-status"])


if __name__ == "__main__":
    unittest.main()
