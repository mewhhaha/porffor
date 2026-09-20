#!/usr/bin/env python3
"""Serialize validated canonical CLDR number profiles into the closed Rust payload."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import struct

from profile_schema import validate

STAGE = Path(__file__).resolve().parents[1]
TOKEN_NAMES = ["Literal", "Number", "MinusSign", "PlusSign", "PercentSign", "Currency", "Compact", "Unit", "Argument1"]
CATEGORIES = ["zero", "one", "two", "few", "many", "other"]
OPERANDS = ["n", "i", "v", "w", "f", "t", "c", "e"]
SYMBOLS = ["decimal", "group", "plusSign", "minusSign", "percentSign", "approximatelySign", "exponential", "infinity", "nan", "currencyDecimal", "currencyGroup"]
NUMBER_PATTERNS = ["decimal", "percent", "currency", "accounting", "currency_alpha", "accounting_alpha"]


class Writer:
    def __init__(self):
        self.content = bytearray(b"LNF47\0\x01\0")

    def u8(self, value):
        self.content.extend(struct.pack("<B", value))

    def u32(self, value):
        self.content.extend(struct.pack("<I", value))

    def u64(self, value):
        self.content.extend(struct.pack("<Q", value))

    def text(self, value):
        raw = value.encode()
        self.u32(len(raw))
        self.content.extend(raw)

    def currency(self, code):
        assert len(code) == 3 and code.isascii() and code.isupper()
        self.content.extend(code.encode())

    def option(self, value, emit):
        self.u8(value is not None)
        if value is not None:
            emit(value)

    def sequence(self, values, emit):
        self.u32(len(values))
        for value in values:
            emit(value)


def encode(profile):
    validate(profile)
    tables = profile["tables"]
    writer = Writer()
    choices = {kind: [] for kind in ["pattern", "string", "compact"]}
    remap = {}
    for original, row in enumerate(tables["choices"]):
        kind = row["kind"]
        remap[original] = (kind, len(choices[kind]))
        choices[kind].append(row["values"])

    def choice_id(original, expected):
        kind, index = remap[original]
        assert kind == expected, (kind, expected, original)
        writer.u32(index)

    writer.sequence(tables["strings"], writer.text)

    def token(row):
        kind, text = row
        writer.u8(TOKEN_NAMES.index(kind))
        if kind in {"Literal", "Compact", "Unit"}:
            writer.u32(text)
        else:
            assert text == 0, row

    writer.sequence(tables["patterns"], lambda tokens: writer.sequence(tokens, token))

    def signed(row):
        writer.u32(row["positive"])
        writer.u32(row["negative"])
        skeleton = row["skeleton"]
        writer.u8(skeleton is not None)
        writer.u8(0 if skeleton is None else skeleton["primary_group"])
        writer.u8(0 if skeleton is None else skeleton["secondary_group"])

    writer.sequence(tables["signed_patterns"], signed)

    def variants(row, emit):
        assert len(row) == 8 and all(value is not None for value in row[:6])
        for value in row[:6]:
            emit(value)
        for value in row[6:]:
            writer.option(value, emit)

    writer.sequence(choices["pattern"], lambda row: variants(row, writer.u32))
    writer.sequence(choices["string"], lambda row: variants(row, writer.u32))

    def compact_pattern(value):
        writer.u8(value["fallback"])
        if not value["fallback"]:
            writer.u32(value["pattern"])

    writer.sequence(choices["compact"], lambda row: variants(row, compact_pattern))
    writer.sequence(tables["symbols"], lambda row: [writer.u32(row[name]) for name in SYMBOLS])
    writer.sequence(tables["unicode_sets"], lambda rows: writer.sequence(rows, lambda row: [writer.u32(value) for value in row]))

    def compact_row(row):
        writer.u32(row["magnitude"])
        writer.u32(row["exponent"])
        choice_id(row["choices"], "compact")

    writer.sequence(tables["compact_sets"], lambda rows: writer.sequence(rows, compact_row))

    def numbering(row):
        writer.u32(row["symbols"])
        for name in NUMBER_PATTERNS:
            writer.u32(row["patterns"][name])
        writer.u8(row["minimum_grouping"])
        for name in ["compact_short", "compact_long", "compact_currency", "compact_currency_alpha"]:
            writer.u32(row[name])
        writer.u32(row["range_pattern"])
        choice_id(row["currency_unit_pattern"], "pattern")
        for side in ["beforeCurrency", "afterCurrency"]:
            for name in ["currencyMatch", "surroundingMatch", "insertBetween"]:
                writer.u32(row["currency_spacing"][side][name])

        def override(value):
            writer.currency(value["code"])
            for name in ["decimal", "group", "pattern"]:
                writer.option(value[name], writer.u32)

        writer.sequence(row["currency_overrides"], override)

    writer.sequence(tables["numbering_profiles"], numbering)

    def currency(row):
        writer.currency(row["code"])
        writer.u32(row["symbol"])
        writer.u32(row["narrow"])
        choice_id(row["names"], "string")

    writer.sequence(tables["currency_sets"], lambda rows: writer.sequence(rows, currency))

    def unit_set(row):
        def unit(value):
            choice_id(value["choices"], "pattern")
            writer.option(value["per_unit"], writer.u32)
            writer.u32(value["denominator"])

        def pair(value):
            writer.u8(value["numerator"])
            writer.u8(value["denominator"])
            choice_id(value["choices"], "pattern")

        writer.sequence(row["simple"], unit)
        writer.sequence(row["pairs"], pair)
        writer.u32(row["per_pattern"])

    writer.sequence(tables["unit_sets"], unit_set)

    def plural_relation(row):
        writer.u8(OPERANDS.index(row["operand"]))
        writer.u64(row["modulus"] or 0)
        writer.u8(row["integer_only"])
        writer.u8(row["negate"])
        writer.sequence(row["ranges"], lambda pair: [writer.u64(value) for value in pair])

    def plural_rule(row):
        name, alternatives = row
        writer.u8(CATEGORIES.index(name))
        writer.sequence(alternatives, lambda conjunction: writer.sequence(conjunction, plural_relation))

    writer.sequence(tables["plural_rules"], lambda rows: writer.sequence(rows, plural_rule))
    writer.sequence(tables["plural_ranges"], lambda row: [writer.u8(CATEGORIES.index(value)) for value in row])

    def locale_profile(row):
        writer.u8(row["default_numbering"])
        writer.sequence(row["numbering"], writer.u32)
        for name in ["plural_rules", "plural_ranges", "currencies"]:
            writer.u32(row[name])
        assert len(row["units"]) == 3
        for unit in row["units"]:
            writer.u32(unit)

    writer.sequence(tables["profiles"], locale_profile)
    writer.sequence(sorted(profile["locales"].items()), lambda row: (writer.text(row[0]), writer.u32(row[1]["profile"])))

    def system(row):
        writer.text(row["name"])
        assert len(row["digits"]) == 10
        for digit in row["digits"]:
            writer.u32(ord(digit))

    writer.sequence(profile["numbering_systems"], system)
    writer.sequence(profile["sanctioned_units"], writer.text)
    writer.u8(profile["currency_fractions"]["default"])
    writer.sequence(profile["currency_fractions"]["overrides"], lambda row: (writer.currency(row[0]), writer.u8(row[1])))
    for property_name in ["L", "Nd", "White_Space"]:
        writer.u32(profile["unicode_properties"][property_name])
    return bytes(writer.content)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    directory = STAGE / "work/crates/lila-intl/data/number-cldr-47"
    canonical_bytes = (directory / "profiles.json.gz").read_bytes()
    profile = json.loads(gzip.decompress(canonical_bytes))
    if not profile["complete"]:
        raise ValueError("a diagnostic inventory cannot be published as a provider payload")
    if profile["cldr_commit"] != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c" or profile["schema"] != 1:
        raise ValueError("review changed canonical profile schema")
    payload = encode(profile)
    digest = hashlib.sha256(payload).hexdigest()
    samples = json.loads((directory / "plural-samples.json").read_text())
    sample_rows = []
    for group in samples:
        for sample in group["samples"]:
            sample_rows.append(f'    ({group["rule"]}, "{sample["spelling"]}", CardinalCategory::{sample["category"].title()}),')
    sample_source = "// Generated pinned CLDR47 cardinal-rule sample endpoints.\nuse super::CardinalCategory;\npub(super) const SAMPLES: &[(usize, &str, CardinalCategory)] = &[\n" + "\n".join(sample_rows) + "\n];\n"
    products = {
        directory / "profiles.bin": payload,
        STAGE / "work/crates/lila-intl/src/number_format/tests/plural_samples.rs": sample_source.encode(),
        directory / "payload-manifest.json": (json.dumps({"schema": 1, "canonical_sha256": hashlib.sha256(canonical_bytes).hexdigest(), "payload_sha256": digest, "payload_bytes": len(payload), "locales": len(profile["locales"]), "numbering_systems": len(profile["numbering_systems"])}, indent=2) + "\n").encode(),
        STAGE / "work/crates/lila-intl/src/number_format/profiles/fingerprint.rs": (
            "// Generated from pinned CLDR47; scripts/generate-intl-numberformat-profile.py.\n"
            f'pub const NUMBER_FORMAT_DATA_SHA256: &str =\n    "{digest}";\n'
        ).encode(),
    }
    for path, content in products.items():
        if args.check:
            if path.read_bytes() != content:
                raise ValueError(f"generated payload differs: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
    print(json.dumps({"payload_sha256": digest, "payload_bytes": len(payload), "locales": len(profile["locales"])}))


if __name__ == "__main__":
    main()
