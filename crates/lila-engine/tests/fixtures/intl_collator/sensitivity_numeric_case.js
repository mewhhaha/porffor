function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
for (var sensitivity of ['base', 'accent', 'case', 'variant']) { var c = new Intl.Collator('en-US', {sensitivity: sensitivity}); var f = c.compare; same(f('a', 'a\u0301') === 0, sensitivity === 'base' || sensitivity === 'case', 'accent sensitivity'); same(f('a', 'A') === 0, sensitivity === 'base' || sensitivity === 'accent', 'case sensitivity'); }
check(new Intl.Collator('en-US', {numeric: true}).compare('2', '10') < 0, 'numeric two before ten');
check(new Intl.Collator('en-US', {numeric: false}).compare('2', '10') > 0, 'lexical numeric false');
check(new Intl.Collator('en-US', {caseFirst: 'upper'}).compare('A', 'a') < 0, 'upper first'); check(new Intl.Collator('en-US', {caseFirst: 'lower'}).compare('a', 'A') < 0, 'lower first');
same(new Intl.Collator('en-US', {ignorePunctuation: true}).compare('ab', 'a-b'), 0, 'punctuation ignored'); check(new Intl.Collator('en-US', {ignorePunctuation: false}).compare('ab', 'a-b') !== 0, 'punctuation retained');
print('ok sensitivity_numeric_case');
262;
