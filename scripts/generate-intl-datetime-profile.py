#!/usr/bin/env python3
"""Generate a checked private CLDR date/time profile; no capability publication."""

import argparse
import hashlib
import json
from pathlib import Path
import re

from intl_cldr_profile import CldrProfile, PatternAlternate, parse_path, path_text
from intl_datetime_patterns import compile_interval, compile_pattern, skeleton_in_profile
from intl_datetime_names import field_names
from intl_datetime_zones import canonical_zone_geography, localized_zone_names
from intl_rbnf_fields import algorithmic_field_tables


REPOSITORY = Path(__file__).resolve().parents[1]
SOURCE_PATH = "crates/lila-intl/data/datetime-cldr-47"
STYLES = ("full", "long", "medium", "short")
NAME_BRANCHES = ("months", "monthPatterns", "days", "dayPeriods", "eras",
                 "cyclicNameSets/cyclicNameSet[@type='years']")


def calendar_preferences(profile, locale, territory):
    aliases = {}
    document = profile.documents["common/bcp47/calendar.xml"]
    for entry in document.findall("./keyword/key[@name='ca']/type"):
        canonical = entry.get("preferred", entry.attrib["name"])
        for spelling in (entry.attrib["name"], *entry.get("alias", "").split()):
            if spelling in aliases and aliases[spelling] != canonical:
                raise ValueError(f"ambiguous calendar preference alias: {spelling}")
            aliases[spelling] = canonical
    result = []
    for identifier in preference(profile, "calendarPreference", "ordering", locale, territory).split():
        if identifier not in aliases:
            raise ValueError(f"unknown LDML calendar preference: {identifier}")
        result.append(aliases[identifier])
    return result


def selected(path):
    return all(part.get("alt") is None for part in path)


def plain_leaf(leaf):
    if leaf.attributes:
        raise ValueError(f"unconsumed LDML value attributes: {path_text(leaf.source_path)}")
    return leaf.value


def pattern_numbering(leaf):
    attributes = dict(leaf.attributes)
    if set(attributes) - {"numbers"}:
        raise ValueError(f"unsupported pattern value attributes: {attributes}")
    overrides = []
    if "numbers" in attributes:
        for assignment in attributes["numbers"].split(";"):
            parts = assignment.split("=")
            if len(parts) == 1:
                field, numbering = None, parts[0]
            elif len(parts) == 2 and len(parts[0]) == 1 and parts[0] in "yrMLdHhKkmsS":
                field, numbering = parts
            else:
                raise ValueError(f"unsupported pattern numbering assignment: {assignment}")
            if not re.fullmatch(r"[a-z0-9]{3,8}", numbering):
                raise ValueError(f"invalid pattern numbering identifier: {numbering}")
            if any(previous["field"] == field for previous in overrides):
                raise ValueError(f"duplicate pattern numbering field: {field}")
            overrides.append({"field": field, "numbering": numbering})
    return overrides


def pattern_leaf(leaf, *, placeholders=()):
    return {"source": leaf.value, "tokens": compile_pattern(leaf.value, placeholders=placeholders),
            "numbering_overrides": pattern_numbering(leaf)}


def required_pattern(profile, locale, path):
    return pattern_leaf(profile.resolve(locale, path))


