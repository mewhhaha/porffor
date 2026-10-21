#!/usr/bin/env python3
"""Inspect top-level evidence metadata with bounded memory.

This is an inspection tool, not an evidence validator. Omitted values are
scanned for framing without decoding their payloads or verifying their claims.
"""
import argparse
import json
from pathlib import Path


def characters(stream):
    while chunk := stream.read(65536):
        yield from chunk


def nonspace(chars):
    return next(character for character in chars if not character.isspace())


def value(chars, first, limit=None):
    captured = [] if limit is not None else None
    depth = []
    quoted = escaped = False
    character = first
    while True:
        if not quoted and not depth and character in ",}":
            return ("".join(captured) if captured is not None else None), character
        if captured is not None:
            if len(captured) >= limit:
                raise ValueError("selected metadata field exceeds the capture limit")
            captured.append(character)
        if quoted:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                quoted = False
        elif character == '"':
            quoted = True
        elif character in "[{":
            depth.append("]" if character == "[" else "}")
        elif character in "]}":
            if not depth or depth.pop() != character:
                raise ValueError("unbalanced evidence framing")
        character = next(chars)


def inspect_metadata(path, fields, max_field_chars=65536):
    if max_field_chars < 1:
        raise ValueError("metadata capture limit must be positive")
    fields = set(fields)
    selected = {}
    seen = set()
    try:
        with Path(path).open(encoding="utf-8") as stream:
            chars = characters(stream)
            if nonspace(chars) != "{":
                raise ValueError("evidence must have a top-level object")
            first = nonspace(chars)
            if first == "}":
                if any(not character.isspace() for character in chars):
                    raise ValueError("trailing evidence content")
                return selected
            while True:
                if first != '"':
                    raise ValueError("expected a top-level field name")
                name = ['"']
                escaped = False
                for character in chars:
                    name.append(character)
                    if len(name) > max_field_chars:
                        raise ValueError(
                            "metadata field name exceeds the capture limit"
                        )
                    if character == '"' and not escaped:
                        break
                    escaped = character == "\\" and not escaped
                key = json.loads("".join(name))
                if key in seen:
                    raise ValueError("duplicate top-level evidence field")
                seen.add(key)
                if len(seen) > 4096:
                    raise ValueError("too many top-level evidence fields")
                if nonspace(chars) != ":":
                    raise ValueError("expected a metadata field separator")
                raw, delimiter = value(
                    chars, nonspace(chars), max_field_chars if key in fields else None
                )
                if key in fields:
                    selected[key] = json.loads(raw)
                if delimiter == "}":
                    if any(not character.isspace() for character in chars):
                        raise ValueError("trailing evidence content")
                    return selected
                first = nonspace(chars)
    except (StopIteration, RuntimeError) as error:
        raise ValueError("truncated evidence framing") from error


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", type=Path)
    parser.add_argument("--field", action="append")
    parser.add_argument("--max-field-chars", type=int, default=65536)
    args = parser.parse_args()
    fields = args.field or [
        "schema",
        "head",
        "profile",
        "complete",
        "passed",
        "total",
        "source_revision",
        "source_unchanged",
        "counts",
        "status",
    ]
    try:
        metadata = inspect_metadata(args.path, fields, args.max_field_chars)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    print(
        json.dumps(
            {
                "inspection_only": True,
                "file_bytes": args.path.stat().st_size,
                "metadata": metadata,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
