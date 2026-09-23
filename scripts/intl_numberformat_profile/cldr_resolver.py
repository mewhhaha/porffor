"""Pinned NumberFormat LDML inheritance with leaf-level provenance."""
from dataclasses import dataclass
from functools import lru_cache
from itertools import product
from pathlib import Path
import hashlib
import json
import re
import xml.etree.ElementTree as ET

from ldml_schema import LdmlSchema

INHERIT = "↑↑↑"
NO_INHERIT = "∅∅∅"
ATTRIBUTE = re.compile(r"\[@([A-Za-z][A-Za-z0-9_-]*)=(?:'([^']*)'|\"([^\"]*)\")\]")


@dataclass(frozen=True, order=True)
class Segment:
    tag: str
    attributes: tuple = ()

    def get(self, name, default=None):
        return dict(self.attributes).get(name, default)

    def __str__(self):
        return self.tag + "".join(f"[@{key}='{value}']" for key, value in self.attributes)


@lru_cache(maxsize=None)
def segment(tag, attributes=()):
    return Segment(tag, attributes)


def path_text(path):
    return "/".join(map(str, path))


def parse_path(source, base=()):
    components = []
    start = 0
    quote = None
    for index, character in enumerate(source):
        if quote is not None:
            if character == quote:
                quote = None
        elif character in "'\"":
            quote = character
        elif character == "/":
            components.append(source[start:index])
            start = index + 1
    if quote is not None:
        raise ValueError(f"unterminated alias quote: {source}")
    components.append(source[start:])
    result = list(base)
    for component in components:
        if component in ("", "."):
            continue
        if component == "..":
            if not result:
                raise ValueError(f"alias escapes root: {source}")
            result.pop()
            continue
        name = component.split("[", 1)[0]
        if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_-]*", name):
            raise ValueError(f"unknown path syntax: {component}")
        predicates = ATTRIBUTE.findall(component[len(name):])
        if ATTRIBUTE.sub("", component[len(name):]):
            raise ValueError(f"unknown predicate syntax: {component}")
        attributes = tuple(sorted((name, a or b) for name, a, b in predicates))
        if len(dict(attributes)) != len(attributes):
            raise ValueError(f"repeated predicate: {component}")
        result.append(segment(name, attributes))
    return tuple(result)


@dataclass(frozen=True)
class Leaf:
    text: str
    attributes: tuple
    locale: str
    path: tuple
    draft: str


class LocaleTree:
    def __init__(self, locale, root, schema):
        self.leaves = {}
        self.aliases = {}
        self.children = {}

        def visit(node, path, draft):
            if node.tag == "special":
                return
            draft = node.get("draft", draft)
            if draft not in {"approved", "contributed", "provisional", "unconfirmed"}:
                raise ValueError(f"unknown draft status: {locale}/{path_text(path)}")
            if node.tag == "alias":
                if node.get("source") != "locale":
                    raise ValueError(f"unsupported alias source: {node.attrib}")
                if path in self.aliases:
                    raise ValueError(f"duplicate alias: {locale}/{path_text(path)}")
                target = parse_path(node.get("path", "."), path)
                self.aliases[path] = tuple(segment(s.tag, schema.normalize_predicates(s.tag, s.attributes)) for s in target)
                return
            # Proposed alternatives are not stable CLDR values. Semantic alt
            # keys such as narrow and alphaNextToNumber remain distinct leaves.
            if "proposed" in node.get("alt", "").split("-"):
                return
            if not list(node):
                attributes = schema.values(node.tag, node.attrib)
                if node.text is None and not attributes:
                    return
                if path in self.leaves:
                    raise ValueError(f"duplicate leaf: {locale}/{path_text(path)}")
                self.leaves[path] = Leaf(node.text, attributes, locale, path, draft)
                return
            if schema.values(node.tag, node.attrib):
                raise ValueError(f"unconsumed container value attributes: {locale}/{path_text(path)}")
            for child in node:
                if child.tag == "alias":
                    visit(child, path, draft)
                else:
                    key = segment(child.tag, schema.distinguishing(child.tag, child.attrib))
                    self.children.setdefault(path, set()).add(key)
                    visit(child, (*path, key), draft)

        for child in root:
            if child.tag in {"numbers", "units"}:
                visit(child, (segment(child.tag),), "approved")

    def redirect(self, path):
        for length in range(len(path), -1, -1):
            prefix = path[:length]
            if prefix in self.aliases:
                return (*self.aliases[prefix], *path[length:])
        return None


