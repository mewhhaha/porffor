// Exact finite epoch and UTF16 conversion controls. Authored, unrun.
function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { check(Object.is(actual, expected), label); }
function throws(kind, operation, label) {
  var caught, threw = false;
  try { operation(); } catch (error) { caught = error; threw = true; }
  check(threw && caught instanceof kind, label);
}
var epochs = [-1000001n, -1000000n, -999999n, -1n, 0n, 1n];
var millis = [-2, -1, -1, -1, 0, 0];
for (var i = 0; i < epochs.length; i++) {
  var instant = new Temporal.Instant(epochs[i]);
  same(instant.epochNanoseconds, epochs[i], 'Instant retains complete BigInt epoch');
  same(instant.epochMilliseconds, millis[i], 'Epoch milliseconds use floor below zero');
  var zoned = instant.toZonedDateTimeISO('UTC');
  same(zoned.epochNanoseconds, epochs[i], 'UTC projection preserves exact epoch');
  same(zoned.epochMilliseconds, millis[i], 'Zoned epoch milliseconds use the same floor');
}
var negative = new Temporal.Instant(-1n);
gc();
same(negative.toString(), '1969-12-31T23:59:59.999999999Z', 'Negative epoch formatting borrows the previous second');
same(negative.toString({timeZone: '+05:45'}), '1970-01-01T05:44:59.999999999+05:45', 'Fixed zone retains negative fractional epoch');
same(negative.toString({smallestUnit: 'second'}), '1969-12-31T23:59:59Z', 'Instant string truncation rounds as if positive');
same(negative.round({smallestUnit: 'microsecond', roundingMode: 'trunc'}).epochNanoseconds, -1000n, 'Instant negative as-if-positive truncation');
same(negative.round({smallestUnit: 'microsecond', roundingMode: 'ceil'}).epochNanoseconds, 0n, 'Instant negative ceil');
same(Temporal.Instant.from(negative.toString()).epochNanoseconds, -1n, 'UTF16 ISO parser round trip');
same(Temporal.Duration.from('PT0.000000001S').nanoseconds, 1, 'Duration parser exact nanosecond');
same(Temporal.Duration.from('-PT0.000000001S').nanoseconds, -1, 'Duration parser signed nanosecond');
same(Temporal.PlainDate.from('2000-02-29[u-ca=iso8601]').toString(), '2000-02-29', 'GC UTF16 calendar annotation');
for (var bad of ['PT1S\uD800', 'PT\uDC00S', 'PT1\u0000S']) {
  throws(RangeError, function() { Temporal.Duration.from(bad); }, 'Duration rejects invalid UTF16 source');
}
for (var badDate of ['2000-02-29\uD800', '2000-02-29[u-ca=iso8601\uDC00]', '2000-02-29\u0000']) {
  throws(RangeError, function() { Temporal.PlainDate.from(badDate); }, 'Date rejects invalid UTF16 source');
}
var trace = '';
var result = Temporal.PlainDate.from({
  get calendar() { trace += 'calendar;'; return 'iso8601'; },
  get day() { trace += 'day;'; return 29; },
  get month() { trace += 'month;'; return 2; },
  get monthCode() {
    trace += 'monthCode;';
    return {[Symbol.toPrimitive](hint) { same(hint, 'string', 'MonthCode primitive hint'); trace += 'code.string;'; return 'M02'; }};
  },
  get year() { trace += 'year;'; return 2000; }
}, {get overflow() { trace += 'overflow;'; return 'reject'; }});
same(result.toString(), '2000-02-29', 'Acquired whole month code resolves with year');
same(trace, 'calendar;day;month;monthCode;code.string;year;overflow;', 'Month code syntax precedes year and overflow');
trace = '';
throws(RangeError, function() {
  Temporal.PlainDate.from({
    get day() { trace += 'day;'; return 1; },
    get month() { trace += 'month;'; return 1; },
    get monthCode() { trace += 'monthCode;'; return 'M01\uD800'; },
    get year() { trace += 'year;'; return 2000; }
  }, {get overflow() { trace += 'overflow;'; return 'reject'; }});
}, 'Invalid month code rejects during acquisition');
same(trace, 'day;month;monthCode;', 'Invalid UTF16 code stops before later getters');
var marker = Symbol('month code abrupt'), caught;
trace = '';
try {
  Temporal.PlainDate.from({
    day: 1, month: 1,
    monthCode: {[Symbol.toPrimitive]() { trace += 'primitive;'; throw marker; }},
    get year() { trace += 'year;'; return 2000; }
  }, {get overflow() { trace += 'overflow;'; return 'reject'; }});
} catch (error) { caught = error; }
same(caught, marker, 'Month code preserves original Symbol throw');
same(trace, 'primitive;', 'Abrupt code coercion stops the field sweep');
throws(TypeError, function() {
  Temporal.PlainDate.from({day: 1, monthCode: {[Symbol.toPrimitive]() { return 1; }}, year: 2000});
}, 'MonthCode requires a String primitive result');
print('temporal-gc:ok');
262;
