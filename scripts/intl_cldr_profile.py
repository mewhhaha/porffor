"""Checked LDML leaf inheritance for the pinned date/time profile generator."""

from dataclasses import dataclass
from enum import Enum
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

from intl_ldml_schema import LdmlSchema

CLDR_COMMIT = "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
INHERIT = "↑↑↑"
NO_INHERIT = "∅∅∅"
DRAFT_RANK = {"unconfirmed": 0, "provisional": 1, "contributed": 2, "approved": 3}
ATTRIBUTE = re.compile(r"\[@([A-Za-z][A-Za-z0-9_-]*)=(?:'([^']*)'|\"([^\"]*)\")\]")


class PatternAlternate(Enum):
    DEFAULT = None
    ASCII = "ascii"


@dataclass(frozen=True, order=True)
class Segment:
    tag: str
    attributes: tuple = ()

    def get(self, attribute, default=None):
        return dict(self.attributes).get(attribute, default)

    def __str__(self):
        return self.tag + "".join(f"[@{key}='{value}']" for key, value in self.attributes)


def segment(tag, **attributes):
    return Segment(tag, tuple(sorted(attributes.items())))


def parse_path(source, base=()):
    # LDML alias predicates may contain a slash in their quoted attribute.
    components = []
    start = 0
    quote = None
    for index, character in enumerate(source):
        if quote is not None:
            if character == quote:
                quote = None
        elif character in ("'", '"'):
            quote = character
        elif character == "/":
            components.append(source[start:index])
            start = index + 1
    if quote is not None:
        raise ValueError(f"unterminated LDML alias quote: {source}")
    components.append(source[start:])
    result = list(base)
    for component in components:
        if component in ("", "."):
            continue
        if component == "..":
            if not result:
                raise ValueError(f"alias escapes LDML root: {source}")
            result.pop()
            continue
        name = component.split("[", 1)[0]
        if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_-]*", name):
            raise ValueError(f"unsupported LDML path component: {component}")
        attributes = ATTRIBUTE.findall(component[len(name):])
        if ATTRIBUTE.sub("", component[len(name):]):
            raise ValueError(f"unsupported LDML predicate: {component}")
        attributes = tuple(sorted((key, single or double) for key, single, double in attributes))
        if len(dict(attributes)) != len(attributes):
            raise ValueError(f"duplicate LDML predicate: {component}")
        result.append(Segment(name, attributes))
    return tuple(result)


def path_text(path):
    return "/".join(map(str, path))


def normalize_path(path, schema):
    return tuple(Segment(part.tag, schema.normalize_predicates(part.tag, part.attributes))
                 for part in path)


@dataclass(frozen=True)
class LdmlValue:
    text: str | None
    attributes: tuple


@dataclass(frozen=True)
class ResolvedLeaf:
    value: str
    attributes: tuple
    source_locale: str
    source_path: tuple


class LocaleTree:
    def __init__(self, locale, document, minimum_draft, schema, *, pattern_alternate=PatternAlternate.DEFAULT):
        self.locale = locale
        self.leaves = {}
        self.aliases = {}
        self.children = {}
        self.pattern_alternates = {}

        def visit(node, path, draft):
            if node.tag == "special":
                return
            rank = DRAFT_RANK[node.get("draft", draft)]
            if node.tag == "alias":
                if rank < minimum_draft:
                    return
                if path in self.aliases:
                    raise ValueError(f"duplicate alias: {locale}/{path_text(path)}")
                source = node.attrib["source"]
                target = normalize_path(parse_path(node.attrib.get("path", "."), path), schema)
                if source != "locale":
                    raise ValueError(f"unsupported cross-locale alias source: {source}")
                self.aliases[path] = target
                return
            if not list(node):
                if rank < minimum_draft:
                    return
                attributes = schema.values(node.tag, node.attrib)
                if node.text is not None or attributes:
                    if path in self.leaves:
                        raise ValueError(f"duplicate selected leaf: {locale}/{path_text(path)}")
                    self.leaves[path] = LdmlValue(node.text, attributes)
                return
            if schema.values(node.tag, node.attrib):
                raise ValueError(f"LDML value attributes on a non-leaf: {locale}/{path_text(path)}")
            for child in node:
                if child.tag == "alias":
                    visit(child, path, node.get("draft", draft))
                    continue
                key = Segment(child.tag, schema.distinguishing(child.tag, child.attrib))
                self.children.setdefault(path, set()).add(key)
                visit(child, (*path, key), node.get("draft", draft))

        visit(document, (), "approved")
        if pattern_alternate is PatternAlternate.ASCII:
            for path in self.leaves:
                if (tuple(part.tag for part in path[:3]) != ("dates", "calendars", "calendar")
                        or path[-1].tag not in ("pattern", "dateFormatItem", "greatestDifference",
                                               "intervalFormatFallback", "appendItem")):
                    continue
                alternates = {part.get("alt") for part in path} - {None}
                if alternates != {pattern_alternate.value}:
                    continue
                default = tuple(Segment(part.tag, tuple((key, value) for key, value in part.attributes
                                                        if key != "alt")) for part in path)
                if default in self.pattern_alternates:
                    raise ValueError(f"ambiguous preferred pattern alternate: {locale}/{path_text(default)}")
                self.pattern_alternates[default] = path
                for length, part in enumerate(default):
                    self.children.setdefault(default[:length], set()).add(part)

    def redirect(self, path):
        for length in range(len(path), -1, -1):
            prefix = path[:length]
            if prefix in self.aliases:
                return (*self.aliases[prefix], *path[length:])
        return None


