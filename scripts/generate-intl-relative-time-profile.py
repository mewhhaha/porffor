#!/usr/bin/env python3
"""Export genuine CLDR47 relative-time fields; no capability publication."""

import argparse
import hashlib
import json
from pathlib import Path

from intl_cldr_profile import (
    CLDR_COMMIT, CldrProfile, INHERIT, NO_INHERIT, ResolvedLeaf, Segment,
    normalize_path, parse_path, path_text,
)

from intl_relative_time_sources import CAPTURE_PATH, load_polish_sources, load_profile

REPOSITORY = Path(__file__).resolve().parents[1]
SOURCE_PATH = "crates/lila-intl/data/datetime-cldr-47"
PROFILE_PATH = "crates/lila-intl/src/relative_time_format/generated/profile.json"
REPORT_PATH = "crates/lila-intl/data/relative-time-cldr-47/report.json"
INPUTS_PATH = "crates/lila-intl/data/relative-time-cldr-47/source-inputs.json"
BASE_LOCALES = ("en", "en-US", "ar", "ar-EG", "zh", "zh-Hans", "zh-Hans-CN",
           "de", "fr", "it", "ja", "ko", "hi")
LOCALES = (*BASE_LOCALES, "pl")
UNITS = ("year", "quarter", "month", "week", "day", "hour", "minute", "second")
STYLES = ("long", "short", "narrow")
CATEGORIES = ("zero", "one", "two", "few", "many", "other")
OFFSETS = (-2, -1, 0, 1, 2)


