#!/usr/bin/env python3
"""Refresh or verify the pinned ICU4X search-collation overlay."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import zipfile


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/lila-intl/src/provider/collation_search/generated"
TOOL = ROOT / "scripts/intl-collation-search-data-gen"
MANIFEST = OUTPUT / "manifest.json"
EXPORT_TOOL = ROOT / "scripts/generate-intl-collation-search-export.py"
EXPORT_DATA = ROOT / "crates/lila-intl/data/collation-search-icu77"
EXPORT_MANIFEST = EXPORT_DATA / "manifest.json"
GENERATED = ("search.postcard",)
SOURCES = (
    {
        "name": "CLDR 47.0.0",
        "url": "https://github.com/unicode-org/cldr-json/releases/download/47.0.0/cldr-47.0.0-json-full.zip",
        "sha256": "bbb9a9aac2dfc534bd18288678a5984023d11d22f712f3c33425f3214bd1def6",
    },
    {
        "name": "ICU icu4x/2025-05-01/77.x",
        "url": "https://github.com/unicode-org/icu/releases/download/icu4x/2025-05-01/77.x/icuexportdata_icu4x-2025-05-01-77.x.zip",
        "sha256": "e5dae398d77a31ee7fcd86e4b35041cf9bf4643c694794629fc7c5dca76d9c6b",
    },
)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def manifest():
    export_names = json.loads(EXPORT_MANIFEST.read_text())["search_data_files"]
    inputs = (
        Path(__file__).resolve(),
        EXPORT_TOOL,
        EXPORT_MANIFEST,
        *(EXPORT_DATA / name for name in export_names),
        TOOL / "Cargo.toml",
        TOOL / "Cargo.lock",
        TOOL / "src/main.rs",
        *(OUTPUT / name for name in (*GENERATED, "LICENSE")),
    )
    return {
        "schema_version": 1,
        "generator": "ICU4X 2.0.0 ExportDriver with patched pinned ICU search Jamo trie data",
        "sources": SOURCES,
        "files": [
            {"path": str(path.relative_to(ROOT)), "bytes": path.stat().st_size, "sha256": digest(path)}
            for path in inputs
        ],
    }


def verified_source(source, cache):
    path = cache / source["url"].rsplit("/", 1)[1]
    if not path.exists():
        cache.mkdir(parents=True, exist_ok=True)
        temporary = path.with_suffix(".download")
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


def overlay_icu_export(original, destination):
    names = json.loads(EXPORT_MANIFEST.read_text())["search_data_files"]
    replacements = {
        f"collation/implicithan/{name}": (EXPORT_DATA / name).read_bytes()
        for name in names
    }
    with zipfile.ZipFile(original) as source, zipfile.ZipFile(destination, "w") as output:
        available = set(source.namelist())
        if not replacements.keys() <= available:
            raise ValueError(f"missing original ICU export entries: {sorted(replacements.keys() - available)}")
        for entry in source.infolist():
            output.writestr(entry, replacements.get(entry.filename, source.read(entry)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="check recorded local file identities without compiling")
    mode.add_argument("--refresh", action="store_true", help="regenerate data and its identity manifest")
    mode.add_argument("--verify-generated", action="store_true", help="regenerate and byte-compare every output")
    parser.add_argument("--source-cache", type=Path, default=ROOT / "target/intl-collation-sources")
    args = parser.parse_args()

    subprocess.run([sys.executable, str(EXPORT_TOOL), "--check"], check=True, cwd=ROOT)

    if not args.check:
        sources = [verified_source(source, args.source_cache.resolve()) for source in SOURCES]
        with tempfile.TemporaryDirectory(prefix="lila-collation-search-") as temporary:
            patched_icu = Path(temporary) / "patched-icuexportdata.zip"
            overlay_icu_export(sources[1], patched_icu)
            generated = Path(temporary) / "generated"
            environment = os.environ.copy()
            environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target/icu4x-datagen-build"))
            environment.setdefault("RAYON_NUM_THREADS", "3")
            subprocess.run(
                ["cargo", "run", "--release", "--locked", "-j3", "--manifest-path", str(TOOL / "Cargo.toml"),
                 "--", str(generated), str(sources[0]), str(patched_icu), str(sources[1])],
                check=True, env=environment, cwd=ROOT,
            )
            for name in GENERATED:
                if args.refresh:
                    shutil.copyfile(generated / name, OUTPUT / name)
                elif (generated / name).read_bytes() != (OUTPUT / name).read_bytes():
                    raise ValueError(f"generated search collation differs: {name}")

    expected = json.dumps(manifest(), indent=2) + "\n"
    if args.refresh:
        MANIFEST.write_text(expected)
    elif not MANIFEST.exists() or MANIFEST.read_text() != expected:
        raise ValueError("search collation identity is stale; run --refresh")
    print("Pinned search collation data verified")


if __name__ == "__main__":
    main()
