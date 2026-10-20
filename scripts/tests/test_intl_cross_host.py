"""Artifact admission controls; fixture evidence does not claim native execution."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "check-intl-cross-host.py"
SPEC = importlib.util.spec_from_file_location("intl_cross_host", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.directories = [root / "linux", root / "macos"]
        for directory, system in zip(self.directories, ("Linux", "Darwin")):
            directory.mkdir()
            receipt = {"schema": 1, "status": "complete",
                       "source": {"commit": "fixture-commit", "cargo_lock_sha256": "fixture-lock"},
                       "host": {"system": system, "machine": "fixture"},
                       "commands": [], "native": {}, "profiles": {}}
            receipt["executable"] = {"file": "scriptw", "bytes": 1, "sha256": system + "-image"}
            receipt["compiler_identity"] = {
                "source_revision": {"kind": "git-commit", "commit": "fixture-commit"},
                "executable_sha256": system + "-image",
                "source_fingerprint_scheme": "fixture-scheme", "source_fingerprint": "fixture-fingerprint"}
            for scope in MODULE.NATIVE_SCOPES:
                log = directory / (scope + ".log")
                log.write_text("test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n")
                receipt["commands"].append({"label": scope, "exit_code": 0, "log": MODULE.seal(log)})
                receipt["native"][scope] = MODULE.native_outcome(log.read_text())
            for profile in MODULE.PROFILES:
                bundle = directory / (profile + ".bundle")
                bundle.write_bytes(b"fixture export " + profile.encode())
                identity = directory / (profile + "-identity.log")
                identity.write_text(f"schema=1\nprofile={profile}\ndigest=fixture\n")
                receipt["profiles"][profile] = {"bundle": MODULE.seal(bundle), "identity": MODULE.seal(identity)}
                receipt["commands"].append({"label": profile + "-identity", "exit_code": 0, "log": MODULE.seal(identity)})
                log = directory / (profile + "-export.log")
                log.write_text("fixture export command\n")
                receipt["commands"].append({"label": profile + "-export", "exit_code": 0, "log": MODULE.seal(log)})
            for label in ("build-cli", "compiler-identity"):
                log = directory / (label + ".log")
                log.write_text(json.dumps(receipt["compiler_identity"]) if label == "compiler-identity" else "fixture build\n")
                receipt["commands"].append({"label": label, "exit_code": 0, "log": MODULE.seal(log)})
            self.save(directory, receipt)

    @staticmethod
    def save(directory, receipt):
        (directory / "receipt.json").write_text(json.dumps(receipt))

    def change(self, mutate):
        directory = self.directories[1]
        receipt = json.loads((directory / "receipt.json").read_text())
        mutate(receipt)
        self.save(directory, receipt)

    def test_complete_independent_hosts_match_actual_files(self):
        result = MODULE.compare(self.directories)
        self.assertEqual(result["status"], "complete")
        self.assertEqual([h["system"] for h in result["hosts"]], ["Linux", "Darwin"])

    def test_one_host_and_repeated_same_host_cannot_establish_reproducibility(self):
        with self.assertRaisesRegex(ValueError, "two independent"):
            MODULE.compare(self.directories[:1])
        self.change(lambda r: r["host"].update(system="Linux"))
        with self.assertRaisesRegex(ValueError, "one operating system"):
            MODULE.compare(self.directories)

    def test_unfinished_capture_cannot_compare(self):
        self.change(lambda r: r.update(status="incomplete"))
        with self.assertRaisesRegex(ValueError, "incomplete"):
            MODULE.compare(self.directories)

    def test_distinct_checkout_or_lock_cannot_compare(self):
        for key in ("commit", "cargo_lock_sha256"):
            with self.subTest(key=key):
                directory = self.directories[1]
                original = (directory / "receipt.json").read_text()
                self.change(lambda r: r["source"].update({key: "different"}))
                with self.assertRaisesRegex(ValueError, "source differs|captured checkout"):
                    MODULE.compare(self.directories)
                (directory / "receipt.json").write_text(original)

    def test_changed_export_rejects_even_with_reminted_file_seal(self):
        directory = self.directories[1]
        bundle = directory / "conformance.bundle"
        bundle.write_bytes(b"different actual bytes")
        with self.assertRaisesRegex(ValueError, "bundle or identity changed"):
            MODULE.compare(self.directories)
        self.change(lambda r: r["profiles"]["conformance"].update(bundle=MODULE.seal(bundle)))
        with self.assertRaisesRegex(ValueError, "profiles differs"):
            MODULE.compare(self.directories)

    def test_empty_filtered_ignored_failed_and_ambiguous_native_summaries_reject(self):
        for summary in ("", "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;",
                        "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;",
                        "test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;",
                        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out;",
                        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n" * 2):
            with self.subTest(summary=summary), self.assertRaises(ValueError):
                MODULE.native_outcome(summary)

    def test_reminted_transcript_still_must_match_native_receipt(self):
        directory = self.directories[1]
        log = directory / "library.log"
        log.write_text(log.read_text().replace("2 passed", "3 passed"))
        self.change(lambda r: r["commands"][0].update(log=MODULE.seal(log)))
        with self.assertRaisesRegex(ValueError, "native outcome differs"):
            MODULE.compare(self.directories)

    def test_missing_native_scope_and_profile_cannot_compare(self):
        for key, name in (("native", "calendar-domain"), ("profiles", "minimal")):
            with self.subTest(key=key):
                directory = self.directories[1]
                original = (directory / "receipt.json").read_text()
                self.change(lambda r: r[key].pop(name))
                with self.assertRaisesRegex(ValueError, "inventory is incomplete"):
                    MODULE.compare(self.directories)
                (directory / "receipt.json").write_text(original)

    def test_failed_command_or_changed_log_rejects_complete_marker(self):
        directory = self.directories[1]
        original = (directory / "receipt.json").read_text()
        self.change(lambda r: r["commands"][0].update(exit_code=101))
        with self.assertRaisesRegex(ValueError, "failed command"):
            MODULE.compare(self.directories)
        (directory / "receipt.json").write_text(original)
        (directory / "library.log").write_text("changed\n")
        with self.assertRaisesRegex(ValueError, "log changed"):
            MODULE.compare(self.directories)

    def test_stale_executable_and_stale_compiler_checkout_reject(self):
        for key, value in (("executable_sha256", "different-image"),
                           ("source_revision", {"kind": "git-commit", "commit": "stale-commit"})):
            with self.subTest(key=key):
                directory = self.directories[1]
                original = (directory / "receipt.json").read_text()
                self.change(lambda r: r["compiler_identity"].update({key: value}))
                with self.assertRaisesRegex(ValueError, "compiler does not bind"):
                    MODULE.compare(self.directories)
                (directory / "receipt.json").write_text(original)

    def test_missing_export_command_cannot_compare(self):
        self.change(lambda r: r["commands"].pop())
        with self.assertRaisesRegex(ValueError, "command inventory"):
            MODULE.compare(self.directories)


if __name__ == "__main__":
    unittest.main()