class CldrResolver:
    def __init__(self, stage):
        self.root = Path(stage) / "reference/cldr"
        manifest = json.loads((Path(stage) / "reference/cldr-input-manifest.json").read_text())
        if manifest["commit"] != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c":
            raise ValueError("review a changed CLDR release before generation")
        self.sources = {}
        for record in manifest["files"]:
            relative = record["path"].removeprefix("reference/cldr/")
            if relative in self.sources or ".." in Path(relative).parts:
                raise ValueError(f"invalid manifest path: {relative}")
            self.sources[relative] = record
        self.schema = LdmlSchema(self.read("common/dtd/ldml.dtd").decode())
        self.parents = {}
        for group in self.xml("common/supplemental/supplementalData.xml").findall("parentLocales"):
            if group.get("component") is not None:
                continue
            for node in group.findall("parentLocale"):
                for locale in node.attrib["locales"].split():
                    if locale in self.parents:
                        raise ValueError(f"duplicate parent: {locale}")
                    self.parents[locale] = node.attrib["parent"]
        metadata = self.xml("common/supplemental/supplementalMetadata.xml")
        self.default_content = set()
        for entry in metadata.findall("./metadata/defaultContent"):
            self.default_content.update(entry.attrib["locales"].split())
        self.locales = {Path(p).stem for p in self.sources if p.startswith("common/main/")}
        self.language_aliases = {n.get("type"): n.get("replacement").split()[0] for n in metadata.findall(".//languageAlias")}
        self.script_aliases = {n.get("type"): n.get("replacement").split()[0] for n in metadata.findall(".//scriptAlias")}
        self.territory_aliases = {n.get("type"): n.get("replacement").split()[0] for n in metadata.findall(".//territoryAlias")}
        self.variant_aliases = {n.get("type"): n.get("replacement").split()[0] for n in metadata.findall(".//variantAlias")}
        self.consumed = {}
        self.cache = {}
        self.child_cache = {}
        self.normalized_paths = {}

    def read(self, path):
        record = self.sources[path]
        content = (self.root / path).read_bytes()
        if len(content) != record["bytes"] or hashlib.sha256(content).hexdigest() != record["sha256"]:
            raise ValueError(f"pinned source mismatch: {path}")
        return content

    def xml(self, path):
        return ET.fromstring(self.read(path))

    @lru_cache(maxsize=40)
    def tree(self, locale):
        return LocaleTree(locale, self.xml(f"common/main/{locale}.xml"), self.schema)

    def lineage(self, locale):
        seen = set()
        while True:
            if locale in seen:
                raise ValueError(f"cyclic parent chain: {locale}")
            seen.add(locale)
            if locale not in self.locales and locale not in self.default_content:
                raise ValueError(f"missing required parent source: {locale}")
            if locale in self.locales:
                yield locale
            if locale == "root":
                return
            locale = self.parents.get(locale, locale.rsplit("_", 1)[0] if "_" in locale else "root")

    def canonical_locale(self, locale):
        source = locale
        seen = set()
        while locale in self.language_aliases:
            if locale in seen:
                raise ValueError(f"cyclic language alias: {source}")
            seen.add(locale)
            locale = self.language_aliases[locale]
        parts = locale.split("_")
        language = parts.pop(0)
        replacement = self.language_aliases.get(language, language).split("_")
        language = replacement.pop(0)
        script = next((x for x in parts if len(x) == 4 and x.isalpha()), None)
        region = next((x for x in parts if len(x) == 2 or len(x) == 3 and x.isdigit()), None)
        variants = [x for x in parts if x != script and x != region]
        for value in replacement:
            if len(value) == 4 and value.isalpha() and script is None:
                script = value
            elif (len(value) == 2 or value.isdigit()) and region is None:
                region = value
            elif value not in variants:
                variants.append(value)
        if script:
            script = self.script_aliases.get(script, script).title()
        if region:
            region = self.territory_aliases.get(region, region).upper()
        variants = sorted(self.variant_aliases.get(x, x).lower() for x in variants)
        return "-".join([language.lower(), *([script] if script else []), *([region] if region else []), *variants])

    def normalize(self, path):
        if isinstance(path, str):
            if path not in self.normalized_paths:
                self.normalized_paths[path] = tuple(segment(s.tag, self.schema.normalize_predicates(s.tag, s.attributes)) for s in parse_path(path))
            return self.normalized_paths[path]
        return tuple(segment(s.tag, self.schema.normalize_predicates(s.tag, s.attributes)) for s in path)

    @lru_cache(maxsize=None)
    def lateral_paths(self, path, *, plural_category=None):
        # Pinned LDML47 lateral order: alt (innermost), case, gender, count.
        # NumberFormat requests no gender variants. Compound denominators can
        # request a grammatical case; numeric count callers supply its category.
        dimensions = []
        for attribute in ("count", "gender", "case", "alt"):
            for index, part in enumerate(path):
                value = part.get(attribute)
                if value is None:
                    continue
                variants = [value]
                if attribute == "count":
                    if value.isdigit():
                        if plural_category is None:
                            raise ValueError("explicit count requires a cardinal fallback")
                        variants.append(plural_category)
                    if "other" not in variants:
                        variants.append("other")
                elif attribute == "case" and value != "nominative":
                    variants.append("nominative")
                elif attribute == "gender":
                    raise ValueError("NumberFormat must resolve component gender explicitly")
                variants.append(None)
                dimensions.append((index, attribute, variants))
        result = []
        for values in product(*(row[2] for row in dimensions)):
            candidate = list(path)
            for (index, attribute, _), value in zip(dimensions, values):
                attributes = dict(candidate[index].attributes)
                if value is None:
                    attributes.pop(attribute, None)
                else:
                    attributes[attribute] = value
                candidate[index] = segment(candidate[index].tag, tuple(sorted(attributes.items())))
            result.append(tuple(candidate))
        return tuple(result)

    def resolve(self, locale, path, *, required=True, lateral=True, plural_category=None, seen=()):
        path = self.normalize(path)
        identity = (locale, path, lateral, plural_category)
        if identity in seen:
            raise ValueError(f"cyclic LDML alias: {locale}/{path_text(path)}")
        if identity not in self.cache:
            result = None
            stopped = False
            candidates = tuple(self.lateral_paths(path, plural_category=plural_category)) if lateral else (path,)
            for ancestor in self.lineage(locale):
                tree = self.tree(ancestor)
                for candidate in candidates:
                    value = tree.leaves.get(candidate)
                    if value is not None:
                        if value.text == NO_INHERIT:
                            stopped = True
                            break
                        if value.text != INHERIT:
                            if value.text is None:
                                raise ValueError(f"value-only leaf needs an explicit consumer: {ancestor}/{path_text(candidate)}")
                            result = value
                            break
                    redirected = tree.redirect(candidate)
                    if redirected is not None:
                        result = self.resolve(locale, redirected, required=False, lateral=lateral,
                                              plural_category=plural_category, seen=(*seen, identity))
                        stopped = True
                        break
                if result is not None or stopped:
                    break
            self.cache[identity] = result
        result = self.cache[identity]
        if result is None:
            if required:
                raise ValueError(f"unresolved leaf: {locale}/{path_text(path)}")
            return None
        self.consumed[(locale, path_text(path), lateral, plural_category)] = result
        return result

    def resolve_local(self, locale, path, *, lateral=True):
        path = self.normalize(path)
        candidates = self.lateral_paths(path) if lateral else (path,)
        tree = self.tree(locale)
        for candidate in candidates:
            value = tree.leaves.get(candidate)
            if value is not None:
                if value.text == NO_INHERIT:
                    return None
                if value.text not in (None, INHERIT):
                    self.consumed[(locale, path_text(path), "local", lateral)] = value
                    return value
            redirected = tree.redirect(candidate)
            if redirected is not None:
                return self.resolve(locale, redirected, required=False, lateral=lateral)
        return None

    def text(self, locale, path, **kwargs):
        leaf = self.resolve(locale, path, **kwargs)
        if leaf is not None and leaf.attributes:
            raise ValueError(f"unconsumed value attributes: {leaf.locale}/{path_text(leaf.path)} {leaf.attributes}")
        return None if leaf is None else leaf.text

    def children(self, locale, path, seen=()):
        path = self.normalize(path)
        identity = locale, path
        if identity in seen:
            raise ValueError(f"cyclic subtree alias: {locale}/{path_text(path)}")
        if identity not in self.child_cache:
            result = set()
            for ancestor in self.lineage(locale):
                tree = self.tree(ancestor)
                result.update(tree.children.get(path, ()))
                redirected = tree.redirect(path)
                if redirected is not None:
                    result.update(self.children(locale, redirected, (*seen, identity)))
                    break
            self.child_cache[identity] = tuple(sorted(result))
        return self.child_cache[identity]

    def clear_locale_cache(self):
        self.consumed.clear()
        self.cache.clear()
        self.child_cache.clear()
