#!/usr/bin/env python3
"""Generate effective RelativeTimeFormat fields from pinned CLDR 47."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from intl_cldr_profile import CldrProfile, parse_path, path_text


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/lila-intl/data/datetime-cldr-47"
OUTPUT = ROOT / "crates/lila-intl/src/provider/relative_time/patterns.json"
UNITS = ("second", "minute", "hour", "day", "week", "month", "quarter", "year")
STYLES = ("long", "short", "narrow")
CATEGORIES = ("zero", "one", "two", "few", "many", "other")
RELATIVE_VALUES = ("-2", "-1", "0", "1", "2")
# The DateTime selector stays intentionally small; relative-time support also
# includes Polish language/region data because both identifiers are in the
# checked Intl.RelativeTimeFormat service inventory.
RELATIVE_TIME_LOCALES = ("pl", "pl-PL")


def effective_entry(profile: CldrProfile, locale: str, unit: str, style: str) -> dict[str, object] | None:
    locale_source = locale.replace("-", "_")
    field = f"dates/fields/field[@type='{unit}{'' if style == 'long' else '-' + style}']"
    patterns: dict[str, dict[str, str]] = {"past": {}, "future": {}}
    relative: dict[str, str] = {}

    for direction in ("future", "past"):
        for category in CATEGORIES:
            path = (f"{field}/relativeTime[@type='{direction}']/"
                    f"relativeTimePattern[@count='{category}']")
            leaf = profile.resolve(locale_source, path, required=False)
            if leaf is not None:
                if leaf.attributes:
                    raise ValueError(f"unconsumed value attributes: {locale}/{path_text(leaf.source_path)}")
                patterns[direction][category] = leaf.value

    for offset in RELATIVE_VALUES:
        path = f"{field}/relative[@type='{offset}']"
        leaf = profile.resolve(locale_source, path, required=False)
        if leaf is not None:
            if leaf.attributes:
                raise ValueError(f"unconsumed value attributes: {locale}/{path_text(leaf.source_path)}")
            relative[offset] = leaf.value

    if not any(patterns.values()) and not relative:
        return None
    if not patterns["past"] or not patterns["future"]:
        raise ValueError(f"incomplete past/future relative-time patterns: {locale}/{unit}/{style}")
    return {"patterns": patterns, "relative": relative}


def generate() -> str:
    profile = CldrProfile(SOURCE)
    locales: dict[str, dict[str, object]] = {}
    selected_locales = dict.fromkeys(
        (*profile.selector["locales"], *RELATIVE_TIME_LOCALES)
    )
    for locale in selected_locales:
        locale_rows: dict[str, object] = {}
        locale_source = locale.replace("-", "_")
        language = locale.split("-", 1)[0]
        for unit in UNITS:
            long = effective_entry(profile, locale, unit, "long")
            for style in STYLES:
                entry = long if style == "long" else effective_entry(profile, locale, unit, style)
                if entry is None:
                    if long is None:
                        raise ValueError(f"missing long relative-time pattern: {locale}/{unit}")
                    # RelativeTimeFormat falls back by style through the CLDR
                    # field aliases. The resolved leaf is reused as data.
                    entry = long
                locale_rows[f"{unit}/{style}"] = entry

            # A short/narrow row may alias the long field, but a localized
            # long row must never silently inherit any pattern or relative
            # term from root.
            if language not in ("en", "root") and long is not None:
                for direction in ("future", "past"):
                    for category in CATEGORIES:
                        long_path = (
                            f"dates/fields/field[@type='{unit}']/relativeTime[@type='{direction}']/"
                            f"relativeTimePattern[@count='{category}']"
                        )
                        long_leaf = profile.resolve(locale_source, long_path, required=False)
                        for style in ("short", "narrow"):
                            path = (
                                f"dates/fields/field[@type='{unit}-{style}']/relativeTime[@type='{direction}']/"
                                f"relativeTimePattern[@count='{category}']"
                            )
                            leaf = profile.resolve(locale_source, path, required=False)
                            if (
                                leaf is not None
                                and long_leaf is not None
                                and leaf.source_locale == "root"
                                and long_leaf.source_locale != "root"
                            ):
                                raise ValueError(
                                    f"{locale}/{unit}/{style}/{direction}/{category} "
                                    "discarded localized CLDR data and fell through to root"
                                )
                for offset in RELATIVE_VALUES:
                    long_path = f"dates/fields/field[@type='{unit}']/relative[@type='{offset}']"
                    long_leaf = profile.resolve(locale_source, long_path, required=False)
                    for style in ("short", "narrow"):
                        path = f"dates/fields/field[@type='{unit}-{style}']/relative[@type='{offset}']"
                        leaf = profile.resolve(locale_source, path, required=False)
                        if (
                            leaf is not None
                            and long_leaf is not None
                            and leaf.source_locale == "root"
                            and long_leaf.source_locale != "root"
                        ):
                            raise ValueError(
                                f"{locale}/{unit}/{style}/relative/{offset} "
                                "discarded localized CLDR data and fell through to root"
                            )

        locales[locale] = locale_rows

    if any("↑↑↑" in json.dumps(locale, ensure_ascii=False) for locale in locales.values()):
        raise ValueError("unresolved CLDR inheritance marker escaped into relative-time data")
    return json.dumps(
        {
            "source": "CLDR 47 LDML dates/fields with checked aliases and inheritance",
            "locales": locales,
        },
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    output = generate()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != output:
            raise SystemExit(f"out-of-date relative-time profile: {OUTPUT}")
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(output)


if __name__ == "__main__":
    main()
