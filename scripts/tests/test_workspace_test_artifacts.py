"""Cargo identity controls for same-name workspace native test selectors."""

import importlib.util
from pathlib import Path
import unittest

SOURCE = Path(__file__).resolve().parents[1] / "workspace_test_artifacts.py"
spec = importlib.util.spec_from_file_location("workspace_test_artifacts", SOURCE)
artifacts = importlib.util.module_from_spec(spec)
spec.loader.exec_module(artifacts)


def message(package, name="differential_corpus", kind="test", *, test=True):
    return {"reason": "compiler-artifact", "package_id": package,
            "target": {"name": name, "kind": [kind]},
            "profile": {"test": test}, "executable": f"/tmp/{package}/{name}"}


class WorkspaceTestArtifactTests(unittest.TestCase):
    def test_same_name_targets_keep_exact_package_identity(self):
        keys = [(package, "differential_corpus", "test") for package in ["cli-id", "test262-id"]]
        found = artifacts.test_artifacts([message("test262-id"), message("cli-id")], keys)
        self.assertEqual(found[keys[0]], Path("/tmp/cli-id/differential_corpus"))
        self.assertEqual(found[keys[1]], Path("/tmp/test262-id/differential_corpus"))

    def test_missing_sibling_is_not_silently_replaced_by_same_name(self):
        with self.assertRaisesRegex(ValueError, "missing"):
            artifacts.test_artifacts([message("cli-id")], [
                ("cli-id", "differential_corpus", "test"),
                ("test262-id", "differential_corpus", "test")])

    def test_duplicate_and_unexpected_test_images_fail_closed(self):
        key = [("cli-id", "differential_corpus", "test")]
        for rows in [[message("cli-id"), message("cli-id")], [message("other-id")]]:
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                artifacts.test_artifacts(rows, key)

    def test_normal_binary_and_nonexecutables_do_not_enter_test_inventory(self):
        found = artifacts.test_artifacts([
            message("cli-id", test=False), {"reason": "build-finished", "success": True},
            message("test262-id")], [("test262-id", "differential_corpus", "test")])
        self.assertEqual(len(found), 1)

    def test_target_kind_cannot_be_substituted(self):
        with self.assertRaisesRegex(ValueError, "unexpected"):
            artifacts.test_artifacts([message("cli-id", kind="bin")],
                                    [("cli-id", "differential_corpus", "test")])

    def test_package_selection_is_stable_for_every_selector(self):
        self.assertEqual(artifacts.package_selection(), ["--workspace"])
        self.assertEqual(artifacts.package_selection(["lila-cli", "lila-test262"]),
                         ["-p", "lila-cli", "-p", "lila-test262"])

    def test_library_cohort_and_duplicate_selectors_share_build_keys(self):
        self.assertEqual(artifacts.selector_key({"kind": "lib", "target": "lila_cli"}),
                         artifacts.selector_key({"kind": "lib", "target": "lila_test262"}))
        self.assertNotEqual(artifacts.selector_key({"kind": "bin", "target": "lila"}),
                            artifacts.selector_key({"kind": "test", "target": "lila"}))


if __name__ == "__main__":
    unittest.main()
