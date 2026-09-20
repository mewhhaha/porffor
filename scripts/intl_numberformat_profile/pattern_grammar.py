"""Closed parsers for the selected CLDR47 numeric and message patterns."""
import re
from functools import lru_cache

# These names are serialized to the matching closed Rust token enum.
LITERAL = "Literal"
NUMBER = "Number"
MINUS = "MinusSign"
PLUS = "PlusSign"
PERCENT = "PercentSign"
CURRENCY = "Currency"
COMPACT = "Compact"
UNIT = "Unit"
ARGUMENT1 = "Argument1"


def append_text(tokens, kind, text):
    if not text:
        return
    if tokens and tokens[-1][0] == kind and kind in {LITERAL, COMPACT, UNIT}:
        tokens[-1] = (kind, tokens[-1][1] + text)
    else:
        tokens.append((kind, text))


def annotated_text(tokens, text, kind):
    if kind == LITERAL:
        append_text(tokens, kind, text)
        return
    # Space and bidi controls adjacent to the label remain literal parts. The
    # runtime still owns them with that measurement/notation when collapsing.
    def boundary(character):
        return character.isspace() or character in "\u061c\u200e\u200f\u202a\u202b\u202c\u202d\u202e\u2066\u2067\u2068\u2069"
    first = 0
    last = len(text)
    while first < last and boundary(text[first]):
        first += 1
    while last > first and boundary(text[last - 1]):
        last -= 1
    append_text(tokens, LITERAL, text[:first])
    append_text(tokens, kind, text[first:last])
    append_text(tokens, LITERAL, text[last:])


@lru_cache(maxsize=None)
def number_pattern(source, *, compact=False):
    subpatterns = [[]]
    quoted = False
    position = 0
    while position < len(source):
        char = source[position]
        if char == "'":
            if position + 1 < len(source) and source[position + 1] == "'":
                subpatterns[-1].append(("'", True))
                position += 2
                continue
            quoted = not quoted
        elif char == ";" and not quoted:
            subpatterns.append([])
        else:
            subpatterns[-1].append((char, quoted))
        position += 1
    if quoted or len(subpatterns) > 2 or any(not value for value in subpatterns):
        raise ValueError(f"invalid number pattern: {source!r}")

    skeletons = []
    parsed = []
    for subpattern in subpatterns:
        tokens = []
        literal = []
        index = 0
        skeleton = None

        def flush():
            annotated_text(tokens, "".join(literal), COMPACT if compact else LITERAL)
            literal.clear()

        while index < len(subpattern):
            char, quoted = subpattern[index]
            if not quoted and char in "#0":
                if skeleton is not None:
                    raise ValueError(f"multiple number skeletons: {source!r}")
                flush()
                start = index
                while index < len(subpattern) and not subpattern[index][1] and subpattern[index][0] in "#0,.":
                    index += 1
                spelling = "".join(c for c, _ in subpattern[start:index])
                if not re.fullmatch(r"[#0]+(?:,[#0]+)*(?:\.[#0]+)?", spelling):
                    raise ValueError(f"unknown numeric skeleton: {source!r}: {spelling!r}")
                integer = spelling.split(".")[0]
                groups = integer.split(",")
                primary = len(groups[-1]) if len(groups) > 1 else 0
                secondary = len(groups[-2]) if len(groups) > 2 else primary
                skeleton = {"minimum_integer": integer.count("0"), "primary_group": primary, "secondary_group": secondary}
                tokens.append((NUMBER, ""))
                continue
            if not quoted and char in "-+%¤":
                flush()
                token = {"-": MINUS, "+": PLUS, "%": PERCENT, "¤": CURRENCY}[char]
                if char == "¤" and index + 1 < len(subpattern) and subpattern[index + 1] == ("¤", False):
                    raise ValueError(f"unresolved multi-currency token: {source!r}")
                tokens.append((token, ""))
            elif not quoted and char in "@*‰":
                raise ValueError(f"unresolved numeric metacharacter: {source!r}")
            else:
                literal.append(char)
            index += 1
        flush()
        if skeleton is None and not compact:
            raise ValueError(f"missing number skeleton: {source!r}")
        parsed.append(tokens)
        skeletons.append(skeleton)
    if len(parsed) == 1:
        parsed.append([(MINUS, ""), *parsed[0]])
    if len(skeletons) == 2 and skeletons[0] != skeletons[1]:
        # LDML ignores the negative number skeleton: only its affixes matter.
        # Consume it syntactically but keep the positive grouping authority.
        pass
    return {"positive": parsed[0], "negative": parsed[1], "skeleton": skeletons[0]}


@lru_cache(maxsize=None)
def message_pattern(source, *, kind=LITERAL, arguments=(0,), number_optional=False):
    tokens = []
    text = []
    count = {argument: 0 for argument in arguments}
    position = 0
    quoted = False
    while position < len(source):
        character = source[position]
        if character == "'":
            if position + 1 < len(source) and source[position + 1] == "'":
                text.append("'")
                position += 2
                continue
            if quoted:
                quoted = False
                position += 1
                continue
            if position + 1 < len(source) and source[position + 1] in "{}":
                quoted = True
                position += 1
                continue
        if character == "{" and not quoted:
            end = source.find("}", position + 1)
            if end < 0 or source[position + 1:end] not in {str(x) for x in arguments}:
                raise ValueError(f"unknown message argument: {source!r}")
            argument = int(source[position + 1:end])
            count[argument] += 1
            annotated_text(tokens, "".join(text), kind)
            text.clear()
            tokens.append((NUMBER if argument == 0 else ARGUMENT1, ""))
            position = end + 1
            continue
        if character == "}" and not quoted:
            raise ValueError(f"unmatched message brace: {source!r}")
        text.append(character)
        position += 1
    annotated_text(tokens, "".join(text), kind)
    if quoted:
        raise ValueError(f"unclosed message quote: {source!r}")
    for argument, occurrences in count.items():
        if occurrences != 1 and not (number_optional and argument == 0 and occurrences == 0):
            raise ValueError(f"invalid message arity: {source!r}: {count}")
    return tokens


class MedialPlaceholder(ValueError):
    pass


def strip_number(tokens):
    """Extract a unit label for a denominator, including its internal spaces."""
    indexes = [i for i, token in enumerate(tokens) if token[0] == NUMBER]
    if not indexes:
        return tokens
    if len(indexes) != 1:
        raise ValueError("unit has multiple numeric placeholders")
    index = indexes[0]
    before = tokens[:index]
    after = tokens[index + 1:]
    meaningful_before = any(kind == UNIT and text for kind, text in before)
    meaningful_after = any(kind == UNIT and text for kind, text in after)
    if meaningful_before and meaningful_after:
        raise MedialPlaceholder("compound denominator has a medial numeric placeholder")
    tokens = [*before, *after]
    while tokens and tokens[0][0] == LITERAL and not tokens[0][1].strip():
        tokens.pop(0)
    while tokens and tokens[-1][0] == LITERAL and not tokens[-1][1].strip():
        tokens.pop()
    return tokens
