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
        self.assertEqual([row["locale"] for row in self.profile["locales"]], ["en", "en-US", "ar", "ar-EG", "zh", "zh-Hans", "zh-Hans-CN", "de", "de-DE", "ja"])
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
                if calendar["calendar"] not in ("chinese", "dangi"):
                    self.assertIsNotNone(calendar["append_era"])
                    self.assertEqual(sorted(token["placeholder"] for token in calendar["append_era"]["tokens"] if "placeholder" in token), [0, 1])
                else:
                    self.assertIsNone(calendar["append_era"])
        consumed = json.loads(self.report)["consumed_leaves"]
        self.assertTrue(any("alt='ascii'" in row["path"] for row in consumed.values()))

    def test_range_context_uses_checked_canonical_patterns_with_identical_fields(self):
        encoded, _ = GENERATOR.generate(SOURCES, range_context=True)
        overlay = GENERATOR.range_pattern_overrides(self.encoded, encoded)
        expected = SCRIPTS.parent / "crates/lila-intl/src/provider/datetime/generated/range-patterns.json"
        self.assertEqual(expected.read_text(), overlay)
        GENERATOR.assert_matching_range_structure(self.encoded, encoded)
        ranged = json.loads(encoded)
        sparse = json.loads(overlay)
        self.assertEqual(self.profile["pattern_context"], "scalar_ascii")
        self.assertEqual(sparse["pattern_context"], "range_default")
        self.assertEqual({row["locale"] for row in sparse["overrides"]}, {"en", "en-US"})
        for scalar_locale, range_locale in zip(self.profile["locales"], ranged["locales"]):
            for key, scalar_calendar in scalar_locale["calendars"].items():
                range_calendar = range_locale["calendars"][key]
                self.assertEqual(scalar_calendar["intervals"], range_calendar["intervals"])
                self.assertEqual(scalar_calendar["interval_fallback"], range_calendar["interval_fallback"])
                if scalar_locale["locale"] in ("de-DE", "ja"):
                    self.assertEqual(scalar_calendar["available"], range_calendar["available"])
        scalar_en = self.profile["locales"][1]["calendars"]["gregorian"]
        range_en = ranged["locales"][1]["calendars"]["gregorian"]
        self.assertEqual(next(row["source"] for row in scalar_en["available"] if row["skeleton"] == "hms"), "h:mm:ss a")
        self.assertEqual(next(row["source"] for row in range_en["available"] if row["skeleton"] == "hms"), "h:mm:ss\u202fa")
        self.assertFalse(any(row["skeleton"] == "hms" for row in range_en["intervals"]))

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
            for calendar_name in ("chinese", "dangi"):
                calendar = locale["calendars"][calendar_name]
                self.assertFalse(any("U" in row["skeleton"] for row in calendar["available"]))
                self.assertTrue(any("U" in row["skeleton"] for row in calendar["excluded_non_ecma_formats"]))
                self.assertTrue(any(token.get("field") == "U" for row in calendar["available"] for token in row["tokens"]))

    def test_selected_era_calendars_resolve_their_own_cldr_names(self):
        self.assertEqual(self.profile["selector"]["calendars"],
                         ["gregory", "iso8601", "chinese", "buddhist", "indian", "persian", "roc", "dangi", "islamic-civil",
                          "coptic", "ethioaa", "ethiopic", "hebrew", "islamic-tbla", "islamic-umalqura", "japanese"])
        eras = {"buddhist": {0}, "indian": {0}, "persian": {0}, "roc": {0, 1}, "islamic-civil": {0, 1}}
        for locale in self.profile["locales"]:
            for calendar, expected in eras.items():
                names = locale["calendars"][calendar]["names"]
                self.assertEqual({name["index"] for name in names if name["kind"] == "era"}, expected)
        english = self.profile["locales"][0]["calendars"]
        wide_era = lambda calendar, index: next(
            name["value"] for name in english[calendar]["names"]
            if name["kind"] == "era" and name["width"] == "wide" and name["index"] == index)
        self.assertEqual(wide_era("buddhist", 0), "BE")
        self.assertEqual(wide_era("roc", 1), "Minguo")
        self.assertEqual(wide_era("persian", 0), "AP")
        self.assertEqual(wide_era("indian", 0), "Saka")
        for locale in self.profile["locales"]:
            names = locale["calendars"]["islamic-civil"]["names"]
            self.assertEqual({(row["width"], row["value"], row.get("source")) for row in names
                              if row["kind"] == "era" and row["index"] == 1},
                             {(width, "BH", "icu_calendar:era:bh") for width in ("wide", "abbreviated", "narrow")})
        japanese, = [row for row in self.profile["locales"] if row["locale"] == "ja"]
        self.assertEqual((japanese["preferred_hour"], japanese["allowed_hours"]),
                         ("H", ["H", "K", "h"]))

    def test_calendar_preferences_use_pinned_bcp_aliases(self):
        for locale in self.profile["locales"]:
            self.assertEqual(locale["calendar_preferences"][0], "gregory")
            self.assertNotIn("gregorian", locale["calendar_preferences"])
            self.assertNotIn("islamicc", locale["calendar_preferences"])

    def test_finite_day_rules_preserve_the_pinned_algorithmic_calendar_override(self):
        row, = [row for row in self.profile["algorithmic_fields"] if row["identifier"] == "hanidays"]
        self.assertEqual((row["identifier"], row["field"], row["minimum"]), ("hanidays", "d", 1))
        self.assertEqual(row["values"], ["初一", "初二", "初三", "初四", "初五", "初六", "初七", "初八", "初九", "初十", "十一", "十二", "十三", "十四", "十五", "十六", "十七", "十八", "十九", "二十", "廿一", "廿二", "廿三", "廿四", "廿五", "廿六", "廿七", "廿八", "廿九", "三十", "丗一"])
        self.assertTrue(row["consumed_rules"])
        for locale in self.profile["locales"]:
            if locale["locale"].startswith("zh"):
                full = locale["calendars"]["chinese"]["styles"]["full"]["date"]
                self.assertEqual(full["source"], "rU年MMMdEEEE")
                self.assertEqual(full["numbering_overrides"], [{"field": "d", "numbering": "hanidays"}])

    def test_japanese_year_override_is_bound_to_checked_rbnf_rules(self):
        row, = [row for row in self.profile["algorithmic_fields"] if row["identifier"] == "jpanyear"]
        self.assertEqual((row["field"], row["method"], row["values"]),
                         ("y", "one_replaced_latin", ["元"]))
        self.assertEqual(row["source"], "common/rbnf/ja.xml")
        self.assertEqual(row["ruleset"], "spellout-numbering-year-latn")
        japanese, = [row for row in self.profile["locales"] if row["locale"] == "ja"]
        self.assertIn({"field": "y", "numbering": "jpanyear"},
                      japanese["calendars"]["japanese"]["styles"]["full"]["date"]["numbering_overrides"])

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
