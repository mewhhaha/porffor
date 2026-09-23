"""Localize the existing pinned CLDR zone geography, never its transitions."""

import importlib.util
from collections import Counter
from pathlib import Path
import xml.etree.ElementTree as ET

from intl_cldr_profile import parse_path


def canonical_zone_geography(repository):
    path = Path(__file__).with_name("generate-intl-time-zone-names.py")
    spec = importlib.util.spec_from_file_location("lila_pinned_zone_names", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    manifest, sources, source = module.extract(repository / module.SOURCE_PATH)
    aliases = dict(source["aliases"])
    counts = Counter(zone["territory"] for zone in source["zones"] if zone["territory"] != "001")
    primary = {node.attrib["iso3166"]: aliases[node.text]
               for node in ET.fromstring(sources["common/supplemental/metaZones.xml"])
               .findall("./primaryZones/primaryZone")}
    zones = []
    for zone in source["zones"]:
        zones.append({"identifier": zone["identifier"], "territory": zone["territory"],
                      "periods": zone["periods"],
                      "location_is_country": zone["territory"] != "001" and
                      (counts[zone["territory"]] == 1 or primary.get(zone["territory"]) == zone["identifier"])})
    metazones = [{"identifier": row["identifier"], "preferred": row["preferred"]}
                for row in source["metazones"]]
    return manifest, {"zones": zones, "aliases": source["aliases"], "metazones": metazones}


def localized_zone_names(leaves, countries, geography):
    patterns, names, cities = {}, {}, {}
    for source, value in leaves:
        path = parse_path(source)
        if len(path) == 1:
            key = path[0].tag
            if key == "regionFormat":
                key += ":" + path[0].get("type", "generic")
            if key in patterns:
                raise ValueError(f"duplicate localized zone pattern: {source}")
            patterns[key] = value
        elif len(path) == 2 and path[0].tag == "zone" and path[1].tag == "exemplarCity":
            cities[path[0].get("type")] = value
        elif (len(path) == 3 and path[0].tag in ("zone", "metazone")
              and path[1].tag in ("short", "long") and path[2].tag in ("generic", "standard", "daylight")):
            names[(path[0].tag, path[0].get("type"), path[1].tag, path[2].tag)] = value
        else:
            raise ValueError(f"unsupported localized zone name: {source}")
    required = {"hourFormat", "gmtFormat", "gmtZeroFormat", "fallbackFormat",
                "regionFormat:generic", "regionFormat:standard", "regionFormat:daylight"}
    if set(patterns) != required or patterns["hourFormat"] != "+HH:mm;-HH:mm":
        raise ValueError("required localized zone pattern domain differs from the renderer")
    for key, value in patterns.items():
        slots = ["{0}", "{1}"] if key == "fallbackFormat" else ["{0}"] if key == "gmtFormat" or key.startswith("regionFormat:") else []
        remainder = value
        for slot in slots:
            if remainder.count(slot) != 1:
                raise ValueError(f"invalid localized zone placeholder: {key}")
            remainder = remainder.replace(slot, "")
        if "{" in remainder or "}" in remainder:
            raise ValueError(f"unexpected localized zone placeholder: {key}")
    country_names = {(identifier, width): value for identifier, width, value in countries}

    def forms(kind, identifier):
        return {width: {name: names.get((kind, identifier, width, name))
                        for name in ("generic", "standard", "daylight")}
                for width in ("short", "long")}

    zones = []
    for zone in geography["zones"]:
        identifier, territory = zone["identifier"], zone["territory"]
        city = cities.get(identifier, identifier.rsplit("/", 1)[-1].replace("_", " "))
        country = None
        location = None
        if territory != "001":
            country = country_names.get((territory, "long"))
            if country is None:
                raise ValueError(f"required localized country name is missing: {territory}")
            location = country_names.get((territory, "short"), country) if zone["location_is_country"] else city
        zones.append({"identifier": identifier, "city": city, "country": country,
                      "location": location, "names": forms("zone", identifier)})
    metas = [{"identifier": row["identifier"], "names": forms("metazone", row["identifier"])}
             for row in geography["metazones"]]
    return {"patterns": patterns, "zones": zones, "metazones": metas}
