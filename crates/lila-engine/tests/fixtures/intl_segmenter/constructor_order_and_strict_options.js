function check(value, message) { if (!value) throw new Error(message); }
function throws(type, fn) { try { fn(); } catch (e) { check(e instanceof type, 'wrong exception'); return; } throw new Error('missing exception'); }
var observed = [];
var locales = { get length() { observed.push('locales'); return 1; }, get 0() { observed.push('locale0'); return 'EN'; } };
var options = { get localeMatcher() { observed.push('matcher'); return 'lookup'; }, get granularity() { observed.push('granularity'); return { toString: function () { observed.push('granularityString'); return 'word'; } }; } };
function Target() {}
var prototype = {};
var target = new Proxy(Target, { get: function (t, key) { if (key === 'prototype') { observed.push('prototype'); return prototype; } return t[key]; } });
var result = Reflect.construct(Intl.Segmenter, [locales, options], target);
check(Object.getPrototypeOf(result) === prototype, 'actual NewTarget prototype');
check(observed.join(',') === 'prototype,locales,locale0,matcher,granularity,granularityString', 'constructor observation order');
observed = [];
throws(TypeError, function () { Intl.Segmenter(locales, options); });
check(observed.length === 0, 'NewTarget rejection first');
// The pinned test accidentally calls Intl.Segment. These controls exercise the
// real Segmenter constructor for every forbidden primitive options category.
var invalid = [null, true, false, 'word', 7, Symbol('options'), 123456789n];
for (var i = 0; i < invalid.length; ++i) {
  (function (option) { throws(TypeError, function () { return new Intl.Segmenter('en', option); }); })(invalid[i]);
}
throws(RangeError, function () { new Intl.Segmenter('en', { granularity: 'Word' }); });
throws(RangeError, function () { new Intl.Segmenter('en', { localeMatcher: 'LOOKUP' }); });
var inherited = 0;
Object.defineProperty(Object.prototype, 'granularity', { configurable: true, get: function () { inherited++; return 'word'; } });
try { check(new Intl.Segmenter('en').resolvedOptions().granularity === 'grapheme', 'undefined options null prototype'); }
finally { delete Object.prototype.granularity; }
check(inherited === 0, 'undefined options do not inherit');
print('ok constructor_order_and_strict_options'); 262;
