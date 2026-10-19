"""The source proof must retain version and body-validation authority."""
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location(
    'validated_profile', ROOT / 'scripts/check-abi8-validated-runtime-profile.py')
controls = importlib.util.module_from_spec(spec)
spec.loader.exec_module(controls)


class ValidatedEnvelopeControls(unittest.TestCase):
    def test_actual_version_and_external_body_validation_cannot_be_removed(self):
        sources = {name: (ROOT / name).read_text() for name in controls.FILES}
        self.assertEqual(controls.check(sources), [])
        for path, before, after in [
            (controls.A + 'runtime_artifact/cache.rs',
             '        version,\n        jobs,\n        super::validation::FunctionBodyValidation::AtUse,',
             '        version,\n        jobs,\n        super::validation::FunctionBodyValidation::ValidatedAtBuild,'),
            (controls.A + 'runtime_artifact/build_package/envelope.rs',
             'cache::decode_payload(payload, identity, version, jobs, body_validation)?',
             'cache::decode_payload(payload, identity, version, jobs, FunctionBodyValidation::AtUse)?'),
            (controls.A + 'runtime_artifact/build_package/envelope.rs',
             'CurrentRuntimeLayoutVersion::admit(u32::from_le_bytes(cache::take(&mut payload)?))?',
             'CurrentRuntimeLayoutVersion::emitted_current()'),
        ]:
            with self.subTest(path=path, before=before):
                self.assertEqual(sources[path].count(before), 1)
                changed = dict(sources)
                changed[path] = sources[path].replace(before, after)
                failures = {row['invariant'] for row in controls.check(changed)}
                self.assertIn('checked_envelopes', failures)
