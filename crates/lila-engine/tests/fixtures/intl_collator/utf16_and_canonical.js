function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
for (var sensitivity of ['base', 'accent', 'case', 'variant']) {
 var cmp = new Intl.Collator('en-US', {sensitivity: sensitivity}).compare;
 same(cmp('\u00e9', 'e\u0301'), 0, 'canonical equivalence'); same(Object.is(cmp('\u00e9', 'e\u0301'), -0), false, 'canonical positive zero');
 for (var pair of [['\ud800', '\ufffd'], ['\udfff', '\ufffd'], ['\ud800\ud800', '\ufffd\ufffd'], ['a\ud800', 'a\ufffd'], ['\udfffz', '\ufffdz']]) same(cmp(pair[0], pair[1]), 0, 'chosen UCA replacement weights');
 check(cmp('\ud83d\ude00', '\ufffd\ufffd') !== 0, 'valid pair is one codepoint'); check(cmp('\ud83d\ude00', '\ud83d\ude01') < 0, 'shared high surrogate prefix');
 same(cmp('x\ud800z', 'x\udfffz'), 0, 'isolated units within retained input');
}
var cmp = new Intl.Collator('en-US').compare; var strings = ['a', 'b', '\ud800', '\udfff', '\ufffd', '\ud83d\ude00'];
function sign(x) { return x < 0 ? -1 : x > 0 ? 1 : 0; }
for (var x of strings) for (var y of strings) { same(sign(cmp(x, y)), -sign(cmp(y, x)) || 0, 'sign symmetry'); for (var z of strings) if (cmp(x, y) <= 0 && cmp(y, z) <= 0) check(cmp(x, z) <= 0, 'transitivity'); }
print('ok utf16_and_canonical');
262;
