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

var log = [], numbers = {minimumIntegerDigits:1,minimumFractionDigits:0,maximumFractionDigits:2,roundingIncrement:1};
var texts = {roundingMode:'halfEven',roundingPriority:'auto',trailingZeroDisplay:'auto'};
var options = new Proxy({}, {get(t, key) {
  log.push('get:' + key);
  if (key in numbers) return {valueOf() { log.push('number:' + key); return numbers[key]; }};
  if (key in texts) return {toString() { log.push('text:' + key); return texts[key]; }};
  return undefined;
}});
new Intl.PluralRules('en', options);
same(log.join(','), 'get:localeMatcher,get:type,get:notation,get:compactDisplay,get:minimumIntegerDigits,number:minimumIntegerDigits,get:minimumFractionDigits,get:maximumFractionDigits,get:minimumSignificantDigits,get:maximumSignificantDigits,get:roundingIncrement,number:roundingIncrement,get:roundingMode,text:roundingMode,get:roundingPriority,text:roundingPriority,get:trailingZeroDisplay,text:trailingZeroDisplay,number:minimumFractionDigits,number:maximumFractionDigits', 'delayed bounds conversion');
var marker = {}, touched = 0, ignored = {valueOf() { touched++; throw marker; }};
var significant = new Intl.PluralRules('en', {minimumFractionDigits:ignored, maximumFractionDigits:ignored, maximumSignificantDigits:3});
same(touched, 0, 'ignored bounds not coerced'); same(significant.select(1), 'one', 'significant digits accepted');
abrupt(marker, function () { new Intl.PluralRules('en', {get minimumFractionDigits() {throw marker;},maximumSignificantDigits:3}); }, 'ignored getter still observed');
var modeRead = false;
throws(RangeError, function () { new Intl.PluralRules('en', {roundingIncrement:3,get roundingMode() {modeRead=true; return 'halfEven';}}); }, 'invalid increment');
same(modeRead, false, 'increment rejection before mode');
throws(TypeError, function () { new Intl.PluralRules('en', {roundingIncrement:2,maximumSignificantDigits:3}); }, 'increment significant conflict');
throws(TypeError, function () { new Intl.PluralRules('en', {roundingIncrement:2,roundingPriority:'morePrecision'}); }, 'increment precision conflict');
throws(RangeError, function () { new Intl.PluralRules('en', {roundingIncrement:2,minimumFractionDigits:0,maximumFractionDigits:1}); }, 'increment unequal fraction digits');
var increment = new Intl.PluralRules('en', {type:'ordinal',roundingIncrement:2}), resolved = increment.resolvedOptions();
same(resolved.minimumFractionDigits, 0, 'increment default min'); same(resolved.maximumFractionDigits, 0, 'increment default max'); same(increment.select('1'), 'two', 'increment exact half expansion');

print("ok digit_lifecycle");
262;