class CldrProfile:
    def __init__(self, source_directory, *, pattern_alternate=PatternAlternate.DEFAULT):
        self.source_directory = Path(source_directory)
        manifest_bytes = (self.source_directory / "manifest.json").read_bytes()
        manifest = json.loads(manifest_bytes)
        if manifest["release"] != "47.0.0" or manifest["commit"] != CLDR_COMMIT:
            raise ValueError("review the pinned CLDR release before generation")
        self.manifest_bytes = manifest_bytes
        self.documents = {}
        self.sources = {}
        for record in manifest["files"]:
            path = record["path"]
            if path in self.sources or Path(path).is_absolute() or ".." in Path(path).parts:
                raise ValueError(f"invalid or repeated source path: {path}")
            contents = (self.source_directory / path).read_bytes()
            if len(contents) != record["bytes"] or hashlib.sha256(contents).hexdigest() != record["sha256"]:
                raise ValueError(f"pinned source checksum mismatch: {path}")
            self.sources[path] = contents
            if path.endswith(".xml"):
                self.documents[path] = ET.fromstring(contents)
        if sum(map(len, self.sources.values())) != manifest["total_bytes"]:
            raise ValueError("source manifest total size mismatch")
        self.selector = json.loads(self.sources["selector.json"])
        # ElementTree does not apply external DTD defaults. An omitted
        # type="standard" must still match the aliases that spell it out.
        self.schema = LdmlSchema(self.sources["common/dtd/ldml.dtd"].decode())
        self.parents = {}
        supplemental = self.documents["common/supplemental/supplementalData.xml"]
        for group in supplemental.findall("./parentLocales"):
            if group.get("component") is not None:
                continue
            for entry in group.findall("parentLocale"):
                for locale in entry.attrib["locales"].split():
                    if locale in self.parents:
                        raise ValueError(f"duplicate locale parent: {locale}")
                    self.parents[locale] = entry.attrib["parent"]
        metadata = self.documents["common/supplemental/supplementalMetadata.xml"]
        self.default_content = set()
        for entry in metadata.findall("./metadata/defaultContent"):
            self.default_content.update(entry.attrib["locales"].split())
        rank = DRAFT_RANK[self.selector["minimum_draft"]]
        self.locales = {
            Path(path).stem: LocaleTree(Path(path).stem, document, rank, self.schema,
                                      pattern_alternate=pattern_alternate)
            for path, document in self.documents.items() if path.startswith("common/main/")
        }
        self._resolved = {}
        self._children = {}
        self.consumed = {}
        for locale in self.selector["locales"]:
            tuple(self.lineage(locale.replace("-", "_")))

    def lineage(self, locale):
        seen = set()
        while True:
            if locale in seen:
                raise ValueError(f"cyclic locale inheritance: {locale}")
            seen.add(locale)
            if locale not in self.locales:
                raise ValueError(f"required parent locale source is absent: {locale}")
            yield locale
            if locale == "root":
                return
            locale = self.parents.get(locale, locale.rsplit("_", 1)[0] if "_" in locale else "root")

    def resolve(self, locale, path, *, required=True, seen=()):
        path = normalize_path(parse_path(path) if isinstance(path, str) else path, self.schema)
        identity = locale, path
        if identity in seen:
            raise ValueError(f"cyclic LDML alias: {locale}/{path_text(path)}")
        if identity not in self._resolved:
            resolved = None
            for ancestor in self.lineage(locale):
                tree = self.locales[ancestor]
                # Apply the declared alternate within each source locale before
                # inheritance, retaining its full value and provenance.
                value_path = tree.pattern_alternates.get(path, path)
                value = tree.leaves.get(value_path)
                if value is not None and value.text == NO_INHERIT:
                    break
                if value is not None and value.text != INHERIT:
                    if not value.text:
                        raise ValueError(f"empty LDML leaf: {ancestor}/{path_text(path)}")
                    resolved = ResolvedLeaf(value.text, value.attributes, ancestor, value_path)
                    break
                redirected = tree.redirect(path)
                if redirected is not None:
                    resolved = self.resolve(locale, redirected, required=False, seen=(*seen, identity))
                    break
            self._resolved[identity] = resolved
        result = self._resolved[identity]
        if result is None and required:
            raise ValueError(f"unresolved required LDML leaf: {locale}/{path_text(path)}")
        if result is not None:
            self.consumed[f"{locale}/{path_text(path)}"] = {
                "locale": result.source_locale,
                "path": path_text(result.source_path),
                "value": result.value,
                "value_attributes": dict(result.attributes),
            }
        return result

    def text(self, locale, path, *, required=True):
        resolved = self.resolve(locale, path, required=required)
        if resolved is not None and resolved.attributes:
            raise ValueError(f"value attributes require an explicit consumer: {locale}/{path_text(resolved.source_path)}")
        return None if resolved is None else resolved.value

    def children(self, locale, path, seen=()):
        path = normalize_path(parse_path(path) if isinstance(path, str) else path, self.schema)
        identity = locale, path
        if identity in seen:
            raise ValueError(f"cyclic LDML subtree alias: {locale}/{path_text(path)}")
        if identity not in self._children:
            children = set()
            for ancestor in self.lineage(locale):
                tree = self.locales[ancestor]
                children.update(tree.children.get(path, ()))
                redirected = tree.redirect(path)
                if redirected is not None:
                    children.update(self.children(locale, redirected, (*seen, identity)))
                    break
            self._children[identity] = tuple(sorted(children))
        return self._children[identity]

    def leaves(self, locale, path):
        path = parse_path(path) if isinstance(path, str) else path
        for child in self.children(locale, path):
            next_path = (*path, child)
            if self.children(locale, next_path):
                yield from self.leaves(locale, next_path)
            else:
                value = self.resolve(locale, next_path, required=False)
                if value is not None:
                    yield next_path, value
