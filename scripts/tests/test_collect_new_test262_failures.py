import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


COLLECTOR = Path(__file__).resolve().parents[1] / "collect-new-test262-failures.py"


class CollectFailuresTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        outcomes = {"Success": 1, "NotImplemented": 1, "Bug": 1, "Crash": 0}
        self.aggregate = {
            "run_kind": "aggregate-matrix",
            "producer": "lila",
            "execution_backend": "wasm-aot",
            "pinned_revisions": {"test262": "pinned-revision"},
            "completed_nodes": ["first"],
            "total": 3,
            "passed": 1,
            "counts_per_outcome": outcomes.copy(),
            "aggregate_entries": [
                {"node_id": "first", "manifest_hash": "first-hash", "failed": 2, "total": 3,
                 "passed": 1, "counts_per_outcome": outcomes.copy()},
                {"node_id": "unfinished", "manifest_hash": "absent", "failed": 1, "total": 1},
            ],
        }
        self.checkpoint = {
            "producer": "lila",
            "execution_backend": "wasm-aot",
            "manifest_hash": "first-hash",
            "run_kind": "matrix-filter-leaf",
            "pinned_revisions": self.aggregate["pinned_revisions"].copy(),
            "total": 3,
            "passed": 1,
            "counts_per_outcome": outcomes.copy(),
            "completed_test_ids": ["strict-script:first.js", "sloppy-script:first.js", "sloppy-script:second.js"],
            "failures": [
                {"test_id": "strict-script:first.js", "outcome": "Bug"},
                {"test_id": "sloppy-script:first.js", "outcome": "NotImplemented"},
            ],
        }
        (self.root / "excluded").write_text("# Prior cohort\nstrict-script:first.js\n")

    def collect(self, root=None):
        root = root or self.root
        (root / "aggregate.json").write_text(json.dumps(self.aggregate))
        (root / "snapshot-first-hash.json").write_text(json.dumps(self.checkpoint))
        return subprocess.run(
            [sys.executable, str(COLLECTOR), str(root / "aggregate.json"),
             "--exclude", str(root / "excluded"),
             "--output-dir", str(root / "collected")],
            text=True, capture_output=True, check=False,
        )

    def assert_rejected(self, reason):
        result = self.collect()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(reason, result.stderr)
        self.assertFalse((self.root / "collected").exists())

    def test_collects_only_new_executions_from_completed_nodes(self):
        result = self.collect()
        self.assertEqual(result.returncode, 0, result.stderr)
        output = self.root / "collected"
        self.assertEqual((output / "failures.executions").read_text(), "sloppy-script:first.js\n")
        self.assertEqual(json.loads((output / "failures.json").read_text()),
                         [self.checkpoint["failures"][1]])

    def test_rejects_a_checkpoint_from_another_suite_pin(self):
        self.checkpoint["pinned_revisions"]["test262"] = "another-revision"
        self.assert_rejected("pin mismatch")

    def test_rejects_missing_failure_evidence(self):
        self.checkpoint["failures"].pop()
        self.assert_rejected("failure count mismatch")

    def test_rejects_duplicate_execution_evidence(self):
        self.checkpoint["failures"][0] = self.checkpoint["failures"][1].copy()
        self.assert_rejected("duplicate failed execution")

    def test_rejects_duplicates_even_when_every_duplicate_is_excluded(self):
        self.checkpoint["failures"][1] = self.checkpoint["failures"][0].copy()
        self.assert_rejected("duplicate failed execution")

    def test_rejects_a_missing_completed_node(self):
        self.aggregate["completed_nodes"].append("missing")
        self.assert_rejected("completed nodes lack aggregate entries")

    def test_rejects_duplicate_completed_node_entries(self):
        self.aggregate["aggregate_entries"].append(self.aggregate["aggregate_entries"][0].copy())
        self.assert_rejected("duplicate completed entry")

    def test_rejects_duplicate_completed_nodes(self):
        self.aggregate["completed_nodes"].append("first")
        self.assert_rejected("duplicate completed node")

    def test_rejects_missing_passed_execution_identities(self):
        self.checkpoint["completed_test_ids"].pop()
        self.assert_rejected("completed execution inventory mismatch")

    def test_rejects_duplicate_passed_identities_across_leaves(self):
        outcomes = {"Success": 1, "NotImplemented": 0, "Bug": 0, "Crash": 0}
        self.aggregate["completed_nodes"].append("second")
        self.aggregate["aggregate_entries"].append({"node_id": "second", "manifest_hash": "second-hash",
            "total": 1, "passed": 1, "failed": 0, "counts_per_outcome": outcomes})
        second = {**self.checkpoint, "manifest_hash": "second-hash", "total": 1, "passed": 1,
                  "counts_per_outcome": outcomes, "completed_test_ids": ["sloppy-script:second.js"], "failures": []}
        (self.root / "snapshot-second-hash.json").write_text(json.dumps(second))
        self.assert_rejected("duplicate completed execution across leaves")

    def test_rejects_failure_outside_the_completed_inventory(self):
        self.checkpoint["failures"][0]["test_id"] = "strict-script:absent.js"
        self.assert_rejected("failure absent from completed execution inventory")

    def test_rejects_a_success_classified_as_a_failure(self):
        self.checkpoint["failures"][0]["outcome"] = "Success"
        self.assert_rejected("invalid failure outcome")

    def test_rejects_incorrect_leaf_outcomes(self):
        self.checkpoint["counts_per_outcome"]["Bug"] = 0
        self.assert_rejected("outcome count mismatch")

    def test_rejects_incorrect_leaf_pass_count(self):
        self.checkpoint["passed"] = 0
        self.assert_rejected("checkpoint total or passed count mismatch")

    def test_rejects_incorrect_aggregate_outcomes(self):
        self.aggregate["counts_per_outcome"]["Bug"] = 0
        self.assert_rejected("aggregate outcome or passed counts differ")

    def test_rejects_unreconciled_completed_totals(self):
        self.aggregate["total"] = 4
        self.assert_rejected("completed checkpoint totals differ")

    def test_rejects_ambiguous_checkpoint_files(self):
        (self.root / "another-first-hash.json").write_text(json.dumps(self.checkpoint))
        self.assert_rejected("expected one checkpoint")

    def test_provenance_reconciles_inputs_exclusions_and_outputs(self):
        result = self.collect()
        self.assertEqual(result.returncode, 0, result.stderr)
        output = self.root / "collected"
        provenance = json.loads((output / "provenance.json").read_text())
        for record in [provenance["aggregate"], provenance["exclusion"], *provenance["completed_leaves"]]:
            path = self.root / record["name"]
            self.assertEqual(record["sha256"], hashlib.sha256(path.read_bytes()).hexdigest())
        self.assertEqual(provenance["observed"], {"completed_nodes": 1, "total": 3, "passed": 1,
            "failed": 2, "outcomes": self.aggregate["counts_per_outcome"]})
        self.assertEqual(provenance["exclusion"]["execution_count"], 1)
        self.assertEqual(provenance["exclusion"]["observed_execution_count"], 1)
        self.assertEqual(provenance["exclusion"]["excluded_failure_count"], 1)
        self.assertEqual(provenance["collected"], {"failed": 1, "outcomes": {
            "Success": 0, "NotImplemented": 1, "Bug": 0, "Crash": 0}})
        for name, record in provenance["outputs"].items():
            self.assertEqual(record["sha256"], hashlib.sha256((output / name).read_bytes()).hexdigest())

    def test_provenance_is_identical_after_input_directory_relocation(self):
        first = self.collect()
        self.assertEqual(first.returncode, 0, first.stderr)
        relocated = self.root / "relocated"
        relocated.mkdir()
        (relocated / "excluded").write_bytes((self.root / "excluded").read_bytes())
        second = self.collect(relocated)
        self.assertEqual(second.returncode, 0, second.stderr)
        expected = (self.root / "collected/provenance.json").read_bytes()
        self.assertEqual((relocated / "collected/provenance.json").read_bytes(), expected)
        self.assertNotIn(str(self.root).encode(), expected)

    def test_provenance_hashes_exact_exclusion_bytes(self):
        first = self.collect()
        self.assertEqual(first.returncode, 0, first.stderr)
        changed = self.root / "changed"
        changed.mkdir()
        (changed / "excluded").write_text((self.root / "excluded").read_text() + "# Same identities\n")
        second = self.collect(changed)
        self.assertEqual(second.returncode, 0, second.stderr)
        before = json.loads((self.root / "collected/provenance.json").read_text())
        after = json.loads((changed / "collected/provenance.json").read_text())
        self.assertNotEqual(before["exclusion"]["sha256"], after["exclusion"]["sha256"])
        self.assertEqual(before["collected"], after["collected"])
        self.assertEqual(before["outputs"], after["outputs"])


if __name__ == "__main__":
    unittest.main()
