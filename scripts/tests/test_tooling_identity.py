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

    def test_frozen_native_doctest_retains_exact_checkout_path(self):
        path = self.root / "docs/rust-rewrite/native-comparison.json"
        path.write_text(
            '{"eliminated": [\n'
            f'  "/workspace/{RETIRED_NAME}/crates/lila-intl-data/src/generated/mod.rs - baked::provider (line 5)"\n'
            ']}\n', encoding="utf-8",
        )
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_native_receipt_exception_does_not_admit_product_drift(self):
        for relative, contents in (
            ("docs/rust-rewrite/native-comparison.json", f'{{"message": "{RETIRED_NAME} runtime"}}\n'),
            ("docs/rust-rewrite/native-comparison.json", f'  "/workspace/{RETIRED_NAME}/crates/{RETIRED_NAME}-cli/src/main.rs - main (line 5)"\n'),
            ("crates/lila-cli/src/receipt.json", f'  "/workspace/{RETIRED_NAME}/crates/lila-cli/src/main.rs - main (line 5)"\n'),
        ):
            with self.subTest(path=relative, contents=contents):
                path = self.root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(contents, encoding="utf-8")
                result = self.audit()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(relative, result.stderr)
                path.unlink()

    def write_proof(self, value, filename="default-workspace-eb1ca63c9-native-checkpoint-20261015.json"):
        import json
        path = self.root / "docs/rust-rewrite" / filename
        path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
        return path

    def proof(self, **fields):
        return {"schema": "portable-current-workspace-retention-v1", **fields}

    def test_captured_proof_roots_and_worktree_paths_keep_literal_bytes(self):
        roots = [f"/workspace/{RETIRED_NAME}{suffix}" for suffix in ("", "-next", "-repairs")]
        value = self.proof(cwd=roots[1], argv=[root + "/crates/lila-engine/src/lib.rs" for root in roots],
            cargo_package="path+file://" + roots[1] + "/crates/lila-engine#0.1.0",
            index=roots[0] + f"/.git/worktrees/{RETIRED_NAME}-next/index",
            diagnostic="actual checkout " + roots[1] + "; source retained",
            dependency_output="left\n\n" + roots[0] + "/target/native.rmeta: input.rs\n")
        path = self.write_proof(value)
        original = path.read_bytes()
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(path.read_bytes(), original)

    def test_historical_proof_paths_do_not_admit_product_api_or_ui_drift(self):
        for drift in (f"{RETIRED_NAME}-cli", f"{RETIRED_NAME}_engine", "$" + RETIRED_NAME.title(),
                RETIRED_NAME + " runtime", f"/workspace/{RETIRED_NAME}-next/crates/{RETIRED_NAME}-cli/src/main.rs"):
            with self.subTest(drift=drift):
                self.write_proof(self.proof(cwd=f"/workspace/{RETIRED_NAME}-next", product=drift))
                result = self.audit()
                self.assertNotEqual(result.returncode, 0)

    def test_unregistered_json_wrong_schema_and_product_context_are_rejected(self):
        for relative, value in (
                ("docs/rust-rewrite/unregistered-proof.json", self.proof(cwd=f"/workspace/{RETIRED_NAME}-next")),
                ("docs/rust-rewrite/default-workspace-eb1ca63c9-native-checkpoint-20261015.json", {"schema": "wrong", "cwd": f"/workspace/{RETIRED_NAME}-next"}),
                ("crates/lila-cli/src/proof.json", self.proof(cwd=f"/workspace/{RETIRED_NAME}-next"))):
            with self.subTest(path=relative):
                import json
                path = self.root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(json.dumps(value) + "\n", encoding="utf-8")
                self.assertNotEqual(self.audit().returncode, 0)
                path.unlink()

    def test_lookalike_checkout_roots_are_not_historical_paths(self):
        for suffix in ("-nextish", "-next.bad", "-next-product"):
            with self.subTest(suffix=suffix):
                self.write_proof(self.proof(cwd=f"/workspace/{RETIRED_NAME}{suffix}"))
                self.assertNotEqual(self.audit().returncode, 0)
        for prefix in ("/foreign", "https://example.invalid"):
            with self.subTest(prefix=prefix):
                self.write_proof(self.proof(cwd=prefix + f"/workspace/{RETIRED_NAME}-next"))
                self.assertNotEqual(self.audit().returncode, 0)

    def test_historical_reader_must_keep_its_exact_captured_source(self):
        path = self.root / "docs/rust-rewrite/verify-workspace-eb1ca63c9-20261015.py"
        path.write_text(f"ROOT = '/workspace/{RETIRED_NAME}-next'\n", encoding="utf-8")
        result = self.audit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid historical provenance", result.stderr)

    def test_only_the_four_existing_source_fence_field_locators_are_admitted(self):
        filename = "workspace-source-revalidation-eb1ca63c9-20261015.json"
        value = {"schema": "portable-default-source-revalidation-eb1ca63c9-v1",
            "fields": [f".{phase}./workspace/{RETIRED_NAME}-next.real_index.{field}"
                for phase in ("before", "after") for field in ("bytes", "sha256")]}
        path = self.write_proof(value, filename)
        original = path.read_bytes()
        self.assertEqual(self.audit().returncode, 0)
        self.assertEqual(path.read_bytes(), original)
        value["fields"].append(f".before./workspace/{RETIRED_NAME}-next.real_index.product")
        self.write_proof(value, filename)
        self.assertNotEqual(self.audit().returncode, 0)

    def test_malformed_or_duplicate_registered_schema_is_rejected(self):
        for text in ('{"schema":', '{"schema":"portable-current-workspace-retention-v1","schema":"portable-current-workspace-retention-v1"}'):
            with self.subTest(text=text):
                path = self.write_proof(self.proof())
                path.write_text(text, encoding="utf-8")
                self.assertNotEqual(self.audit().returncode, 0)


if __name__ == "__main__":
    unittest.main()
