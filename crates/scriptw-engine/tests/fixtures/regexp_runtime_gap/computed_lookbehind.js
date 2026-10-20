// Compute every pattern at run time; no prepared-source or literal-program path.
function computed(pattern, flags) {
  var units = [];
  for (var i = 0; i < pattern.length; i++) units.push(pattern.charCodeAt(i));
  return new RegExp(fromUnits(units), flags);
}

var result = computed('(?<=([ab]+)([bc]+))$', 'd').exec('abc');
require(result[0] === '' && result.index === 3, 'lookbehind does not consume');
require(result[1] === 'a' && result[2] === 'bc', 'reverse greedy capture order');
require(result.indices[1][0] === 0 && result.indices[1][1] === 1 &&
        result.indices[2][0] === 1 && result.indices[2][1] === 3, 'reverse capture endpoints');

result = computed('(?<=(a|ba))c').exec('bac');
require(result[1] === 'a', 'alternatives retain source priority');
result = computed('(?<=\\1(a))b').exec('aab');
require(result[0] === 'b' && result.index === 2 && result[1] === 'a', 'reverse numbered reference');
result = computed('(?<=(a)\\1)b').exec('ab');
require(result[0] === 'b' && result[1] === 'a', 'not yet participating reverse reference is empty');
result = computed('(?<=\\k<x>(?<x>a))b', 'd').exec('aab');
require(result.groups.x === 'a' && result.indices.groups.x[0] === 1 &&
        result.indices.groups.x[1] === 2, 'reverse named reference and indices');

require(computed('(?<=a(?=b))b').exec('ab')[0] === 'b', 'nested lookahead restores reverse direction');
require(computed('(?=(?<=a)b)b').exec('ab')[0] === 'b', 'nested lookbehind restores forward direction');
result = computed('(?<!([ab]))c').exec('cc');
require(result.index === 0 && result[1] === undefined, 'negative assertion restores captures');
require(computed('(?<!a)b').exec('ab') === null &&
        computed('(?<!a)b').exec('xb')[0] === 'b', 'negative lookbehind polarity');

require(computed('(?<=^a)b').exec('ab')[0] === 'b' &&
        computed('(?<=^a)b').exec('xab') === null, 'anchors keep their input orientation');
require(computed('(?<=^a)b', 'm').exec('x\nab').index === 3, 'multiline anchor');
require(computed('(?<=a(?i:b))c').exec('aBc')[0] === 'c', 'scoped modifier in reverse body');
require(computed('(?<=\\uD83D\\uDE00)x').exec('\uD83D\uDE00x').index === 2, 'legacy surrogate pair');
require(computed('(?<=(a?)*b)c').exec('abc')[1] === 'a', 'nullable quantifier progress in reverse');
require(computed('(?<=a{4294967296})b').exec('b') === null &&
        computed('(?<!a{4294967296})b').exec('b')[0] === 'b', 'huge-bound empty assertion restores parent direction');

var sticky = computed('(?<=a)b', 'gy');
sticky.lastIndex = 1;
require(sticky.exec('ab')[0] === 'b' && sticky.lastIndex === 2, 'sticky position and lastIndex');
require(sticky.exec('ab') === null && sticky.lastIndex === 0, 'failed sticky search resets lastIndex');

var expression = /z/g;
expression.lastIndex = 9;
expression.compile(fromUnits([40,63,60,61,97,41,98]));
require(expression.exec('ab')[0] === 'b' && expression.lastIndex === 0, 'recompile publishes complete program');
for (var source of ['(?<=a)+', '(?<!a)?', '(?<=a){2}', '(?<=a']) {
  var syntaxError = false;
  try { computed(source); } catch (error) { syntaxError = error instanceof SyntaxError; }
  require(syntaxError, 'invalid assertion syntax: ' + source);
}
print('ok');
262;
