var grinning = codePoints([0x1f600]);
var keycap = codePoints([0x31, 0xfe0f, 0x20e3]);
var toned = codePoints([0x1f44d, 0x1f3fd]);
var flag = codePoints([0x1f1fa, 0x1f1f8]);
var tag = codePoints([0x1f3f4, 0xe0067, 0xe0062, 0xe0065, 0xe006e, 0xe0067, 0xe007f]);
var zwj = codePoints([0x1f469, 0x200d, 0x1f4bb]);
var marked = codePoints([0x24c2, 0xfe0f]);
for (var row of [
  ['Basic_Emoji', [grinning, marked], ['x', keycap, flag]],
  ['Emoji_Keycap_Sequence', [keycap], ['1', grinning, codePoints([0x31, 0x20e3])]],
  ['RGI_Emoji_Modifier_Sequence', [toned], [grinning, codePoints([0x1f44d])]],
  ['RGI_Emoji_Flag_Sequence', [flag], [grinning, codePoints([0x1f1fa])]],
  ['RGI_Emoji_Tag_Sequence', [tag], [grinning, tag.slice(0, tag.length - 2)]],
  ['RGI_Emoji_ZWJ_Sequence', [zwj], [grinning, codePoints([0x1f469, 0x1f4bb])]],
  ['RGI_Emoji', [grinning, marked, keycap, toned, flag, tag, zwj], ['x', '1', '']]
]) {
  var source = String.fromCharCode(92) + 'p{' + row[0] + '}';
  checkMembers(source, 'v', row[1], row[2]);
  checkMembers('[' + source + ']', 'v', row[1], row[2]);
}
checkMembers("[\\p{RGI_Emoji}&&\\p{RGI_Emoji_Flag_Sequence}]", 'v', [flag], [grinning, keycap, zwj]);
checkMembers("[\\p{RGI_Emoji}--\\p{Basic_Emoji}]", 'v', [keycap, toned, flag, tag, zwj], [grinning, marked]);
checkMembers("[[\\p{Emoji_Keycap_Sequence}]&&\\q{1\\uFE0F\\u20E3|a}]", 'v', [keycap], ['a', grinning]);
checkMembers("[\\q{1\\uFE0F\\u20E3|ab}--\\p{Emoji_Keycap_Sequence}]", 'v', ['ab'], [keycap, grinning]);
checkMembers("[\\p{Basic_Emoji}&&[^\\p{ASCII}]]", 'v', [grinning], [marked, keycap, 'a']);
checkMembers("[^\\p{Basic_Emoji}&&[a]]", 'v', ['a', 'b', grinning], ['', 'ab']);
var indices = computed("^(\\p{RGI_Emoji_Flag_Sequence})$", 'dv').exec(flag);
require(indices && indices.indices[0][1] === 4 && indices.indices[1][1] === 4, 'finite codepoint sequence retains UTF16 indices');
print('regexp-computed-strings-properties:ok');
262;
