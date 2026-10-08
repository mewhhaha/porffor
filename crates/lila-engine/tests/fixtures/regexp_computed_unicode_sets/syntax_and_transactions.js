for (var source of [
  '[', '[[a]', '[a&&]', '[&&a]', '[a--]', '[--a]', '[a&&&b]',
  '[a&&b--c]', '[a--b&&c]', '[ab&&c]', '[a&&bc]', '[a-b&&c]',
  '[[a]-z]', '[a-[z]]', '[\\d-a]', '[a-\\p{ASCII}]', '[a-\\q{x}]',
  '[a-]', '[-a]', '[()]', '[!!]', '[##]', '[$$]', '[%%]', '[**]',
  '[++]', '[,,]', '[..]', '[::]', '[;;]', '[<<]', '[==]', '[>>]',
  '[??]', '[@@]', '[a^^]', '[``]', '[~~]',
  '[\\c1]', '[\\c_]', '[\\01]', '[\\xG1]', '[\\uGGGG]', '[\\u{110000}]',
  '[\\q]', '[\\q{]', '[\\q{ab]', '[\\q{\\p{ASCII}}]', '[\\q{a-}]',
  '[^\\q{ab}]', '[^\\q{}]', '[^\\q{a|bc}]',
  '[^\\q{ab}--\\q{ab}]', '[^\\q{ab}&&\\q{ab}]',
  '[^\\q{\\u{D800}\\u{DC00}}]',
  '[^\\p{Basic_Emoji}--\\p{Basic_Emoji}]',
  '[[\\q{ab}]&&]', '[[\\q{ab}]--]',
  '[[\\q{ab}]&&[\\p{No_Such_Property}]]',
  '[[\\p{Basic_Emoji}]&&[\\p{No_Such_Property}]]',
  '[\\q{ab}](', '[\\q{ab}]**', '[\\q{ab}]\\k<missing>',
  '[\\p{Basic_Emoji}](', '[\\p{Basic_Emoji}]**', '[\\p{Basic_Emoji}]\\k<missing>',
  '\\q{ab}'
]) rejects(source, 'v');
for (var row of [
  ['[\\cA]', [1]], ['[\\0]', [0]], ['[\\x41]', [65]], ['[\\u0041]', [65]],
  ['[\\&]', [38]], ['[\\u{1F600}]', [0xd83d, 0xde00]]
]) require(computed('^' + row[0] + '$', 'v').test(fromUnits(row[1])), 'strict validated ClassSetCharacter escape');
rejects('[\\c1]', 'u');
rejects('[\\01]', 'u');
var receiver = computed('old', 'g');
receiver.lastIndex = 7;
var failed = false;
try { receiver.compile(fromUnits(textUnits('[[a-z]&&]')), fromUnits([118])); }
catch (error) { failed = error instanceof SyntaxError; }
require(failed && receiver.source === 'old' && receiver.flags === 'g' && receiver.lastIndex === 7, 'unfinished class cannot publish a partial receiver');
receiver.lastIndex = 0;
require(receiver.test('old') && receiver.lastIndex === 3, 'private old descriptor survives workspace rollback');
var installedSource = '^(?<letters>[[a-z]--[q]]+)\\k<letters>$';
receiver.compile(fromUnits(textUnits(installedSource)), fromUnits([100, 118]));
var match = receiver.exec('abab');
require(receiver.lastIndex === 0 && match && match.groups.letters === 'ab' && match.indices.groups.letters === match.indices[1], 'nested set recompile publishes complete named inventory');
require(!receiver.test('qq'), 'installed descriptor retains subtraction semantics');
receiver.lastIndex = 9;
failed = false;
try { receiver.compile(fromUnits(textUnits('[\\q{ab}]\\k<missing>')), fromUnits([118])); }
catch (error) { failed = error instanceof SyntaxError; }
require(failed && receiver.source === installedSource && receiver.flags === 'dv' && receiver.lastIndex === 9, 'unknown name precedes final finite compilation and preserves public slots');
require(receiver.exec('abab').groups.letters === 'ab', 'syntax rollback also retains the recovered private descriptor');
print('regexp-computed-sets-syntax:ok');
262;
