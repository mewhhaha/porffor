checkMembers("[\\q{Ab|xy}&&\\q{aB}]", "iv", ["Ab", "aB", "AB", "ab"], ["xy", "a"]);
checkMembers("[\\q{Ab|xy}--\\q{aB}]", "iv", ["XY", "xy", "xY"], ["ab", "Ab"]);
checkMembers("[\\q{\\u212Ab}&&\\q{kb}]", "iv", ["Kb", "kB", "\u212ab"], ["b", "kbz"]);
checkMembers("(?i:[\\q{Ab}])", "v", ["ab", "AB"], ["xy"]);
checkMembers("(?-i:[\\q{Ab}])", "iv", ["Ab"], ["ab", "AB"]);
var original = codePoints([0x24c2, 0xfe0f]);
var canonical = codePoints([0x24dc, 0xfe0f]);
checkMembers("[\\p{Basic_Emoji}&&\\q{\\u24DC\\uFE0F}]", "iv", [original, canonical], ['a']);
checkMembers("[\\p{Basic_Emoji}--\\q{\\u24DC\\uFE0F}]", "iv", [], [original, canonical]);
checkMembers("[\\p{Basic_Emoji}&&\\q{\\u24DC\\uFE0F}]", "v", [], [original, canonical]);
var reverse = computed("(?<=(?<pair>[\\q{ab|a}]))c", 'dv').exec('abc');
require(reverse && reverse.index === 2 && reverse.groups.pair === 'ab' && reverse.indices.groups.pair[0] === 0 && reverse.indices.groups.pair[1] === 2, 'reverse finite atom uses longest keys and preserves capture bounds');
var reverseAstral = computed("(?<=([\\q{\\u{1F600}a}]))b", 'dv').exec(codePoint(0x1f600) + 'ab');
require(reverseAstral && reverseAstral.index === 3 && reverseAstral.indices[1][0] === 0 && reverseAstral.indices[1][1] === 3, 'reverse string instructions retain complete codepoints and UTF16 bounds');
require(computed("(?<![\\q{ab}])c", 'v').test('ac') && !computed("(?<![\\q{ab}])c", 'v').test('abc'), 'negative reverse finite alternative preserves polarity');
for (var invalid of ["[^\\q{ab}]", "[^\\q{}]", "[^\\q{ab}--\\q{ab}]", "[^\\p{Basic_Emoji}--\\p{Basic_Emoji}]", "[\\q{ab}&&]", "[\\q{ab}](", "[\\q{ab}]\\k<missing>", "\\P{Basic_Emoji}", "\\p{basic_emoji}", "\\q{ab}"]) rejects(invalid, 'v');
rejects("\\p{Basic_Emoji}", 'u');
var receiver = computed('old', 'g');
receiver.lastIndex = 7;
var failed = false;
try { receiver.compile(fromUnits(textUnits("[\\q{ab}]\\k<missing>")), fromUnits([118])); }
catch (error) { failed = error instanceof SyntaxError; }
require(failed && receiver.source === 'old' && receiver.flags === 'g' && receiver.lastIndex === 7, 'late name failure preserves receiver and private descriptor');
receiver.lastIndex = 0;
require(receiver.test('old') && receiver.lastIndex === 3, 'failed finite preparation retains prior matcher');
var installed = "^(?<piece>[\\q{ab|a}])\\k<piece>$";
receiver.compile(fromUnits(textUnits(installed)), fromUnits([100, 118]));
var matching = receiver.exec('abab');
require(matching && matching.groups.piece === 'ab' && matching.indices.groups.piece === matching.indices[1] && receiver.flags === 'dv' && receiver.lastIndex === 0, 'finite recompile publishes descriptor names and capture bounds together');
receiver.lastIndex = 9;
failed = false;
try { receiver.compile(fromUnits(textUnits("[^\\q{ab}--\\q{ab}]")), fromUnits([118])); }
catch (error) { failed = error instanceof SyntaxError; }
require(failed && receiver.source === installed && receiver.flags === 'dv' && receiver.lastIndex === 9, 'static MCS failure cannot publish even when algebra removes every string');
require(receiver.exec('abab').groups.piece === 'ab', 'workspace rollback keeps published finite payload');
print('regexp-computed-strings-folding:ok');
262;
