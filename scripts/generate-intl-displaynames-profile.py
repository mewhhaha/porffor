#!/usr/bin/env python3
"""Materialize six DisplayNames domains from the checked, pinned CLDR47 LDML.

The catalogue is deliberately the thirteen source locales already captured for
DateTimeFormat. No machine locale database or guessed English names participate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

from intl_cldr_profile import CldrProfile, CLDR_COMMIT, path_text, segment

REPOSITORY = Path(__file__).resolve().parents[1]
SOURCE = Path("crates/lila-intl/data/datetime-cldr-47")
BCP47 = Path("crates/lila-intl/data/cldr-47-bcp47")
OUTPUT = Path("crates/lila-intl/src/display_names/profile.json")
REPORT = Path("crates/lila-intl/src/display_names/provenance.json")
IDENTITY = Path("crates/lila-intl/src/display_names/profile_identity.rs")
LOCALES = sorted(["en", "en-US", "ar", "ar-EG", "zh", "zh-Hans", "zh-Hans-CN",
                  "de", "fr", "it", "ja", "ko", "hi"])
STYLES = ("long", "short", "narrow")
FIELDS = {"era": "era", "year": "year", "quarter": "quarter", "month": "month",
          "weekOfYear": "week", "weekday": "weekday", "day": "day",
          "dayPeriod": "dayperiod", "hour": "hour", "minute": "minute",
          "second": "second", "timeZoneName": "zone"}
LANGUAGE = re.compile(r"(?:[A-Za-z]{2,3}|[A-Za-z]{5,8})(?:_[A-Za-z]{4})?"
                      r"(?:_(?:[A-Za-z]{2}|[0-9]{3}))?"
                      r"(?:_(?:[A-Za-z0-9]{5,8}|[0-9][A-Za-z0-9]{3}))*")
VARIANT = re.compile(r"(?:[A-Za-z0-9]{5,8}|[0-9][A-Za-z0-9]{3})")
TYPE = re.compile(r"[a-z0-9]{3,8}(?:-[a-z0-9]{3,8})*")


def serialized(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def pool_names(locales):
    records = {}
    for locale in locales:
        for names in locale["styles"].values():
            for kind, entries in names.items():
                row = {"kind": kind, "entries": entries}
                records[canonical(row)] = row
    keys = sorted(records)
    indices = {key: index for index, key in enumerate(keys)}
    pooled_locales = []
    for locale in locales:
        styles = {style: {kind: indices[canonical({"kind": kind, "entries": entries})]
                          for kind, entries in names.items()}
                  for style, names in locale["styles"].items()}
        pooled_locales.append({**locale, "styles": styles})
    return [records[key] for key in keys], pooled_locales


def expand_names(profile):
    rows = profile["name_pool"]
    keys = [canonical(row) for row in rows]
    if keys != sorted(set(keys)):
        raise ValueError("noncanonical or duplicate name pools")
    used = set()
    locales = []
    for locale in profile["locales"]:
        styles = {}
        if set(locale["styles"]) != set(STYLES):
            raise ValueError("incomplete DisplayNames styles")
        for style, refs in locale["styles"].items():
            if set(refs) != {"language", "region", "script", "language_script", "currency",
                             "calendar", "variant", "date_time_field"}:
                raise ValueError("incomplete DisplayNames name-map domain")
            names = {}
            for kind, index in refs.items():
                if type(index) is not int or not 0 <= index < len(rows) or rows[index]["kind"] != kind:
                    raise ValueError("invalid DisplayNames pool reference/domain")
                names[kind] = rows[index]["entries"]
                used.add(index)
            styles[style] = names
        locales.append({**locale, "styles": styles})
    if used != set(range(len(rows))):
        raise ValueError("unused DisplayNames name pool")
    return locales


def checked_bcp47(repository):
    directory = repository / BCP47
    contents = (directory / "manifest.json").read_bytes()
    manifest = json.loads(contents)
    if manifest["release"] != "47.0.0" or manifest["commit"] != CLDR_COMMIT:
        raise ValueError("BCP47 calendar authority changed")
    documents = {}
    for row in manifest["files"]:
        relative = row["path"]
        if Path(relative).is_absolute() or ".." in Path(relative).parts or relative in documents:
            raise ValueError("invalid BCP47 source path")
        data = (directory / relative).read_bytes()
        if len(data) != row["bytes"] or hashlib.sha256(data).hexdigest() != row["sha256"]:
            raise ValueError(f"BCP47 source checksum mismatch: {relative}")
        documents[relative] = data
    key = ET.fromstring(documents["common/bcp47/calendar.xml"]).find("./keyword/key[@name='ca']")
    rows = {entry.attrib["name"]: entry for entry in key.findall("type")}
    mapping = {}
    for name, entry in rows.items():
        seen = set()
        canonical = name
        while rows[canonical].get("preferred"):
            if canonical in seen:
                raise ValueError("cyclic calendar preferred mapping")
            seen.add(canonical)
            canonical = rows[canonical].attrib["preferred"]
        # CanonicalCodeForDisplayNames only case-regularizes calendar codes.
        # Both canonical and well-formed deprecated identifiers therefore have
        # their own genuine field, sharing the registered terminal source name.
        source = next((alias for alias in rows[canonical].get("alias", "").split()
                       if alias != canonical), canonical)
        mapping[name] = source
        for alias in rows[canonical].get("alias", "").split():
            if TYPE.fullmatch(alias):
                mapping[alias] = source
    return hashlib.sha256(contents).hexdigest(), mapping


def names(profile, locale, root, tag, style, valid, spelling, *, stand_alone=False):
    prefix = tuple(segment(part) for part in root.split("/"))
    keys = {child.get("type") for child in profile.children(locale, prefix)
            if child.tag == tag and valid(child.get("type", ""))}
    result = []
    for key in sorted(keys):
        # Narrow uses a supplied narrow form, then short, then the full form;
        # it never truncates, transliterates, or manufactures a name.
        alternates = [None] if style == "long" else (["short", None] if style == "short"
                                                      else ["narrow", "short", None])
        if stand_alone:
            alternates.insert(len(alternates) - 1, "stand-alone")
        value = None
        for alternate in alternates:
            attrs = {"type": key}
            if alternate is not None:
                attrs["alt"] = alternate
            value = profile.text(locale, (*prefix, segment(tag, **attrs)), required=False)
            if value is not None:
                break
        if value is not None:
            result.append([spelling(key), value])
    result.sort()
    if len({row[0] for row in result}) != len(result):
        raise ValueError("source name spelling collision")
    return result


def language_spelling(value):
    fields = value.split("_")
    result = [fields[0].lower()]
    for field in fields[1:]:
        result.append(field.title() if len(field) == 4 and field.isalpha()
                      else field.upper() if len(field) == 2 and field.isalpha()
                      else field.lower())
    return "-".join(result)


def generate(repository):
    profile = CldrProfile(repository / SOURCE)
    if sorted(profile.selector["locales"]) != LOCALES:
        raise ValueError("review DisplayNames source catalogue before changing it")
    bcp_sha, calendars = checked_bcp47(repository)
    locales = []
    choices = {}
    for tag in LOCALES:
        locale = tag.replace("-", "_")
        patterns = {key: profile.text(locale, f"localeDisplayNames/localeDisplayPattern/{key}")
                    for key in ("localePattern", "localeSeparator")}
        for value in patterns.values():
            remainder = value.replace("{0}", "").replace("{1}", "")
            if value.count("{0}") != 1 or value.count("{1}") != 1 or "{" in remainder or "}" in remainder:
                raise ValueError("unknown locale composition pattern")
        styles = {}
        for style in STYLES:
            row = {}
            row["language"] = names(profile, locale, "localeDisplayNames/languages", "language",
                                    style, lambda code: LANGUAGE.fullmatch(code), language_spelling)
            row["region"] = names(profile, locale, "localeDisplayNames/territories", "territory",
                                  style, lambda code: re.fullmatch(r"[A-Z]{2}|[0-9]{3}", code), str)
            row["script"] = names(profile, locale, "localeDisplayNames/scripts", "script",
                                  style, lambda code: re.fullmatch(r"[A-Z][a-z]{3}", code), str,
                                  stand_alone=True)
            row["language_script"] = names(profile, locale, "localeDisplayNames/scripts", "script",
                                           style, lambda code: re.fullmatch(r"[A-Z][a-z]{3}", code), str)
            row["variant"] = names(profile, locale, "localeDisplayNames/variants", "variant",
                                   style, lambda code: VARIANT.fullmatch(code), str.lower)
            row["currency"] = []
            for child in profile.children(locale, "numbers/currencies"):
                code = child.get("type", "")
                if child.tag != "currency" or not re.fullmatch(r"[A-Z]{3}", code):
                    continue
                path = f"numbers/currencies/currency[@type='{code}']/displayName"
                value = profile.text(locale, path, required=False)
                if value is not None:
                    # A DisplayNames currency name is not a currency symbol or
                    # a count-dependent plural name. CLDR supplies one full name.
                    row["currency"].append([code, value])
            row["currency"].sort()
            row["calendar"] = []
            for code, source in sorted(calendars.items()):
                path = f"localeDisplayNames/types/type[@key='calendar'][@type='{source}']"
                value = profile.text(locale, path, required=False)
                if value is not None:
                    row["calendar"].append([code, value])
            row["date_time_field"] = []
            for code, field in sorted(FIELDS.items()):
                source = field + ("" if style == "long" else "-" + style)
                path = f"dates/fields/field[@type='{source}']/displayName"
                value = profile.text(locale, path)
                row["date_time_field"].append([code, value])
            if any(not values for values in row.values()):
                raise ValueError("incomplete DisplayNames domain")
            styles[style] = row
            choices[f"{tag}/{style}"] = {key: len(values) for key, values in row.items()}
        locales.append({"locale": tag, "locale_pattern": patterns["localePattern"],
                        "locale_separator": patterns["localeSeparator"], "styles": styles})
    source_sha = hashlib.sha256(profile.manifest_bytes).hexdigest()
    name_pool, pooled_locales = pool_names(locales)
    data = {"schema_version": 1, "cldr_release": "47.0.0", "cldr_commit": CLDR_COMMIT,
            "source_manifest_sha256": source_sha, "bcp47_manifest_sha256": bcp_sha,
            "default_locale": "en-US", "name_pool": name_pool, "locales": pooled_locales}
    if expand_names(data) != locales:
        raise ValueError("DisplayNames pooling changed genuine fields")
    report = {"schema_version": 1, "cldr_release": "47.0.0", "cldr_commit": CLDR_COMMIT,
              "source_manifest_sha256": source_sha, "bcp47_manifest_sha256": bcp_sha,
              "locale_catalogue": LOCALES, "styles": list(STYLES), "field_counts": choices,
              "name_pool_count": len(name_pool), "name_map_associations": len(LOCALES) * 3 * 8,
              "calendar_source_keys": calendars, "consumed_leaves": profile.consumed,
              "selection": {"names": "narrow -> short -> default; short -> default; long -> default",
                            "script": "stand-alone before default only for independent script names",
                            "currency": "count-less displayName, never symbol",
                            "language_aliases": "existing native CanonicalizeLocale operation on admission and requests; deprecated source translations excluded from canonical field ownership"}}
    data_bytes, report_bytes = serialized(data), serialized(report)
    digest = hashlib.sha256(data_bytes).digest()
    identity = ("// Generated by scripts/generate-intl-displaynames-profile.py.\n"
                + "// This is the payload identity; the final provider recipe must also bind native/producer owners.\n"
                + "pub const DISPLAY_NAMES_DATA_SHA256: [u8; 32] = [\n    "
                + ", ".join(f"0x{byte:02x}" for byte in digest[:16]) + ",\n    "
                + ", ".join(f"0x{byte:02x}" for byte in digest[16:]) + ",\n];\n"
                + f'pub(super) const SOURCE_MANIFEST_SHA256: &str =\n    "{source_sha}";\n'
                + f'pub(super) const BCP47_MANIFEST_SHA256: &str =\n    "{bcp_sha}";\n').encode()
    return data_bytes, report_bytes, identity


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    data, report, identity = generate(REPOSITORY)
    for path, contents in ((OUTPUT, data), (REPORT, report), (IDENTITY, identity)):
        path = REPOSITORY / path
        if args.check:
            if path.read_bytes() != contents:
                raise ValueError(f"non-reproducible DisplayNames artifact: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(contents)
    print(f"DisplayNames: {len(LOCALES)} locales, six types, three styles; {len(data)} profile bytes")


if __name__ == "__main__":
    main()
