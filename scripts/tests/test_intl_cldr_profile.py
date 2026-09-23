import pathlib
import sys
import unittest
import xml.etree.ElementTree as ET

SCRIPTS = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

from intl_cldr_profile import CldrProfile, LocaleTree, PatternAlternate, parse_path
from intl_datetime_patterns import compile_interval, compile_pattern
from intl_ldml_schema import AttributeRole, LdmlSchema


SOURCES = SCRIPTS.parent / "crates/lila-intl/data/datetime-cldr-47"
CHINESE_FULL = "dates/calendars/calendar[@type='chinese']/dateFormats/dateFormatLength[@type='full']/dateFormat/pattern"


class LdmlAttributeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.schema = LdmlSchema((SOURCES / "common/dtd/ldml.dtd").read_text())

    def test_pattern_numbering_is_a_value_and_metadata_is_not_a_key(self):
        self.assertEqual(self.schema.rule("pattern", "numbers").role, AttributeRole.VALUE)
        self.assertEqual(self.schema.rule("pattern", "draft").role, AttributeRole.METADATA)
        self.assertEqual(self.schema.distinguishing("pattern", {
            "type": "standard", "numbers": "d=hanidays", "draft": "approved",
        }), ())
        self.assertEqual(self.schema.values("pattern", {"numbers": "d=hanidays"}), (("numbers", "d=hanidays"),))

    def test_xml_namespace_metadata_uses_the_dtd_attribute(self):
        self.assertEqual(self.schema.distinguishing("nativeSpaceReplacement", {
            "{http://www.w3.org/XML/1998/namespace}space": "preserve",
        }), ())

    def test_value_predicates_and_unknown_attributes_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "not distinguishing"):
            self.schema.normalize_predicates("pattern", (("numbers", "d=hanidays"),))
        with self.assertRaisesRegex(ValueError, "absent from pinned"):
            self.schema.distinguishing("pattern", {"unknown": "standard"})

    def test_approved_child_survives_unconfirmed_ancestor(self):
        tree = LocaleTree("test", ET.fromstring('''
            <ldml draft="unconfirmed"><dates><calendars><calendar type="chinese">
            <dateFormats><dateFormatLength type="full"><dateFormat>
            <pattern draft="approved" numbers="d=hanidays">rU年MMMdEEEE</pattern>
            </dateFormat></dateFormatLength></dateFormats>
            </calendar></calendars></dates></ldml>'''), 2, self.schema)
        leaf = tree.leaves[parse_path(CHINESE_FULL)]
        self.assertEqual(leaf.text, "rU年MMMdEEEE")
        self.assertEqual(leaf.attributes, (("numbers", "d=hanidays"),))

    def test_alias_path_preserves_slashes_inside_both_quote_styles(self):
        for source in ["zone[@type='Europe/Paris']/long/generic", 'zone[@type="Europe/Paris"]/long/generic']:
            parsed = parse_path(source)
            self.assertEqual(len(parsed), 3)
            self.assertEqual(parsed[0].get("type"), "Europe/Paris")
        with self.assertRaisesRegex(ValueError, "unterminated"):
            parse_path("zone[@type='Europe/Paris]/long")


class PinnedProfileTests(unittest.TestCase):
    def setUp(self):
        self.profile = CldrProfile(SOURCES)

    def test_chinese_style_inherits_pattern_and_numbering_as_one_value(self):
        for locale in ["zh", "zh_Hans", "zh_Hans_CN"]:
            leaf = self.profile.resolve(locale, CHINESE_FULL)
            self.assertEqual(leaf.value, "rU年MMMdEEEE")
            self.assertEqual(leaf.attributes, (("numbers", "d=hanidays"),))
            self.assertEqual(leaf.source_locale, "zh")
        with self.assertRaisesRegex(ValueError, "explicit consumer"):
            self.profile.text("zh", CHINESE_FULL)

    def test_default_pattern_type_and_omitted_type_resolve_identically(self):
        self.assertEqual(self.profile.resolve("zh", CHINESE_FULL),
                         self.profile.resolve("zh", CHINESE_FULL + "[@type='standard']"))

    def test_inheritance_marker_does_not_supply_a_partial_attribute_value(self):
        path = CHINESE_FULL.replace("type='full'", "type='medium'").replace("/pattern", "/datetimeSkeleton")
        leaf = self.profile.resolve("zh", path)
        self.assertNotEqual(leaf.value, "↑↑↑")
        self.assertEqual(leaf.source_locale, "root")
        self.assertEqual(leaf.attributes, ())


class PatternTests(unittest.TestCase):
    def test_interval_standalone_and_format_month_are_the_same_field(self):
        pattern = compile_interval("LLLL d – MMMM d, y")
        self.assertEqual(pattern["tokens"][pattern["second_start"]], {"field": "M", "width": 4})
        self.assertEqual(pattern["shared_fields"], ["year"])

    def test_related_and_cyclic_year_fields_do_not_start_second_endpoint(self):
        pattern = compile_interval("rU年M月d日至d日")
        self.assertEqual(pattern["tokens"][pattern["second_start"]], {"field": "d", "width": 1})
        self.assertEqual(pattern["shared_fields"], ["month", "relatedYear", "yearName"])

    def test_interval_order_is_explicit(self):
        earliest = compile_interval("earliestFirst:d–d")
        latest = compile_interval("latestFirst:d–d")
        self.assertEqual(earliest["tokens"], latest["tokens"])
        self.assertEqual(earliest["endpoint_order"], "earliest_first")
        self.assertEqual(latest["endpoint_order"], "latest_first")

    def test_quoted_literals_and_incomplete_syntax_remain_distinct(self):
        self.assertEqual(compile_pattern("d 'o''clock'"), [{"field": "d", "width": 1}, {"literal": " o'clock"}])
        with self.assertRaisesRegex(ValueError, "unterminated"):
            compile_pattern("d 'day")
        with self.assertRaisesRegex(ValueError, "unsupported"):
            compile_pattern("QQQ d")



