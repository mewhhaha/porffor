#!/usr/bin/env python3
"""Create an explicitly amended Test262 copy; never alter the stock suite."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import sys

from check_conformance_source_pins import git_index_modes, git_tree_id, load_pins, verify


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def relative_file(root, name):
    if not isinstance(name, str):
        raise ValueError("erratum paths must be strings")
    path = PurePosixPath(name)
    if (not name or "\\" in name or path.is_absolute() or path.as_posix() != name
            or any(part in {".", ".."} for part in path.parts)):
        raise ValueError(f"invalid relative erratum path: {name}")
    result = root / name
    if not result.is_file() or any(part.is_symlink() for part in [result, *result.parents]
                                   if part != root and root in part.parents):
        raise ValueError(f"erratum requires a regular file: {name}")
    return result


def checked_manifest(suite, errata, pins_path):
    manifest_path = errata / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("schema") != "scriptw-test262-errata-v1":
        raise ValueError("unsupported Test262 errata schema")
    pins = load_pins(pins_path)
    if manifest["base_source_pins"] != pins:
        raise ValueError("errata do not match the declared base source pins")
    verify(pins_path, suite)
    seen = set()
    if not manifest["corrections"]:
        raise ValueError("errata must identify their corrections")
    for entry in manifest["corrections"]:
        name = entry["path"]
        original = relative_file(suite, name)
        replacement = relative_file(errata, entry["replacement"])
        if not name.startswith("test/") or not name.endswith(".js") or name in seen:
            raise ValueError(f"duplicate or non-test correction: {name}")
        seen.add(name)
        if sha(original) != entry["original_sha256"]:
            raise ValueError(f"original test hash differs: {name}")
        if sha(replacement) != entry["replacement_sha256"]:
            raise ValueError(f"replacement test hash differs: {name}")
        if entry["original_sha256"] == entry["replacement_sha256"]:
            raise ValueError(f"correction does not change its test: {name}")
    return manifest


def prepare(suite, errata, pins_path, output=None):
    suite, errata, pins_path = (path.resolve() for path in (suite, errata, pins_path))
    manifest = checked_manifest(suite, errata, pins_path)
    receipt = {
        "schema": "scriptw-test262-errata-preparation-v1",
        "complete": output is None,
        "check_only": output is None,
        "upstream_unmodified": False,
        "eligible_for_stock_test262_publication": False,
        "base_source_pins": manifest["base_source_pins"],
        "manifest_sha256": sha(errata / "manifest.json"),
        "corrections": manifest["corrections"],
    }
    if output is None:
        return receipt
    output = output.resolve()
    for protected in (suite, errata, pins_path.parent):
        if output == protected or output in protected.parents or protected in output.parents:
            raise ValueError("output must be separate from the suite, errata and source pins")
    output.mkdir(parents=True, exist_ok=False)
    receipt_path = output / "errata-receipt.json"
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
    amended = output / "suite"
    modes = git_index_modes(suite) if sys.platform == "win32" else None
    shutil.copytree(suite, amended, symlinks=True)
    copied_modes = ({amended / path.relative_to(suite): mode for path, mode in modes.items()}
                    if modes is not None else None)
    # Validate the complete copy before editing it. No symlink sharing or hard
    # links can let an amended test write through to the original suite.
    if git_tree_id(amended, copied_modes)[0].hex() != manifest["base_source_pins"]["test262_tree"]:
        raise ValueError("copied suite differs from the verified stock tree")
    for entry in manifest["corrections"]:
        destination = relative_file(amended, entry["path"])
        destination.write_bytes(relative_file(errata, entry["replacement"]).read_bytes())
        if sha(destination) != entry["replacement_sha256"]:
            raise ValueError(f"copied correction hash differs: {entry['path']}")
    receipt.update(complete=True, amended_tree=git_tree_id(amended, copied_modes)[0].hex())
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def main():
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", type=Path, default=root / "test262/vendor/test262")
    parser.add_argument("--errata", type=Path, default=root / "test262/errata")
    parser.add_argument("--pins", type=Path,
                        default=root / "crates/scriptw-test262/src/source-pins.json")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        receipt = prepare(args.suite, args.errata, args.pins, args.output)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f"test262-errata: {error}\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
