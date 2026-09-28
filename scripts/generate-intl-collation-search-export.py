#!/usr/bin/env python3
"""Verify or regenerate ICU4C's pinned search-collation Jamo export repair."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import urllib.request
import zipfile


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/lila-intl/data/collation-search-icu77"
MANIFEST = OUTPUT / "manifest.json"
PATCH = OUTPUT / "source.patch"
SOURCE_COMMIT = "594c9b890abfa794a259162f1a29513445365f89"
SOURCE = {
    "url": f"https://codeload.github.com/unicode-org/icu/tar.gz/{SOURCE_COMMIT}",
    "sha256": "70eec6bc52e221110ce7345f3349dcffbe6900cfe8b649b29de1e1b7c1dc11e8",
}
ICU_EXPORT = {
    "url": "https://github.com/unicode-org/icu/releases/download/icu4x/2025-05-01/77.x/icuexportdata_icu4x-2025-05-01-77.x.zip",
    "sha256": "e5dae398d77a31ee7fcd86e4b35041cf9bf4643c694794629fc7c5dca76d9c6b",
}
SOURCE_DIRECTORY = f"icu-{SOURCE_COMMIT}"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def verified_source(source, cache, *, download):
    path = cache / source["url"].rsplit("/", 1)[1]
    if not path.exists():
        if not download:
            raise ValueError(f"missing pinned source: {path}")
        cache.mkdir(parents=True, exist_ok=True)
        temporary = path.with_suffix(path.suffix + ".download")
        try:
            with urllib.request.urlopen(source["url"]) as response, temporary.open("wb") as output:
                shutil.copyfileobj(response, output)
            if digest(temporary) != source["sha256"]:
                raise ValueError(f"source checksum mismatch: {source['url']}")
            temporary.replace(path)
        finally:
            temporary.unlink(missing_ok=True)
    if digest(path) != source["sha256"]:
        raise ValueError(f"source checksum mismatch: {path}")
    return path


def search_names(archive):
    names = sorted(
        path.rsplit("/", 1)[1]
        for path in archive.namelist()
        if path.startswith("collation/implicithan/")
        and path.endswith("_data.toml")
        and "_search" in path.rsplit("/", 1)[1]
    )
    if len(names) != 22 or len(set(names)) != len(names):
        raise ValueError(f"expected 22 distinct pinned search data files, found {len(names)}")
    return names


def recorded_manifest(names):
    inputs = (Path(__file__).resolve(), PATCH, OUTPUT / "README.md", OUTPUT / "LICENSE", *(OUTPUT / name for name in names))
    return {
        "schema_version": 1,
        "generator": "pinned ICU4C genrb -X with Jamo trie export patch",
        "source_commit": SOURCE_COMMIT,
        "source_archive": SOURCE,
        "original_export": ICU_EXPORT,
        "collation_root_han": "implicithan",
        "search_data_files": names,
        "files": [
            {"path": str(path.relative_to(ROOT)), "bytes": path.stat().st_size, "sha256": digest(path)}
            for path in inputs
        ],
    }


def check_manifest():
    if not MANIFEST.exists():
        raise ValueError("missing search collation export manifest; run --refresh")
    recorded = json.loads(MANIFEST.read_text())
    names = recorded.get("search_data_files", [])
    if len(names) != 22 or len(set(names)) != 22 or any(Path(name).name != name or not name.endswith("_data.toml") or "_search" not in name for name in names):
        raise ValueError("invalid pinned search collation data file list")
    if MANIFEST.read_text() != json.dumps(recorded_manifest(names), indent=2) + "\n":
        raise ValueError("search collation export identity is stale; run --refresh")
    return names


def build_export(source_archive, directory):
    with tarfile.open(source_archive, "r:gz") as archive:
        archive.extractall(directory, filter="data")
    checkout = directory / SOURCE_DIRECTORY
    subprocess.run(["patch", "-p1", "-i", str(PATCH)], check=True, cwd=checkout)
    source = checkout / "icu4c/source"
    environment = os.environ.copy()
    environment.update({"CC": "clang", "CXX": "clang++"})
    subprocess.run(["./runConfigureICU", "Linux"], check=True, cwd=source, env=environment)
    subprocess.run(["make", "-j3"], check=True, cwd=source, env=environment)
    environment["LD_LIBRARY_PATH"] = str(source / "lib")
    files = sorted(path.name for path in (source / "data/coll").glob("*.txt"))
    output = directory / "collation"
    for han in ("unihan", "implicithan"):
        destination = output / han
        destination.mkdir(parents=True)
        subprocess.run(
            ["./bin/genrb", "-X", "-s", "data/coll", "--ucadata",
             f"data/in/coll/ucadata-{han}-icu4x.icu", "-d", str(destination), *files],
            check=True, cwd=source, env=environment,
        )
    return output


def compare_export(original, generated):
    names = search_names(original)
    original_names = {
        path for path in original.namelist()
        if path.startswith("collation/") and path.endswith(".toml")
    }
    generated_names = {"collation/" + str(path.relative_to(generated)) for path in generated.rglob("*.toml")}
    if generated_names != original_names:
        raise ValueError(f"export file set differs: missing={sorted(original_names - generated_names)}, extra={sorted(generated_names - original_names)}")
    changed = []
    for name in sorted(original_names):
        previous = original.read(name)
        current = (generated / name.removeprefix("collation/")).read_bytes()
        if previous == current:
            continue
        if not name.endswith("_data.toml") or name.endswith("/root_standard_data.toml"):
            raise ValueError(f"unexpected non-tailoring export change: {name}")
        old = tomllib.loads(previous.decode())
        new = tomllib.loads(current.decode())
        if old.keys() != new.keys() or old["trie"] == new["trie"]:
            raise ValueError(f"invalid changed collation data: {name}")
        for key in old.keys() - {"trie"}:
            if old[key] != new[key]:
                raise ValueError(f"changed {key} outside collation trie: {name}")
        changed.append(name)
    expected_changes = {f"collation/{han}/{name}" for han in ("unihan", "implicithan") for name in names}
    if set(changed) != expected_changes:
        raise ValueError(f"unexpected changed tries: missing={sorted(expected_changes - set(changed))}, extra={sorted(set(changed) - expected_changes)}")
    return changed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="check checked-in data identities without compiling")
    mode.add_argument("--refresh", action="store_true", help="regenerate patched ICU export and identity manifest")
    mode.add_argument("--verify-generated", action="store_true", help="regenerate and byte-compare checked-in data")
    parser.add_argument("--source-cache", type=Path, default=ROOT / "target/intl-collation-sources")
    args = parser.parse_args()
    if args.check:
        check_manifest()
        print("Pinned search collation export data verified")
        return

    original_zip = verified_source(ICU_EXPORT, args.source_cache.resolve(), download=True)
    with zipfile.ZipFile(original_zip) as original:
        names = search_names(original)
        with tempfile.TemporaryDirectory(prefix="lila-icu-collation-export-") as temporary:
            source_archive = verified_source(SOURCE, args.source_cache.resolve(), download=True)
            generated = build_export(source_archive, Path(temporary))
            changed = compare_export(original, generated)
            for name in names:
                path = generated / "implicithan" / name
                if args.refresh:
                    shutil.copyfile(path, OUTPUT / name)
                elif path.read_bytes() != (OUTPUT / name).read_bytes():
                    raise ValueError(f"generated search collation differs: {name}")

    expected = json.dumps(recorded_manifest(names), indent=2) + "\n"
    if args.refresh:
        MANIFEST.write_text(expected)
    elif not MANIFEST.exists() or MANIFEST.read_text() != expected:
        raise ValueError("search collation export identity is stale; run --refresh")
    print(f"Pinned search collation export verified ({len(changed)} changed tries)")


if __name__ == "__main__":
    main()
