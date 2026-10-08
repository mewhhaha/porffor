// Finite named-provider, calendar and intrinsic-Realm controls. Authored, unrun.
function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { check(Object.is(actual, expected), label); }
function throws(kind, operation, label) {
  var caught, threw = false;
  try { operation(); } catch (error) { caught = error; threw = true; }
  check(threw && caught instanceof kind, label);
}
var fold = new Temporal.PlainDateTime(2000, 10, 29, 1, 45, 0, 123, 456, 789, 'buddhist');
var early = fold.toZonedDateTime('America/Los_Angeles', {disambiguation: 'earlier'});
var late = fold.toZonedDateTime('America/Los_Angeles', {disambiguation: 'later'});
same(early.epochNanoseconds, 972809100123456789n, 'Provider earlier fold candidate');
same(late.epochNanoseconds, 972812700123456789n, 'Provider later fold candidate');
same(early.calendarId, 'buddhist', 'Provider proof retains canonical calendar');
same(late.offset, '-08:00', 'Later fold offset');
same(early.toPlainDate().year, 2543, 'Zoned to Date calendar projection');
same(early.toPlainDateTime().nanosecond, 789, 'Zoned to DateTime complete subsecond');
var marker = { identity: 37 };
for (var key of ['calendar', 'calendarId', 'timeZoneId', 'year', 'month', 'day', 'hour', 'epochNanoseconds']) {
  Object.defineProperty(early, key, {get() { throw marker; }, configurable: true});
}
gc();
same(Temporal.PlainDate.from(early).year, 2543, 'Zoned to Date uses retained record/provider slots');
same(Temporal.PlainDateTime.from(early).nanosecond, 789, 'Zoned to DateTime ignores public poison');
same(Temporal.ZonedDateTime.from(early).epochNanoseconds, 972809100123456789n, 'Zoned branded conversion retains exact epoch');
var gap = new Temporal.PlainDateTime(2000, 4, 2, 2, 30);
same(gap.toZonedDateTime('America/Los_Angeles', {disambiguation: 'earlier'}).epochNanoseconds, 954667800000000000n, 'Provider earlier gap candidate');
same(gap.toZonedDateTime('America/Los_Angeles', {disambiguation: 'compatible'}).epochNanoseconds, 954671400000000000n, 'Provider compatible gap candidate');
throws(RangeError, function() { gap.toZonedDateTime('America/Los_Angeles', {disambiguation: 'reject'}); }, 'Gap reject uses JavaScript RangeError');
var spring = Temporal.ZonedDateTime.from('2020-03-29T00:00+01:00[Europe/Paris]');
var autumn = Temporal.ZonedDateTime.from('2020-10-25T00:00+02:00[Europe/Paris]');
same(spring.hoursInDay, 23, 'Provider spring day length');
same(autumn.hoursInDay, 25, 'Provider autumn day length');
same(spring.add({days: 1}).epochNanoseconds - spring.epochNanoseconds, 82800000000000n, 'Calendar day resolves across transition');
same(spring.add({hours: 24}).epochNanoseconds - spring.epochNanoseconds, 86400000000000n, 'Elapsed day retains exact epoch arithmetic');
same(new Temporal.Duration(0, 0, 0, 1).total({unit: 'hour', relativeTo: spring}), 23, 'Duration relativeTo consumes branded zone proof');
same(new Temporal.Duration(0, 0, 0, 1).total({unit: 'hour', relativeTo: autumn}), 25, 'RelativeTo retains backward transition');

