import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "check_conformance_source_pins.py"
spec = importlib.util.spec_from_file_location("conformance_source_pins", SCRIPT)
pins = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pins)


def git_id(kind, data):
    result = subprocess.run(
        ["git", "hash-object", "--stdin", "-t", kind],
        input=data, capture_output=True, check=True,
    )
    return bytes.fromhex(result.stdout.decode().strip())


class SourcePinTests(unittest.TestCase):
    def test_git_tree_order_modes_unicode_and_symlinks_match_git_objects(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "alpha").mkdir()
            (root / "alpha" / "é.js").write_bytes(b"nested\n")
            (root / "alpha.c").write_bytes(b"plain\n")
            (root / "alpha0").write_bytes(b"executable\n")
            (root / "link").write_bytes(os.fsencode("alpha/é.js"))
            (root / "empty").mkdir()
            nested = git_id("tree", b"100644 \xc3\xa9.js\0" + git_id("blob", b"nested\n"))
            expected = git_id("tree", b"".join([
                b"100644 alpha.c\0" + git_id("blob", b"plain\n"),
                b"40000 alpha\0" + nested,
                b"100755 alpha0\0" + git_id("blob", b"executable\n"),
                b"120000 link\0" + git_id("blob", os.fsencode("alpha/é.js")),
            ]))
            modes = {path: b"100644" for path in root.rglob("*") if path.is_file()}
            modes[root / "alpha0"] = b"100755"
            modes[root / "link"] = b"120000"
            self.assertEqual(pins.git_tree_id(root, modes), (expected, True))
            if os.name == "posix":
                (root / "alpha0").chmod(0o755)
                (root / "link").unlink()
                (root / "link").symlink_to("alpha/é.js")
                self.assertEqual(pins.git_tree_id(root), (expected, True))

    def test_empty_directory_is_the_known_git_empty_tree(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(pins.git_tree_id(Path(directory)), (
                bytes.fromhex("4b825dc642cb6eb9a060e54bf8d69288fbee4904"), False,
            ))

    def test_metadata_binding_rejects_changed_bytes_and_modes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            suite = root / "suite"
            suite.mkdir()
            source = suite / "test.js"
            source.write_text("262;\n")
            subprocess.run(["git", "init", "--quiet", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "suite/test.js"], check=True)
            tree, _ = pins.git_tree_id(suite)
            metadata = root / "pins.json"
            metadata.write_text(json.dumps({
                "ecma262": "a" * 40, "test262_upstream": "b" * 40,
                "test262_tree": tree.hex(),
            }))
            self.assertTrue(pins.verify(metadata, suite)["vendored_tree_verified"])
            source.write_text("263;\n")
            with self.assertRaisesRegex(ValueError, "differs from declared"):
                pins.verify(metadata, suite)
            source.write_text("262;\n")
            if os.name == "posix":
                source.chmod(0o755)
            else:
                subprocess.run(["git", "-C", str(root), "update-index", "--chmod=+x", "suite/test.js"], check=True)
            with self.assertRaisesRegex(ValueError, "differs from declared"):
                pins.verify(metadata, suite)

    def test_index_modes_bind_actual_paths_and_preserve_non_posix_executable_modes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "exec.sh"
            source.write_bytes(b"exit 0\n")
            subprocess.run(["git", "init", "--quiet", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "suite"], check=True)
            subprocess.run(["git", "-C", str(root), "update-index", "--chmod=+x", "suite/exec.sh"], check=True)
            modes = pins.git_index_modes(suite)
            self.assertEqual(modes, {source: b"100755"})
            expected = git_id("tree", b"100755 exec.sh\0" + git_id("blob", b"exit 0\n"))
            self.assertEqual(pins.git_tree_id(suite, modes), (expected, True))
            (suite / "untracked.js").write_bytes(b"262;\n")
            with self.assertRaisesRegex(ValueError, "worktree paths"):
                pins.git_index_modes(suite)

    def test_source_pins_reject_ambiguous_incomplete_and_unversioned_metadata(self):
        valid = {field: "a" * 40 for field in ["ecma262", "test262_upstream", "test262_tree"]}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "pins.json"
            for field in valid:
                missing = dict(valid)
                del missing[field]
                path.write_text(json.dumps(missing))
                with self.assertRaises(ValueError):
                    pins.load_pins(path)
                for invalid in [None, 1, "", "current-draft", "a" * 39, "a" * 64, "A" * 40]:
                    changed = dict(valid)
                    changed[field] = invalid
                    path.write_text(json.dumps(changed))
                    with self.assertRaises(ValueError):
                        pins.load_pins(path)
            path.write_text(json.dumps({**valid, "test262_revision": "b" * 40}))
            with self.assertRaises(ValueError):
                pins.load_pins(path)
            path.write_text(json.dumps(valid)[:-1] + ', "ecma262":"' + "b" * 40 + '"}')
            with self.assertRaisesRegex(ValueError, "duplicate source pin"):
                pins.load_pins(path)


if __name__ == "__main__":
    unittest.main()
