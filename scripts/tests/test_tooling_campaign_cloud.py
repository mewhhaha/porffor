"""Exercise wrapper argument delivery; fake launchers confer no runtime acceptance."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GRAMMARS = ["integer-arithmetic-v1", "integer-bitwise-v2", "integer-product-v3",
            "object-probe-v1", "object-mutations-v2", "module-graph-v1", "module-graph-v2",
            "control-flow-v1", "negative-source-v1", "builtin-stateful-v2", "metamorphic-stateful-v2"]


class CampaignCloudWrappers(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix="lila cloud campaign ")
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.trace = self.root / "launcher.jsonl"
        self.cli = self.root / "selected CLI"
        self.cli.write_text("#!/bin/sh\nexit 0\n")
        self.cli.chmod(0o700)
        launcher = self.root / "python3"
        launcher.write_text(f"#!{sys.executable}\n" + """
import json, os, sys
with open(os.environ['CAMPAIGN_TRACE'], 'a') as trace:
    trace.write(json.dumps(sys.argv[1:]) + '\\n')
raise SystemExit(7 if os.environ.get('CAMPAIGN_FAIL_ARG') in sys.argv[1:] else 0)
""")
        launcher.chmod(0o700)
        self.environment = dict(os.environ, PATH=str(self.root) + os.pathsep + os.environ["PATH"],
                                CAMPAIGN_TRACE=str(self.trace), LILA_BIN=str(self.cli),
                                CLOSURE_EVIDENCE_DIR=str(self.root / "closure"))

    def invoke(self, script, arguments, fail_argument=None):
        self.trace.unlink(missing_ok=True)
        environment = dict(self.environment)
        if fail_argument is not None:
            environment["CAMPAIGN_FAIL_ARG"] = fail_argument
        result = subprocess.run(["bash", str(ROOT / "scripts" / script), *arguments],
                                cwd=ROOT, env=environment, capture_output=True, text=True)
        calls = [json.loads(line) for line in self.trace.read_text().splitlines()] if self.trace.exists() else []
        return result, calls

    def test_all_grammars_keep_counts_and_request_the_selected_budget(self):
        for cloud in (False, True):
            with self.subTest(cloud=cloud):
                output = self.root / ("cloud" if cloud else "local")
                arguments = (["--cloud"] if cloud else []) + [str(self.cli), str(output), "pr-fast"]
                result, calls = self.invoke("run-differential-campaign-tier.sh", arguments,
                                            fail_argument="object-probe-v1")
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual(len(calls), len(GRAMMARS), "a red grammar must not omit later evidence")
                flags = ["--cloud"] if cloud else ["--memory-mib", "4096"]
                for call, grammar in zip(calls, GRAMMARS):
                    self.assertEqual(call[:len(flags) + 2], ["scripts/limited_verification.py", *flags, "--"])
                    self.assertEqual(call[len(flags) + 2:len(flags) + 7],
                                     [str(self.cli), "--jobs", "1", "differential", "campaign"])
                    for flag, value in [("--grammar", grammar), ("--cases", "2"),
                                        ("--max-replays", "16"), ("--oracle", "spec-exec")]:
                        self.assertEqual(call[call.index(flag) + 1], value)

    def test_closure_uses_one_budget_for_sync_and_both_fresh_families(self):
        for cloud in (False, True):
            with self.subTest(cloud=cloud):
                result, calls = self.invoke("check-test262-closure.sh", ["--cloud"] if cloud else [])
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(len(calls), 2)
                flags = ["--cloud"] if cloud else ["--memory-mib", "4096"]
                for call in calls:
                    self.assertEqual(call[:len(flags) + 2],
                                     [str(ROOT / "scripts/limited_verification.py"), *flags, "--"])
                    self.assertEqual(call[len(flags) + 2:len(flags) + 6],
                                     [str(self.cli), "--jobs", "1", "test262"])
                self.assertIn("sync", calls[0])
                self.assertIn("close-release", calls[1])
                self.assertEqual(calls[1][-2:], ["--snapshot-dir", str(self.root / "closure")])
                result, calls = self.invoke("check-test262-closure.sh", ["--cloud"] if cloud else [],
                                            fail_argument="sync")
                self.assertEqual(result.returncode, 7)
                self.assertEqual(len(calls), 1, "failed admission cannot start the fresh families")

    def test_unknown_arguments_do_not_start_the_verification_launcher(self):
        for script in ("check-test262-closure.sh", "run-differential-campaign-tier.sh"):
            with self.subTest(script=script):
                result, calls = self.invoke(script, ["--unknown"])
                self.assertEqual(result.returncode, 2)
                self.assertEqual(calls, [])


if __name__ == "__main__":
    unittest.main()
