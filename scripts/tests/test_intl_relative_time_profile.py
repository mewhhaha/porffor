import importlib.util
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from intl_cldr_profile import CldrProfile, LocaleTree, ResolvedLeaf, parse_path

spec = importlib.util.spec_from_file_location("relative_time_producer", SCRIPTS / "generate-intl-relative-time-profile.py")
producer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(producer)
SOURCE = SCRIPTS.parent / producer.SOURCE_PATH
DAY_ONE = "dates/fields/field[@type='day']/relativeTime[@type='future']/relativeTimePattern[@count='one']"


class RelativeTimeDataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.profile = producer.load_profile(SCRIPTS.parent)
        cls.generated, cls.report = producer.generate(cls.profile)
        cls.rows = {locale["locale"]: {(field["unit"], field["style"]): field
                    for field in locale["fields"]} for locale in cls.generated["locales"]}

    def fixture(self, child, parent):
        profile = CldrProfile.__new__(CldrProfile)
        profile.schema = self.profile.schema
        profile.parents = {"en_US": "en", "en": "root"}
        profile.locales = {
            locale: LocaleTree(locale, ET.fromstring(f"<ldml><dates><fields>{text}</fields></dates></ldml>"), 2, profile.schema)
            for locale, text in [("en_US", child), ("en", parent), ("root", "")]
        }
        return profile

    def test_complete_captured_domain_and_category_order(self):
        self.assertEqual([row["locale"] for row in self.generated["locales"]], list(producer.LOCALES))
        self.assertEqual(self.generated["cldr_commit"], "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c")
        for locale in self.generated["locales"]:
            self.assertEqual([(row["unit"], row["style"]) for row in locale["fields"]],
                             [(unit, style) for unit in producer.UNITS for style in producer.STYLES])
            for row in locale["fields"]:
                self.assertEqual(len(row["past"]), 6)
                self.assertEqual(len(row["future"]), 6)
                offsets = [entry["offset"] for entry in row["relative"]]
                self.assertEqual(offsets, sorted(set(offsets)))
                self.assertTrue(set(offsets) <= set(producer.OFFSETS))
        self.assertEqual(self.report["unit_style_rows"], 336)
        self.assertEqual(self.report["numeric_patterns"], 4032)

    def test_independent_english_arabic_and_relative_vectors(self):
        english = self.rows["en-US"][("day", "long")]
        self.assertEqual(english["future"][1], "in {0} day")
        self.assertEqual(english["future"][5], "in {0} days")
        self.assertEqual(english["past"][1], "{0} day ago")
        self.assertEqual(dict((entry["offset"], entry["value"]) for entry in english["relative"]),
                         {-1: "yesterday", 0: "today", 1: "tomorrow"})
        arabic = self.rows["ar"][("day", "long")]
        self.assertEqual(arabic["future"][0], "خلال {0} يوم")
        self.assertEqual(arabic["future"][1:3], ["خلال يوم واحد", "خلال يومين"])
        self.assertEqual(arabic["past"][1:3], ["قبل يوم واحد", "قبل يومين"])
        self.assertEqual(self.rows["ar-EG"][("day", "long")], arabic)
        self.assertGreater(self.report["numeric_patterns_without_placeholder"], 0)

    def test_provenance_keeps_lateral_other_and_style_alias_sources(self):
        path = "ar/dates/fields/field[@type='day']/relativeTime[@type='future']/relativeTimePattern[@count='zero']"
        leaf = self.report["consumed_leaves"][path]
        self.assertEqual(leaf["requested_category"], "zero")
        self.assertEqual(leaf["source_category"], "other")
        self.assertEqual(leaf["source_locale"], "ar")
        inherited = self.report["consumed_leaves"][path.replace("ar/", "ar_EG/", 1)]
        self.assertEqual(inherited, leaf)
        self.assertTrue(any("-narrow" in requested and "-narrow" not in value["source_path"]
                            for requested, value in self.report["consumed_leaves"].items()))
        self.assertTrue(all(value["value"] not in ("↑↑↑", "∅∅∅")
                            for value in self.report["consumed_leaves"].values()))

    def test_child_other_precedes_parent_exact_category(self):
        profile = self.fixture(
            '<field type="day"><relativeTime type="future"><relativeTimePattern count="other">child {0}</relativeTimePattern></relativeTime></field>',
            '<field type="day"><relativeTime type="future"><relativeTimePattern count="one">parent {0}</relativeTimePattern></relativeTime></field>')
        leaf = producer.resolve_count(profile, "en_US", DAY_ONE)
        self.assertEqual((leaf.value, leaf.source_locale, leaf.source_path[-1].get("count")),
                         ("child {0}", "en_US", "other"))

    def test_exact_count_and_inheritance_marker_have_distinct_ownership(self):
        profile = self.fixture(
            '<field type="day"><relativeTime type="future"><relativeTimePattern count="one">↑↑↑</relativeTimePattern><relativeTimePattern count="other">child {0}</relativeTimePattern></relativeTime></field>',
            '<field type="day"><relativeTime type="future"><relativeTimePattern count="one">parent {0}</relativeTimePattern></relativeTime></field>')
        self.assertEqual(producer.resolve_count(profile, "en_US", DAY_ONE).value, "child {0}")
        profile = self.fixture(
            '<field type="day"><relativeTime type="future"><relativeTimePattern count="one">exact {0}</relativeTimePattern><relativeTimePattern count="other">other {0}</relativeTimePattern></relativeTime></field>', "")
        self.assertEqual(producer.resolve_count(profile, "en_US", DAY_ONE).value, "exact {0}")

    def test_root_style_alias_restarts_requested_locale(self):
        profile = self.fixture(
            '<field type="day"><relativeTime type="future"><relativeTimePattern count="one">child {0}</relativeTimePattern></relativeTime></field>', "")
        profile.locales["root"] = LocaleTree("root", ET.fromstring('''<ldml><dates><fields>
            <field type="day-short"><alias source="locale" path="../field[@type='day']"/></field>
            <field type="day-narrow"><alias source="locale" path="../field[@type='day-short']"/></field>
            </fields></dates></ldml>'''), 2, profile.schema)
        leaf = producer.resolve_count(profile, "en_US", DAY_ONE.replace("type='day'", "type='day-narrow'"))
        self.assertEqual((leaf.value, leaf.source_locale), ("child {0}", "en_US"))

    def test_cyclic_alias_and_empty_override_never_become_patterns(self):
        profile = self.fixture('''<field type="day"><alias source="locale" path="../field[@type='day-short']"/></field>
            <field type="day-short"><alias source="locale" path="../field[@type='day']"/></field>''', "")
        with self.assertRaisesRegex(ValueError, "cyclic"):
            producer.resolve_count(profile, "en_US", DAY_ONE)
        profile = self.fixture(
            '<field type="day"><relativeTime type="future"><relativeTimePattern count="one">∅∅∅</relativeTimePattern></relativeTime></field>',
            '<field type="day"><relativeTime type="future"><relativeTimePattern count="one">parent {0}</relativeTimePattern></relativeTime></field>')
        self.assertIsNone(producer.resolve_count(profile, "en_US", DAY_ONE))

    def test_placeholder_domain_preserves_literal_text(self):
        for value in ["خلال يومين", " {0}\u00a0jours ", "{0}日後"]:
            self.assertEqual(producer.checked_text(value, numeric=True), value)
        for value in ["", "↑↑↑", "∅∅∅", "{1}", "{0}{0}", "{{0}}", "}", "{0:x}"]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                producer.checked_text(value, numeric=True)
        with self.assertRaises(ValueError):
            producer.checked_text("today {0}", numeric=False)

    def test_unconsumed_value_attributes_are_rejected(self):
        path = parse_path(DAY_ONE)
        leaf = ResolvedLeaf("{0} days", (("unhandled", "value"),), "en", path)
        with self.assertRaisesRegex(ValueError, "unconsumed"):
            producer.consume({}, "en", path, leaf, numeric=True)

    def test_wrong_locale_recipe_and_wrong_count_path_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "six relative-time"):
            producer.resolve_count(self.profile, "en", DAY_ONE.replace("count='one'", "count='invalid'"))
        old = self.profile.selector["locales"]
        try:
            self.profile.selector["locales"] = old[:-1]
            with self.assertRaisesRegex(ValueError, "thirteen"):
                producer.generate(self.profile)
        finally:
            self.profile.selector["locales"] = old

    def test_real_pinned_source_tamper_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            copied = Path(temporary) / "datetime-cldr-47"
            shutil.copytree(SOURCE, copied)
            leaf = copied / "common/main/en.xml"
            leaf.chmod(0o644)
            leaf.write_bytes(leaf.read_bytes().replace(b"in {0} day", b"wrong {0} day", 1))
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                CldrProfile(copied)

    def test_genuine_polish_plural_auto_and_style_sources(self):
        polish = self.rows["pl"][("day", "long")]
        self.assertEqual(polish["future"][1], "za {0} dzień")
        self.assertEqual(polish["future"][3:6], ["za {0} dni", "za {0} dni", "za {0} dnia"])
        self.assertEqual(polish["past"][1], "{0} dzień temu")
        self.assertEqual({row["offset"]: row["value"] for row in polish["relative"]},
                         {-2: "przedwczoraj", -1: "wczoraj", 0: "dzisiaj", 1: "jutro", 2: "pojutrze"})
        leaf = self.report["consumed_leaves"]["pl/" + DAY_ONE]
        self.assertEqual((leaf["source_locale"], leaf["requested_category"], leaf["source_category"]),
                         ("pl", "one", "one"))
        self.assertEqual(tuple(self.profile.lineage("pl_PL")), ("pl_PL", "pl", "root"))

    def test_polish_primary_capture_tamper_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary)
            for name in ["crates/lila-intl/data/relative-time-cldr-47", "crates/lila-intl/data/number-cldr-47"]:
                shutil.copytree(SCRIPTS.parent / name, repository / name)
            leaf = repository / "crates/lila-intl/data/relative-time-cldr-47/common/main/pl.xml"
            leaf.chmod(0o644)
            leaf.write_bytes(leaf.read_bytes().replace(b"za {0} ", b"bad {0} ", 1))
            with self.assertRaisesRegex(ValueError, "captured source checksum"):
                producer.load_polish_sources(repository)

    def test_source_recipe_binds_actual_producer_and_all_checked_dependencies(self):
        inputs = producer.source_inputs(SCRIPTS.parent, self.profile)
        names = {row["path"] for row in inputs["files"]}
        self.assertIn("scripts/generate-intl-relative-time-profile.py", names)
        self.assertIn("scripts/intl_cldr_profile.py", names)
        self.assertIn("crates/lila-intl/data/datetime-cldr-47/docs/ldml/tr35.md", names)
        self.assertIn("crates/lila-intl/data/numbering-tols-cldr-48/source-manifest.json", names)
        self.assertIn("crates/lila-intl/data/calendar-eras-cldr-48/manifest.json", names)
        self.assertIn(producer.CAPTURE_PATH, names)
        self.assertIn("scripts/intl_relative_time_sources.py", names)
        self.assertIn("crates/lila-intl/data/relative-time-cldr-47/common/main/pl.xml", names)
        self.assertIn("crates/lila-intl/data/number-cldr-47/sources.tar.gz", names)
        self.assertEqual(inputs["total_bytes"], sum(row["bytes"] for row in inputs["files"]))
        self.assertFalse(inputs["public_capability_admission"])


if __name__ == "__main__":
    unittest.main()
