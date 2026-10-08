"""Source/data identity admission tests; never compiles or executes a product."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
PRIMARY = Path(os.environ.get('LILA_LOCALE_NUMBERING_SYSTEMS_PRIMARY_ROOT', str(ROOT)))
SPEC = importlib.util.spec_from_file_location('nu_identity', ROOT / 'scripts/generate-intl-locale-numbering-systems-identity.py')
GEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GEN)


class IdentityAdmission(unittest.TestCase):
    def test_genuine_two_outputs_match_sparse_source(self):
        outputs = GEN.outputs(PRIMARY, ROOT)
        self.assertEqual(len(outputs), 2)
        for name, raw in outputs.items():
            self.assertEqual((ROOT / name).read_bytes(), raw)

    def test_closed_whole_number_format_inventory_is_bound(self):
        rows = GEN.kernel_sources(PRIMARY, ROOT)
        self.assertEqual(len(rows), len(GEN.NUMBER_PRODUCTION) + len(GEN.HELPERS) + 1)
        names = {r['path'] for r in rows}
        self.assertTrue(set(GEN.NUMBER_PRODUCTION) <= names)
        self.assertIn('crates/lila-intl/src/provider.rs', names)
        self.assertIn('crates/lila-intl/src/provider/region_preference.rs', names)

    def test_unknown_native_production_module_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shutil.copytree(ROOT / 'crates', root / 'crates')
            p = root / GEN.NATIVE.with_suffix('') / 'unchecked.rs'
            p.write_text('pub fn bypass() {}\n')
            with self.assertRaisesRegex(ValueError, 'unreviewed Locale'):
                GEN.kernel_sources(PRIMARY, root)

    def test_unknown_number_format_production_module_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shutil.copytree(ROOT / 'crates', root / 'crates')
            p = root / GEN.NUMBER_ROOT / 'unchecked.rs'
            p.write_text('pub fn bypass() {}\n')
            with self.assertRaisesRegex(ValueError, 'NumberFormat production'):
                GEN.kernel_sources(PRIMARY, root)

    def test_changed_prefix_query_changes_kernel_identity_without_data_change(self):
        before = GEN.outputs(PRIMARY, ROOT)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shutil.copytree(ROOT / 'crates', root / 'crates')
            p = root / 'crates/lila-intl/src/number_format/configuration.rs'
            p.write_text(p.read_text() + '\n// changed production owner witness\n')
            after = GEN.outputs(PRIMARY, root)
            self.assertNotEqual(before[GEN.IDENTITY], after[GEN.IDENTITY])
            a, z = (json.loads(outputs[GEN.DEST]) for outputs in [before, after])
            self.assertEqual(a['data'], z['data'])
            self.assertEqual(a['NumberFormat_payload_sha256'], z['NumberFormat_payload_sha256'])

    def test_changed_shared_keyword_admission_changes_identity(self):
        before = GEN.kernel_sources(PRIMARY, ROOT)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shutil.copytree(ROOT / 'crates', root / 'crates')
            p = root / 'crates/lila-intl/src/provider/region_preference.rs'
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes((PRIMARY / 'crates/lila-intl/src/provider/region_preference.rs').read_bytes() + b'\n// changed keyword owner witness\n')
            after = GEN.kernel_sources(PRIMARY, root)
            self.assertNotEqual(before, after)

    def test_corrupt_number_payload_is_rejected_before_identity_generation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shutil.copytree(ROOT / 'crates', root / 'crates')
            p = root / GEN.DATA / 'profiles.bin'
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes((PRIMARY / GEN.DATA / 'profiles.bin').read_bytes() + b'corrupt')
            with self.assertRaisesRegex(ValueError, 'data digest differs'):
                GEN.admitted_data(PRIMARY, root)


if __name__ == '__main__':
    unittest.main()
