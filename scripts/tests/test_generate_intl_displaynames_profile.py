import copy
import hashlib
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
SPEC = importlib.util.spec_from_file_location("displaynames_profile", SCRIPTS / "generate-intl-displaynames-profile.py")
GEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GEN)
from intl_cldr_profile import CldrProfile


class DisplayNamesSourceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.artifacts = GEN.generate(SCRIPTS.parent)
        cls.data = json.loads(cls.artifacts[0])
        cls.report = json.loads(cls.artifacts[1])
        cls.locales = GEN.expand_names(cls.data)

    def test_all_generated_artifacts_reproduce_from_genuine_sources(self):
        for relative, actual in zip((GEN.OUTPUT, GEN.REPORT, GEN.IDENTITY), self.artifacts):
            self.assertEqual((SCRIPTS.parent / relative).read_bytes(), actual)
        self.assertEqual(self.data["cldr_commit"], "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c")
        self.assertEqual([row["locale"] for row in self.data["locales"]], GEN.LOCALES)
        self.assertEqual(self.data["default_locale"], "en-US")
        self.assertEqual(len(self.data["name_pool"]), 105)
        self.assertEqual(self.report["name_map_associations"], 312)
        digest = hashlib.sha256(self.artifacts[0]).digest()
        for byte in digest:
            self.assertIn(f"0x{byte:02x}", self.artifacts[2].decode())

    def test_every_consumed_name_is_the_exact_pinned_primary_xml_text(self):
        directory = SCRIPTS.parent / GEN.SOURCE
        documents = {path.stem: ET.parse(path).getroot() for path in (directory / "common/main").glob("*.xml")}
        self.assertEqual(len(self.report["consumed_leaves"]), 19183)
        for requested, leaf in self.report["consumed_leaves"].items():
            node = documents[leaf["locale"]].find(leaf["path"])
            self.assertIsNotNone(node, requested)
            self.assertEqual(node.text, leaf["value"], requested)
            self.assertEqual(leaf["value_attributes"], {}, requested)
            self.assertNotIn(node.text, ("↑↑↑", "∅∅∅"))

    def test_all_six_domains_three_widths_and_twelve_fields_are_complete(self):
        for locale in self.locales:
            self.assertEqual(set(locale["styles"]), {"long", "short", "narrow"})
            for names in locale["styles"].values():
                self.assertEqual(set(names), {"language", "region", "script", "language_script", "currency", "calendar", "date_time_field", "variant"})
                for entries in names.values():
                    self.assertTrue(entries)
                    self.assertEqual([row[0] for row in entries], sorted({row[0] for row in entries}))
                self.assertEqual({row[0] for row in names["date_time_field"]}, set(GEN.FIELDS))

    def test_real_short_narrow_standalone_currency_and_dialect_source_vectors(self):
        rows = {row["locale"]: row for row in self.locales}
        def name(locale, style, kind, code):
            return dict(rows[locale]["styles"][style][kind])[code]
        for style, year in (("long", "year"), ("short", "yr."), ("narrow", "yr")):
            self.assertEqual(name("en", style, "date_time_field", "year"), year)
            self.assertEqual(name("en", style, "currency", "USD"), "US Dollar")
            self.assertEqual(name("en", style, "script", "Hans"), "Simplified Han")
            self.assertEqual(name("en", style, "language_script", "Hans"), "Simplified")
        self.assertEqual(name("en", "long", "language", "en-GB"), "British English")
        self.assertEqual(name("en", "short", "language", "en-GB"), "UK English")
        self.assertEqual(name("fr", "short", "region", "US"), "É.-U.")
        self.assertEqual(name("fr", "long", "currency", "USD"), "dollar des États-Unis")
        self.assertEqual(name("zh", "long", "currency", "USD"), "美元")
        self.assertEqual(name("ar", "long", "calendar", "gregory"), "التقويم الميلادي")

    def test_calendar_source_identifiers_use_the_actual_bcp47_authority(self):
        mapping = self.report["calendar_source_keys"]
        self.assertEqual(mapping["gregory"], "gregorian")
        self.assertEqual(mapping["ethioaa"], "ethiopic-amete-alem")
        self.assertEqual(mapping["islamicc"], "islamic-civil")
        self.assertEqual(mapping["ethiopic-amete-alem"], "ethiopic-amete-alem")
        self.assertIn("islamic", mapping)  # Display names are independent of DTF16 admission.
        for names in (row["styles"]["long"] for row in self.locales):
            fields = dict(names["calendar"])
            self.assertEqual(fields["ethioaa"], fields["ethiopic-amete-alem"])
            self.assertEqual(fields["islamicc"], fields["islamic-civil"])

    def test_pooling_is_byte_lossless_and_references_have_closed_domains(self):
        pool, locales = GEN.pool_names(self.locales)
        self.assertEqual(pool, self.data["name_pool"])
        self.assertEqual(locales, self.data["locales"])
        self.assertLess(len(self.artifacts[0]), len(GEN.serialized(self.locales)) // 2)
        def reject(change):
            data = copy.deepcopy(self.data)
            change(data)
            with self.assertRaises(ValueError): GEN.expand_names(data)
        reject(lambda d: d["locales"][0]["styles"]["long"].__setitem__("calendar", True))
        reject(lambda d: d["locales"][0]["styles"]["long"].__setitem__("calendar", len(d["name_pool"])))
        reject(lambda d: d["locales"][0]["styles"]["long"].__setitem__("calendar", d["locales"][0]["styles"]["long"]["currency"]))
        reject(lambda d: d["name_pool"].append(copy.deepcopy(d["name_pool"][0])))
        reject(lambda d: d["locales"][0]["styles"]["long"].pop("currency"))
        data = copy.deepcopy(self.data)
        row = copy.deepcopy(data["name_pool"][0]); row["entries"][0][1] += " unused"
        indexed = list(enumerate(data["name_pool"] + [row])); indexed.sort(key=lambda pair: GEN.canonical(pair[1]))
        indices = {old: new for new, (old, _) in enumerate(indexed)}
        data["name_pool"] = [record for _, record in indexed]
        for locale in data["locales"]:
            for refs in locale["styles"].values():
                for kind in refs: refs[kind] = indices[refs[kind]]
        with self.assertRaisesRegex(ValueError, "unused"): GEN.expand_names(data)

    def test_private_tamper_fixture_rejects_changed_primary_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "datetime-cldr-47"
            shutil.copytree(SCRIPTS.parent / GEN.SOURCE, source)
            en = source / "common/main/en.xml"
            en.chmod(0o600)  # Only the copied private fixture is made writable.
            en.write_bytes(en.read_bytes().replace(b"British English", b"Invented English", 1))
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                CldrProfile(source)

    def test_display_field_mapping_uses_cldr_week_dayperiod_and_zone_names(self):
        self.assertEqual(GEN.FIELDS["weekOfYear"], "week")
        self.assertEqual(GEN.FIELDS["dayPeriod"], "dayperiod")
        self.assertEqual(GEN.FIELDS["timeZoneName"], "zone")
        for requested in self.report["consumed_leaves"]:
            if "/dates/fields/" in requested:
                self.assertTrue(requested.endswith("/displayName"))
        self.assertNotIn("millisecond", GEN.FIELDS)


if __name__ == "__main__": unittest.main()
