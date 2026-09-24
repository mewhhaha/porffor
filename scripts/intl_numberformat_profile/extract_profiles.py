#!/usr/bin/env python3
"""Generate complete canonical NumberFormat profiles from pinned CLDR47 XML."""
import argparse
from functools import lru_cache
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

from cldr_resolver import CldrResolver, path_text
from pattern_grammar import number_pattern, message_pattern, strip_number, UNIT, MedialPlaceholder
from unicode_sets import properties as unicode_properties, parse as parse_unicode_set
from plural_grammar import CATEGORIES, parse_rule, check_samples, category, sample_operands

STAGE = Path(__file__).resolve().parents[1]
NUMERIC = STAGE / "work/crates/lila-intl/src/number_format"


def stable(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":"))


class Pool:
    def __init__(self):
        self.rows = []
        self.index = {}

    def add(self, value):
        key = stable(value)
        if key not in self.index:
            self.index[key] = len(self.rows)
            self.rows.append(value)
        return self.index[key]


class Extractor:
    def __init__(self, stage):
        self.resolver = CldrResolver(stage)
        self.pools = {name: Pool() for name in [
            "strings", "patterns", "signed_patterns", "choices", "symbols", "compact_sets",
            "numbering_profiles", "currency_sets", "unit_sets", "plural_rules", "plural_ranges", "profiles", "unicode_sets",
        ]}
        self.medial_denominators = []
        self.string("")
        self.pattern([])
        self.unicode_properties = unicode_properties(Path(stage) / "reference/ucd")
        self.property_sets = {name: self.add("unicode_sets", ranges) for name, ranges in self.unicode_properties.items()}
        r = self.resolver
        systems = r.xml("common/supplemental/numberingSystems.xml")
        self.systems = []
        for node in systems.findall(".//numberingSystem"):
            if node.get("type") == "algorithmic":
                continue
            if node.get("type") != "numeric" or node.get("radix", "10") != "10":
                raise ValueError(f"unknown numbering system definition: {node.attrib}")
            digits = node.get("digits")
            if len(digits) != 10 or len(set(digits)) != 10:
                raise ValueError(f"invalid digit alphabet: {node.attrib}")
            self.systems.append({"name": node.get("id"), "digits": digits})
        self.systems.sort(key=lambda row: row["name"])
        if len(self.systems) != 77:
            raise ValueError("review changed numeric digit inventory")
        self.system_indexes = {row["name"]: i for i, row in enumerate(self.systems)}
        option_source = (NUMERIC / "options.rs").read_text()
        units = re.search(r"closed_option!\(SingleUnit\s*\{(.+?)\}\);", option_source, re.DOTALL)
        if units is None:
            raise ValueError("read the approved sanctioned-unit domain")
        self.units = [spelling for _, spelling in re.findall(r'(\w+)\s*=>\s*"([^"]+)"', units[1])]
        if len(self.units) != 45 or len(set(self.units)) != 45:
            raise ValueError("review changed sanctioned-unit inventory")
        self.unit_types = set()
        self.currency_codes = set()
        self.currency_override_codes = set()
        # Inventory every primary source, not merely translated top-level files.
        for locale in sorted(r.locales):
            root = r.xml(f"common/main/{locale}.xml")
            self.unit_types.update(n.get("type") for n in root.findall("./units/unitLength/unit"))
            for currency in root.findall("./numbers/currencies/currency"):
                code = currency.get("type")
                if not re.fullmatch(r"[A-Z]{3}", code):
                    raise ValueError(f"invalid currency key: {code}")
                self.currency_codes.add(code)
                if any(currency.find(name) is not None for name in ("pattern", "decimal", "group")):
                    self.currency_override_codes.add(code)
        self.unit_map = {}
        for unit in self.units:
            matches = [key for key in self.unit_types if key.partition("-")[2] == unit]
            if len(matches) != 1:
                raise ValueError(f"ambiguous sanctioned unit mapping: {unit} {matches}")
            self.unit_map[unit] = matches[0]
        self.unit_pairs = []
        for i, numerator in enumerate(self.units):
            for j, denominator in enumerate(self.units):
                joined = f"{numerator}-per-{denominator}"
                matches = [key for key in self.unit_types if key.partition("-")[2] == joined]
                if len(matches) > 1:
                    raise ValueError(f"ambiguous compound unit mapping: {joined}")
                if matches:
                    self.unit_pairs.append((i, j, matches[0]))
        self.plural_by_locale = {}
        self.plural_samples = []
        for group in r.xml("common/supplemental/plurals.xml").findall(".//pluralRules"):
            source_rules = [(n.get("count"), n.text) for n in group.findall("pluralRule")]
            rules = [(name, parse_rule(text)) for name, text in source_rules]
            if any(name not in CATEGORIES for name, _ in rules):
                raise ValueError("unknown cardinal category")
            samples = check_samples(rules, source_rules)
            index = self.add("plural_rules", rules)
            for locale in group.get("locales").split():
                if locale in self.plural_by_locale:
                    raise ValueError(f"duplicate plural locale: {locale}")
                self.plural_by_locale[locale] = index
            self.plural_samples.append({"rule": index, "locales": group.get("locales").split(), "samples": samples})
        self.root_plural = self.add("plural_rules", [])
        self.range_by_locale = {}
        self.root_range = self.add("plural_ranges", [end for _ in CATEGORIES for end in CATEGORIES])
        for group in r.xml("common/supplemental/pluralRanges.xml").findall(".//pluralRanges"):
            table = [end for _ in CATEGORIES for end in CATEGORIES]
            seen = set()
            for node in group.findall("pluralRange"):
                start, end, result = (node.get(name) for name in ("start", "end", "result"))
                if any(value not in CATEGORIES for value in (start, end, result)) or (start, end) in seen:
                    raise ValueError(f"unknown/duplicate plural-range case: {node.attrib}")
                seen.add((start, end))
                table[CATEGORIES.index(start) * 6 + CATEGORIES.index(end)] = result
            index = self.add("plural_ranges", table)
            for locale in group.get("locales").split():
                if locale in self.range_by_locale:
                    raise ValueError(f"duplicate range locale: {locale}")
                self.range_by_locale[locale] = index
        self.derivations = {}
        for group in r.xml("common/supplemental/grammaticalFeatures.xml").findall(".//grammaticalDerivations"):
            rows = {node.get("feature"): [node.get("value0"), node.get("value1")]
                    for node in group.findall("deriveComponent") if node.get("structure") == "per"}
            for locale in group.get("locales").split():
                self.derivations[locale] = rows
        self.fractions = []
        self.default_fraction = None
        for node in r.xml("common/supplemental/supplementalData.xml").findall("./currencyData/fractions/info"):
            code = node.get("iso4217")
            digits = int(node.get("digits", "2"))
            if not 0 <= digits <= 100:
                raise ValueError(f"currency digits outside approved domain: {node.attrib}")
            if code == "DEFAULT":
                if self.default_fraction is not None:
                    raise ValueError("duplicate currency default")
                self.default_fraction = digits
            else:
                self.fractions.append([code, digits])
        if self.default_fraction is None:
            raise ValueError("missing currency default")
        self.fractions.sort()

    def add(self, table, value):
        return self.pools[table].add(value)

    def string(self, value):
        return self.add("strings", value)

    def pattern(self, tokens):
        return self.add("patterns", [(kind, self.string(text)) for kind, text in tokens])

    def signed(self, source, *, compact=False):
        parsed = number_pattern(source, compact=compact)
        return self.add("signed_patterns", {
            "positive": self.pattern(parsed["positive"]), "negative": self.pattern(parsed["negative"]),
            "skeleton": parsed["skeleton"],
        })

    @staticmethod
    def component_locale(locale, table, default):
        # Plurals/grammaticalFeatures explicitly do not use main's nonlikely-
        # script parent map. Component fallback truncates the locale identifier.
        while True:
            if locale in table:
                return table[locale]
            if "_" not in locale:
                return default
            locale = locale.rsplit("_", 1)[0]

    def rules(self, locale):
        index = self.component_locale(locale, self.plural_by_locale, self.root_plural)
        return index, self.pools["plural_rules"].rows[index]

    def choice(self, locale, path, transform, *, fallback=None, kind="pattern"):
        r = self.resolver
        values = []
        for plural in CATEGORIES:
            text = r.text(locale, path.format(count=plural), required=fallback is None)
            values.append(transform(fallback if text is None else text))
        # Only genuine explicit numeric entries override category selection.
        # A missing explicit entry stays absent; lateral fallback was resolved
        # above into the category rows and cannot accidentally become exact-one.
        for explicit in ("0", "1"):
            text = r.text(locale, path.format(count=explicit), required=False, lateral=False)
            values.append(None if text is None else transform(text))
        return self.add("choices", {"kind": kind, "values": values})

    @lru_cache(maxsize=None)
    def compact(self, locale, system, family, *, alpha=False):
        r = self.resolver
        if family == "currency":
            prefix = f"numbers/currencyFormats[@numberSystem='{system}']/currencyFormatLength[@type='short']/currencyFormat[@type='standard']"
        else:
            prefix = f"numbers/decimalFormats[@numberSystem='{system}']/decimalFormatLength[@type='{family}']/decimalFormat"
        magnitudes = set()
        for child in r.children(locale, prefix):
            value = child.get("type")
            if child.tag != "pattern" or value is None:
                continue
            if not re.fullmatch(r"10*", value):
                raise ValueError(f"non-decimal compact magnitude: {locale}/{prefix}/{child}")
            magnitudes.add(len(value) - 1)
        if not magnitudes:
            if system != "latn":
                return self.compact(locale, "latn", family, alpha=alpha)
            if family == "long":
                return self.compact(locale, system, "short", alpha=alpha)
            raise ValueError(f"missing required compact set after documented fallback: {locale} {family}")
        rows = []
        for magnitude in range(max(magnitudes) + 1):
            scale = "1" + "0" * magnitude
            path = prefix + f"/pattern[@type='{scale}'][@count='{{count}}']"
            if alpha:
                path += "[@alt='alphaNextToNumber']"
            other = r.text(locale, path.format(count="other"), required=False)
            if other is None:
                other = "0"
            parsed = number_pattern(other, compact=True)
            skeleton = parsed["skeleton"]
            fallback = other == "0"
            if not fallback and (skeleton is None or skeleton["minimum_integer"] < 1):
                raise ValueError(f"compact other form cannot establish exponent: {locale} {path} {other!r}")
            exponent = 0 if fallback else magnitude + 1 - skeleton["minimum_integer"]
            if not 0 <= exponent <= magnitude:
                raise ValueError(f"compact exponent outside numeric contract: {locale} {path} {exponent}")

            def transform(source):
                parsed = number_pattern(source, compact=True)
                if source != "0" and parsed["skeleton"] is not None:
                    selected = magnitude + 1 - parsed["skeleton"]["minimum_integer"]
                    if selected != exponent:
                        raise ValueError(f"plural-dependent compact exponent: {locale} {path} {source!r}")
                return {"pattern": self.signed(source, compact=True), "fallback": source == "0"}

            choices = self.choice(locale, path, transform, fallback="0", kind="compact")
            rows.append({"magnitude": magnitude, "exponent": exponent, "choices": choices})
        return self.add("compact_sets", rows)

    def currency_labels(self, locale):
        r = self.resolver
        rows = []
        for code in sorted(self.currency_codes):
            prefix = f"numbers/currencies/currency[@type='{code}']"
            symbol = r.text(locale, prefix + "/symbol", required=False)
            narrow = r.text(locale, prefix + "/symbol[@alt='narrow']", required=False)
            names = self.choice(locale, prefix + "/displayName[@count='{count}']", self.string, fallback=code, kind="string")
            rows.append({"code": code, "symbol": self.string(code if symbol is None else symbol),
                         "narrow": self.string(code if narrow is None else narrow), "names": names})
        return self.add("currency_sets", rows)

    def units_for_width(self, locale, width):
        r = self.resolver
        prefix = f"units/unitLength[@type='{width}']"
        derivation = self.component_locale(locale, self.derivations, {})
        rules = {**self.derivations["root"], **derivation}
        if rules["plural"] != ["compound", "one"] or rules["case"][0] != "compound":
            raise ValueError(f"review new per-unit derivation: {locale} {rules}")
        denominator_case = rules["case"][1]
        if denominator_case not in {"nominative", "accusative"}:
            raise ValueError(f"unresolved denominator case: {locale} {denominator_case}")

        def transform(source):
            return self.pattern(message_pattern(source, kind=UNIT, number_optional=True))

        units = []
        for unit in self.units:
            unit_path = prefix + f"/unit[@type='{self.unit_map[unit]}']"
            choices = self.choice(locale, unit_path + "/unitPattern[@count='{count}']", transform)
            per_unit = r.text(locale, unit_path + "/perUnitPattern", required=False)
            per_unit = None if per_unit is None else self.pattern(message_pattern(per_unit, kind=UNIT))
            denominator = r.text(locale, unit_path + f"/unitPattern[@count='one'][@case='{denominator_case}']")
            denominator_tokens = message_pattern(denominator, kind=UNIT, number_optional=True)
            try:
                denominator_tokens = strip_number(denominator_tokens)
            except MedialPlaceholder:
                display_name = r.text(locale, unit_path + "/displayName")
                denominator_tokens = message_pattern(display_name, kind=UNIT, arguments=())
                self.medial_denominators.append({"locale": locale, "width": width, "unit": unit, "pattern": denominator, "selected_display_name": display_name})
            denominator = self.pattern(denominator_tokens)
            units.append({"choices": choices, "per_unit": per_unit, "denominator": denominator})
        pairs = []
        for numerator, denominator, cldr_type in self.unit_pairs:
            unit_path = prefix + f"/unit[@type='{cldr_type}']/unitPattern[@count='{{count}}']"
            # A precomposed form is preferred only where CLDR actually resolves
            # it. Missing optional pairs compose from complete simple units.
            if r.text(locale, unit_path.format(count="other"), required=False) is None:
                continue
            choices = self.choice(locale, unit_path, transform)
            pairs.append({"numerator": numerator, "denominator": denominator, "choices": choices})
        per_pattern = r.text(locale, prefix + "/compoundUnit[@type='per']/compoundUnitPattern")
        return self.add("unit_sets", {"simple": units, "pairs": pairs,
                                     "per_pattern": self.pattern(message_pattern(per_pattern, kind=UNIT, arguments=(0, 1)))})

    @lru_cache(maxsize=None)
    def currency_unit_choices(self, locale):
        r = self.resolver
        values = []
        for count in (*CATEGORIES, "0", "1"):
            text = None
            for ancestor in r.lineage(locale):
                # ICU's CurrencyUnitPatterns resource is emitted only from a
                # locale's default system and then inherits as a locale resource.
                # In particular, ckb's default arab has no root arab unitPattern;
                # its currency-name placement inherits root's default latn row.
                system = r.text(ancestor, "numbers/defaultNumberingSystem")
                path = f"numbers/currencyFormats[@numberSystem='{system}']/unitPattern[@count='{count}']"
                leaf = r.resolve_local(ancestor, path, lateral=not count.isdigit())
                if leaf is not None:
                    if leaf.attributes:
                        raise ValueError(f"unconsumed currency unit attributes: {leaf}")
                    text = leaf.text
                    break
            if text is None and not count.isdigit():
                raise ValueError(f"missing currency unit resource: {locale} {count}")
            values.append(None if text is None else self.pattern(message_pattern(text, arguments=(0, 1))))
        return self.add("choices", {"kind": "pattern", "values": values})

    def numbering_profile(self, locale, system):
        r = self.resolver
        prefix = f"numbers/symbols[@numberSystem='{system}']"
        symbols = {}
        for name in ["decimal", "group", "plusSign", "minusSign", "percentSign", "approximatelySign", "exponential", "infinity", "nan"]:
            symbols[name] = self.string(r.text(locale, prefix + "/" + name))
        for name, fallback in [("currencyDecimal", "decimal"), ("currencyGroup", "group")]:
            text = r.text(locale, prefix + "/" + name, required=False)
            symbols[name] = symbols[fallback] if text is None else self.string(text)
        symbols = self.add("symbols", symbols)
        decimal_path = f"numbers/decimalFormats[@numberSystem='{system}']/decimalFormatLength/decimalFormat/pattern"
        percent_path = f"numbers/percentFormats[@numberSystem='{system}']/percentFormatLength/percentFormat/pattern"
        currency_prefix = f"numbers/currencyFormats[@numberSystem='{system}']"
        standard_path = currency_prefix + "/currencyFormatLength/currencyFormat[@type='standard']/pattern"
        accounting_path = currency_prefix + "/currencyFormatLength/currencyFormat[@type='accounting']/pattern"
        paths = {"decimal": decimal_path, "percent": percent_path, "currency": standard_path,
                 "accounting": accounting_path, "currency_alpha": standard_path + "[@alt='alphaNextToNumber']",
                 "accounting_alpha": accounting_path + "[@alt='alphaNextToNumber']"}
        patterns = {name: self.signed(r.text(locale, path)) for name, path in paths.items()}
        spacing = {}
        for side in ("beforeCurrency", "afterCurrency"):
            base = currency_prefix + "/currencySpacing/" + side
            spacing[side] = {name: r.text(locale, base + "/" + name)
                             for name in ["currencyMatch", "surroundingMatch", "insertBetween"]}
        for side in spacing.values():
            for name in ("currencyMatch", "surroundingMatch"):
                side[name] = self.add("unicode_sets", parse_unicode_set(side[name], self.unicode_properties))
            side["insertBetween"] = self.string(side["insertBetween"])
        range_pattern = r.text(locale, f"numbers/miscPatterns[@numberSystem='{system}']/pattern[@type='range']")
        unit_pattern = self.currency_unit_choices(locale)
        minimum = int(r.text(locale, "numbers/minimumGroupingDigits"))
        if not 1 <= minimum <= 3:
            raise ValueError(f"unresolved grouping minimum: {locale} {minimum}")
        overrides = []
        for code in sorted(self.currency_override_codes):
            cp = f"numbers/currencies/currency[@type='{code}']"
            override = {"code": code}
            for name in ("decimal", "group", "pattern"):
                source = r.text(locale, cp + "/" + name, required=False)
                override[name] = None if source is None else self.signed(source) if name == "pattern" else self.string(source)
            if any(override[name] is not None for name in ("decimal", "group", "pattern")):
                overrides.append(override)
        return self.add("numbering_profiles", {
            "symbols": symbols, "patterns": patterns, "minimum_grouping": minimum,
            "compact_short": self.compact(locale, system, "short"),
            "compact_long": self.compact(locale, system, "long"),
            "compact_currency": self.compact(locale, system, "currency"),
            "compact_currency_alpha": self.compact(locale, system, "currency", alpha=True),
            "range_pattern": self.pattern(message_pattern(range_pattern, arguments=(0, 1))),
            "currency_unit_pattern": unit_pattern, "currency_overrides": overrides,
            "currency_spacing": spacing,
        })

    def locale_profile(self, locale):
        r = self.resolver
        default = r.text(locale, "numbers/defaultNumberingSystem")
        if default not in self.system_indexes:
            raise ValueError(f"default is not an admitted decimal numbering system: {locale} {default}")
        numbering = [self.numbering_profile(locale, system["name"]) for system in self.systems]
        plural, _ = self.rules(locale)
        ranges = self.component_locale(locale, self.range_by_locale, self.root_range)
        return self.add("profiles", {
            "default_numbering": self.system_indexes[default], "numbering": numbering,
            "plural_rules": plural, "plural_ranges": ranges,
            "currencies": self.currency_labels(locale),
            "units": [self.units_for_width(locale, width) for width in ("short", "narrow", "long")],
        })


