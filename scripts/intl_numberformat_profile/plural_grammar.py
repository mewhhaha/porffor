"""Closed LDML cardinal predicates and exact generator-side sample checks."""
from decimal import Decimal
import re

CATEGORIES = ("zero", "one", "two", "few", "many", "other")
OPERANDS = ("n", "i", "v", "w", "f", "t", "c", "e")
RELATION = re.compile(r"([nivwftce])(?:\s+(?:%|mod)\s+(\d+))?\s+(is not|not within|not in|within|in|is|!=|=)\s+(.+)")


def parse_rule(source):
    condition = source.split("@", 1)[0].strip()
    if not condition:
        return []
    result = []
    for disjunction in re.split(r"\s+or\s+", condition):
        conjunction = []
        for relation in re.split(r"\s+and\s+", disjunction):
            match = RELATION.fullmatch(relation.strip())
            if match is None:
                raise ValueError(f"unresolved plural relation: {relation!r}")
            operand, modulus, operator, interval_text = match.groups()
            modulus = None if modulus is None else int(modulus)
            if modulus is not None and not 0 < modulus <= 2**64 - 1:
                raise ValueError(f"plural modulus outside exact domain: {modulus}")
            intervals = []
            for interval in interval_text.split(","):
                values = interval.strip().split("..")
                if len(values) not in (1, 2) or not all(re.fullmatch(r"\d+", x) for x in values):
                    raise ValueError(f"unresolved plural interval: {interval!r}")
                lower, upper = int(values[0]), int(values[-1])
                if not 0 <= lower <= upper <= 2**64 - 1:
                    raise ValueError(f"plural interval outside exact domain: {interval}")
                intervals.append((lower, upper))
            conjunction.append({"operand": operand, "modulus": modulus,
                                "integer_only": "within" not in operator,
                                "negate": operator.startswith("not") or operator in {"!=", "is not"},
                                "ranges": intervals})
        result.append(conjunction)
    return result


def sample_operands(spelling):
    match = re.fullmatch(r"(\d+)(?:\.(\d+))?(?:c(\d+))?", spelling)
    if match is None:
        raise ValueError(f"unknown CLDR sample: {spelling}")
    integer, fraction, compact = match.groups()
    fraction = fraction or ""
    compact = int(compact or 0)
    digits = integer + fraction
    split = len(integer) + compact
    if split >= len(digits):
        integer, fraction = digits + "0" * (split - len(digits)), ""
    else:
        integer, fraction = digits[:split], digits[split:]
    trimmed = fraction.rstrip("0")
    return {"n": Decimal(integer + ("." + fraction if fraction else "")),
            "i": Decimal(integer), "v": len(fraction), "w": len(trimmed),
            "f": int(fraction or "0"), "t": int(trimmed or "0"),
            "c": compact, "e": compact}


def evaluate(rule, operands):
    for conjunction in rule:
        for relation in conjunction:
            value = operands[relation["operand"]]
            if relation["modulus"] is not None:
                value %= relation["modulus"]
            matches = (not relation["integer_only"] or value == int(value)) and any(
                lower <= value <= upper for lower, upper in relation["ranges"])
            if matches == relation["negate"]:
                break
        else:
            return True
    return False


def category(rules, operands):
    for name, rule in rules:
        if name != "other" and evaluate(rule, operands):
            return name
    return "other"


def check_samples(rules, source_rules):
    samples = []
    for name, source in source_rules:
        for kind, values in re.findall(r"@(integer|decimal)\s+([^@]+)", source):
            for interval in values.strip().split(","):
                interval = interval.strip()
                if interval == "…":
                    continue
                ends = interval.split("~")
                for spelling in dict.fromkeys(ends):
                    operands = sample_operands(spelling)
                    observed = category(rules, operands)
                    if observed != name:
                        raise ValueError(f"plural sample disagrees with rules: {spelling} {observed} != {name}")
                    samples.append({"category": name, "kind": kind, "spelling": spelling})
    return samples
