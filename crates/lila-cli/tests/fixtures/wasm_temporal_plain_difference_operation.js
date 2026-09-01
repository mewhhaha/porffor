function check(condition, label) {
  if (!condition) throw label;
}

var options = { smallestUnit: "years", roundingMode: "ceil" };

var earlierDate = new Temporal.PlainDate(2019, 1, 8);
var laterDate = new Temporal.PlainDate(2021, 9, 7);
check(earlierDate.until(laterDate, options).years === 3, "PlainDate until");
check(earlierDate.since(laterDate, options).years === -2, "PlainDate since");

var earlierYearMonth = new Temporal.PlainYearMonth(2019, 1);
var laterYearMonth = new Temporal.PlainYearMonth(2021, 9);
check(
  earlierYearMonth.until(laterYearMonth, options).years === 3,
  "PlainYearMonth until",
);
check(
  earlierYearMonth.since(laterYearMonth, options).years === -2,
  "PlainYearMonth since",
);

var timeOptions = { smallestUnit: "hours", roundingMode: "ceil" };
var earlierTime = new Temporal.PlainTime(8, 22, 36, 123, 456, 789);
var laterTime = new Temporal.PlainTime(12, 39, 40, 987, 654, 289);
check(earlierTime.until(laterTime, timeOptions).hours === 5, "PlainTime until");
check(
  earlierTime.since(laterTime, timeOptions).hours === -4,
  "PlainTime since",
);

var earlierDateTime = new Temporal.PlainDateTime(
  2019,
  1,
  8,
  8,
  22,
  36,
  123,
  456,
  789,
);
var laterDateTime = new Temporal.PlainDateTime(
  2021,
  9,
  7,
  12,
  39,
  40,
  987,
  654,
  289,
);
check(
  earlierDateTime.until(laterDateTime, options).years === 3,
  "PlainDateTime until",
);
check(
  earlierDateTime.since(laterDateTime, options).years === -2,
  "PlainDateTime since",
);

262;
