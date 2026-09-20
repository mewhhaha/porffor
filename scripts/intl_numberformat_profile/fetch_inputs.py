#!/usr/bin/env python3
"""Fetch pinned primary CLDR inputs and verify each Git blob before extraction."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import tarfile
import urllib.request

STAGE = Path(__file__).resolve().parents[1]
COMMIT = "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
TREE_SHA256 = "4d232d06d1208ef0a90a16149ee98035ed9e64b376e17ab7138909d9d88c8183"
SUPPLEMENTAL = {
    "supplementalData.xml", "supplementalMetadata.xml", "numberingSystems.xml",
    "plurals.xml", "pluralRanges.xml", "grammaticalFeatures.xml", "units.xml",
    "likelySubtags.xml",
}
DOCS = {"tr35.md", "tr35-numbers.md", "tr35-general.md", "tr35-info.md"}


def wanted(path):
    return (path.startswith("common/main/") and path.endswith(".xml")) or (
        path.startswith("common/supplemental/") and path.split("/")[-1] in SUPPLEMENTAL
    ) or (path.startswith("docs/ldml/") and path.split("/")[-1] in DOCS) or path in {
        "LICENSE", "common/dtd/ldml.dtd", "common/dtd/ldmlSupplemental.dtd",
        "common/bcp47/number.xml", "common/properties/scriptMetadata.txt",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify local inputs without fetching")
    args = parser.parse_args()
    tree_bytes = (STAGE / "reference/cldr-tree.json").read_bytes()
    assert hashlib.sha256(tree_bytes).hexdigest() == TREE_SHA256
    tree = json.loads(tree_bytes)
    assert tree["sha"] == COMMIT and not tree["truncated"]
    expected = {row["path"]: row for row in tree["tree"] if row["type"] == "blob" and wanted(row["path"])}
    root = STAGE / "reference/cldr"
    missing = [p for p in expected if not (root / p).is_file()]
    archive_receipt = None
    if missing and not args.check:
        url = f"https://codeload.github.com/unicode-org/cldr/tar.gz/{COMMIT}"
        with urllib.request.urlopen(url, timeout=90) as response:
            archive = response.read()
        archive_receipt = {"url": url, "bytes": len(archive), "sha256": hashlib.sha256(archive).hexdigest()}
        with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as packed:
            for member in packed:
                path = member.name.partition("/")[2]
                if path not in expected or not member.isfile():
                    continue
                stream = packed.extractfile(member)
                assert stream is not None
                content = stream.read()
                blob = hashlib.sha1(b"blob " + str(len(content)).encode() + b"\0" + content).hexdigest()
                assert blob == expected[path]["sha"], path
                destination = root / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(content)
    rows = []
    for path, entry in sorted(expected.items()):
        content = (root / path).read_bytes()
        blob = hashlib.sha1(b"blob " + str(len(content)).encode() + b"\0" + content).hexdigest()
        assert blob == entry["sha"] and len(content) == entry["size"], path
        rows.append({"path": f"reference/cldr/{path}", "url": f"https://raw.githubusercontent.com/unicode-org/cldr/{COMMIT}/{path}", "git_blob_sha1": blob, "sha256": hashlib.sha256(content).hexdigest(), "bytes": len(content)})
    receipt = {"upstream": "unicode-org/cldr", "release": "47", "commit": COMMIT, "git_tree_sha256": TREE_SHA256, "files": rows}
    text = json.dumps(receipt, indent=2, ensure_ascii=False) + "\n"
    destination = STAGE / "reference/cldr-input-manifest.json"
    if args.check:
        assert destination.read_text() == text
    else:
        destination.write_text(text)
        if archive_receipt:
            (STAGE / "reference/cldr-archive-receipt.json").write_text(json.dumps(archive_receipt, indent=2) + "\n")
    print(json.dumps({"files": len(rows), "main_files": sum('/common/main/' in r['path'] for r in rows), "bytes": sum(r["bytes"] for r in rows), "manifest_sha256": hashlib.sha256(text.encode()).hexdigest()}))


if __name__ == "__main__":
    main()
