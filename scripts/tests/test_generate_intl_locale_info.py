"""Source-identity and data-shape failure controls for Locale information generation."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "generate-intl-locale-info.py"
SPEC = importlib.util.spec_from_file_location("intl_locale_info", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
DATA = SCRIPT.parents[1] / "crates/lila-intl/data"
SOURCE = DATA / "locale-info-cldr-47"


class IntlLocaleInfoGenerationTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        self.source = root / "locale-info-cldr-47"
        shutil.copytree(SOURCE, self.source)
        manifest = json.loads((SOURCE / "manifest.json").read_text())
        for entry in manifest["shared_inputs"]:
            relative = Path(entry["path"]).relative_to("..")
            (root / relative).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(DATA / relative, root / relative)

    def rewrite(self, relative, old, new, refresh_checksum=True):
        path = (self.source / relative).resolve()
        original = path.read_text()
        self.assertIn(old, original)
        path.write_text(original.replace(old, new, 1))
        if not refresh_checksum:
            return
        manifest_path = self.source / "manifest.json"
        manifest = json.loads(manifest_path.read_text())
        payload = path.read_bytes()
        for entry in manifest["files"] + manifest["shared_inputs"]:
            if entry["path"] == relative:
                entry["sha256"] = hashlib.sha256(payload).hexdigest()
                entry["bytes"] = len(payload)
                if "git_blob_sha1" in entry:
                    entry["git_blob_sha1"] = hashlib.sha1(
                        b"blob %d\0" % len(payload) + payload).hexdigest()
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

    def test_complete_pinned_directory_regenerates_identically(self):
        generated, report = MODULE.generate(self.source)
        self.assertEqual((generated, report), MODULE.generate(self.source))
        checked = SCRIPT.parents[1] / "crates/lila-intl/src/provider/locale_info/generated.rs"
        self.assertEqual(generated, checked.read_text())
        self.assertEqual(report, (SOURCE / "generated-report.json").read_text())
        parsed = json.loads(report)
        self.assertEqual(parsed["collation_files"], 133)
        self.assertEqual(parsed["excluded_posix_locales"], ["en_US_POSIX"])
        self.assertNotIn("standard", parsed["available_collations"])
        self.assertNotIn("search", parsed["available_collations"])
        self.assertIn('("fr-CA", &[DateTimeHourCycle::H23, DateTimeHourCycle::H12])', generated)
        self.assertIn('("TH", &["buddhist", "gregory"])', generated)
        self.assertIn('("de", 3)', generated)

    def test_modified_upstream_bytes_require_a_new_source_identity(self):
        self.rewrite("common/collation/de.xml", 'type="phonebook"', 'type="phonebk"',
                     refresh_checksum=False)
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            MODULE.generate(self.source)

    def test_unlisted_collation_file_cannot_escape_generation(self):
        shutil.copyfile(self.source / "common/collation/de.xml",
                        self.source / "common/collation/xx.xml")
        with self.assertRaisesRegex(ValueError, "collation directory and pinned manifest disagree"):
            MODULE.generate(self.source)

    def test_language_keyed_calendar_preferences_are_rejected(self):
        self.rewrite("../datetime-cldr-47/common/supplemental/supplementalData.xml",
                     '<calendarPreference territories="JP"', '<calendarPreference territories="ja_JP"')
        with self.assertRaisesRegex(ValueError, "calendar preference key is not a region"):
            MODULE.generate(self.source)

    def test_unknown_hour_symbols_are_rejected(self):
        self.rewrite("../datetime-cldr-47/common/supplemental/supplementalData.xml",
                     'preferred="H" allowed="H K h" regions="JP"',
                     'preferred="H" allowed="H J h" regions="JP"')
        with self.assertRaisesRegex(ValueError, "unknown time-data hour symbol"):
            MODULE.generate(self.source)

    def test_root_collations_must_match_the_ecma402_fallback(self):
        self.rewrite("common/collation/root.xml", "<collation type='emoji'>",
                     "<collation type='phonebook'>")
        with self.assertRaisesRegex(ValueError, "disagree with ECMA-402's fallback list"):
            MODULE.generate(self.source)

    def test_zone_tab_rows_must_be_primary_identifiers(self):
        self.rewrite("../iana-tzdb-2026a/source/zone.tab", "Europe/Andorra", "Asia/Calcutta")
        with self.assertRaisesRegex(ValueError, "not an ECMA-402 primary"):
            MODULE.generate(self.source)

    def test_changed_data_changes_the_provider_identity(self):
        _, previous = MODULE.generate(self.source)
        self.rewrite("../datetime-cldr-47/common/supplemental/supplementalData.xml",
                     '<firstDay day="fri" territories="MV"/>', '<firstDay day="thu" territories="MV"/>')
        _, current = MODULE.generate(self.source)
        self.assertNotEqual(json.loads(previous)["provider_data_sha256"],
                            json.loads(current)["provider_data_sha256"])


if __name__ == "__main__":
    unittest.main()
