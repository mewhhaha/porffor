function same(actual, expected) {
  if (!Object.is(actual, expected)) throw new Error(actual + ' != ' + expected);
}
function parts(duration, years, months, weeks) {
  same(duration.years, years);
  same(duration.months, months);
  same(duration.weeks, weeks);
  for (var name of ['days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds']) {
    same(duration[name], 0);
  }
}
// Issue3316: the whole start DateDuration, including retained larger units,
// determines origin reuse. Zero rounded month/week/day alone is insufficient.
for (var relativeTo of ['2020-01-01', '2020-01-01T12:00[UTC]', '2020-01-01T12:00[America/New_York]']) {
  for (var sign of [1, -1]) {
    var year = Temporal.Duration.from({years:sign, hours:sign});
    parts(year.round({largestUnit:'years', smallestUnit:'months', relativeTo:relativeTo}), sign, 0, 0);
    parts(year.round({largestUnit:'years', smallestUnit:'weeks', relativeTo:relativeTo}), sign, 0, 0);
    parts(year.round({largestUnit:'years', smallestUnit:'days', relativeTo:relativeTo}), sign, 0, 0);
    var month = Temporal.Duration.from({months:sign, hours:sign});
    parts(month.round({largestUnit:'months', smallestUnit:'weeks', relativeTo:relativeTo}), 0, sign, 0);
    parts(month.round({largestUnit:'months', smallestUnit:'days', relativeTo:relativeTo}), 0, sign, 0);
    var week = Temporal.Duration.from({weeks:sign, hours:sign});
    parts(week.round({largestUnit:'weeks', smallestUnit:'days', relativeTo:relativeTo}), 0, 0, sign);
  }
}
// An actually empty start DateDuration reuses this exact later fold occurrence.
// Compatible inversion of its wall time would instead select the earlier one.
var folded = Temporal.ZonedDateTime.from('2024-11-03T01:30-05:00[America/New_York]');
var hour = Temporal.Duration.from({hours:1});
same(hour.total({unit:'days', relativeTo:folded}), 1 / 24);
same(hour.round({smallestUnit:'days', roundingMode:'trunc', relativeTo:folded}).days, 0);
same(hour.round({smallestUnit:'days', roundingMode:'ceil', relativeTo:folded}).days, 1);
262;
