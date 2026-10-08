"""Turn resolved CLDR name leaves into closed date/time field records."""

from intl_cldr_profile import parse_path


WIDTHS = {"abbreviated", "wide", "narrow", "short"}
CONTEXTS = {"format", "stand-alone"}
DAYS = {day: index for index, day in enumerate(("sun", "mon", "tue", "wed", "thu", "fri", "sat"))}
PERIODS = {"am", "pm", "midnight", "noon", "morning1", "morning2", "afternoon1",
           "afternoon2", "evening1", "evening2", "night1", "night2"}


def field_names(leaves, *, calendar=None):
    records = []
    keys = set()
    for source, value in leaves:
        path = parse_path(source)
        tags = tuple(segment.tag for segment in path)
        context, width, index, period = None, None, None, None
        year_type = None
        if tags == ("eras", "eraAbbr", "era"):
            kind, width = "era", "abbreviated"
            index = int(path[-1].get("type"))
        elif tags == ("eras", "eraNames", "era"):
            kind, width = "era", "wide"
            index = int(path[-1].get("type"))
        elif tags == ("eras", "eraNarrow", "era"):
            kind, width = "era", "narrow"
            index = int(path[-1].get("type"))
        elif tags in (("months", "monthContext", "monthWidth", "month"),
                       ("days", "dayContext", "dayWidth", "day"),
                       ("dayPeriods", "dayPeriodContext", "dayPeriodWidth", "dayPeriod")):
            kind = {"months": "month", "days": "weekday", "dayPeriods": "day_period"}[tags[0]]
            context, width = path[1].get("type"), path[2].get("type")
            if kind == "weekday":
                index = DAYS[path[-1].get("type")]
            elif kind == "month":
                index = int(path[-1].get("type"))
                year_type = path[-1].get("yeartype")
                if year_type is not None and not (calendar == "hebrew" and index == 7 and year_type == "leap"):
                    raise ValueError(f"unreviewed month year-type: {source}")
            else:
                period = path[-1].get("type")
                if period not in PERIODS:
                    raise ValueError(f"unknown date/time day period: {source}")
        elif tags == ("monthPatterns", "monthPatternContext", "monthPatternWidth", "monthPattern"):
            if path[-1].get("type") != "leap":
                raise ValueError(f"unsupported month pattern: {source}")
            kind, context, width = "leap_month", path[1].get("type"), path[2].get("type")
            if context == "numeric" and width == "all":
                context, width = None, None
            if value.count("{0}") != 1 or "{" in value.replace("{0}", "") or "}" in value.replace("{0}", ""):
                raise ValueError(f"invalid leap-month placeholder: {source}")
        elif tags == ("cyclicNameSets", "cyclicNameSet", "cyclicNameContext", "cyclicNameWidth", "cyclicName"):
            if path[1].get("type") != "years":
                raise ValueError(f"unselected cyclic name domain: {source}")
            kind, context, width = "cyclic_year", path[2].get("type"), path[3].get("type")
            index = int(path[-1].get("type"))
        else:
            raise ValueError(f"unknown name path in selected profile: {source}")
        if (context is not None and context not in CONTEXTS) or (width is not None and width not in WIDTHS):
            raise ValueError(f"unknown name context or width: {source}")
        if index is not None:
            era_max = 236 if calendar == "japanese" else 1
            month_max = 13 if calendar in ("coptic", "ethiopic", "ethiopic-amete-alem", "hebrew") else 12
            bounds = {"era": (0, era_max), "month": (1, month_max), "weekday": (0, 6), "cyclic_year": (1, 60)}[kind]
            if not bounds[0] <= index <= bounds[1]:
                raise ValueError(f"name index outside selected calendar: {source}")
        if kind == "era" and calendar == "japanese" and index < 232:
            # The projection uses Gregorian eras before1873, then modern five.
            continue
        key = kind, context, width, index, period, year_type
        if key in keys:
            raise ValueError(f"duplicate typed name key: {source}")
        keys.add(key)
        record = dict(kind=kind, context=context, width=width, index=index, period=period, value=value)
        if year_type is not None:
            record["year_type"] = year_type
        records.append(record)
    return records