def calendar_profile(profile, locale, calendar):
    base = f"dates/calendars/calendar[@type='{calendar}']"
    names = []
    for branch in NAME_BRANCHES:
        for path, leaf in profile.leaves(locale, base + "/" + branch):
            if selected(path):
                names.append([path_text(path[len(parse_path(base)):]), plain_leaf(leaf)])
    styles = {}
    for style in STYLES:
        styles[style] = {
            kind: required_pattern(profile, locale, f"{base}/{kind}Formats/{kind}FormatLength[@type='{style}']/{kind}Format/pattern")
            for kind in ("date", "time")
        }
        for kind, element in [("standard", "dateTimeFormat"), ("atTime", "dateTimeFormat[@type='atTime']")]:
            leaf = profile.resolve(locale, f"{base}/dateTimeFormats/dateTimeFormatLength[@type='{style}']/{element}/pattern")
            styles[style][kind] = pattern_leaf(leaf, placeholders=(0, 1))
    available = []
    excluded = []
    for path, leaf in profile.leaves(locale, base + "/dateTimeFormats/availableFormats"):
        if not selected(path):
            continue
        skeleton = path[-1].get("id")
        if path[-1].tag != "dateFormatItem" or skeleton is None:
            raise ValueError(f"unexpected available format path: {path_text(path)}")
        if not skeleton_in_profile(skeleton):
            excluded.append({"skeleton": skeleton, "path": path_text(path), "reason": "requested field absent from ECMA-402 DateTimeFormat options"})
            continue
        if path[-1].get("count") is not None:
            raise ValueError(f"admitted format requires unimplemented plural matching: {path_text(path)}")
        available.append({"skeleton": skeleton, **pattern_leaf(leaf)})
    intervals = []
    fallback = None
    for path, leaf in profile.leaves(locale, base + "/dateTimeFormats/intervalFormats"):
        if not selected(path):
            continue
        if path[-1].tag == "intervalFormatFallback":
            fallback = pattern_leaf(leaf, placeholders=(0, 1))
            continue
        if path[-1].tag != "greatestDifference" or path[-2].tag != "intervalFormatItem":
            raise ValueError(f"unexpected interval format path: {path_text(path)}")
        skeleton = path[-2].get("id")
        if not skeleton_in_profile(skeleton):
            excluded.append({"skeleton": skeleton, "path": path_text(path), "reason": "interval requested field absent from ECMA-402 DateTimeFormat options"})
            continue
        difference = path[-1].get("id")
        if difference not in "GyMdahHmsB":
            raise ValueError(f"unsupported greatest-difference field: {difference}")
        intervals.append({"skeleton": skeleton, "greatest_difference": difference,
                          "source": leaf.value, "numbering_overrides": pattern_numbering(leaf),
                          **compile_interval(leaf.value)})
    if fallback is None or not available or not intervals:
        raise ValueError(f"incomplete calendar pattern closure: {locale}/{calendar}")
    append_zone = pattern_leaf(profile.resolve(locale, base + "/dateTimeFormats/appendItems/appendItem[@request='Timezone']"), placeholders=(0, 1))
    names = field_names(sorted(names))
    append_era = None
    if any(name["kind"] == "era" for name in names):
        append_era = pattern_leaf(profile.resolve(locale, base + "/dateTimeFormats/appendItems/appendItem[@request='Era']"), placeholders=(0, 1))
    return {"calendar": calendar, "names": names, "styles": styles,
            "available": available, "intervals": intervals, "interval_fallback": fallback,
            "append_zone": append_zone, "append_era": append_era,
            "excluded_non_ecma_formats": excluded}


def positional_numbering_systems(profile):
    result = []
    document = profile.documents["common/supplemental/numberingSystems.xml"]
    for node in document.findall("./numberingSystems/numberingSystem"):
        identifier = node.attrib["id"]
        kind = node.attrib["type"]
        if kind == "algorithmic":
            if not node.get("rules") or node.get("digits"):
                raise ValueError(f"invalid algorithmic numbering source: {identifier}")
            continue
        digits = node.attrib["digits"]
        if kind != "numeric" or node.get("rules") or len(digits) != 10 or len(set(digits)) != 10:
            raise ValueError(f"invalid positional numbering system: {identifier}")
        if not re.fullmatch(r"[a-z0-9]{3,8}", identifier):
            raise ValueError(f"invalid numbering identifier: {identifier}")
        result.append({"identifier": identifier, "digits": digits})
    result.sort(key=lambda row: row["identifier"])
    if len({row["identifier"] for row in result}) != len(result):
        raise ValueError("duplicate numbering system")
    return result


def locale_territory(profile, locale):
    pieces = locale.split("_")
    explicit = next((part for part in pieces[1:] if re.fullmatch(r"[A-Z]{2}|[0-9]{3}", part)), None)
    if explicit is not None:
        return explicit
    likely = {node.attrib["from"]: node.attrib["to"] for node in profile.documents["common/supplemental/likelySubtags.xml"].findall("./likelySubtags/likelySubtag")}
    for candidate in (locale, pieces[0]):
        if candidate in likely:
            return likely[candidate].split("_")[-1]
    raise ValueError(f"missing likely territory for selected locale: {locale}")


def preference(profile, tag, attribute, locale, territory):
    root = profile.documents["common/supplemental/supplementalData.xml"]
    choices = {}
    if tag == "calendarPreference":
        nodes = root.findall("./calendarPreferenceData/calendarPreference")
        key = "territories"
    else:
        nodes = root.findall("./timeData/hours")
        key = "regions"
    for node in nodes:
        for region in node.attrib[key].split():
            if region in choices:
                raise ValueError(f"duplicate {tag} preference: {region}")
            choices[region] = node.attrib[attribute]
    for key in (locale, territory, "001"):
        if key in choices:
            return choices[key]
    raise ValueError(f"missing {tag} default: {locale}")


