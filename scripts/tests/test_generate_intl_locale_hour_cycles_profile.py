import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET

NATIVE_ROOT = Path(__file__).resolve().parents[2]
PRIMARY_ROOT = Path(os.environ.get('LILA_LOCALE_HOUR_CYCLES_PRIMARY_ROOT', NATIVE_ROOT))
spec = importlib.util.spec_from_file_location('locale_hour_cycles_generator', NATIVE_ROOT / 'scripts/generate-intl-locale-hour-cycles-profile.py')
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)


class TimeDataAdmission(unittest.TestCase):
    def setUp(self):
        self.root = ET.parse(PRIMARY_ROOT / generator.PRIMARY / 'common/supplemental/supplementalData.xml').getroot()

    def test_complete_primary_coverage(self):
        selectors, regions, rows, inheritance = generator.extract(self.root)
        self.assertEqual((len(rows), len(selectors), len(regions)), (25, 275, 292))
        self.assertEqual(sum('-' in key for key in selectors), 23)
        self.assertEqual(len(inheritance), 292)

    def test_day_period_symbols_collapse_with_stable_uniqueness(self):
        self.assertEqual(generator.project('hB hb h H K k'), ['h12', 'h23', 'h11', 'h24'])

    def test_allowed_order_does_not_prepend_compatibility_preferred(self):
        selectors, _, rows, _ = generator.extract(self.root)
        self.assertEqual(selectors['CD'], ['h12', 'h23'])
        self.assertTrue(any(row['preferred'] == 'H' and row['allowed'] == 'hB H' and 'CD' in row['selectors'] for row in rows))

    def test_language_region_rows_are_preserved(self):
        selectors, _, _, _ = generator.extract(self.root)
        self.assertEqual(selectors['fr-CA'], ['h23', 'h12'])
        self.assertEqual(selectors['CA'], ['h12', 'h23'])

    def test_sparse_recognized_region_inherits_primary_world_row(self):
        _, regions, _, inheritance = generator.extract(self.root)
        self.assertIn({'region': 'AQ', 'cycles': ['h23', 'h12']}, regions)
        self.assertIn({'region': 'AQ', 'source_selector': '001'}, inheritance)

    def test_unknown_region_and_week_only_deprecated_region_are_not_inferred(self):
        _, regions, _, _ = generator.extract(self.root)
        self.assertFalse({'XY', 'ZZ', 'AN'} & {row['region'] for row in regions})

    def test_missing_time_data_is_rejected(self):
        self.root.remove(self.root.find('timeData'))
        with self.assertRaises(ValueError): generator.extract(self.root)

    def test_missing_world_default_is_rejected(self):
        for row in self.root.find('timeData'):
            row.attrib['regions'] = ' '.join(value for value in row.attrib['regions'].split() if value != '001')
        with self.assertRaises(ValueError): generator.extract(self.root)

    def test_duplicate_selectors_are_rejected(self):
        self.root.find('timeData').append(copy.deepcopy(self.root.find('timeData/hours')))
        with self.assertRaises(ValueError): generator.extract(self.root)

    def test_changed_row_schema_is_rejected(self):
        self.root.find('timeData/hours').set('future', 'true')
        with self.assertRaises(ValueError): generator.extract(self.root)

    def test_unknown_symbols_and_empty_allowed_values_are_rejected(self):
        for value in ['Q', '']:
            with self.assertRaises(ValueError): generator.project(value)

    def test_malformed_selector_is_rejected(self):
        self.root.find('timeData/hours').set('regions', 'en_US_extra')
        with self.assertRaises(ValueError): generator.extract(self.root)

    def test_missing_shared_region_helper_is_a_generation_error(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for path in generator.PRODUCTION:
                destination = root / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes((NATIVE_ROOT / path).read_bytes())
            with self.assertRaises(FileNotFoundError): generator.kernel_sources(PRIMARY_ROOT, root)

    def test_three_outputs_reproduce_and_bind_shared_helper_and_closed_domain(self):
        outputs = generator.outputs(PRIMARY_ROOT, NATIVE_ROOT)
        self.assertEqual(len(outputs), 3)
        for path, data in outputs.items():
            self.assertEqual((NATIVE_ROOT / path).read_bytes(), data)
        provenance = json.loads(outputs[generator.DEST / 'provenance.json'])
        sources = {row['path'] for row in provenance['kernel_input']['sources']}
        self.assertIn(generator.REGION_HELPER.as_posix(), sources)
        self.assertIn('crates/lila-intl/src/datetime.rs', sources)
        self.assertNotIn((generator.NATIVE / 'tests.rs').as_posix(), sources)


if __name__ == '__main__':
    unittest.main()
