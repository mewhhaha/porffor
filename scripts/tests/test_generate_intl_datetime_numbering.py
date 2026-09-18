import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import xml.etree.ElementTree as ET


SCRIPT = Path(__file__).resolve().parents[1] / "generate-intl-datetime-numbering.py"
SPEC = importlib.util.spec_from_file_location("generate_intl_datetime_numbering", SCRIPT)
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)
SOURCE = SCRIPT.parent.parent / GENERATOR.SOURCE_PATH


class DateTimeNumberingGenerationTests(unittest.TestCase):
    def test_pinned_sources_regenerate_the_complete_positional_domain(self):
        generated, report = GENERATOR.generate(SOURCE)
        self.assertEqual(
            (SCRIPT.parent.parent / GENERATOR.OUTPUT_PATH).read_text(), generated
        )
        report = json.loads(report)
        self.assertEqual(report["positional_numbering_systems"], 77)
        self.assertEqual(report["default_numbering_system"], "latn")
        self.assertIn("roman", report["excluded_algorithmic_numbering_systems"])
        self.assertIn('"hanidec", "〇一二三四五六七八九", "."', generated)
        self.assertIn('"arab", "٠١٢٣٤٥٦٧٨٩", "٫"', generated)
        self.assertIn('"mathbold", "𝟎𝟏𝟐𝟑𝟒𝟓𝟔𝟕𝟖𝟗", "."', generated)

    def rewrite_pinned_xml(self, source, relative, edit):
        path = source / relative
        tree = ET.parse(path)
        edit(tree.getroot())
        tree.write(path, encoding="utf-8")
        manifest_path = source / "manifest.json"
        manifest = json.loads(manifest_path.read_text())
        entry = next(row for row in manifest["files"] if row["path"] == relative)
        entry["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        manifest_path.write_text(json.dumps(manifest))

    def test_changed_unpinned_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "cldr"
            shutil.copytree(SOURCE, source)
            path = source / "common/supplemental/numberingSystems.xml"
            path.write_text(path.read_text() + "\n")
            with self.assertRaisesRegex(ValueError, "source checksum mismatch"):
                GENERATOR.generate(source)

    def test_new_variable_width_digits_require_a_renderer_change(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "cldr"
            shutil.copytree(SOURCE, source)

            def variable_width(root):
                root.find("./numberingSystems/numberingSystem[@id='latn']").set(
                    "digits", "０123456789"
                )

            self.rewrite_pinned_xml(
                source, "common/supplemental/numberingSystems.xml", variable_width
            )
            with self.assertRaisesRegex(ValueError, "variable UTF-8 digit widths"):
                GENERATOR.generate(source)

    def test_missing_decimal_symbols_do_not_silently_fall_back(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "cldr"
            shutil.copytree(SOURCE, source)

            def remove_arabic_decimal(root):
                symbols = root.find("./numbers/symbols[@numberSystem='arab']")
                symbols.remove(symbols.find("decimal"))

            self.rewrite_pinned_xml(source, "common/main/root.xml", remove_arabic_decimal)
            with self.assertRaisesRegex(ValueError, "missing decimal separator: arab"):
                GENERATOR.generate(source)


if __name__ == "__main__":
    unittest.main()
