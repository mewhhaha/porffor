"""Source identity invariants using copies of the actual captured input closure."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('service_identity', ROOT / 'scripts/generate-intl-display-relative-identity.py')
IDENTITY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(IDENTITY)


class ConsumedKernelIdentity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.actual = IDENTITY.generate(ROOT)
        cls.recipes = {json.loads(text)['service']: json.loads(text)
                       for path, text in cls.actual.items() if path.endswith('.json')}

    def copy_inputs(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        paths = {row['path'] for recipe in self.recipes.values() for row in recipe['files']}
        paths.add('Cargo.lock')
        for relative in paths:
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((ROOT / relative).read_bytes())
        return root

    def test_real_components_bind_profiles_codecs_actual_hosts_and_source_authorities(self):
        for service, owner, host in [('DisplayNames', 'display_names', 'intl_display_names_host'),
                                     ('RelativeTimeFormat', 'relative_time_format', 'intl_relative_time_host')]:
            recipe = self.recipes[service]
            paths = {row['path'] for row in recipe['files']}
            self.assertIn(f'crates/lila-intl/src/{owner}.rs', paths)
            self.assertIn(f'crates/lila-engine/src/{host}.rs', paths)
            image = 'display_names_image' if service == 'DisplayNames' else 'relative_time_image'
            self.assertIn(f'crates/lila-intl/src/{image}.rs', paths)
            self.assertIn('crates/lila-intl/src/locale_image.rs', paths)
            self.assertIn('crates/lila-engine/src/intl_data_images.rs', paths)
            self.assertIn('crates/lila-engine/src/wasm_gc_intl_host.rs', paths)
            self.assertIn('crates/lila-intl/src/supported_values.rs', paths)
            self.assertIn('scripts/intl_calendar_eras.py', paths)
            self.assertTrue(any('calendar-eras-cldr-48/common/main' in path for path in paths))
            self.assertTrue(any('numbering-tols-cldr-48' in path for path in paths))
            self.assertFalse(any('/tests/' in path or path.endswith('/tests.rs') or path.endswith('/kernel_identity.rs') for path in paths))
        self.assertIn('crates/lila-intl/src/number_format/numeric/round.rs',
                      {row['path'] for row in self.recipes['RelativeTimeFormat']['files']})
        self.assertNotIn('ryu-js', {row['name'] for row in self.recipes['DisplayNames']['packages']})
        formatter = next(row for row in self.recipes['RelativeTimeFormat']['packages'] if row['name'] == 'ryu-js')
        self.assertEqual(formatter['version'], '1.0.2')
        self.assertEqual(formatter['source'], 'registry+https://github.com/rust-lang/crates.io-index')
        self.assertEqual(formatter['checksum'], 'dd29631678d6fb0903b69223673e122c32e9ae559d0960a38d574695ebc0ea15')

    def test_relative_finite_formatter_package_invalidates_only_consuming_component(self):
        root = self.copy_inputs()
        lock = root / 'Cargo.lock'
        original = lock.read_text()
        checksum = 'dd29631678d6fb0903b69223673e122c32e9ae559d0960a38d574695ebc0ea15'
        self.assertEqual(original.count(checksum), 1)
        lock.write_text(original.replace(checksum, '0' * 64))
        changed = IDENTITY.generate(root)
        dn = 'crates/lila-intl/src/display_names/kernel_identity.rs'
        rtf = 'crates/lila-intl/src/relative_time_format/kernel_identity.rs'
        self.assertEqual(changed[dn], self.actual[dn])
        self.assertNotEqual(changed[rtf], self.actual[rtf])
        lock.write_text(original.replace('name = "ryu-js"', 'name = "missing-relative-formatter"'))
        with self.assertRaisesRegex(ValueError, 'wrong locked dependency: ryu-js'):
            IDENTITY.generate(root)

    def test_relative_rounding_owner_change_invalidates_only_relative_component(self):
        root = self.copy_inputs()
        target = root / 'crates/lila-intl/src/number_format/numeric/round.rs'
        target.write_bytes(target.read_bytes() + b'\n// deliberate source-identity tamper\n')
        changed = IDENTITY.generate(root)
        dn = 'crates/lila-intl/src/display_names/kernel_identity.rs'
        rtf = 'crates/lila-intl/src/relative_time_format/kernel_identity.rs'
        self.assertEqual(changed[dn], self.actual[dn])
        self.assertNotEqual(changed[rtf], self.actual[rtf])

    def test_shared_cardinal_algorithm_change_invalidates_relative_component(self):
        root = self.copy_inputs()
        target = root / 'crates/lila-intl/src/plural_rules/rules.rs'
        target.write_bytes(target.read_bytes() + b'\n// deliberate cardinal-owner tamper\n')
        changed = IDENTITY.generate(root)
        rtf = 'crates/lila-intl/src/relative_time_format/kernel_identity.rs'
        self.assertNotEqual(changed[rtf], self.actual[rtf])

    def test_ignored_test_files_and_self_outputs_do_not_enter_identity(self):
        root = self.copy_inputs()
        for relative in ['crates/lila-intl/src/display_names/tests.rs',
                         'crates/lila-intl/src/relative_time_format/tests.rs',
                         'crates/lila-intl/src/display_names/kernel_identity.rs']:
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text('deliberately unrelated test or generated self bytes\n')
        self.assertEqual(IDENTITY.generate(root), self.actual)

    def test_tampered_real_captured_input_is_rejected_before_identity_generation(self):
        root = self.copy_inputs()
        target = root / 'crates/lila-intl/data/datetime-cldr-47/common/main/en.xml'
        target.write_bytes(target.read_bytes() + b'\n')
        with self.assertRaisesRegex(ValueError, 'captured input changed'):
            IDENTITY.generate(root)

    def test_missing_actual_host_consumer_is_rejected(self):
        root = self.copy_inputs()
        (root / 'crates/lila-engine/src/intl_relative_time_host.rs').unlink()
        with self.assertRaises(FileNotFoundError):
            IDENTITY.generate(root)


if __name__ == '__main__':
    unittest.main()
