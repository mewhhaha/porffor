function expression(units, flags) { return new RegExp(fromUnits(units), fromUnits(flags)); }
var face = fromUnits([0xd83d, 0xde00]);
var nextFace = fromUnits([0xd83d, 0xde01]);
var outsideRange = fromUnits([0xd83d, 0xde02]);
for (var mode = 117; mode <= 118; mode++) {
  var zero = expression([94, 92, 117, 123, 48, 125, 36], [mode]);
  require(zero.test(fromUnits([0])) && !zero.test('0'), 'braced zero is a null character');
  require(expression([94, 92, 117, 123, 48, 48, 48, 48, 48, 49, 125, 36], [mode]).test(fromUnits([1])), 'leading zeros do not change the code-point value');
  var repeated = expression([94, 92, 117, 123, 49, 102, 54, 48, 48, 125, 123, 50, 125, 36], [100, 103, mode]);
  var match = repeated.exec(face + face);
  require(match && match[0] === face + face && match.indices[0][1] === 4 && repeated.lastIndex === 4, 'braced astral escape is one quantified atom with UTF-16 indices');
  require(expression([94, 92, 117, 123, 48, 48, 49, 70, 54, 48, 48, 125, 36], [mode]).test(face), 'mixed uppercase hex and leading zeros decode the same scalar');
  require(expression([94, 92, 117, 123, 49, 48, 70, 70, 70, 70, 125, 36], [mode]).test(fromUnits([0xdbff, 0xdfff])), 'maximum code point is admitted');
  require(expression([94, 92, 117, 123, 68, 56, 48, 48, 125, 36], [mode]).test(fromUnits([0xd800])), 'braced lead surrogate remains a legal lone character');
  require(expression([94, 92, 117, 123, 68, 70, 70, 70, 125, 36], [mode]).test(fromUnits([0xdfff])), 'braced trail surrogate remains a legal lone character');
  for (var units of [
    [94, 92, 117, 123, 68, 56, 51, 68, 125, 92, 117, 123, 68, 69, 48, 48, 125, 36],
    [94, 92, 117, 123, 68, 56, 51, 68, 125, 92, 117, 68, 69, 48, 48, 36],
    [94, 92, 117, 68, 56, 51, 68, 92, 117, 123, 68, 69, 48, 48, 125, 36],
    [94, 0xd83d, 92, 117, 123, 68, 69, 48, 48, 125, 36],
    [94, 92, 117, 123, 68, 56, 51, 68, 125, 0xde00, 36]
  ]) {
    require(!expression(units, [mode]).test(face), 'braced surrogate halves do not pair with braced fixed or raw halves');
  }
  for (var units of [
    [94, 91, 92, 117, 123, 68, 56, 51, 68, 125, 92, 117, 123, 68, 69, 48, 48, 125, 93, 36],
    [94, 91, 92, 117, 123, 68, 56, 51, 68, 125, 92, 117, 68, 69, 48, 48, 93, 36],
    [94, 91, 92, 117, 68, 56, 51, 68, 92, 117, 123, 68, 69, 48, 48, 125, 93, 36],
    [94, 91, 0xd83d, 92, 117, 123, 68, 69, 48, 48, 125, 93, 36],
    [94, 91, 92, 117, 123, 68, 56, 51, 68, 125, 0xde00, 93, 36]
  ]) {
    require(!expression(units, [mode]).test(face), 'braced class halves retain their separate decoded characters');
  }
  require(expression([94, 91, 92, 117, 123, 48, 125, 93, 36], [mode]).test(fromUnits([0])), 'braced class includes null');
  require(expression([94, 91, 92, 117, 123, 49, 48, 70, 70, 70, 70, 125, 93, 36], [mode]).test(fromUnits([0xdbff, 0xdfff])), 'braced class includes the maximum code point');
  var range = expression([94, 91, 92, 117, 123, 49, 70, 54, 48, 48, 125, 45, 92, 117, 123, 49, 70, 54, 48, 49, 125, 93, 36], [mode]);
  require(range.test(face) && range.test(nextFace) && !range.test(outsideRange), 'braced class endpoints form a scalar range');
  require(expression([94, 91, 94, 92, 117, 123, 49, 70, 54, 48, 48, 125, 93, 36], [mode]).test(nextFace), 'braced class negation uses the complete character domain');
  require(expression([94, 91, 92, 117, 123, 50, 49, 50, 65, 125, 93, 36], [105, mode]).test('k'), 'braced class participates in Unicode case closure');
  for (var digitsAndUnit of [[50, 68, 45], [50, 56, 40], [53, 68, 93], [53, 67, 92], [53, 69, 94]]) {
    require(expression([94, 91, 92, 117, 123, digitsAndUnit[0], digitsAndUnit[1], 125, 93, 36], [mode]).test(fromUnits([digitsAndUnit[2]])), 'escaped syntax value remains a class character');
  }
  require(expression([94, 91, 92, 117, 123, 50, 54, 125, 92, 117, 123, 50, 54, 125, 93, 36], [mode]).test('&'), 'decoded punctuators do not form a reserved raw pair');
  var named = expression([40, 63, 60, 92, 117, 123, 54, 49, 125, 62, 120, 41], [100, mode]).exec('x');
  require(named && named.groups.a === 'x' && named.indices.groups.a[1] === 1, 'existing name decoder validates exposed braced GroupSpecifier');
}
require(expression([94, 92, 117, 123, 50, 125, 36], []).test('uu'), 'Legacy braced-looking digits remain a quantifier on identity u');
require(expression([94, 91, 92, 117, 123, 54, 49, 125, 93, 36], []).test('u'), 'Legacy class retains identity u and literal braces');
print('ok');
262;
