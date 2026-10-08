function check(value, message) { if (!value) throw new Error(message); }
function rows(locale, input) { return Array.from(new Intl.Segmenter(locale, { granularity: 'word' }).segment(input)); }
check(rows('sv', 'hello:world').map(function (r) { return r.segment; }).join('|') === 'hello:world', 'Swedish colon override');
check(rows('en', 'hello:world').map(function (r) { return r.segment; }).join('|') === 'hello|:|world', 'English colon');
var values = rows('en', 'Hi, 123!');
check(values.map(function (r) { return r.index; }).join(',') === '0,2,3,4,7', 'word indices');
check(values.map(function (r) { return r.isWordLike; }).join(',') === 'true,false,false,true,false', 'preceding native word classification');
check(Object.keys(values[0]).join(',') === 'segment,index,input,isWordLike', 'word property order');
check(rows('en', 'ภาษาไทยภาษาไทย').map(function (r) { return r.index; }).join(',') === '0,4,7,11', 'genuine Thai auto model');
check(rows('ja', 'うなぎうなじ').map(function (r) { return r.index; }).join(',') === '0,3', 'genuine CJK dictionary');
var english = Array.from(new Intl.Segmenter('en', { granularity: 'sentence' }).segment('hello; world'));
var greek = Array.from(new Intl.Segmenter('el', { granularity: 'sentence' }).segment('hello; world'));
check(english.length === 1 && greek.length === 2 && greek[1].index === 7, 'Greek sentence override');
check(!('isWordLike' in greek[0]), 'sentence word-like absence');
print('ok word_status_and_genuine_tailoring'); 262;
