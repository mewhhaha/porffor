import copy
import importlib.util
import json
from pathlib import Path
import unittest
import xml.etree.ElementTree as ET
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('calendar_generator',ROOT/'scripts/generate-intl-locale-calendars-profile.py')
generator=importlib.util.module_from_spec(spec);spec.loader.exec_module(generator)
class CalendarPreferenceAdmission(unittest.TestCase):
    def setUp(self):
        self.root=ET.parse(ROOT/generator.PRIMARY/'common/supplemental/supplementalData.xml').getroot()
        self.calendar=ET.parse(ROOT/generator.PRIMARY/'common/bcp47/calendar.xml').getroot()
    def extract(self):return generator.extract(self.root,self.calendar)
    def test_complete_primary_coverage(self):
        selectors,regions,rows,_,_,domain=self.extract()
        self.assertEqual((len(rows),len(selectors),len(regions),len(domain)),(15,52,292,18))
    def test_aliases_project_in_preference_order(self):
        selectors,_,_,_,aliases,_=self.extract()
        self.assertEqual(aliases['gregorian'],'gregory')
        self.assertEqual(selectors['IR'],['persian','gregory','islamic','islamic-civil','islamic-tbla'])
        self.assertEqual(selectors['TH'],['buddhist','gregory'])
    def test_raw_unsupported_calendars_remain_visible_before_runtime_filter(self):
        selectors,*_=self.extract()
        self.assertIn('islamic-rgsa',selectors['SA']);self.assertIn('islamic',selectors['IR'])
    def test_world_inheritance_is_only_for_known_regions(self):
        _,regions,_,inheritance,_,_=self.extract()
        self.assertIn({'region':'AQ','calendars':['gregory']},regions)
        self.assertIn({'region':'AQ','source_selector':'001'},inheritance)
        self.assertFalse({'XY','ZZ','AN'} & {row['region'] for row in regions})
    def test_missing_preference_data_is_rejected(self):
        self.root.remove(self.root.find('calendarPreferenceData'))
        with self.assertRaises(ValueError):self.extract()
    def test_missing_world_is_rejected(self):
        for row in self.root.find('calendarPreferenceData'):
            row.set('territories',' '.join(x for x in row.attrib['territories'].split() if x!='001'))
        with self.assertRaises(ValueError):self.extract()
    def test_duplicate_selectors_are_rejected(self):
        self.root.find('calendarPreferenceData').append(copy.deepcopy(self.root.find('calendarPreferenceData/calendarPreference')))
        with self.assertRaises(ValueError):self.extract()
    def test_unknown_empty_and_unreviewed_row_schema_are_rejected(self):
        for value in ['', 'foobar']:
            root=copy.deepcopy(self.root);root.find('calendarPreferenceData/calendarPreference').set('ordering',value)
            with self.assertRaises(ValueError):generator.extract(root,self.calendar)
        self.root.find('calendarPreferenceData/calendarPreference').set('future','1')
        with self.assertRaises(ValueError):self.extract()
    def test_malformed_selector_is_rejected(self):
        self.root.find('calendarPreferenceData/calendarPreference').set('territories','en_US_extra')
        with self.assertRaises(ValueError):self.extract()
    def test_genuine_outputs_bind_consumed_available_calendar_query_and_primary_alias_data(self):
        outputs=generator.outputs(ROOT,ROOT)
        for path,data in outputs.items():self.assertEqual((ROOT/path).read_bytes(),data)
        proof=json.loads(outputs[generator.DEST/'provenance.json'])
        self.assertTrue(any(row['path'].endswith('common/bcp47/calendar.xml') for row in proof['inputs']))
        sources={row['path'] for row in proof['kernel_input']['sources']}
        self.assertIn('crates/lila-intl/src/provider/datetime.rs',sources)
        self.assertIn('crates/lila-intl/src/provider/datetime/generated/profile.json',sources)
        self.assertNotIn((generator.NATIVE/'tests.rs').as_posix(),sources)
if __name__=='__main__':unittest.main()
