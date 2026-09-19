import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import xml.etree.ElementTree as ET


SCRIPT = Path(__file__).resolve().parents[1] / "generate-intl-time-zone-names.py"
SPEC = importlib.util.spec_from_file_location("generate_intl_time_zone_names", SCRIPT)
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)
SOURCE = SCRIPT.parent.parent / GENERATOR.SOURCE_PATH


class TimeZoneNameGenerationTests(unittest.TestCase):
    def rewrite_source(self, source, relative, contents):
        path = source / relative
        path.write_bytes(contents)
        manifest_path = source / "manifest.json"
        manifest = json.loads(manifest_path.read_text())
        entry = next(row for row in manifest["files"] if row["path"] == relative)
        entry["sha256"] = hashlib.sha256(contents).hexdigest()
        entry["bytes"] = len(contents)
        manifest["total_bytes"] = sum(row["bytes"] for row in manifest["files"])
        manifest_path.write_text(json.dumps(manifest))

    def rewrite_xml(self, source, relative, edit):
        root = ET.parse(source / relative).getroot()
        edit(root)
        self.rewrite_source(source, relative, ET.tostring(root, encoding="utf-8"))

    def test_complete_pinned_domain_regenerates_identically(self):
        generated, report = GENERATOR.generate(SOURCE)
        self.assertEqual((SCRIPT.parent.parent / GENERATOR.OUTPUT_PATH).read_text(), generated)
        report = json.loads(report)
        self.assertEqual((report["zones"], report["aliases"], report["metazones"], report["periods"]), (446, 600, 190, 669))
        self.assertEqual(report["supported_locales"], ["en", "en-US"])
        self.assertEqual(report["consumed_icu_fields"], ["Names", "Regions:array"])

    def test_pinned_country_authority_is_independent_of_cldr_alias_grouping(self):
        countries = GENERATOR.icu_countries((SOURCE / "icu-77-1-zoneinfo64.icu").read_text(encoding="utf-8-sig"))
        self.assertEqual(countries["Antarctica/South_Pole"], "AQ")
        self.assertEqual(countries["Atlantic/Jan_Mayen"], "SJ")
        _, _, rows = GENERATOR.extract(SOURCE)
        zones = {zone["identifier"]: zone for zone in rows["zones"]}
        self.assertEqual(zones["Antarctica/McMurdo"]["country"], "Antarctica")
        self.assertEqual(zones["Arctic/Longyearbyen"]["location"], "Svalbard & Jan Mayen")
        self.assertEqual(zones["Europe/London"]["location"], "UK")
        self.assertEqual(zones["Europe/London"]["country"], "United Kingdom")

    def test_utc_periods_preserve_an_overlap_boundary_and_missing_tail(self):
        _, _, rows = GENERATOR.extract(SOURCE)
        zones = {zone["identifier"]: zone for zone in rows["zones"]}
        self.assertIn((972_802_800, 973_400_400, "America_Eastern"), zones["America/Cambridge_Bay"]["periods"])
        self.assertIn((973_400_400, 986_115_600, "America_Central"), zones["America/Cambridge_Bay"]["periods"])
        self.assertEqual(zones["Africa/Casablanca"]["periods"][-1][1], 1_540_692_000)

    def test_unpinned_input_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "cldr"
            shutil.copytree(SOURCE, source)
            path = source / "common/main/en.xml"
            path.write_text(path.read_text() + "\n")
            with self.assertRaisesRegex(ValueError, "source checksum mismatch"):
                GENERATOR.generate(source)

    def test_metazone_overlap_is_rejected_instead_of_silently_reordered(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "cldr"
            shutil.copytree(SOURCE, source)

            def overlap(root):
                periods = root.find("./metaZones/metazoneInfo/timezone[@type='America/Cambridge_Bay']")
                periods[1].set("from", "1999-10-31 07:59")

            self.rewrite_xml(source, "common/supplemental/metaZones.xml", overlap)
            with self.assertRaisesRegex(ValueError, "overlapping or unordered metazone"):
                GENERATOR.generate(source)

    def test_missing_preferred_zone_and_changed_hour_pattern_fail_generation(self):
        for relative, edit, message in [
            ("common/supplemental/metaZones.xml",
             lambda root: root.find("./metaZones/mapTimezones/mapZone[@other='America_Pacific'][@territory='001']").set("type", "America/Missing_Zone"),
             "unknown preferred time zone"),
            ("common/main/en.xml",
             lambda root: setattr(root.find("./dates/timeZoneNames/hourFormat"), "text", "+H.mm;-H.mm"),
             "unsupported localized offset hour pattern"),
        ]:
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as temporary:
                source = Path(temporary) / "cldr"
                shutil.copytree(SOURCE, source)
                self.rewrite_xml(source, relative, edit)
                with self.assertRaisesRegex(ValueError, message):
                    GENERATOR.generate(source)

    def test_country_arrays_must_have_matching_cardinality(self):
        with self.assertRaisesRegex(ValueError, "mismatched or duplicate names"):
            GENERATOR.icu_countries('Names { "A", "B" }\nRegions:array { "US" }')


if __name__ == "__main__":
    unittest.main()
