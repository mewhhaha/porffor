"""Summary generation and failure-before-write controls; not conformance tests."""
from collections import Counter
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "update-shortcut-status.py"
SPEC = importlib.util.spec_from_file_location("shortcut_status", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def row(index=1, classification="semantic-shortcut", removal="T13", reason="dynamic-source-substitution"):
    return f"source-text-predicate/fixture/{index:03d}\t{'a' * 64}\t{classification}\tT03\t{removal}\t{reason}\n"


def fixture(root, ledger):
    (root / "tasks").mkdir()
    (root / "test262/backlog").mkdir(parents=True)
    (root / "test262/backlog/shortcut-allowlist.tsv").write_text(ledger, encoding="utf-8")
    document = root / "tasks/README.md"
    document.write_text(f"before\n{MODULE.BEGIN}\nstale\n{MODULE.END}\nafter\n", encoding="utf-8")
    (root / MODULE.DOCUMENTS[1]).write_text(document.read_text(encoding="utf-8"), encoding="utf-8")
    return document


class SummaryTests(unittest.TestCase):
    def test_counts_every_classification_and_only_semantic_removal_debt(self):
        text = (row() + row(2, removal="T17") + row(3, removal="T17")
                + row(4, "legitimate-harness-adaptation", "T03", "suite-selection")
                + row(5, "diagnostic-instrumentation", "T03", "artifact-routing"))
        counts, removals = MODULE.ledger_counts(text)
        self.assertEqual(counts, Counter({"semantic-shortcut": 3,
                         "legitimate-harness-adaptation": 1, "diagnostic-instrumentation": 1}))
        self.assertEqual(removals, Counter({"T13": 1, "T17": 2}))
        summary = MODULE.render_summary(text)
        self.assertIn("| **Total** | **5** |", summary)
        self.assertIn("| `T17` | 2 |", summary)
        self.assertNotIn("| `T03` |", summary)

    def test_rendering_is_independent_of_ledger_order(self):
        self.assertEqual(MODULE.render_summary(row() + row(2, removal="T17")),
                         MODULE.render_summary(row(2, removal="T17") + row()))

    def test_zero_shortcuts_never_claims_conformance_completion(self):
        summary = MODULE.render_summary(row(classification="diagnostic-instrumentation", reason="artifact-routing"))
        self.assertIn("| `semantic-shortcut` | 0 |", summary)
        self.assertIn("does not establish", summary)
        self.assertNotIn("100%", summary)

    def test_empty_and_comment_only_ledgers_fail(self):
        for text in ("", "\n# header\n"):
            with self.subTest(text=text), self.assertRaises(ValueError):
                MODULE.render_summary(text)

    def test_duplicate_keys_fail(self):
        with self.assertRaisesRegex(ValueError, "duplicate"):
            MODULE.render_summary(row() + row())

    def test_invalid_envelopes_fail_closed(self):
        valid = row()
        for invalid in (valid.rstrip() + "\textra\n", valid.replace("\tT03\t", "\tT30\t"),
                        valid.replace("semantic-shortcut", "not-a-shortcut"),
                        valid.replace("dynamic-source-substitution", "artifact-routing"),
                        valid.replace("a" * 64, "0" * 63), valid.replace("fixture/001", "fixture/one")):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                MODULE.render_summary(invalid)

    def test_only_the_marked_block_is_replaced(self):
        summary = MODULE.render_summary(row())
        document = f"before\n{MODULE.BEGIN}\nstale\n{MODULE.END}\nafter\n"
        updated = MODULE.replace_summary(document, summary)
        self.assertEqual(updated, f"before\n{summary}\nafter\n")
        self.assertEqual(MODULE.replace_summary(updated, summary), updated)

    def test_missing_duplicate_and_reversed_markers_fail(self):
        for document in ("", MODULE.BEGIN, MODULE.END,
                         MODULE.BEGIN + MODULE.BEGIN + MODULE.END,
                         MODULE.BEGIN + MODULE.END + MODULE.END,
                         MODULE.END + MODULE.BEGIN):
            with self.subTest(document=document), self.assertRaises(ValueError):
                MODULE.replace_summary(document, MODULE.render_summary(row()))

    def test_check_is_read_only_and_write_is_idempotent(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            document = fixture(root, row())
            original = document.read_bytes()
            with patch.object(MODULE.subprocess, "run") as audit:
                self.assertFalse(MODULE.refresh(root, check=True))
                self.assertEqual(document.read_bytes(), original)
                self.assertTrue(MODULE.refresh(root, check=False))
                updated = document.read_bytes()
                self.assertTrue(MODULE.refresh(root, check=True))
                self.assertTrue(MODULE.refresh(root, check=False))
                self.assertEqual(document.read_bytes(), updated)
                audit.assert_called_with([str(root / "scripts/audit-test262-shortcuts.sh"), "--check"],
                                         cwd=root, check=True)
                self.assertEqual(audit.call_count, 4)

    def test_invalid_second_document_does_not_partially_update_the_first(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            document = fixture(root, row())
            original = document.read_bytes()
            (root / MODULE.DOCUMENTS[1]).write_text("missing markers", encoding="utf-8")
            with patch.object(MODULE.subprocess, "run"), self.assertRaises(ValueError):
                MODULE.refresh(root, check=False)
            self.assertEqual(document.read_bytes(), original)

    def test_failed_source_audit_never_rewrites_the_document(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            document = fixture(root, row())
            original = document.read_bytes()
            error = subprocess.CalledProcessError(1, ["audit"])
            with patch.object(MODULE.subprocess, "run", side_effect=error):
                with self.assertRaises(subprocess.CalledProcessError):
                    MODULE.refresh(root, check=False)
            self.assertEqual(document.read_bytes(), original)

    def test_invalid_ledger_after_audit_never_rewrites_the_document(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            document = fixture(root, row() + row())
            original = document.read_bytes()
            with patch.object(MODULE.subprocess, "run"), self.assertRaises(ValueError):
                MODULE.refresh(root, check=False)
            self.assertEqual(document.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
