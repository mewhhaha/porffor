#!/usr/bin/env python3
"""Check scratch headroom before starting a verification command.

Example: python3 scripts/check-scratch-space.py --scratch /tmp --minimum-free-mib 512 -- python3 scripts/limited_verification.py --cloud -- cargo test -p scriptw-engine

This checks available space at launch; it does not reserve it against other
processes. Set the requirement for the command's peak outputs and caches.
"""
import argparse
import os
from pathlib import Path
import sys
import tempfile

from filesystem_probe import probe_filesystem
from resource_numbers import positive_decimal


def positive_mib(value: str) -> int:
    try:
        return positive_decimal(value, "MiB count")
    except ValueError as error:
        raise argparse.ArgumentTypeError("expected a positive MiB count") from error


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scratch", type=Path, default=Path(tempfile.gettempdir()))
    parser.add_argument("--minimum-free-mib", type=positive_mib, default=512)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        parser.error("a verification command is required after --")
    try:
        scratch, _, _, available = probe_filesystem(
            args.scratch, require_directory=True
        )
    except (OSError, ValueError) as error:
        parser.error(str(error))
    required = args.minimum_free_mib * 1024 * 1024
    if available < required:
        print(
            f"scratch admission: {scratch} has {available // (1024 * 1024)} MiB "
            f"available; {args.minimum_free_mib} MiB required. "
            "Command was not started.",
            file=sys.stderr,
        )
        return 1
    print(
        f"scratch admission: {scratch} has {available // (1024 * 1024)} MiB "
        f"available; {args.minimum_free_mib} MiB required.",
        file=sys.stderr,
        flush=True,
    )
    environment = dict(os.environ, TMPDIR=str(scratch))
    os.execvpe(command[0], command, environment)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
