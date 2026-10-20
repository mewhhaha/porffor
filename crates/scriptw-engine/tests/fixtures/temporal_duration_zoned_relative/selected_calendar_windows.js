function same(actual, expected) {
  if (!Object.is(actual, expected)) throw new Error(actual + ' != ' + expected);
}
function fields(duration, unit, value) {
  for (var name of ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds']) {
    same(duration[name], name === unit ? value : 0);
  }
}
// CalendarDateUntil conservatively leaves the constrained month/year in the
// smaller fields. The destination is one hour beyond the first far endpoint;
// the prescribed second year/month window must be selected before rounding.
for (var zone of ['UTC', '+05:30', 'America/New_York']) {
  var origin = Temporal.ZonedDateTime.from('2023-01-31T12:00[' + zone + ']');
  var duration = Temporal.Duration.from({days:28, hours:1});
  var destination = origin.add(duration);
  var expectedTotal = zone === 'America/New_York' ? 1.0013458950201883 : 1.0013440860215055;
  same(duration.total({unit:'months', relativeTo:origin}), expectedTotal);
  for (var mode of ['trunc','floor','halfEven','halfExpand']) {
    fields(duration.round({largestUnit:'months', smallestUnit:'months', roundingMode:mode, relativeTo:origin}), 'months', 1);
    fields(origin.until(destination, {largestUnit:'months', smallestUnit:'months', roundingMode:mode}), 'months', 1);
  }
  fields(duration.round({largestUnit:'months', smallestUnit:'months', roundingMode:'ceil', relativeTo:origin}), 'months', 2);
  fields(origin.until(destination, {largestUnit:'months', smallestUnit:'months', roundingMode:'expand'}), 'months', 2);
}
for (var start of ['1968-02-29T12:00[UTC]', '2024-02-29T12:00[UTC]']) {
  var origin = Temporal.ZonedDateTime.from(start);
  var duration = Temporal.Duration.from({days:365, hours:1});
  var destination = origin.add(duration);
  same(duration.total({unit:'years', relativeTo:origin}), 1.0001141552511414);
  fields(duration.round({largestUnit:'years', smallestUnit:'years', roundingMode:'trunc', relativeTo:origin}), 'years', 1);
  fields(origin.until(destination, {largestUnit:'years', smallestUnit:'years', roundingMode:'ceil'}), 'years', 2);
}
// The unselected end of the recomputed window still undergoes its required
// Instant range validation. Choosing its near end does not suppress RangeError.
for (var item of [
  ['+275760-07-31T12:00[UTC]', {days:31, hours:1}, 'months'],
  ['+275759-02-28T12:00[UTC]', {days:365, hours:1}, 'years']
]) {
  var origin = Temporal.ZonedDateTime.from(item[0]);
  var duration = Temporal.Duration.from(item[1]);
  var destination = origin.add(duration);
  var caught = 0;
  try { duration.round({largestUnit:item[2], smallestUnit:item[2], roundingMode:'trunc', relativeTo:origin}); }
  catch (error) { if (!(error instanceof RangeError)) throw error; caught += 1; }
  try { duration.total({unit:item[2], relativeTo:origin}); }
  catch (error) { if (!(error instanceof RangeError)) throw error; caught += 1; }
  try { origin.until(destination, {largestUnit:item[2], smallestUnit:item[2], roundingMode:'trunc'}); }
  catch (error) { if (!(error instanceof RangeError)) throw error; caught += 1; }
  same(caught, 3);
}
262;
