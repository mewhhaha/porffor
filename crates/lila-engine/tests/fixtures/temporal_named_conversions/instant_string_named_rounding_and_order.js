function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual);
}
function check(value, label) { if (!value) throw new Error(label); }
function throws(C, body, label) {
  var caught;
  try { body(); } catch (error) { caught = error; }
  check(caught && caught.constructor === C, label);
}
var epoch = new Temporal.Instant(0n);
same(epoch.toString(), '1970-01-01T00:00:00Z', 'absent zone uses Z');
same(epoch.toString({timeZone: undefined}), '1970-01-01T00:00:00Z', 'undefined zone uses Z');
same(epoch.toString({timeZone: 'UTC'}), '1970-01-01T00:00:00+00:00', 'explicit UTC uses numeric offset');
same(epoch.toString({timeZone: '+05:45'}), '1970-01-01T05:45:00+05:45', 'fixed nonhour zone');
same(epoch.toString({timeZone: 'Europe/Berlin'}), '1970-01-01T01:00:00+01:00', 'named positive offset');
same(epoch.toString({timeZone: 'America/New_York'}), '1969-12-31T19:00:00-05:00', 'named negative offset');
same(epoch.toString({timeZone: 'Africa/Monrovia'}), '1969-12-31T23:15:30-00:45', 'exact civil seconds with nearest-minute offset');
var negative = new Temporal.Instant(-1n);
same(negative.toString(), '1969-12-31T23:59:59.999999999Z', 'negative nanosecond default');
same(negative.toString({timeZone: 'Europe/Berlin'}), '1970-01-01T00:59:59.999999999+01:00', 'negative nanosecond positive zone');
same(negative.toString({timeZone: 'Africa/Monrovia'}), '1969-12-31T23:15:29.999999999-00:45', 'negative nanosecond historical seconds');
same(negative.toString({timeZone: 'Africa/Monrovia', smallestUnit: 'second'}), '1969-12-31T23:15:29-00:45', 'epoch truncation rounds toward earlier time');

for (var row of [
  [1585443599999999999n, 'Europe/Berlin', '2020-03-29T03:00:00+02:00'],
  [1603587599999999999n, 'Europe/Berlin', '2020-10-25T02:00:00+01:00'],
  [1583650799999999999n, 'America/New_York', '2020-03-08T03:00:00-04:00'],
  [1604210399999999999n, 'America/New_York', '2020-11-01T01:00:00-05:00']
]) {
  same(new Temporal.Instant(row[0]).toString({timeZone: row[1], smallestUnit: 'second', roundingMode: 'halfExpand'}), row[2], 'rounded epoch chooses post-transition offset');
}
var limit = 8640000000000000000000n;
same(new Temporal.Instant(limit).toString(), '+275760-09-13T00:00:00Z', 'upper Instant edge');
same(new Temporal.Instant(-limit).toString(), '-271821-04-20T00:00:00Z', 'lower Instant edge');
same(new Temporal.Instant(limit).toString({timeZone: '+01:00'}), '+275760-09-13T01:00:00+01:00', 'upper projection beyond Date TimeClip');
same(new Temporal.Instant(-limit).toString({timeZone: '-01:00'}), '-271821-04-19T23:00:00-01:00', 'lower projection beyond Date TimeClip');
same(new Temporal.Instant(limit - 1n).toString({timeZone: 'UTC'}), '+275760-09-12T23:59:59.999999999+00:00', 'upper neighboring exact nanosecond');
same(new Temporal.Instant(-limit + 1n).toString({timeZone: 'UTC'}), '-271821-04-20T00:00:00.000000001+00:00', 'lower neighboring exact nanosecond');

var log = [];
var options = {
  get fractionalSecondDigits() { log.push('digits'); return {toString() { log.push('digits.string'); return 'auto'; }}; },
  get roundingMode() { log.push('mode'); return {toString() { log.push('mode.string'); return 'halfExpand'; }}; },
  get smallestUnit() { log.push('unit'); return {toString() { log.push('unit.string'); return 'second'; }}; },
  get timeZone() { log.push('zone'); return 'Europe/Berlin'; }
};
same(epoch.toString(options), '1970-01-01T01:00:00+01:00', 'observed options formatting');
same(log.join(','), 'digits,digits.string,mode,mode.string,unit,unit.string,zone', 'exact Get/coercion order');
var record = new Temporal.ZonedDateTime(0n, 'Europe/Berlin');
Object.defineProperty(record, 'timeZoneId', {get() { throw new Error('public slot getter observed'); }});
same(epoch.toString({timeZone: record}), '1970-01-01T01:00:00+01:00', 'branded zone proof without public reads');

for (var unit of ['hour', 'month']) {
  var gets = [], conversions = 0;
  var badZone = {toString() { conversions++; throw new Error('zone must not be coerced'); }};
  var opts = new Proxy({}, {get(target, key) {
    gets.push(key);
    return key === 'smallestUnit' ? unit : key === 'timeZone' ? badZone : undefined;
  }});
  throws(RangeError, function() { epoch.toString(opts); }, 'unit error before nonstring zone TypeError');
  same(gets.join(','), 'fractionalSecondDigits,roundingMode,smallestUnit,timeZone', 'timeZone Get before unit validation');
  same(conversions, 0, 'invalid unit cannot convert zone');
}
throws(TypeError, function() { epoch.toString({smallestUnit: 'second', timeZone: {toString() { throw new Error('not string coercion'); }}}); }, 'valid unit reaches nonstring zone rejection');
for (var fault of ['fractionalSecondDigits', 'roundingMode', 'smallestUnit', 'timeZone']) {
  var marker = {}, gets = [];
  var opts = new Proxy({}, {get(target, key) {
    gets.push(key); if (key === fault) throw marker;
    return key === 'smallestUnit' ? 'hour' : undefined;
  }});
  var caught = undefined;
  try { epoch.toString(opts); } catch (error) { caught = error; }
  same(caught, marker, 'option getter abrupt identity ' + fault);
  var all = ['fractionalSecondDigits', 'roundingMode', 'smallestUnit', 'timeZone'];
  same(gets.join(','), all.slice(0, all.indexOf(fault) + 1).join(','), 'no later Get after abrupt');
}
var touches = 0;
var poison = new Proxy({}, {get() { touches++; throw new Error('brand must precede options'); }});
for (var bad of [{}, Temporal.Instant.prototype, new Proxy(epoch, {})]) {
  throws(TypeError, function() { Temporal.Instant.prototype.toString.call(bad, poison); }, 'actual Instant brand');
}
same(touches, 0, 'brand before options Get');
throws(TypeError, function() { epoch.toString(1); }, 'strict options object');
var foreign = __lilaCreateRealm().global;
var otherMethod = foreign.Temporal.Instant.prototype.toString;
throws(foreign.RangeError, function() { otherMethod.call(epoch, {smallestUnit: 'hour', timeZone: 1}); }, 'borrowed method owns unit RangeError');
throws(foreign.TypeError, function() { otherMethod.call(epoch, {timeZone: 1}); }, 'borrowed method owns zone TypeError');
same(otherMethod.call(epoch, {timeZone: 'Europe/Berlin'}), '1970-01-01T01:00:00+01:00', 'borrowed method projects retained epoch');
print('ok');
262;
