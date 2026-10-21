import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

RUNNER = Path(__file__).resolve().parents[1] / "run-ci-target.py"


class CiTargetTests(unittest.TestCase):
    def test_cargo_receives_literal_selector_and_original_exit_status(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cargo = root / "cargo"
            cargo.write_text(
                f"#!{sys.executable}\nimport json,sys\n"
                "print(json.dumps(sys.argv[1:]))\nsys.exit(19)\n"
            )
            cargo.chmod(0o755)
            result = subprocess.run(
                [sys.executable, str(RUNNER), "aot-integrations", "--release"],
                env=dict(os.environ, PATH=directory),
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 19)
            self.assertEqual(
                json.loads(result.stdout),
                [
                    "test",
                    "--release",
                    "--locked",
                    "-p",
                    "scriptw-aot-wasm",
                    "--test",
                    "*",
                    "--",
                    "--test-threads=1",
                ],
            )

    def test_unknown_target_fails_before_starting_cargo(self):
        result = subprocess.run(
            [sys.executable, str(RUNNER), "unknown"],
            env=dict(os.environ, PATH=""),
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("unknown CI target", result.stderr)


if __name__ == "__main__":
    unittest.main()
