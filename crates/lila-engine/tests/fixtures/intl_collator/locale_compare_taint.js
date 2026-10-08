function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var Original = Intl.Collator; var originalPrototype = Original.prototype;
Intl.Collator = function() { throw new Error('public constructor observed'); };
Object.defineProperty(originalPrototype, 'compare', {configurable: true, get() { throw new Error('public compare observed'); }});
check('2'.localeCompare('10', 'en-US', {numeric: true}) < 0, 'immutable constructor/private CompareStrings');
same('\u00e9'.localeCompare('e\u0301', 'en-US'), 0, 'immutable canonical kernel');
var originalGet = {get usage() { Original.prototype.resolvedOptions.call(new Original('de')); return 'search'; }};
check('AE'.localeCompare('\u00c4', 'de', originalGet) <= 0, 'nested hook preserves immutable search selection');
print('ok locale_compare_taint');
262;
