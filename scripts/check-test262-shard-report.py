#!/usr/bin/env python3
"""Verify a complete all-passing shard against the CLI's execution inventory."""
from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys

FAILURE_KINDS = ('Parser', 'EarlyError', 'Lowering', 'Runtime', 'WasmBackend', 'HostHarness', 'Unsupported')
FIELDS = ('execution_backend', 'shard', 'total', 'passed', 'failed', *FAILURE_KINDS)
COUNT = re.compile(r'0|[1-9][0-9]*')


def integer(text: str) -> int:
    if not COUNT.fullmatch(text) or len(text) > 20:
        raise ValueError(f'noncanonical or oversized count {text!r}')
    value = int(text)
    if value > 2**64 - 1:
        raise ValueError('count exceeds the supported 64-bit domain')
    return value


def inventory_total(text: str) -> int:
    # `test262 list` prints execution identities (not physical filenames), then
    # a truthful truncation footer after its first 50 entries. Do not double
    # unflagged files here: the discovery manifest already expands their modes.
    lines = text.splitlines()
    if not lines or not lines[0].startswith('count: '):
        raise ValueError('missing execution inventory header')
    total = integer(lines[0][len('count: '):])
    if not total:
        raise ValueError('execution inventory is empty')
    shown = min(total, 50)
    expected_lines = 1 + shown + int(total > 50)
    if len(lines) != expected_lines:
        raise ValueError('truncated or oversized execution inventory')
    entries = lines[1:1 + shown]
    if len(set(entries)) != shown:
        raise ValueError('duplicate displayed execution identity')
    for entry in entries:
        mode, separator, path = entry.partition(':')
        if mode not in ('sloppy-script', 'strict-script', 'raw-script', 'module', 'raw-module') or not separator or not path:
            raise ValueError(f'invalid execution identity {entry!r}')
    if total > 50 and lines[-1] != f'... {total - 50} more':
        raise ValueError('inventory truncation footer disagrees with its total')
    return total


def expected_shard(total: int, selector: str) -> int:
    parts = selector.split('/')
    if len(parts) != 2:
        raise ValueError('shard must be a one-based index/count pair')
    index, count = map(integer, parts)
    if not 1 <= index <= count or not total:
        raise ValueError('invalid shard selector or empty inventory')
    # The native shard_cases implementation selects enumerate-index % count.
    quotient, remainder = divmod(total, count)
    expected = quotient + int(index <= remainder)
    if not expected:
        raise ValueError('the selected shard would contain no executions')
    return expected


def verify(inventory: str, report: str, selector: str) -> int:
    expected = expected_shard(inventory_total(inventory), selector)
    lines = report.splitlines()
    starts = [index for index, line in enumerate(lines) if line.startswith('execution_backend:')]
    if len(starts) != 1:
        raise ValueError('report must contain exactly one backend summary')
    start = starts[0]
    # Progress output may precede the final summary, but it cannot impersonate
    # count fields. Unknown/additional fields after it require a parser update.
    if any(any(line.startswith(key + ':') for key in FIELDS) for line in lines[:start]):
        raise ValueError('duplicate or out-of-order summary fields')
    summary = lines[start:]
    if len(summary) != len(FIELDS):
        raise ValueError('incomplete, duplicate or unexpected summary fields')
    values = {}
    for key, line in zip(FIELDS, summary):
        prefix = key + ': '
        if not line.startswith(prefix):
            raise ValueError(f'missing or reordered summary field {key}')
        values[key] = line[len(prefix):]
    if values['execution_backend'] != 'wasm-aot' or values['shard'] != selector:
        raise ValueError('backend or shard identity mismatch')
    numbers = {key: integer(values[key]) for key in FIELDS[2:]}
    if numbers['total'] != expected or numbers['passed'] != expected:
        raise ValueError(f'shard must pass exactly {expected} discovered executions')
    if numbers['failed'] or any(numbers[key] for key in FAILURE_KINDS):
        raise ValueError('shard contains a non-success result')
    return expected


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inventory', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--shard', required=True)
    args = parser.parse_args()
    try:
        count = verify(args.inventory.read_text(encoding='utf-8'),
                       args.report.read_text(encoding='utf-8'), args.shard)
    except (OSError, UnicodeError, ValueError) as error:
        print(f'shard-report: {error}', file=sys.stderr)
        return 1
    print(f'shard-report: verified {count}/{count} Wasm-AOT executions for {args.shard}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
