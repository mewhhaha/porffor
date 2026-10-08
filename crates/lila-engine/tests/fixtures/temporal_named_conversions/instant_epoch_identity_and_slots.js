function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}
var instant = new Temporal.Instant(-1n);
Object.defineProperty(instant, 'epochNanoseconds', {
  get: function () { throw new Error('receiver slots must suppress getters'); }
});
var paris = instant.toZonedDateTimeISO('europe/paris');
same(paris.epochNanoseconds, -1n, 'negative fractional epoch is unchanged');
same(paris.timeZoneId, 'Europe/Paris', 'normalized named identifier');
same(paris.calendarId, 'iso8601', 'Instant conversion always selects ISO');
same(paris.hour, 0, 'offset belongs to the negative fractional instant');
same(paris.minute, 59, 'negative fractional wall minute');
same(paris.second, 59, 'negative fractional wall second');
same(paris.nanosecond, 999, 'Euclidean fractional nanosecond');
same(instant.toZonedDateTimeISO('CET').timeZoneId, 'CET',
  'observable link identifier is not replaced by primary identity');

var zoneRecord = new Temporal.ZonedDateTime(0n, 'America/Los_Angeles', 'buddhist');
Object.defineProperty(zoneRecord, 'timeZoneId', {
  get: function () { throw new Error('branded zone slots must suppress getters'); }
});
var copy = instant.toZonedDateTimeISO(zoneRecord);
same(copy.epochNanoseconds, -1n, 'branded zone conversion retains exact epoch');
same(copy.timeZoneId, 'America/Los_Angeles', 'branded zone identity');
same(copy.calendarId, 'iso8601', 'branded zone does not substitute its calendar');
print('ok');
262;
