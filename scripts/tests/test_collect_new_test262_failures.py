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
        self.aggregate = {
            "run_kind": "aggregate-matrix",
            "pinned_revisions": {"test262": "pinned-revision"},
            "completed_nodes": ["first"],
            "total": 3,
            "aggregate_entries": [
                {"node_id": "first", "manifest_hash": "first-hash", "failed": 2, "total": 3},
                {"node_id": "unfinished", "manifest_hash": "absent", "failed": 1, "total": 1},
            ],
        }
        self.checkpoint = {
            "pinned_revisions": self.aggregate["pinned_revisions"].copy(),
            "failures": [
                {"test_id": "strict-script:first.js", "classification": "Bug"},
                {"test_id": "sloppy-script:first.js", "classification": "NotImplemented"},
            ],
        }
        (self.root / "excluded").write_text("# Prior cohort\nstrict-script:first.js\n")

    def collect(self):
        (self.root / "aggregate.json").write_text(json.dumps(self.aggregate))
        (self.root / "snapshot-first-hash.json").write_text(json.dumps(self.checkpoint))
        return subprocess.run(
            [sys.executable, str(COLLECTOR), str(self.root / "aggregate.json"),
             "--exclude", str(self.root / "excluded"),
             "--output-dir", str(self.root / "collected")],
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

    def test_rejects_unreconciled_completed_totals(self):
        self.aggregate["total"] = 4
        self.assert_rejected("completed checkpoint totals differ")

    def test_rejects_ambiguous_checkpoint_files(self):
        (self.root / "another-first-hash.json").write_text(json.dumps(self.checkpoint))
        self.assert_rejected("expected one checkpoint")


if __name__ == "__main__":
    unittest.main()
