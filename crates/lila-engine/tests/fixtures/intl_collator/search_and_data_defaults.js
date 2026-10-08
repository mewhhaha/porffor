function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var values = ['AE', '\u00c4'];
same(values.slice().sort(new Intl.Collator('de', {usage: 'sort'}).compare).join('|'), '\u00c4|AE', 'German real sort');
same(values.slice().sort(new Intl.Collator('de', {usage: 'search'}).compare).join('|'), 'AE|\u00c4', 'German genuine search');
same(new Intl.Collator('de', {usage: 'search'}).resolvedOptions().sensitivity, 'variant', 'owned search default');
same(new Intl.Collator('th').resolvedOptions().ignorePunctuation, true, 'Thai data default');
same(new Intl.Collator('th', {ignorePunctuation: false}).resolvedOptions().ignorePunctuation, false, 'explicit false overrides Thai');
same(new Intl.Collator('da').resolvedOptions().caseFirst, 'upper', 'Danish data default');
same(new Intl.Collator('da', {caseFirst: 'lower'}).resolvedOptions().caseFirst, 'lower', 'explicit first overrides data');
print('ok search_and_data_defaults');
262;
