"""Validate every reached pattern role in the canonical NumberFormat extract."""
from collections import Counter


def validate(profile):
    tables = profile["tables"]
    strings = tables["strings"]
    checked = set()

    def pattern(index, role):
        if (index, role) in checked:
            return
        checked.add((index, role))
        tokens = tables["patterns"][index]
        counts = Counter(kind for kind, _ in tokens)
        allowed = {"Literal", "Number"}
        if role in {"decimal", "percent", "currency", "compact", "compact_currency"}:
            allowed |= {"MinusSign", "PlusSign"}
        if role in {"currency", "compact_currency"}:
            allowed.add("Currency")
        if role == "percent":
            allowed.add("PercentSign")
        if role in {"compact", "compact_currency"}:
            allowed.add("Compact")
        if role in {"message", "per"}:
            allowed.add("Argument1")
        if role in {"unit", "denominator", "per_unit", "per"}:
            allowed.add("Unit")
        valid = not (set(counts) - allowed)
        if role in {"compact", "compact_currency", "unit"}:
            valid &= counts["Number"] <= 1
        else:
            valid &= counts["Number"] == (0 if role == "denominator" else 1)
        valid &= counts["Currency"] == int(role in {"currency", "compact_currency"})
        valid &= counts["Argument1"] == int(role in {"message", "per"})
        valid &= counts["PercentSign"] == int(role == "percent")
        if not valid:
            raise ValueError(f"invalid pattern role {index}/{role}: {tokens}")
        for kind, text in tokens:
            if not 0 <= text < len(strings):
                raise ValueError(f"invalid pattern text index: {index}/{text}")
            if kind not in {"Literal", "Compact", "Unit"} and strings[text] != "":
                raise ValueError(f"unexpected structural token text: {index}/{kind}")

    def signed(index, role):
        row = tables["signed_patterns"][index]
        pattern(row["positive"], role)
        pattern(row["negative"], role)

    def choices(index, role):
        row = tables["choices"][index]
        if row["kind"] != "pattern" or len(row["values"]) != 8 or any(value is None for value in row["values"][:6]):
            raise ValueError(f"invalid pattern choice: {index}")
        for value in row["values"]:
            if value is not None:
                pattern(value, role)

    for row in tables["numbering_profiles"]:
        for key, index in row["patterns"].items():
            signed(index, "decimal" if key == "decimal" else "percent" if key == "percent" else "currency")
        pattern(row["range_pattern"], "message")
        choices(row["currency_unit_pattern"], "message")
        for override in row["currency_overrides"]:
            if override["pattern"] is not None:
                signed(override["pattern"], "currency")
        for key in ["compact_short", "compact_long", "compact_currency", "compact_currency_alpha"]:
            rows = tables["compact_sets"][row[key]]
            for magnitude, compact in enumerate(rows):
                if compact["magnitude"] != magnitude or not 0 <= compact["exponent"] <= magnitude:
                    raise ValueError("invalid compact exponent extent")
                choice = tables["choices"][compact["choices"]]
                if choice["kind"] != "compact":
                    raise ValueError("wrong compact choice kind")
                for selected in choice["values"]:
                    if selected is not None and not selected["fallback"]:
                        signed(selected["pattern"], "compact_currency" if "currency" in key else "compact")
        compact = tables["compact_sets"][row["compact_currency"]]
        alpha = tables["compact_sets"][row["compact_currency_alpha"]]
        if [(r["magnitude"], r["exponent"]) for r in compact] != [(r["magnitude"], r["exponent"]) for r in alpha]:
            raise ValueError("currency alpha pattern changed the compact exponent authority")
    for row in tables["unit_sets"]:
        if len(row["simple"]) != len(profile["sanctioned_units"]):
            raise ValueError("incomplete sanctioned unit set")
        for unit in row["simple"]:
            choices(unit["choices"], "unit")
            pattern(unit["denominator"], "denominator")
            if unit["per_unit"] is not None:
                pattern(unit["per_unit"], "per_unit")
        for pair in row["pairs"]:
            choices(pair["choices"], "unit")
        pattern(row["per_pattern"], "per")
    if profile["default_locale"] not in profile["locales"]:
        raise ValueError("missing default locale")
    if len(profile["numbering_systems"]) != 77:
        raise ValueError("incomplete decimal numbering-system inventory")
    for row in tables["profiles"]:
        if len(row["numbering"]) != 77 or not 0 <= row["default_numbering"] < 77 or len(row["units"]) != 3:
            raise ValueError("incomplete locale profile")
    return {"validated_pattern_roles": len(checked), "locale_count": len(profile["locales"])}
