function same(actual, expected) {
  if (!Object.is(actual, expected)) throw new Error(actual + ' != ' + expected);
}
// Days and weeks retain their original coefficients when a window is
// recomputed. Actual elapsed spans, including half-hour changes, own totals.
for (var item of [
  ['2024-03-09T12:00[America/New_York]', 23],
  ['2024-11-02T12:00[America/New_York]', 25],
  ['2020-10-03T12:00[Australia/Lord_Howe]', 23.5]
]) {
  var origin = Temporal.ZonedDateTime.from(item[0]);
  var half = Temporal.Duration.from({minutes:item[1]*30});
  same(half.total({unit:'days', relativeTo:origin}), 0.5);
  same(half.round({smallestUnit:'days', roundingMode:'halfTrunc', relativeTo:origin}).days, 0);
  same(half.round({smallestUnit:'days', roundingMode:'halfExpand', relativeTo:origin}).days, 1);
  var endpoint = origin.add({days:1});
  var reverse = Temporal.Duration.from({minutes:-item[1]*30});
  same(reverse.total({unit:'days', relativeTo:endpoint}), -0.5);
  same(reverse.round({smallestUnit:'days', roundingMode:'halfTrunc', relativeTo:endpoint}).days, 0);
  same(reverse.round({smallestUnit:'days', roundingMode:'halfExpand', relativeTo:endpoint}).days, -1);
  same(origin.until(endpoint, {largestUnit:'days', smallestUnit:'days'}).days, 1);
  same(endpoint.until(origin, {largestUnit:'days', smallestUnit:'days'}).days, -1);
}
var utc = Temporal.ZonedDateTime.from('1969-12-31T12:00[UTC]');
same(Temporal.Duration.from({days:7}).total({unit:'weeks', relativeTo:utc}), 1);
same(Temporal.Duration.from({days:-7}).total({unit:'weeks', relativeTo:utc}), -1);
262;
