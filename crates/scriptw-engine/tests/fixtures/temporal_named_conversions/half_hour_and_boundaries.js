function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}
function range(operation, label) {
  var caught = false;
  try { operation(); }
  catch (error) { if (!(error instanceof RangeError)) throw error; caught = true; }
  if (!caught) throw new Error(label);
}
var local = new Temporal.PlainDateTime(2020, 10, 4, 2, 15);
var earlier = local.toZonedDateTime('Australia/Lord_Howe', {disambiguation: 'earlier'});
var compatible = local.toZonedDateTime('Australia/Lord_Howe');
same(compatible.epochNanoseconds - earlier.epochNanoseconds, 1800000000000n,
  'inverse gap need not be one elapsed hour');
same(earlier.hour, 1, 'earlier half-hour gap wall hour');
same(earlier.minute, 45, 'earlier half-hour gap wall minute');
same(compatible.hour, 2, 'compatible half-hour gap wall hour');
same(compatible.minute, 45, 'compatible half-hour gap wall minute');

var limit = 8640000000000000000000n;
same(new Temporal.Instant(limit).toZonedDateTimeISO('Europe/Paris').epochNanoseconds,
  limit, 'Instant conversion does not range-reject valid local projection');
same(new Temporal.Instant(-limit).toZonedDateTimeISO('America/Los_Angeles').epochNanoseconds,
  -limit, 'negative Instant boundary is retained');
same(new Temporal.PlainDate(275760, 9, 13).toZonedDateTime('UTC').epochNanoseconds,
  limit, 'date-only upper boundary');
range(function () {
  new Temporal.PlainDate(275760, 9, 13).toZonedDateTime({
    timeZone: 'UTC', plainTime: '00:00:00.000000001'
  });
}, 'explicit one nanosecond outside the Instant limit');
range(function () {
  new Temporal.PlainDate(-271821, 4, 19).toZonedDateTime({
    timeZone: 'UTC', plainTime: new Temporal.PlainTime()
  });
}, 'combined ISO date/time lower limit is enforced');
print('ok');
262;