def product_content(name, content):
    """The reproducible bytes of a product: gzip streams are compared and hashed
    by their decompressed content, because zlib implementations (zlib, zlib-ng)
    compress identical input to different bytes."""
    return gzip.decompress(content) if name.endswith(".gz") else content


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--locales", help="diagnostic extraction only, comma-separated raw CLDR IDs")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    extractor = Extractor(STAGE)
    resolver = extractor.resolver
    diagnostics = []
    candidates = (resolver.locales | resolver.default_content) - {"root"}
    for locale in sorted(candidates):
        try:
            tuple(resolver.lineage(locale))
        except ValueError:
            # Reviewed upstream inconsistency: there is no ife source in the
            # complete pinned tree. Real main locales can never use this path.
            if locale not in resolver.locales and locale == "ife_TG":
                diagnostics.append({"locale": locale, "kind": "DanglingDefaultContent", "missing_parent": "ife",
                                    "authority": "supplementalMetadata.xml defaultContent versus complete pinned Git tree"})
                continue
            raise
    candidates -= {row["locale"] for row in diagnostics}
    complete = args.locales is None
    if args.locales is not None:
        candidates = set(args.locales.split(","))
    output = STAGE / "work/crates/lila-intl/data/number-cldr-47" if complete else STAGE / "evidence/diagnostic-profile"
    output.mkdir(parents=True, exist_ok=True)
    locales = {}
    provenance_leaves = Pool()
    provenance_sets = Pool()
    provenance_locales = {}
    for position, locale in enumerate(sorted(candidates)):
        index = extractor.locale_profile(locale)
        canonical = resolver.canonical_locale(locale)
        if canonical in locales and locales[canonical]["profile"] != index:
            raise ValueError(f"canonical locale alias has conflicting profiles: {locale} -> {canonical}")
        locales.setdefault(canonical, {"profile": index, "sources": []})["sources"].append(locale)
        selected_sources = set()
        for value in resolver.consumed.values():
            selected_sources.add(provenance_leaves.add({
                "source_locale": value.locale, "source_path": path_text(value.path),
                "draft": value.draft, "value_sha256": hashlib.sha256(value.text.encode()).hexdigest(),
            }))
        provenance_locales[locale] = provenance_sets.add(sorted(selected_sources))
        resolver.clear_locale_cache()
        if position % 25 == 0 or position + 1 == len(candidates):
            print(f"profiles {position + 1}/{len(candidates)}: {locale}", file=sys.stderr, flush=True)
    canonical = {
        "schema": 1, "cldr_commit": "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c",
        "complete": complete, "default_locale": "en-US", "locales": locales,
        "numbering_systems": extractor.systems, "sanctioned_units": extractor.units,
        "unicode_properties": extractor.property_sets,
        "currency_fractions": {"default": extractor.default_fraction, "overrides": extractor.fractions},
        "source_diagnostics": diagnostics, "tables": {name: pool.rows for name, pool in extractor.pools.items()},
    }
    products = {
        "profiles.json.gz": gzip.compress((stable(canonical) + "\n").encode(), mtime=0),
        "profile-provenance.json.gz": gzip.compress((stable({"source_leaves": provenance_leaves.rows, "selection_sets": provenance_sets.rows, "locales": provenance_locales}) + "\n").encode(), mtime=0),
        "medial-denominator-policy.json": (json.dumps(extractor.medial_denominators, ensure_ascii=False, indent=2) + "\n").encode(),
        "plural-samples.json": (json.dumps(extractor.plural_samples, ensure_ascii=False, indent=2) + "\n").encode(),
    }
    summary = {"complete": complete, "canonical_locales": len(locales), "raw_candidates": len(candidates),
               "numbering_systems": len(extractor.systems), "sanctioned_units": len(extractor.units),
               "precomposed_pairs": len(extractor.unit_pairs), "currency_codes": len(extractor.currency_codes),
               "table_rows": {name: len(pool.rows) for name, pool in extractor.pools.items()},
               "source_diagnostics": diagnostics,
               "products": [{"path": name, "sha256": hashlib.sha256(product_content(name, content)).hexdigest(),
                             "bytes": len(product_content(name, content))} for name, content in products.items()]}
    products["coverage.json"] = (json.dumps(summary, indent=2) + "\n").encode()
    for name, content in products.items():
        path = output / name
        if args.check:
            if product_content(name, path.read_bytes()) != product_content(name, content):
                raise ValueError(f"non-reproducible generated profile: {path}")
        else:
            path.write_bytes(content)
    print(json.dumps(summary, ensure_ascii=False))


if __name__ == "__main__":
    main()
