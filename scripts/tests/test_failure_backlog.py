"""Accounting regressions: a task edit must not silently lose failure evidence."""

import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    "failure_backlog", Path(__file__).resolve().parents[1] / "check-failure-backlog.py")
BACKLOG = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BACKLOG)


class BacklogAccounting(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "tasks"
        (self.root / "evidence").mkdir(parents=True)
        outcomes = {"Success": 1, "Bug": 1, "NotImplemented": 1, "Crash": 0}
        failures = [
            {"test_id": "sloppy-script:example.js", "test_path": "example.js", "outcome": "Bug",
             "detail": "wrong result: \r\x00\t", "kind": "Runtime", "origin": "unknown"},
            {"test_id": "strict-script:example.js", "test_path": "example.js", "outcome": "NotImplemented",
             "detail": "missing lowering", "kind": "Unsupported", "origin": "unknown"},
        ]
        baseline = {"schema_version": 1, "refresh_date": "2026-09-28", "total": 3, "passed": 1,
                    "failed": 2, "execution_backend": "wasm-aot", "completed_nodes": 1, "total_nodes": 1,
                    "manifest_hash": 42, "pinned_revisions": {"test262": "fixture-only"},
                    "counts_per_outcome": outcomes, "compiler_sha256": "compiler",
                    "source_inputs_sha256": "source", "suite_sha256": "suite"}
        aggregate = {**baseline, "producer": "lila", "run_kind": "aggregate-matrix",
                     "completed_nodes": ["example"],
                     "aggregate_entries": [{"node_id": "example", "total": 3, "passed": 1,
                                            "failed": 2, "counts_per_outcome": outcomes}]}
        self.write_json("evidence/aggregate.json", aggregate)
        self.write_json("evidence/failures.json", failures)
        self.write_json("evidence/published-status.json", {"real_suite": baseline})
        self.write_json("evidence/publication-session.json", {
            "identity": {"executable_sha256": "compiler", "source_inputs_sha256": "source", "suite_sha256": "suite"},
            "progress": {"completed": 1, "total": 1}})
        self.write_json("evidence/provenance.json", {
            "aggregate": {"sha256": BACKLOG.digest(self.root / "evidence/aggregate.json")},
            "outputs": {"failures.json": {"sha256": BACKLOG.digest(self.root / "evidence/failures.json")}},
            "exclusion": {"execution_count": 0}, "collected": {"failed": 2},
            "observed": {"failed": 2, "total": 3, "passed": 1},
            "completed_leaves": [{"node_id": "example", "total": 3, "passed": 1,
                                  "failed": 2, "outcomes": outcomes}]})
        (self.root / "evidence/failures.executions").write_text(
            "sloppy-script:example.js\nstrict-script:example.js\n")
        baseline["artifacts"] = {path.relative_to(self.root).as_posix(): BACKLOG.digest(path)
                                 for path in (self.root / "evidence").iterdir()}
        self.write_json("evidence/baseline.json", baseline)
        self.registry = {"schema_version": 1, "baseline": "evidence/baseline.json", "tasks": []}
        for number, failure in enumerate(failures, 1):
            identity = f"F{number:03d}"
            self.registry["tasks"].append({
                "id": identity, "key": f"example-{number}", "title": f"Example {number}",
                "owner": "lila-aot-wasm", "status": "open", "cause_status": "suspected",
                "disposition": "required", "root_cause": "Investigate operation result propagation.",
                "evidence": [{"path": "crates/example.rs", "line": 1, "note": "Investigation entry point."}],
                "fix": "Confirm and repair operation.", "validation": "Run adjacent operations.",
                "test_ids": [failure["test_id"]], "counts": BACKLOG.counts_for([failure]),
                "document": f"{identity}-example-{number}.md", "cases": f"cases/{identity}.executions"})
        self.save_registry()

    def write_json(self, name, value):
        (self.root / name).write_text(json.dumps(value, indent=2) + "\n")

    def save_registry(self):
        self.write_json("registry.json", self.registry)

    def test_roundtrip_preserves_frozen_evidence(self):
        before = {path.name: path.read_bytes() for path in (self.root / "evidence").iterdir()}
        self.assertEqual(BACKLOG.check(self.root, write=True), (2, 2))
        self.assertEqual(BACKLOG.check(self.root), (2, 2))
        self.assertEqual(before, {path.name: path.read_bytes() for path in (self.root / "evidence").iterdir()})
        self.assertIn(r"\r\u0000\t", (self.root / "F001-example-1.md").read_text())

    def test_deleting_task_cannot_drop_a_failure(self):
        self.registry["tasks"].pop()
        self.save_registry()
        with self.assertRaisesRegex(ValueError, "unassigned failures"):
            BACKLOG.check(self.root, write=True)

    def test_two_tasks_cannot_claim_same_execution(self):
        duplicate = copy.deepcopy(self.registry["tasks"][0])
        duplicate.update(id="F003", key="duplicate", document="F003-duplicate.md", cases="cases/F003.executions")
        self.registry["tasks"].append(duplicate)
        self.save_registry()
        with self.assertRaisesRegex(ValueError, "assigned to multiple tasks"):
            BACKLOG.validate_data(self.root)

    def test_invented_execution_cannot_replace_a_failure(self):
        self.registry["tasks"][0]["test_ids"] = ["sloppy-script:passing.js"]
        self.save_registry()
        with self.assertRaisesRegex(ValueError, "non-baseline execution"):
            BACKLOG.validate_data(self.root)

    def test_raw_diagnostic_changes_invalidate_frozen_evidence(self):
        failures = BACKLOG.read_json(self.root / "evidence/failures.json")
        failures[0]["detail"] = "success"
        self.write_json("evidence/failures.json", failures)
        with self.assertRaisesRegex(ValueError, "baseline evidence changed"):
            BACKLOG.validate_data(self.root)

    def test_wrong_task_outcome_counts_fail(self):
        self.registry["tasks"][0]["counts"]["Bug"] = 0
        self.save_registry()
        with self.assertRaisesRegex(ValueError, "counts differ from evidence"):
            BACKLOG.validate_data(self.root)

    def test_fixed_task_needs_evidence_and_keeps_membership(self):
        task = self.registry["tasks"][0]
        task["status"] = "fixed"
        self.save_registry()
        with self.assertRaisesRegex(ValueError, "fixed task needs replay evidence"):
            BACKLOG.validate_data(self.root)
        task["resolution"] = "Revision example; native replay and adjacent-family evidence recorded."
        self.save_registry()
        self.assertEqual(BACKLOG.check(self.root, write=True), (2, 2))

    def test_changed_mode_in_replay_list_is_rejected(self):
        BACKLOG.check(self.root, write=True)
        (self.root / "cases/F001.executions").write_text("strict-script:example.js\n")
        with self.assertRaisesRegex(ValueError, "generated backlog file drifted"):
            BACKLOG.check(self.root)

    def test_unknown_old_plan_is_rejected(self):
        (self.root / "old-plan.md").write_text("# Stale plan\n")
        with self.assertRaisesRegex(ValueError, "unregistered task documents"):
            BACKLOG.check(self.root, write=True)

    def test_duplicate_json_keys_are_rejected(self):
        (self.root / "registry.json").write_text('{"schema_version": 1, "schema_version": 2}')
        with self.assertRaisesRegex(ValueError, "duplicate JSON key"):
            BACKLOG.validate_data(self.root)

    def change_initial_provenance(self, edit):
        provenance = BACKLOG.read_json(self.root / "evidence/provenance.json")
        edit(provenance)
        self.write_json("evidence/provenance.json", provenance)
        baseline = BACKLOG.read_json(self.root / "evidence/baseline.json")
        baseline["artifacts"]["evidence/provenance.json"] = BACKLOG.digest(self.root / "evidence/provenance.json")
        self.write_json("evidence/baseline.json", baseline)

    def test_extraction_cannot_substitute_a_node_with_same_count(self):
        self.change_initial_provenance(lambda value: value["completed_leaves"][0].update(node_id="unrelated"))
        with self.assertRaisesRegex(ValueError, "duplicate or missing matrix nodes"):
            BACKLOG.validate_data(self.root)

    def test_extraction_cannot_miscount_one_leaf(self):
        self.change_initial_provenance(lambda value: value["completed_leaves"][0].update(failed=1))
        with self.assertRaisesRegex(ValueError, "extraction counts disagree"):
            BACKLOG.validate_data(self.root)


if __name__ == "__main__":
    unittest.main()
