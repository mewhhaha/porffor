"""T00: the developer wrapper must run the binary Cargo selected and built."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "dev.sh"


class DeveloperWrapperTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        cargo = self.bin / "cargo"
        cargo.write_text(f"#!{sys.executable} -S\n" + '''
import json, os, sys
with open("calls.jsonl", "a") as stream:
    stream.write(json.dumps({"args": sys.argv[1:], "target_dir": os.environ.get("CARGO_TARGET_DIR"),
                            "target": os.environ.get("CARGO_BUILD_TARGET")}) + "\\n")
sys.exit(int(os.environ.get("CARGO_STATUS", "0")))
''')
        cargo.chmod(0o755)
        stale = self.root / "target/debug/lila"
        stale.parent.mkdir(parents=True)
        stale.write_text("#!/bin/sh\nprintf 'stale binary executed\\n' >&2\nexit 99\n")
        stale.chmod(0o755)

    def run_dev(self, *args, **settings):
        env = dict(os.environ, PATH=f"{self.bin}{os.pathsep}{os.environ['PATH']}", LILA_JOBS="")
        env.update(settings)
        return subprocess.run(["sh", str(SCRIPT), *args], cwd=self.root, env=env,
                              capture_output=True, text=True, timeout=10)

    def calls(self):
        return [json.loads(line) for line in (self.root / "calls.jsonl").read_text().splitlines()]

    def test_custom_target_directory_does_not_run_stale_binary(self):
        directory = str(self.root / "custom target")
        result = self.run_dev("test262", "run", "a filter", "--suite-root", "suite with spaces",
                              CARGO_TARGET_DIR=directory)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls(), [{"args": ["run", "-p", "lila-cli", "--", "test262", "run",
                                                   "a filter", "--suite-root", "suite with spaces"],
                                        "target_dir": directory, "target": os.environ.get("CARGO_BUILD_TARGET")}])
        self.assertNotIn("stale binary", result.stderr)

    def test_cargo_owns_target_triple_and_runner_selection(self):
        result = self.run_dev("test262", "report", CARGO_BUILD_TARGET="aarch64-unknown-linux-gnu")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls()[0]["args"][0], "run")
        self.assertEqual(self.calls()[0]["target"], "aarch64-unknown-linux-gnu")

    def test_jobs_are_cargo_arguments_not_lila_arguments(self):
        result = self.run_dev("test262", "run", "--threads", "1", LILA_JOBS="2")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls()[0]["args"],
                         ["run", "--jobs", "2", "-p", "lila-cli", "--", "test262", "run", "--threads", "1"])

    def test_failure_status_propagates_without_running_stale_binary(self):
        result = self.run_dev("test262", "run", CARGO_STATUS="23")
        self.assertEqual(result.returncode, 23)
        self.assertEqual(len(self.calls()), 1)
        self.assertNotIn("stale binary", result.stderr)

    def test_other_developer_commands_keep_their_defaults(self):
        for command, expected in (("build", ["build", "-p", "lila-cli"]),
                                  ("check", ["check", "--workspace"]),
                                  ("timings", ["build", "--timings", "-p", "lila-ir", "-p", "lila-aot-wasm"])):
            with self.subTest(command=command):
                self.assertEqual(self.run_dev(command).returncode, 0)
                self.assertEqual(self.calls()[-1]["args"], expected)

    def test_invalid_job_count_never_invokes_cargo(self):
        result = self.run_dev("test262", LILA_JOBS="invalid")
        self.assertEqual(result.returncode, 2)
        self.assertFalse((self.root / "calls.jsonl").exists())


if __name__ == "__main__":
    unittest.main()
