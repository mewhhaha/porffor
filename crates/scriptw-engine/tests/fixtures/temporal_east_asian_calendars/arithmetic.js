function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function range(action) {
  try {action();} catch (caught) {if (!(caught instanceof RangeError)) throw caught; return;}
  throw new Error('missing RangeError');
}
for (const calendar of ['chinese', 'dangi']) {
  const date = (year, code, day = 1) => Temporal.PlainDate.from({calendar, year, monthCode: code, day}, {overflow: 'reject'});
  const ym = (year, code) => Temporal.PlainYearMonth.from({calendar, year, monthCode: code}, {overflow: 'reject'});
  function fields(value, year, code, day) {
    same(value.year, year, 'arithmetic related year');
    same(value.monthCode, code, 'arithmetic canonical code');
    if (day !== undefined) same(value.day, day, 'arithmetic day');
  }
  fields(date(1966,'M03L').add({years: 1}), 1967, 'M03', 1);
  fields(date(1966,'M03L').subtract({years: 1}), 1965, 'M03', 1);
  fields(date(1938,'M07L',30).add({years: 1}), 1939, 'M07', 29);
  range(() => date(1966,'M03L').add({years: 1}, {overflow: 'reject'}));
  range(() => date(1966,'M03L').add({years: 1, months: 1}, {overflow: 'reject'}));
  fields(date(2000,'M08',2).add({years: 1, months: 12}), 2002, 'M08', 2);
  same(date(2000,'M08').add({years: 1}).month, 9, 'year preserves code before month step');
  fields(date(2020,'M03').add({months: 2}), 2020, 'M04L', 1);
  fields(date(2020,'M04L').subtract({months: 1}), 2020, 'M04', 1);
  fields(date(2019,'M04').add({months: 12}), 2020, 'M04', 1);
  fields(date(2019,'M04').add({months: 13}), 2020, 'M04L', 1);
  fields(date(2020,'M04').add({months: 12}), 2021, 'M03', 1);
  fields(date(2020,'M04').add({months: 13}), 2021, 'M04', 1);
  fields(date(2021,'M04').subtract({months: 12}), 2020, 'M04L', 1);
  fields(date(2021,'M04').subtract({months: 25}), 2019, 'M04', 1);
  const pairs = [
    [2000,'M04',2001,'M04',1,0,12],
    [2001,'M04',2002,'M04',1,0,13],
    [2000,'M04',2002,'M04',2,0,25],
    [2000,'M04',2001,'M04L',1,1,13],
    [2001,'M04L',2002,'M04',0,12,12],
    [2000,'M05',2001,'M04L',0,12,12],
    [2001,'M04L',2002,'M05',1,1,13],
    [2002,'M04',2001,'M04L',0,-12,-12],
    [2001,'M04L',2000,'M04',-1,0,-13],
    [2002,'M05',2001,'M04L',-1,-1,-13],
    [2001,'M04L',2000,'M05',0,-12,-12]
  ];
  for (const [y1,c1,y2,c2,years,months,total] of pairs) {
    for (const [left,right] of [[date(y1,c1),date(y2,c2)], [ym(y1,c1),ym(y2,c2)]]) {
      const result = left.until(right, {largestUnit: 'years'});
      same(result.years, years, 'original-code virtual year comparison');
      same(result.months, months, 'retained actual month pair');
      same(result.days, 0, 'first-day remainder');
      same(left.until(right, {largestUnit: 'months'}).months, total, 'native month serial difference');
      // since negates this receiver's difference; swapping endpoints can
      // change the year/month split around a constrained leap month.
      same(left.since(right, {largestUnit: 'years'}).toString(), result.negated().toString(), 'since negates receiver-relative difference');
      same(left.add(result).equals(right), true, 'difference reconstructs endpoint');
    }
  }
  // Balanced ordinal comparison must precede day clamping as well.
  for (const largestUnit of ['years', 'months']) {
    const end = date(2020,'M04L',29);
    same(date(2020,'M04',29).until(end, {largestUnit}).months, 1, 'matching day admits whole leap step');
    const short = date(2020,'M04',30).until(end, {largestUnit});
    same(short.months, 0, 'day overflow cannot invent a whole month');
    same(short.days, 29, 'real remainder before clamp');
  }
  const origin = ym(2000,'M04');
  const beforeYear = ym(2001,'M03');
  same(origin.until(beforeYear, {largestUnit:'years', smallestUnit:'years', roundingMode:'floor'}).years, 0, 'year bracket floor');
  same(origin.until(beforeYear, {largestUnit:'years', smallestUnit:'years', roundingMode:'ceil'}).years, 1, 'year bracket ceil');
  same(beforeYear.until(origin, {largestUnit:'years', smallestUnit:'years', roundingMode:'floor'}).years, -1, 'negative year floor');
  const end = origin.add({years: 2, months: 1});
  const rounded = origin.until(end, {largestUnit:'years', smallestUnit:'months', roundingIncrement:2, roundingMode:'ceil'});
  same(rounded.years, 2, 'retained whole years after rounding');
  same(rounded.months, 2, 'actual month increment after year anchor');
  for (const year of [-10000,-3654,-3653,4703,4704,10000]) {
    const start = date(year,'M01');
    const far = start.add({months: 20000});
    same(start.until(far, {largestUnit:'months'}).months, 20000, 'wide actual serial shift');
    same(far.subtract({months: 20000}).equals(start), true, 'wide bounded inverse serial');
  }
}
print('east-asian-arithmetic:ok');
262;
