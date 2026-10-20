function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var log = [];
var locale = {toString() { log.push('locale.toString'); return 'en-US'; }};
var locales = new Proxy({length: 1, 0: locale}, {get(t, k) { log.push('locales.get ' + String(k)); return Reflect.get(t, k); }, has(t, k) { log.push('locales.has ' + String(k)); return Reflect.has(t, k); }});
var words = {usage: 'sort', localeMatcher: 'lookup', collation: 'emoji', caseFirst: 'upper', sensitivity: 'case'};
var truthy = {[Symbol.toPrimitive]() { throw new Error('Boolean coerced'); }, valueOf() { throw new Error('Boolean valueOf'); }};
var options = new Proxy({}, {get(t, k) { log.push('options.get ' + String(k)); if (k === 'numeric' || k === 'ignorePunctuation') return truthy; return {toString() { log.push(String(k) + '.toString'); return words[k]; }}; }});
function Target() {} var proto = {inherited: 42};
var nt = new Proxy(Target, {get(t, k) { if (k === 'prototype') { log.push('newTarget.prototype'); return proto; } return Reflect.get(t, k); }});
var c = Reflect.construct(Intl.Collator, [locales, options], nt);
same(log.join('|'), 'newTarget.prototype|locales.get length|locales.has 0|locales.get 0|locale.toString|options.get usage|usage.toString|options.get localeMatcher|localeMatcher.toString|options.get collation|collation.toString|options.get numeric|options.get caseFirst|caseFirst.toString|options.get sensitivity|sensitivity.toString|options.get ignorePunctuation', 'complete order');
same(Object.getPrototypeOf(c), proto, 'retained tagged prototype'); same(c.inherited, 42, 'prototype property');
var o = Intl.Collator.prototype.resolvedOptions.call(c); same(o.numeric, true, 'Boolean numeric'); same(o.ignorePunctuation, true, 'Boolean ignore'); same(o.sensitivity, 'case', 'final sensitivity');
print('ok constructor_observation');
262;
