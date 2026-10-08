function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
var lf = new Intl.ListFormat('en-US');
var first = lf.formatToParts(['A', 'B']);
check(Array.isArray(first), 'parts array'); same(Object.getPrototypeOf(first), Array.prototype, 'array prototype');
var length = Object.getOwnPropertyDescriptor(first, 'length'); same(length.value, 3, 'length value'); same(length.writable, true, 'length writable'); same(length.enumerable, false, 'length nonenumerable'); same(length.configurable, false, 'length nonconfigurable');
same(Object.keys(first).join(','), '0,1,2', 'index order');
for (var i = 0; i < first.length; i++) {
  var part = first[i]; same(Object.getPrototypeOf(part), Object.prototype, 'ordinary part'); same(Object.keys(part).join(','), 'type,value', 'part creation order');
  for (var key of ['type', 'value']) { var d = Object.getOwnPropertyDescriptor(part, key); same(d.writable, true, 'part writable'); same(d.enumerable, true, 'part enumerable'); same(d.configurable, true, 'part configurable'); }
  var index = Object.getOwnPropertyDescriptor(first, String(i)); same(index.writable, true, 'index writable'); same(index.enumerable, true, 'index enumerable'); same(index.configurable, true, 'index configurable');
}
first[0].value = 'changed'; first[1].type = 'changed'; first.length = 0;
var second = lf.formatToParts(['A', 'B']);
check(first !== second, 'fresh array'); same(second[0].value, 'A', 'fresh element'); same(second[1].type, 'literal', 'fresh literal'); same(lf.format(['A', 'B']), 'A and B', 'private config untouched');
var third = lf.formatToParts(['A', 'B']);
for (var i = 0; i < second.length; i++) check(second[i] !== third[i], 'fresh part objects');
// These archived CLDR templates put literal text after the final element.
// A complete partition therefore need not finish with an element token.
for (var row of [
  ['ml', 'conjunction', ['A', 'B', 'C'], 'A, B, C എന്നിവ', ['A', ', ', 'B', ', ', 'C', ' എന്നിവ']],
  ['mi', 'disjunction', ['A', 'B'], 'A, B rānei', ['A', ', ', 'B', ' rānei']],
  ['mi', 'disjunction', ['A', 'B', 'C', 'D'], 'A, B, C, D rānei', ['A', ', ', 'B', ', ', 'C', ', ', 'D', ' rānei']],
  ['mi', 'disjunction', ['A', ''], 'A,  rānei', ['A', ', ', '', ' rānei']]
]) {
  var formatter = new Intl.ListFormat(row[0], {type: row[1], style: 'long'});
  same(formatter.format(row[2]), row[3], 'exact suffix rendering');
  var suffixParts = formatter.formatToParts(row[2]);
  same(suffixParts.length, row[4].length, 'suffix partition length');
  for (var i = 0; i < suffixParts.length; i++) {
    same(suffixParts[i].type, i % 2 === 0 ? 'element' : 'literal', 'suffix partition kind');
    same(suffixParts[i].value, row[4][i], 'suffix exact token');
  }
  same(suffixParts[suffixParts.length - 1].type, 'literal', 'final suffix retained');
  same(suffixParts.map(function(p) { return p.value; }).join(''), row[3], 'suffix format/parts agreement');
}
print('ok parts_property_identity');
262;
