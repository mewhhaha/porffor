"""Pinned LDML attribute roles and defaults, derived from its annotated DTD."""

from dataclasses import dataclass
from enum import Enum
import re


class AttributeRole(Enum):
    DISTINGUISHING = "distinguishing"
    VALUE = "value"
    METADATA = "metadata"


@dataclass(frozen=True)
class AttributeRule:
    role: AttributeRole
    default: str | None


NAME = r"[A-Za-z_:][A-Za-z0-9_.:-]*"
DECLARATION = re.compile(
    rf'<!ATTLIST\s+({NAME})\s+({NAME})\s+(?:\([^>]*?\)|{NAME})\s+'
    r'(#IMPLIED|#REQUIRED|(?:#FIXED\s+)?"[^"]*")\s*>'
)
COMMENTS = re.compile(r"(?:\s*<!--.*?-->)*", re.DOTALL)


class LdmlSchema:
    def __init__(self, source):
        self.attributes = {}
        self.value_defaults = {}
        declarations = list(DECLARATION.finditer(source))
        if len(declarations) != source.count("<!ATTLIST"):
            raise ValueError("unsupported pinned LDML attribute declaration")
        for declaration in declarations:
            tag, name, default = declaration.groups()
            comments = COMMENTS.match(source, declaration.end()).group()
            roles = [role for role in (AttributeRole.VALUE, AttributeRole.METADATA)
                     if f"<!--@{role.value.upper()}-->" in comments]
            if len(roles) > 1:
                raise ValueError(f"conflicting LDML attribute roles: {tag}@{name}")
            role = roles[0] if roles else AttributeRole.DISTINGUISHING
            default = re.search(r'"([^"]*)"', default)
            rule = AttributeRule(role, None if default is None else default[1])
            if (tag, name) in self.attributes:
                raise ValueError(f"duplicate LDML attribute declaration: {tag}@{name}")
            self.attributes[tag, name] = rule
            if role is AttributeRole.VALUE and rule.default is not None:
                self.value_defaults.setdefault(tag, {})[name] = rule.default

    def rule(self, tag, name):
        xml_namespace = "{http://www.w3.org/XML/1998/namespace}"
        if name.startswith(xml_namespace):
            name = "xml:" + name[len(xml_namespace):]
        try:
            return self.attributes[tag, name]
        except KeyError as error:
            raise ValueError(f"attribute absent from pinned LDML schema: {tag}@{name}") from error

    def distinguishing(self, tag, attributes):
        return tuple(sorted(
            (name, value) for name, value in attributes.items()
            if self.rule(tag, name).role is AttributeRole.DISTINGUISHING
            and self.rule(tag, name).default != value
        ))

    def values(self, tag, attributes):
        values = {
            name: value for name, value in attributes.items()
            if self.rule(tag, name).role is AttributeRole.VALUE
        }
        for name, default in self.value_defaults.get(tag, {}).items():
            values.setdefault(name, default)
        return tuple(sorted(values.items()))

    def normalize_predicates(self, tag, attributes):
        result = []
        for name, value in attributes:
            rule = self.rule(tag, name)
            if rule.role is not AttributeRole.DISTINGUISHING:
                raise ValueError(f"LDML alias predicate is not distinguishing: {tag}@{name}")
            if rule.default != value:
                result.append((name, value))
        return tuple(result)
