#!/usr/bin/env python3
"""Require a complete passing Wasm-AOT shard report; never decode snapshot JSON here."""
from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys

KINDS = ("Parser", "EarlyError", "Lowering", "Runtime", "WasmBackend", "HostHarness", "Unsupported")
FIELDS = ("execution_backend", "shard", "total", "passed", "failed", *KINDS)
DECIMAL = re.compile(r"(?:0|[1-9][0-9]{0,18})\Z")
PROGRESS = re.compile(r"test262 checkpoint: ([0-9]+)/([0-9]+) cases\Z")


def count(value: str, name: str) -> int:
    if not DECIMAL.fullmatch(value):
        raise ValueError(f"{name}: expected a canonical non-negative decimal count")
    return int(value)


def selected_count(inventory: str, index: int, shards: int) -> int:
    # `test262 list` reports the complete discovered execution count but only
    # displays the first 50 IDs. Never treat that display as a complete ID set.
    # shard_cases in lila-test262 owns selection, in index % shard_count order.
    if shards < 1 or not 1 <= index <= shards:
        raise ValueError("shard selector must be one-based and within its shard count")
    lines = inventory.splitlines()
    if not lines or not lines[0].startswith("count: "):
        raise ValueError("inventory must start with its complete discovered count")
    if sum(line.startswith("count:") for line in lines) != 1:
        raise ValueError("inventory must contain exactly one discovered count")
    total = count(lines[0][len("count: "):], "inventory count")
    selected = (total + shards - index) // shards
    if total == 0 or selected == 0:
        raise ValueError("discovered inventory and selected shard must both be nonempty")
    return selected


def validate_summary(report: str, inventory: str, index: int, shards: int) -> int:
    expected = selected_count(inventory, index, shards)
    fields: dict[str, str] = {}
    previous_progress = 0
    for line in report.splitlines():
        if not line:
            continue
        progress = PROGRESS.fullmatch(line)
        if progress:
            completed, total = map(int, progress.groups())
            if fields or total != expected or not previous_progress < completed <= total:
                raise ValueError(f"inconsistent or out-of-order progress: {line}")
            previous_progress = completed
            continue
        key, separator, value = line.partition(": ")
        if not separator or key not in FIELDS or key in fields:
            raise ValueError(f"unknown, malformed or duplicate report field: {line!r}")
        fields[key] = value
    if set(fields) != set(FIELDS):
        raise ValueError(f"missing report fields: {sorted(set(FIELDS) - set(fields))}")
    if fields["execution_backend"] != "wasm-aot":
        raise ValueError("report must name the Wasm-AOT product backend")
    if fields["shard"] != f"{index}/{shards}":
        raise ValueError("report shard differs from the requested one-based selection")
    for key in ("total", "passed"):
        if count(fields[key], key) != expected:
            raise ValueError(f"{key} must equal the complete expected shard count {expected}")
    for key in ("failed", *KINDS):
        if count(fields[key], key) != 0:
            raise ValueError(f"{key} must be zero")
    return expected


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--inventory", required=True, type=Path)
    parser.add_argument("--shard-index", required=True, type=int)
    parser.add_argument("--shard-count", required=True, type=int)
    args = parser.parse_args()
    try:
        selected = validate_summary(args.report.read_text(encoding="utf-8"),
                                    args.inventory.read_text(encoding="utf-8"),
                                    args.shard_index, args.shard_count)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"Test262 shard summary failed: {error}", file=sys.stderr)
        return 1
    print(f"Verified Wasm-AOT shard {args.shard_index}/{args.shard_count}: "
          f"{selected}/{selected} passed; every failure bucket is zero")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
