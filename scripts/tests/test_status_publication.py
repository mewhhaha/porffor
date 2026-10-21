import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import status_publication as publication


class StatusGeneration(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.parent = self.root / Path(publication.MANIFEST).parent
        self.parent.mkdir(parents=True)
        self.json_bytes = (
            b'{"producer":"scriptw","real_suite":{"backend":"wasm-aot","total":123}}'
        )
        self.text_bytes = b"total=123\n"
        generation = publication.generation_id(self.json_bytes, self.text_bytes)
        self.manifest = {
            "schema_version": 1,
            "backend": "wasm-aot",
            "generation": generation,
        }
        for field, suffix, data in (
            ("json", "json", self.json_bytes),
            ("text", "txt", self.text_bytes),
        ):
            name = f"published-status-wasm-aot-{generation}.{suffix}"
            (self.parent / name).write_bytes(data)
            self.manifest[field] = {
                "file": name,
                "sha256": hashlib.sha256(data).hexdigest(),
            }
        self.write_manifest()

    def write_manifest(self):
        (self.root / publication.MANIFEST).write_text(json.dumps(self.manifest))

    def test_one_manifest_selects_both_members_and_ignores_orphans(self):
        (self.parent / "unselected.json").write_text("not valid JSON")
        parsed, source = publication.load_publication(self.root)
        self.assertEqual(parsed["real_suite"]["total"], 123)
        self.assertEqual(
            source,
            (
                Path(publication.MANIFEST).parent / self.manifest["json"]["file"]
            ).as_posix(),
        )

    def test_damage_cannot_fall_back_to_the_legacy_pair(self):
        (self.root / publication.SNAPSHOT).write_text('{"legacy":true}')
        member = self.parent / self.manifest["text"]["file"]
        member.write_text("wrong generation")
        with self.assertRaisesRegex(ValueError, "digest mismatch"):
            publication.load_publication(self.root)
        member.unlink()
        with self.assertRaises(FileNotFoundError):
            publication.load_publication(self.root)

    def test_pointer_cannot_escape_its_directory_or_mix_generation_members(self):
        original = copy.deepcopy(self.manifest)
        for bad in (
            "../../README.md",
            "published-status-wasm-aot-other.txt",
            "/tmp/report.txt",
        ):
            self.manifest["text"]["file"] = bad
            self.write_manifest()
            with self.assertRaisesRegex(ValueError, "own immutable generation"):
                publication.load_publication(self.root)
        self.manifest = original
        self.manifest["generation"] = "0" * 64
        self.write_manifest()
        with self.assertRaises(ValueError):
            publication.load_publication(self.root)

    def test_legacy_evidence_is_readable_only_when_no_manifest_exists(self):
        (self.root / publication.MANIFEST).unlink()
        (self.root / publication.SNAPSHOT).write_text('{"legacy":true}')
        self.assertEqual(
            publication.load_publication(self.root),
            ({"legacy": True}, publication.SNAPSHOT),
        )
        (self.root / publication.MANIFEST).write_text("broken manifest")
        with self.assertRaises(ValueError):
            publication.load_publication(self.root)

    def test_consistent_hashes_cannot_relabel_oracle_or_unknown_payloads(self):
        for value in (
            {"producer": "scriptw", "real_suite": {"backend": "spec-exec"}},
            {},
            [],
        ):
            payload = json.dumps(value).encode()
            generation = publication.generation_id(payload, self.text_bytes)
            self.manifest["generation"] = generation
            for field, suffix, data in (
                ("json", "json", payload),
                ("text", "txt", self.text_bytes),
            ):
                name = f"published-status-wasm-aot-{generation}.{suffix}"
                (self.parent / name).write_bytes(data)
                self.manifest[field] = {
                    "file": name,
                    "sha256": hashlib.sha256(data).hexdigest(),
                }
            self.write_manifest()
            with self.assertRaisesRegex(ValueError, "wasm-aot publication evidence"):
                publication.load_publication(self.root)

    def test_boolean_schema_and_oracle_backend_are_rejected(self):
        for key, value in (("schema_version", True), ("backend", "spec-exec")):
            prior = self.manifest[key]
            self.manifest[key] = value
            self.write_manifest()
            with self.assertRaises(ValueError):
                publication.load_publication(self.root)
            self.manifest[key] = prior


if __name__ == "__main__":
    unittest.main()
