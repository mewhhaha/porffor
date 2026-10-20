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

var sl = new Intl.PluralRules('sl'), en = new Intl.PluralRules('en');
same(sl.select(1), 'one', 'Slovenian start category'); same(sl.select(101), 'one', 'Slovenian end category');
same(sl.selectRange(1,101), 'few', 'distinct bare strings consult One-to-One matrix');
same(sl.selectRange(1,'1.0004'), 'one', 'rounded equal bare strings bypass matrix');
same(en.selectRange(-1,1), 'one', 'bare strings omit sign');
same(new Intl.PluralRules('en',{minimumIntegerDigits:4}).selectRange(-1,1), 'one', 'padded bare equality');
same(new Intl.PluralRules('en',{minimumFractionDigits:1}).selectRange(-1,1), 'other', 'equal visible fractional zeros retain category');
var ordinal = new Intl.PluralRules('en',{type:'ordinal'});
same(ordinal.selectRange(4,1), 'one', 'declared ordinal missing-matrix End policy');
same(en.selectRange(4,1), 'other', 'cardinal matrix remains separate');
same(en.selectRange(Infinity,Infinity), 'other', 'equal infinity representation');

print("ok range_bare_identity");
262;
