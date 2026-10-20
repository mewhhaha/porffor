function check(value, message) { if (!value) throw new Error(message); }
var input = 'a\u0301\ud83d\udc69\u200d\ud83d\ude80\ud800X\udc00';
var segments = new Intl.Segmenter('en').segment(input);
var rows = Array.from(segments);
check(rows.length === 5, 'genuine grapheme count');
check(rows.map(function (r) { return r.index; }).join(',') === '0,2,7,8,9', 'code-unit indices');
check(rows.map(function (r) { return r.segment; }).join('') === input, 'original UTF16 reconstruction');
for (var i = 0; i < rows.length; ++i) {
  check(rows[i].input === input && !('isWordLike' in rows[i]), 'grapheme shape');
  check(Object.keys(rows[i]).join(',') === 'segment,index,input', 'property order');
}
check(segments.containing(3).index === 2 && segments.containing(3).segment.length === 5, 'low surrogate in ZWJ');
check(segments.containing(7).segment.charCodeAt(0) === 0xd800, 'isolated high');
check(segments.containing(9).segment.charCodeAt(0) === 0xdc00, 'isolated low');
check(segments.containing(-0.5).index === 0 && segments.containing(NaN).index === 0, 'ToInteger truncation and NaN');
check(segments.containing(8.9).index === 8, 'finite truncation');
check(segments.containing(Infinity) === undefined && segments.containing(-Infinity) === undefined, 'infinite bounds');
check(segments.containing(-1) === undefined && segments.containing(input.length) === undefined, 'finite bounds');
check(new Intl.Segmenter().segment('').containing(0) === undefined, 'empty bounds');
print('ok utf16_graphemes_and_containing'); 262;
