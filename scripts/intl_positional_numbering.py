"""Checked CLDR48/UCD17 tols contribution to the unchanged CLDR47 profiles."""

from copy import deepcopy
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET


CLDR_COMMIT = "acd6d88ae493633240e19a87a721076a8a75c310"
FILES = {
    "cldr/common/supplemental/numberingSystems.xml": "c5b9b208c6fe7bd3ce4e1c6cbb8f3337480aca620c9cf703ef3b6c74d4258c16",
    "cldr/common/bcp47/number.xml": "0c10e9578f2191527475d0f57105d56c399b2f44425ceebadf474e50157abbb6",
    "cldr/common/main/root.xml": "c16e0df1d410c8f481fcb9c4dd195709e08ec9f8a6519d2d9905078e32990f17",
    "ucd/UnicodeData.txt": "2e1efc1dcb59c575eedf5ccae60f95229f706ee6d031835247d843c11d96470c",
}
BRANCHES = frozenset(("symbols", "decimalFormats", "scientificFormats",
                      "percentFormats", "currencyFormats", "miscPatterns"))


class TolsSupplement:
    def __init__(self, directory):
        directory = Path(directory)
        raw = (directory / "source-manifest.json").read_bytes()
        manifest = json.loads(raw)
        if (manifest["schema"] != 1 or manifest["cldr_release"] != "48.0.0"
                or manifest["cldr_commit"] != CLDR_COMMIT
                or manifest["unicode_release"] != "17.0.0"
                or manifest["identifier"] != "tols"):
            raise ValueError("unreviewed numbering supplement recipe")
        records = {row["path"]: row for row in manifest["files"]}
        if len(records) != len(manifest["files"]) or records.keys() != FILES.keys():
            raise ValueError("incomplete numbering supplement source inventory")
        contents = {}
        for path, digest in FILES.items():
            content = (directory / path).read_bytes()
            record = records[path]
            if (record["sha256"] != digest or len(content) != record["bytes"]
                    or hashlib.sha256(content).hexdigest() != digest):
                raise ValueError(f"numbering supplement checksum mismatch: {path}")
            contents[path] = content
        self.documents = {path.removeprefix("cldr/"): ET.fromstring(content)
                          for path, content in contents.items() if path.startswith("cldr/")}
        self.system = self.documents["common/supplemental/numberingSystems.xml"].find(
            "./numberingSystems/numberingSystem[@id='tols']")
        if self.system is None or self.system.attrib.keys() != {"id", "type", "digits"}:
            raise ValueError("missing positional supplement alphabet")
        self.digits = self.system.attrib["digits"]
        if self.system.attrib["type"] != "numeric" or len(self.digits) != 10 or len(set(self.digits)) != 10:
            raise ValueError("invalid positional supplement alphabet")
        unicode_rows = {int(line.split(";", 1)[0], 16): line.split(";")
                        for line in contents["ucd/UnicodeData.txt"].decode().splitlines()}
        for index, digit in enumerate(self.digits):
            fields = unicode_rows[ord(digit)]
            if fields[2] != "Nd" or fields[6:9] != [str(index)] * 3:
                raise ValueError("supplement digits disagree with actual Unicode decimal values")
        self.aliases = [node for node in self.documents["common/main/root.xml"].findall("./numbers/*")
                        if node.get("numberSystem") == "tols"]
        if len(self.aliases) != 6 or {node.tag for node in self.aliases} != BRANCHES:
            raise ValueError("incomplete supplement number-pattern alias closure")
        for node in self.aliases:
            if (node.attrib != {"numberSystem": "tols"} or len(node) != 1
                    or node[0].tag != "alias" or node[0].attrib != {
                        "source": "locale", "path": f"../{node.tag}[@numberSystem='latn']"}):
                raise ValueError("unreviewed supplement source alias")
        self.identity = {
            "identifier": "tols", "cldr_release": "48.0.0", "cldr_commit": CLDR_COMMIT,
            "unicode_release": "17.0.0", "source_manifest_sha256": hashlib.sha256(raw).hexdigest(),
        }

    def apply(self, path, document):
        """Add only source-declared tols rows; never replace a base47 row."""
        if path == "common/main/root.xml":
            numbers = document.find("./numbers")
            if numbers is None or any(node.get("numberSystem") == "tols" for node in numbers):
                raise ValueError("supplement must add an absent root numbering system")
            for node in self.aliases:
                numbers.append(deepcopy(node))
        elif path in ("common/supplemental/numberingSystems.xml", "common/bcp47/number.xml"):
            if "supplemental" in path:
                base = document.find("./numberingSystems")
                source = self.documents[path].find("./numberingSystems")
                key = "id"
            else:
                base = document.find("./keyword/key[@name='nu']")
                source = self.documents[path].find("./keyword/key[@name='nu']")
                key = "name"
            old = {node.attrib[key]: node for node in base}
            new = {node.attrib[key]: node for node in source}
            if (len(old) != len(base) or len(new) != len(source)
                    or new.keys() - old.keys() != {"tols"} or old.keys() - new.keys()
                    or any(old[name].attrib != new[name].attrib for name in old)):
                raise ValueError("numbering supplement changes an existing base47 declaration")
            base.append(deepcopy(new["tols"]))
        return document

    def decimal_ranges(self):
        return [(ord(digit), ord(digit)) for digit in self.digits]
