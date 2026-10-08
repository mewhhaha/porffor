function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}
var torontoDate = new Temporal.PlainDate(1919, 3, 31);
var shorthand = torontoDate.toZonedDateTime('America/Toronto');
var omitted = torontoDate.toZonedDateTime({timeZone: 'America/Toronto'});
var undefinedTime = torontoDate.toZonedDateTime({
  timeZone: 'America/Toronto', plainTime: undefined
});
var explicit = torontoDate.toZonedDateTime({
  timeZone: 'America/Toronto', plainTime: new Temporal.PlainTime()
});
same(shorthand.hour, 0, 'first instant of date starts in hour zero');
same(shorthand.minute, 30, 'historical gap crosses midnight');
same(explicit.hour, 1, 'compatible explicit midnight moves by the full gap');
same(explicit.minute, 0, 'explicit midnight is not start of day');
same(explicit.epochNanoseconds - shorthand.epochNanoseconds, 1800000000000n,
  'start of day precedes compatible midnight by thirty minutes');
same(omitted.epochNanoseconds, shorthand.epochNanoseconds, 'omitted plainTime');
same(undefinedTime.epochNanoseconds, shorthand.epochNanoseconds, 'undefined plainTime');

var skipped = new Temporal.PlainDate(2011, 12, 30).toZonedDateTime('Pacific/Apia');
same(skipped.epochNanoseconds, 1325239200000000000n, 'whole skipped date boundary');
same(skipped.year, 2011, 'skipped date resulting year');
same(skipped.month, 12, 'skipped date resulting month');
same(skipped.day, 31, 'first instant after the whole-date jump');
same(skipped.hour, 0, 'skipped date resulting midnight');
print('ok');
262;
