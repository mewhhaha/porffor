for (var row of [
  ['[a[b-d]x]', ['a', 'b', 'c', 'd', 'x'], ['e', 'y', 'z']],
  ['[[a-f]&&[d-z]]', ['d', 'e', 'f'], ['a', 'c', 'g', 'z']],
  ['[[a-f]--[c-e]]', ['a', 'b', 'f'], ['c', 'd', 'e', 'g']],
  ['[[a-g]--[b-d]--[f-g]]', ['a', 'e'], ['b', 'c', 'd', 'f', 'g']],
  ['[[a-g]&&[b-f]&&[d-z]]', ['d', 'e', 'f'], ['a', 'b', 'c', 'g', 'z']],
  ['[[a-z]--[[b-y]--[m-p]]]', ['a', 'm', 'n', 'o', 'p', 'z'], ['b', 'l', 'q', 'y']],
  ['[[]a]', ['a'], ['b', '[', ']']],
  ['[[]]', [], ['a', codePoint(0xd800), codePoint(0x10ffff)]],
  ['[[a]&&[]]', [], ['a', 'b']],
  ['[[a]--[]]', ['a'], ['b']],
  ['[[]--[a]]', [], ['a', 'b']],
  ['[[^]&&[a-z]]', ['a', 'm', 'z'], ['A', '0', codePoint(0x1f600)]],
  ['[^[]]', ['a', codePoint(0xd800), codePoint(0x1f600), codePoint(0x10ffff)], []],
  ['[^^]', ['a', '0'], ['^']],
  ['[^[a-c]x]', ['z', '0', codePoint(0xd800), codePoint(0x1f600)], ['a', 'b', 'c', 'x']],
  ['[a\\-z]', ['a', '-', 'z'], ['m']],
  ['[\\u{1F600}-\\u{1F602}]', [codePoint(0x1f600), codePoint(0x1f601), codePoint(0x1f602)], [codePoint(0x1f603), codePoint(0xd800)]],
  ['[[\\u{D800}]&&\\p{gc=Cs}]', [codePoint(0xd800)], [codePoint(0xd801), codePoint(0x1f600)]],
  ['[\\p{Any}--[\\u{D800}]]', ['a', codePoint(0xdfff), codePoint(0x1f600), codePoint(0x10ffff)], [codePoint(0xd800)]],
  ['[[\\p{ASCII}]&&[\\p{Letter}]]', ['A', 'z'], ['0', codePoint(0x3a9)]],
  ['[[\\p{ASCII}]--[\\p{Lowercase_Letter}]]', ['A', '0'], ['a', codePoint(0x3a9)]],
  ['[[\\p{scx=Hira}]&&[\\p{sc=Zyyy}]]', [codePoint(0x30fc)], ['a', codePoint(0x3042)]]
]) checkSet(row[0], 'v', row[1], row[2]);
var first = computed('^[[a-z]--[b-y]]$', 'v');
var second = computed('^[[a-z]&&[m-p]]$', 'v');
var third = computed('^[[a-c]--[b]][\\p{ASCII}&&[^a-z]]$', 'v');
require(first.test('a') && first.test('z') && !first.test('m'), 'first descriptor survives later nested classes');
require(second.test('m') && second.test('p') && !second.test('a'), 'second descriptor retains its own bitmap');
require(third.test('a5') && third.test('cA') && !third.test('b5') && !third.test('ca'), 'completed class scratch is independent from following class');
var astral = computed('^[\\p{Any}--[a-z]]$', 'dgv');
var match = astral.exec(codePoint(0x1f600));
require(match && match.indices[0][0] === 0 && match.indices[0][1] === 2 && astral.lastIndex === 2, 'nested set publishes UTF16 indices');
require(computed('^[[a]\\]$', 'u').test('a]') && computed('^[[a]\\]$', 'u').test('[]') && !computed('^[[a]\\]$', 'u').test('a'), 'ordinary Unicode class grammar keeps raw opening bracket and escaped closing bracket');
require(computed('^[[a]]$', '').test('a]'), 'Legacy raw closing bracket stays outside the UnicodeSets dispatcher');
require(computed('^[\\c1]$', '').test(fromUnits([17])), 'Legacy class control escape retains Annex B domain');
print('regexp-computed-sets-algebra:ok');
262;
