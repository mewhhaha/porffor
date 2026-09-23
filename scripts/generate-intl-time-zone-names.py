#!/usr/bin/env python3
"""Generate exact English time-zone names from pinned CLDR 47 and ICU country data."""

import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

REPOSITORY = Path(__file__).resolve().parents[1]
SOURCE_PATH = "crates/lila-intl/data/zone-names-cldr-47"
OUTPUT_PATH = "crates/lila-intl/src/provider/time_zone_names/generated.rs"
WIDTHS = ("short", "long")
KINDS = ("generic", "standard", "daylight")
CLDR_COMMIT = "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
ICU_COMMIT = "457157a92aa053e632cc7fcfd0e12f8a943b2d11"
EPOCH = datetime(1970, 1, 1, tzinfo=timezone.utc)
UNBOUNDED_START = -(2**63)
UNBOUNDED_END = 2**63 - 1


def rust_string(value):
    if any(ord(char) < 32 for char in value):
        raise ValueError(f"control character in generated string: {value!r}")
    return json.dumps(value, ensure_ascii=False)


def checked_sources(source_dir):
    manifest_bytes = (source_dir / "manifest.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    if manifest["release"] != "47.0.0" or manifest["commit"] != CLDR_COMMIT:
        raise ValueError("review the pinned CLDR version before regeneration")
    if manifest["country_authority"]["commit"] != ICU_COMMIT:
        raise ValueError("review the pinned ICU country version before regeneration")
    sources = {}
    for entry in manifest["files"]:
        path = entry["path"]
        if path in sources or Path(path).is_absolute() or ".." in Path(path).parts:
            raise ValueError(f"invalid or duplicate source path: {path}")
        contents = (source_dir / path).read_bytes()
        if (hashlib.sha256(contents).hexdigest() != entry["sha256"]
                or len(contents) != entry["bytes"]):
            raise ValueError(f"upstream source checksum mismatch: {path}")
        sources[path] = contents
    expected = {
        "common/main/root.xml", "common/main/en.xml", "common/main/en_US.xml",
        "common/bcp47/timezone.xml", "common/supplemental/metaZones.xml",
        "icu-77-1-zoneinfo64.icu", "icu-country-source.json", "selector.json",
        "LICENSE", "ICU-LICENSE",
    }
    if set(sources) != expected:
        raise ValueError("time-zone source manifest and generator disagree")
    if sum(map(len, sources.values())) != manifest["total_bytes"]:
        raise ValueError("time-zone source manifest size mismatch")
    return manifest_bytes, sources


def icu_countries(source):
    # Only these parallel string arrays are consumed. TZif transitions come
    # independently from the newer IANA catalogue, never from this ICU file.
    def string_array(key):
        match = re.search(r"(?m)^\s*" + key + r"\s*\{([^{}]*)\}", source)
        if match is None:
            raise ValueError(f"missing ICU country array: {key}")
        body = re.sub(r"//[^\n]*", "", match[1])
        strings = re.findall(r'"([^"\\]*)"', body)
        if re.sub(r'"[^"\\]*"|[,\s]', "", body):
            raise ValueError(f"unsupported ICU country array syntax: {key}")
        return strings

    names, regions = string_array("Names"), string_array("Regions:array")
    if len(names) != len(regions) or len(set(names)) != len(names):
        raise ValueError("ICU country arrays have mismatched or duplicate names")
    if not all(re.fullmatch(r"[A-Z]{2}|001", region) for region in regions):
        raise ValueError("invalid ICU country code")
    return dict(zip(names, regions))


def canonical_aliases(document):
    entries = document.findall("./keyword/key[@name='tz']/type")
    types = {entry.attrib["name"]: entry for entry in entries}
    if len(types) != len(entries):
        raise ValueError("duplicate CLDR time-zone type")

    def preferred(name):
        seen = set()
        while types[name].get("preferred"):
            if name in seen:
                raise ValueError(f"cyclic time-zone preferred alias: {name}")
            seen.add(name)
            name = types[name].attrib["preferred"]
            if name not in types:
                raise ValueError(f"missing time-zone preferred alias target: {name}")
        aliases = types[name].get("alias", "").split()
        if not aliases:
            raise ValueError(f"CLDR time-zone type has no canonical identifier: {name}")
        return aliases[0]

    aliases = {}
    canonical = set()
    for name, entry in types.items():
        target = preferred(name)
        canonical.add(target)
        for alias in entry.get("alias", "").split():
            if alias in aliases and aliases[alias] != target:
                raise ValueError(f"conflicting CLDR time-zone alias: {alias}")
            aliases[alias] = target
    if any(aliases.get(target) != target for target in canonical):
        raise ValueError("nonterminal CLDR time-zone alias")
    return aliases, canonical


def inherited_names(documents):
    names, cities, countries, patterns = {}, {}, {}, {}
    for locale in ("root", "en", "en_US"):
        root = documents[f"common/main/{locale}.xml"]
        zone_names = root.find("./dates/timeZoneNames")
        if zone_names is not None:
            if zone_names.findall(".//alias"):
                raise ValueError(f"unresolved time-zone name alias: {locale}")
            for entry in zone_names:
                if entry.tag in ("zone", "metazone"):
                    key = entry.tag, entry.attrib["type"]
                    for width in WIDTHS:
                        width_node = entry.find(width)
                        if width_node is None:
                            continue
                        for name in width_node:
                            if name.tag not in KINDS:
                                raise ValueError(f"unknown time-zone name type: {name.tag}")
                            text = name.text
                            if text == "↑↑↑":
                                continue
                            if text in (None, "", "∅∅∅"):
                                raise ValueError(f"unresolved name: {locale}/{key}/{width}")
                            names[key, width, name.tag] = text
                    city = entry.find("exemplarCity")
                    if city is not None and city.text != "↑↑↑":
                        if not city.text or city.text == "∅∅∅":
                            raise ValueError(f"unresolved exemplar city: {locale}/{key}")
                        cities[entry.attrib["type"]] = city.text
                elif entry.tag in ("hourFormat", "gmtFormat", "gmtZeroFormat",
                                   "regionFormat", "fallbackFormat"):
                    if entry.text == "↑↑↑":
                        continue
                    if not entry.text or entry.text == "∅∅∅":
                        raise ValueError(f"unresolved time-zone pattern: {locale}/{entry.tag}")
                    patterns[entry.tag, entry.get("type", "generic")] = entry.text
        for entry in root.findall("./localeDisplayNames/territories/territory"):
            if entry.get("alt") not in (None, "short") or entry.text == "↑↑↑":
                continue
            if not entry.text or entry.text == "∅∅∅":
                raise ValueError(f"unresolved country name: {locale}/{entry.attrib}")
            countries[entry.attrib["type"], entry.get("alt", "long")] = entry.text
    required_patterns = {
        ("hourFormat", "generic"), ("gmtFormat", "generic"),
        ("gmtZeroFormat", "generic"), ("regionFormat", "generic"),
        ("regionFormat", "standard"), ("regionFormat", "daylight"),
        ("fallbackFormat", "generic"),
    }
    if set(patterns) != required_patterns:
        raise ValueError("time-zone pattern domain differs from the renderer")
    # The actual supported locale inventory is en/en-US. A changed pattern
    # requires updating the closed renderer rather than silently ignoring it.
    if patterns["hourFormat", "generic"] != "+HH:mm;-HH:mm":
        raise ValueError("unsupported localized offset hour pattern")
    for key, value in patterns.items():
        expected = ["{0}", "{1}"] if key[0] == "fallbackFormat" else ["{0}"] if key[0] in ("gmtFormat", "regionFormat") else []
        if sorted(re.findall(r"\{[^{}]*\}", value)) != expected:
            raise ValueError(f"invalid time-zone pattern placeholders: {key}")
    return names, cities, countries, patterns


def epoch_seconds(source):
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}", source):
        raise ValueError(f"invalid UTC metazone boundary: {source}")
    instant = datetime.strptime(source, "%Y-%m-%d %H:%M").replace(tzinfo=timezone.utc)
    delta = instant - EPOCH
    return delta.days * 86400 + delta.seconds


