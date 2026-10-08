function expression(units, flags) { return new RegExp(fromUnits(units), fromUnits(flags)); }
function syntax(units, flags) {
  var caught = false;
  try { expression(units, flags); }
  catch (error) { caught = error instanceof SyntaxError; }
  require(caught, 'computed invalid grammar is SyntaxError');
}
for (var mode = 117; mode <= 118; mode++) {
  syntax([92, 49], [mode]);
  syntax([40, 97, 41, 92, 50], [mode]);
  syntax([40, 97, 41, 92, 49, 48], [mode]);
  syntax([40, 63, 61, 97, 41, 63], [mode]);
  syntax([40, 63, 33, 97, 41, 42], [mode]);
  syntax([91, 92, 100, 45, 97, 93], [mode]);
  syntax([91, 97, 45, 92, 115, 93], [mode]);
  require(expression([94, 92, 49, 40, 97, 41, 36], [mode]).test('a'), 'forward reference uses complete capture census');
  require(expression([94, 40, 97, 41, 92, 49, 36], [mode]).test('aa'), 'valid decimal reference remains supported');
  var retained = expression([120], [103]);
  retained.lastIndex = 7;
  var caught = false;
  try { retained.compile(fromUnits([40, 63, 61, 97, 41, 63]), fromUnits([mode])); }
  catch (error) { caught = error instanceof SyntaxError; }
  require(caught && retained.source === 'x' && retained.flags === 'g' && retained.lastIndex === 7, 'mode-sensitive failure preserves receiver transaction');
}
require(expression([92, 49], []).test(fromUnits([1])), 'legacy missing reference retains octal fallback');
require(expression([40, 63, 61, 97, 41, 63], []).test(''), 'Annex B lookahead quantification remains');
for (var unit of [40, 41, 123, 125, 47, 124, 45]) {
  syntax([91, unit, 93], [118]);
}
for (var unit of [40, 41, 91, 93, 123, 125, 47, 124, 45, 92]) {
  require(expression([91, 92, unit, 93], [118]).test(fromUnits([unit])), 'v syntax character is admitted when escaped');
}
for (var unit of [38, 45, 33, 35, 37, 44, 58, 59, 60, 61, 62, 64, 96, 126]) {
  require(expression([91, 92, unit, 93], [118]).test(fromUnits([unit])), 'v reserved punctuator has an identity escape');
}
syntax([91, 92, 95, 93], [118]);
for (var unit of [38, 33, 35, 36, 37, 42, 43, 44, 46, 58, 59, 60, 61, 62, 63, 64, 94, 96, 126]) {
  syntax([91, 97, unit, unit, 93], [118]);
  require(expression([91, 97, 92, unit, unit, 93], [118]).test(fromUnits([unit])), 'escaped first punctuator removes reserved raw pair');
}
require(expression([91, 33, 92, 33, 93], [118]).test('!'), 'escaped second punctuator removes reserved raw pair');
require(expression([91, 94, 94, 93], [118]).test('a') && !expression([91, 94, 94, 93], [118]).test('^'), 'initial caret is negation rather than first reserved-pair character');
require(expression([91, 40, 41, 93], [117]).test('('), 'ordinary u class punctuation is retained');
require(expression([91, 33, 33, 93], [117]).test('!'), 'u class has no v reserved-double rule');
require(expression([91, 92, 95, 93], []).test('_'), 'legacy identity escape is retained');
print('ok');
262;
