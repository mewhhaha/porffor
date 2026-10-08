"""Checked CLDR48 era-only overlay for the expanded private calendar recipe."""

from copy import deepcopy
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET


CLDR_COMMIT = "acd6d88ae493633240e19a87a721076a8a75c310"
CALENDARS = ("coptic", "ethiopic", "ethiopic-amete-alem", "islamic",
             "islamic-civil", "islamic-tbla", "islamic-umalqura")
LANGUAGES = ("root", "en", "ar", "zh", "de", "fr", "it", "ja", "ko", "hi")


class CalendarEraSupplement:
    def __init__(self, directory):
        raw = (directory / "manifest.json").read_bytes()
        manifest = json.loads(raw)
        if (manifest["schema_version"] != 1 or manifest["release"] != "48.0.0"
                or manifest["commit"] != CLDR_COMMIT
                or manifest["allowed_calendar_era_subtrees"] != list(CALENDARS)):
            raise ValueError("unreviewed calendar-era supplement")
        expected = {"LICENSE", *(f"common/main/{locale}.xml" for locale in LANGUAGES)}
        self.documents = {}
        self.sources = {}
        self.applied = {}
        for row in manifest["files"]:
            name = row["path"]
            if name not in expected or name in self.sources:
                raise ValueError("unreviewed calendar-era source file")
            data = (directory / name).read_bytes()
            if (len(data) != row["bytes"]
                    or hashlib.sha256(data).hexdigest() != row["sha256"]
                    or hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() != row["git_blob_sha1"]):
                raise ValueError(f"calendar-era source checksum mismatch: {name}")
            self.sources[name] = data
            if name.endswith(".xml"):
                self.documents[name] = ET.fromstring(data)
        if set(self.sources) != expected or sum(map(len, self.sources.values())) != manifest["total_bytes"]:
            raise ValueError("incomplete calendar-era source closure")
        self.identity = {"identifier": "selected-calendar-eras", "cldr_release": "48.0.0",
                         "cldr_commit": CLDR_COMMIT,
                         "source_manifest_sha256": hashlib.sha256(raw).hexdigest()}

    def apply(self, name, document):
        donor = self.documents.get(name)
        if donor is None:
            return
        # The complete original locale XML remains pinned. Replace only the
        # reviewed era subtrees; patterns, months and old4 names stay CLDR47.
        for kind in CALENDARS:
            path = f"./dates/calendars/calendar[@type='{kind}']"
            source = donor.find(path)
            target = document.find(path)
            if source is None or source.find("eras") is None:
                # CLDR aliases/inheritance supply the same explicit closure.
                continue
            if target is None:
                calendars = document.find("./dates/calendars")
                if calendars is None:
                    dates = document.find("dates")
                    if dates is None:
                        dates = ET.SubElement(document, "dates")
                    calendars = ET.SubElement(dates, "calendars")
                target = ET.SubElement(calendars, "calendar", {"type": kind})
            eras = target.find("eras")
            if eras is not None:
                target.remove(eras)
            replacement = deepcopy(source.find("eras"))
            target.append(replacement)
            self.applied[(Path(name).stem, kind)] = hashlib.sha256(ET.tostring(replacement)).hexdigest()

    def provenance(self, locale, path):
        if (len(path) >= 4 and tuple(segment.tag for segment in path[:4]) == ("dates", "calendars", "calendar", "eras")
                and (locale, path[2].get("type")) in self.applied):
            return {"source_cldr_commit": CLDR_COMMIT,
                    "source_manifest_sha256": self.identity["source_manifest_sha256"]}
        return {}