// Finite fixed-offset parser and independent property-bag initializer controls.
for (var offsetCase of [
  ['+00', '+00:00', 0], ['-00', '+00:00', 0],
  ['+01', '+01:00', 3600000000000], ['-01', '-01:00', -3600000000000],
  ['+0130', '+01:30', 5400000000000], ['-0130', '-01:30', -5400000000000],
  ['+01:30', '+01:30', 5400000000000], ['-01:30', '-01:30', -5400000000000]
]) {
  var fixed = new Temporal.ZonedDateTime(0n, offsetCase[0]);
  same(fixed.timeZoneId, offsetCase[1], 'Fixed offset identifier ' + offsetCase[0]);
  same(fixed.offset, offsetCase[1], 'Fixed offset text ' + offsetCase[0]);
  same(fixed.offsetNanoseconds, offsetCase[2], 'Fixed offset nanoseconds ' + offsetCase[0]);
}
for (var invalidOffset of ['+', '+1', '+010', '+01:', '+01:6', '+01:60', '+24', '-24:00']) {
  throws(RangeError, function() { new Temporal.ZonedDateTime(0n, invalidOffset); }, 'Malformed/range fixed offset ' + invalidOffset);
}
var absentOffsetBag = {year: 1970, month: 1, day: 1, timeZone: '+01'};
var explicitOffsetBag = {year: 1970, month: 1, day: 1, timeZone: '+01', offset: '+01:00'};
same(Temporal.ZonedDateTime.from(absentOffsetBag).epochNanoseconds, -3600000000000n, 'Absent offset bag starts with independent zero locals');
same(Temporal.ZonedDateTime.from(explicitOffsetBag).epochNanoseconds, -3600000000000n, 'Explicit offset bag replaces initialized offset');
var calendarDifference = new Temporal.PlainDate(2000, 1, 1).until(new Temporal.PlainDate(2001, 3, 1), {largestUnit: 'year'});
same(calendarDifference.years, 1, 'Calendar difference independent years initializer');
same(calendarDifference.months, 2, 'Calendar difference independent months initializer');
same(calendarDifference.days, 0, 'Calendar difference exact remaining days');

var foreign = __lilaCreateRealm().global;
var ForeignRangeError = foreign.RangeError, ForeignTypeError = foreign.TypeError;
var rangePrototype = ForeignRangeError.prototype, typePrototype = ForeignTypeError.prototype;
var ForeignDate = foreign.Temporal.PlainDate, ForeignInstant = foreign.Temporal.Instant;
var foreignToZoned = foreign.Temporal.PlainDateTime.prototype.toZonedDateTime;
var foreignDurationWith = foreign.Temporal.Duration.prototype.with;
foreign.RangeError = foreign.TypeError = function() { throw 'mutable public error constructor'; };
var caught, prototypeReads = 0;
var target = new Proxy(function() {}, {get(original, key, receiver) {
  if (key === 'prototype') { prototypeReads++; return ForeignDate.prototype; }
  return Reflect.get(original, key, receiver);
}});
try {
  Reflect.construct(ForeignDate, [2000, {valueOf() { new Temporal.Instant(0n); return 13; }}, 1], target);
} catch (error) { caught = error; }
check(caught && Object.getPrototypeOf(caught) === rangePrototype && prototypeReads === 0, 'Constructor validates before prototype and restores its Realm after nested Call');
caught = undefined;
try { new ForeignInstant(Symbol('epoch')); } catch (error) { caught = error; }
check(caught && Object.getPrototypeOf(caught) === typePrototype, 'Instant conversion TypeError belongs to constructor Realm');
caught = undefined;
var gets = 0;
try {
  foreignToZoned.call(gap, 'America/Los_Angeles', {get disambiguation() {
    gets++; new Temporal.PlainDateTime(2000, 1, 1).toZonedDateTime('UTC'); return 'reject';
  }});
} catch (error) { caught = error; }
check(caught && Object.getPrototypeOf(caught) === rangePrototype && gets === 1, 'Borrowed inverse retains defining Realm after option hook');
caught = undefined;
try { foreignDurationWith.call(new Temporal.Duration(), {get days() { throw marker; }}); }
catch (error) { caught = error; }
same(caught, marker, 'Borrowed Duration preserves user abrupt identity');
caught = undefined;
try { new foreign.Temporal.ZonedDateTime(0n, '+24'); } catch (error) { caught = error; }
check(caught && Object.getPrototypeOf(caught) === rangePrototype, 'Fixed offset parser failure retains foreign defining Realm');
gc();
same(ForeignDate.from(new Temporal.PlainDate(2000, 2, 29)).toString(), '2000-02-29', 'Foreign allocation retains complete branded value');
print('temporal-gc:ok');
262;
