function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}

function range(operation, label) {
  var caught = false;
  try { operation(); }
  catch (error) {
    if (!(error instanceof RangeError)) throw error;
    caught = true;
  }
  if (!caught) throw new Error(label);
}

var fixed = Temporal.ZonedDateTime.from('1970-01-01T12:00+01:00[+01:00]');
same(fixed.epochNanoseconds, 39600000000000n, 'fixed epoch');
same(fixed.timeZoneId, '+01:00', 'fixed identity');
same(fixed.hour, 12, 'fixed wall time');
same(fixed.offset, '+01:00', 'fixed offset');

var utc = Temporal.ZonedDateTime.from('1970-01-01T12:00Z[UTC]');
same(utc.epochNanoseconds, 43200000000000n, 'UTC epoch');
same(utc.timeZoneId, 'UTC', 'UTC identity');
same(utc.hour, 12, 'UTC wall time');
same(utc.offset, '+00:00', 'UTC offset');
same(new Temporal.ZonedDateTime(0n, 'uTc').timeZoneId, 'UTC', 'UTC case folding');
same(new Temporal.ZonedDateTime(0n, '+0100').timeZoneId, '+01:00', 'fixed normalization');

same(Temporal.ZonedDateTime.from({year:1970, month:1, day:1, hour:12,
  timeZone:'+01:00'}).epochNanoseconds, 39600000000000n, 'fixed property bag');
same(Temporal.ZonedDateTime.from({year:1970, month:1, day:1, hour:12,
  timeZone:'UTC'}).epochNanoseconds, 43200000000000n, 'UTC property bag');

var oneDay = new Temporal.Duration(0, 0, 0, 1);
same(oneDay.total({unit:'hour', relativeTo:'1970-01-01T12:00+01:00[+01:00]'}),
  24, 'fixed relative duration');
same(oneDay.total({unit:'hour', relativeTo:'1970-01-01T12:00Z[UTC]'}),
  24, 'UTC relative duration');

var mismatch = '1970-01-01T12:00+02:00[+01:00]';
range(function () { Temporal.ZonedDateTime.from(mismatch); }, 'default offset rejection');
range(function () { Temporal.ZonedDateTime.from(mismatch, {offset:'reject'}); },
  'explicit offset rejection');
same(Temporal.ZonedDateTime.from(mismatch, {offset:'use'}).epochNanoseconds,
  36000000000000n, 'use explicit offset');
for (var offset of ['prefer', 'ignore']) {
  same(Temporal.ZonedDateTime.from(mismatch, {offset:offset}).epochNanoseconds,
    39600000000000n, offset + ' fixed zone offset');
}

print('ok');
262;