def extract(source_dir):
    manifest_bytes, sources = checked_sources(source_dir)
    documents = {name: ET.fromstring(contents) for name, contents in sources.items()
                 if name.endswith(".xml")}
    aliases, canonical = canonical_aliases(documents["common/bcp47/timezone.xml"])
    territories = icu_countries(sources["icu-77-1-zoneinfo64.icu"].decode("utf-8-sig"))
    names, cities, countries, patterns = inherited_names(documents)
    meta_root = documents["common/supplemental/metaZones.xml"]
    periods = {}
    for entry in meta_root.findall("./metaZones/metazoneInfo/timezone"):
        identifier = entry.attrib["type"]
        if identifier not in canonical or identifier in periods:
            raise ValueError(f"unknown or duplicate CLDR period zone: {identifier}")
        rows = []
        for period in entry:
            if period.tag != "usesMetazone" or set(period.attrib) - {"from", "to", "mzone"}:
                raise ValueError(f"unknown metazone period domain: {identifier}")
            start = epoch_seconds(period.attrib["from"]) if "from" in period.attrib else UNBOUNDED_START
            end = epoch_seconds(period.attrib["to"]) if "to" in period.attrib else UNBOUNDED_END
            if start >= end or (rows and start < rows[-1][1]):
                raise ValueError(f"empty, overlapping or unordered metazone periods: {identifier}")
            rows.append((start, end, period.attrib["mzone"]))
        periods[identifier] = rows
    preferred = {}
    for entry in meta_root.findall("./metaZones/mapTimezones/mapZone"):
        metazone, territory, identifier = (entry.attrib[key] for key in ("other", "territory", "type"))
        if identifier not in aliases:
            raise ValueError(f"unknown preferred time zone: {identifier}")
        key = metazone, territory
        if key in preferred:
            raise ValueError(f"duplicate preferred time zone: {key}")
        preferred[key] = aliases[identifier]
    primary = {}
    for entry in meta_root.findall("./primaryZones/primaryZone"):
        territory = entry.attrib["iso3166"]
        if territory in primary or entry.text not in aliases:
            raise ValueError(f"invalid primary time zone: {territory}")
        primary[territory] = aliases[entry.text]
    if canonical - territories.keys():
        raise ValueError(f"missing country authority: {sorted(canonical - territories.keys())}")
    country_counts = Counter(territories[zone] for zone in canonical if territories[zone] != "001")

    def display_country(territory, short):
        normal = countries.get((territory, "long"), territory)
        return countries.get((territory, "short"), normal) if short else normal

    def forms(kind, identifier):
        return [[names.get(((kind, identifier), width, name)) for name in KINDS] for width in WIDTHS]

    metazones = sorted({row[2] for rows in periods.values() for row in rows}
                       | {key[1] for key, _, _ in names if key[0] == "metazone"})
    for metazone in metazones:
        if (metazone, "001") not in preferred:
            raise ValueError(f"missing golden time zone: {metazone}")
    for (metazone, territory), identifier in preferred.items():
        if not any(period[2] == metazone for period in periods.get(identifier, [])):
            raise ValueError(f"preferred time zone has no reverse period: {metazone}/{territory}")
    zone_rows = []
    for identifier in sorted(canonical):
        territory = territories[identifier]
        is_location = territory != "001"
        city = cities.get(identifier, identifier.rsplit("/", 1)[-1].replace("_", " "))
        location = None
        if is_location:
            location = display_country(territory, short=True) if country_counts[territory] == 1 or primary.get(territory) == identifier else city
        zone_rows.append({"identifier": identifier, "territory": territory,
                          "city": city, "country": display_country(territory, short=False) if is_location else None,
                          "location": location, "names": forms("zone", identifier),
                          "periods": periods.get(identifier, [])})
    meta_rows = [{"identifier": metazone, "names": forms("metazone", metazone),
                  "preferred": sorted((territory, identifier) for (name, territory), identifier in preferred.items() if name == metazone)}
                 for metazone in metazones]
    rows = {"zones": zone_rows, "aliases": sorted(aliases.items()), "metazones": meta_rows,
            "patterns": [[*key, value] for key, value in sorted(patterns.items())]}
    return manifest_bytes, sources, rows


