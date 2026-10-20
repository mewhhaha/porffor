import importlib.util
import json
from pathlib import Path
import sys
import subprocess
import tempfile
import unittest
from unittest.mock import patch


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
spec = importlib.util.spec_from_file_location("prepare_test262_errata", SCRIPTS / "prepare_test262_errata.py")
errata = importlib.util.module_from_spec(spec)
spec.loader.exec_module(errata)


class Test262ErrataTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.suite = self.root / "stock"
        (self.suite / "test").mkdir(parents=True)
        (self.suite / "test/corrected.js").write_bytes(b"old assertion;\n")
        (self.suite / "test/control.js").write_bytes(b"unchanged control;\n")
        self.errata = self.root / "errata"
        self.errata.mkdir()
        (self.errata / "replacement.js").write_bytes(b"correct assertion;\n")
        self.pins = self.root / "pins/source-pins.json"
        self.pins.parent.mkdir()
        pins = {"ecma262": "a" * 40, "test262_upstream": "b" * 40,
                "test262_tree": errata.git_tree_id(self.suite)[0].hex()}
        self.pins.write_text(json.dumps(pins))
        self.manifest = {
            "schema": "scriptw-test262-errata-v1", "base_source_pins": pins,
            "corrections": [{"path": "test/corrected.js", "replacement": "replacement.js",
                             "original_sha256": errata.sha(self.suite / "test/corrected.js"),
                             "replacement_sha256": errata.sha(self.errata / "replacement.js")}],
        }
        self.output = self.root / "amended"
        self.save_manifest()

    def save_manifest(self):
        (self.errata / "manifest.json").write_text(json.dumps(self.manifest))

    def prepare(self, output=True):
        return errata.prepare(self.suite, self.errata, self.pins, self.output if output else None)

    def test_copy_preserves_stock_and_controls_and_cannot_be_stock_publication(self):
        receipt = self.prepare()
        amended = self.output / "suite"
        self.assertEqual((self.suite / "test/corrected.js").read_bytes(), b"old assertion;\n")
        self.assertEqual((amended / "test/corrected.js").read_bytes(), b"correct assertion;\n")
        self.assertEqual((amended / "test/control.js").read_bytes(), b"unchanged control;\n")
        self.assertEqual(sorted(p.name for p in (amended / "test").iterdir()),
                         sorted(p.name for p in (self.suite / "test").iterdir()))
        self.assertTrue(receipt["complete"])
        self.assertFalse(receipt["upstream_unmodified"])
        self.assertFalse(receipt["eligible_for_stock_test262_publication"])
        self.assertEqual(receipt["amended_tree"], errata.git_tree_id(amended)[0].hex())
        self.assertTrue(errata.verify(self.pins, self.suite)["vendored_tree_verified"])
        with self.assertRaisesRegex(ValueError, "differs from declared"):
            errata.verify(self.pins, amended)
        (amended / "test/control.js").write_bytes(b"changed copy;\n")
        self.assertEqual((self.suite / "test/control.js").read_bytes(), b"unchanged control;\n")

    def test_check_only_does_not_create_output(self):
        self.assertTrue(self.prepare(output=False)["check_only"])
        self.assertFalse(self.output.exists())

    def test_changed_base_pins_or_suite_or_replacement_fail_before_copy(self):
        for kind in ("pins", "suite", "replacement"):
            with self.subTest(kind=kind):
                path = {"pins": self.pins, "suite": self.suite / "test/control.js",
                        "replacement": self.errata / "replacement.js"}[kind]
                original = path.read_bytes()
                try:
                    path.write_bytes(b"{}" if kind == "pins" else b"changed;\n")
                    with self.assertRaises(ValueError):
                        self.prepare()
                    self.assertFalse(self.output.exists())
                finally:
                    path.write_bytes(original)

    def test_output_must_be_new_and_separate(self):
        for output in (self.suite, self.suite / "inside", self.errata / "inside", self.root):
            with self.subTest(output=output), self.assertRaises(ValueError):
                errata.prepare(self.suite, self.errata, self.pins, output)
        self.output.mkdir()
        sentinel = self.output / "keep"
        sentinel.write_text("keep")
        with self.assertRaises(FileExistsError):
            self.prepare()
        self.assertEqual(sentinel.read_text(), "keep")

    def test_paths_cannot_escape_or_duplicate_tests(self):
        entry = self.manifest["corrections"][0]
        for name in ("../outside.js", "/outside.js", "test/../outside.js", "test\\..\\outside.js"):
            with self.subTest(name=name):
                for field in ("path", "replacement"):
                    original = entry[field]
                    entry[field] = name
                    self.save_manifest()
                    with self.assertRaises(ValueError):
                        self.prepare()
                    entry[field] = original
        self.manifest["corrections"].append(dict(entry))
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_failed_copy_never_gets_a_complete_receipt(self):
        with patch.object(errata.shutil, "copytree", side_effect=OSError("copy failed")):
            with self.assertRaisesRegex(OSError, "copy failed"):
                self.prepare()
        receipt = json.loads((self.output / "errata-receipt.json").read_text())
        self.assertFalse(receipt["complete"])
        self.assertNotIn("amended_tree", receipt)

    def test_copied_windows_modes_use_source_index_even_outside_git(self):
        subprocess.run(["git", "init", "--quiet", str(self.root)], check=True)
        subprocess.run(["git", "-C", str(self.root), "add", "stock"], check=True)
        subprocess.run(["git", "-C", str(self.root), "update-index", "--chmod=+x",
                        "stock/test/control.js"], check=True)
        modes = errata.git_index_modes(self.suite)
        pins = json.loads(self.pins.read_text())
        pins["test262_tree"] = errata.git_tree_id(self.suite, modes)[0].hex()
        self.pins.write_text(json.dumps(pins))
        self.manifest["base_source_pins"] = pins
        self.save_manifest()
        # There is no Git index at the output path, including when the copy
        # lives outside the source repository. Remap the verified source modes.
        with tempfile.TemporaryDirectory() as outside, patch.object(sys, "platform", "win32"):
            output = Path(outside) / "amended"
            receipt = errata.prepare(self.suite, self.errata, self.pins, output)
            copied_modes = {output / "suite" / path.relative_to(self.suite): mode
                            for path, mode in modes.items()}
            self.assertTrue(receipt["complete"])
            self.assertEqual(receipt["amended_tree"],
                             errata.git_tree_id(output / "suite", copied_modes)[0].hex())


if __name__ == "__main__":
    unittest.main()
