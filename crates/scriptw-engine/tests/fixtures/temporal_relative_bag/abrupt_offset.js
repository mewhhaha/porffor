function check(actual, expected, label) {
  if (actual !== expected) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function total(bag) {
  return Temporal.Duration.from({hours: 1}).total({unit: 'hours', relativeTo: bag});
}
function expect(Type, body, label) {
  var caught;
  try {body();} catch (error) {caught = error;}
  if (!caught || caught.constructor !== Type) throw new Error(label + ': wrong completion');
}
var later = 0;
var invalidOffset = {year: 2021, month: 3, day: 14, offset: 'bad'};
Object.defineProperty(invalidOffset, 'second', {get: function() {later++; return 0;}});
Object.defineProperty(invalidOffset, 'timeZone', {get: function() {later++; return 'UTC';}});
expect(RangeError, function() {total(invalidOffset);}, 'offset grammar checked during field read');
check(later, 0, 'invalid offset stops before second and timeZone');
expect(TypeError, function() {total({year: 2021, month: 3, day: 14, offset: 5});}, 'offset requires primitive string');
var zoneCalls = 0;
var yearCalls = 0;
var invalidZone = {month: 3, day: 14, timeZone: {toString: function() {zoneCalls++; return 'UTC';}}};
Object.defineProperty(invalidZone, 'year', {get: function() {yearCalls++; return 2021;}});
expect(TypeError, function() {total(invalidZone);}, 'zone conversion requires identifier or brand');
check(zoneCalls, 0, 'timeZone does not invoke arbitrary ToString');
check(yearCalls, 0, 'zone rejection precedes year read');
var marker = {};
var stopped = 0;
var throwing = {calendar: 'gregory'};
Object.defineProperty(throwing, 'day', {get: function() {throw marker;}});
Object.defineProperty(throwing, 'era', {get: function() {stopped++; return 'ce';}});
var actual;
try {total(throwing);} catch (error) {actual = error;}
check(actual, marker, 'getter abrupt identity');
check(stopped, 0, 'day abrupt precedes era');
expect(RangeError, function() {total({year: 2021, month: 3, day: 14, hour: 2, minute: 30, timeZone: 'America/New_York', offset: '-05:00'});}, 'gap explicit offset reject');
expect(RangeError, function() {total({year: 1900, month: 1, day: 1, timeZone: 'Europe/Paris', offset: '+00:09'});}, 'bag uses match-exactly');
check(total({year: 2021, month: 3, day: 14, hour: 2, minute: 30, timeZone: 'America/New_York'}), 1, 'gap missing offset compatible');
262;