def generate(source_dir):
    manifest_bytes, sources, rows = extract(source_dir)
    canonical_rows = json.dumps(rows, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    rows_hash = hashlib.sha256(canonical_rows).digest()
    source_hash = hashlib.sha256(manifest_bytes).digest()
    generator_hash = hashlib.sha256(Path(__file__).read_bytes()).digest()
    composite = hashlib.sha256(b"lila-cldr47-time-zone-names-v1\0" + source_hash + rows_hash + generator_hash).digest()
    lines = ["// Generated by scripts/generate-intl-time-zone-names.py; do not edit.",
             f"// CLDR 47.0.0, commit {CLDR_COMMIT}.",
             "// Sources, selector, checksums and licenses: data/zone-names-cldr-47/.",
             "use super::{Metazone, MetazonePeriod, NameVariants, WidthNames, Zone};", ""]

    def optional(value):
        return "None" if value is None else f"Some({rust_string(value)})"

    def write_names(forms, indent):
        lines.append(f"{indent}names: WidthNames {{")
        for width, values in zip(WIDTHS, forms):
            lines.append(f"{indent}    {width}: NameVariants {{")
            for kind, value in zip(KINDS, values):
                lines.append(f"{indent}        {kind}: {optional(value)},")
            lines.append(f"{indent}    }},")
        lines.append(f"{indent}}},")

    for tag, kind, value in rows["patterns"]:
        if tag == "regionFormat" and kind != "generic":
            continue
        constant = re.sub(r"(?<!^)(?=[A-Z])", "_", tag).upper()
        if tag == "regionFormat":
            constant += "_" + kind.upper()
        lines.append(f"pub(super) const {constant}: &str = {rust_string(value)};")
    lines.extend(["", "pub(super) const METAZONES: &[Metazone] = &["])
    for metazone in rows["metazones"]:
        lines.append("    Metazone {")
        lines.append(f'        identifier: {rust_string(metazone["identifier"])},')
        write_names(metazone["names"], "        ")
        preferred = [f"({rust_string(territory)}, {rust_string(zone)})" for territory, zone in metazone["preferred"]]
        if len("[" + ", ".join(preferred) + "]") <= 60:
            lines.append("        preferred: &[" + ", ".join(preferred) + "],")
        else:
            lines.append("        preferred: &[")
            lines.extend("            " + row + "," for row in preferred)
            lines.append("        ],")
        lines.append("    },")
    lines.extend(["];", "", "pub(super) const ZONES: &[Zone] = &["])
    meta_indices = {row["identifier"]: index for index, row in enumerate(rows["metazones"])}
    for zone in rows["zones"]:
        lines.append("    Zone {")
        for key in ("identifier", "territory", "city"):
            lines.append(f"        {key}: {rust_string(zone[key])},")
        for key in ("country", "location"):
            lines.append(f"        {key}: {optional(zone[key])},")
        write_names(zone["names"], "        ")
        if len(zone["periods"]) == 1:
            start, end, metazone = zone["periods"][0]
            lines.extend(["        periods: &[MetazonePeriod {", f"            start: {start},",
                          f"            end: {end},", f"            metazone: {meta_indices[metazone]},", "        }],"])
        elif zone["periods"]:
            lines.append("        periods: &[")
            for start, end, metazone in zone["periods"]:
                lines.extend(["            MetazonePeriod {", f"                start: {start},",
                              f"                end: {end},", f"                metazone: {meta_indices[metazone]},", "            },"])
            lines.append("        ],")
        else:
            lines.append("        periods: &[],")
        lines.append("    },")
    lines.extend(["];", "", "pub(super) const ALIASES: &[(&str, usize)] = &["])
    zone_indices = {row["identifier"]: index for index, row in enumerate(rows["zones"])}
    for alias, canonical in rows["aliases"]:
        lines.append(f"    ({rust_string(alias)}, {zone_indices[canonical]}),")
    lines.extend(["];", "", "pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = ["])
    for start in range(0, 32, 16):
        lines.append("    " + ", ".join(f"0x{byte:02x}" for byte in composite[start:start+16]) + ",")
    lines.extend(["];", ""])
    generated = "\n".join(lines)
    report = {"cldr_release": "47.0.0", "cldr_commit": CLDR_COMMIT,
              "country_icu_commit": ICU_COMMIT, "source_manifest_sha256": source_hash.hex(),
              "generator_sha256": generator_hash.hex(), "normalized_rows_sha256": rows_hash.hex(),
              "provider_data_sha256": composite.hex(), "generated_sha256": hashlib.sha256(generated.encode()).hexdigest(),
              "generated_bytes": len(generated.encode()), "source_bytes": sum(map(len, sources.values())),
              "zones": len(rows["zones"]), "aliases": len(rows["aliases"]), "metazones": len(rows["metazones"]),
              "periods": sum(len(zone["periods"]) for zone in rows["zones"]),
              "supported_locales": ["en", "en-US"], "fallback_chain": ["root", "en", "en_US"],
              "consumed_icu_fields": ["Names", "Regions:array"], "epoch_boundaries": "half-open UTC seconds; omitted boundaries unbounded"}
    return generated, json.dumps(report, indent=2) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, default=REPOSITORY / SOURCE_PATH)
    parser.add_argument("--output", type=Path, default=REPOSITORY / OUTPUT_PATH)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args()
    generated, report = generate(options.source_dir)
    for path, contents in ((options.output, generated), (options.report, report)):
        if path is None:
            continue
        if options.check:
            if path.read_text() != contents:
                raise ValueError(f"generated output differs: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents)
    print(report, end="")


if __name__ == "__main__":
    main()
