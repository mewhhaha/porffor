import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import json

SPEC = importlib.util.spec_from_file_location("generated_transaction", Path(__file__).resolve().parents[1] / "generated_output_transaction.py")
TX = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(TX)


class GeneratedOutputTransaction(unittest.TestCase):
    def fixture(self, root):
        outputs = {Path("data/a.json"): b"new a", Path("src/b.rs"): b"new b", Path("data/c.txt"): b"new c"}
        for name in outputs:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"old")
        return outputs

    def interrupt(self, root, outputs, index):
        def stopped(current):
            if current == index:
                raise RuntimeError("injected interruption")
        with self.assertRaisesRegex(RuntimeError, "injected interruption"):
            TX.publish_outputs(root, "fixture", outputs, after_replace=stopped)

    def test_every_interruption_recovers_the_complete_generation(self):
        for index in range(3):
            with self.subTest(index=index), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                outputs = self.fixture(root)
                self.interrupt(root, outputs, index)
                before = {p: p.read_bytes() for p in root.rglob("*") if p.is_file()}
                with self.assertRaisesRegex(ValueError, "pending generated-output recovery"):
                    TX.recover_outputs(root, "fixture", check=True)
                self.assertEqual(before, {p: p.read_bytes() for p in root.rglob("*") if p.is_file()})
                TX.recover_outputs(root, "fixture")
                self.assertEqual(outputs, {name: (root / name).read_bytes() for name in outputs})
                self.assertFalse(TX._journal(root, "fixture").exists())
                self.assertEqual(list(root.rglob(".scriptw-generated-*")), [])
                TX.recover_outputs(root, "fixture", check=True)

    def test_tampered_remaining_stage_prevents_any_more_replacement(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); outputs = self.fixture(root)
            self.interrupt(root, outputs, 0)
            rows = json.loads(TX._journal(root, "fixture").read_bytes())["members"]
            stage = root / rows[-1]["staged"]
            original = stage.read_bytes(); stage.write_bytes(b"tampered")
            before = {name: (root / name).read_bytes() for name in outputs}
            with self.assertRaisesRegex(ValueError, "recovery bytes"):
                TX.recover_outputs(root, "fixture")
            self.assertEqual(before, {name: (root / name).read_bytes() for name in outputs})
            stage.write_bytes(original); TX.recover_outputs(root, "fixture")
            self.assertEqual(outputs, {name: (root / name).read_bytes() for name in outputs})

    def test_stage_failure_keeps_old_generation_and_removes_unpublished_stages(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); outputs = self.fixture(root)
            original = TX._stage; calls = 0
            def stage(path, data):
                nonlocal calls
                calls += 1
                if calls == 2: raise OSError("disk full")
                return original(path, data)
            with patch.object(TX, "_stage", stage), self.assertRaisesRegex(OSError, "disk full"):
                TX.publish_outputs(root, "fixture", outputs)
            self.assertTrue(all((root / name).read_bytes() == b"old" for name in outputs))
            self.assertFalse(TX._journal(root, "fixture").exists())
            self.assertEqual(list(root.rglob(".scriptw-generated-*")), [])

    def test_check_without_pending_generation_creates_no_state(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            TX.recover_outputs(root, "fixture", check=True)
            self.assertEqual(list(root.iterdir()), [])

    def test_escape_symlink_and_duplicate_targets_are_rejected_before_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); self.fixture(root)
            (root / "alias").symlink_to(root / "data/a.json")
            for outputs in [{"../outside": b"bad"}, {"alias": b"bad"}, {"data/a.json": b"one", root / "data/a.json": b"two"}]:
                with self.subTest(outputs=outputs), self.assertRaises(ValueError):
                    TX.publish_outputs(root, "fixture", outputs)
                self.assertEqual((root / "data/a.json").read_bytes(), b"old")
                self.assertFalse(TX._journal(root, "fixture").exists())
                self.assertEqual(list(root.rglob(".scriptw-generated-*")), [])

    def test_publication_preserves_unchanged_member_mtime(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            outputs = self.fixture(root)
            stable = root / "data/a.json"
            stable.write_bytes(outputs[Path("data/a.json")])
            import os
            os.utime(stable, ns=(1000000000, 1000000000))
            TX.publish_outputs(root, "fixture", outputs)
            self.assertEqual(stable.stat().st_mtime_ns, 1000000000)
            self.assertEqual(outputs, {name: (root / name).read_bytes() for name in outputs})

    def test_transaction_directory_escape_and_state_collisions_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as outside:
            root = Path(directory)
            (root / "target").symlink_to(outside, target_is_directory=True)
            for check in [False, True]:
                with self.assertRaisesRegex(ValueError, "symlinks"):
                    TX.recover_outputs(root, "fixture", check=check)
            with self.assertRaisesRegex(ValueError, "symlinks"):
                TX.publish_outputs(root, "fixture", {"out": b"new"})
            self.assertEqual(list(Path(outside).iterdir()), [])
            (root / "target").unlink()
            for suffix in ["json", "lock"]:
                with self.assertRaisesRegex(ValueError, "transaction state"):
                    TX.publish_outputs(root, "fixture", {f"target/generated-output-transactions/fixture.{suffix}": b"bad"})
                self.assertFalse(TX._journal(root, "fixture").exists())


if __name__ == "__main__":
    unittest.main()
