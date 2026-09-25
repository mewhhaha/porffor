#!/usr/bin/env python3
"""Generate Intl.Locale information tables from hash-pinned CLDR47 and IANA2026a inputs.

The tables back ECMA-402 15.5.9-15.5.17 (CalendarsOfLocale, CollationsOfLocale,
HourCyclesOfLocale, TimeZonesOfLocale, TextDirectionOfLocale and
WeekInfoOfLocale) and the collation part of Intl.supportedValuesOf. Every input
is verified against crates/lila-intl/data/locale-info-cldr-47/manifest.json
before it is parsed; the script uses only the Python standard library.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

REPOSITORY = Path(__file__).resolve().parents[1]
DATA_DIR = REPOSITORY / "crates/lila-intl/data/locale-info-cldr-47"
OUTPUT = REPOSITORY / "crates/lila-intl/src/provider/locale_info/generated.rs"
REPORT = DATA_DIR / "generated-report.json"

REGION = re.compile(r"(?:[A-Z]{2}|[0-9]{3})")
LANGUAGE = re.compile(r"[a-z]{2,3}|[a-z]{5,8}")
SCRIPT = re.compile(r"[A-Z][a-z]{3}")
VALUE = re.compile(r"[a-z0-9]{3,8}(?:-[a-z0-9]{3,8})*")
WEEKDAYS = {"mon": "Monday", "tue": "Tuesday", "wed": "Wednesday", "thu": "Thursday",
            "fri": "Friday", "sat": "Saturday", "sun": "Sunday"}
ISO_WEEKDAY = {name: index + 1 for index, name in enumerate(WEEKDAYS)}
# UTS35 Part 4 "Time Data": skeleton hour symbols and their ECMA-402 hour cycles.
# The day-period forms b/B only select a different day-period field.
HOUR_SYMBOLS = {"h": "H12", "hb": "H12", "hB": "H12", "H": "H23", "Hb": "H23", "HB": "H23",
                "K": "H11", "k": "H24"}
HOUR_CYCLE_NAMES = {"H11": "h11", "H12": "h12", "H23": "h23", "H24": "h24"}
# ECMA-402 15.5.10 step 3.b: the CollationsOfLocale fallback for unmatched locales.
ROOT_COLLATIONS = ["emoji", "eor"]
# ECMA-402 10.2.3: these types never appear in a [[co]] list; private types are
# CLDR-internal tailoring building blocks.
EXCLUDED_COLLATIONS = {"standard", "search"}


def read_verified(path, entry):
    payload = path.read_bytes()
    if len(payload) != entry["bytes"] or hashlib.sha256(payload).hexdigest() != entry["sha256"]:
        raise ValueError(f"pinned input checksum mismatch: {path}")
    return payload


def load_inputs(data_dir):
    manifest_bytes = (data_dir / "manifest.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    if manifest["release"] != "47.0.0" or manifest["schema_version"] != 1:
        raise ValueError("review the pinned CLDR release before regenerating locale info")
    files = {}
    for entry in manifest["files"]:
        if "git_blob_sha1" in entry:
            payload = read_verified(data_dir / entry["path"], entry)
            blob = hashlib.sha1(b"blob %d\0" % len(payload) + payload).hexdigest()
            if blob != entry["git_blob_sha1"]:
                raise ValueError(f"upstream git blob mismatch: {entry['path']}")
            files[entry["path"]] = payload
    pinned_collations = {path for path in files if path.startswith("common/collation/")}
    present = {str(path.relative_to(data_dir)) for path in (data_dir / "common/collation").glob("*.xml")}
    if pinned_collations != present:
        raise ValueError("collation directory and pinned manifest disagree")
    shared = {}
    for entry in manifest["shared_inputs"]:
        shared[entry["path"]] = read_verified((data_dir / entry["path"]).resolve(), entry)
    return manifest_bytes, manifest, files, shared


def bcp47_types(document, key_name):
    """Return (registered canonical types, alias -> canonical) for one BCP47 key."""
    key = document.find(f".//key[@name='{key_name}']")
    if key is None:
        raise ValueError(f"missing BCP47 key {key_name}")
    registered = set()
    aliases = {}
    preferred = {}
    for entry in key.findall("type"):
        name = entry.attrib["name"]
        registered.add(name)
        for alias in entry.get("alias", "").split():
            aliases[alias] = name
        if entry.get("preferred"):
            preferred[name] = entry.attrib["preferred"]

    def canonical(value):
        value = aliases.get(value, value)
        seen = set()
        while value in preferred:
            if value in seen:
                raise ValueError(f"cyclic preferred BCP47 type {key_name}-{value}")
            seen.add(value)
            value = preferred[value]
        if value not in registered:
            return None
        return value

    return registered, canonical


def regions_attribute(element, attribute):
    return element.attrib[attribute].split()


def calendar_preferences(supplemental, canonical_calendar):
    rows = {}
    data = supplemental.find("calendarPreferenceData")
    for preference in data.findall("calendarPreference"):
        ordering = []
        for name in preference.attrib["ordering"].split():
            value = canonical_calendar(name)
            if value is None:
                raise ValueError(f"unregistered preferred calendar: {name}")
            if value not in ordering:
                ordering.append(value)
        for territory in regions_attribute(preference, "territories"):
            # CLDR47 keys calendar preferences by region only. A future
            # locale-keyed row must extend the runtime lookup, not vanish.
            if not REGION.fullmatch(territory):
                raise ValueError(f"calendar preference key is not a region: {territory}")
            if territory in rows:
                raise ValueError(f"duplicate calendar preference: {territory}")
            rows[territory] = ordering
    if "001" not in rows:
        raise ValueError("calendar preference data lacks the 001 default")
    return rows


def locale_key(identifier):
    """Convert a CLDR region or language_region key to its ECMA-402 lookup key."""
    if REGION.fullmatch(identifier):
        return identifier
    language, separator, region = identifier.partition("_")
    if not separator or not LANGUAGE.fullmatch(language) or not REGION.fullmatch(region):
        raise ValueError(f"unexpected time-data key: {identifier}")
    return f"{language}-{region}"


def hour_cycles(supplemental):
    rows = {}
    for hours in supplemental.find("timeData").findall("hours"):
        cycles = []
        for symbol in [hours.attrib["preferred"], *hours.attrib["allowed"].split()]:
            if symbol not in HOUR_SYMBOLS:
                raise ValueError(f"unknown time-data hour symbol: {symbol}")
            cycle = HOUR_SYMBOLS[symbol]
            if cycle not in cycles:
                cycles.append(cycle)
        for identifier in regions_attribute(hours, "regions"):
            key = locale_key(identifier)
            if key in rows:
                raise ValueError(f"duplicate time data: {identifier}")
            rows[key] = cycles
    if "001" not in rows:
        raise ValueError("time data lacks the 001 default")
    return rows


def week_data(supplemental):
    fields = {"firstDay": {}, "weekendStart": {}, "weekendEnd": {}}
    available = set()
    for element in supplemental.find("weekData"):
        if not isinstance(element.tag, str):
            continue
        if element.tag == "weekOfPreference":
            continue  # locale-keyed ordering preference, not week data for a region
        if element.tag not in {"minDays", *fields}:
            raise ValueError(f"unknown weekData element: {element.tag}")
        if element.get("alt"):
            continue  # alternative variants are not the region's week data
        territories = regions_attribute(element, "territories")
        for territory in territories:
            if not REGION.fullmatch(territory):
                raise ValueError(f"week data key is not a region: {territory}")
            available.add(territory)
        if element.tag == "minDays":
            continue
        day = element.attrib["day"]
        if day not in WEEKDAYS:
            raise ValueError(f"unknown week day: {day}")
        for territory in territories:
            if territory in fields[element.tag]:
                raise ValueError(f"duplicate {element.tag}: {territory}")
            fields[element.tag][territory] = day
    for name, values in fields.items():
        if "001" not in values:
            raise ValueError(f"{name} lacks the 001 default")

    def row(region):
        first = fields["firstDay"].get(region, fields["firstDay"]["001"])
        start = ISO_WEEKDAY[fields["weekendStart"].get(region, fields["weekendStart"]["001"])]
        end = ISO_WEEKDAY[fields["weekendEnd"].get(region, fields["weekendEnd"]["001"])]
        weekend = [start]
        while weekend[-1] != end:
            weekend.append(weekend[-1] % 7 + 1)
            if len(weekend) > 7:
                raise ValueError(f"unterminated weekend range for {region}")
        return first, sorted(weekend)

    return {region: row(region) for region in sorted(available | {"001"})}


def script_directions(metadata):
    rows = {}
    for line in metadata.decode("utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        fields = [field.strip() for field in line.split(";")]
        if len(fields) < 11 or not SCRIPT.fullmatch(fields[0]):
            raise ValueError(f"malformed script metadata row: {line}")
        if fields[0] in rows:
            raise ValueError(f"duplicate script metadata row: {fields[0]}")
        direction = {"YES": "RightToLeft", "NO": "LeftToRight", "UNKNOWN": None}[fields[6]]
        rows[fields[0]] = direction
    return {script: direction for script, direction in rows.items() if direction is not None}


def cldr_to_bcp47_locale(identifier):
    subtags = identifier.split("_")
    if not LANGUAGE.fullmatch(subtags[0]):
        raise ValueError(f"unexpected collation locale language: {identifier}")
    output = [subtags[0]]
    rest = subtags[1:]
    if rest and SCRIPT.fullmatch(rest[0]):
        output.append(rest.pop(0))
    if rest and REGION.fullmatch(rest[0]):
        output.append(rest.pop(0))
    if rest:
        raise ValueError(f"unexpected collation locale subtags: {identifier}")
    return "-".join(output)


def collations(files, supplemental, canonical_collation):
    parents = {}
    for parent_locales in supplemental.findall("parentLocales"):
        # UTS35 Part 1 "Parent Locales": component-specific parents stand alone
        # and are not merged with the main-component rules.
        if "collations" not in parent_locales.get("component", "").split():
            continue
        for parent_locale in parent_locales.findall("parentLocale"):
            for child in parent_locale.attrib["locales"].split():
                if child in parents:
                    raise ValueError(f"duplicate collation parent: {child}")
                parents[child] = parent_locale.attrib["parent"]

    own = {}
    excluded = []
    for path, payload in files.items():
        if not path.startswith("common/collation/"):
            continue
        identifier = path.removeprefix("common/collation/").removesuffix(".xml")
        document = ET.fromstring(payload)
        identity = document.find("identity")
        language = identity.find("language").attrib["type"]
        if identifier != "root" and not identifier.startswith(language):
            raise ValueError(f"collation identity mismatch: {identifier}")
        if document.findall(".//alias"):
            raise ValueError(f"collation alias requires inheritance support: {identifier}")
        types = set()
        for collation in document.iter("collation"):
            name = collation.attrib["type"]
            if name.startswith("private-") or name in EXCLUDED_COLLATIONS:
                continue
            value = canonical_collation(name)
            if value is None or not VALUE.fullmatch(value):
                excluded.append({"locale": identifier, "type": name,
                                 "reason": "not a registered BCP47 collation type"})
                continue
            types.add(value)
        own[identifier] = types

    def chain(identifier):
        seen = []
        current = identifier
        while True:
            if current in seen:
                raise ValueError(f"cyclic collation parent chain at {identifier}")
            seen.append(current)
            if current == "root":
                return seen
            if current in parents:
                current = parents[current]
            elif "_" in current:
                current = current.rsplit("_", 1)[0]
            else:
                current = "root"

    locales = {}
    posix = []
    for identifier in sorted(own):
        if identifier == "root":
            continue
        if identifier.endswith("_POSIX"):
            # UTS35 converts the legacy POSIX variant to -u-va-posix; an
            # Available Locales List element cannot carry a Unicode extension.
            posix.append(identifier)
            continue
        types = set()
        for ancestor in chain(identifier):
            types |= own.get(ancestor, set())
        locales[cldr_to_bcp47_locale(identifier)] = sorted(types)
    root_types = sorted(own["root"])
    if root_types != ROOT_COLLATIONS:
        raise ValueError(f"root collations {root_types} disagree with ECMA-402's fallback list")
    for locale in list(locales):
        subtags = locale.split("-")
        # ECMA-402 9.1: every narrower element needs its less narrow fallbacks.
        for length in range(1, len(subtags)):
            if "-".join(subtags[:length]) not in locales:
                raise ValueError(f"collation locale {locale} lacks its fallback prefix")
    if "en-US" not in locales:
        raise ValueError("collation locales must include the default locale en-US")
    return locales, excluded, posix


def time_zones(zone_tab, catalogue):
    primaries = set()
    for line in catalogue.decode("utf-8").splitlines():
        identifier, primary, _digest = line.split("\t")
        if identifier == primary:
            primaries.add(identifier)
    rows = {}
    for line in zone_tab.decode("utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        country, identifier = fields[0], fields[2]
        if not re.fullmatch(r"[A-Z]{2}", country):
            raise ValueError(f"unexpected zone.tab country: {country}")
        if identifier not in primaries:
            raise ValueError(f"zone.tab identifier is not an ECMA-402 primary: {identifier}")
        zones = rows.setdefault(country, [])
        if identifier in zones:
            raise ValueError(f"duplicate zone.tab row: {country} {identifier}")
        zones.append(identifier)
    return {country: sorted(zones) for country, zones in sorted(rows.items())}


def rust_string_list(values):
    return "&[" + ", ".join(f'"{value}"' for value in values) + "]"


def emit_table(lines, name, row_type, rows, render):
    lines.append(f"pub(super) const {name}: &[{row_type}] = &[")
    for row in rows:
        lines.append(f"    {render(row)},")
    lines.extend(["];", ""])


def generate(data_dir):
    manifest_bytes, manifest, files, shared = load_inputs(data_dir)
    supplemental = ET.fromstring(shared["../datetime-cldr-47/common/supplemental/supplementalData.xml"])
    _, canonical_calendar = bcp47_types(
        ET.fromstring(shared["../cldr-47-bcp47/common/bcp47/calendar.xml"]), "ca")
    _, canonical_collation = bcp47_types(
        ET.fromstring(shared["../cldr-47-bcp47/common/bcp47/collation.xml"]), "co")

    calendars = calendar_preferences(supplemental, canonical_calendar)
    cycles = hour_cycles(supplemental)
    weeks = week_data(supplemental)
    directions = script_directions(files["common/properties/scriptMetadata.txt"])
    collation_locales, excluded_collations, posix_locales = collations(
        files, supplemental, canonical_collation)
    zones = time_zones(shared["../iana-tzdb-2026a/source/zone.tab"],
                       shared["../iana-tzdb-2026a/catalogue.tsv"])

    collation_sets = sorted({tuple(types) for types in collation_locales.values()})
    set_index = {types: index for index, types in enumerate(collation_sets)}
    available_collations = sorted({value for types in collation_sets for value in types})

    tables = {
        "calendar_preferences": {key: calendars[key] for key in sorted(calendars)},
        "hour_cycles": {key: [HOUR_CYCLE_NAMES[cycle] for cycle in cycles[key]]
                        for key in sorted(cycles)},
        "week_data": {key: {"first_day": weeks[key][0], "weekend": weeks[key][1]}
                      for key in sorted(weeks)},
        "script_directions": {key: directions[key] for key in sorted(directions)},
        "collation_locales": {key: collation_locales[key] for key in sorted(collation_locales)},
        "available_collations": available_collations,
        "region_time_zones": zones,
    }
    rows_bytes = json.dumps(tables, sort_keys=True, separators=(",", ":")).encode()
    rows_hash = hashlib.sha256(rows_bytes).digest()
    manifest_hash = hashlib.sha256(manifest_bytes).digest()
    composite = hashlib.sha256(b"lila-intl-locale-info-v1\0" + manifest_hash + rows_hash).digest()

    lines = [
        "// Generated by scripts/generate-intl-locale-info.py; do not edit.",
        f"// CLDR {manifest['release']}, commit {manifest['commit']}; IANA tzdb 2026a zone.tab.",
        "// Unicode license and all source checksums: data/locale-info-cldr-47/.",
        "use super::{IsoWeekday, TextDirection, WeekRow};",
        "use crate::DateTimeHourCycle;",
        "",
        "/// calendarPreferenceData, canonicalized through the BCP47 `ca` aliases.",
    ]
    emit_table(lines, "CALENDAR_PREFERENCES", "(&str, &[&str])", sorted(calendars.items()),
               lambda row: f'("{row[0]}", {rust_string_list(row[1])})')
    lines.append("/// timeData keyed by region or by language-region, preferred symbol first.")
    emit_table(lines, "HOUR_CYCLES", "(&str, &[DateTimeHourCycle])", sorted(cycles.items()),
               lambda row: f'("{row[0]}", &[' + ", ".join(
                   f"DateTimeHourCycle::{cycle}" for cycle in row[1]) + "])")
    lines.append("/// Every region with CLDR week data, each field resolved against 001.")
    emit_table(lines, "WEEK_DATA", "(&str, WeekRow)", sorted(weeks.items()),
               lambda row: f'("{row[0]}", WeekRow {{ first_day: IsoWeekday::{WEEKDAYS[row[1][0]]}, '
               f'weekend: &[' + ", ".join(
                   f"IsoWeekday::{list(WEEKDAYS.values())[day - 1]}" for day in row[1][1]) + "] })")
    lines.append("/// scriptMetadata.txt RTL column; UNKNOWN scripts are absent.")
    emit_table(lines, "SCRIPT_DIRECTIONS", "(&str, TextDirection)", sorted(directions.items()),
               lambda row: f'("{row[0]}", TextDirection::{row[1]})')
    lines.append("/// Interned sorted [[co]] lists (without the null default) of the Collator locales.")
    emit_table(lines, "COLLATION_SETS", "&[&str]", collation_sets, rust_string_list)
    lines.append("/// %Intl.Collator%.[[AvailableLocales]] with their COLLATION_SETS index.")
    emit_table(lines, "COLLATOR_LOCALES", "(&str, usize)", sorted(collation_locales.items()),
               lambda row: f'("{row[0]}", {set_index[tuple(row[1])]})')
    lines.append("/// AvailableCanonicalCollations: the union of every Collator [[co]] list.")
    emit_table(lines, "AVAILABLE_COLLATIONS", "&str", available_collations,
               lambda value: f'"{value}"')
    lines.append("/// IANA zone.tab primary identifiers grouped by country, sorted.")
    emit_table(lines, "REGION_TIME_ZONES", "(&str, &[&str])", zones.items(),
               lambda row: f'("{row[0]}", {rust_string_list(row[1])})')
    lines.extend(["// Composite identity covers the source manifest and every generated row.",
                  "pub(super) const PROVIDER_DATA_SHA256: [u8; 32] = ["])
    for offset in range(0, 32, 16):
        lines.append("    " + ", ".join(f"0x{byte:02x}" for byte in composite[offset:offset + 16]) + ",")
    lines.extend(["];", ""])

    report = {
        "cldr_release": manifest["release"],
        "cldr_commit": manifest["commit"],
        "source_manifest_sha256": manifest_hash.hex(),
        "rows_sha256": rows_hash.hex(),
        "provider_data_sha256": composite.hex(),
        "calendar_preference_regions": len(calendars),
        "hour_cycle_keys": len(cycles),
        "hour_cycle_language_region_keys": sum("-" in key for key in cycles),
        "week_data_regions": len(weeks),
        "script_directions": len(directions),
        "collation_files": sum(path.startswith("common/collation/") for path in files),
        "collator_locales": len(collation_locales),
        "collation_sets": len(collation_sets),
        "available_collations": available_collations,
        "excluded_collation_types": excluded_collations,
        "excluded_posix_locales": posix_locales,
        "time_zone_regions": len(zones),
        "time_zones": sum(len(value) for value in zones.values()),
    }
    return "\n".join(lines), json.dumps(report, indent=2) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data-dir", type=Path, default=DATA_DIR)
    parser.add_argument("--output", type=Path, default=OUTPUT)
    parser.add_argument("--report", type=Path, default=REPORT)
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args()
    generated, report = generate(options.data_dir)
    for path, contents in [(options.output, generated), (options.report, report)]:
        if options.check:
            if not path.exists() or path.read_text() != contents:
                raise SystemExit(f"generated output differs: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents)
    parsed = json.loads(report)
    print(f"CLDR {parsed['cldr_release']}: {parsed['collator_locales']} Collator locales, "
          f"{parsed['calendar_preference_regions']} calendar regions, "
          f"{parsed['hour_cycle_keys']} hour-cycle keys, {parsed['week_data_regions']} week regions, "
          f"{parsed['time_zones']} zones; provider SHA256 {parsed['provider_data_sha256']}")


if __name__ == "__main__":
    main()
