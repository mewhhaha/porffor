import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "conformance_owners.py"
spec = importlib.util.spec_from_file_location("conformance_owners", SCRIPT)
ownership = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ownership)


def registry_source():
    rows = [f'T{n:02} = {n} => "T{n:02}" => "Area {n}",' for n in range(30)]
    return "define_conformance_owners! {\n" + "\n".join(rows) + "\n}"


class ConformanceOwnershipTests(unittest.TestCase):
    def test_maintained_document_inventory_rejects_missing_and_duplicate_owners(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            inventory = root / "scripts/maintained-documents.json"
            (root / "current.md").write_text("[Contract](contract.md)")
            (root / "contract.md").write_text("# Contract\n")
            inventory.write_text(json.dumps(["current.md"]))
            self.assertEqual(ownership.check_maintained_documents(root), 1)
            inventory.write_text(json.dumps(["current.md", "current.md"]))
            with self.assertRaisesRegex(ValueError, "duplicate"):
                ownership.check_maintained_documents(root)
            inventory.write_text(json.dumps(["missing.md"]))
            with self.assertRaises(OSError):
                ownership.check_maintained_documents(root)

    def test_closed_registry_rejects_unknown_duplicate_inconsistent_and_empty_owners(
        self,
    ):
        source = registry_source()
        self.assertEqual(
            list(ownership.owners(source)), [f"T{n:02}" for n in range(30)]
        )
        malformed = [
            source.replace('T29 = 29 => "T29"', 'T30 = 30 => "T30"'),
            source.replace('T29 = 29 => "T29"', 'T28 = 28 => "T28"'),
            source.replace('T29 = 29 => "T29"', 'T29 = 28 => "T29"'),
            source.replace('T29 = 29 => "T29"', 'T29 = 29 => "T28"'),
            source.replace('"Area 29"', '""'),
            source.replace('"Area 29"', '" padded "'),
            source + source,
            source.replace("T29 =", "unknown ="),
        ]
        for changed in malformed:
            with self.subTest(source=changed[-120:]), self.assertRaises(ValueError):
                ownership.owners(changed)

    def test_durable_documents_require_every_area_anchor_and_acceptance_link(self):
        registry = ownership.owners(registry_source())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / ownership.DOCUMENT.parent
            docs.mkdir(parents=True)
            sections = [
                f'<a id="{code.lower()}"></a>\n\n## {code} — {area}\n'
                for code, area in registry.items()
            ]
            accepted = "\n".join(sections)
            document = "T26-unclassified\n" + "\n".join(
                section + f"[Acceptance](conformance-acceptance.md#t{n:02})\n"
                for n, section in enumerate(sections)
            )
            (root / ownership.ACCEPTANCE).write_text(accepted)
            (root / ownership.DOCUMENT).write_text(document)
            ownership.check_documents(root, registry)
            for changed in [
                accepted.replace('id="t29"', 'id="wrong"'),
                accepted.replace("## T29 — Area 29", "## T29 — Wrong"),
            ]:
                (root / ownership.ACCEPTANCE).write_text(changed)
                with self.assertRaises(ValueError):
                    ownership.check_documents(root, registry)
            (root / ownership.ACCEPTANCE).write_text(accepted)
            (root / ownership.DOCUMENT).write_text(
                document.replace(
                    "(conformance-acceptance.md#t29)",
                    "(conformance-acceptance.md#wrong)",
                )
            )
            with self.assertRaises(ValueError):
                ownership.check_documents(root, registry)

    def test_local_contract_links_reject_missing_files_anchors_and_root_escape(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "owner.md"
            contract = root / "contract.md"
            contract.write_text(
                "## 13. Shared Unicode identifier spelling authority\n"
                "## Repeated\n## Repeated\n"
            )
            ownership.check_links(
                root,
                path,
                "[valid](contract.md#13-shared-unicode-identifier-spelling-authority)",
            )
            ownership.check_links(root, path, "[valid](contract.md#repeated-1)")
            for target in ["absent.md", "contract.md#absent", "../outside.md"]:
                with self.subTest(target=target), self.assertRaises(ValueError):
                    ownership.check_links(root, path, f"[broken]({target})")


if __name__ == "__main__":
    unittest.main()
