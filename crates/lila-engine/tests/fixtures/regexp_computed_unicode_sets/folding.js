var kelvin = codePoint(0x212a);
var longS = codePoint(0x17f);
for (var row of [
  ['[[K]--[k]]', [], ['K', 'k', kelvin]],
  ['[[K]&&[k]]', ['K', 'k', kelvin], ['A', 'a']],
  ['[[\\p{ASCII}]--[K]]', ['A', 'a', 'S', 's', longS], ['K', 'k', kelvin]],
  ['[[^a]&&[A]]', [], ['a', 'A']],
  ['[^[a]]', ['K', 'k', kelvin], ['a', 'A']],
  ['[[\\P{Lowercase_Letter}]&&[a]]', [], ['a', 'A']],
  ['[[\\u{10400}]&&[\\u{10428}]]', [codePoint(0x10400), codePoint(0x10428)], ['A', codePoint(0x10401)]],
  ['[[\\u{10400}]--[\\u{10428}]]', [], [codePoint(0x10400), codePoint(0x10428)]]
]) checkSet(row[0], 'iv', row[1], row[2]);
checkSet('[[K]--[k]]', 'v', ['K'], ['k', kelvin]);
checkSet('[[K]&&[k]]', 'v', [], ['K', 'k', kelvin]);
var scoped = computed('^(?i:[[A-Z]--[Q]])(?-i:[[A-Z]--[Q]])$', 'v');
require(scoped.test('kK') && scoped.test(kelvin + 'K') && scoped.test('aA'), 'scoped operand closure uses local ignoreCase');
require(!scoped.test('qA') && !scoped.test('Kk') && !scoped.test('AQ'), 'scoped subtraction and restored sensitive sibling');
var unicodeComplement = computed('^\\P{Lowercase_Letter}$', 'iu');
var setsComplement = computed('^[[\\P{Lowercase_Letter}]&&[A]]$', 'iv');
require(unicodeComplement.test('a') && !setsComplement.test('a') && !setsComplement.test('A'), 'u and v property complement order remains distinct');
var restored = computed('^(?i:[[K]&&[k]])[[K]--[k]]$', 'v');
require(restored.test(kelvin + 'K') && !restored.test('Kk'), 'nested class completion restores following modifier context');
print('regexp-computed-sets-folding:ok');
262;
