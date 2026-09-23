import importlib.util
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
SPEC = importlib.util.spec_from_file_location("generate_intl_datetime_profile", SCRIPTS / "generate-intl-datetime-profile.py")
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)
SOURCES = SCRIPTS.parent / GENERATOR.SOURCE_PATH


class DateTimeProfileGenerationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.encoded, cls.report = GENERATOR.generate(SOURCES)
        cls.profile = json.loads(cls.encoded)

    def test_pinned_profile_regenerates_all_selected_locales_and_numeric_systems(self):
        expected = SCRIPTS.parent / "crates/lila-intl/src/provider/datetime/generated/profile.json"
        self.assertEqual(expected.read_text(), self.encoded)
        self.assertEqual([row["locale"] for row in self.profile["locales"]], ["en", "en-US", "ar", "ar-EG", "zh", "zh-Hans", "zh-Hans-CN"])
        digits = {row["identifier"]: row["digits"] for row in self.profile["numbering_systems"]}
        self.assertEqual(len(digits), 77)
        self.assertEqual(digits["arab"], "٠١٢٣٤٥٦٧٨٩")
        self.assertEqual(digits["hanidec"], "〇一二三四五六七八九")
        self.assertEqual(digits["mathbold"], "𝟎𝟏𝟐𝟑𝟒𝟓𝟔𝟕𝟖𝟗")
        self.assertNotIn("roman", digits)

    def test_ascii_policy_selects_supplied_patterns_and_preserves_unprovided_intervals(self):
        for locale in self.profile["locales"]:
            for calendar in locale["calendars"].values():
                if locale["locale"] in ("en", "en-US"):
                    self.assertEqual(calendar["styles"]["medium"]["time"]["source"], "h:mm:ss a")
                    formats = {row["skeleton"]: row for row in calendar["available"]}
                    self.assertEqual(formats["hms"]["source"], "h:mm:ss a")
                    self.assertTrue(any("\u202f" in row["source"] for row in calendar["intervals"]))
                if calendar["calendar"] == "gregorian":
                    self.assertIsNotNone(calendar["append_era"])
                    self.assertEqual(sorted(token["placeholder"] for token in calendar["append_era"]["tokens"] if "placeholder" in token), [0, 1])
                else:
                    self.assertIsNone(calendar["append_era"])
        consumed = json.loads(self.report)["consumed_leaves"]
        self.assertTrue(any("alt='ascii'" in row["path"] for row in consumed.values()))

    def test_numeric_year_requests_retain_cyclic_output_without_admitting_name_only_skeletons(self):
        from intl_datetime_patterns import compile_pattern, skeleton_in_profile
        for skeleton in ["U", "UM", "UMd", "UMMMd"]:
            self.assertFalse(skeleton_in_profile(skeleton))
        for skeleton in ["y", "yyyyMd", "rMd"]:
            self.assertTrue(skeleton_in_profile(skeleton))
        self.assertEqual(compile_pattern("r(U)"), [
            {"field": "r", "width": 1}, {"literal": "("},
            {"field": "U", "width": 1}, {"literal": ")"},
        ])
        for locale in self.profile["locales"]:
            calendar = locale["calendars"]["chinese"]
            self.assertFalse(any("U" in row["skeleton"] for row in calendar["available"]))
            self.assertTrue(any("U" in row["skeleton"] for row in calendar["excluded_non_ecma_formats"]))
            self.assertTrue(any(token.get("field") == "U" for row in calendar["available"] for token in row["tokens"]))

    def test_calendar_preferences_use_pinned_bcp_aliases(self):
        for locale in self.profile["locales"]:
            self.assertEqual(locale["calendar_preferences"][0], "gregory")
            self.assertNotIn("gregorian", locale["calendar_preferences"])
            self.assertNotIn("islamicc", locale["calendar_preferences"])

    def test_finite_day_rules_preserve_the_pinned_algorithmic_calendar_override(self):
        row, = self.profile["algorithmic_fields"]
        self.assertEqual((row["identifier"], row["field"], row["minimum"]), ("hanidays", "d", 1))
        self.assertEqual(row["values"], ["初一", "初二", "初三", "初四", "初五", "初六", "初七", "初八", "初九", "初十", "十一", "十二", "十三", "十四", "十五", "十六", "十七", "十八", "十九", "二十", "廿一", "廿二", "廿三", "廿四", "廿五", "廿六", "廿七", "廿八", "廿九", "三十", "丗一"])
        self.assertTrue(row["consumed_rules"])
        for locale in self.profile["locales"]:
            if locale["locale"].startswith("zh"):
                full = locale["calendars"]["chinese"]["styles"]["full"]["date"]
                self.assertEqual(full["source"], "rU年MMMdEEEE")
                self.assertEqual(full["numbering_overrides"], [{"field": "d", "numbering": "hanidays"}])

    def test_distinct_unicode_digits_need_no_uniform_utf8_width_assumption(self):
        profile = GENERATOR.CldrProfile(SOURCES)
        document = profile.documents["common/supplemental/numberingSystems.xml"]
        node = document.find("./numberingSystems/numberingSystem[@id='latn']")
        node.set("digits", "０123456789")
        digits = {row["identifier"]: row["digits"] for row in GENERATOR.positional_numbering_systems(profile)}
        self.assertEqual(digits["latn"], "０123456789")
        for invalid in ["0012345678", "123456789", "0123456789０"]:
            node.set("digits", invalid)
            with self.subTest(digits=invalid), self.assertRaisesRegex(ValueError, "invalid positional"):
                GENERATOR.positional_numbering_systems(profile)

    def test_missing_required_decimal_symbol_is_not_replaced_with_another_numbering_system(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "profile"
            shutil.copytree(SOURCES, source)
            relative = "common/main/root.xml"
            path = source / relative
            document = ET.parse(path)
            symbols = document.getroot().find("./numbers/symbols[@numberSystem='arab']")
            decimal = symbols.find("decimal")
            self.assertIsNotNone(decimal)
            symbols.remove(decimal)
            document.write(path, encoding="utf-8", xml_declaration=True)
            manifest_path = source / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            for record in manifest["files"]:
                if record["path"] == relative:
                    contents = path.read_bytes()
                    record["bytes"] = len(contents)
                    record["sha256"] = hashlib.sha256(contents).hexdigest()
            manifest["total_bytes"] = sum(record["bytes"] for record in manifest["files"])
            manifest_path.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, "unresolved required LDML leaf: .*decimal"):
                GENERATOR.generate(source)

    def test_changed_source_is_rejected_before_inheritance(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "profile"
            shutil.copytree(SOURCES, source)
            path = source / "common/main/ar.xml"
            path.write_text(path.read_text() + "\n")
            with self.assertRaisesRegex(ValueError, "checksum"):
                GENERATOR.generate(source)

    def test_numbering_assignments_are_not_dropped_or_treated_as_distinguishing_attributes(self):
        profile = GENERATOR.CldrProfile(SOURCES)
        leaf = profile.resolve("zh", "dates/calendars/calendar[@type='chinese']/dateFormats/dateFormatLength[@type='full']/dateFormat/pattern")
        self.assertEqual(GENERATOR.pattern_numbering(leaf), [{"field": "d", "numbering": "hanidays"}])
        with self.assertRaisesRegex(ValueError, "unconsumed"):
            GENERATOR.plain_leaf(leaf)


if __name__ == "__main__":
    unittest.main()
