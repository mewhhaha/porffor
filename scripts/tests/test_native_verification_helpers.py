"""Reject ambiguous Cargo artifacts and source changes before native admission."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import native_test_build
from verification_identity import source_map
from workspace_test_artifacts import continuation_cases


class NativeVerificationHelperTests(unittest.TestCase):
    def test_cargo_target_name_cannot_substitute_a_foreign_package_or_kind(self):
        metadata = {
            "workspace_members": ["owner"],
            "packages": [{"name": "example", "id": "owner"}],
        }
        artifact = {
            "reason": "compiler-artifact",
            "package_id": "owner",
            "target": {"name": "same", "kind": ["test"]},
            "profile": {"test": True},
            "executable": "/tmp/same",
        }
        finish = {"reason": "build-finished", "success": True}
        for mutation in (None, "package", "kind", "duplicate", "unfinished"):
            with self.subTest(mutation=mutation):
                row = json.loads(json.dumps(artifact))
                if mutation == "package":
                    row["package_id"] = "foreign"
                if mutation == "kind":
                    row["target"]["kind"] = ["bin"]
                rows = [row, finish]
                if mutation == "duplicate":
                    rows.insert(0, row)
                if mutation == "unfinished":
                    rows.pop()
                result = subprocess.CompletedProcess(
                    [], 0, stdout="\n".join(map(json.dumps, rows))
                )
                with patch.object(
                    native_test_build.subprocess,
                    "check_output",
                    return_value=json.dumps(metadata),
                ), patch.object(
                    native_test_build.subprocess, "run", return_value=result
                ):
                    if mutation is None:
                        self.assertEqual(
                            native_test_build.build_test_binary(
                                "example", "same", "test"
                            ),
                            Path("/tmp/same"),
                        )
                    else:
                        with self.assertRaises(ValueError):
                            native_test_build.build_test_binary(
                                "example", "same", "test"
                            )

    def test_source_identity_binds_dirty_and_untracked_bytes_but_ignores_build_outputs(
        self,
    ):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def git(*args):
                subprocess.run(
                    ["git", *args], cwd=root, check=True, capture_output=True
                )

            git("init")
            (root / ".gitignore").write_text("target/\n")
            (root / "source").write_text("before")
            git("add", ".")
            git(
                "-c",
                "user.name=Control",
                "-c",
                "user.email=control@example.invalid",
                "commit",
                "-m",
                "control",
            )
            before = source_map(root)
            (root / "target").mkdir()
            (root / "target" / "output").write_text("ignored")
            self.assertEqual(source_map(root), before)
            (root / "source").write_text("after")
            dirty = source_map(root)
            self.assertNotEqual(
                dirty["tracked_patch_sha256"], before["tracked_patch_sha256"]
            )
            (root / "new source").write_text("first")
            untracked = source_map(root)
            (root / "new source").write_text("second")
            self.assertNotEqual(source_map(root)["untracked"], untracked["untracked"])

    def test_continuation_keeps_original_failures_and_selects_only_admitted_pending_scopes(
        self,
    ):
        cases = [
            {
                "package": "p",
                "target": name,
                "kind": "test",
                "selector": ["--test", name],
            }
            for name in ("passed", "failed", "pending", "interrupted", "barrier")
        ]
        previous = [
            dict(case, state=state)
            for case, state in zip(
                cases, ("passed", "failed", None, "interrupted", None)
            )
        ]
        options = dict(
            retry_interrupted=True,
            barrier_phase=False,
            is_barrier=lambda c: c["target"] == "barrier",
        )
        selected = continuation_cases(cases, previous, **options)
        self.assertEqual([c["target"] for c in selected], ["pending", "interrupted"])
        self.assertEqual(len(cases), 5)
        previous[2]["selector"] = ["--lib"]
        with self.assertRaisesRegex(ValueError, "selector changed"):
            continuation_cases(cases, previous, **options)


if __name__ == "__main__":
    unittest.main()
