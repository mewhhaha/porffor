"""Genuine Collator ancestry and closed Locale native source checks; no compiler execution."""
import importlib.util
import copy
import hashlib
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('locale_collation_identity', ROOT / 'scripts/generate-intl-locale-collations-identity.py')
GEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GEN)


def source_fixture(root):
    for item in GEN.kernel_sources(ROOT):
        path = root / item['path']
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes((ROOT / item['path']).read_bytes())
    path = root / 'scripts/generate-intl-locale-numbering-systems-identity.py'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((ROOT / path.relative_to(root)).read_bytes())


class CollationIdentity(unittest.TestCase):
    def root_evidence(self):
        with tarfile.open(ROOT / GEN.COLLATOR / 'sources.tar.gz', 'r:gz') as archive:
            raw = archive.extractfile('evidence/admission-receipt.json').read()
        return json.loads(raw), hashlib.sha256(raw).hexdigest()

    def test_exact_root_pair_has_real_historical_constructor_associations(self):
        admission, digest = self.root_evidence()
        proof = GEN.locale_independent_profile_evidence(admission, digest)
        self.assertEqual(proof['historical_admission_receipt_sha256'], digest)
        self.assertEqual([(p['locale'], p['collation'], p['constructor_configurations'])
                          for p in proof['profiles']], [('und', 'emoji', 49), ('und', 'eor', 49)])

    def test_missing_or_duplicate_required_root_profile_is_rejected(self):
        admission, digest = self.root_evidence()
        for name in ['emoji', 'eor']:
            missing = copy.deepcopy(admission)
            missing['profiles'] = [p for p in missing['profiles']
                                   if (p['locale'], p['attributes']) != ('und', name)]
            with self.assertRaisesRegex(ValueError, 'missing or duplicate'):
                GEN.locale_independent_profile_evidence(missing, digest)
            duplicate = copy.deepcopy(admission)
            duplicate['profiles'].append(next(p for p in duplicate['profiles']
                                             if (p['locale'], p['attributes']) == ('und', name)))
            with self.assertRaisesRegex(ValueError, 'missing or duplicate'):
                GEN.locale_independent_profile_evidence(duplicate, digest)

    def test_root_metadata_foreign_locale_fallback_and_incomplete_controls_are_rejected(self):
        admission, digest = self.root_evidence()
        for field, value in [('requested_locale', 'en'), ('returned_locale', 'de'),
                             ('requested_attributes', 'phonebk'), ('internal_locale_fallback', True)]:
            bad = copy.deepcopy(admission)
            root = next(p for p in bad['profiles'] if (p['locale'], p['attributes']) == ('und', 'emoji'))
            metadata = next(p for p in root['default']['loads'] if p['marker'] == 'CollationMetadataV1')
            metadata[field] = value
            with self.assertRaisesRegex(ValueError, 'foreign association'):
                GEN.locale_independent_profile_evidence(bad, digest)
        bad = copy.deepcopy(admission)
        root = next(p for p in bad['profiles'] if (p['locale'], p['attributes']) == ('und', 'eor'))
        root['option_controls'].pop()
        with self.assertRaisesRegex(ValueError, 'incomplete'):
            GEN.locale_independent_profile_evidence(bad, digest)

    def test_genuine_two_outputs_match_current_source(self):
        results = GEN.outputs(ROOT)
        self.assertEqual(len(results), 2)
        for path, raw in results.items():
            self.assertEqual((ROOT / path).read_bytes(), raw)
        recipe = json.loads(results[GEN.DATA])
        self.assertFalse(recipe['global_union_selection'])
        self.assertFalse(recipe['formatting_DefaultLocale_selection'])
        self.assertEqual(recipe['excluded'], ['standard', 'search', 'searchjl'])

    def test_actual_historical_collator_exports_and_number_payload_are_checked(self):
        rows = GEN.admitted_data(ROOT)
        names = {item['path'] for item in rows}
        self.assertIn('crates/lila-intl/data/collator-icu-2/sources.tar.gz', names)
        self.assertIn('crates/lila-intl-collator-data/src/generated/collation_tailoring_v1.rs.data', names)
        self.assertIn('crates/lila-intl/data/number-cldr-47/profiles.bin', names)

    def test_actual_profile_query_keyword_owner_and_codec_are_bound(self):
        names = {item['path'] for item in GEN.kernel_sources(ROOT)}
        for name in ['crates/lila-intl/src/collator/profiles.rs',
                     'crates/lila-intl/src/provider/region_preference.rs',
                     'crates/lila-intl/src/locale_information_wire.rs', str(GEN.NATIVE)]:
            self.assertIn(name, names)
        # The closed NumberFormat inventory also binds domains and the
        # projection, projection/closure and projection/encode owners.
        self.assertEqual(len(GEN.number_production(ROOT)), 26)

    def test_unknown_native_production_module_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            source_fixture(root)
            path = root / GEN.NATIVE.with_suffix('') / 'unchecked.rs'
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('pub fn bypass() {}\n')
            with self.assertRaisesRegex(ValueError, 'unreviewed Locale'):
                GEN.kernel_sources(root)

    def test_unknown_collator_production_module_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            source_fixture(root)
            path = root / 'crates/lila-intl/src/collator/unchecked.rs'
            path.write_text('pub fn bypass() {}\n')
            with self.assertRaisesRegex(ValueError, 'Collator production'):
                GEN.kernel_sources(root)

    def test_query_body_change_changes_identity_rows(self):
        before = GEN.kernel_sources(ROOT)
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            source_fixture(root)
            path = root / 'crates/lila-intl/src/collator/profiles.rs'
            path.write_bytes(path.read_bytes() + b'\n// changed actual per-locale selector\n')
            after = GEN.kernel_sources(root)
            self.assertNotEqual(before, after)
            changed = [left['path'] for left, right in zip(before, after) if left != right]
            self.assertEqual(changed, ['crates/lila-intl/src/collator/profiles.rs'])

    def test_wire_contract_change_changes_identity_rows(self):
        before = GEN.kernel_sources(ROOT)
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            source_fixture(root)
            path = root / 'crates/lila-intl/src/locale_information_wire.rs'
            path.write_bytes(path.read_bytes() + b'\n// changed framed response contract\n')
            after = GEN.kernel_sources(root)
            self.assertNotEqual(before, after)


if __name__ == '__main__':
    unittest.main()
