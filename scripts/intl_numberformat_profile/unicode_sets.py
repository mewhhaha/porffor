"""Pinned Unicode16 property ranges and the reached closed UnicodeSet grammar."""
from pathlib import Path
import re


def normalize(ranges):
    result = []
    for start, end in sorted(ranges):
        if not 0 <= start <= end <= 0x10FFFF:
            raise ValueError(f"invalid Unicode range: {start}..{end}")
        if result and start <= result[-1][1] + 1:
            result[-1] = (result[-1][0], max(end, result[-1][1]))
        else:
            result.append((start, end))
    return result


def complement(ranges):
    result = []
    next_start = 0
    for start, end in ranges:
        if next_start < start:
            result.append((next_start, start - 1))
        next_start = end + 1
    if next_start <= 0x10FFFF:
        result.append((next_start, 0x10FFFF))
    return result


def intersection(left, right):
    result = []
    i = j = 0
    while i < len(left) and j < len(right):
        start = max(left[i][0], right[j][0])
        end = min(left[i][1], right[j][1])
        if start <= end:
            result.append((start, end))
        if left[i][1] < right[j][1]:
            i += 1
        else:
            j += 1
    return result


def properties(source_directory):
    source_directory = Path(source_directory)
    tables = {name: [] for name in ["L", "S", "Z", "Nd", "White_Space"]}
    pending = None
    for line in (source_directory / "UnicodeData.txt").read_text().splitlines():
        fields = line.split(";")
        point, name, category = int(fields[0], 16), fields[1], fields[2]
        if name.endswith(", First>"):
            if pending is not None:
                raise ValueError("nested UnicodeData range")
            pending = (point, category)
            continue
        start = point
        if name.endswith(", Last>"):
            if pending is None or pending[1] != category:
                raise ValueError("mismatched UnicodeData range")
            start = pending[0]
            pending = None
        for key in (category, category[0]):
            if key in tables:
                tables[key].append((start, point))
    if pending is not None:
        raise ValueError("unclosed UnicodeData range")
    for line in (source_directory / "PropList.txt").read_text().splitlines():
        line = line.partition("#")[0].strip()
        if not line:
            continue
        spelling, prop = (part.strip() for part in line.split(";"))
        if prop == "White_Space":
            ends = spelling.split("..")
            tables[prop].append((int(ends[0], 16), int(ends[-1], 16)))
    return {name: normalize(rows) for name, rows in tables.items()}


class Parser:
    def __init__(self, source, properties):
        self.source = source
        self.position = 0
        self.properties = properties

    def parse(self):
        result = self.term()
        if self.position != len(self.source):
            raise ValueError(f"unconsumed UnicodeSet syntax: {self.source!r}")
        return result

    def term(self):
        if self.source.startswith("[:", self.position):
            end = self.source.find(":]", self.position + 2)
            if end < 0:
                raise ValueError(f"unclosed UnicodeSet property: {self.source!r}")
            name = self.source[self.position + 2:end]
            self.position = end + 2
            negate = name.startswith("^")
            name = name.removeprefix("^")
            name = {"digit": "Nd", "Letter": "L", "Symbol": "S", "Separator": "Z", "Decimal_Number": "Nd"}.get(name, name)
            if name not in self.properties:
                raise ValueError(f"unresolved UnicodeSet property: {name}")
            value = self.properties[name]
            return complement(value) if negate else value
        if self.position >= len(self.source) or self.source[self.position] != "[":
            raise ValueError(f"unresolved UnicodeSet term: {self.source!r}")
        self.position += 1
        negate = self.position < len(self.source) and self.source[self.position] == "^"
        if negate:
            self.position += 1
        value = None
        operation = "union"
        while self.position < len(self.source) and self.source[self.position] != "]":
            if self.source[self.position] == "&":
                if value is None or operation != "union":
                    raise ValueError(f"missing UnicodeSet intersection operand: {self.source!r}")
                operation = "intersection"
                self.position += 1
                continue
            term = self.term()
            if value is None:
                value = term
            elif operation == "intersection":
                value = intersection(value, term)
            else:
                value = normalize([*value, *term])
            operation = "union"
        if self.position == len(self.source) or operation != "union" or value is None:
            raise ValueError(f"incomplete UnicodeSet: {self.source!r}")
        self.position += 1
        return complement(value) if negate else value


def parse(source, property_tables):
    return Parser(source, property_tables).parse()
