function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function parts(actual, years, months, weeks, days, label) {
  same(actual.years, years, label + ' years');
  same(actual.months, months, label + ' months');
  same(actual.weeks, weeks, label + ' weeks');
  same(actual.days, days, label + ' days');
}
function range(action) {
  try {action();} catch (caught) {
    if (!(caught instanceof RangeError)) throw new Error('wrong arithmetic error');
    return;
  }
  throw new Error('missing arithmetic RangeError');
}
for (const [calendar, commonYear] of [['coptic', 1740], ['ethiopic', 2016], ['ethioaa', 7516]]) {
  const date = (year, month, day) => Temporal.PlainDate.from({calendar, year, month, day});
  const leapStart = date(commonYear - 1, 1, 1);
  const commonStart = date(commonYear, 1, 1);
  same(leapStart.until(commonStart).days, 366, 'leap-year span');
  parts(leapStart.until(commonStart, {largestUnit: 'year'}), 1, 0, 0, 0, 'exact year');
  parts(leapStart.until(commonStart, {largestUnit: 'month'}), 0, 13, 0, 0, 'thirteen months');
  same(leapStart.add({months: 13}).equals(commonStart), true, 'positive thirteen-month balance');
  same(commonStart.subtract({months: 13}).equals(leapStart), true, 'negative thirteen-month balance');
  same(commonStart.add({months: -13}).equals(leapStart), true, 'signed month addition');
  const leapEnd = date(commonYear - 1, 13, 6);
  same(leapEnd.add({years: 1}).day, 5, 'year constrain');
  range(() => leapEnd.add({years: 1}, {overflow: 'reject'}));
  same(date(commonYear, 13, 5).add({days: 1}).equals(date(commonYear + 1, 1, 1)), true, 'short-month rollover');
  for (const year of [-1, 0]) {
    const start = date(year, 1, 1);
    const next = start.add({years: 1});
    same(next.year, year + 1, 'signed year balance');
    parts(start.until(next, {largestUnit: 'month'}), 0, 13, 0, 0, 'signed full year');
    same(next.subtract({months: 13}).equals(start), true, 'signed inverse month balance');
  }
  const lastLongDay = date(commonYear, 12, 30);
  const shortEnd = lastLongDay.add({months: 1});
  same(shortEnd.month, 13, 'short month');
  same(shortEnd.day, 5, 'short day');
  parts(lastLongDay.until(shortEnd, {largestUnit: 'month'}), 0, 0, 0, 5, 'positive virtual anchor');
  parts(shortEnd.until(lastLongDay, {largestUnit: 'month'}), 0, 0, 0, -5, 'negative virtual anchor');
  parts(date(commonYear - 1, 12, 30).until(leapEnd, {largestUnit: 'month'}), 0, 0, 0, 6, 'leap virtual anchor');
  // A nudge leaving twelve months must not bubble through a twelve-month year.
  const nearEnd = date(commonYear, 13, 2);
  parts(commonStart.until(nearEnd, {largestUnit: 'year', smallestUnit: 'month', roundingMode: 'trunc'}), 0, 12, 0, 0, 'twelve residual months');
  parts(nearEnd.until(commonStart, {largestUnit: 'year', smallestUnit: 'month', roundingMode: 'trunc'}), 0, -12, 0, 0, 'negative residual months');
  parts(commonStart.until(date(commonYear, 13, 4), {largestUnit: 'year', smallestUnit: 'month', roundingMode: 'halfExpand'}), 1, 0, 0, 0, 'thirteen-month expanded carry');
  const ym = commonStart.toPlainYearMonth();
  const nextYear = ym.add({months: 13});
  same(nextYear.year, commonYear + 1, 'YearMonth thirteen-month carry');
  same(nextYear.month, 1, 'YearMonth carried month');
  same(ym.until(nextYear, {largestUnit: 'month'}).months, 13, 'YearMonth month difference');
  parts(ym.until(ym.add({months: 26}), {largestUnit: 'year'}), 2, 0, 0, 0, 'YearMonth whole years');
  parts(ym.until(ym.add({months: 24}), {largestUnit: 'year', smallestUnit: 'month', roundingIncrement: 2, roundingMode: 'ceil'}), 1, 12, 0, 0, 'YearMonth retained-year rounding');
  parts(ym.until(ym.add({months: 6}), {largestUnit: 'year', smallestUnit: 'year', roundingMode: 'halfExpand'}), 0, 0, 0, 0, 'YearMonth below year half');
  parts(ym.until(ym.add({months: 7}), {largestUnit: 'year', smallestUnit: 'year', roundingMode: 'halfExpand'}), 1, 0, 0, 0, 'YearMonth above year half');
  const time = date(commonYear, 12, 30).toPlainDateTime({hour: 23});
  const timeEnd = time.add({months: 1, hours: 2});
  same(timeEnd.month, 1, 'datetime carried month');
  same(timeEnd.year, commonYear + 1, 'datetime carried year');
  same(timeEnd.day, 1, 'datetime carried day');
  same(timeEnd.hour, 1, 'datetime carried hour');
  const zoned = Temporal.ZonedDateTime.from({calendar, year: commonYear - 1, month: 1, day: 1, hour: 12, timeZone: 'UTC'});
  const zonedEnd = zoned.add({months: 13});
  same(zonedEnd.year, commonYear, 'zoned year');
  same(zoned.until(zonedEnd, {largestUnit: 'month'}).months, 13, 'zoned month count');
  same(zoned.until(zonedEnd, {largestUnit: 'day'}).days, 366, 'zoned day span');
}
same(new Temporal.PlainDate(2020, 1, 31).add({months: 1}).toString(), '2020-02-29', 'retained ISO arithmetic');
same(Temporal.PlainDate.from({calendar: 'indian', year: 1942, month: 1, day: 1}).add({years: 1}).withCalendar('iso8601').toString(), '2021-03-22', 'retained Indian arithmetic');
same(Temporal.PlainDate.from({calendar: 'persian', year: 1403, month: 1, day: 1}).add({months: 12}).withCalendar('iso8601').toString(), '2025-03-21', 'retained Persian twelve months');
print('thirteen-month-arithmetic:ok');
262;
