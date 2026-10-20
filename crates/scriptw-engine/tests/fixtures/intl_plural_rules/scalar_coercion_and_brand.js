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

var rules = new Intl.PluralRules('en', {type:'ordinal'}), log = [], marker = {};
same(rules.select({[Symbol.toPrimitive](hint) { log.push(hint); return 3n; }}), 'few', 'primitive BigInt'); same(log.join(','), 'number', 'number hint');
log = [];
same(rules.select({valueOf() {log.push('valueOf'); return {};},toString() {log.push('toString'); return '2';}}), 'two', 'numeric string fallback');
same(log.join(','), 'valueOf,toString', 'ordinary primitive order');
abrupt(marker, function () { rules.select({valueOf() {throw marker;}}); }, 'scalar abrupt identity');
throws(TypeError, function () { rules.select(Symbol('number')); }, 'symbol rejects');
var touched = false, input = {valueOf() {touched=true; return 1;}};
for (var receiver of [Intl.PluralRules.prototype, {}, {select:rules.select}, new Proxy(rules,{get() {throw marker;}})]) {
  throws(TypeError, function () { Intl.PluralRules.prototype.select.call(receiver, input); }, 'private brand');
  same(touched, false, 'brand precedes coercion');
}
var saved = Intl.NumberFormat;
try { Intl.NumberFormat = function () {throw marker;}; same(rules.select(3), 'few', 'no public NumberFormat dependency'); }
finally { Intl.NumberFormat = saved; }

print("ok scalar_coercion_and_brand");
262;
