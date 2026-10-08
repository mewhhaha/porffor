function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var formatter = new Intl.DurationFormat('en');
for (var bad of [0.5, NaN, Infinity, -Infinity, Symbol('number'), 1n]) {
  var seen = [];
  var bag = new Proxy({}, { get(t, key) { seen.push(key); if (key === 'days') return bad; throw new Error('next read'); } });
  throws(typeof bad === 'symbol' || typeof bad === 'bigint' ? TypeError : RangeError,
    function () { formatter.format(bag); }, 'field conversion rejects');
  check(seen.join(',') === 'days', 'invalid field stops immediately');
}
var poison = new Proxy({}, { get() { throw new Error('brand must precede Get'); } });
for (var method of [Intl.DurationFormat.prototype.format, Intl.DurationFormat.prototype.formatToParts]) {
  throws(TypeError, function () { method.call({}, poison); }, 'receiver checked before input');
  throws(TypeError, function () { method.call(Object.create(Intl.DurationFormat.prototype), poison); }, 'prototype inheritance does not mint brand');
  throws(TypeError, function () { method.call(new Proxy(formatter, {}), poison); }, 'proxy does not inherit brand');
}
throws(RangeError, function () { formatter.format('bad string'); }, 'invalid ISO duration string');
for (var primitive of [undefined,null,true,1,1n,Symbol('duration')]) {
  throws(TypeError, function () { formatter.format(primitive); }, 'nonstring primitive duration');
}
print('ok abrupt_integrality_stops_next_get'); 262;
