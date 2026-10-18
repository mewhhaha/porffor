"""Keep source ownership controls effective when validation passes are fused."""
import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location(
    'native_templates', ROOT / 'scripts/check-native-program-templates.py')
controls = importlib.util.module_from_spec(spec)
spec.loader.exec_module(controls)


class NativeTemplateSourceControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.sources = {name: (ROOT / name).read_text() for name in controls.FILES}

    def test_current_fused_pass_preserves_the_ownership_controls(self):
        self.assertEqual(controls.check(self.sources), [])

    def test_missing_field_local_or_plan_wiring_cannot_pass(self):
        owner = 'payload_owner_has_actual_typed_initialized_local_and_bootstrap_witness'
        wiring = 'ownership_facts_share_the_actual_planning_pass'
        mutations = [
            ('/contexts/facts.rs', '!field.mutable', 'field.mutable', owner),
            ('/contexts.rs', 'self.signatures.local(index, body, *local, self.schema)?;', '', owner),
            ('/plan.rs', 'context_facts.record_type(result.types, ty);', '', wiring),
            ('/plan.rs', 'contexts.body(', 'unvalidated_body(', wiring),
        ]
        for suffix, before, after, expected in mutations:
            with self.subTest(suffix=suffix, before=before):
                path = controls.T + suffix
                sources = dict(self.sources)
                self.assertEqual(sources[path].count(before), 1)
                sources[path] = sources[path].replace(before, after)
                self.assertIn(expected, controls.check(sources))


if __name__ == '__main__':
    unittest.main()
