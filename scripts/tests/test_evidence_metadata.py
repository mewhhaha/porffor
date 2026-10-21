import json
from pathlib import Path
import sys
import tempfile
import tracemalloc
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from evidence_metadata import inspect_metadata


class EvidenceMetadataTests(unittest.TestCase):
    def test_skips_nested_embedded_payloads_with_bounded_memory(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "evidence.json"
            path.write_text(
                json.dumps(
                    {
                        "head": "source",
                        "payload": {"bytes": 'escaped \\" } [ ' * 300000},
                        "total": 17,
                    }
                )
            )
            tracemalloc.start()
            try:
                self.assertEqual(
                    inspect_metadata(path, ["head", "total"]),
                    {"head": "source", "total": 17},
                )
                _, peak = tracemalloc.get_traced_memory()
            finally:
                tracemalloc.stop()
            self.assertLess(peak, 1024 * 1024)

    def test_rejects_truncated_duplicate_unbalanced_and_oversized_selected_fields(self):
        cases = [
            '{"head":',
            '{"head":1,"head":2}',
            '{"payload":[}',
            "{} trailing",
            '{"head":"' + "x" * 300 + '"}',
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "evidence.json"
            for value in cases:
                with self.subTest(value=value[:30]):
                    path.write_text(value)
                    with self.assertRaises(ValueError):
                        inspect_metadata(path, ["head"], 256)


if __name__ == "__main__":
    unittest.main()
