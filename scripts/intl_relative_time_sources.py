"""Authenticate service-local Polish CLDR47 files to the retained NF archive."""
import hashlib
import json
from pathlib import Path
import tarfile
import xml.etree.ElementTree as ET

from intl_cldr_profile import CLDR_COMMIT, CldrProfile, DRAFT_RANK, LocaleTree

CAPTURE_PATH = "crates/lila-intl/data/relative-time-cldr-47/source-capture.json"
SOURCE_PATH = "crates/lila-intl/data/datetime-cldr-47"


def load_polish_sources(repository):
    capture = json.loads((repository / CAPTURE_PATH).read_bytes())
    if (capture["schema"], capture["release"], capture["commit"], capture["locale_admission"]) != (
            1, "47.0.0", CLDR_COMMIT, ["pl"]):
        raise ValueError("review the exact Polish CLDR47 source capture")
    binding = capture["source_manifest"]
    if binding["path"] != "crates/lila-intl/data/number-cldr-47/source-manifest.json":
        raise ValueError("unreviewed Polish source manifest")
    raw = (repository / binding["path"]).read_bytes()
    if hashlib.sha256(raw).hexdigest() != binding["sha256"]:
        raise ValueError("Polish source manifest checksum mismatch")
    manifest = json.loads(raw)
    binding = capture["archive"]
    if binding["path"] != "crates/lila-intl/data/number-cldr-47/sources.tar.gz":
        raise ValueError("unreviewed Polish source archive")
    archive_path = repository / binding["path"]
    raw = archive_path.read_bytes()
    if (len(raw), hashlib.sha256(raw).hexdigest()) != (binding["bytes"], binding["sha256"]) or (
            binding["bytes"], binding["sha256"]) != (manifest["archive_bytes"], manifest["archive_sha256"]):
        raise ValueError("Polish source archive checksum mismatch")
    rows = capture["files"]
    if len(rows) != 2 or [row["path"] for row in rows] != ["common/main/pl.xml", "common/main/pl_PL.xml"]:
        raise ValueError("Polish capture must preserve both genuine locale sources exactly once")
    primary = {row["path"]: row for row in manifest["files"]}
    if len(primary) != len(manifest["files"]):
        raise ValueError("duplicate primary manifest rows")
    sources = {}
    with tarfile.open(archive_path, "r:gz") as archive:
        for row in rows:
            member = "reference/cldr/" + row["path"]
            authority = primary[member]
            url = f"https://raw.githubusercontent.com/unicode-org/cldr/{CLDR_COMMIT}/" + row["path"]
            if row["archive_member"] != member or row["url"] != authority["url"] or row["url"] != url:
                raise ValueError("unreviewed Polish source provenance")
            if any(row[key] != authority[key] for key in ("bytes", "sha256", "git_blob_sha1")):
                raise ValueError("Polish source row differs from primary manifest")
            payload = (repository / "crates/lila-intl/data/relative-time-cldr-47" / row["path"]).read_bytes()
            blob = hashlib.sha1(b"blob " + str(len(payload)).encode() + b"\0" + payload).hexdigest()
            if (payload != archive.extractfile(member).read() or len(payload) != row["bytes"] or
                    hashlib.sha256(payload).hexdigest() != row["sha256"] or blob != row["git_blob_sha1"]):
                raise ValueError("Polish captured source checksum mismatch")
            sources[row["path"]] = payload
    return capture, sources


def load_profile(repository):
    profile = CldrProfile(repository / SOURCE_PATH)
    capture, sources = load_polish_sources(repository)
    rank = DRAFT_RANK[profile.selector["minimum_draft"]]
    for path, raw in sources.items():
        locale = Path(path).stem
        if locale in profile.locales:
            raise ValueError("Polish source cannot replace an existing shared locale owner")
        profile.locales[locale] = LocaleTree(locale, ET.fromstring(raw), rank, profile.schema)
    if tuple(profile.lineage("pl")) != ("pl", "root") or tuple(profile.lineage("pl_PL")) != ("pl_PL", "pl", "root"):
        raise ValueError("unreviewed Polish locale inheritance")
    profile.relative_source_capture = capture
    return profile
