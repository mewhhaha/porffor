function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
var lf = new Intl.ListFormat('en-US');
var elements = ['', '\ud800', '\udc00', '\ud83d\ude00', 'A\0B', '', '\ud800x\udc00'];
var parts = lf.formatToParts(elements);
var found = parts.filter(function(p) { return p.type === 'element'; });
same(found.length, elements.length, 'every original element retained including empties');
for (var i = 0; i < elements.length; i++) {
  same(found[i].value, elements[i], 'original UTF16 identity/order');
  same(found[i].value.length, elements[i].length, 'UTF16 unit length');
  for (var n = 0; n < elements[i].length; n++) same(found[i].value.charCodeAt(n), elements[i].charCodeAt(n), 'exact code unit');
}
same(lf.format(elements), ', \ud800, \udc00, \ud83d\ude00, A\0B, , and \ud800x\udc00', 'UTF16 exact full rendering');
same(parts.map(function(p) { return p.value; }).join(''), lf.format(elements), 'parts rendering exact');
same(lf.formatToParts([]).length, 0, 'empty list has no element');
var singleton = lf.formatToParts(['']);
same(singleton.length, 1, 'zero-width singleton still has element'); same(singleton[0].type, 'element', 'empty singleton kind'); same(singleton[0].value, '', 'empty singleton value');
var pair = lf.formatToParts(['', '']);
same(pair.length, 3, 'two empty elements survive'); same(pair[0].value, '', 'first empty'); same(pair[1].type, 'literal', 'between empties'); same(pair[1].value, ' and ', 'empty pair literal'); same(pair[2].value, '', 'second empty');
var repeated = lf.formatToParts(['same', 'same', 'same']);
same(repeated.filter(function(p) { return p.type === 'element'; }).length, 3, 'duplicate values not deduplicated');
same(lf.format('\ud83d\ude00A'), '\ud83d\ude00 and A', 'String iterator keeps surrogate pair as one element');
same(lf.format('\ud800A'), '\ud800 and A', 'String iterator preserves lone surrogate');
print('ok utf16_and_empty_elements');
262;
