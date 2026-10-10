function segmenterAssert(value, label) { if (!value) throw 'projected Segmenter ' + label; }
function indexes(locale, granularity, text) {
  return Array.from(new Intl.Segmenter(locale, {granularity: granularity}).segment(text), function(row) { return row.index; }).join(',');
}
segmenterAssert(Intl.Segmenter.supportedLocalesOf(['sv', 'el', 'fr', 'sv-FI']).join(',') === 'sv,el,sv-FI', 'public catalogue');
segmenterAssert(new Intl.Segmenter('fr').resolvedOptions().locale === 'en-US', 'excluded locale fallback');
segmenterAssert(new Intl.Segmenter('el-GR').resolvedOptions().locale === 'el', 'lookup parent');
segmenterAssert(indexes('sv', 'word', 'hello:world') === '0', 'selected Swedish override');
segmenterAssert(indexes('en-US', 'word', 'hello:world') === '0,5,6', 'fallback word rules');
segmenterAssert(indexes('el', 'sentence', 'hello; world') === '0,7', 'selected Greek override');
segmenterAssert(indexes('en-US', 'sentence', 'hello; world') === '0', 'fallback sentence rules');
segmenterAssert(indexes('en-US', 'word', 'ภาษาไทยภาษาไทย') === '0,4,7,11', 'Thai model outside locale domain');
segmenterAssert(indexes('el', 'word', 'မြန်မာစာမြန်မာစာမြန်မာစာ') === '0,8,16,22', 'Burmese model outside locale domain');
segmenterAssert(indexes('sv', 'word', 'うなぎうなじ') === '0,3', 'CJK dictionary outside locale domain');
segmenterAssert(indexes('en-US', 'grapheme', '\ud800\ud83d\ude00\udc00a') === '0,1,3,4', 'original UTF16');
var retained = new Intl.Segmenter('el', {granularity:'sentence'}).segment('hello; world');
segmenterAssert(retained.containing(8).segment === 'world', 'containing retained partition');
segmenterAssert(retained.containing(12) === undefined, 'end of retained partition');
var first = Array.from(retained), second = Array.from(retained);
segmenterAssert(first.length === 2 && second.length === 2 && first[0] !== second[0], 'independent iterators');
print('intl-segmenter-projection:ok');
