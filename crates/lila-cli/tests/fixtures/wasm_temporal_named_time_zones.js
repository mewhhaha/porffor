// Temporal with named IANA time zones, through the pinned IANA 2026a data.
//
// Expected values follow the pinned
// `intl402/Temporal/ZonedDateTime/{timezone-case-insensitive,links,
// iana-legacy-names}.js`,
// `ZonedDateTime/prototype/{getTimeZoneTransition/specific-tzdb-values,
// getTimeZoneTransition/subtract-second-and-nanosecond-from-last-transition,
// hoursInDay/same-date-starts-twice,startOfDay/*,since/dst,
// add/dst}.js` and
// `ZonedDateTime/from/argument-string-dst-option-disambiguation.js` cases.

function check(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}

function throwsRangeError(callback, label) {
  try {
    callback();
  } catch (error) {
    if (error instanceof RangeError) return;
    throw label + ": " + error;
  }
  throw label + ": no exception";
}

var out = [];

// Identifiers: case-insensitive lookup, catalogue spelling, links kept.
check(new Temporal.ZonedDateTime(0n, "america/new_york").timeZoneId, "America/New_York", "case");
check(new Temporal.ZonedDateTime(0n, "utc").timeZoneId, "UTC", "utc");
check(new Temporal.ZonedDateTime(0n, "Etc/UTC").timeZoneId, "Etc/UTC", "Etc/UTC link");
check(new Temporal.ZonedDateTime(0n, "Asia/Calcutta").timeZoneId, "Asia/Calcutta", "link");
check(
  new Temporal.ZonedDateTime(0n, "Asia/Calcutta").equals(new Temporal.ZonedDateTime(0n, "Asia/Kolkata")),
  true,
  "link equals primary"
);
check(
  new Temporal.ZonedDateTime(0n, "UTC").equals(new Temporal.ZonedDateTime(0n, "+00:00")),
  false,
  "named is not offset"
);
throwsRangeError(function () {
  new Temporal.ZonedDateTime(0n, "Mars/Olympus_Mons");
}, "unknown");
out.push("identifiers");

// Offsets, including a historical local mean time with seconds.
var ny = new Temporal.ZonedDateTime(0n, "America/New_York");
check(ny.offset, "-05:00", "NY offset");
check(ny.offsetNanoseconds, -18000000000000, "NY offset ns");
check(ny.toString(), "1969-12-31T19:00:00-05:00[America/New_York]", "NY toString");
var paris1800 = new Temporal.PlainDateTime(1800, 1, 1).toZonedDateTime("Europe/Paris");
check(paris1800.toString(), "1800-01-01T00:00:00+00:09[Europe/Paris]", "LMT rounded");
check(paris1800.offset, "+00:09:21", "LMT offset");
check(
  Temporal.Instant.from("2020-01-01T00:00Z").toString({ timeZone: "Europe/Paris" }),
  "2020-01-01T01:00:00+01:00",
  "instant toString zone"
);
out.push("offsets");

// Disambiguation around the 2017 New York transitions.
var repeated = "2017-11-05T01:30[America/New_York]";
check(Temporal.ZonedDateTime.from(repeated).offset, "-04:00", "repeated compatible");
check(
  Temporal.ZonedDateTime.from(repeated, { disambiguation: "later" }).offset,
  "-05:00",
  "repeated later"
);
var skipped = "2017-03-12T02:30[America/New_York]";
check(
  Temporal.ZonedDateTime.from(skipped).toString(),
  "2017-03-12T03:30:00-04:00[America/New_York]",
  "skipped compatible"
);
check(
  Temporal.ZonedDateTime.from(skipped, { disambiguation: "earlier" }).toString(),
  "2017-03-12T01:30:00-05:00[America/New_York]",
  "skipped earlier"
);
throwsRangeError(function () {
  Temporal.ZonedDateTime.from(skipped, { disambiguation: "reject" });
}, "skipped reject");
throwsRangeError(function () {
  Temporal.ZonedDateTime.from("2017-11-05T01:30-06:00[America/New_York]");
}, "offset mismatch");
check(
  Temporal.ZonedDateTime.from("2017-11-05T01:30-05:00[America/New_York]").offset,
  "-05:00",
  "offset selects the later candidate"
);
out.push("disambiguation");

// Transitions.
var first = paris1800.getTimeZoneTransition("next");
check(first.toString(), "1911-03-10T23:50:39+00:00[Europe/Paris]", "first Paris transition");
check(
  first.add({ nanoseconds: -1 }).getTimeZoneTransition("next").toString(),
  first.toString(),
  "strictly after"
);
check(
  Temporal.ZonedDateTime.from(repeated).getTimeZoneTransition("next").toString(),
  "2017-11-05T01:00:00-05:00[America/New_York]",
  "NY next"
);
check(new Temporal.ZonedDateTime(0n, "+05:30").getTimeZoneTransition("next"), null, "offset zone");
check(new Temporal.ZonedDateTime(0n, "UTC").getTimeZoneTransition("previous"), null, "UTC");
out.push("transitions");

// Day lengths and starts.
check(Temporal.ZonedDateTime.from(repeated).hoursInDay, 25, "fall back day");
check(Temporal.ZonedDateTime.from("2017-03-12T12:00[America/New_York]").hoursInDay, 23, "spring day");
check(
  Temporal.ZonedDateTime.from("2010-11-07T23:00:00-03:30[America/St_Johns]").hoursInDay,
  25,
  "St Johns"
);
check(
  Temporal.ZonedDateTime.from("2010-03-05T00:45:00+11:00[Antarctica/Casey]").hoursInDay,
  27,
  "Casey"
);
check(
  Temporal.PlainDate.from("2018-11-04").toZonedDateTime("America/Sao_Paulo").toString(),
  "2018-11-04T01:00:00-02:00[America/Sao_Paulo]",
  "skipped midnight"
);
out.push("days");

// Zoned arithmetic: calendar days versus exact hours.
var before = Temporal.ZonedDateTime.from(repeated);
check(before.add({ days: 1 }).toString(), "2017-11-06T01:30:00-05:00[America/New_York]", "add days");
check(before.add({ hours: 24 }).toString(), "2017-11-06T00:30:00-05:00[America/New_York]", "add hours");
var from = Temporal.ZonedDateTime.from("2000-04-02T01:30:00-08:00[America/Vancouver]");
var to = Temporal.ZonedDateTime.from("2000-04-02T04:30:00-07:00[America/Vancouver]");
check(to.since(from, { largestUnit: "days" }).toString(), "PT2H", "same-day since");
check(
  Temporal.ZonedDateTime.from("2000-04-03T02:00:00-07:00[America/Vancouver]")
    .since(Temporal.ZonedDateTime.from("2000-04-02T01:00:00-08:00[America/Vancouver]"), { largestUnit: "days" })
    .toString(),
  "P1DT1H",
  "since across the transition"
);
check(
  Temporal.Duration.from({ hours: 24 }).total({
    unit: "days",
    relativeTo: Temporal.ZonedDateTime.from("2017-11-05T00:00[America/New_York]"),
  }),
  24 / 25,
  "total in a 25-hour day"
);
out.push("arithmetic");

print("temporal-named-time-zones:" + out.join("|"));

262;
