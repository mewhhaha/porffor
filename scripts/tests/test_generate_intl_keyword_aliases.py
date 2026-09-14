"""Source-identity and data-shape failure controls for CLDR alias generation."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "generate-intl-keyword-aliases.py"
SPEC = importlib.util.spec_from_file_location("intl_keyword_aliases", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
SOURCE = SCRIPT.parents[1] / "crates/lila-intl/data/cldr-47-bcp47"


class IntlKeywordAliasGenerationTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.source = Path(temporary.name) / "bcp47"
        shutil.copytree(SOURCE, self.source)

    def change_calendar(self, old, new, refresh_checksum=True):
        path = self.source / "common/bcp47/calendar.xml"
        original = path.read_text()
        self.assertIn(old, original)
        path.write_text(original.replace(old, new))
        if refresh_checksum:
            manifest_path = self.source / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            for entry in manifest["files"]:
                if entry["path"] == "common/bcp47/calendar.xml":
                    entry["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
                    entry["bytes"] = path.stat().st_size
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

    def test_complete_pinned_directory_regenerates_identically(self):
        generated, report = MODULE.generate(self.source)
        self.assertEqual((generated, report), MODULE.generate(self.source))
        checked = SCRIPT.parents[1] / "crates/lila-intl/src/provider/keyword_aliases/generated.rs"
        self.assertEqual(generated, checked.read_text())
        parsed = json.loads(report)
        self.assertEqual(parsed["xml_files"], 15)
        self.assertEqual(parsed["unicode_aliases"], 60)
        self.assertEqual(parsed["transform_aliases"], 5)
        self.assertIn('(\"islamicc\", \"islamic-civil\")', generated)
        self.assertIn('(\"ethiopic-amete-alem\", \"ethioaa\")', generated)

    def test_modified_upstream_bytes_require_a_new_source_identity(self):
        self.change_calendar('alias="calendar"', 'alias="changed"', refresh_checksum=False)
        with self.assertRaisesRegex(ValueError, "source checksum mismatch"):
            MODULE.generate(self.source)

    def test_unlisted_xml_cannot_silently_escape_generation(self):
        (self.source / "common/bcp47/new.xml").write_text("<ldmlBCP47/>")
        with self.assertRaisesRegex(ValueError, "directory and pinned source manifest disagree"):
            MODULE.generate(self.source)

    def test_cyclic_preferred_types_are_rejected(self):
        self.change_calendar('name="islamic-civil" description=',
                             'name="islamic-civil" preferred="islamicc" description=')
        with self.assertRaisesRegex(ValueError, "cyclic preferred alias"):
            MODULE.generate(self.source)

    def test_conflicting_alias_targets_are_rejected(self):
        self.change_calendar('alias="gregorian"', 'alias="gregorian islamicc"')
        with self.assertRaisesRegex(ValueError, "conflicting alias"):
            MODULE.generate(self.source)

    def test_new_key_aliases_require_their_own_semantics(self):
        self.change_calendar('alias="calendar"', 'alias="xx"')
        with self.assertRaisesRegex(ValueError, "key aliases require key replacement"):
            MODULE.generate(self.source)

    def test_changed_canonical_data_changes_the_composite_provider_identity(self):
        _, previous = MODULE.generate(self.source)
        self.change_calendar('alias="gregorian"', 'alias="gregorian customxy"')
        generated, current = MODULE.generate(self.source)
        self.assertIn('(\"customxy\", \"gregory\")', generated)
        self.assertNotEqual(json.loads(previous)["provider_data_sha256"],
                            json.loads(current)["provider_data_sha256"])


if __name__ == "__main__":
    unittest.main()
