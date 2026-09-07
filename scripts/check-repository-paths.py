#!/usr/bin/env python3
"""Reject tracked paths that cannot be checked out unambiguously on supported hosts."""
from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import sys
import unicodedata
from collections.abc import Iterable

INVALID = frozenset('<>:"\\|?*')
RESERVED = {"con", "prn", "aux", "nul", "conin$", "conout$"} | {
    f"{prefix}{digit}" for prefix in ("com", "lpt") for digit in "123456789\u00b9\u00b2\u00b3"
}


def check_paths(paths: Iterable[str]) -> list[str]:
    """Check components and aliases, including differently cased parent directories."""
    errors: set[str] = set()
    seen: dict[str, str] = {}
    for path in sorted(set(paths)):
        parts = path.split("/")
        for index, part in enumerate(parts):
            prefix = "/".join(parts[: index + 1])
            if not part or part in (".", ".."):
                errors.add(f"{path!r}: empty or relative path component")
                continue
            if any(char in INVALID or ord(char) < 32 for char in part):
                errors.add(f"{path!r}: Windows-invalid character in {part!r}")
            if part.endswith((" ", ".")):
                errors.add(f"{path!r}: component ends in a space or period")
            if part.split(".", 1)[0].rstrip(" ").casefold() in RESERVED:
                errors.add(f"{path!r}: reserved Windows device name {part!r}")
            try:
                units = len(part.encode("utf-16-le")) // 2
            except UnicodeEncodeError:
                errors.add(f"{path!r}: path is not valid Unicode")
            else:
                if units > 255:
                    errors.add(f"{path!r}: component exceeds 255 UTF-16 code units")
            identity = unicodedata.normalize("NFC", prefix).casefold()
            previous = seen.setdefault(identity, prefix)
            if previous != prefix:
                errors.add(f"{previous!r} and {prefix!r}: case/Unicode path collision")
    return sorted(errors)


def tracked_paths(root: Path) -> list[str]:
    # NUL separation preserves spaces, newlines and non-ASCII paths. Inspect the
    # index, not just files present on this machine or the current diff.
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "--cached", "--full-name", "-z"],
        check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    return [item.decode("utf-8", "surrogateescape") for item in result.stdout.split(b"\0") if item]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    try:
        paths = tracked_paths(args.root)
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"Cannot inspect the Git index: {error}", file=sys.stderr)
        return 2
    if not paths:
        print("Refusing to pass an empty tracked-path inventory", file=sys.stderr)
        return 1
    errors = check_paths(paths)
    if errors:
        print("Repository path portability failed:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1
    print(f"Repository paths: {len(set(paths))} tracked files checked")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
