// Source/data-derived expectations; authored control, not an executed result.
function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { check(Object.is(actual, expected), label); }
function throws(ctor, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  check(caught instanceof ctor && caught.constructor === ctor, label);
}
function abrupt(marker, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  same(caught, marker, label);
}

var log = [], marker = {}, values = {localeMatcher:'lookup',type:'ordinal',notation:'standard',compactDisplay:'long'};
var options = new Proxy(values, {get(target, key) {
  log.push('get:' + key);
  if (['localeMatcher','type','notation','compactDisplay','minimumIntegerDigits','minimumFractionDigits','maximumFractionDigits','minimumSignificantDigits','maximumSignificantDigits','roundingIncrement','roundingMode','roundingPriority','trailingZeroDisplay'].indexOf(key) < 0) throw marker;
  return target[key];
}});
var locale = {toString() { log.push('locale'); return 'en'; }};
var ctor = function () {}, explicit = {}, target = new Proxy(ctor, {get(t, key) {
  if (key === 'prototype') { log.push('prototype'); return explicit; }
  return Reflect.get(t, key);
}});
var value = Reflect.construct(Intl.PluralRules, [[locale], options], target);
same(Object.getPrototypeOf(value), explicit, 'explicit prototype');
same(Intl.PluralRules.prototype.select.call(value, 3), 'few', 'retained brand');
same(log.join(','), 'prototype,locale,get:localeMatcher,get:type,get:notation,get:compactDisplay,get:minimumIntegerDigits,get:minimumFractionDigits,get:maximumFractionDigits,get:minimumSignificantDigits,get:maximumSignificantDigits,get:roundingIncrement,get:roundingMode,get:roundingPriority,get:trailingZeroDisplay', 'constructor order');
log = [];
throws(TypeError, function () { Intl.PluralRules([locale], options); }, 'plain call'); same(log.length, 0, 'plain call before observations');
var badTarget = new Proxy(ctor, {get() { throw marker; }});
abrupt(marker, function () { Reflect.construct(Intl.PluralRules, [[locale], options], badTarget); }, 'prototype abrupt'); same(log.length, 0, 'prototype before locale');
log = [];
var bad = new Proxy({}, {get(t, key) { log.push(key); if (key === 'notation') return {toString() { log.push('notation string'); throw marker; }}; return undefined; }});
abrupt(marker, function () { new Intl.PluralRules('en', bad); }, 'notation conversion abrupt');
same(log.join(','), 'localeMatcher,type,notation,notation string', 'notation abrupt stops compact');

print("ok constructor_observation");
262;
