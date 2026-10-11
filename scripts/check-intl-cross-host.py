#!/usr/bin/env python3
"""Retain native Intl runs and compare admitted exports from independent hosts."""
import argparse
import hashlib
import json
import platform
import re
import subprocess
import time
from pathlib import Path

PROFILES = ("minimal", "conformance")
NATIVE_SCOPES = ("library", "calendar-domain", "datetime-calendar16")
COMMAND_LABELS = {*NATIVE_SCOPES, "build-cli", "compiler-identity",
                  *(profile + suffix for profile in PROFILES for suffix in ("-export", "-identity"))}
SUMMARY = re.compile(
    r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out;", re.MULTILINE
)


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def seal(path):
    return {"file": path.name, "bytes": path.stat().st_size, "sha256": digest(path)}


def source_identity(root):
    def git(*args):
        return subprocess.check_output(["git", *args], cwd=root)
    if git("status", "--porcelain", "--untracked-files=normal"):
        raise ValueError("cross-host evidence requires a clean checkout")
    return {"commit": git("rev-parse", "HEAD").decode().strip(),
            "cargo_lock_sha256": digest(root / "Cargo.lock")}


def native_outcome(log):
    summaries = SUMMARY.findall(log)
    if len(summaries) != 1:
        raise ValueError("expected one complete native test summary")
    passed, failed, ignored, measured, filtered = map(int, summaries[0])
    if passed == 0 or any((failed, ignored, measured, filtered)):
        raise ValueError("native Intl scope was empty, incomplete or unsuccessful")
    return {"passed": passed, "failed": failed, "ignored": ignored,
            "measured": measured, "filtered": filtered}


def compiler_source(receipt):
    compiler = receipt["compiler_identity"]
    if compiler["source_revision"] != {"kind": "git-commit", "commit": receipt["source"]["commit"]}:
        raise ValueError("compiler does not bind the captured checkout")
    if compiler["executable_sha256"] != receipt["executable"]["sha256"]:
        raise ValueError("compiler does not bind the executing image")
    return {key: compiler[key] for key in ("source_fingerprint_scheme", "source_fingerprint", "source_revision")}


def capture(root, output):
    identity = source_identity(root)
    output.mkdir(parents=True, exist_ok=False)
    receipt = {"schema": 1, "status": "incomplete", "source": identity,
               "host": {"system": platform.system(), "machine": platform.machine()},
               "commands": [], "native": {}, "profiles": {}}

    def save():
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")

    def run(label, command):
        log = output / (label + ".log")
        start = time.monotonic()
        with log.open("wb") as stream:
            result = subprocess.run(command, cwd=root, stdout=stream, stderr=subprocess.STDOUT)
        receipt["commands"].append({"label": label, "command": command,
                                    "exit_code": result.returncode,
                                    "seconds": round(time.monotonic() - start, 3),
                                    "log": seal(log)})
        save()
        # The retained output also remains visible in the CI job transcript.
        print(log.read_text(errors="replace"), end="", flush=True)
        if result.returncode:
            raise ValueError(f"{label} failed with exit {result.returncode}; see {log}")
        return log

    save()
    for label, selector in zip(NATIVE_SCOPES, (
        ["--lib"], ["--test", "calendar_domain"], ["--test", "datetime_calendar16"]
    )):
        log = run(label, ["cargo", "test", "--release", "--locked", "-p", "lila-intl",
                          *selector, "--", "--test-threads=1", "--include-ignored"])
        receipt["native"][label] = native_outcome(log.read_text(errors="replace"))
        save()
    run("build-cli", ["cargo", "build", "--release", "--locked", "-p", "lila-cli"])
    binary = root / "target" / "release" / ("lila.exe" if platform.system() == "Windows" else "lila")
    receipt["executable"] = seal(binary)
    compiler_log = run("compiler-identity", [str(binary), "compiler-identity"])
    receipt["compiler_identity"] = json.loads(compiler_log.read_text())
    compiler_source(receipt)
    for profile in PROFILES:
        bundle = output / (profile + ".bundle")
        run(profile + "-export", [str(binary), "--jobs", "1", "--intl-profile", profile,
                                  "intl", "export", "--output", str(bundle)])
        admitted = run(profile + "-identity", [str(binary), "--jobs", "1", "intl", "inspect",
                                                "--input", str(bundle)])
        if not admitted.read_bytes() or f"profile={profile}\n".encode() not in admitted.read_bytes():
            raise ValueError("export did not admit its explicit complete profile identity")
        receipt["profiles"][profile] = {"bundle": seal(bundle), "identity": seal(admitted)}
        save()
    if source_identity(root) != identity:
        raise ValueError("source changed during native Intl evidence capture")
    if digest(binary) != receipt["executable"]["sha256"]:
        raise ValueError("compiler executable changed during capture")
    receipt["status"] = "complete"
    save()


