"""Admission controls use fixture bytes; actual native staging has its own owner."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import native_verification_worker as staging


class NativeWorkerAdmissionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="scriptw-native-worker-admission-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / ".gitignore").write_text("target/\n")
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        subprocess.run(["git", "add", ".gitignore"], cwd=self.root, check=True)
        subprocess.run(["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                        "commit", "-qm", "fixture"], cwd=self.root, check=True)
        self.target = self.root / "target"
        build = self.target / "owned-build"
        build.mkdir(parents=True)
        self.image = build / "scriptw-worker"
        self.image.write_bytes(b"fixture, never executed")
        self.shared = self.target / "shared-test-runtime-images" / "fixture.so"
        self.shared.parent.mkdir()
        self.shared.write_bytes(b"shared fixture, never loaded")
        bound = staging.source(self.root)
        self.data = {"complete": True, "passed": True, "source_unchanged": True,
                     "source_before": bound, "source_after": bound,
                     "native_cli_worker": {"path": str(self.image), "bytes": self.image.stat().st_size,
                         "sha256": staging.sha(self.image), "graph": "fixture graph",
                         "cargo_artifact": {"target": {"name": "scriptw", "kind": ["bin"]},
                             "profile": {"test": False, "opt_level": "3"}, "features": []},
                         "link": {"shared_image": str(self.shared), "shared_image_sha256": staging.sha(self.shared)},
                         "identity": {"source_revision": {"kind": "git-commit", "commit": bound["head"]}}}}
        self.owner = build / "receipt.json"
        self.output = self.target / "staged"

    def rejects(self, message):
        self.owner.write_text(json.dumps(self.data))
        with self.assertRaisesRegex(ValueError, message):
            staging.stage(self.root, self.target, self.output, self.owner,
                          profile="release", oracle=False, share_runtime_images=True)
        self.assertFalse(self.output.exists(), "invalid evidence must reject before staging")

    def test_failed_native_owner_cannot_stage_a_worker(self):
        self.data["passed"] = False
        self.rejects("incomplete or failed")

    def test_different_source_owner_cannot_stage_a_worker(self):
        self.data["source_before"]["head"] = "0" * 40
        self.rejects("different source")

    def test_changed_captured_cli_bytes_cannot_stage_a_worker(self):
        self.image.write_bytes(b"different executable")
        self.rejects("CLI bytes changed")

    def test_changed_shared_runtime_bytes_cannot_stage_a_worker(self):
        self.shared.write_bytes(b"different runtime image")
        self.rejects("runtime image changed")

    def test_oracle_artifact_cannot_be_staged_as_default_product(self):
        self.data["native_cli_worker"]["cargo_artifact"]["features"] = ["spec-exec-oracle"]
        self.rejects("profile/oracle features")

    def test_modified_source_cannot_reuse_a_bound_native_worker(self):
        (self.root / ".gitignore").write_text("target/\n# modified source\n")
        self.rejects("clean source")


if __name__ == "__main__":
    unittest.main()
