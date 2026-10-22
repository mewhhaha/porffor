import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
TOOL = Path(os.environ.get("SOURCE_GUARD_RUNNER", SCRIPTS / "check-source-guards.py"))
SPEC = importlib.util.spec_from_file_location("source_guard_runner", TOOL)
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)


class SourceGuardRunnerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / ".cargo").mkdir()
        (self.root / ".cargo/config.toml").write_text("[build]\njobs = 1\n")
        (self.root / "Cargo.toml").write_text('[workspace.package]\nedition = "2024"\n')
        entry = self.root / "crates/scriptw-aot-wasm/tests/structure_builtins"
        entry.mkdir(parents=True)
        (entry / "main.rs").write_text("\n".join(
            f"mod {name};" for name in sorted(RUNNER.CARGO_MODULES | {"guard_a", "guard_b"})
        ) + "\n")
        self.output = self.root / "evidence"
        self.output.mkdir()

    def invoke(self, inventory, summary, status=0):
        def native(command, **kwargs):
            if "--test" in command:
                return subprocess.CompletedProcess(command, 0)
            if "--list" in command:
                self.assertEqual(command[1:], ["--list"])
                return subprocess.CompletedProcess(command, 0, inventory, "")
            kwargs["stdout"].write(summary)
            return subprocess.CompletedProcess(command, status)
        with patch.object(RUNNER.subprocess, "run", side_effect=native), contextlib.redirect_stdout(io.StringIO()):
            return RUNNER.run(self.root, self.output)

    def test_complete_inventory_and_exact_success_are_recorded(self):
        self.assertEqual(self.invoke(
            "guard_a::check: test\nguard_b::check: test\n\n2 tests, 0 benchmarks\n",
            "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n",
        ), 0)
        receipt = json.loads((self.output / "receipt.json").read_text())
        self.assertTrue(receipt["passed"])
        self.assertEqual(len(receipt["test_inventory"]), 2)

    def test_empty_native_inventory_cannot_be_green(self):
        with self.assertRaisesRegex(ValueError, "nonempty"):
            self.invoke("0 tests, 0 benchmarks\n", "")

    def test_missing_cargo_config_stops_before_compilation(self):
        (self.root / ".cargo/config.toml").unlink()
        with patch.object(RUNNER.subprocess, "run") as native:
            with self.assertRaisesRegex(ValueError, "missing tracked .cargo/config.toml"):
                RUNNER.run(self.root, self.output)
        native.assert_not_called()

    def test_every_selected_module_must_own_an_executed_test(self):
        with self.assertRaisesRegex(ValueError, "every selected module"):
            self.invoke("guard_a::check: test\n\n1 test, 0 benchmarks\n", "")

    def test_native_zero_exit_cannot_hide_unexecuted_inventory(self):
        with self.assertRaisesRegex(ValueError, "exact nonempty inventory"):
            self.invoke(
                "guard_a::check: test\nguard_b::check: test\n\n2 tests, 0 benchmarks\n",
                "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s\n",
            )

    def test_actual_guard_failure_propagates_and_remains_failed_evidence(self):
        code = self.invoke(
            "guard_a::check: test\nguard_b::check: test\n\n2 tests, 0 benchmarks\n",
            "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",
            status=101,
        )
        self.assertEqual(code, 101)
        receipt = json.loads((self.output / "receipt.json").read_text())
        self.assertFalse(receipt["passed"])
        self.assertEqual(receipt["test_inventory"], ["guard_a::check", "guard_b::check"])


if __name__ == "__main__":
    unittest.main()
