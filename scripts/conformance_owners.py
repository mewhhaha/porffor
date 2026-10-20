#!/usr/bin/env python3
"""Read and validate the compiler's closed conformance ownership registry."""

import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = Path("crates/scriptw-ir/src/ownership.rs")
DOCUMENT = Path("docs/rust-rewrite/conformance-ownership.md")
ACCEPTANCE = Path("docs/rust-rewrite/conformance-acceptance.md")


def owners(source):
    declaration = "define_conformance_owners! {"
    if source.count(declaration) != 1:
        raise ValueError("ownership registry must have one declaration")
    body = source.split(declaration, 1)[1].split("}", 1)[0]
    literal = r'"(?:[^"\\]|\\.)*"'
    pattern = re.compile(
        rf'\s*(T[0-9]{{2}})\s*=\s*([0-9]+)\s*=>\s*({literal})\s*=>\s*({literal})\s*,?'
    )
    rows = []
    position = 0
    while position < len(body) and body[position:].strip():
        match = pattern.match(body, position)
        if match is None:
            raise ValueError("unrecognized ownership registry record")
        variant, number, code, area = match.groups()
        rows.append((variant, int(number), json.loads(code), json.loads(area)))
        position = match.end()
    expected = [(f"T{number:02}", number, f"T{number:02}") for number in range(30)]
    if [row[:3] for row in rows] != expected:
        raise ValueError("ownership registry must contain exactly ordered T00 through T29")
    if any(not area.strip() or area != area.strip() for _, _, _, area in rows):
        raise ValueError("ownership areas must have unpadded descriptive names")
    return {code: area for _, _, code, area in rows}


def check_document(registry, document):
    headings = re.findall(r"^## (T[0-9]{2}) — (.+)$", document, re.MULTILINE)
    if headings != list(registry.items()):
        raise ValueError("durable ownership headings differ from the compiler registry")
    if "T26-unclassified" not in document:
        raise ValueError("durable ownership must describe the separate unclassified bucket")


def markdown_anchors(document):
    anchors = set(re.findall(r'<a id="([^"]+)"></a>', document))
    duplicates = {}
    for heading in re.findall(r"^#{1,6} (.+)$", document, re.MULTILINE):
        heading = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", heading)
        base = re.sub(r"[^\w -]", "", heading.lower()).replace(" ", "-")
        count = duplicates.get(base, 0)
        duplicates[base] = count + 1
        anchors.add(base if count == 0 else f"{base}-{count}")
    return anchors


def check_links(root, path, document):
    for target in re.findall(r"\[[^\]]+\]\(([^)]+)\)", document):
        target = target.strip("<>")
        if re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*:", target):
            continue
        name, _, anchor = target.partition("#")
        linked = (path.parent / name).resolve() if name else path.resolve()
        if not linked.is_relative_to(root.resolve()) or not linked.exists():
            raise ValueError(f"{path.relative_to(root)}: broken local link {target!r}")
        if anchor and (not linked.is_file() or anchor not in markdown_anchors(linked.read_text())):
            raise ValueError(f"{path.relative_to(root)}: missing anchor {target!r}")


def check_documents(root, registry):
    ownership = (root / DOCUMENT).read_text()
    check_document(registry, ownership)
    acceptance = (root / ACCEPTANCE).read_text()
    headings = re.findall(r"^## (T[0-9]{2}) — (.+)$", acceptance, re.MULTILINE)
    if headings != list(registry.items()):
        raise ValueError("acceptance headings differ from the compiler registry")
    for code in registry:
        anchor = code.lower()
        for name, document in ((DOCUMENT, ownership), (ACCEPTANCE, acceptance)):
            if document.count(f'<a id="{anchor}"></a>') != 1:
                raise ValueError(f"{name}: expected one explicit {anchor} anchor")
        if f"(conformance-acceptance.md#{anchor})" not in ownership:
            raise ValueError(f"ownership area {code} has no acceptance link")
    for name, document in ((DOCUMENT, ownership), (ACCEPTANCE, acceptance)):
        check_links(root, root / name, document)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ids", action="store_true", help="print concrete owner codes")
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        registry = owners((args.root / REGISTRY).read_text())
        check_documents(args.root, registry)
    except (OSError, ValueError) as error:
        parser.exit(1, f"conformance-ownership: {error}\n")
    print(" ".join(registry) if args.ids else "conformance-ownership: ok (30 permanent areas)")


if __name__ == "__main__":
    main()
