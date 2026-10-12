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


    def test_excluded_test_roots_are_absent_from_normal_cargo_selection(self):
        self.assertEqual(artifacts.package_selection(excluded=["lila-intl"]),
                         ["--workspace", "--exclude", "lila-intl"])
        self.assertEqual(artifacts.package_selection(
            ["lila-cli", "lila-intl", "lila-test262"], excluded=["lila-intl"]),
            ["-p", "lila-cli", "-p", "lila-test262"])

    def test_exclusion_cannot_implicitly_select_the_default_package(self):
        with self.assertRaisesRegex(ValueError, "nonempty"):
            artifacts.package_selection(["lila-intl"], excluded=["lila-intl"])

    def test_isolated_library_and_named_targets_cannot_widen_normal_cohorts(self):
        for kind in ["lib", "test"]:
            normal = {"package": "lila-cli", "kind": kind, "target": "native_contract"}
            isolated = {"package": "lila-intl", "kind": kind, "target": "native_contract"}
            sibling = {"package": "lila-test262", "kind": kind, "target": "native_contract"}
            self.assertNotEqual(artifacts.selector_key(normal, ["lila-intl"]),
                                artifacts.selector_key(isolated, ["lila-intl"]))
            self.assertEqual(artifacts.selector_key(normal, ["lila-intl"]),
                             artifacts.selector_key(sibling, ["lila-intl"]))

    def test_distinct_isolated_packages_keep_distinct_cargo_cohorts(self):
        cases = [{"package": p, "kind": "test", "target": "same"}
                 for p in ["lila-intl", "lila-cli"]]
        self.assertNotEqual(*[artifacts.selector_key(c, ["lila-intl", "lila-cli"])
                              for c in cases])

    def test_docs_default_to_one_complete_workspace_cohort(self):
        docs = [{"package": p, "kind": "doc"} for p in ["lila-cli", "lila-intl"]]
        self.assertEqual(artifacts.doc_cohorts(docs),
                         [("doc:*", ["--workspace"], {"lila-cli", "lila-intl"})])

    def test_docs_keep_intl_test_data_out_of_normal_roots(self):
        docs = [{"package": p, "kind": "doc"} for p in ["lila-cli", "lila-intl"]]
        self.assertEqual(artifacts.doc_cohorts(docs, isolated_packages=["lila-intl"]), [
            ("doc:*", ["--workspace", "--exclude", "lila-intl"], {"lila-cli"}),
            ("doc:lila-intl::*", ["-p", "lila-intl"], {"lila-intl"})])

    def test_docs_subset_and_all_isolated_selection_remain_exact(self):
        docs = [{"package": p, "kind": "doc"} for p in ["lila-intl", "lila-cli"]]
        self.assertEqual(artifacts.doc_cohorts(
            docs, ["lila-intl", "lila-cli"], ["lila-intl", "lila-cli"]), [
            ("doc:lila-cli::*", ["-p", "lila-cli"], {"lila-cli"}),
            ("doc:lila-intl::*", ["-p", "lila-intl"], {"lila-intl"})])
        self.assertEqual(artifacts.doc_cohorts(docs, ["lila-intl", "lila-cli"], ["lila-intl"])[0],
                         ("doc:*", ["-p", "lila-cli"], {"lila-cli"}))

    def test_docs_without_doctests_remain_in_cargo_selection(self):
        inventory = [{"package": "normal", "kind": "doc"},
                     {"package": "no-doc", "kind": "lib"},
                     {"package": "isolated-no-doc", "kind": "lib"}]
        self.assertEqual(artifacts.doc_cohorts(
            inventory, ["normal", "no-doc", "isolated-no-doc"], ["isolated-no-doc"]),
            [("doc:*", ["-p", "normal", "-p", "no-doc"], {"normal"})])

    def test_docs_empty_inventory_never_selects_implicit_default_root(self):
        self.assertEqual(artifacts.doc_cohorts([], ["lila-intl"], ["lila-intl"]), [])

    def test_docs_duplicate_package_inventory_fails_closed(self):
        docs = [{"package": "lila-cli", "kind": "doc"}] * 2
        with self.assertRaisesRegex(ValueError, "duplicate"):
            artifacts.doc_cohorts(docs)

    def test_doc_cohort_union_covers_every_package_exactly_once(self):
        names = {"a", "b", "c", "d"}
        inventory = [{"package": p, "kind": "doc"} for p in sorted(names)]
        for isolated in [[], ["b"], ["a", "c"], sorted(names)]:
            groups = artifacts.doc_cohorts(inventory, isolated_packages=isolated)
            covered = [p for _, _, docs in groups for p in docs]
            self.assertEqual(set(covered), names)
            self.assertEqual(len(covered), len(names))

if __name__ == "__main__":
    unittest.main()
