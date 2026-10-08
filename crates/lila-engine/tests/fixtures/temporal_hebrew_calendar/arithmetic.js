function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function range(action) {
  try {action();} catch (caught) {if (!(caught instanceof RangeError)) throw caught; return;}
  throw new Error('missing RangeError');
}
const calendar = 'hebrew';
const date = (year, monthCode, day = 1) => Temporal.PlainDate.from({calendar, year, monthCode, day}, {overflow: 'reject'});
const ym = (year, monthCode) => Temporal.PlainYearMonth.from({calendar, year, monthCode}, {overflow: 'reject'});
function fields(value, year, code, day) {
  same(value.year, year, 'arithmetic year');
  same(value.monthCode, code, 'arithmetic code');
  if (day !== undefined) same(value.day, day, 'arithmetic day');
}
fields(date(5783,'M06').add({years: 1}, {overflow: 'reject'}), 5784, 'M06', 1);
same(date(5783,'M06').add({years: 1}).month, 7, 'year preserves code before ordinal');
fields(date(5784,'M05L').add({years: 1}), 5785, 'M06', 1);
range(() => date(5784,'M05L').add({years: 1}, {overflow: 'reject'}));
range(() => date(5784,'M05L').add({years: 1, months: 1}, {overflow: 'reject'}));
range(() => ym(5000,'M05L').add({years: 1, months: 1}, {overflow: 'reject'}));
fields(date(5782,'M05L').add({years: 2}, {overflow: 'reject'}), 5784, 'M05L', 1);
fields(date(5784,'M05L',30).add({years: 1}), 5785, 'M06', 29);
fields(date(5783,'M06').add({months: 12}), 5784, 'M05L', 1);
fields(date(5783,'M06').add({months: 13}), 5784, 'M06', 1);
fields(date(5784,'M05L').add({months: 12}), 5785, 'M05', 1);
fields(date(5784,'M05L').add({months: 13}), 5785, 'M06', 1);
fields(date(5785,'M06').subtract({months: 25}), 5783, 'M06', 1);
fields(date(5783,'M08',2).add({years: 1, months: 12}), 5785, 'M08', 2);
const virtual = date(5782,'M05L',30).until(date(5781,'M06',1), {largestUnit:'years'});
same(virtual.years,0,'original leap-code virtual year surpasses');
same(virtual.months,-12,'virtual boundary retains actual months');
same(virtual.days,-28,'virtual midpoint clamps only after comparison');
// Pinned leap-month differences exercise actual month counts and the missing
// M05L virtual insertion point in both directions, before day clamp.
const pairs = [
  [5783,'M05',5784,'M05',1,0,12],
  [5784,'M05',5785,'M05',1,0,13],
  [5783,'M05',5785,'M05',2,0,25],
  [5783,'M06',5784,'M05L',0,12,12],
  [5784,'M05L',5785,'M05',0,12,12],
  [5784,'M05L',5785,'M06',1,0,13],
  [5784,'M05L',5783,'M05',-1,-1,-13],
  [5785,'M06',5784,'M05L',-1,-1,-13],
  [5784,'M05L',5783,'M06',0,-12,-12]
];
for (const [y1,c1,y2,c2,years,months,total] of pairs) {
  for (const [left,right] of [[date(y1,c1),date(y2,c2)], [ym(y1,c1),ym(y2,c2)]]) {
    const diff = left.until(right, {largestUnit: 'years'});
    same(diff.years, years, 'separate year component');
    same(diff.months, months, 'remaining actual months');
    same(diff.days, 0, 'first-day remainder');
    same(left.until(right, {largestUnit: 'months'}).months, total, 'exact serial difference');
    // since negates this receiver's difference; swapping endpoints can
    // change the year/month split around a constrained leap month.
    same(left.since(right, {largestUnit: 'years'}).toString(), diff.negated().toString(), 'since negates receiver-relative difference');
    same(left.add(diff).equals(right), true, 'difference reconstructs endpoint');
  }
}
const origin = ym(5783,'M05');
const beforeLeapYearEnd = ym(5784,'M04');
same(origin.until(beforeLeapYearEnd, {largestUnit:'years', smallestUnit:'years', roundingMode:'floor'}).years, 0, 'year bracket floor');
same(origin.until(beforeLeapYearEnd, {largestUnit:'years', smallestUnit:'years', roundingMode:'ceil'}).years, 1, 'year bracket ceil');
same(beforeLeapYearEnd.until(origin, {largestUnit:'years', smallestUnit:'years', roundingMode:'ceil'}).years, 0, 'negative year bracket ceil');
same(beforeLeapYearEnd.until(origin, {largestUnit:'years', smallestUnit:'years', roundingMode:'floor'}).years, -1, 'negative year bracket floor');
const afterTwoYears = origin.add({years: 2, months: 1});
const rounded = origin.until(afterTwoYears, {largestUnit:'years', smallestUnit:'months', roundingIncrement:2, roundingMode:'ceil'});
same(rounded.years, 2, 'rounded retained whole years');
same(rounded.months, 2, 'month increment after real year anchor');
same(origin.add(rounded).equals(origin.add({years:2,months:2})), true, 'actual rounded pair anchor');
print('hebrew-arithmetic:ok');
262;
