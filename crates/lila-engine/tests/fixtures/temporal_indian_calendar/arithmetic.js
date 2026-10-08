function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function date(year, month, day) {
  return Temporal.PlainDate.from({calendar: 'indian', year, month, day});
}
function parts(actual, years, months, weeks, days, label) {
  same(actual.years, years, label + ' years');
  same(actual.months, months, label + ' months');
  same(actual.weeks, weeks, label + ' weeks');
  same(actual.days, days, label + ' days');
}
function range(action) {
  try {action();} catch (error) {
    if (!(error instanceof RangeError)) throw new Error('wrong arithmetic error');
    return;
  }
  throw new Error('missing arithmetic RangeError');
}
const leapStart = date(1942, 1, 1);
const nextYear = leapStart.add({years: 1});
same(nextYear.withCalendar('iso8601').toString(), '2021-03-22', 'calendar year addition');
same(leapStart.until(nextYear).days, 366, 'calendar leap-year span');
parts(leapStart.until(nextYear, {largestUnit: 'year'}), 1, 0, 0, 0, 'exact calendar year');
same(nextYear.subtract({years: 1}).equals(leapStart), true, 'negative calendar year');
const leapEnd = date(1942, 1, 31);
same(leapEnd.add({years: 1}).day, 30, 'calendar constrain');
range(() => leapEnd.add({years: 1}, {overflow: 'reject'}));
const longMonth = date(1943, 2, 1);
const followingMonth = longMonth.add({months: 1});
same(followingMonth.withCalendar('iso8601').toString(), '2021-05-22', 'calendar month addition');
parts(longMonth.until(followingMonth, {largestUnit: 'month'}), 0, 1, 0, 0, 'exact calendar month');
same(longMonth.until(followingMonth).days, 31, 'calendar month span');
const lastLongDay = date(1942, 6, 31);
const shortMonthEnd = lastLongDay.add({months: 1});
same(shortMonthEnd.month, 7, 'constrained short month');
same(shortMonthEnd.day, 30, 'constrained short day');
parts(lastLongDay.until(shortMonthEnd, {largestUnit: 'month'}), 0, 0, 0, 30, 'virtual unclamped positive anchor');
parts(shortMonthEnd.until(lastLongDay, {largestUnit: 'month'}), 0, 0, 0, -30, 'virtual unclamped negative anchor');
parts(lastLongDay.until(lastLongDay.add({months: 2}), {largestUnit: 'month'}), 0, 1, 0, 30, 'virtual two-month anchor');
parts(longMonth.until(longMonth.add({days: 15}), {largestUnit: 'month', smallestUnit: 'month', roundingMode: 'halfExpand'}), 0, 0, 0, 0, 'below calendar half');
parts(longMonth.until(longMonth.add({days: 16}), {largestUnit: 'month', smallestUnit: 'month', roundingMode: 'halfExpand'}), 0, 1, 0, 0, 'above calendar half');
parts(leapStart.until(leapStart.add({days: 183}), {largestUnit: 'year', smallestUnit: 'year', roundingMode: 'halfExpand'}), 1, 0, 0, 0, 'calendar leap-year tie');
for (const delta of [{days: 40}, {weeks: -3}, {years: 1, months: 2, days: 4}]) {
  const calendar = longMonth.add(delta);
  if (delta.years === undefined && delta.months === undefined) {
    same(calendar.withCalendar('iso8601').equals(longMonth.withCalendar('iso8601').add(delta)), true, 'epoch-only arithmetic');
  }
}
const time = longMonth.toPlainDateTime({hour: 23, minute: 30});
const timeEnd = time.add({months: 1, hours: 2});
same(timeEnd.month, 3, 'datetime calendar month');
same(timeEnd.day, 2, 'datetime elapsed carry');
same(timeEnd.hour, 1, 'datetime clock carry');
same(time.until(time.add({months: 1}), {largestUnit: 'month'}).months, 1, 'datetime calendar difference');
const ym = longMonth.toPlainYearMonth();
const ymEnd = ym.add({years: 2, months: 3});
same(ymEnd.year, 1945, 'partial calendar year');
same(ymEnd.month, 5, 'partial calendar month');
same(ym.until(ymEnd, {largestUnit: 'month'}).months, 27, 'partial calendar difference');
same(ymEnd.subtract({years: 2, months: 3}).equals(ym), true, 'partial negative addition');
const zoned = Temporal.ZonedDateTime.from('2020-03-21T12:00[UTC][u-ca=indian]');
const zonedEnd = zoned.add({years: 1});
same(zonedEnd.toPlainDate().withCalendar('iso8601').toString(), '2021-03-22', 'zoned calendar year');
same(zoned.until(zonedEnd, {largestUnit: 'year'}).years, 1, 'zoned calendar difference');
same(zoned.until(zonedEnd, {largestUnit: 'day'}).days, 366, 'zoned actual span');
// Retain the established Gregorian calendar domains while adding this algorithm.
for (const calendar of ['iso8601', 'gregory', 'buddhist', 'roc', 'japanese']) {
  const old = new Temporal.PlainDate(2020, 1, 31, calendar);
  const result = old.add({months: 1}).withCalendar('iso8601');
  same(result.toString(), '2020-02-29', 'Gregorian-family month');
}
print('indian-arithmetic:ok');
262;
