import importlib.util
import json
import pathlib
import shutil
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parents[1] / "generate-intl-segmenter-identity.py"
SPEC = importlib.util.spec_from_file_location("segmenter_identity", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
ROOT = SCRIPT.parents[1]

class SegmenterIdentityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temp.name)
        shutil.copytree(ROOT / MODULE.DATA, self.root / MODULE.DATA)
        shutil.copyfile(ROOT / "Cargo.lock", self.root / "Cargo.lock")
        self.manifest_path = self.root / MODULE.DATA / "manifest.json"
        self.manifest_path.chmod(0o600)
        self.original = json.loads(self.manifest_path.read_bytes())
    def tearDown(self):
        self.temp.cleanup()
    def reject(self, changed):
        self.manifest_path.write_text(json.dumps(changed))
        with self.assertRaises(ValueError):
            MODULE.verified_manifest(self.root)
    def test_original_official_archive_and_full_normative_corpora(self):
        MODULE.verified_manifest(self.root)
    def test_reject_removed_or_substituted_complex_model(self):
        changed = self.original.copy(); changed["required_complex_models"] = {"lstm": ["Thai_"], "dictionary": ["cjdict"]}; self.reject(changed)
    def test_reject_missing_locale_override(self):
        changed = self.original.copy(); changed["required_tailored_overrides"] = {"word": ["sv"], "sentence": ["el"]}; self.reject(changed)
    def test_reject_changed_official_archive_bytes(self):
        path = self.root / MODULE.DATA / self.original["packages"][0]["archive"]
        path.chmod(0o600); data = bytearray(path.read_bytes()); data[-1] ^= 1; path.write_bytes(data)
        with self.assertRaises(ValueError): MODULE.verified_manifest(self.root)
    def test_reject_corpus_tampering_without_silent_vector_exclusions(self):
        path = self.root / MODULE.DATA / "corpora/WordBreakTest.txt"
        path.chmod(0o600); path.write_bytes(path.read_bytes() + b"\n# changed\n")
        with self.assertRaises(ValueError): MODULE.verified_manifest(self.root)
    def test_reject_unbound_lock_or_omitted_auto_feature_proposal(self):
        path = self.root / "Cargo.lock"; path.chmod(0o600); path.write_text(path.read_text().replace(MODULE.PINS["icu_segmenter"][1], "0" * 64))
        with self.assertRaises(ValueError): MODULE.verified_manifest(self.root)
        shutil.copyfile(ROOT / "Cargo.lock", path)
        changed = self.original.copy(); changed["feature_proposal"] = {"required": ["compiled_data"], "actual_manifest_edit": False}; self.reject(changed)

    def test_actual_transitive_auto_package_closure_is_bound(self):
        packages = MODULE.locked_packages(self.root)
        names = {(p["name"], p["version"]) for p in packages}
        self.assertIn(("core_maths", "0.1.1"), names)
        self.assertIn(("libm", "0.2.16"), names)
        path = self.root / "Cargo.lock"; path.chmod(0o600)
        import tomllib
        row = next(p for p in tomllib.loads(path.read_text())["package"] if p["name"] == "core_maths")
        original = path.read_text()
        path.write_text(original.replace(row["checksum"], "0" * 64))
        self.assertNotEqual(MODULE.locked_packages(self.root), packages)
        path.write_text(original.replace('name = "core_maths"', 'name = "unselected_core_maths"'))
        with self.assertRaises(ValueError): MODULE.locked_packages(self.root)
    def test_consumed_shared_and_host_sources_exclude_tests_and_self_identity(self):
        paths = {p.as_posix() for p in MODULE.kernel_sources(ROOT)}
        for owner in ["lib.rs", "provider.rs", "protocol.rs", "supported_values.rs"]:
            self.assertIn("crates/lila-intl/src/" + owner, paths)
        self.assertIn("crates/lila-engine/src/intl_segmenter_host.rs", paths)
        for owner in [
            "crates/lila-engine/src/wasm_gc_intl_host.rs",
            "crates/lila-engine/src/intl_data_images.rs",
            "crates/lila-intl/src/segmenter_image.rs",
            "crates/lila-intl/src/image_build/segmenter.rs",
            "crates/lila-intl/src/locale_image.rs",
            "crates/lila-intl/build.rs",
        ]:
            self.assertIn(owner, paths)
        self.assertFalse(any(p.endswith("/kernel_identity.rs") or p.endswith("/tests.rs") or "/tests/" in p for p in paths))

if __name__ == "__main__":
    unittest.main()
