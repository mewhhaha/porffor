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

var ar = new Intl.PluralRules('ar'), ru = new Intl.PluralRules('ru'), sl = new Intl.PluralRules('sl');
for (var row of [[0,'zero'],[1,'one'],[2,'two'],[3,'few'],[11,'many'],[100,'other']]) same(ar.select(row[0]), row[1], 'Arabic category');
same(ar.resolvedOptions().pluralCategories.join(','), 'zero,one,two,few,many,other', 'Arabic canonical categories');
for (var row of [[1,'one'],[2,'few'],[5,'many'],[11,'many'],[21,'one'],[1.5,'other']]) same(ru.select(row[0]), row[1], 'Russian category');
for (var row of [[1,'one'],[2,'two'],[3,'few'],[5,'other'],[1.5,'few']]) same(sl.select(row[0]), row[1], 'Slovenian category');
var ordinal = new Intl.PluralRules('en', {type:'ordinal'});
for (var row of [[1,'one'],[2,'two'],[3,'few'],[11,'other'],[12,'other'],[13,'other'],[21,'one']]) same(ordinal.select(row[0]), row[1], 'English ordinal');
same(ordinal.resolvedOptions().pluralCategories.join(','), 'one,two,few,other', 'ordinal categories');
var shaw = new Intl.PluralRules('en-Shaw', {type:'ordinal'}), ido = new Intl.PluralRules('io', {type:'ordinal'});
same(shaw.select(3), 'few', 'ordinal locale independent of NF pooled profile'); same(ido.select(3), 'other', 'Other-only ordinal locale');
same(ido.resolvedOptions().pluralCategories.join(','), 'other', 'Other-only category mask');

print("ok scalar_categories");
262;
