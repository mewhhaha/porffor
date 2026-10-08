function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
var log = [];
var locale = {toString() { log.push('locale.toString'); return 'en-US'; }};
var locales = new Proxy({length: 1, 0: locale}, {
  get(target, key) { log.push('locales.get ' + String(key)); return Reflect.get(target, key); },
  has(target, key) { log.push('locales.has ' + String(key)); return Reflect.has(target, key); }
});
var values = {localeMatcher: 'lookup', type: 'disjunction', style: 'short'};
var options = new Proxy({}, {get(target, key) {
  log.push('options.get ' + String(key));
  return {toString() { log.push(String(key) + '.toString'); return values[key]; }};
}});
function Target() {}
var proto = {inherited: 42};
var nt = new Proxy(Target, {get(target, key) {
  if (key === 'prototype') { log.push('newTarget.prototype'); return proto; }
  return Reflect.get(target, key);
}});
var lf = Reflect.construct(Intl.ListFormat, [locales, options], nt);
same(log.join('|'), 'newTarget.prototype|locales.get length|locales.has 0|locales.get 0|locale.toString|options.get localeMatcher|localeMatcher.toString|options.get type|type.toString|options.get style|style.toString', 'complete observation order');
same(Object.getPrototypeOf(lf), proto, 'retained prototype');
same(lf.inherited, 42, 'inherited property');
same(Intl.ListFormat.prototype.format.call(lf, ['A', 'B']), 'A or B', 'checked configuration attached');
same(Intl.ListFormat.prototype.resolvedOptions.call(lf).style, 'short', 'observed style retained');
print('ok constructor_observation');
262;
