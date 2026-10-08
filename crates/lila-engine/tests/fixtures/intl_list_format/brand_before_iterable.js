function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
var lf = new Intl.ListFormat('en-US');
var touches = 0;
var iterable = new Proxy({}, {get() { touches++; throw new Error('brand must be checked first'); }});
var spoof = {locale: 'en-US', type: 'conjunction', style: 'long', $InitializedListFormat: true};
for (var bad of [undefined, null, 7, spoof, Intl.ListFormat.prototype, new Proxy(lf, {})]) {
  for (var method of ['format', 'formatToParts', 'resolvedOptions']) {
    var caught = undefined;
    try { Intl.ListFormat.prototype[method].call(bad, iterable); } catch (e) { caught = e; }
    check(caught && caught.constructor === TypeError, 'real brand ' + method);
  }
}
same(touches, 0, 'all brands precede iterator Get');
same(lf.format(undefined), '', 'valid undefined empty list');
same(lf.formatToParts(undefined).length, 0, 'valid undefined empty parts');
print('ok brand_before_iterable');
262;
