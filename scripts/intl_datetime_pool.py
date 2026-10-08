"""Canonical, lossless schema-2 pools for the admitted DateTimeFormat recipe.

Calendar records keep their physical CLDR domain. Public calendar references
remain separate, including gregory and iso8601 sharing one Gregorian record.
The existing four/seven and proposed genuine sixteen/thirteen recipes are
explicit closed source selections; this helper publishes no product capability.
"""

from copy import deepcopy
import json


CALENDAR_IDENTIFIERS = {
    "gregory": "gregorian", "iso8601": "gregorian",
    "chinese": "chinese", "buddhist": "buddhist",
}
CALENDARS = tuple(CALENDAR_IDENTIFIERS)
EXPANDED_CALENDAR_IDENTIFIERS = {
    **CALENDAR_IDENTIFIERS,
    "coptic": "coptic", "dangi": "dangi", "ethioaa": "ethiopic-amete-alem",
    "ethiopic": "ethiopic", "hebrew": "hebrew", "indian": "indian",
    "islamic-civil": "islamic-civil", "islamic-tbla": "islamic-tbla",
    "islamic-umalqura": "islamic-umalqura", "japanese": "japanese",
    "persian": "persian", "roc": "roc",
}
EXPANDED_ERA_SUPPLEMENT = {
    "identifier": "selected-calendar-eras",
    "cldr_release": "48.0.0",
    "cldr_commit": "acd6d88ae493633240e19a87a721076a8a75c310",
    "source_manifest_sha256": "d551300d46e7a64558d60bb2732b702902aa68ae60ee62284d0a67b314c25932",
}
EXPANDED_LOCALES = ["en", "en-US", "ar", "ar-EG", "zh", "zh-Hans", "zh-Hans-CN", "de", "fr", "it", "ja", "ko", "hi"]

PROFILE_FIELDS = {
    "schema_version", "selector", "numbering_systems", "numbering_supplement",
    "algorithmic_fields", "zone_geography", "locales",
}
LOCALE_FIELDS = {
    "locale", "territory", "parent_chain", "default_content", "default_numbering",
    "decimal_separators", "minus_signs", "calendar_preferences", "preferred_hour",
    "allowed_hours", "day_period_rules", "calendars", "zone_names",
}


def canonical(record):
    return json.dumps(record, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def _recipe(profile, version):
    selector = profile["selector"]
    identifiers = EXPANDED_CALENDAR_IDENTIFIERS if len(selector["calendars"]) == 16 else CALENDAR_IDENTIFIERS
    extra = {"era_supplement"} if identifiers is EXPANDED_CALENDAR_IDENTIFIERS else set()
    expected = PROFILE_FIELDS | extra if version == 1 else PROFILE_FIELDS | extra | {"calendar_pool", "zone_name_pool"}
    if set(profile) != expected or profile["schema_version"] != version:
        raise ValueError("unexpected date/time pool schema")
    selector = profile["selector"]
    if (selector["schema_version"] != 1
            or selector["calendars"] != list(identifiers)
            or selector["calendar_identifiers"] != identifiers):
        raise ValueError("unreviewed date/time pool calendar recipe")
    if extra and (selector["locales"] != EXPANDED_LOCALES or profile["era_supplement"] != EXPANDED_ERA_SUPPLEMENT):
        raise ValueError("unreviewed expanded date/time source recipe")
    if extra and any(row.get("first_weekday") not in ("sun", "mon", "tue", "wed", "thu", "fri", "sat") for row in profile["locales"]):
        raise ValueError("unreviewed first-weekday domain")
    if [row["locale"] for row in profile["locales"]] != selector["locales"]:
        raise ValueError("locale records differ from the exact source selection")
    return identifiers


def _locale_fields(identifiers):
    return LOCALE_FIELDS | ({"first_weekday"} if identifiers is EXPANDED_CALENDAR_IDENTIFIERS else set())


def _intern(records):
    encoded = sorted({canonical(record) for record in records})
    return [json.loads(record) for record in encoded], {record: index for index, record in enumerate(encoded)}


def pool_profile(materialized):
    """Intern whole resolved records, preserving all leaves and source provenance."""
    identifiers = _recipe(materialized, 1)
    rows = deepcopy(materialized)
    for locale in rows["locales"]:
        if set(locale) != _locale_fields(identifiers) or set(locale["calendars"]) != set(identifiers.values()):
            raise ValueError("unexpected materialized locale calendar inventory")
        if any(record["calendar"] != physical for physical, record in locale["calendars"].items()):
            raise ValueError("calendar record has the wrong physical domain")
    calendars, calendar_index = _intern(
        record for locale in rows["locales"] for record in locale["calendars"].values())
    zones, zone_index = _intern(locale["zone_names"] for locale in rows["locales"])
    for locale in rows["locales"]:
        materialized_calendars = locale.pop("calendars")
        locale["calendar_refs"] = [
            [public, calendar_index[canonical(materialized_calendars[physical])]]
            for public, physical in identifiers.items()
        ]
        locale["zone_name_ref"] = zone_index[canonical(locale.pop("zone_names"))]
    rows.update(schema_version=2, calendar_pool=calendars, zone_name_pool=zones)
    # The inverse validates closed references, domains, coverage and canonical pools.
    if expand_profile(rows) != materialized:
        raise ValueError("date/time pooling changed materialized data")
    return rows


def _checked_pool(profile, name):
    pool = profile[name]
    if not isinstance(pool, list) or not pool:
        raise ValueError("empty or invalid date/time pool")
    encoded = [canonical(record) for record in pool]
    if encoded != sorted(set(encoded)):
        raise ValueError("noncanonical or duplicate date/time pool records")
    return pool


def _index(value, pool):
    if type(value) is not int or not 0 <= value < len(pool) or value > 0xffffffff:
        raise ValueError("invalid date/time pool reference")
    return value


def expand_profile(pooled):
    """Validate schema-2 ownership, then reconstruct the exact schema-1 shape."""
    identifiers = _recipe(pooled, 2)
    rows = deepcopy(pooled)
    calendars = _checked_pool(rows, "calendar_pool")
    zones = _checked_pool(rows, "zone_name_pool")
    used_calendars, used_zones = set(), set()
    expected = _locale_fields(identifiers) - {"calendars", "zone_names"} | {"calendar_refs", "zone_name_ref"}
    for locale in rows["locales"]:
        if set(locale) != expected:
            raise ValueError("unexpected pooled locale fields")
        refs = locale.pop("calendar_refs")
        if (not isinstance(refs, list) or len(refs) != len(identifiers)
                or any(not isinstance(row, list) or len(row) != 2 for row in refs)
                or [row[0] for row in refs] != list(identifiers)):
            raise ValueError("duplicate, incomplete or unordered canonical calendar references")
        refs = {public: _index(index, calendars) for public, index in refs}
        if refs["gregory"] != refs["iso8601"]:
            raise ValueError("Gregorian/ISO references differ")
        materialized = {}
        for public, physical in identifiers.items():
            index = refs[public]
            if calendars[index]["calendar"] != physical:
                raise ValueError("calendar reference has the wrong physical domain")
            used_calendars.add(index)
            materialized[physical] = deepcopy(calendars[index])
        zone_index = _index(locale.pop("zone_name_ref"), zones)
        used_zones.add(zone_index)
        locale.update(calendars=materialized, zone_names=deepcopy(zones[zone_index]))
    if used_calendars != set(range(len(calendars))) or used_zones != set(range(len(zones))):
        raise ValueError("unused date/time pool records")
    rows.pop("calendar_pool")
    rows.pop("zone_name_pool")
    rows["schema_version"] = 1
    return rows
