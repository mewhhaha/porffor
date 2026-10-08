import importlib.util,json,shutil,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('duration_identity',ROOT/'scripts/generate-intl-duration-identity.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
class DurationIdentityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.outputs=module.generate(ROOT)
  cls.manifest=json.loads(next(raw for name,raw in cls.outputs.items() if name.endswith('.json')))
 def tree(self,directory):
  root=Path(directory)
  for row in self.manifest['files']:
   dst=root/row['path'];dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/row['path'],dst)
  shutil.copyfile(ROOT/'Cargo.lock',root/'Cargo.lock');return root
 def changed(self,path):
  with tempfile.TemporaryDirectory() as directory:
   root=self.tree(directory);source=root/path;source.write_bytes(source.read_bytes()+b'\n// independently altered production source\n')
   self.assertNotEqual(module.generate(root),self.outputs)
 def test_checked_integer_arithmetic_is_bound(self):self.changed('crates/lila-intl/src/duration_format/record.rs')
 def test_completed_primitive_wire_and_actual_host_consumers_are_bound(self):
  for path in ('crates/lila-intl/src/duration_wire/configuration.rs','crates/lila-intl/src/duration_wire/requests.rs','crates/lila-intl/src/duration_wire/responses.rs','crates/lila-engine/src/intl_duration_host.rs'):
   with self.subTest(path=path):self.changed(path)
 def test_installed_global_provider_admission_is_bound(self):
  # Duration shares ABI17, including the typed Locale information lists.
  self.assertEqual(self.manifest['host_call_abi'],17)
  self.assertFalse(self.manifest['foundation_only'])
  self.assertEqual(self.manifest['global_operations'],[36,37,38])
  for path in ('crates/lila-intl/src/provider.rs','crates/lila-intl/src/protocol.rs','crates/lila-intl/src/supported_values.rs','crates/lila-intl/src/duration_image.rs','crates/lila-intl/src/number_image.rs','crates/lila-intl/src/list_image.rs','crates/lila-engine/src/intl_data_images.rs','crates/lila-engine/src/wasm_gc_intl_host.rs'):
   with self.subTest(path=path):self.changed(path)
 def test_shared_numeric_and_cardinal_owners_are_bound(self):
  for path in ('crates/lila-intl/src/number_format/mod.rs','crates/lila-intl/src/number_format/numeric/round.rs'):
   with self.subTest(path=path):self.changed(path)
 def test_real_list_partition_algorithm_is_bound(self):self.changed('crates/lila-intl/src/list_format/partition.rs')
 def test_captured_primary_source_tamper_rejects(self):
  with tempfile.TemporaryDirectory() as directory:
   root=self.tree(directory);source=root/'crates/lila-intl/data/duration-cldr-47/common/main/sr.xml';source.write_bytes(source.read_bytes()+b'\n')
   with self.assertRaisesRegex(ValueError,'captured Duration input changed'):module.generate(root)
 def test_genuine_locked_dependency_checksums_are_bound(self):
  with tempfile.TemporaryDirectory() as directory:
   root=self.tree(directory);lock=root/'Cargo.lock';text=lock.read_text();checksum=next(row['checksum'] for row in self.manifest['packages'] if row['name']=='icu_list');lock.write_text(text.replace(checksum,'0'*64));self.assertNotEqual(module.generate(root),self.outputs)
 def test_missing_or_wrong_locked_version_rejects(self):
  with tempfile.TemporaryDirectory() as directory:
   root=self.tree(directory);lock=root/'Cargo.lock';text=lock.read_text();lock.write_text(text.replace('name = "icu_list"\nversion = "2.0.1"','name = "icu_list"\nversion = "2.0.9"'))
   with self.assertRaisesRegex(ValueError,'wrong consumed package'):module.generate(root)
 def test_tests_and_self_identity_are_excluded(self):
  with tempfile.TemporaryDirectory() as directory:
   root=self.tree(directory)
   for name in ('tests.rs','profile_identity.rs'):
    path=root/'crates/lila-intl/src/duration_format'/name;path.write_text('deliberately irrelevant bytes')
   self.assertEqual(module.generate(root),self.outputs)
if __name__=='__main__':unittest.main()
