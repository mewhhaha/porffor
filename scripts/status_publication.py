"""Resolve one committed JSON/text status generation and verify both members."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

SNAPSHOT = "test262/snapshots/published-status-wasm-aot.json"
MANIFEST = "test262/snapshots/published-status-wasm-aot.manifest.json"
DOMAIN = b"scriptw-status-generation-v1\0"


def generation_id(json_bytes, text_bytes):
    digest = hashlib.sha256(DOMAIN)
    for data in (json_bytes, text_bytes):
        digest.update(len(data).to_bytes(8, "little"))
        digest.update(data)
    return digest.hexdigest()


def read_local_or_index(root, name):
    path = root / name
    if path.is_symlink():
        raise ValueError(f"status evidence must be a regular file: {name}")
    try:
        return path.read_bytes()
    except FileNotFoundError:
        result = subprocess.run(
            ["git", "show", ":" + name], cwd=root, capture_output=True
        )
        if result.returncode:
            raise FileNotFoundError(name) from None
        return result.stdout


def read_committed(root, name):
    path = root / name
    if not path.is_file() or path.is_symlink():
        raise ValueError(f"status evidence must be a regular file: {name}")
    entry = subprocess.check_output(
        ["git", "ls-tree", "HEAD", "--", name], cwd=root, text=True
    ).strip()
    metadata, separator, recorded = entry.partition("\t")
    if not separator or recorded != name or not metadata.startswith("100644 blob "):
        raise ValueError(f"status evidence must be a committed regular blob: {name}")
    return subprocess.check_output(["git", "show", "HEAD:" + name], cwd=root)


def read_generation(
    root, manifest_name=MANIFEST, read=read_local_or_index, manifest_bytes=None
):
    manifest = json.loads(
        read(root, manifest_name) if manifest_bytes is None else manifest_bytes
    )
    if (
        not isinstance(manifest, dict)
        or set(manifest) != {"schema_version", "backend", "generation", "json", "text"}
        or type(manifest["schema_version"]) is not int
        or manifest["schema_version"] != 1
        or manifest["backend"] != "wasm-aot"
        or not isinstance(manifest["generation"], str)
        or not re.fullmatch(r"[0-9a-f]{64}", manifest["generation"])
    ):
        raise ValueError("invalid status generation manifest")
    data, paths = [], []
    for field, suffix in (("json", "json"), ("text", "txt")):
        member = manifest[field]
        expected = f"published-status-wasm-aot-{manifest['generation']}.{suffix}"
        if not isinstance(member, dict) or set(member) != {"file", "sha256"}:
            raise ValueError("invalid status generation member")
        if member["file"] != expected:
            raise ValueError("status member must name its own immutable generation")
        path = (Path(manifest_name).parent / expected).as_posix()
        payload = read(root, path)
        if hashlib.sha256(payload).hexdigest() != member["sha256"]:
            raise ValueError(f"status member digest mismatch: {path}")
        data.append(payload)
        paths.append(path)
    if generation_id(*data) != manifest["generation"]:
        raise ValueError("status generation does not bind both members")
    publication = json.loads(data[0])
    if (
        not isinstance(publication, dict)
        or publication.get("producer") != "scriptw"
        or not isinstance(publication.get("real_suite"), dict)
        or publication["real_suite"].get("backend") != "wasm-aot"
    ):
        raise ValueError(
            "status generation requires Scriptw wasm-aot publication evidence"
        )
    return publication, paths


def load_publication(root):
    try:
        manifest_bytes = read_local_or_index(root, MANIFEST)
    except FileNotFoundError:
        # Existing historical publication pairs remain readable. Once a
        # manifest exists, damage must fail rather than select that old pair.
        return json.loads(read_local_or_index(root, SNAPSHOT)), SNAPSHOT
    publication, paths = read_generation(root, manifest_bytes=manifest_bytes)
    return publication, paths[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--check-committed", action="store_true")
    args = parser.parse_args()
    _, paths = read_generation(
        args.root, read=read_committed if args.check_committed else read_local_or_index
    )
    for path in paths:
        print(path)


if __name__ == "__main__":
    main()
