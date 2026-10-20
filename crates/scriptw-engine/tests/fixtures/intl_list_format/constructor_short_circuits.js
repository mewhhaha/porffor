function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
function throws(C, body, label) { var caught = undefined; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C, label); }
var touches = 0;
var poison = new Proxy({}, {get() { touches++; throw new Error('unexpected read'); }});
throws(TypeError, function() { Intl.ListFormat(poison, poison); }, 'requires NewTarget before reads');
same(touches, 0, 'plain call observes nothing');
var marker = {};
function T() {}
var nt = new Proxy(T, {get(t, k) { if (k === 'prototype') throw marker; return Reflect.get(t, k); }});
var caught = undefined;
try { Reflect.construct(Intl.ListFormat, [poison, poison], nt); } catch (e) { caught = e; }
same(caught, marker, 'NewTarget prototype abrupt retained');
same(touches, 0, 'prototype abrupt before locales/options');
caught = undefined;
try { new Intl.ListFormat([{toString() { throw marker; }}], poison); } catch (e) { caught = e; }
same(caught, marker, 'locale coercion abrupt retained');
same(touches, 0, 'locale abrupt before options');
for (var value of [null, true, false, 7, 'short', Symbol('options'), 1n]) {
  var log = [];
  var locale = {toString() { log.push('locale'); return 'en-US'; }};
  throws(TypeError, function() { new Intl.ListFormat([locale], value); }, 'constructor strict options');
  same(log.join(','), 'locale', 'locales before strict options rejection');
}
for (var bad of ['localeMatcher', 'type', 'style']) {
  var reads = [];
  var values = {localeMatcher: 'lookup', type: 'conjunction', style: 'long'};
  var opts = new Proxy({}, {get(t, k) { reads.push(k); return k === bad ? 'invalid' : values[k]; }});
  throws(RangeError, function() { new Intl.ListFormat('en-US', opts); }, 'invalid closed option ' + bad);
  same(reads.join(','), bad === 'localeMatcher' ? 'localeMatcher' : bad === 'type' ? 'localeMatcher,type' : 'localeMatcher,type,style', 'later option not read');
}
for (var abrupt of ['localeMatcher', 'type', 'style']) {
  var gets = [];
  var opts = new Proxy({}, {get(t, k) { gets.push(k); if (k === abrupt) throw marker; return k === 'localeMatcher' ? 'lookup' : 'conjunction'; }});
  caught = undefined;
  try { new Intl.ListFormat('en-US', opts); } catch (e) { caught = e; }
  same(caught, marker, 'getter abrupt retained ' + abrupt);
  same(gets.join(','), abrupt === 'localeMatcher' ? 'localeMatcher' : abrupt === 'type' ? 'localeMatcher,type' : 'localeMatcher,type,style', 'getter short circuit');
}
print('ok constructor_short_circuits');
262;
