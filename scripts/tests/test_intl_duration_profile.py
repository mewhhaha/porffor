import copy,importlib.util,json,sys,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
spec=importlib.util.spec_from_file_location('duration_export',ROOT/'scripts/generate-intl-duration-profile.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
class DurationSourceTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.profile=module.CldrProfile(ROOT/module.SOURCE)
  cls.output,cls.report=module.generate(ROOT,cls.profile)
 def test_shared_numeric_data_contains_exact_all2700_primary_unit_patterns(self):
  self.assertEqual(self.report['numeric_plural_patterns_compared_to_NF'],2700)
  self.assertEqual({r['locale'] for r in self.output['locales']},set(module.LOCALES))
  self.assertIn('es',module.LOCALES);self.assertIn('sr',module.LOCALES)
 def test_every_consumed_leaf_is_exact_primary_xml_with_its_distinguishing_key(self):
  for row in self.report['consumed_leaves'].values():
   leaf=self.profile.locales[row['source_locale']].leaves[module.parse_path(row['source_path'])]
   self.assertEqual(leaf.text,row['value']);self.assertEqual(dict(leaf.attributes),row['value_attributes'])
 def test_serbian_digital_literals_do_not_substitute_number_timeSeparator(self):
  sr=next(r for r in self.output['locales'] if r['locale']=='sr')
  self.assertEqual(sr['digital']['hms']['pattern'],'h.mm.ss')
  self.assertEqual(sr['digital']['hms']['separators'],['.','.'])
  self.assertEqual(module.digital('hh:mm:ss','hms')['widths'],[2,2,2])
 def test_unsupported_digital_fields_quotes_and_placeholder_multiplicity_reject(self):
  for value in ('h:mm:ss z',"h'X'mm",'h:m:m','h:mm:{0}'):
   with self.assertRaises(ValueError):module.digital(value,'hms')
  for value in ('{0}{0} days','{2} days','{0} {1}'):
   with self.assertRaises(ValueError):module.placeholders(value,[0],True)
 def test_pinned_source_tamper_is_rejected_before_leaf_resolution(self):
  with tempfile.TemporaryDirectory() as temp:
   directory=Path(temp)/'duration-cldr-47';directory.mkdir()
   for name,raw in self.profile.sources.items():
    path=directory/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
   (directory/'manifest.json').write_bytes(self.profile.manifest_bytes)
   # CldrProfile verifies raw checksums before loading sibling supplements.
   source=directory/'common/main/sr.xml';source.write_bytes(source.read_bytes()+b'\n')
   with self.assertRaisesRegex(ValueError,'checksum mismatch'):module.CldrProfile(directory)
 def test_unit_choice_tamper_breaks_the_primary_shared_source_contract(self):
  import gzip
  nf=json.loads(gzip.decompress((ROOT/'crates/lila-intl/data/number-cldr-47/profiles.json.gz').read_bytes()))
  actual=module.number_patterns(nf,'en','second','long')
  unit=nf['tables']['unit_sets'][nf['tables']['profiles'][nf['locales']['en']['profile']]['units'][2]]['simple'][nf['sanctioned_units'].index('second')]
  nf['tables']['choices'][unit['choices']]['values'][1]=0
  self.assertNotEqual(module.number_patterns(nf,'en','second','long'),actual)
 def test_count_other_fallback_remains_local_before_parent_and_aliases(self):
  path="units/unitLength[@type='long']/unit[@type='duration-day']/unitPattern[@count='few']"
  leaf=module.resolve_count(self.profile,'en',path)
  self.assertEqual(leaf.source_locale,'en');self.assertEqual(leaf.source_path[-1].get('count'),'other')
  self.assertEqual(leaf.value,'{0} days')
if __name__=='__main__':unittest.main()
