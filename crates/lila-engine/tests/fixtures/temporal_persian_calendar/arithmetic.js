function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function date(year, month, day) {
  return Temporal.PlainDate.from({calendar: 'persian', year, month, day});
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
const leapStart = date(1403, 1, 1);
const nextYear = leapStart.add({years: 1});
same(nextYear.withCalendar('iso8601').toString(), '2025-03-21', 'calendar year addition');
same(leapStart.until(nextYear).days, 366, 'leap-year span');
parts(leapStart.until(nextYear, {largestUnit: 'year'}), 1, 0, 0, 0, 'exact year');
same(nextYear.subtract({years: 1}).equals(leapStart), true, 'negative year');
const leapEnd = date(1403, 12, 30);
same(leapEnd.add({years: 1}).day, 29, 'last-month constrain');
range(() => leapEnd.add({years: 1}, {overflow: 'reject'}));
same(date(1502, 1, 1).until(date(1503, 1, 1)).days, 365, 'corrected common year');
same(date(1503, 1, 1).until(date(1504, 1, 1)).days, 366, 'following corrected leap year');
for (const year of [-1, 0]) {
  parts(date(year, 1, 1).until(date(year + 1, 1, 1), {largestUnit: 'year'}), 1, 0, 0, 0, 'signed year transition');
}
const longMonth = date(1400, 2, 1);
const followingMonth = longMonth.add({months: 1});
same(followingMonth.withCalendar('iso8601').toString(), '2021-05-22', 'calendar month addition');
same(longMonth.until(followingMonth).days, 31, 'long-month span');
parts(longMonth.until(followingMonth, {largestUnit: 'month'}), 0, 1, 0, 0, 'exact month');
// Pinned Test262 virtual anchors: an unclamped day31 is not one full month.
const lastLongDay = date(1400, 6, 31);
const shortMonthEnd = lastLongDay.add({months: 1});
same(shortMonthEnd.month, 7, 'short month');
same(shortMonthEnd.day, 30, 'short day');
parts(lastLongDay.until(shortMonthEnd, {largestUnit: 'month'}), 0, 0, 0, 30, 'positive virtual anchor');
parts(shortMonthEnd.until(lastLongDay, {largestUnit: 'month'}), 0, 0, 0, -30, 'negative virtual anchor');
parts(lastLongDay.until(lastLongDay.add({months: 2}), {largestUnit: 'month'}), 0, 1, 0, 30, 'two-month virtual anchor');
parts(date(1400, 11, 30).until(date(1400, 12, 29), {largestUnit: 'month'}), 0, 0, 0, 29, 'common Esfand virtual anchor');
parts(longMonth.until(longMonth.add({days: 15}), {largestUnit: 'month', smallestUnit: 'month', roundingMode: 'halfExpand'}), 0, 0, 0, 0, 'below month half');
parts(longMonth.until(longMonth.add({days: 16}), {largestUnit: 'month', smallestUnit: 'month', roundingMode: 'halfExpand'}), 0, 1, 0, 0, 'above month half');
parts(leapStart.until(leapStart.add({days: 183}), {largestUnit: 'year', smallestUnit: 'year', roundingMode: 'halfExpand'}), 1, 0, 0, 0, 'leap-year tie');
for (const delta of [{days: 40}, {weeks: -3}]) {
  same(longMonth.add(delta).withCalendar('iso8601').equals(longMonth.withCalendar('iso8601').add(delta)), true, 'epoch-only addition');
}
const time = longMonth.toPlainDateTime({hour: 23, minute: 30});
const timeEnd = time.add({months: 1, hours: 2});
same(timeEnd.month, 3, 'datetime month');
same(timeEnd.day, 2, 'datetime day carry');
same(timeEnd.hour, 1, 'datetime clock carry');
same(time.until(time.add({months: 1}), {largestUnit: 'month'}).months, 1, 'datetime difference');
const ym = longMonth.toPlainYearMonth();
const ymEnd = ym.add({years: 2, months: 3});
same(ymEnd.year, 1402, 'partial year');
same(ymEnd.month, 5, 'partial month');
same(ym.until(ymEnd, {largestUnit: 'month'}).months, 27, 'partial difference');
same(ymEnd.subtract({years: 2, months: 3}).equals(ym), true, 'partial negative addition');
const zoned = Temporal.ZonedDateTime.from('2024-03-20T12:00[UTC][u-ca=persian]');
const zonedEnd = zoned.add({years: 1});
same(zonedEnd.toPlainDate().withCalendar('iso8601').toString(), '2025-03-21', 'zoned calendar year');
same(zoned.until(zonedEnd, {largestUnit: 'year'}).years, 1, 'zoned calendar difference');
same(zoned.until(zonedEnd, {largestUnit: 'day'}).days, 366, 'zoned actual days');
for (const calendar of ['iso8601', 'gregory', 'buddhist', 'roc', 'japanese']) {
  same(new Temporal.PlainDate(2020, 1, 31, calendar).add({months: 1}).withCalendar('iso8601').toString(), '2020-02-29', 'retained Gregorian domain');
}
same(Temporal.PlainDate.from({calendar: 'indian', year: 1942, month: 1, day: 1}).add({years: 1}).withCalendar('iso8601').toString(), '2021-03-22', 'retained Indian domain');
print('persian-arithmetic:ok');
262;
