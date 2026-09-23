"""Catalogue identity, country ownership and corrupt-source controls."""
import importlib.util
from pathlib import Path
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "generate-intl-named-time-zones.py"
SPEC = importlib.util.spec_from_file_location("intl_named_time_zones", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
ROOT = SCRIPT.parents[1]


class NamedTimeZoneGenerationTests(unittest.TestCase):
    def test_checked_outputs_are_exact_and_repeatable(self):
        outputs = MODULE.outputs(ROOT)
        self.assertEqual(outputs, MODULE.outputs(ROOT))
        for path, content in outputs.items():
            self.assertEqual((ROOT / path).read_bytes(), content, str(path))

    def test_country_names_are_explicit_and_not_inferred_from_alias_groups(self):
        source = (ROOT / MODULE.DATA / "icu-77-1-zoneinfo64.icu").read_text()
        countries = MODULE.country_records(source)
        self.assertEqual(countries["Antarctica/South_Pole"], "AQ")
        self.assertEqual(countries["Atlantic/Jan_Mayen"], "SJ")
        self.assertEqual(countries["CET"], "BE")
        self.assertEqual(countries["Etc/UTC"], "001")

    def test_explicit_unflattened_alias_country_precedes_icu_fallback(self):
        sources = MODULE.archive_contents(
            ROOT / MODULE.DATA / "tzdata2026a.tar.gz",
            MODULE.ARCHIVES["tzdata2026a.tar.gz"])
        countries = MODULE.country_records(
            (ROOT / MODULE.DATA / "icu-77-1-zoneinfo64.icu").read_text())
        tab, _ = MODULE.tab_records(sources["zone.tab"].decode(), countries)
        zones, links, targets = MODULE.zone_records(sources)
        resolved = MODULE.geographic_link_countries(zones, links, tab, countries, targets)
        self.assertEqual(countries["Iceland"], "CI")
        self.assertEqual(resolved["Iceland"], "IS")
        self.assertEqual(resolved["Atlantic/Jan_Mayen"], "SJ")
        self.assertEqual(resolved["Antarctica/South_Pole"], "AQ")
        self.assertEqual(targets, {
            "Australia/ACT": "Australia/Canberra", "Brazil/Acre": "America/Porto_Acre",
            "Iceland": "Atlantic/Reykjavik", "Navajo": "America/Shiprock",
            "America/Virgin": "America/St_Thomas", "Africa/Asmera": "Africa/Asmara",
            "Asia/Chungking": "Asia/Chongqing", "Pacific/Ponape": "Pacific/Pohnpei",
            "Pacific/Truk": "Pacific/Chuuk",
        })

    def test_annotation_chain_country_is_order_independent_and_tab_is_authoritative(self):
        zones = {"Foreign/Zone"}
        links = {"Country/Primary": "Foreign/Zone", "Old/A": "Foreign/Zone",
                 "Old/B": "Foreign/Zone"}
        tab = {"Country/Primary": "CA"}
        countries = {"Foreign/Zone": "US", "Old/A": "US", "Old/B": "US"}
        targets = {"Old/A": "Old/B", "Old/B": "Country/Primary",
                   "Country/Primary": "Foreign/Zone"}
        expected = countries | tab | {"Old/A": "CA", "Old/B": "CA"}
        self.assertEqual(MODULE.geographic_link_countries(
            zones, links, tab, countries, targets), expected)
        self.assertEqual(MODULE.geographic_link_countries(
            zones, links, tab, countries, dict(reversed(list(targets.items())))), expected)
        self.assertEqual(countries["Old/A"], "US")

    def test_corrupt_annotation_targets_fail_before_primary_selection(self):
        zones = {"Zone/A", "Zone/B"}
        links = {"Old/A": "Zone/A", "Old/B": "Zone/A"}
        countries = {"Zone/A": "US", "Zone/B": "CA"}
        for targets, message in [
            ({"Old/A": "Missing"}, "unknown unflattened"),
            ({"Old/A": "Zone/B"}, "changes terminal"),
            ({"Old/A": "Old/A"}, "cyclic unflattened"),
            ({"Old/A": "Old/B", "Old/B": "Old/A"}, "cyclic unflattened"),
            ({"Missing": "Zone/A"}, "unknown unflattened"),
        ]:
            with self.assertRaisesRegex(ValueError, message):
                MODULE.geographic_link_countries(zones, links, {}, countries, targets)
        with self.assertRaisesRegex(ValueError, "missing unflattened Link country"):
            MODULE.geographic_link_countries(zones, links, {}, {}, {"Old/A": "Zone/A"})

    def test_unflattened_annotation_parser_rejects_malformed_declarations(self):
        for declaration in ["Link A B #=", "Link A B #= C D", "Link A #= B",
                            "Zone A #= B"]:
            sources = {name: b"" for name in MODULE.SOURCE_FILES}
            sources["backward"] = declaration.encode()
            with self.assertRaisesRegex(ValueError, "invalid unflattened Link annotation"):
                MODULE.zone_records(sources)

    def test_broken_country_arrays_and_new_country_conflicts_fail(self):
        for source in ['Names { "A" } Regions:array { "US", "CA" }',
                       'Names { "A", "A" } Regions:array { "US", "US" }',
                       'Names { "A" } Regions:array { "usa" }']:
            with self.assertRaises(ValueError):
                MODULE.country_records(source)
        with self.assertRaisesRegex(ValueError, "disagrees"):
            MODULE.tab_records("CA +0000 A\n", {"A": "US"})

    def test_country_backzone_and_world_region_rules_are_not_simple_link_resolution(self):
        zones = {"Etc/UTC", "Region/Zone"}
        links = {"UTC": "Etc/UTC", "Other/Main": "Region/Zone",
                 "Other/Old": "Region/Zone", "World": "Region/Zone"}
        tab = {"Region/Zone": "US", "Other/Main": "CA"}
        by_country = {"US": ["Region/Zone"], "CA": ["Other/Main"]}
        countries = {"Region/Zone": "US", "Other/Main": "CA", "Other/Old": "CA", "World": "001"}
        result = MODULE.primary_identifiers(zones, links, tab, by_country, countries, set(), {})
        self.assertEqual(result["Other/Old"], "Other/Main")
        self.assertEqual(result["Other/Main"], "Other/Main")
        self.assertEqual(result["World"], "Region/Zone")
        self.assertEqual(result["Etc/UTC"], "UTC")
        self.assertEqual(result["UTC"], "UTC")

    def test_cyclic_links_and_unknown_country_never_get_guessed_primaries(self):
        with self.assertRaisesRegex(ValueError, "cyclic"):
            MODULE.resolve_link("A", set(), {"A": "B", "B": "A"})
        with self.assertRaisesRegex(ValueError, "missing geographic country"):
            MODULE.primary_identifiers({"Zone", "Etc/UTC"}, {"UTC": "Etc/UTC", "Alias": "Zone"},
                                       {}, {}, {}, set(), {})

    def test_cross_country_self_primary_requires_an_explicit_backzone_zone(self):
        zones = {"Elsewhere", "Etc/UTC"}
        links = {"UTC": "Etc/UTC", "Country/Old": "Elsewhere", "Country/First": "Elsewhere", "Country/Second": "Elsewhere"}
        tab = {"Country/First": "CA", "Country/Second": "CA", "Elsewhere": "US"}
        by_country = {"CA": ["Country/First", "Country/Second"], "US": ["Elsewhere"]}
        countries = tab | {"Country/Old": "CA"}
        with self.assertRaisesRegex(ValueError, "missing cross-country backzone"):
            MODULE.primary_identifiers(zones, links, tab, by_country, countries, set(), {})
        result = MODULE.primary_identifiers(zones, links, tab, by_country, countries, {"Country/Old"}, {})
        self.assertEqual(result["Country/Old"], "Country/Old")


if __name__ == "__main__":
    unittest.main()
