"""Isolated checkout ownership and generator recovery entry-point controls."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), ROOT / 'scripts' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class IntlGenerationPublication(unittest.TestCase):
    def test_collator_explicit_root_binds_only_that_checkout_and_rejects_owner_changes(self):
        producer = load('generate-intl-collator-data')
        manifest = json.loads((ROOT / producer.DATA / 'manifest.json').read_bytes())
        owners = {row['path'] for row in manifest['owners']} | {
            'Cargo.lock', 'crates/scriptw-intl/src/collator_markers.rs',
            'crates/scriptw-intl/src/image_build/export_contract.rs',
            'scripts/generated_output_transaction.py',
            str(producer.DATA / 'source-inputs.json')}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in owners:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes((ROOT / name).read_bytes())
            inputs = json.loads((root / producer.DATA / 'source-inputs.json').read_bytes())
            expected = producer.result(inputs, root=ROOT)
            self.assertEqual(producer.result(inputs, root=root), expected)
            helper = root / 'scripts/generated_output_transaction.py'
            original = helper.read_bytes()
            helper.write_bytes(original + b'\n# isolated owner mutation\n')
            self.assertNotEqual(producer.result(inputs, root=root), expected)
            helper.unlink()
            with self.assertRaises(FileNotFoundError):
                producer.result(inputs, root=root)
            helper.write_bytes(original)
            unknown = root / 'crates/scriptw-intl/src/collator_image/unknown.rs'
            unknown.write_text('// unreviewed owner')
            with self.assertRaisesRegex(ValueError, 'unreviewed Collator projection'):
                producer.result(inputs, root=root)

    def test_all_generator_checks_reject_pending_recovery_before_reading_inputs(self):
        transaction = load('generated_output_transaction')
        for script, owner in [('generate-intl-collator-data', 'intl-collator'),
                              ('generate-intl-locale-collations-identity', 'intl-locale-collations'),
                              ('generate-intl-segmenter-identity', 'intl-segmenter')]:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                def interrupted(_):
                    raise RuntimeError('interrupted')
                with self.assertRaises(RuntimeError):
                    transaction.publish_outputs(root, owner, {'first': b'one', 'second': b'two'}, after_replace=interrupted)
                before = {p: p.read_bytes() for p in root.rglob('*') if p.is_file()}
                result = subprocess.run([sys.executable, str(ROOT / 'scripts' / (script + '.py')), '--root', str(root), '--check'], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('pending generated-output recovery', result.stderr)
                self.assertEqual(before, {p: p.read_bytes() for p in root.rglob('*') if p.is_file()})


if __name__ == '__main__':
    unittest.main()
