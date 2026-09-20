#!/usr/bin/env python3
"""Regenerate the complete pinned NumberFormat profile without network access."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / "crates/lila-intl/data/number-cldr-47"
LEAVES = ROOT / "scripts/intl_numberformat_profile"
SOURCE = ROOT / "crates/lila-intl/src/number_format"
GENERATED = ["profiles.json.gz", "profile-provenance.json.gz", "medial-denominator-policy.json", "plural-samples.json", "coverage.json", "profiles.bin", "payload-manifest.json"]
RUST_GENERATED = ["profiles/fingerprint.rs", "tests/plural_samples.rs"]


def unpack(destination):
    manifest = json.loads((DIRECTORY / "source-manifest.json").read_text())
    if manifest["schema"] != 1 or manifest["archive"] != "sources.tar.gz":
        raise ValueError("unknown pinned source package")
    packed = DIRECTORY / manifest["archive"]
    raw = packed.read_bytes()
    if len(raw) != manifest["archive_bytes"] or hashlib.sha256(raw).hexdigest() != manifest["archive_sha256"]:
        raise ValueError("pinned NumberFormat source archive differs")
    expected = {row["path"]: row for row in manifest["files"]}
    if len(expected) != len(manifest["files"]):
        raise ValueError("duplicate pinned source path")
    seen = set()
    with tarfile.open(packed, "r:gz") as archive:
        for member in archive:
            if not member.isfile() or member.name not in expected or member.name in seen or ".." in Path(member.name).parts or Path(member.name).is_absolute():
                raise ValueError(f"invalid pinned source member: {member.name}")
            content = archive.extractfile(member).read()
            row = expected[member.name]
            if len(content) != row["bytes"] or hashlib.sha256(content).hexdigest() != row["sha256"]:
                raise ValueError(f"pinned source differs: {member.name}")
            seen.add(member.name)
            target = destination / member.name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(content)
    if seen != expected.keys():
        raise ValueError("source package has missing files")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="compare regenerated outputs without modifying them")
    parser.add_argument("--verify-sources", action="store_true", help="verify the complete offline source archive only")
    parser.add_argument("--scratch-parent", type=Path, help="optional directory for temporary extraction")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="lila-number-cldr-47-", dir=args.scratch_parent) as scratch_name:
        scratch = Path(scratch_name)
        manifest = unpack(scratch)
        if args.verify_sources:
            print(json.dumps({"source_files": len(manifest["files"]), "archive_sha256": manifest["archive_sha256"]}))
            return
        shutil.copytree(LEAVES, scratch / "scripts", ignore=shutil.ignore_patterns("__pycache__"))
        generated = scratch / "work/crates/lila-intl/data/number-cldr-47"
        generated.mkdir(parents=True)
        rust = scratch / "work/crates/lila-intl/src/number_format"
        rust.mkdir(parents=True)
        shutil.copyfile(SOURCE / "options.rs", rust / "options.rs")
        if args.check:
            for name in GENERATED:
                shutil.copyfile(DIRECTORY / name, generated / name)
            for name in RUST_GENERATED:
                (rust / name).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(SOURCE / name, rust / name)
        flags = ["--check"] if args.check else []
        subprocess.run([sys.executable, str(scratch / "scripts/fetch_inputs.py"), "--check"], check=True)
        subprocess.run([sys.executable, str(scratch / "scripts/extract_profiles.py"), *flags], check=True)
        subprocess.run([sys.executable, str(scratch / "scripts/generate_payload.py"), *flags], check=True)
        if not args.check:
            for name in GENERATED:
                shutil.copyfile(generated / name, DIRECTORY / name)
            for name in RUST_GENERATED:
                (SOURCE / name).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(rust / name, SOURCE / name)


if __name__ == "__main__":
    main()
