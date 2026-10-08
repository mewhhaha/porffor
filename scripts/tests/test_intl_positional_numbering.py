from pathlib import Path
import shutil
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from intl_positional_numbering import TolsSupplement

DATA = SCRIPTS.parent / "crates/lila-intl/data"


class TolsSourceTests(unittest.TestCase):
    def setUp(self):
        self.supplement = TolsSupplement(DATA / "numbering-tols-cldr-48")

    def test_unicode_decimal_values_and_all_actual_aliases_are_consumed(self):
        self.assertEqual(self.supplement.digits, "𑷠𑷡𑷢𑷣𑷤𑷥𑷦𑷧𑷨𑷩")
        self.assertEqual(self.supplement.decimal_ranges(), [(point, point) for point in range(0x11DE0, 0x11DEA)])
        root = ET.parse(DATA / "datetime-cldr-47/common/main/root.xml").getroot()
        original = ET.tostring(root)
        self.supplement.apply("common/main/root.xml", root)
        numbers = root.find("./numbers")
        aliases = [node for node in numbers if node.get("numberSystem") == "tols"]
        self.assertEqual(len(aliases), 6)
        for node in aliases:
            self.assertEqual(node[0].attrib, {"source": "locale", "path": f"../{node.tag}[@numberSystem='latn']"})
            numbers.remove(node)
        self.assertEqual(ET.tostring(root), original)

    def test_existing_declarations_cannot_be_replaced_by_the_supplement(self):
        path = "common/supplemental/numberingSystems.xml"
        document = ET.parse(DATA / "datetime-cldr-47" / path).getroot()
        document.find("./numberingSystems/numberingSystem[@id='latn']").set("digits", "０１２３４５６７８９")
        with self.assertRaisesRegex(ValueError, "changes an existing"):
            self.supplement.apply(path, document)

    def test_supplement_is_not_applied_twice_or_over_a_declared_system(self):
        path = "common/main/root.xml"
        document = ET.parse(DATA / "datetime-cldr-47" / path).getroot()
        self.supplement.apply(path, document)
        with self.assertRaisesRegex(ValueError, "absent root"):
            self.supplement.apply(path, document)

    def test_tampered_primary_source_cannot_mint_a_supplement(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "supplement"
            shutil.copytree(DATA / "numbering-tols-cldr-48", target)
            path = target / "ucd/UnicodeData.txt"
            path.chmod(0o600)
            path.write_bytes(path.read_bytes().replace(b"TOLONG SIKI DIGIT ZERO;Nd", b"TOLONG SIKI DIGIT ZERO;Lo"))
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                TolsSupplement(target)


if __name__ == "__main__":
    unittest.main()
