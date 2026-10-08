"""Real pinned country membership and closed source-identity checks; no product execution."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('locale_time_zone_profile', ROOT / 'scripts/generate-intl-locale-time-zones-profile.py')
GEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GEN)


class CountryMembership(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.zone, cls.catalogue_raw, cls.pairs = GEN.admitted_inputs(ROOT)
        cls.catalogue = GEN.primary_catalogue(cls.catalogue_raw)

    def test_actual_archive_member_and_closed_catalogue_are_admitted(self):
        self.assertEqual(GEN.sha(self.zone), GEN.ZONE_TAB_SHA256)
        self.assertEqual(GEN.sha(self.catalogue_raw), GEN.CATALOGUE_SHA256)
        self.assertEqual(len(self.catalogue), 598)

    def test_complete_country_projection_has_exact_regions_and_no_duplicate_pairs(self):
        self.assertEqual(len({country for country, _ in self.pairs}), 247)
        self.assertEqual(len(self.pairs), len(set(self.pairs)))
        self.assertEqual(self.pairs, sorted(self.pairs))
        self.assertNotIn('001', {country for country, _ in self.pairs})
        self.assertNotIn('ZZ', {country for country, _ in self.pairs})

    def test_country_primary_exceptions_are_preserved(self):
        for pair in [('SK', 'Europe/Bratislava'), ('CZ', 'Europe/Prague'),
                     ('DE', 'Europe/Busingen'), ('AX', 'Europe/Mariehamn')]:
            self.assertIn(pair, self.pairs)
        self.assertNotIn(('SK', 'Europe/Prague'), self.pairs)

    def test_links_project_to_actual_terminal_catalogue_primary(self):
        self.assertIn(('IN', 'Asia/Kolkata'), self.pairs)
        self.assertNotIn(('IN', 'Asia/Calcutta'), self.pairs)
        self.assertIn(('UA', 'Europe/Kyiv'), self.pairs)
        self.assertTrue(all(self.catalogue.get(name) == name for _, name in self.pairs))

    def test_single_country_table_does_not_merge_samoa_membership(self):
        self.assertEqual([name for country, name in self.pairs if country == 'AS'], ['Pacific/Pago_Pago'])
        self.assertEqual([name for country, name in self.pairs if country == 'WS'], ['Pacific/Apia'])

    def test_missing_country_row_is_rejected(self):
        lines = self.zone.splitlines(keepends=True)
        index = next(i for i, line in enumerate(lines) if line.startswith(b'AD\t'))
        with self.assertRaisesRegex(ValueError, 'incomplete'):
            GEN.memberships(b''.join(lines[:index] + lines[index+1:]), self.catalogue)

    def test_duplicate_country_row_is_rejected(self):
        line = next(line for line in self.zone.splitlines(keepends=True) if line.startswith(b'AD\t'))
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            GEN.memberships(self.zone + line, self.catalogue)

    def test_missing_named_catalogue_identity_is_rejected(self):
        catalogue = dict(self.catalogue)
        del catalogue['Europe/Andorra']
        with self.assertRaisesRegex(ValueError, 'absent'):
            GEN.memberships(self.zone, catalogue)

    def test_nonterminal_primary_is_rejected(self):
        catalogue = dict(self.catalogue)
        catalogue['Europe/Andorra'] = 'Europe/Missing'
        with self.assertRaisesRegex(ValueError, 'non-terminal'):
            GEN.memberships(self.zone, catalogue)

    def test_multi_country_and_invalid_coordinate_rows_are_rejected(self):
        for row in [b'AD,AE\t+4230+00131\tEurope/Andorra\n', b'AD\tbad\tEurope/Andorra\n']:
            with self.assertRaisesRegex(ValueError, 'malformed'):
                GEN.memberships(row, self.catalogue)

    def test_unknown_native_production_leaf_is_rejected(self):
        profile = GEN.outputs(ROOT)[GEN.PROFILE_IDENTITY]
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for row in GEN.kernel_sources(ROOT, profile):
                path = root / row['path']
                path.parent.mkdir(parents=True, exist_ok=True)
                if Path(row['path']) != GEN.PROFILE_IDENTITY:
                    path.write_bytes((ROOT / row['path']).read_bytes())
            unexpected = root / GEN.NATIVE.with_suffix('') / 'unchecked.rs'
            unexpected.parent.mkdir(parents=True, exist_ok=True)
            unexpected.write_text('pub fn bypass() {}\n')
            with self.assertRaisesRegex(ValueError, 'unreviewed Locale'):
                GEN.kernel_sources(root, profile)

    def test_all_six_genuine_outputs_match_current_source(self):
        outputs = GEN.outputs(ROOT)
        self.assertEqual(len(outputs), 6)
        for name, raw in outputs.items():
            self.assertEqual((ROOT / name).read_bytes(), raw)
        manifest = json.loads(outputs[GEN.DATA / 'manifest.json'])
        self.assertFalse(manifest['region_inference'])
        self.assertFalse(manifest['rg_sd_overrides'])


if __name__ == '__main__':
    unittest.main()
