// Temporal.Duration.prototype.{round,total} and Temporal.Duration.compare with
// a relativeTo.
//
// Every expected value is copied from a pinned Test262 case:
// `Duration/prototype/round/{exact-multiple-of-larger-unit-plaindate,
// relativeto-rounding-near-minimum-date,rounding-increment-relativeto}.js`,
// the `total` sanity checks of the `*/prototype/{since,until}/
// roundingmode-half-boundary.js` family, and `Duration/compare/relativeto-*`.
// A cold Test262 case in the ZonedDateTime family costs minutes, so this is
// the affordable regression gate for the relativeTo resolution and both
// difference paths (plain and fixed-offset zoned).

function fields(value) {
  return [
    value.years,
    value.months,
    value.weeks,
    value.days,
    value.hours,
    value.minutes,
    value.seconds,
    value.milliseconds,
    value.microseconds,
    value.nanoseconds,
  ].join(",");
}

function check(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}

var out = [];
var plain = new Temporal.PlainDate(2012, 1, 1);
var zoned = new Temporal.ZonedDateTime(0n, "UTC");

// round: the difference from relativeTo, rounded; `P31D` is exactly one month
// from 2012-01-01 whatever the rounding mode.
check(
  fields(
    new Temporal.Duration(0, 0, 0, 31).round({
      smallestUnit: "weeks",
      largestUnit: "months",
      roundingMode: "ceil",
      relativeTo: plain,
    })
  ),
  "0,1,0,0,0,0,0,0,0,0",
  "P31D ceil"
);
check(
  fields(
    new Temporal.Duration(0, 0, 0, -31).round({
      smallestUnit: "weeks",
      largestUnit: "months",
      roundingMode: "floor",
      relativeTo: plain,
    })
  ),
  "0,-1,0,0,0,0,0,0,0,0",
  "-P31D floor"
);
// Year boundaries before the minimum date are never computed.
check(
  fields(
    new Temporal.Duration(0, 0, 0, 0, -23).round({
      largestUnit: "year",
      smallestUnit: "day",
      roundingMode: "expand",
      relativeTo: new Temporal.PlainDate(-271821, 5, 19),
    })
  ),
  "0,0,0,-1,0,0,0,0,0,0",
  "near minimum"
);
out.push("round");
for (var relativeTo of [new Temporal.PlainDate(2020, 1, 1), zoned]) {
  check(
    fields(
      new Temporal.Duration(0, 0, 1, 0, 168).round({
        smallestUnit: "weeks",
        roundingIncrement: 2,
        relativeTo: relativeTo,
      })
    ),
    "0,0,2,0,0,0,0,0,0,0",
    `1w168h ${relativeTo}`
  );
}
check(
  fields(
    new Temporal.Duration(0, 1, 0, 30).round({
      smallestUnit: "months",
      roundingIncrement: 2,
      relativeTo: new Temporal.PlainDate(1970, 7, 31),
    })
  ),
  "0,2,0,0,0,0,0,0,0,0",
  "1m30d"
);
check(
  fields(
    new Temporal.Duration(0, 0, 0, 0, 25, 30).round({
      largestUnit: "days",
      smallestUnit: "hours",
      relativeTo: zoned,
    })
  ),
  "0,0,0,1,2,0,0,0,0,0",
  "zoned hours"
);
out.push("increment");

// total: one exact quotient over the calendar window.
var start = new Temporal.PlainDate(2019, 1, 1);
check(
  start.until(new Temporal.PlainDate(2020, 7, 2)).total({ unit: "years", relativeTo: start }),
  1.5,
  "1.5 years"
);
check(
  start.until(new Temporal.PlainDate(2019, 2, 15)).total({ unit: "months", relativeTo: start }),
  1.5,
  "1.5 months"
);
check(
  new Temporal.Duration(0, 0, 0, 10, 12).total({ unit: "weeks", relativeTo: start }),
  1.5,
  "1.5 weeks"
);
check(new Temporal.Duration(0, 1).total({ unit: "hours", relativeTo: start }), 744, "hours");
var zonedStart = Temporal.ZonedDateTime.from("2019-01-01T00:00+00:00[UTC]");
check(
  zonedStart
    .until(Temporal.ZonedDateTime.from("2020-07-02T00:00+00:00[UTC]"))
    .total({ unit: "years", relativeTo: zonedStart }),
  1.5,
  "zoned years"
);
out.push("total");

// relativeTo as a string, a zoned string and a property bag.
check(
  new Temporal.Duration(1, 6).total({ unit: "years", relativeTo: "2019-01-01" }),
  1.4972677595628416,
  "string"
);
check(
  new Temporal.Duration(1, 6).total({
    unit: "years",
    relativeTo: "2019-01-01T00:00+00:00[UTC]",
  }),
  1.4972677595628416,
  "zoned string"
);
check(
  new Temporal.Duration(1, 6).total({
    unit: "months",
    relativeTo: { year: 2019, month: 1, day: 1, timeZone: "+01:00" },
  }),
  18,
  "zoned bag"
);
var rejected = "";
try {
  new Temporal.Duration(1).total({ unit: "days", relativeTo: 42 });
} catch (error) {
  rejected = error.constructor.name;
}
check(rejected, "TypeError", "number relativeTo");
out.push("sources");

// compare: calendar units need a relative date.
check(
  Temporal.Duration.compare({ months: 1 }, { days: 30 }, { relativeTo: "2019-02-01" }),
  -1,
  "February"
);
check(Temporal.Duration.compare({ months: 1 }, { days: 30 }, { relativeTo: zoned }), 1, "January");
out.push("compare");

print("temporal-duration-relative-to:" + out.join("|"));

262;
