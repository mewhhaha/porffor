import importlib.util
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
from copy import deepcopy


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
SPEC = importlib.util.spec_from_file_location("generate_intl_datetime_profile", SCRIPTS / "generate-intl-datetime-profile.py")
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)
SOURCES = SCRIPTS.parent / GENERATOR.SOURCE_PATH
from intl_datetime_pool import canonical, expand_profile, pool_profile, EXPANDED_CALENDAR_IDENTIFIERS, EXPANDED_LOCALES


class DateTimeProfileGenerationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.encoded, cls.report = GENERATOR.generate(SOURCES)
        cls.pooled = json.loads(cls.encoded)
        cls.profile = expand_profile(cls.pooled)

    def test_pools_retain_exact_resolved_fields_and_canonical_alias_ownership(self):
        self.assertEqual(pool_profile(self.profile), self.pooled)
        self.assertEqual(self.encoded, canonical(self.pooled) + "\n")
        self.assertEqual(len(self.pooled["calendar_pool"]), 135)
        self.assertEqual(len(self.pooled["zone_name_pool"]), 9)
        for locale in self.pooled["locales"]:
            refs = dict(locale["calendar_refs"])
            self.assertEqual(list(refs), list(EXPANDED_CALENDAR_IDENTIFIERS))
            self.assertEqual(refs["gregory"], refs["iso8601"])
            self.assertEqual(self.pooled["calendar_pool"][refs["chinese"]]["calendar"], "chinese")

    def test_invalid_pool_owners_are_rejected_before_materialization(self):
        def changed(mutate):
            rows = deepcopy(self.pooled)
            mutate(rows)
            return rows
        def refs(rows):
            return rows["locales"][0]["calendar_refs"]
        cases = {
            "missing canonical calendar": lambda r: refs(r).pop(),
            "duplicate canonical calendar": lambda r: refs(r).__setitem__(1, refs(r)[0]),
            "unknown canonical calendar": lambda r: refs(r)[0].__setitem__(0, "gregorian"),
            "out of bounds": lambda r: refs(r)[0].__setitem__(1, len(r["calendar_pool"])),
            "wrong domain": lambda r: refs(r)[2].__setitem__(1, refs(r)[0][1]),
            "divergent ISO alias": lambda r: refs(r)[1].__setitem__(1, refs(r)[2][1]),
            "boolean index": lambda r: refs(r)[0].__setitem__(1, True),
            "zone out of bounds": lambda r: r["locales"][0].__setitem__("zone_name_ref", len(r["zone_name_pool"])),
            "duplicate pool row": lambda r: r["calendar_pool"].append(deepcopy(r["calendar_pool"][0])),
            "unknown locale field": lambda r: r["locales"][0].__setitem__("calendars", {}),
        }
        for name, mutate in cases.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                expand_profile(changed(mutate))
        unused = deepcopy(self.pooled)
        unused["zone_name_pool"].append({"patterns": {}, "zones": [], "metazones": []})
        unused["zone_name_pool"].sort(key=canonical)
        # Preserve previous zone references while inserting a canonical unused row.
        indices = {canonical(r): i for i, r in enumerate(unused["zone_name_pool"])}
        for before, after in zip(self.pooled["locales"], unused["locales"]):
            after["zone_name_ref"] = indices[canonical(self.pooled["zone_name_pool"][before["zone_name_ref"]])]
        with self.assertRaisesRegex(ValueError, "unused"):
            expand_profile(unused)

    def test_pinned_profile_regenerates_all_selected_locales_and_numeric_systems(self):
        expected = SCRIPTS.parent / "crates/lila-intl/src/provider/datetime/generated/profile.json"
        self.assertEqual(expected.read_text(), self.encoded)
        self.assertEqual([row["locale"] for row in self.profile["locales"]], EXPANDED_LOCALES)
        digits = {row["identifier"]: row["digits"] for row in self.profile["numbering_systems"]}
        self.assertEqual(len(digits), 78)
        self.assertEqual(digits["tols"], "𑷠𑷡𑷢𑷣𑷤𑷥𑷦𑷧𑷨𑷩")
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

    def test_range_endpoint_companions_retain_genuine_default_text_and_provenance(self):
        from intl_cldr_profile import CldrProfile, PatternAlternate
        default = CldrProfile(SOURCES, pattern_alternate=PatternAlternate.DEFAULT)
        report = json.loads(self.report)
        self.assertTrue(report["range_default_consumed_leaves"])
        count = 0
        for locale in self.profile["locales"]:
            for physical, calendar in locale["calendars"].items():
                base = f"dates/calendars/calendar[@type='{physical}']"
                candidates = [(row, f"{base}/dateTimeFormats/availableFormats/dateFormatItem[@id='{row['skeleton']}']") for row in calendar["available"]]
                candidates.extend((row[kind], f"{base}/{kind}Formats/{kind}FormatLength[@type='{style}']/{kind}Format/pattern") for style, row in calendar["styles"].items() for kind in ("date", "time"))
                for primary, path in candidates:
                    genuine = GENERATOR.pattern_leaf(default.resolve(locale["locale"].replace('-', '_'), path))
                    companion = primary.get("range_pattern")
                    expected = {key: value for key, value in primary.items() if key not in ("skeleton", "range_pattern")}
                    if genuine == expected:
                        self.assertIsNone(companion)
                    else:
                        count += 1
                        self.assertEqual(companion, genuine)
                        self.assertEqual([token for token in primary["tokens"] if "field" in token], [token for token in companion["tokens"] if "field" in token])
                        self.assertEqual(primary["numbering_overrides"], companion["numbering_overrides"])
                if locale["locale"] in ("en", "en-US"):
                    formats = {row["skeleton"]: row for row in calendar["available"]}
                    self.assertEqual(formats["hms"]["source"], "h:mm:ss a")
                    self.assertEqual(formats["hms"]["range_pattern"]["source"], "h:mm:ss\u202fa")
        self.assertEqual(count, 274)

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

    def test_buddhist_profiles_consume_single_era_and_actual_calendar_patterns(self):
        for locale in self.profile["locales"]:
            calendar = locale["calendars"]["buddhist"]
            self.assertEqual(calendar["calendar"], "buddhist")
            eras = [row for row in calendar["names"] if row["kind"] == "era"]
            self.assertTrue(eras)
            self.assertTrue(all(row["index"] == 0 for row in eras))
            self.assertTrue(any(token.get("field") == "G" for token in calendar["styles"]["long"]["date"]["tokens"]))
            self.assertTrue(calendar["intervals"])
        consumed = json.loads(self.report)["consumed_leaves"]
        self.assertTrue(any("calendar[@type='buddhist']" in row["path"] for row in consumed.values()))

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

    def test_expansion_preserves_genuine_thirteen_month_and_era_namespaces(self):
        for locale in self.profile["locales"]:
            for physical in ("hebrew", "coptic", "ethiopic", "ethiopic-amete-alem"):
                names = locale["calendars"][physical]["names"]
                months = [r for r in names if r["kind"] == "month"]
                self.assertEqual({r["index"] for r in months}, set(range(1, 14)))
                leap = [r for r in months if r.get("year_type") == "leap"]
                self.assertEqual({r["index"] for r in leap}, {7} if physical == "hebrew" else set())
            japanese = locale["calendars"]["japanese"]["names"]
            eras = [r for r in japanese if r["kind"] == "era"]
            self.assertEqual({r["index"] for r in eras if "era_source_calendar" not in r}, set(range(232, 237)))
            self.assertEqual({r["index"] for r in eras if r.get("era_source_calendar") == "gregorian"}, {0, 1})
            coptic = [r for r in locale["calendars"]["coptic"]["names"] if r["kind"] == "era"]
            self.assertEqual({r["index"] for r in coptic}, {1})
        provenance = json.loads(self.report)["consumed_leaves"]
        supplied = [r for r in provenance.values() if r.get("source_cldr_commit") == "acd6d88ae493633240e19a87a721076a8a75c310"]
        self.assertTrue(supplied)
        self.assertTrue(all("/eras/" in r["path"] for r in supplied))
        self.assertFalse(any("calendar[@type='buddhist']" in r["path"] for r in supplied))

    def test_first_weekday_and_source_recipe_are_closed_before_pool_expansion(self):
        expected = ["sun", "sun", "sat", "sat", "mon", "mon", "mon", "mon", "mon", "mon", "sun", "sun", "sun"]
        self.assertEqual([r["first_weekday"] for r in self.profile["locales"]], expected)
        for key, value in (("cldr_commit", "wrong"), ("source_manifest_sha256", "0" * 64), ("cldr_release", "47.0.0")):
            tampered = deepcopy(self.pooled)
            tampered["era_supplement"][key] = value
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, "source recipe"):
                expand_profile(tampered)
        for value in ("monday", 1, None):
            tampered = deepcopy(self.pooled)
            tampered["locales"][0]["first_weekday"] = value
            with self.subTest(first_weekday=value), self.assertRaisesRegex(ValueError, "first-weekday"):
                expand_profile(tampered)

    def test_numeric_weekday_intervals_keep_genuine_field_and_skeleton_domains(self):
        from intl_datetime_patterns import compile_pattern, compile_interval
        for skeleton in (None, "d", "e", "cc"):
            with self.subTest(skeleton=skeleton), self.assertRaisesRegex(ValueError, "textual skeleton"):
                compile_interval("e – e", skeleton=skeleton)
        self.assertEqual(compile_interval("e – e", skeleton="E")["second_start"], 2)
        self.assertEqual(compile_pattern("e ee c cc"), [
            {"field": "e", "width": 1}, {"literal": " "}, {"field": "e", "width": 2},
            {"literal": " "}, {"field": "c", "width": 1}, {"literal": " "}, {"field": "c", "width": 2},
        ])
        found = []
        for locale in self.profile["locales"]:
            for physical, calendar in locale["calendars"].items():
                for row in calendar["available"]:
                    self.assertFalse(any(t.get("field") in ("e", "c") and t["width"] <= 2 for t in row["tokens"]))
                for row in calendar["intervals"]:
                    if any(t.get("field") in ("e", "c") and t["width"] <= 2 for t in row["tokens"]):
                        self.assertIn("E", row["skeleton"])
                        found.append((locale["locale"], physical))
        self.assertTrue(found)
        self.assertEqual({locale for locale, _ in found}, {"ko"})

    def test_japanese_first_year_kernel_keeps_exact_integer_fallback_rules(self):
        row, = [r for r in self.profile["algorithmic_fields"] if r["identifier"] == "jpanyear"]
        self.assertEqual((row["field"], row["minimum"], row["values"], row["positional_fallback"]), ("y", 1, ["元"], "latn"))
        self.assertEqual(row["consumed_rules"], [
            ["spellout-numbering-year-latn", 0, "=0="],
            ["spellout-numbering-year-latn", 1, "元"],
            ["spellout-numbering-year-latn", 2, "=0="],
        ])
        self.assertEqual(row["source"], "common/rbnf/ja.xml")
        japanese = next(r for r in self.profile["locales"] if r["locale"] == "ja")["calendars"]["japanese"]
        self.assertEqual(japanese["styles"]["full"]["date"]["numbering_overrides"], [{"field": "y", "numbering": "jpanyear"}])
        self.assertEqual(japanese["styles"]["medium"]["date"]["numbering_overrides"], [{"field": "y", "numbering": "jpanyear"}])
        self.assertFalse(any(p["numbering_overrides"] for p in japanese["available"]))

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
            shutil.copytree(SOURCES.parent / "numbering-tols-cldr-48", source.parent / "numbering-tols-cldr-48")
            shutil.copytree(SOURCES.parent / "calendar-eras-cldr-48", source.parent / "calendar-eras-cldr-48")
            relative = "common/main/root.xml"
            path = source / relative
            path.chmod(0o600)
            document = ET.parse(path)
            symbols = document.getroot().find("./numbers/symbols[@numberSystem='arab']")
            decimal = symbols.find("decimal")
            self.assertIsNotNone(decimal)
            symbols.remove(decimal)
            document.write(path, encoding="utf-8", xml_declaration=True)
            manifest_path = source / "manifest.json"
            manifest_path.chmod(0o600)
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
            shutil.copytree(SOURCES.parent / "numbering-tols-cldr-48", source.parent / "numbering-tols-cldr-48")
            shutil.copytree(SOURCES.parent / "calendar-eras-cldr-48", source.parent / "calendar-eras-cldr-48")
            path = source / "common/main/ar.xml"
            path.chmod(0o600)
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
