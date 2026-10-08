var ascii = /\p{ASCII}+/u;
require(ascii.exec('AB')[0] === 'AB' && !ascii.test('é'), 'compiled u property');
var strings = /[\q{ab|cd}]/v;
require(strings.exec('xab')[0] === 'ab' && !strings.test('x'), 'compiled v strings');
var basicEmoji = /\p{Basic_Emoji}/v;
require(basicEmoji.test('😀'), 'provider-backed Basic_Emoji remains compiled');
print('ok');
262;
