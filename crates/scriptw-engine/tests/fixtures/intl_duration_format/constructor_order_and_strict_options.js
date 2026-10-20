function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var seen = [], target = new Proxy(function () {}, { get(t, key, receiver) {
  if (key === 'prototype') seen.push('prototype'); return Reflect.get(t, key, receiver);
}});
var locales = { get length() { seen.push('length'); return 1; }, get 0() { seen.push('locale'); return 'en'; } };
var options = { get localeMatcher() { seen.push('localeMatcher'); return 'lookup'; },
  get numberingSystem() { seen.push('numberingSystem'); return 'latn'; },
  get style() { seen.push('style'); return 'short'; } };
Reflect.construct(Intl.DurationFormat, [locales, options], target);
check(seen.join(',') === 'prototype,length,locale,localeMatcher,numberingSystem,style', 'NewTarget before locales/options');
var poison = new Proxy({}, { get() { throw new Error('unexpected options read'); } });
throws(TypeError, function () { Intl.DurationFormat(poison, poison); }, 'new required before argument reads');
for (var primitive of [null, true, 'short', 1, Symbol('option'), 1n]) {
  throws(TypeError, function () { new Intl.DurationFormat('en', primitive); }, 'strict GetOptionsObject');
}
var inherited = 0;
Object.defineProperty(Object.prototype, 'years', { configurable: true, get() { inherited++; return 'long'; } });
try { new Intl.DurationFormat('en', undefined); } finally { delete Object.prototype.years; }
check(inherited === 0, 'undefined options is a null-prototype object');
check(new Intl.DurationFormat('en', function () {}).resolvedOptions().style === 'short', 'function options accepted');
print('ok constructor_order_and_strict_options'); 262;
