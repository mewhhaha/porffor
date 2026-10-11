import importlib.util
from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
sys.path.insert(0, str(ROOT / 'scripts/intl_numberformat_profile'))
from cldr_resolver import CldrResolver

spec = importlib.util.spec_from_file_location(
    'numberformat_extraction', ROOT / 'scripts/intl_numberformat_profile/extract_profiles.py')
extraction = importlib.util.module_from_spec(spec)
spec.loader.exec_module(extraction)


def extractor(digits):
    owner = extraction.Extractor.__new__(extraction.Extractor)
    owner.pools = {name: extraction.Pool() for name in
                   ('strings', 'patterns', 'signed_patterns', 'unicode_sets')}
    owner.unicode_properties = {'Nd': digits}
    return owner


def resolver():
    owner = CldrResolver.__new__(CldrResolver)
    owner.locales = {'en', 'root'}
    owner.default_content = {'en_US'}
    owner.parents = {'en_US': 'en', 'en': 'root'}
    owner.lineages = {}
    return owner


class NumberFormatMemoization(unittest.TestCase):
    def test_interned_ids_and_unicode_tables_belong_to_each_extractor(self):
        ascii_owner = extractor([(0x30, 0x39)])
        tols_owner = extractor([(0x11DE0, 0x11DE9)])
        ascii_owner.string('prefix')
        self.assertEqual(ascii_owner.string('shared'), 1)
        self.assertEqual(tols_owner.string('shared'), 0)
        ascii_set = ascii_owner.unicode_set('[:Nd:]')
        tols_set = tols_owner.unicode_set('[:Nd:]')
        self.assertEqual(ascii_owner.pools['unicode_sets'].rows[ascii_set], [(0x30, 0x39)])
        self.assertEqual(tols_owner.pools['unicode_sets'].rows[tols_set], [(0x11DE0, 0x11DE9)])

    def test_compact_and_literal_pattern_policies_keep_distinct_tokens(self):
        owner = extractor([])
        literal = owner.signed('0K')
        compact = owner.signed('0K', compact=True)
        self.assertNotEqual(literal, compact)
        rows = owner.pools['signed_patterns'].rows
        patterns = owner.pools['patterns'].rows
        self.assertIn(('Literal', owner.string('K')), patterns[rows[literal]['positive']])
        self.assertIn(('Compact', owner.string('K')), patterns[rows[compact]['positive']])

    def test_partial_lineage_cannot_hide_an_unvalidated_parent_cycle(self):
        owner = resolver()
        partial = owner.lineage('en_US')
        self.assertEqual(next(partial), 'en')
        owner.parents['en'] = 'en_US'
        with self.assertRaisesRegex(ValueError, 'cyclic parent chain'):
            tuple(owner.lineage('en_US'))
        partial.close()

    def test_failed_lineage_is_not_reused_by_a_later_valid_inventory(self):
        owner = resolver()
        owner.parents['en'] = 'missing'
        with self.assertRaisesRegex(ValueError, 'missing required parent source'):
            tuple(owner.lineage('en_US'))
        owner.parents['en'] = 'root'
        self.assertEqual(tuple(owner.lineage('en_US')), ('en', 'root'))


if __name__ == '__main__':
    unittest.main()
