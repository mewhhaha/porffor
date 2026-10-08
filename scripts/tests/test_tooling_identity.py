"""Identity checks ignore local recovery files but still reject product drift."""

from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


REPO = Path(__file__).resolve().parents[2]
SCRIPT = REPO / "scripts/check-lila-identity.sh"
RETIRED_NAME = "por" + "ffor"


class IdentityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git("init", "-q")
        shutil.copyfile(REPO / ".gitignore", self.root / ".gitignore")
        mapping = self.root / "docs/rust-rewrite/lila-identity-map.tsv"
        mapping.parent.mkdir(parents=True)
        shutil.copyfile(REPO / mapping.relative_to(self.root), mapping)
        for package in (
            "front", "ir", "runtime", "spec-exec", "aot-wasm", "backend-c",
            "backend-native", "engine", "test262", "cli",
        ):
            (self.root / f"crates/lila-{package}").mkdir(parents=True)
        entry = self.root / "crates/lila-cli/src/bin/lila.rs"
        entry.parent.mkdir(parents=True)
        entry.write_text("fn main() {}\n", encoding="utf-8")

    def git(self, *args):
        return subprocess.run(
            ["git", "-C", str(self.root), *args], check=True,
            capture_output=True, text=True,
        )

    def old_identity_file(self, relative):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        contents = f'{{"recovery_path": "/old/checkout/{RETIRED_NAME}/source"}}\n'
        path.write_text(contents, encoding="utf-8")
        return path, contents

    def audit(self):
        return subprocess.run(
            ["bash", str(SCRIPT)], cwd=self.root,
            capture_output=True, text=True, timeout=10,
        )

    def test_local_recovery_is_ignored_and_preserved(self):
        path, contents = self.old_identity_file(".lila-task-work/recovery/SOURCE.json")
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("check-lila-identity: ok", result.stdout)
        self.assertEqual(path.read_text(encoding="utf-8"), contents)
        listed = self.git("ls-files", "--others", "--exclude-standard").stdout
        self.assertNotIn(".lila-task-work/", listed)

    def test_forced_tracked_recovery_still_gets_audited(self):
        relative = ".lila-task-work/recovery/SOURCE.json"
        self.old_identity_file(relative)
        self.git("add", "-f", "--", relative)
        result = self.audit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(relative, result.stderr)

    def test_untracked_product_and_nested_recovery_still_get_audited(self):
        for relative in (
            "crates/lila-cli/src/new_product.json",
            "crates/lila-cli/.lila-task-work/SOURCE.json",
        ):
            with self.subTest(path=relative):
                path, _ = self.old_identity_file(relative)
                result = self.audit()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(relative, result.stderr)
                path.unlink()


if __name__ == "__main__":
    unittest.main()
