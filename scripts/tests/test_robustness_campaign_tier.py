"""Driver controls use fake CLI responses and never establish product acceptance."""

import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
spec = importlib.util.spec_from_file_location("robustness_tier", SCRIPTS / "run-robustness-campaign-tier.py")
tier = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = tier
spec.loader.exec_module(tier)


class RobustnessTierTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix="lila robustness tier ")
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.output = self.root / "evidence"
        self.cli = self.root / "fake CLI"
        self.cli.write_bytes(b"not an executable compiler; driver fixture only\n")
        self.seeds = [tier.Seed("frontmatter", b"/* seed */\x00\xff", True, 384),
                      tier.Seed("corpus", b"{}", True, 640)]

    def response(self, command, **kwargs):
        # This stub writes native evidence shape solely to exercise the driver.
        def argument(flag):
            return command[command.index(flag) + 1]

        target = argument("--target")
        seed = next(seed for seed in self.seeds if seed.target == target)
        output = Path(argument("--output-dir"))
        output.mkdir()
        raw = Path(argument("--input")).read_bytes()
        self.assertEqual(raw, seed.data)
        self.assertEqual(command[:5], [str(self.cli), "--jobs", "1", "differential", "robustness"])
        self.assertEqual(argument("--timeout-ms"), str(tier.ATTEMPT_TIMEOUT_MS))
        self.assertEqual(argument("--oracle"), "spec-exec")
        self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
        first_seed, count = int(argument("--seed")), int(argument("--cases"))
        (output / "base.bin").write_bytes(raw)
        (output / "base.input.json").write_bytes(tier.json_bytes({
            "schema_version": 1, "id": "robustness/input", "target": seed.wire_target(),
            "timeout_ms": tier.ATTEMPT_TIMEOUT_MS, "bytes_hex": raw.hex()}))
        records = []
        for ordinal in range(count):
            request, observation = f"case-{ordinal:03}.input.json", f"case-{ordinal:03}.observation.json"
            for name in (request, observation):
                (output / name).write_text("{}")
            records.append({"ordinal": ordinal, "seed": first_seed + ordinal, "request": request,
                "observation": observation, "disposition": "rejected", "completed_without_failure": True})
        (output / "robustness.json").write_bytes(tier.json_bytes({"schema_version": 1,
            "controller_identity": {"executable_sha256": tier.file_digest(self.cli)},
            "target": seed.wire_target(), "seed": first_seed, "total": count, "completed": count,
            "state": "finished", "semantic_equivalence": "not_established", "cases": records}))
        return subprocess.CompletedProcess(command, 0)

    def run_driver(self, response=None):
        with patch.object(tier.subprocess, "run", side_effect=response or self.response) as command:
            result = tier.run_tier(self.cli, self.output, "pr-fast", self.seeds)
        return result, command

    def test_complete_inventory_retains_exact_bytes_and_does_not_resume_old_output(self):
        result, command = self.run_driver()
        self.assertEqual(result, 0)
        self.assertEqual(command.call_count, 2)
        report = json.loads((self.output / "tier.json").read_bytes())
        self.assertEqual(report["state"], "completed")
        self.assertTrue(all(record["state"] == "completed" for record in report["targets"]))
        with patch.object(tier.subprocess, "run") as replay:
            with self.assertRaises(FileExistsError):
                tier.run_tier(self.cli, self.output, "pr-fast", self.seeds)
            replay.assert_not_called()

    def test_zero_exit_cannot_hide_missing_evidence_and_independent_targets_still_run(self):
        def response(command, **kwargs):
            completed = self.response(command, **kwargs)
            if command[command.index("--target") + 1] == "frontmatter":
                output = Path(command[command.index("--output-dir") + 1])
                (output / "case-001.observation.json").unlink()
            return completed

        result, command = self.run_driver(response)
        self.assertEqual((result, command.call_count), (1, 2))
        report = json.loads((self.output / "tier.json").read_bytes())
        self.assertEqual([record["state"] for record in report["targets"]], ["failed", "completed"])
        self.assertIn("lost its original input", report["targets"][0]["error"])

    def test_partial_failed_or_foreign_report_never_becomes_completed(self):
        for change in ({"completed": 1}, {"state": "failed"}, {"seed": 999},
                       {"target": {"kind": "snapshot"}}, {"controller_identity": {}},
                       {"semantic_equivalence": "established"}):
            with self.subTest(change=change), tempfile.TemporaryDirectory(dir=self.root) as root:
                def response(command, **kwargs):
                    completed = self.response(command, **kwargs)
                    path = Path(command[command.index("--output-dir") + 1]) / "robustness.json"
                    report = json.loads(path.read_bytes())
                    report.update(change)
                    path.write_bytes(tier.json_bytes(report))
                    return completed

                with patch.object(tier.subprocess, "run", side_effect=response):
                    result = tier.run_tier(self.cli, Path(root) / "fresh", "pr-fast", self.seeds)
                self.assertEqual(result, 1)

    def test_interruption_retains_running_and_pending_inventory_without_completed_tier(self):
        with self.assertRaises(KeyboardInterrupt):
            self.run_driver(lambda *args, **kwargs: (_ for _ in ()).throw(KeyboardInterrupt()))
        report = json.loads((self.output / "tier.json").read_bytes())
        self.assertEqual(report["state"], "incomplete")
        self.assertEqual([record["state"] for record in report["targets"]], ["interrupted", "pending"])

    def test_pr_seeds_are_an_identical_subset_of_the_complete_native_target_inventory(self):
        repository = SCRIPTS.parent
        small, large = tier.inventory(repository, "pr-fast"), tier.inventory(repository, "nightly")
        self.assertEqual((len(small), len(large)), (7, 19))
        self.assertTrue(all(seed in large and seed.native for seed in small))
        self.assertEqual(len({seed.first_seed for seed in large}), len(large))
        for seed in large:
            self.assertLessEqual(seed.first_seed + tier.TIERS["nightly"], 19 * 128)
            self.assertLessEqual(len(seed.data), tier.SEED_LIMIT)


if __name__ == "__main__":
    unittest.main()
