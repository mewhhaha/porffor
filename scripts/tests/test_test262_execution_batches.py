"""Native list evidence must retain every ID and remain red on damage/interruption."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import test262_execution_batches as batches


def snapshot(ids, outcomes=None):
    outcomes = outcomes or {execution: "Success" for execution in ids}
    return {
        "snapshot_version": 8, "run_kind": "full", "execution_backend": "wasm-aot",
        "compiler_identity": {"frozen": "compiler"},
        "pinned_revisions": {"ecma262": "0" * 40, "test262": "1" * 40},
        "total": len(ids), "passed": sum(value == "Success" for value in outcomes.values()),
        "completed_test_ids": ids, "timeout_test_ids": [],
        "counts_per_outcome": {kind: sum(value == kind for value in outcomes.values()) for kind in batches.OUTCOMES},
        "failures": [{"test_id": key, "outcome": value} for key, value in outcomes.items() if value != "Success"],
    }


class ExecutionBatchTests(unittest.TestCase):
    def test_native_nonpasses_and_modes_remain_exact(self):
        ids = ["sloppy-script:a.js", "strict-script:a.js", "raw-module:b.js"]
        outcomes = dict(zip(ids, ["Success", "Bug", "NotImplemented"]))
        value = snapshot(ids, outcomes)
        self.assertEqual(batches.snapshot_verdicts(value, ids, value["compiler_identity"], 1), outcomes)
        with self.assertRaises(ValueError):
            batches.snapshot_verdicts(value, ids, value["compiler_identity"], 0)

    def test_inventory_requires_full_native_ids_and_keeps_modes_distinct(self):
        value = "count: 2\nsloppy-script:a.js\nstrict-script:a.js\n"
        self.assertEqual(batches.native_inventory(value), {"sloppy-script:a.js", "strict-script:a.js"})
        for damaged in ["count: 2\nsloppy-script:a.js\n", "count: 2\nsloppy-script:a.js\nsloppy-script:a.js\n",
                        "count: 100\n... 50 more\n", "count: 1\nunknown:case.js\n"]:
            with self.subTest(damaged=damaged), self.assertRaises(ValueError):
                batches.native_inventory(damaged)

    def test_partial_duplicate_wrong_mode_producer_pin_counts_and_failure_evidence_reject(self):
        ids = ["sloppy-script:a.js", "strict-script:a.js"]
        original = snapshot(ids, {ids[0]: "Success", ids[1]: "Bug"})
        mutations = [
            lambda d: d.update(completed_test_ids=[ids[0]]),
            lambda d: d.update(completed_test_ids=[ids[0], ids[0]]),
            lambda d: d.update(completed_test_ids=[ids[0], "raw-script:a.js"]),
            lambda d: d.update(compiler_identity={"different": "image"}),
            lambda d: d.update(snapshot_version=7),
            lambda d: d.update(execution_backend="spec-exec"),
            lambda d: d.update(run_kind="shard-1/2"),
            lambda d: d["pinned_revisions"].update(test262="unbound"),
            lambda d: d["counts_per_outcome"].update(Success=2),
            lambda d: d["counts_per_outcome"].update(Bug=True),
            lambda d: d.update(total=True),
            lambda d: d.update(failures=[]),
            lambda d: d["failures"].append(copy.deepcopy(d["failures"][0])),
            lambda d: d["failures"][0].update(outcome="Skipped"),
            lambda d: d.update(timeout_test_ids=[ids[0]]),
        ]
        for mutate in mutations:
            value = copy.deepcopy(original)
            mutate(value)
            with self.subTest(value=value), self.assertRaises(ValueError):
                batches.snapshot_verdicts(value, ids, original["compiler_identity"], 1)

    def create(self, root, interrupted=False):
        (root / "suite/test").mkdir(parents=True)
        for path in ["a.js", "b.js"]:
            (root / "suite/test" / path).write_text("/*---\n---*/\n")
        executions = root / "executions"
        executions.write_text("sloppy-script:a.js\nstrict-script:b.js\nstrict-script:absent.js\n")
        binary = root / "compiler-source"
        binary.write_text(
            f"#!{sys.executable}\n"
            "import hashlib,json,sys\nfrom pathlib import Path\n"
            f"calls=Path({str(root / 'calls')!r})\n"
            "identity={'source_fingerprint_scheme':'lila-program-cache-compiler-v3','source_fingerprint':'2'*64,"
            "'source_revision':{'kind':'git-commit','commit':'3'*40},"
            "'executable_sha256':hashlib.sha256(Path(sys.argv[0]).read_bytes()).hexdigest()}\n"
            "if sys.argv[-1]=='compiler-identity':\n print(json.dumps(identity));sys.exit(0)\n"
            "subcommand=sys.argv[sys.argv.index('test262')+1]\n"
            "with calls.open('a') as f:f.write(subcommand+' '+str('--resume' in sys.argv)+'\\n')\n"
            "if subcommand=='list-all':\n print('count: 2\\nsloppy-script:a.js\\nstrict-script:b.js');sys.exit(0)\n"
            "if subcommand=='run':\n print('no matching execution',file=sys.stderr);sys.exit(1)\n"
            "assert subcommand=='run-list' and '--reuse-case-workers' in sys.argv\n"
            "ids=Path(sys.argv[sys.argv.index('run-list')+1]).read_text().splitlines()\n"
            f"interrupted={interrupted!r} and '--resume' not in sys.argv\n"
            "if interrupted:ids=ids[:1]\n"
            "outcomes={key:('Bug' if key.startswith('strict-script:') else 'Success') for key in ids}\n"
            "value={'snapshot_version':8,'run_kind':'full','execution_backend':'wasm-aot','compiler_identity':identity,"
            "'pinned_revisions':{'ecma262':'0'*40,'test262':'1'*40},'total':len(ids),"
            "'passed':sum(v=='Success' for v in outcomes.values()),'completed_test_ids':ids,'timeout_test_ids':[],"
            "'counts_per_outcome':{k:sum(v==k for v in outcomes.values()) for k in ['Success','NotImplemented','Crash','Bug']},"
            "'failures':[{'test_id':k,'outcome':v} for k,v in outcomes.items() if v!='Success']}\n"
            "destination=Path(sys.argv[sys.argv.index('--snapshot-dir')+1]);destination.mkdir(exist_ok=True)\n"
            "(destination/'exact-batch-123.json').write_text(json.dumps(value))\n"
            "print('total: '+str(len(ids)))\n"
            "sys.exit(2 if interrupted else int(value['passed']!=len(ids)))\n"
        )
        binary.chmod(0o755)
        return [sys.executable, str(SCRIPTS / "replay-test262-executions.py"), str(executions),
                "--binary", str(binary), "--suite-root", str(root / "suite"),
                "--output-dir", str(root / "evidence"), "--workers", "1", "--reuse-case-workers"]

    def test_one_native_batch_retains_failures_and_absent_ids_in_original_denominator(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); command = self.create(root)
            result = subprocess.run(command, capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            summary = json.loads((root / "evidence/summary.json").read_text())
            self.assertEqual(summary["total"], 3)
            self.assertEqual(summary["outcomes"], {"Success": 1, "NotImplemented": 0, "Crash": 0, "Bug": 1})
            self.assertEqual(sum("infrastructure_error" in row for row in summary["results"]), 1)
            self.assertEqual((root / "calls").read_text().splitlines(), ["list-all False", "run False", "run-list False"])
            repeated = subprocess.run(command + ["--resume"], capture_output=True, text=True, timeout=15)
            self.assertEqual(repeated.returncode, 2, repeated.stdout + repeated.stderr)
            self.assertEqual((root / "calls").read_text().splitlines(), ["list-all False", "run False", "run-list False"])

    def test_existing_file_under_missing_mode_gets_its_own_native_infrastructure_error(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); command = self.create(root)
            with (root / "executions").open("a") as stream:
                stream.write("raw-script:a.js\n")
            result = subprocess.run(command, capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            summary = json.loads((root / "evidence/summary.json").read_text())
            self.assertEqual(summary["total"], 4)
            missing = next(row for row in summary["results"] if row["execution_id"] == "raw-script:a.js")
            self.assertIn("infrastructure_error", missing)
            self.assertNotIn("outcome", missing)
            self.assertEqual(sum(row.get("outcome") == "Success" for row in summary["results"]), 1)

    def test_interruption_preserves_native_checkpoint_and_resume_uses_same_exact_cohort(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); command = self.create(root, interrupted=True)
            first = subprocess.run(command, capture_output=True, text=True, timeout=15)
            self.assertNotEqual(first.returncode, 0)
            self.assertFalse((root / "evidence/summary.json").exists())
            self.assertEqual(len(json.loads((root / "evidence/snapshots/exact-batch-123.json").read_text())["completed_test_ids"]), 1)
            resumed = subprocess.run(command + ["--resume"], capture_output=True, text=True, timeout=15)
            self.assertEqual(resumed.returncode, 2, resumed.stdout + resumed.stderr)
            self.assertEqual((root / "calls").read_text().splitlines(), ["list-all False", "run False", "run-list False", "run-list True"])
            self.assertEqual(json.loads((root / "evidence/summary.json").read_text())["total"], 3)

    def test_resume_rejects_changed_snapshot_transcript_result_list_mode_and_threads(self):
        for changed in ["snapshot", "transcript", "outcome", "batch", "mode", "threads", "overlap",
                        "snapshot traversal", "transcript traversal", "fixture"]:
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory); command = self.create(root)
                first = subprocess.run(command, capture_output=True, text=True, timeout=15)
                self.assertEqual(first.returncode, 2, first.stdout + first.stderr)
                evidence = root / "evidence"
                summary = json.loads((evidence / "summary.json").read_text())
                row = next(row for row in summary["results"] if row.get("protocol"))
                if changed == "snapshot":
                    (evidence / row["native_snapshot"]["path"]).write_text("{}")
                elif changed == "transcript":
                    (evidence / row["transcript"]).write_text("replacement evidence")
                elif changed == "outcome":
                    row["outcome"] = "Success" if row["outcome"] != "Success" else "Bug"
                    (evidence / (hashlib.sha256(row["execution_id"].encode()).hexdigest() + ".json")).write_text(json.dumps(row))
                elif changed == "batch":
                    (evidence / "batch-executions").write_text("sloppy-script:a.js\n")
                elif changed == "mode":
                    command.remove("--reuse-case-workers")
                elif changed == "threads":
                    command[command.index("--workers") + 1] = "2"
                elif changed == "overlap":
                    manifest_path = evidence / "run.json"
                    manifest = json.loads(manifest_path.read_text())
                    manifest["absent_executions"].append(manifest["batch_executions"][0])
                    manifest_path.write_text(json.dumps(manifest))
                elif changed == "fixture":
                    (root / "suite/test/a.js").write_text("changed original fixture")
                else:
                    key = "native_snapshot" if changed == "snapshot traversal" else "transcript"
                    original = row[key]["path"] if key == "native_snapshot" else row[key]
                    outside = root / "outside"
                    outside.write_bytes((evidence / original).read_bytes())
                    if key == "native_snapshot":
                        row[key]["path"] = str(outside)
                    else:
                        row[key] = "../outside"
                    (evidence / (hashlib.sha256(row["execution_id"].encode()).hexdigest() + ".json")).write_text(json.dumps(row))
                result = subprocess.run(command + ["--resume"], capture_output=True, text=True, timeout=15)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(len((root / "calls").read_text().splitlines()), 3)


if __name__ == "__main__":
    unittest.main()
