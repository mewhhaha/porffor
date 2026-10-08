for (var mode of [117, 118]) {
  var positive = property('Lowercase_Letter', 112, [105, mode]);
  var complement = property('Lowercase_Letter', 80, [105, mode]);
  require(positive.test('a') && positive.test('A'), 'positive property receives Unicode case folding');
  require(complement.test('a') === (mode === 117) && complement.test('A') === (mode === 117), 'P folds after complement in u and before complement in v');
  require(complement.test('1'), 'P retains nonletters in either mode');
  require(complement.test(codePoint(0x212a)) === (mode === 117), 'operand complement order includes non-ASCII folds');
  require(complement.test(codePoint(0x10428)) === (mode === 117), 'operand complement order includes astral folds');
  var negativeClass = computed([94, 91, 94].concat(propertyUnits('Lowercase_Letter', 112), [93, 36]), [105, mode]);
  require(!negativeClass.test('a') && !negativeClass.test('A') && negativeClass.test('1'), 'bracket negation complements the completed case-closed class');
  var complementClass = computed([94, 91].concat(propertyUnits('Lowercase_Letter', 80), [93, 36]), [105, mode]);
  require(complementClass.test('a') === (mode === 117), 'P retains operand-local polarity inside a class');
  var union = computed([94, 91].concat(propertyUnits('Lowercase_Letter', 80), [98, 93, 36]), [105, mode]);
  require(union.test('b') && union.test('B') && union.test('1'), 'positive literal unions with complemented property');
  require(union.test('a') === (mode === 117), 'class union cannot repair or leak property complement ordering');
  var negativeUnion = computed([94, 91, 94].concat(propertyUnits('Lowercase_Letter', 112), [48, 93, 36]), [105, mode]);
  require(!negativeUnion.test('A') && !negativeUnion.test('0') && negativeUnion.test('1'), 'outer complement follows the completed property/literal union');
  var scoped = computed([94, 40, 63, 105, 58].concat(propertyUnits('Lowercase_Letter', 80), [41, 36]), [mode]);
  require(scoped.test('a') === (mode === 117) && scoped.test('1'), 'scoped i governs the property operand');
  var removed = computed([94, 40, 63, 45, 105, 58].concat(propertyUnits('Lowercase_Letter', 80), [41, 36]), [105, mode]);
  require(!removed.test('a') && removed.test('A'), 'scoped i removal restores raw complement membership');
  var siblings = computed([94, 40, 63, 105, 58].concat(propertyUnits('Lowercase_Letter', 80), [41, 40, 63, 45, 105, 58], propertyUnits('Lowercase_Letter', 112), [41, 36]), [mode]);
  require(siblings.test('1a') && !siblings.test('1A'), 'sibling scoped modifiers retain independent property operands');
  require(siblings.test('Aa') === (mode === 117), 'scoped complement order remains local to its operand');
}
print('regexp-property-folding:ok');
262;
