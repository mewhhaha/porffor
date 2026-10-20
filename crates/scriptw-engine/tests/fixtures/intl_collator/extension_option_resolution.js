function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var c = new Intl.Collator('de-u-co-phonebk-kf-upper-kn', {numeric: false, caseFirst: 'lower'}); var o = c.resolvedOptions();
same(o.collation, 'phonebk', 'admitted phonebook type'); same(o.numeric, false, 'numeric override'); same(o.caseFirst, 'lower', 'first override');
same(o.locale.indexOf('co-phonebk') >= 0, true, 'co extension retained'); same(o.locale.indexOf('kf-upper'), -1, 'conflicting first removed'); same(o.locale.indexOf('-kn'), -1, 'conflicting numeric removed');
for (var co of ['standard', 'search', 'foobar']) same(new Intl.Collator('en-US', {collation: co}).resolvedOptions().collation, 'default', 'wellformed unsupported type defaults');
throws(RangeError, function() { new Intl.Collator('en-US', {collation: 'not_a_type'}); }, 'invalid Unicode type');
same(new Intl.Collator('de', {usage: 'search', collation: 'phonebk'}).resolvedOptions().collation, 'default', 'search uses selected SearchLocaleData');
print('ok extension_option_resolution');
262;
