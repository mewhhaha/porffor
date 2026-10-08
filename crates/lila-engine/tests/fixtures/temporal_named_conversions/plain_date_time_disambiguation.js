function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}
function range(operation, label) {
  var caught = false;
  try { operation(); }
  catch (error) { if (!(error instanceof RangeError)) throw error; caught = true; }
  if (!caught) throw new Error(label);
}
var fold = new Temporal.PlainDateTime(2000, 10, 29, 1, 45, 0, 123, 456, 789, 'buddhist');
var early = 972809100123456789n;
var late = 972812700123456789n;
same(fold.toZonedDateTime('America/Los_Angeles').epochNanoseconds, early, 'fold default');
for (var policy of ['compatible', 'earlier', 'later']) {
  var converted = fold.toZonedDateTime('America/Los_Angeles', {disambiguation: policy});
  same(converted.epochNanoseconds, policy === 'later' ? late : early, 'retained fold policy');
  same(converted.calendarId, 'buddhist', 'PlainDateTime actual calendar');
}
range(function () {
  fold.toZonedDateTime('America/Los_Angeles', {disambiguation: 'reject'});
}, 'fold reject');

var gap = new Temporal.PlainDateTime(2000, 4, 2, 2, 30);
same(gap.toZonedDateTime('America/Los_Angeles').epochNanoseconds,
  954671400000000000n, 'gap default compatible');
for (var policy of ['compatible', 'earlier', 'later']) {
  same(gap.toZonedDateTime('America/Los_Angeles', {disambiguation: policy}).epochNanoseconds,
    policy === 'earlier' ? 954667800000000000n : 954671400000000000n,
    'retained gap policy');
}
range(function () {
  gap.toZonedDateTime('America/Los_Angeles', {disambiguation: 'reject'});
}, 'gap reject');
print('ok');
262;
