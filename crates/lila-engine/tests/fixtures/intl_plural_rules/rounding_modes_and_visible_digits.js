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

var categories = {ceil:'one',floor:'two',expand:'two',trunc:'one',halfCeil:'one',halfFloor:'two',halfExpand:'two',halfTrunc:'one',halfEven:'two'};
for (var mode of Object.keys(categories)) {
  var rules = new Intl.PluralRules('en', {type:'ordinal',maximumFractionDigits:0,roundingMode:mode});
  same(rules.select('-1.5'), categories[mode], 'negative signed rounding ' + mode);
}
same(new Intl.PluralRules('en',{type:'ordinal',maximumFractionDigits:0,roundingMode:'halfEven'}).select('2.5'), 'two', 'positive even tie');
same(new Intl.PluralRules('en',{type:'ordinal',maximumFractionDigits:0,roundingMode:'halfExpand'}).select('2.5'), 'few', 'positive expansion tie');
same(new Intl.PluralRules('en',{minimumFractionDigits:1}).select(1), 'other', 'visible zero');
same(new Intl.PluralRules('en',{minimumFractionDigits:1,trailingZeroDisplay:'stripIfInteger'}).select(1), 'one', 'strip visible integer zero');
same(new Intl.PluralRules('ru',{minimumFractionDigits:2}).select(1), 'other', 'Russian visible zero');
same(new Intl.PluralRules('ru',{minimumFractionDigits:2,trailingZeroDisplay:'stripIfInteger'}).select(1), 'one', 'Russian stripped zero');
same(new Intl.PluralRules('en',{minimumIntegerDigits:4}).select(1), 'one', 'integer padding does not change operands');
var options = {minimumFractionDigits:0,maximumFractionDigits:1,minimumSignificantDigits:1,maximumSignificantDigits:3,roundingPriority:'morePrecision'};
same(new Intl.PluralRules('en',options).select('1.04'), 'other', 'more precision exact fraction');
options.roundingPriority = 'lessPrecision'; same(new Intl.PluralRules('en',options).select('1.04'), 'one', 'less precision integer');

print("ok rounding_modes_and_visible_digits");
262;
