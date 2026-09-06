#!/usr/bin/env python3
"""Publication-driver contracts. The fake CLI is not product conformance evidence."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


WRAPPER = Path(os.environ.get("PUBLISH_WRAPPER", Path(__file__).with_name("publish-real-status-low-ram.sh")))
FAKE_CLI = r'''#!/usr/bin/env python3
import json
import os
from pathlib import Path
import subprocess
import sys

root = Path(os.environ["FAKE_ROOT"])
config = json.loads((root / "config.json").read_text())
state_path = root / "state.json"
state = json.loads(state_path.read_text()) if state_path.exists() else {}
command = sys.argv[2]
index = state.get(command, 0)
state[command] = index + 1
state_path.write_text(json.dumps(state))
with (root / "calls.jsonl").open("a") as log:
    log.write(json.dumps({"args": sys.argv[1:], "isolation": os.environ.get("LILA_TEST262_FORCE_CASE_RUNNER")}) + "\n")
if index > 3:
    print("fake CLI safety limit: wrapper kept retrying", file=sys.stderr)
    sys.exit(86)
mutation = config.get("mutation", {})
if mutation.get("command") == command:
    if mutation["kind"] == "binary":
        with Path(__file__).open("a") as binary:
            binary.write("\n# changed executable\n")
    elif mutation["kind"] == "permission":
        Path(__file__).chmod(0o644)
    elif mutation["kind"] == "source":
        subprocess.run(["git", "-C", str(root / "repo"), "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "--allow-empty", "-qm", "move source"], check=True)
if command == "progress-status":
    entries = config["progress"]
    entry = entries[min(index, len(entries) - 1)]
    if "stderr" in entry:
        print(entry["stderr"], file=sys.stderr)
    if "raw" in entry:
        print(entry["raw"])
    elif "completed" in entry:
        print("matrix_nodes_completed: " + str(entry["completed"]))
        print("matrix_nodes_total: " + str(entry["total"]))
    sys.exit(entry.get("exit", 0))
sys.exit(config.get(command + "_exit", 0))
'''


class PublicationDriverTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="lila publication ")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.script = self.repo / "scripts" / WRAPPER.name
        self.script.parent.mkdir(parents=True)
        shutil.copyfile(WRAPPER, self.script)
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        subprocess.run(["git", "-C", str(self.repo), "add", "."], check=True)
        subprocess.run(["git", "-C", str(self.repo), "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "fixture"], check=True)
        self.source = subprocess.check_output(["git", "-C", str(self.repo), "rev-parse", "HEAD"], text=True).strip()
        self.binary = self.root / "fake lila"
        self.binary.write_text(FAKE_CLI)
        self.binary.chmod(0o755)
        self.digest = hashlib.sha256(self.binary.read_bytes()).hexdigest()
        self.env = os.environ.copy()
        self.env.pop("LILA_TEST262_FORCE_CASE_RUNNER", None)
        self.env.update({
            "FAKE_ROOT": str(self.root),
            "LILA_BIN": str(self.binary),
            "SUITE_ROOT": str(self.root / "suite root"),
            "SNAPSHOT_DIR": str(self.root / "snapshots"),
            "THREADS": "1", "JOBS": "1", "MAX_MATRIX_NODES": "1",
            "ISOLATE_CASES": "1", "README_PATH": "",
        })

    def run_driver(self, progress, *, backend="wasm-aot", env=None, **config):
        (self.root / "config.json").write_text(json.dumps({"progress": progress, **config}))
        for name in ("state.json", "calls.jsonl"):
            (self.root / name).unlink(missing_ok=True)
        return subprocess.run(
            ["bash", str(self.script), backend, "baseline with spaces"],
            cwd=self.repo, env={**self.env, **(env or {})},
            text=True, capture_output=True, timeout=10,
        )

    def calls(self, command=None):
        path = self.root / "calls.jsonl"
        calls = [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []
        return [call for call in calls if command is None or call["args"][1] == command]

    def assert_failure(self, result, message):
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(message, result.stderr)
        self.assertEqual(self.calls("publish-status"), [])

    def test_fresh_run_bootstraps_once_and_publishes_after_completion(self):
        result = self.run_driver([{"exit": 1, "stderr": "no checkpoint"}, {"completed": 1, "total": 2}, {"completed": 2, "total": 2}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call["args"][1] for call in self.calls()], ["progress-status", "report-all", "progress-status", "report-all", "progress-status", "publish-status"])
        self.assertIn("no checkpoint", result.stderr)
        self.assertIn("matrix_progress: 0/unknown", result.stdout)

    def test_resume_uses_strictly_increasing_progress(self):
        result = self.run_driver([{"completed": n, "total": 3} for n in (1, 2, 3)])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.calls("report-all")), 2)
        self.assertEqual(len(self.calls("publish-status")), 1)

    def test_already_complete_calls_rust_publisher_without_reporting(self):
        result = self.run_driver([{"completed": 4, "total": 4}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls("report-all"), [])
        self.assertEqual(len(self.calls("publish-status")), 1)

    def test_known_empty_progress_can_advance(self):
        result = self.run_driver([{"completed": 0, "total": 1}, {"completed": 1, "total": 1}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.calls("report-all")), 1)

    def test_logs_checkout_and_exact_executable_identity(self):
        result = self.run_driver([{"completed": 1, "total": 1}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("source_commit: " + self.source, result.stdout)
        self.assertIn("compiler_sha256: " + self.digest, result.stdout)
        self.assertIn("execution_backend: wasm-aot", result.stdout)

    def test_forwards_quoted_paths_and_resource_limits(self):
        readme = str(self.root / "read me.md")
        result = self.run_driver([{"completed": 0, "total": 1}, {"completed": 1, "total": 1}], env={"JOBS": "2", "THREADS": "3", "MAX_MATRIX_NODES": "4", "README_PATH": readme})
        self.assertEqual(result.returncode, 0, result.stderr)
        for call in self.calls():
            args = call["args"]
            for flag, value in (("--suite-root", self.env["SUITE_ROOT"]), ("--snapshot-dir", self.env["SNAPSHOT_DIR"]), ("--snapshot-name", "baseline with spaces")):
                self.assertEqual(args[args.index(flag) + 1], value)
            self.assertEqual(call["isolation"], "1")
        args = self.calls("report-all")[0]["args"]
        for flag, value in (("--jobs", "2"), ("--threads", "3"), ("--max-matrix-nodes", "4")):
            self.assertEqual(args[args.index(flag) + 1], value)
        self.assertIn("--resume", args)
        self.assertEqual(self.calls("publish-status")[0]["args"][-2:], ["--readme-path", readme])
        self.assertNotIn("--readme-path", args)

    def test_wasm_alias_is_normalized(self):
        result = self.run_driver([{"completed": 1, "total": 1}], backend="wasm")
        self.assertEqual(result.returncode, 0, result.stderr)
        for call in self.calls():
            args = call["args"]
            self.assertEqual(args[args.index("--execution-backend") + 1], "wasm-aot")

    def test_oracle_backend_is_rejected_before_invocation(self):
        result = self.run_driver([], backend="spec-exec")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(self.calls(), [])

    def test_missing_executable_is_rejected(self):
        self.binary.unlink()
        result = self.run_driver([])
        self.assert_failure(result, "missing executable")
        self.assertEqual(self.calls(), [])

    def test_invalid_resource_limits_are_rejected_before_invocation(self):
        for setting in ("THREADS", "JOBS", "MAX_MATRIX_NODES"):
            for value in ("0", "-1", "1.5", "x", "01", "9" * 19):
                with self.subTest(setting=setting, value=value):
                    result = self.run_driver([], env={setting: value})
                    self.assert_failure(result, setting + " must be a positive decimal integer")
                    self.assertEqual(self.calls(), [])

    def test_invalid_isolation_is_rejected(self):
        result = self.run_driver([], env={"ISOLATE_CASES": "2"})
        self.assert_failure(result, "ISOLATE_CASES must be 0 or 1")
        self.assertEqual(self.calls(), [])

    def test_disabled_isolation_does_not_force_case_runner(self):
        result = self.run_driver([{"completed": 1, "total": 1}], env={"ISOLATE_CASES": "0"})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(all(call["isolation"] is None for call in self.calls()))

    def test_stalled_progress_stops_after_one_report(self):
        result = self.run_driver([{"completed": 0, "total": 2}])
        self.assert_failure(result, "did not advance")
        self.assertEqual(len(self.calls("report-all")), 1)

    def test_bootstrap_must_complete_at_least_one_node(self):
        result = self.run_driver([{"exit": 1}, {"completed": 0, "total": 2}])
        self.assert_failure(result, "did not advance")
        self.assertEqual(len(self.calls("report-all")), 1)

    def test_regressing_progress_is_rejected(self):
        result = self.run_driver([{"completed": 2, "total": 3}, {"completed": 1, "total": 3}])
        self.assert_failure(result, "did not advance")
        self.assertEqual(len(self.calls("report-all")), 1)

    def test_changing_total_is_rejected_even_when_new_total_is_complete(self):
        result = self.run_driver([{"completed": 1, "total": 2}, {"completed": 3, "total": 3}])
        self.assert_failure(result, "matrix total changed")
        self.assertEqual(len(self.calls("report-all")), 1)

    def test_completed_exceeding_total_never_publishes(self):
        result = self.run_driver([{"completed": 3, "total": 2}])
        self.assert_failure(result, "completed exceeds total")
        self.assertEqual(self.calls("report-all"), [])

    def test_zero_total_is_not_a_publishable_matrix(self):
        result = self.run_driver([{"completed": 0, "total": 0}])
        self.assert_failure(result, "total must be positive")
        self.assertEqual(self.calls("report-all"), [])

    def test_progress_requires_unique_well_formed_fields(self):
        valid = "matrix_nodes_completed: 1\nmatrix_nodes_total: 1"
        for raw in ("", "matrix_nodes_completed: 1", valid + "\nmatrix_nodes_total: 1", valid + "\nmatrix_nodes_completed: 1", "matrix_nodes_completed: 1: 2\nmatrix_nodes_total: 1"):
            with self.subTest(raw=raw):
                result = self.run_driver([{"raw": raw}])
                self.assert_failure(result, "invalid matrix progress")
                self.assertEqual(self.calls("report-all"), [])

    def test_progress_rejects_noncanonical_and_overflowing_counts(self):
        for key in ("completed", "total"):
            for value in ("-1", "1.0", "01", "x", " 1", "9" * 19):
                with self.subTest(key=key, value=value):
                    result = self.run_driver([{"completed": 1, "total": 1, key: value}])
                    self.assert_failure(result, "invalid matrix progress")
                    self.assertEqual(self.calls("report-all"), [])

    def test_other_status_fields_and_stderr_are_preserved(self):
        result = self.run_driver([{"raw": "producer: lila\nmatrix_nodes_completed: 1\nmatrix_nodes_total: 1\npassed: 0", "stderr": "retained diagnostic"}])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("retained diagnostic", result.stderr)
        self.assertEqual(len(self.calls("publish-status")), 1)

    def test_progress_failure_after_bootstrap_is_not_retried(self):
        result = self.run_driver([{"exit": 7, "stderr": "cannot read snapshot"}])
        self.assert_failure(result, "progress-status failed after report-all (exit 7)")
        self.assertIn("cannot read snapshot", result.stderr)
        self.assertEqual(len(self.calls("report-all")), 1)

    def test_progress_failure_after_known_progress_is_not_retried(self):
        result = self.run_driver([{"completed": 0, "total": 2}, {"exit": 8}])
        self.assert_failure(result, "progress-status failed after report-all (exit 8)")
        self.assertEqual(len(self.calls("report-all")), 1)

    def test_report_failure_exit_status_is_preserved(self):
        result = self.run_driver([{"completed": 0, "total": 2}], **{"report-all_exit": 17})
        self.assertEqual(result.returncode, 17, result.stderr)
        self.assertEqual(len(self.calls("progress-status")), 1)
        self.assertEqual(self.calls("publish-status")), [])

    def test_publisher_failure_exit_status_is_preserved(self):
        result = self.run_driver([{"completed": 1, "total": 1}], **{"publish-status_exit": 19})
        self.assertEqual(result.returncode, 19, result.stderr)
        self.assertEqual(len(self.calls("publish-status")), 1)

    def test_binary_change_after_report_prevents_further_cli_calls(self):
        result = self.run_driver([{"completed": 0, "total": 1}, {"completed": 1, "total": 1}], mutation={"command": "report-all", "kind": "binary"})
        self.assert_failure(result, "compiler changed during publication")
        self.assertEqual(len(self.calls("progress-status")), 1)

    def test_binary_change_during_final_progress_prevents_publication(self):
        result = self.run_driver([{"completed": 1, "total": 1}], mutation={"command": "progress-status", "kind": "binary"})
        self.assert_failure(result, "compiler changed during publication")

    def test_lost_executable_permission_is_rejected(self):
        result = self.run_driver([{"completed": 0, "total": 1}, {"completed": 1, "total": 1}], mutation={"command": "report-all", "kind": "permission"})
        self.assert_failure(result, "no longer executable")
        self.assertEqual(len(self.calls("progress-status")), 1)

    def test_source_commit_change_prevents_publication(self):
        result = self.run_driver([{"completed": 1, "total": 1}], mutation={"command": "progress-status", "kind": "source"})
        self.assert_failure(result, "source commit changed during publication")


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(PublicationDriverTests)
    expected = suite.countTestCases()
    if not expected:
        raise SystemExit("publication driver contract inventory is empty")
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    raise SystemExit(0 if result.wasSuccessful() and not result.skipped and result.testsRun == expected else 1)
