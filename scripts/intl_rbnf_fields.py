"""Resolve finite date-field numbering from pinned RBNF, only at generation."""

from dataclasses import dataclass
import re


@dataclass(frozen=True)
class Rule:
    base: int
    divisor: int
    source: str


class FiniteRbnf:
    def __init__(self, document, grouping):
        groups = document.findall(f"./rbnf/rulesetGrouping[@type='{grouping}']")
        if len(groups) != 1:
            raise ValueError(f"missing or duplicate RBNF grouping: {grouping}")
        self.rules = {}
        self.consumed = set()
        for ruleset in groups[0].findall("ruleset"):
            name = ruleset.attrib["type"]
            if name in self.rules:
                raise ValueError(f"duplicate RBNF ruleset: {name}")
            rules = []
            for node in ruleset.findall("rbnfrule"):
                value = node.attrib["value"]
                if not value.isascii() or not value.isdecimal():
                    # Negative and fractional rules cannot be selected by the
                    # admitted, finite nonnegative calendar field domain.
                    continue
                if set(node.attrib) != {"value"} or node.text is None:
                    raise ValueError(f"unsupported finite RBNF rule: {name}/{value}")
                base = int(value)
                divisor = 10 ** max(0, len(str(base)) - 1)
                source = node.text.strip()
                if not source.endswith(";") or ";" in source[:-1]:
                    raise ValueError(f"invalid RBNF rule terminator: {name}/{value}")
                rules.append(Rule(base, divisor, source[:-1]))
            if not rules or any(left.base >= right.base for left, right in zip(rules, rules[1:])):
                raise ValueError(f"invalid RBNF rule ordering: {name}")
            self.rules[name] = rules

    def format(self, ruleset, value, stack=()):
        if value < 0 or (ruleset, value) in stack or len(stack) >= 64:
            raise ValueError("finite RBNF substitution escaped its checked domain")
        rules = self.rules.get(ruleset)
        if rules is None:
            raise ValueError(f"missing RBNF substitution ruleset: {ruleset}")
        selected = next((rule for rule in reversed(rules) if rule.base <= value), None)
        if selected is None:
            raise ValueError(f"RBNF rules do not cover {ruleset}/{value}")
        self.consumed.add((ruleset, selected.base, selected.source))
        stack = (*stack, (ruleset, value))

        def render(source):
            output = []
            index = 0
            while index < len(source):
                character = source[index]
                if character == "[":
                    end = source.find("]", index + 1)
                    if end < 0 or "[" in source[index + 1:end]:
                        raise ValueError("unsupported nested or unclosed RBNF optional text")
                    if value % selected.divisor:
                        output.append(render(source[index + 1:end]))
                    index = end + 1
                    continue
                if character in "=←→":
                    end = source.find(character, index + 1)
                    if end < 0:
                        raise ValueError("unterminated RBNF substitution")
                    target = source[index + 1:end]
                    if target:
                        match = re.fullmatch(r"%{1,2}([a-z0-9-]+)", target)
                        if match is None:
                            raise ValueError(f"unsupported required RBNF substitution: {target}")
                        target = match[1]
                    elif character == "=":
                        raise ValueError("RBNF identity substitution requires a ruleset")
                    else:
                        target = ruleset
                    operand = value
                    if character == "←":
                        operand //= selected.divisor
                    elif character == "→":
                        operand %= selected.divisor
                    output.append(self.format(target, operand, stack))
                    index = end + 1
                    continue
                if character in "]'%":
                    raise ValueError(f"unsupported required RBNF syntax: {source}")
                output.append(character)
                index += 1
            return "".join(output)

        return render(selected.source)


def algorithmic_field_tables(profile, locales):
    definitions = {
        node.attrib["id"]: node.attrib
        for node in profile.documents["common/supplemental/numberingSystems.xml"]
        .findall("./numberingSystems/numberingSystem")
    }
    required = set()

    def inspect(value):
        if isinstance(value, dict):
            for override in value.get("numbering_overrides", ()):
                definition = definitions.get(override["numbering"])
                if definition is None:
                    raise ValueError(f"pattern names an unknown numbering system: {override}")
                if definition["type"] == "algorithmic":
                    required.add((override["numbering"], override["field"]))
            for child in value.values():
                inspect(child)
        elif isinstance(value, list):
            for child in value:
                inspect(child)

    inspect(locales)
    tables = []
    for identifier, field in sorted(required):
        # A day has a finite exact domain for every admitted calendar. Other
        # algorithmic fields need their own complete domain before admission.
        if field != "d":
            raise ValueError(f"algorithmic numbering has no checked finite field domain: {identifier}/{field}")
        locale, grouping, ruleset = definitions[identifier]["rules"].split("/")
        path = f"common/rbnf/{locale}.xml"
        if path not in profile.documents:
            raise ValueError(f"missing pinned RBNF source: {path}")
        compiler = FiniteRbnf(profile.documents[path], grouping)
        values = [compiler.format(ruleset, value) for value in range(1, 32)]
        if any(not value for value in values):
            raise ValueError(f"empty finite RBNF field: {identifier}/{field}")
        tables.append({"identifier": identifier, "field": field, "minimum": 1,
                       "values": values, "source": path, "ruleset": ruleset,
                       "consumed_rules": [list(rule) for rule in sorted(compiler.consumed)]})
    return tables
