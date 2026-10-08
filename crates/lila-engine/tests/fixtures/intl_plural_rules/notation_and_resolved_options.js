// Source/data-derived expectations; authored control, not an executed result.
function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { check(Object.is(actual, expected), label); }
function throws(ctor, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  check(caught instanceof ctor && caught.constructor === ctor, label);
}
function abrupt(marker, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  same(caught, marker, label);
}

var standard = new Intl.PluralRules('fr'), compact = new Intl.PluralRules('fr',{notation:'compact'});
for (var row of [[1000000,'many','many'],[1500000,'other','many'],[0.000001,'one','one']]) {
  same(standard.select(row[0]), row[1], 'French standard'); same(compact.select(row[0]), row[2], 'French compact');
}
for (var notation of ['scientific','engineering']) same(new Intl.PluralRules('fr',{notation:notation}).select(1500000), 'other', 'unscaled bare numeric notation');
var carry = new Intl.PluralRules('fr',{notation:'compact',maximumFractionDigits:0});
same(carry.select('999999.5'), 'many', 'compact exponent from final bare rounded magnitude'); same(carry.select('999999.4'), 'other', 'compact neighbor before carry');
var normalKeys = 'locale,type,notation,minimumIntegerDigits,minimumFractionDigits,maximumFractionDigits,pluralCategories,roundingIncrement,roundingMode,roundingPriority,trailingZeroDisplay';
same(Object.keys(standard.resolvedOptions()).join(','), normalKeys, 'default resolved option order');
var result = compact.resolvedOptions();
same(Object.keys(result).join(','), 'locale,type,notation,compactDisplay,minimumIntegerDigits,minimumFractionDigits,maximumFractionDigits,minimumSignificantDigits,maximumSignificantDigits,pluralCategories,roundingIncrement,roundingMode,roundingPriority,trailingZeroDisplay', 'compact resolved option order');
same(result.minimumFractionDigits,0,'compact min fraction'); same(result.maximumFractionDigits,0,'compact max fraction'); same(result.minimumSignificantDigits,1,'compact min significant'); same(result.maximumSignificantDigits,2,'compact max significant'); same(result.roundingPriority,'morePrecision','compact default priority');
for (var display of ['short','long']) {
  var de = new Intl.PluralRules('de',{notation:'compact',compactDisplay:display}), resolved = de.resolvedOptions();
  same(resolved.compactDisplay,display,'retained compact display'); same(de.select(1000),'other','German compact category');
}
var first = standard.resolvedOptions(), second = standard.resolvedOptions();
check(first !== second && first.pluralCategories !== second.pluralCategories,'fresh results and category arrays');
first.pluralCategories.length = 0; same(standard.resolvedOptions().pluralCategories.join(','),'one,many,other','result mutation does not affect private slots');
for (var key of Object.keys(second)) {var descriptor = Object.getOwnPropertyDescriptor(second,key); check(descriptor.writable && descriptor.enumerable && descriptor.configurable,'result data flags');}

print("ok notation_and_resolved_options");
262;
