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

var ordinal = new Intl.PluralRules('en', {type:'ordinal'});
same(ordinal.select('9007199254740993'), 'few', 'exact large decimal string');
same(ordinal.select(9007199254740993n), 'few', 'exact BigInt');
same(ordinal.select(9007199254740993), 'two', 'Number shortest decimal remains rounded Number');
for (var row of [['0x1','one'],['0b10','two'],['0o3','few']]) same(ordinal.select(row[0]), row[1], 'nondecimal numeric string');
var precise = new Intl.PluralRules('en', {maximumFractionDigits:20});
same(precise.select('1.0000000000000001'), 'other', 'exact fractional string'); same(precise.select(1.0000000000000001), 'one', 'Number fractional input');
var huge = '1' + '0'.repeat(398) + '3';
same(ordinal.select(BigInt(huge)), 'few', 'BigInt does not undergo string overflow classification');
same(ordinal.select(huge), 'other', 'string overflow becomes infinity');
for (var value of [NaN,Infinity,-Infinity,'1e400','not numeric','\uD800']) same(ordinal.select(value), 'other', 'nonfinite or invalid numeric');
same(new Intl.PluralRules('ar').select(-0), 'zero', 'negative zero');

print("ok exact_mathematical_values");
262;