class PatternAlternateTests(unittest.TestCase):
    def setUp(self):
        self.profile = CldrProfile(SOURCES, pattern_alternate=PatternAlternate.ASCII)
        self.base = "dates/calendars/calendar[@type='gregorian']"

    def replace_child(self, contents):
        document = ET.fromstring('<ldml><dates><calendars><calendar type="gregorian">'
                                 + contents + '</calendar></calendars></dates></ldml>')
        self.profile.locales["en_US"] = LocaleTree(
            "en_US", document, 2, self.profile.schema, pattern_alternate=PatternAlternate.ASCII)

    def test_supplied_alternate_keeps_source_path_and_inherits_through_calendar_aliases(self):
        path = self.base + "/timeFormats/timeFormatLength[@type='medium']/timeFormat/pattern"
        ordinary = CldrProfile(SOURCES).resolve("en_US", path)
        preferred = self.profile.resolve("en_US", path)
        self.assertEqual(ordinary.value, "h:mm:ss\u202fa")
        self.assertEqual(preferred.value, "h:mm:ss a")
        self.assertEqual(preferred.source_locale, "en")
        self.assertEqual(preferred.source_path[-1].get("alt"), "ascii")
        chinese = path.replace("'gregorian'", "'chinese'")
        self.assertEqual(self.profile.resolve("en_US", chinese), preferred)

    def test_child_default_precedes_parent_alternate(self):
        self.replace_child('<timeFormats><timeFormatLength type="medium"><timeFormat>'
                           "<pattern>h:mm:ss 'child\u202ftext' a</pattern>"
                           '</timeFormat></timeFormatLength></timeFormats>')
        leaf = self.profile.resolve("en_US", self.base + "/timeFormats/timeFormatLength[@type='medium']/timeFormat/pattern")
        self.assertEqual(leaf.source_locale, "en_US")
        self.assertEqual(leaf.value, "h:mm:ss 'child\u202ftext' a")
        self.assertIsNone(leaf.source_path[-1].get("alt"))

    def test_alternate_only_skeleton_is_enumerated_once_and_keeps_quoted_text(self):
        self.replace_child("""<dateTimeFormats><availableFormats>
          <dateFormatItem id="hms" alt="ascii">h:mm:ss 'literal\u202ftext' a</dateFormatItem>
          <dateFormatItem id="hms" alt="ascii-proposed" draft="unconfirmed">HH</dateFormatItem>
        </availableFormats></dateTimeFormats>""")
        branch = self.base + "/dateTimeFormats/availableFormats"
        values = [(path, leaf) for path, leaf in self.profile.leaves("en_US", branch)
                  if path[-1].get("id") == "hms" and path[-1].get("alt") is None]
        self.assertEqual(len(values), 1)
        leaf = values[0][1]
        self.assertEqual(leaf.attributes, ())
        self.assertEqual(leaf.source_locale, "en_US")
        self.assertEqual(compile_pattern(leaf.value)[-2], {"literal": " literal\u202ftext "})

    def test_selected_style_retains_value_attributes_with_its_pattern(self):
        self.replace_child("""<timeFormats><timeFormatLength type="medium"><timeFormat>
          <pattern numbers="latn">h:mm:ss a</pattern>
          <pattern alt="ascii" numbers="h=arab">h:mm:ss 'literal\u202ftext' a</pattern>
        </timeFormat></timeFormatLength></timeFormats>""")
        leaf = self.profile.resolve("en_US", self.base + "/timeFormats/timeFormatLength[@type='medium']/timeFormat/pattern")
        self.assertEqual(leaf.attributes, (("numbers", "h=arab"),))
        self.assertEqual(leaf.source_path[-1].get("alt"), "ascii")
        self.assertEqual(compile_pattern(leaf.value)[-2], {"literal": " literal\u202ftext "})

    def test_interval_fallback_append_and_names_have_distinct_alternate_ownership(self):
        self.replace_child("""<eras><eraNames>
          <era type="0">default name</era><era type="0" alt="ascii">alternate name</era>
        </eraNames></eras><dateTimeFormats>
          <intervalFormats><intervalFormatFallback>{0}–{1}</intervalFormatFallback>
            <intervalFormatFallback alt="ascii">{0} - {1}</intervalFormatFallback>
            <intervalFormatItem id="Hm"><greatestDifference id="H">HH:mm–HH:mm</greatestDifference>
              <greatestDifference id="H" alt="ascii">HH:mm - HH:mm</greatestDifference>
            </intervalFormatItem></intervalFormats>
          <appendItems><appendItem request="Era">{1} {0}</appendItem>
            <appendItem request="Era" alt="ascii">{0} {1}</appendItem></appendItems>
        </dateTimeFormats>""")
        branch = self.base + "/dateTimeFormats/"
        interval = self.profile.resolve("en_US", branch + "intervalFormats/intervalFormatItem[@id='Hm']/greatestDifference[@id='H']")
        self.assertEqual(compile_interval(interval.value)["tokens"][3], {"literal": " - "})
        self.assertEqual(self.profile.resolve("en_US", branch + "intervalFormats/intervalFormatFallback").value, "{0} - {1}")
        self.assertEqual(self.profile.resolve("en_US", branch + "appendItems/appendItem[@request='Era']").value, "{0} {1}")
        self.assertEqual(self.profile.resolve("en_US", self.base + "/eras/eraNames/era[@type='0']").value, "default name")

if __name__ == "__main__":
    unittest.main()
