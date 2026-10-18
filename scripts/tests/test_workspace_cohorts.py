"""Coverage and destructive-boundary controls for bounded native cohorts."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('cohorts', Path(__file__).resolve().parents[1] / 'run_workspace_cohorts.py')
cohorts = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cohorts)


def case(package, target, kind='test'):
    return {'package': package, 'target': target, 'kind': kind,
            'selector': ['--lib'] if kind == 'lib' else ['--' + kind, target]}


class CohortControls(unittest.TestCase):
    def test_same_selector_siblings_are_never_split_and_full_inventory_is_retained(self):
        cases = [case('a', 'a', 'lib'), case('b', 'b', 'lib'),
                 case('a', 'one'), case('a', 'shared'), case('b', 'shared'),
                 case('a', 'two'), case('intl', 'intl', 'lib'), case('intl', 'shared'),
                 case('a', 'doc', 'doc'), case('intl', 'doc', 'doc')]
        groups = cohorts.cohorts(cases, ['intl'], 2)
        self.assertEqual(sorted(cohorts.key(c) for g in groups for c in g),
                         sorted(map(cohorts.key, cases)))
        shared = [g for g in groups if any(c['package'] != 'intl' and c['target'] == 'shared' for c in g)]
        self.assertEqual(len(shared), 1)
        self.assertEqual({c['package'] for c in shared[0]}, {'a', 'b'})
        self.assertTrue(all(len({c['package'] == 'intl' for c in g}) == 1 for g in groups))

    def test_duplicate_inventory_and_oversized_same_selector_group_reject(self):
        for rows, size in [([case('a', 'shared')] * 2, 8),
                           ([case('a', 'shared'), case('b', 'shared')], 1)]:
            with self.assertRaises(ValueError):
                cohorts.cohorts(rows, [], size)

    def fixture(self, root):
        profile = root / 'target/debug'
        (profile / 'deps').mkdir(parents=True)
        artifact = profile / 'deps/test-123'
        artifact.write_bytes(b'native executable bytes')
        inventory = root / 'inventory'
        inventory.write_bytes(b'test_one: test\n')
        transcript = root / 'transcript'
        transcript.write_bytes(b'test test_one ... ok\n')
        selected = case('a', 'test')
        row = dict(selected, state='passed', inventory_matches=True, exit_code=0,
                   inventory_log='inventory', inventory_sha256=cohorts.sha(inventory),
                   transcript_log='transcript', transcript_sha256=cohorts.sha(transcript),
                   executable_sha256=cohorts.sha(artifact), executable_bytes=artifact.stat().st_size,
                   build_group='selected-native:*', package_id='exact-package-id')
        receipt = {'target_directory': str(root / 'target'), 'profile': 'debug',
                   'cases': [row], 'build_groups': {'selected-native:*': {'executables': [
                       dict(selected, package_id='exact-package-id', path=str(artifact))]}}}
        return selected, receipt, artifact

    def test_retirement_is_sealed_first_and_preserves_evidence_and_library_siblings(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            selected, receipt, artifact = self.fixture(root)
            library = artifact.with_name('library-456')
            library.write_bytes(b'preserve library')
            calls = []

            def record(rows, state):
                calls.append((state, artifact.exists(), list(rows)))

            cohorts.retire_integrations(root, receipt, [selected], record)
            self.assertEqual(calls[0][:2], ('sealed-before-unlink', True))
            self.assertEqual(calls[-1][:2], ('complete', False))
            self.assertTrue(library.exists())
            self.assertTrue((root / 'inventory').exists())
            self.assertTrue((root / 'transcript').exists())

    def test_changed_evidence_failed_scope_wrong_package_and_wrong_profile_never_delete(self):
        for defect in ['inventory', 'transcript', 'artifact', 'failed', 'package', 'profile']:
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                selected, receipt, artifact = self.fixture(root)
                if defect in ['inventory', 'transcript']:
                    (root / defect).write_bytes(b'changed')
                elif defect == 'artifact':
                    artifact.write_bytes(b'changed')
                elif defect == 'failed':
                    receipt['cases'][0]['state'] = 'failed'
                elif defect == 'package':
                    receipt['cases'][0]['package_id'] = 'other-package'
                else:
                    receipt['profile'] = 'release'
                with self.assertRaises(ValueError):
                    cohorts.retire_integrations(root, receipt, [selected], lambda *_: None)
                self.assertTrue(artifact.exists())


if __name__ == '__main__':
    unittest.main()