def admitted_receipt(directory):
    receipt = json.loads((directory / "receipt.json").read_text())
    if receipt.get("schema") != 1 or receipt.get("status") != "complete":
        raise ValueError("Intl evidence is incomplete or has an unsupported schema")
    if set(receipt["native"]) != set(NATIVE_SCOPES) or set(receipt["profiles"]) != set(PROFILES):
        raise ValueError("Intl native or profile inventory is incomplete")
    if len(receipt["commands"]) != len(COMMAND_LABELS) or {c["label"] for c in receipt["commands"]} != COMMAND_LABELS:
        raise ValueError("Intl command inventory is incomplete or duplicated")
    for command in receipt["commands"]:
        if command["exit_code"] != 0:
            raise ValueError("Intl evidence contains a failed command")
        entry = command["log"]
        if seal(directory / entry["file"]) != entry:
            raise ValueError("retained command log changed")
    compiler_source(receipt)
    compiler_log = next(c["log"] for c in receipt["commands"] if c["label"] == "compiler-identity")
    if json.loads((directory / compiler_log["file"]).read_text()) != receipt["compiler_identity"]:
        raise ValueError("compiler identity differs from its retained command")
    for label in NATIVE_SCOPES:
        matches = [c for c in receipt["commands"] if c["label"] == label]
        if len(matches) != 1 or native_outcome((directory / matches[0]["log"]["file"]).read_text()) != receipt["native"][label]:
            raise ValueError("native outcome differs from its complete retained transcript")
    for profile in PROFILES:
        for role in ("bundle", "identity"):
            entry = receipt["profiles"][profile][role]
            if entry["file"] != profile + (".bundle" if role == "bundle" else "-identity.log") or seal(directory / entry["file"]) != entry:
                raise ValueError("retained Intl bundle or identity changed")
        if f"profile={profile}\n" not in (directory / receipt["profiles"][profile]["identity"]["file"]).read_text():
            raise ValueError("Intl identity does not bind its explicit profile")
    return receipt


def compare(directories):
    if len(directories) < 2:
        raise ValueError("cross-host evidence requires at least two independent operating systems")
    receipts = [admitted_receipt(directory) for directory in directories]
    if len({r["host"]["system"] for r in receipts}) < 2:
        raise ValueError("multiple runs on one operating system are not cross-host evidence")
    first = receipts[0]
    for index, current in enumerate(receipts[1:], 1):
        for role in ("source", "native", "profiles"):
            if current[role] != first[role]:
                raise ValueError(f"cross-host Intl {role} differs")
        if compiler_source(current) != compiler_source(first):
            raise ValueError("cross-host compiler source fingerprints differ")
        # Compare actual files as well as their retained hashes.
        for profile in PROFILES:
            for role in ("bundle", "identity"):
                name = first["profiles"][profile][role]["file"]
                if (directories[0] / name).read_bytes() != (directories[index] / name).read_bytes():
                    raise ValueError(f"cross-host Intl {profile} {role} bytes differ")
    return {"schema": 1, "status": "complete", "source": first["source"],
            "hosts": [r["host"] for r in receipts], "native": first["native"],
            "profiles": first["profiles"],
            "receipts": [{"directory": str(d), "sha256": digest(d / "receipt.json")} for d in directories]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    capture_parser = commands.add_parser("capture")
    capture_parser.add_argument("--root", type=Path, default=Path.cwd())
    capture_parser.add_argument("--output", type=Path, required=True)
    compare_parser = commands.add_parser("compare")
    compare_parser.add_argument("directories", type=Path, nargs="+")
    compare_parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "capture":
        capture(args.root.resolve(), args.output.resolve())
    else:
        result = compare([path.resolve() for path in args.directories])
        with args.output.open("x") as destination:
            destination.write(json.dumps(result, indent=2) + "\n")
        print("Complete native Intl scopes and both admitted exports match across hosts.")


if __name__ == "__main__":
    main()
