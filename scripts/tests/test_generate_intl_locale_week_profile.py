"""Primary-data admission controls; no JavaScript or Rust execution."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import xml.etree.ElementTree as ET

SCRIPT = Path(__file__).resolve().parents[1] / 'generate-intl-locale-week-profile.py'
spec = importlib.util.spec_from_file_location('locale_week_generator', SCRIPT)
gen = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gen)
PRIMARY_ROOT = Path(os.environ.get('LILA_WEEK_PRIMARY_ROOT', SCRIPT.parents[1]))


class LocaleWeekGeneration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.primary, _, _ = gen.primary_inputs(PRIMARY_ROOT)
        cls.xml = ET.parse(cls.primary / 'common/supplemental/supplementalData.xml').getroot()

    def test_complete_territory_domain_includes_inherited_rows(self):
        rows, maps, _, territory, containment = gen.extract(self.xml)
        actual = {r['region'] for r in rows}
        expected = set().union(territory, containment, *(set(x) for x in maps.values())) - {'ZZ'}
        self.assertEqual(actual, expected)
        self.assertIn('AQ', actual)
        self.assertNotIn('AQ', maps['firstDay'])
        self.assertNotIn('ZZ', actual)

    def test_primary_examples_include_single_day_and_non_saturday_weekends(self):
        rows = {r['region']: r for r in gen.extract(self.xml)[0]}
        expected = {'001': (1, 0x60), 'GB': (1, 0x60), 'US': (7, 0x60),
                    'AF': (6, 0x18), 'IR': (6, 0x10), 'IN': (7, 0x40),
                    'UG': (1, 0x40), 'AQ': (1, 0x60), 'MV': (5, 0x60)}
        for region, (first, mask) in expected.items():
            self.assertEqual((rows[region]['first_day'], rows[region]['weekend_mask']), (first, mask))

    def test_alternative_gb_first_day_is_disclosed_and_not_default(self):
        rows, _, alternatives, _, _ = gen.extract(self.xml)
        self.assertEqual(next(r['first_day'] for r in rows if r['region'] == 'GB'), 1)
        self.assertEqual(alternatives, [{'field': 'firstDay', 'attributes': {
            'day': 'sun', 'territories': 'GB', 'alt': 'variant',
            'references': 'Shorter Oxford Dictionary (5th edition, 2002)'}}])

    def test_every_cyclic_weekend_mask_is_sorted_unique_and_nonempty(self):
        for start in range(1, 8):
            for end in range(1, 8):
                mask = gen.weekend_mask(start, end)
                days = [day for day in range(1, 8) if mask & (1 << (day - 1))]
                self.assertEqual(len(days), (end - start) % 7 + 1)
                self.assertEqual(days, sorted(set(days)))
                self.assertTrue(0 < mask < 128)
        self.assertEqual(gen.weekend_mask(7, 1), 0x41)

    def test_duplicate_region_field_is_rejected_before_output(self):
        modified = copy.deepcopy(self.xml)
        ET.SubElement(modified.find('weekData'), 'firstDay', {'day': 'mon', 'territories': 'US'})
        with self.assertRaisesRegex(ValueError, 'duplicate region field'):
            gen.extract(modified)

    def test_missing_world_defaults_are_rejected(self):
        for field in ['firstDay', 'weekendStart', 'weekendEnd']:
            modified = copy.deepcopy(self.xml)
            for row in modified.findall('weekData/' + field):
                row.set('territories', ' '.join(r for r in row.get('territories').split() if r != '001'))
            with self.assertRaisesRegex(ValueError, 'missing 001 default'):
                gen.extract(modified)

    def test_unknown_weekday_and_partial_day_boundaries_are_rejected(self):
        modified = copy.deepcopy(self.xml)
        modified.find('weekData/weekendStart').set('day', 'funday')
        with self.assertRaisesRegex(ValueError, 'unknown weekday'):
            gen.extract(modified)
        modified = copy.deepcopy(self.xml)
        modified.find('weekData/weekendStart').set('time', '12:00')
        with self.assertRaisesRegex(ValueError, 'partial-day weekend'):
            gen.extract(modified)

    def test_profile_bytes_reproduce_and_have_only_two_week_info_fields(self):
        first = gen.outputs(PRIMARY_ROOT)
        self.assertEqual(first, gen.outputs(PRIMARY_ROOT))
        profile = json.loads(first[gen.DEST / 'profile.json'])
        for row in profile['regions']:
            self.assertEqual(set(row), {'region', 'first_day', 'weekend_mask'})
            self.assertNotIn('minimalDays', row)
        provenance = json.loads(first[gen.DEST / 'provenance.json'])
        self.assertEqual(provenance['profile_sha256'], gen.sha(first[gen.DEST / 'profile.json']))
        self.assertEqual(provenance['effective_region_rows'], len(profile['regions']))

    def test_kernel_identity_changes_for_production_while_data_remains_exact(self):
        original = gen.outputs(PRIMARY_ROOT)
        with tempfile.TemporaryDirectory() as directory:
            native_root = Path(directory)
            for path in gen.NATIVE_PRODUCTION:
                (native_root / path).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(SCRIPT.parents[1] / path, native_root / path)
            record = native_root / gen.NATIVE / 'record.rs'
            record.write_text(record.read_text() + '\n// separate production source byte\n')
            changed = gen.outputs(PRIMARY_ROOT, native_root)
            self.assertEqual(original[gen.DEST / 'profile.json'], changed[gen.DEST / 'profile.json'])
            old = json.loads(original[gen.DEST / 'provenance.json'])
            new = json.loads(changed[gen.DEST / 'provenance.json'])
            self.assertEqual(old['profile_sha256'], new['profile_sha256'])
            self.assertNotEqual(old['kernel_sha256'], new['kernel_sha256'])

    def test_missing_and_added_production_reachability_rejects(self):
        with tempfile.TemporaryDirectory() as directory:
            native_root = Path(directory)
            for path in gen.NATIVE_PRODUCTION:
                (native_root / path).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(SCRIPT.parents[1] / path, native_root / path)
            parent = native_root / gen.NATIVE.with_suffix('.rs')
            parent.write_text(parent.read_text() + '\npub(crate) mod unbound;\n')
            with self.assertRaisesRegex(ValueError, 'module closure changed'):
                gen.outputs(PRIMARY_ROOT, native_root)
            shutil.copyfile(SCRIPT.parents[1] / gen.NATIVE.with_suffix('.rs'), parent)
            (native_root / gen.NATIVE / 'unbound.rs').write_text('pub fn unbound() {}\n')
            with self.assertRaisesRegex(ValueError, 'source reachability'):
                gen.outputs(PRIMARY_ROOT, native_root)


if __name__ == '__main__':
    unittest.main()
