"""The source guard must retain the canonical-global bootstrap repair."""
import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "prelude_guard", ROOT / "scripts/check-independent-global-prelude.py")
GUARD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GUARD)
BOOTSTRAP_SPEC = importlib.util.spec_from_file_location(
    "bootstrap_guard", ROOT / "scripts/check-canonical-global-bootstrap.py")
BOOTSTRAP = importlib.util.module_from_spec(BOOTSTRAP_SPEC)
BOOTSTRAP_SPEC.loader.exec_module(BOOTSTRAP)
OWNER = "crates/scriptw-ir/src/lowering/global_script_prelude.rs"


class PreludeSourceGuardTests(unittest.TestCase):
    def test_current_independent_entry_and_bootstrap_owners_are_admitted(self):
        self.assertEqual(GUARD.check(ROOT), [])
        self.assertEqual(BOOTSTRAP.check(ROOT), [])

    def test_only_referenced_globals_cannot_replace_realm_creation(self):
        source = (ROOT / OWNER).read_text()
        broken = source.replace(
            "HostBuiltinId::every_realm_globals().chain(script.host_builtins.iter().copied())",
            "script.host_builtins.iter().copied()")
        self.assertNotEqual(source, broken)
        self.assertIn("separate test declaration/body owner and bootstrap host surface",
                      GUARD.check(ROOT, {OWNER: broken}))
        self.assertIn("delayed entry declarations and canonical bootstrap are independent owners",
                      BOOTSTRAP.check(ROOT, {OWNER: broken}))
