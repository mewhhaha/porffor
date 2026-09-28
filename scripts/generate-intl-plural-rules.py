#!/usr/bin/env python3
"""Refresh or verify the pinned full-coverage ICU4X plural-rules blob."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import urllib.request
import zipfile


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "scripts/intl-plural-rules-data-gen"
OUTPUT = ROOT / "crates/lila-intl/src/provider/plural_rules/generated"
MANIFEST = OUTPUT / "manifest.json"
SOURCE = {
    "name": "CLDR 47.0.0",
    "url": "https://github.com/unicode-org/cldr-json/releases/download/47.0.0/cldr-47.0.0-json-full.zip",
    "sha256": "bbb9a9aac2dfc534bd18288678a5984023d11d22f712f3c33425f3214bd1def6",
}
MARKERS = {
    "PluralsCardinalV1": ("plurals.json", "plurals-type-cardinal", 219, ("und", "gv", "bal", "kw", "lld", "scn")),
    "PluralsOrdinalV1": ("ordinals.json", "plurals-type-ordinal", 104, ("und", "bal", "kw", "lld", "scn")),
    "PluralsRangesV1": ("pluralRanges.json", "plurals", 92, ("und", "scn")),
}


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def verified_source(cache):
    path = cache / SOURCE["url"].rsplit("/", 1)[1]
    if not path.exists():
        cache.mkdir(parents=True, exist_ok=True)
        temporary = path.with_suffix(path.suffix + ".download")
        try:
            with urllib.request.urlopen(SOURCE["url"]) as response, temporary.open("wb") as output:
                shutil.copyfileobj(response, output)
            if digest(temporary) != SOURCE["sha256"]:
                raise ValueError(f"source checksum mismatch: {SOURCE['url']}")
            temporary.replace(path)
        finally:
            temporary.unlink(missing_ok=True)
    if digest(path) != SOURCE["sha256"]:
        raise ValueError(f"source checksum mismatch: {path}")
    return path


def inventory(archive):
    result = {}
    for marker, (name, key, count, required) in MARKERS.items():
        rules = json.loads(archive.read(f"cldr-core/supplemental/{name}"))["supplemental"][key]
        ids = sorted(set(rules) | ({"und"} if marker == "PluralsRangesV1" else set()))
        if len(ids) != count or not set(required) <= set(ids):
            raise ValueError(f"pinned {marker} inventory differs: found {len(ids)}, expected {count}")
        result[marker] = ids
    return result


def inventory_record(ids):
    result = {}
    for marker, locales in ids.items():
        count = MARKERS[marker][2]
        if len(locales) != count or sorted(set(locales)) != locales or not set(MARKERS[marker][3]) <= set(locales):
            raise ValueError(f"invalid {marker} inventory")
        encoded = ("\n".join(locales) + "\n").encode()
        result[marker] = {
            "count": count,
            "locales": locales,
            "locales_sha256": hashlib.sha256(encoded).hexdigest(),
        }
    if set(result) != set(MARKERS):
        raise ValueError("plural marker inventory is incomplete")
    return result


def manifest(ids):
    inputs = (
        Path(__file__).resolve(),
        TOOL / "Cargo.toml",
        TOOL / "Cargo.lock",
        TOOL / "src/main.rs",
        TOOL / "README.md",
        OUTPUT / "plurals.postcard",
        OUTPUT / "LICENSE",
    )
    return {
        "schema_version": 1,
        "generator": "ICU4X 2.0.0 full plural markers with every pinned CLDR 47 source identifier",
        "source": SOURCE,
        "marker_inventories": inventory_record(ids),
        "files": [
            {"path": str(path.relative_to(ROOT)), "bytes": path.stat().st_size, "sha256": digest(path)}
            for path in inputs
        ],
    }


def check_manifest():
    if not MANIFEST.exists():
        raise ValueError("missing plural-rules manifest; run --refresh")
    recorded = json.loads(MANIFEST.read_text())
    ids = {marker: details["locales"] for marker, details in recorded["marker_inventories"].items()}
    expected = json.dumps(manifest(ids), indent=2) + "\n"
    if MANIFEST.read_text() != expected:
        raise ValueError("plural-rules identity is stale; run --refresh")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="check recorded local identities without compiling")
    mode.add_argument("--refresh", action="store_true", help="regenerate blob and identity manifest")
    mode.add_argument("--verify-generated", action="store_true", help="regenerate and byte-compare every output")
    parser.add_argument("--source-cache", type=Path, default=ROOT / "target/intl-collation-sources")
    args = parser.parse_args()

    if args.check:
        check_manifest()
        print("Pinned full plural-rules data verified")
        return

    source = verified_source(args.source_cache.resolve())
    with zipfile.ZipFile(source) as archive, tempfile.TemporaryDirectory(prefix="lila-plural-rules-") as temporary:
        ids = inventory(archive)
        license_bytes = archive.read("LICENSE")
        generated = Path(temporary) / "generated"
        environment = os.environ.copy()
        environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target/icu4x-datagen-build"))
        environment.setdefault("RAYON_NUM_THREADS", "3")
        subprocess.run(
            ["cargo", "run", "--release", "--locked", "-j3", "--manifest-path", str(TOOL / "Cargo.toml"),
             "--", str(generated), str(source)],
            check=True, env=environment, cwd=ROOT,
        )
        if args.refresh:
            OUTPUT.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(generated / "plurals.postcard", OUTPUT / "plurals.postcard")
            (OUTPUT / "LICENSE").write_bytes(license_bytes)
        elif ((generated / "plurals.postcard").read_bytes() != (OUTPUT / "plurals.postcard").read_bytes()
              or license_bytes != (OUTPUT / "LICENSE").read_bytes()):
            raise ValueError("generated plural-rules data differs")

    expected = json.dumps(manifest(ids), indent=2) + "\n"
    if args.refresh:
        MANIFEST.write_text(expected)
    elif not MANIFEST.exists() or MANIFEST.read_text() != expected:
        raise ValueError("plural-rules identity is stale; run --refresh")
    print("Pinned full plural-rules data verified")


if __name__ == "__main__":
    main()