def serialized(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode()


def checked_text(value, *, numeric):
    if not isinstance(value, str) or not value or value in (INHERIT, NO_INHERIT):
        raise ValueError("relative-time leaf must be a nonempty resolved string")
    placeholders = value.count("{0}")
    remainder = value.replace("{0}", "")
    if "{" in remainder or "}" in remainder or placeholders > int(numeric):
        raise ValueError("relative-time pattern needs zero or one {0}; auto literals need none")
    return value


def resolve_count(profile, locale, path, *, seen=()):
    """The count lateral fallback is local to relativeTimePattern only.

    Captured CLDR47 tr35.md requires exact count, then other in each locale,
    before the original count is tried in the parent. Root style aliases
    restart at the requested locale. This does not alter shared LDML lookup.
    """
    path = normalize_path(parse_path(path) if isinstance(path, str) else path, profile.schema)
    if path[-1].tag != "relativeTimePattern" or path[-1].get("count") not in CATEGORIES:
        raise ValueError("count resolver accepts only the six relative-time categories")
    identity = locale, path
    if identity in seen:
        raise ValueError(f"cyclic relative-time alias: {locale}/{path_text(path)}")
    other = (*path[:-1], Segment(path[-1].tag, tuple(sorted(
        (key, "other" if key == "count" else value) for key, value in path[-1].attributes))))
    candidates = (path,) if path == other else (path, other)
    for ancestor in profile.lineage(locale):
        tree = profile.locales[ancestor]
        for candidate in candidates:
            value = tree.leaves.get(candidate)
            if value is not None and value.text == NO_INHERIT:
                return None
            if value is not None and value.text != INHERIT:
                if not value.text:
                    raise ValueError(f"empty relative-time leaf: {ancestor}/{path_text(candidate)}")
                return ResolvedLeaf(value.text, value.attributes, ancestor, candidate)
        redirected = tree.redirect(path)
        if redirected is not None:
            return resolve_count(profile, locale, redirected, seen=(*seen, identity))
    return None


def consume(consumed, locale, path, leaf, *, numeric):
    if leaf is None:
        raise ValueError(f"unresolved required relative-time pattern: {locale}/{path_text(path)}")
    if leaf.attributes:
        raise ValueError(f"unconsumed relative-time value attributes: {leaf.attributes}")
    value = checked_text(leaf.value, numeric=numeric)
    key = f"{locale}/{path_text(path)}"
    record = {"source_locale": leaf.source_locale, "source_path": path_text(leaf.source_path),
              "value": value, "value_attributes": {},
              "requested_category": path[-1].get("count"),
              "source_category": leaf.source_path[-1].get("count")}
    if key in consumed and consumed[key] != record:
        raise ValueError(f"inconsistent relative-time provenance: {key}")
    consumed[key] = record
    return value


def generate(profile):
    if profile.selector["locales"] != list(BASE_LOCALES):
        raise ValueError("relative-time recipe requires the exact thirteen captured locales")
    if profile.relative_source_capture["locale_admission"] != ["pl"] or "pl" not in profile.locales:
        raise ValueError("missing authenticated Polish locale source")
    locales, consumed = [], {}
    for identifier in LOCALES:
        locale = identifier.replace("-", "_")
        fields = []
        for unit in UNITS:
            for style in STYLES:
                source_unit = unit if style == "long" else f"{unit}-{style}"
                base = f"dates/fields/field[@type='{source_unit}']"
                row = {"unit": unit, "style": style}
                for direction in ("past", "future"):
                    patterns = []
                    for category in CATEGORIES:
                        path = parse_path(f"{base}/relativeTime[@type='{direction}']/relativeTimePattern[@count='{category}']")
                        leaf = resolve_count(profile, locale, path)
                        patterns.append(consume(consumed, locale, path, leaf, numeric=True))
                    row[direction] = patterns
                relative = []
                for offset in OFFSETS:
                    path = parse_path(f"{base}/relative[@type='{offset}']")
                    leaf = profile.resolve(locale, path, required=False)
                    if leaf is not None:
                        relative.append({"offset": offset,
                                         "value": consume(consumed, locale, path, leaf, numeric=False)})
                for child in profile.children(locale, base):
                    if child.tag == "relative" and child.get("alt") is None and child.get("type") not in {str(x) for x in OFFSETS}:
                        raise ValueError(f"unreviewed relative-time offset: {locale}/{child}")
                row["relative"] = relative
                fields.append(row)
        locales.append({"locale": identifier, "fields": fields})
    result = {"schema": 1, "cldr_commit": CLDR_COMMIT, "locales": locales}
    report = {
        "schema": 1, "cldr_release": "47.0.0", "cldr_commit": CLDR_COMMIT,
        "locale_order": list(LOCALES), "unit_order": list(UNITS), "style_order": list(STYLES),
        "category_order": list(CATEGORIES), "locale_count": len(locales),
        "unit_style_rows": sum(len(row["fields"]) for row in locales),
        "numeric_patterns": len(locales) * len(UNITS) * len(STYLES) * 2 * len(CATEGORIES),
        "numeric_patterns_without_placeholder": sum("{0}" not in pattern for row in locales
            for field in row["fields"] for direction in ("past", "future") for pattern in field[direction]),
        "relative_literals": sum(len(field["relative"]) for row in locales for field in row["fields"]),
        "count_fallback_policy": "exact category then other within each locale before parent; requested-locale style aliases retained",
        "required_DTD_count_attribute": True,
        "text_normalization": "none",
        "profile_sha256": hashlib.sha256(serialized(result)).hexdigest(),
        "consumed_leaves": dict(sorted(consumed.items())),
        "native_or_product_admission": False,
    }
    return result, report


def source_inputs(repository, profile):
    names = {"scripts/generate-intl-relative-time-profile.py", "scripts/intl_cldr_profile.py",
             "scripts/intl_ldml_schema.py", "scripts/intl_positional_numbering.py", "scripts/intl_calendar_eras.py"}
    names.add(f"{SOURCE_PATH}/manifest.json")
    capture, _ = load_polish_sources(repository)
    names.update(["scripts/intl_relative_time_sources.py", CAPTURE_PATH, capture["source_manifest"]["path"], capture["archive"]["path"]])
    names.update("crates/lila-intl/data/relative-time-cldr-47/" + row["path"] for row in capture["files"])
    names.update(f"{SOURCE_PATH}/{name}" for name in profile.sources)
    for directory, manifest_name in [("numbering-tols-cldr-48", "source-manifest.json"),
                                      ("calendar-eras-cldr-48", "manifest.json")]:
        base = f"crates/lila-intl/data/{directory}"
        manifest_path = repository / base / manifest_name
        manifest = json.loads(manifest_path.read_bytes())
        names.add(f"{base}/{manifest_name}")
        names.update(f"{base}/{row['path']}" for row in manifest["files"])
    files = []
    for name in sorted(names):
        raw = (repository / name).read_bytes()
        files.append({"path": name, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
    return {"schema": 1, "cldr_commit": CLDR_COMMIT, "files": files,
            "total_bytes": sum(row["bytes"] for row in files),
            "supplements": "verified shared CldrProfile dependencies; relative-time leaves are unmodified CLDR47 fields",
            "public_capability_admission": False}


def write_or_check(path, value, check):
    raw = serialized(value)
    if check:
        if not path.is_file() or path.read_bytes() != raw:
            raise ValueError(f"generated relative-time artifact differs: {path}")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path(PROFILE_PATH))
    parser.add_argument("--report", type=Path, default=Path(REPORT_PATH))
    parser.add_argument("--source-inputs", type=Path, default=Path(INPUTS_PATH))
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    profile = load_profile(REPOSITORY)
    result, report = generate(profile)
    inputs = source_inputs(REPOSITORY, profile)
    for path, value in [(args.output, result), (args.report, report), (args.source_inputs, inputs)]:
        write_or_check(REPOSITORY / path, value, args.check)
    print(json.dumps({"mode": "check" if args.check else "generate", "locales": len(LOCALES),
                      "unit_style_rows": report["unit_style_rows"], "numeric_patterns": report["numeric_patterns"],
                      "relative_literals": report["relative_literals"], "profile_sha256": report["profile_sha256"]}, sort_keys=True))


if __name__ == "__main__":
    main()
