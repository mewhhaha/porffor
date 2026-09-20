"""Compile the closed LDML pattern domain used by ECMA date/time fields."""

import re


# These skeleton requests have no corresponding ECMA-402 option. U requests a
# cyclic year name, whereas the year option is numeric. U remains a supported
# output field when the locale supplies it for a numeric-year skeleton or style.
NON_ECMA_REQUEST_FIELDS = frozenset("YuUqQwWDFgA")
FIELD_WIDTHS = {
    "G": range(1, 6), "y": range(1, 7), "r": range(1, 7), "U": range(1, 6),
    "M": range(1, 6), "L": range(1, 6), "d": range(1, 3),
    "E": range(1, 7), "e": range(3, 7), "c": range(3, 7),
    "a": range(1, 6), "b": range(1, 6), "B": range(1, 6),
    "h": range(1, 3), "H": range(1, 3), "K": range(1, 3), "k": range(1, 3),
    "m": range(1, 3), "s": range(1, 3), "S": range(1, 4),
    "z": range(1, 5), "v": (1, 4), "O": (1, 4),
}


def compile_pattern(source, *, placeholders=()):
    tokens = []
    literal = []
    quoted = False
    index = 0

    def flush_literal():
        if literal:
            tokens.append({"literal": "".join(literal)})
            literal.clear()

    while index < len(source):
        character = source[index]
        if character == "'":
            if source[index:index + 2] == "''":
                literal.append("'")
                index += 2
                continue
            quoted = not quoted
            index += 1
            continue
        if not quoted and character == "{" and placeholders:
            match = re.match(r"\{([0-9])\}", source[index:])
            if match is None or int(match[1]) not in placeholders:
                raise ValueError(f"unexpected pattern placeholder: {source!r}")
            flush_literal()
            tokens.append({"placeholder": int(match[1])})
            index += len(match[0])
            continue
        if not quoted and character.isascii() and character.isalpha():
            end = index + 1
            while end < len(source) and source[end] == character:
                end += 1
            width = end - index
            if character not in FIELD_WIDTHS or width not in FIELD_WIDTHS[character]:
                raise ValueError(f"unsupported required pattern field {character * width}: {source!r}")
            flush_literal()
            tokens.append({"field": character, "width": width})
            index = end
            continue
        if not quoted and character in "{}":
            raise ValueError(f"unquoted brace in date/time pattern: {source!r}")
        literal.append(character)
        index += 1
    if quoted:
        raise ValueError(f"unterminated LDML quote: {source!r}")
    flush_literal()
    actual = sorted(token["placeholder"] for token in tokens if "placeholder" in token)
    if actual != sorted(placeholders):
        raise ValueError(f"missing or repeated pattern placeholder: {source!r}")
    return tokens


def skeleton_in_profile(skeleton):
    if not re.fullmatch(r"[A-Za-z]+", skeleton):
        raise ValueError(f"invalid date/time skeleton: {skeleton!r}")
    if set(skeleton) & NON_ECMA_REQUEST_FIELDS:
        return False
    for symbol in set(skeleton):
        if symbol not in FIELD_WIDTHS and symbol not in "jJC":
            raise ValueError(f"unknown skeleton symbol: {symbol}")
    return True


def interval_field(symbol):
    if symbol in "yUr":
        return {"y": "year", "U": "yearName", "r": "relatedYear"}[symbol]
    if symbol in "ML":
        return "month"
    if symbol in "Eec":
        return "weekday"
    if symbol in "abB":
        return "dayPeriod"
    if symbol in "hHKk":
        return "hour"
    return {"G": "era", "d": "day", "m": "minute", "s": "second", "S": "fractionalSecond", "z": "timeZoneName", "v": "timeZoneName", "O": "timeZoneName"}[symbol]


def compile_interval(source):
    order = "earliest_first"
    pattern = source
    for prefix, selected in [("earliestFirst:", "earliest_first"), ("latestFirst:", "latest_first")]:
        if pattern.startswith(prefix):
            order = selected
            pattern = pattern[len(prefix):]
            break
    tokens = compile_pattern(pattern)
    seen = set()
    split = None
    for index, token in enumerate(tokens):
        if "field" not in token:
            continue
        # r and U represent distinct output fields of the same calendar year;
        # encountering both is not the repeated-year boundary of an interval.
        field = interval_field(token["field"])
        if field in seen:
            split = index
            break
        seen.add(field)
    if split is None:
        raise ValueError(f"interval has no repeated field: {source!r}")
    left = {interval_field(token["field"]) for token in tokens[:split] if "field" in token}
    right = {interval_field(token["field"]) for token in tokens[split:] if "field" in token}
    shared = sorted(left ^ right)
    return {"tokens": tokens, "second_start": split, "shared_fields": shared, "endpoint_order": order}
