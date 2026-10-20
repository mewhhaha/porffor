function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}
var trace = '';
var epoch = {valueOf: function () { trace += 'B'; return -1n; }};
var zdt = new Temporal.ZonedDateTime(epoch, 'asia/kolkata', 'ISO8601');
same(trace, 'B', 'constructor performs ToBigInt once');
same(zdt.epochNanoseconds, -1n, 'constructor keeps negative fractional epoch');
same(zdt.year, 1970, 'projected year');
same(zdt.hour, 5, 'projected hour');
same(zdt.minute, 29, 'projected minute');
same(zdt.second, 59, 'projected second');
same(zdt.millisecond, 999, 'floor millisecond');
same(zdt.microsecond, 999, 'canonical microsecond');
same(zdt.nanosecond, 999, 'canonical nanosecond');
var historic = new Temporal.ZonedDateTime(-2208988800000000000n, 'Europe/Paris');
same(historic.offset, '+00:09:21', 'exact historical offset getter');
same(historic.offsetNanoseconds, 561000000000, 'exact offset nanoseconds');
var alias = new Temporal.ZonedDateTime(0n, 'US/Eastern', 'gregory');
var primary = new Temporal.ZonedDateTime(0n, 'America/New_York', 'gregory');
same(alias.equals(primary), true, 'primary identity equality');
var moved = alias.withTimeZone('Europe/Paris');
same(moved.epochNanoseconds, 0n, 'withTimeZone keeps exact branded epoch');
same(moved.calendarId, 'gregory', 'withTimeZone keeps actual branded calendar');
var caught;
try { new Temporal.ZonedDateTime(0, 'Unknown/NotAZone', {}); }
catch (error) { caught = error; }
same(caught instanceof TypeError, true, 'ToBigInt failure precedes zone and calendar');
print('ok');
262;
