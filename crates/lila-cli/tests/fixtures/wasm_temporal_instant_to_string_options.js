// Temporal.Instant.prototype.toString options and toZonedDateTimeISO.
//
// Expected values are copied from the pinned
// `Instant/prototype/toString/{fractionalseconddigits-number,
// smallestunit-valid-units,timezone-string,timezone-string-datetime,
// timezone-string-multiple-offsets,timezone-string-unknown,
// negative-instant-rounding}.js` and
// `Instant/prototype/toZonedDateTimeISO/{timezone-string,
// timezone-string-unknown}.js` cases.

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
var instant = new Temporal.Instant(1_000_000_000_123_456_789n);
check(instant.toString(), "2001-09-09T01:46:40.123456789Z", "auto");
check(instant.toJSON(), "2001-09-09T01:46:40.123456789Z", "toJSON");
check(instant.toString({ fractionalSecondDigits: 2 }), "2001-09-09T01:46:40.12Z", "digits");
check(instant.toString({ smallestUnit: "minute" }), "2001-09-09T01:46Z", "minute");
check(
  instant.toString({ smallestUnit: "microsecond", roundingMode: "ceil" }),
  "2001-09-09T01:46:40.123457Z",
  "ceil"
);
throwsRangeError(function () {
  instant.toString({ smallestUnit: "hour" });
}, "hour");
out.push("precision");

var epoch = new Temporal.Instant(0n);
check(epoch.toString({ timeZone: "UTC" }).slice(-6), "+00:00", "UTC");
check(epoch.toString({ timeZone: "-01:30" }).slice(-6), "-01:30", "offset");
check(epoch.toString({ timeZone: "2021-08-19T17:30Z" }).slice(-6), "+00:00", "Z string");
check(
  epoch.toString({ timeZone: "2021-08-19T17:30:45.123456789-12:12[+01:46]" }).slice(-6),
  "+01:46",
  "bracket wins"
);
throwsRangeError(function () {
  epoch.toString({ timeZone: "2021-08-19T17:30" });
}, "bare date-time");
throwsRangeError(function () {
  epoch.toString({ timeZone: "Mars/Olympus_Mons" });
}, "unknown zone");
throwsRangeError(function () {
  epoch.toString({ timeZone: "1970-01-01T00:00+01:00[America/Nonexistent]" });
}, "unknown annotation");
out.push("timeZone");

var zoned = epoch.toZonedDateTimeISO("-01:30");
check(zoned.timeZoneId, "-01:30", "zoned zone");
check(zoned.calendarId, "iso8601", "zoned calendar");
check(zoned.epochNanoseconds, 0n, "zoned epoch");
throwsRangeError(function () {
  epoch.toZonedDateTimeISO("Mars/Olympus_Mons");
}, "zoned unknown");
out.push("toZonedDateTimeISO");

print("temporal-instant-to-string:" + out.join("|"));

262;
