function checkDateParts(duration, years, months, weeks, days, message) {
  if (duration.years !== years || duration.months !== months ||
      duration.weeks !== weeks || duration.days !== days ||
      duration.hours !== 0 || duration.minutes !== 0 || duration.seconds !== 0 ||
      duration.milliseconds !== 0 || duration.microseconds !== 0 ||
      duration.nanoseconds !== 0) throw new Error(message);
}

// A zero month/week count does not mean the rounding window starts at the
// original date: the window retains the duration's larger calendar fields.
const positive = new Temporal.Duration(1, 0, 0, 1);
const negative = new Temporal.Duration(-1, 0, 0, -1);
checkDateParts(positive.round({largestUnit: 'year', smallestUnit: 'month',
  relativeTo: '2020-01-01'}), 1, 0, 0, 0, 'positive year/month window');
checkDateParts(negative.round({largestUnit: 'year', smallestUnit: 'month',
  relativeTo: '2020-01-01'}), -1, 0, 0, 0, 'negative year/month window');
checkDateParts(positive.round({largestUnit: 'year', smallestUnit: 'week',
  relativeTo: '2020-01-01'}), 1, 0, 0, 0, 'year/week window');
checkDateParts(new Temporal.Duration(0, 1, 0, 1).round({largestUnit: 'month',
  smallestUnit: 'week', relativeTo: '2020-02-01'}),
  0, 1, 0, 0, 'month/week window');
checkDateParts(positive.round({largestUnit: 'year', smallestUnit: 'month',
  relativeTo: '2020-01-01T12:34:56.123456789+05:30[+05:30]'}),
  1, 0, 0, 0, 'fixed-zone year/month window');

// February 2020 has 29 days, so fourteen days and twelve hours is an exact
// half-month. The decision must use exact spans and the selected tie mode.
const halfMonth = new Temporal.Duration(0, 0, 0, 14, 12);
checkDateParts(halfMonth.round({smallestUnit: 'month', roundingMode: 'halfExpand',
  relativeTo: '2020-02-01'}), 0, 1, 0, 0, 'half expand');
checkDateParts(halfMonth.round({smallestUnit: 'month', roundingMode: 'halfEven',
  relativeTo: '2020-02-01'}), 0, 0, 0, 0, 'half even');
checkDateParts(new Temporal.Duration(0, 0, 0, -14, -12).round({smallestUnit: 'month',
  roundingMode: 'halfFloor', relativeTo: '2020-03-01'}),
  0, -1, 0, 0, 'negative half floor');

true;