def day_period_rules(profile, locale):
    root = profile.documents["common/supplemental/dayPeriods.xml"]
    choices = {}
    for rule_set in root.findall("./dayPeriodRuleSet"):
        if rule_set.get("type") is not None:
            continue
        for group in rule_set.findall("dayPeriodRules"):
            rules = [dict(node.attrib) for node in group.findall("dayPeriodRule")]
            for key in group.attrib["locales"].split():
                if key in choices:
                    raise ValueError(f"duplicate day-period locale: {key}")
                choices[key] = rules
    for ancestor in profile.lineage(locale):
        if ancestor in choices:
            return choices[ancestor]
    raise ValueError(f"missing day period rules: {locale}")


def generate(source_directory):
    profile = CldrProfile(source_directory, pattern_alternate=PatternAlternate.ASCII)
    if profile.selector["alt_selection"] != "ascii date/time patterns when supplied; default names; short territory for generic location names":
        raise ValueError("unreviewed date/time alternate selection policy")
    numberings = positional_numbering_systems(profile)
    zone_manifest, zone_geography = canonical_zone_geography(REPOSITORY)
    locales = []
    for language_tag in profile.selector["locales"]:
        locale = language_tag.replace("-", "_")
        territory = locale_territory(profile, locale)
        default_numbering = profile.text(locale, "numbers/defaultNumberingSystem")
        if default_numbering not in {row["identifier"] for row in numberings}:
            raise ValueError(f"default is not an admitted positional system: {locale}/{default_numbering}")
        separators = []
        minus_signs = []
        for numbering in numberings:
            identifier = numbering["identifier"]
            path = f"numbers/symbols[@numberSystem='{identifier}']/decimal"
            separators.append([identifier, profile.text(locale, path)])
            minus_signs.append([identifier, profile.text(locale, path.rsplit('/', 1)[0] + '/minusSign')])
        calendars = {}
        for identifier in profile.selector["calendars"]:
            source = profile.selector["calendar_identifiers"][identifier]
            if source not in calendars:
                calendars[source] = calendar_profile(profile, locale, source)
        names = []
        for path, leaf in profile.leaves(locale, "dates/timeZoneNames"):
            if selected(path):
                names.append([path_text(path[2:]), plain_leaf(leaf)])
        countries = []
        for path, leaf in profile.leaves(locale, "localeDisplayNames/territories"):
            if path[-1].tag != "territory" or path[-1].get("alt") not in (None, "short"):
                continue
            countries.append([path[-1].get("type"), path[-1].get("alt", "long"), plain_leaf(leaf)])
        locales.append({
            "locale": language_tag, "territory": territory,
            "parent_chain": list(profile.lineage(locale)), "default_content": locale in profile.default_content,
            "default_numbering": default_numbering, "decimal_separators": separators,
            "minus_signs": minus_signs,
            "calendar_preferences": calendar_preferences(profile, locale, territory),
            "preferred_hour": preference(profile, "hours", "preferred", locale, territory),
            "allowed_hours": preference(profile, "hours", "allowed", locale, territory).split(),
            "day_period_rules": day_period_rules(profile, locale),
            "calendars": calendars,
            "zone_names": localized_zone_names(names, countries, zone_geography),
        })
    rows = {"schema_version": 1, "selector": profile.selector, "numbering_systems": numberings,
            "algorithmic_fields": algorithmic_field_tables(profile, locales),
            "zone_geography": zone_geography, "locales": locales}
    encoded = json.dumps(rows, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    report = {
        "source_manifest_sha256": hashlib.sha256(profile.manifest_bytes).hexdigest(),
        "zone_geography_manifest_sha256": hashlib.sha256(zone_manifest).hexdigest(),
        "profile_sha256": hashlib.sha256(encoded.encode()).hexdigest(),
        "numbering_systems": len(numberings), "locales": len(locales),
        "consumed_leaf_count": len(profile.consumed), "consumed_leaves": profile.consumed,
        "publication": "private foundation only; no product capability registered",
    }
    return encoded, json.dumps(report, ensure_ascii=False, sort_keys=True, indent=2) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, default=REPOSITORY / SOURCE_PATH)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args()
    outputs = generate(options.source_dir)
    files = [(options.output, outputs[0])]
    if options.report is not None:
        files.append((options.report, outputs[1]))
    for path, contents in files:
        if options.check:
            if path.read_text() != contents:
                raise ValueError(f"generated profile is stale: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents)
    print(json.dumps({"profile_bytes": len(outputs[0].encode()), "report_bytes": len(outputs[1].encode())}))


if __name__ == "__main__":
    main()
