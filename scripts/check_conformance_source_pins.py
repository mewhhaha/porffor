#!/usr/bin/env python3
"""Bind declared Test262 upstream provenance to the actual vendored Git tree."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys


def git_object_id(kind, data):
    digest = hashlib.sha1(usedforsecurity=False)
    digest.update(kind + b" " + str(len(data)).encode("ascii") + b"\0")
    digest.update(data)
    return digest.digest()


def git_blob_id(path):
    digest = hashlib.sha1(usedforsecurity=False)
    digest.update(b"blob " + str(path.stat().st_size).encode("ascii") + b"\0")
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.digest()


def git_index_modes(root):
    repo = Path(subprocess.check_output(
        ["git", "-C", str(root), "rev-parse", "--show-toplevel"],
        text=True,
    ).strip()).resolve()
    prefix = root.resolve().relative_to(repo).as_posix()
    output = subprocess.check_output([
        "git", "-C", str(repo), "ls-files", "--stage", "-z", "--", prefix,
    ])
    modes = {}
    for record in output.split(b"\0"):
        if not record:
            continue
        metadata, name = record.split(b"\t", 1)
        mode, _blob, stage = metadata.split()
        if stage != b"0" or mode not in {b"100644", b"100755", b"120000"}:
            raise ValueError("suite requires a resolved Git index with regular files or symlinks")
        path = repo / os.fsdecode(name)
        if path in modes:
            raise ValueError(f"ambiguous indexed suite path: {path}")
        modes[path] = mode
    actual = set()
    for directory, subdirs, files in os.walk(root.resolve(), followlinks=False):
        subdirs[:] = [name for name in subdirs if name != ".git"]
        for name in subdirs[:]:
            path = Path(directory) / name
            if path.is_symlink():
                actual.add(path)
                subdirs.remove(name)
        actual.update(Path(directory) / name for name in files if name != ".git")
    if not modes or set(modes) != actual:
        raise ValueError("suite worktree paths must match its complete Git index on Windows")
    return modes


def git_tree_id(root, indexed_modes=None):
    entries = []
    for path in root.iterdir():
        if path.name == ".git":
            continue
        mode = path.lstat().st_mode
        name = os.fsencode(path.name)
        if stat.S_ISDIR(mode):
            oid, populated = git_tree_id(path, indexed_modes)
            if not populated:
                continue
            entries.append((name + b"/", b"40000 " + name + b"\0" + oid))
        elif stat.S_ISLNK(mode):
            if indexed_modes is not None and indexed_modes[path] != b"120000":
                raise ValueError(f"symlink differs from indexed file kind: {path}")
            oid = git_object_id(b"blob", os.fsencode(os.readlink(path)))
            entries.append((name, b"120000 " + name + b"\0" + oid))
        elif stat.S_ISREG(mode):
            wire_mode = (indexed_modes[path] if indexed_modes is not None
                         else b"100755" if mode & stat.S_IXUSR else b"100644")
            entries.append((name, wire_mode + b" " + name + b"\0" + git_blob_id(path)))
        else:
            raise ValueError(f"unsupported suite file type: {path}")
    contents = b"".join(entry for _, entry in sorted(entries))
    return git_object_id(b"tree", contents), bool(entries)


def load_pins(path):
    def unique_fields(pairs):
        result = {}
        for name, value in pairs:
            if name in result:
                raise ValueError(f"duplicate source pin field: {name}")
            result[name] = value
        return result

    pins = json.loads(path.read_text(), object_pairs_hook=unique_fields)
    expected = {"ecma262", "test262_upstream", "test262_tree"}
    if not isinstance(pins, dict) or set(pins) != expected:
        raise ValueError("source pins must contain exactly ecma262, test262_upstream and test262_tree")
    for field, value in pins.items():
        if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{40}", value) is None:
            raise ValueError(f"{field} must identify an exact lowercase Git SHA-1 source revision")
    return pins


def verify(pins_path, suite_root):
    pins = load_pins(pins_path)
    suite_root = suite_root.resolve()
    # Windows checkouts retain executable modes in Git's index. A symlink may
    # likewise be a regular file containing its link target when core.symlinks
    # is false. Hash actual bytes, with a complete indexed path/mode binding.
    modes = git_index_modes(suite_root) if sys.platform == "win32" else None
    actual, populated = git_tree_id(suite_root, modes)
    if not populated or actual.hex() != pins["test262_tree"]:
        raise ValueError(f"vendored Test262 tree {actual.hex()} differs from declared {pins['test262_tree']}")
    return {**pins, "vendored_tree_verified": True}


def main():
    repo = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pins", type=Path, default=repo / "crates/lila-test262/src/source-pins.json")
    parser.add_argument("--suite", type=Path, default=repo / "test262/vendor/test262")
    args = parser.parse_args()
    try:
        result = verify(args.pins, args.suite)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"conformance-source-pins: {error}\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
