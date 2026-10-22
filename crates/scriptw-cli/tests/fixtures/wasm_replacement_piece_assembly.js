function check(ok, name) { if (!ok) throw new Error(name); }
let trace = [];
const receiver = { toString() { trace.push('receiver'); return 'abc'; } };
const search = { toString() { trace.push('search'); return 'z'; } };
const replacement = { toString() { trace.push('replacement'); return 'X'; } };
check(String.prototype.replaceAll.call(receiver, search, replacement) === 'abc', 'literal no match');
check(trace.join(',') === 'receiver,search,replacement', 'no-match conversions');
const marker = {};
try { 'abc'.replaceAll('z', { toString() { throw marker; } }); throw new Error('missing conversion'); }
catch (e) { check(e === marker, 'no-match abrupt conversion'); }
let called = 0;
const never = function () { called++; return 'x'; };
never.toString = function () { throw new Error('callable converted'); };
check('abc'.replaceAll('z', never) === 'abc' && called === 0, 'no-match callable');
trace = [];
const fake = { get global() { trace.push('global'); return true; },
  get unicode() { trace.push('unicode'); return false; },
  get unicodeSets() { trace.push('unicodeSets'); return false; },
  set lastIndex(v) { trace.push('set:' + v); },
  exec(s) { trace.push('exec'); check(s === 'abc', 'exec input'); return null; } };
check(RegExp.prototype[Symbol.replace].call(fake, 'abc', replacement) === 'abc', 'regex no match');
check(trace[0] === 'replacement' && trace.indexOf('set:0') < trace.indexOf('exec'), 'regex no-match effects');
let index = 0;
const custom = /x/g;
custom.exec = function () { trace.push('exec:' + index); if (index === 2) return null; return { 0: 'x', index: index++ === 0 ? 2 : 0, length: 1 }; };
trace = [];
const overlapping = 'abcdef'.replace(custom, function (m, p) { trace.push('callback:' + p); return String(p); });
check(overlapping === 'ab2def', 'overlap output');
check(trace.join(',') === 'exec:0,exec:1,exec:2,callback:2,callback:0', 'collect before callbacks');
const payload = 'q'.repeat(256);
const n = 12000;
let calls = 0;
const assembled = 'a'.repeat(n).replaceAll('a', function (m, p, input) {
  check(m === 'a' && p === calls && input.length === n, 'callback arguments');
  if (p === 8 || p === 12) gc();
  calls++; return String.fromCharCode(65 + p % 26) + payload;
});
check(calls === n && assembled.length === n * 257, 'retained pieces length');
for (let k = 0; k < n; k += 257) check(assembled.charCodeAt(k * 257) === 65 + k % 26 && assembled.charAt(k * 257 + 256) === 'q', 'retained pieces roots');
for (let k = 0; k < 8; ++k) check(assembled.charCodeAt(k * 257) === 65 + k && assembled.charAt(k * 257 + 256) === 'q', 'all inline roots after GC');
const units = '\uD800x\uDC00';
check('𝄞a𝄞a'.replaceAll('a', units) === '𝄞\uD800x\uDC00𝄞\uD800x\uDC00', 'literal UTF16');
check('𝄞a𝄞a'.replace(/a/gu, units) === '𝄞\uD800x\uDC00𝄞\uD800x\uDC00', 'regex UTF16');
check('𝄞'.replaceAll('', '|') === '|\uD834|\uDD1E|', 'literal empty search units');
check('𝄞'.replace(/(?:)/gu, '|') === '|𝄞|', 'regex empty search codepoints');
check('abc'.replaceAll('b', '') === 'ac' && ''.replace(/x/g, '') === '', 'empty pieces');
check('abc'.replaceAll('b', '$`-$&-$\'') === 'aa-b-cc', 'substitution pieces');
// Named captures still require ToObject even when the template has no tokens.
const nullGroups = /a/;
nullGroups.exec = function () { return { 0: 'a', index: 0, length: 1, groups: null }; };
try { 'a'.replace(nullGroups, 'x'); throw new Error('null groups accepted'); }
catch (e) { check(e instanceof TypeError, 'token-free named capture admission'); }
let namedReads = 0;
const named = /a/;
named.exec = function () { return { 0: 'a', index: 0, length: 1, groups: { get v() { namedReads++; return 'G'; } } }; };
check('a'.replace(named, 'long literal replacement') === 'long literal replacement' && namedReads === 0, 'token-free named getter');
check('a'.replace(named, '$<v>') === 'G' && namedReads === 1, 'named getter retained');
// Cross the bounded-prefix boundary before/at/after a completed callback piece.
for (const size of [0, 1, 31, 63, 64, 65, 66, 129, 256]) {
  const piece = 'q'.repeat(size);
  for (const input of ['a', 'a,a,a', 'z'.repeat(64) + 'a', 'z'.repeat(65) + 'a']) {
    const expected = input.split('a').join(piece);
    let literalCalls = 0;
    let regexpCalls = 0;
    check(input.replaceAll('a', () => { ++literalCalls; return piece; }) === expected, 'literal prefix spill');
    check(input.replace(/a/g, () => { ++regexpCalls; return piece; }) === expected, 'regexp prefix spill');
    check(literalCalls === input.split('a').length - 1 && regexpCalls === literalCalls, 'prefix spill callbacks');
  }
}
check('xy'.repeat(64).replace(/y/g, '') === 'x'.repeat(64), 'short output after many matches');
for (const count of [0, 1, 2, 3, 4, 7, 8, 9, 12, 13, 16]) {
  const input = 'a'.repeat(count);
  for (const skipEmpty of [false, true]) {
    let expected = '';
    for (let i = 0; i < count; ++i) if (!skipEmpty || i % 3 !== 0) expected += String.fromCharCode(65 + i);
    let literalCalls = 0;
    let regexpCalls = 0;
    check(input.replaceAll('a', (_, index) => { ++literalCalls; if (index === 8 || index === 12) gc(); return skipEmpty && index % 3 === 0 ? '' : String.fromCharCode(65 + index); }) === expected, 'literal inline/spill counts');
    check(input.replace(/a/g, (_, index) => { ++regexpCalls; if (index === 8 || index === 12) gc(); return skipEmpty && index % 3 === 0 ? '' : String.fromCharCode(65 + index); }) === expected, 'regexp inline/spill counts');
    check(literalCalls === count && regexpCalls === count, 'inline/spill callback counts');
  }
}
print('replacement-piece-assembly-ok');
