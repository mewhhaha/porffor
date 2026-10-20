import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('collator_projection', ROOT / 'scripts/generate-intl-collator-data.py')
PRODUCER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PRODUCER)


class DocumentationProjection(unittest.TestCase):
    def fixture(self, root):
        (root / 'data').mkdir()
        (root / 'generated').mkdir()
        raw = PRODUCER.SCHEMATIC_EXAMPLE.encode()
        (root / 'data/exported-mod.rs').write_bytes(raw)
        (root / 'generated/mod.rs').write_bytes(PRODUCER.normalize_provider_documentation(raw))
        (root / 'generated/data.rs').write_bytes(b'original ICU macro/data bytes')
        row = lambda name, data: {'path': name, 'bytes': len(data), 'sha256': PRODUCER.sha(data)}
        exported = [row('mod.rs', raw), row('data.rs', (root / 'generated/data.rs').read_bytes())]
        inputs = {'documentation_projection': {'schema_version': 1, 'policy': 'exact provider example documentation only',
                  'raw_mod': row('exported-mod.rs', raw)}, 'generated_outputs': exported}
        return inputs, {'generated_files': exported}

    def test_refresh_admits_only_the_projected_documentation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            inputs, exported = self.fixture(root)
            with patch.multiple(PRODUCER, ROOT=root, DATA=Path('data'), EXPORT_OWNER=Path('.')):
                with self.assertRaisesRegex(ValueError, 'projection inventory mismatch'):
                    PRODUCER.consumed_export(inputs, exported)
                PRODUCER.consumed_export(inputs, exported, refresh_documentation=True)
                PRODUCER.consumed_export(inputs, exported)
                self.assertNotEqual(inputs['generated_outputs'][0], exported['generated_files'][0])
                self.assertEqual(inputs['generated_outputs'][1], exported['generated_files'][1])

    def test_changed_actual_data_is_rejected_even_during_refresh(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            inputs, exported = self.fixture(root)
            (root / 'generated/data.rs').write_bytes(b'changed ICU data')
            with patch.multiple(PRODUCER, ROOT=root, DATA=Path('data'), EXPORT_OWNER=Path('.')):
                with self.assertRaisesRegex(ValueError, 'generated source identity mismatch'):
                    PRODUCER.consumed_export(inputs, exported, refresh_documentation=True)
