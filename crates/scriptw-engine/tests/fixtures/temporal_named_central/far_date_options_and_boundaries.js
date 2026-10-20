function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}
function rejectsAfterOptions(input) {
  var trace = '';
  var options = {
    get disambiguation() { trace += 'D'; return 'compatible'; },
    get offset() { trace += 'O'; return 'reject'; },
    get overflow() { trace += 'V'; return 'constrain'; }
  };
  var caught;
  try { Temporal.ZonedDateTime.from(input, options); }
  catch (error) { caught = error; }
  same(caught instanceof RangeError, true, 'far date uses JavaScript RangeError');
  same(trace, 'DOV', 'all options precede interpretation range failure');
}
rejectsAfterOptions('+999999-01-01T12:00[America/New_York]');
rejectsAfterOptions('-999999-01-01T12:00[America/New_York]');
rejectsAfterOptions('+999999-01-01[America/New_York]');
rejectsAfterOptions({year: -271821, month: 1, day: 1, timeZone: 'America/New_York'});
rejectsAfterOptions({year: 275760, month: 12, day: 31, timeZone: 'America/New_York'});
var marker = {};
var caught;
try {
  Temporal.ZonedDateTime.from('+999999-01-01T12:00[America/New_York]', {
    get disambiguation() { return 'compatible'; },
    get offset() { throw marker; },
    get overflow() { throw new Error('overflow read after abrupt option'); }
  });
} catch (error) { caught = error; }
same(caught, marker, 'option abrupt completion precedes far-date interpretation');
var minimum = Temporal.ZonedDateTime.from('-271821-04-19T23:00[Etc/GMT+1]');
same(minimum.epochNanoseconds, -8640000000000000000000n,
  'wall local day below inverse checked-day domain can map to valid Instant');
var maximum = Temporal.ZonedDateTime.from('+275760-09-13T00:00[UTC]');
same(maximum.epochNanoseconds, 8640000000000000000000n, 'inclusive maximum epoch');
print('ok');
262;
