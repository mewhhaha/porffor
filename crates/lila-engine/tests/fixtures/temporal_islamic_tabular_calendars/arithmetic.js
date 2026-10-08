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
for (const calendar of ['islamic-civil', 'islamic-tbla']) {
  const date = (year, month, day) => Temporal.PlainDate.from({calendar, year, month, day});
  const leapStart = date(1390, 1, 1);
  const commonStart = date(1391, 1, 1);
  same(leapStart.until(commonStart).days, 355, 'lunar leap span');
  same(commonStart.until(date(1392, 1, 1)).days, 354, 'lunar common span');
  parts(leapStart.until(commonStart, {largestUnit: 'year'}), 1, 0, 0, 0, 'exact year');
  parts(leapStart.until(commonStart, {largestUnit: 'month'}), 0, 12, 0, 0, 'twelve months');
  same(leapStart.add({months: 12}).equals(commonStart), true, 'positive year balance');
  same(commonStart.add({months: -12}).equals(leapStart), true, 'negative year balance');
  same(commonStart.subtract({months: 12}).equals(leapStart), true, 'month subtraction');
  const leapEnd = date(1390, 12, 30);
  same(leapEnd.add({years: 1}).day, 29, 'common year constrain');
  range(() => leapEnd.add({years: 1}, {overflow: 'reject'}));
  same(date(1391, 12, 29).add({days: 1}).equals(date(1392, 1, 1)), true, 'common year rollover');
  same(leapEnd.add({days: 1}).equals(commonStart), true, 'leap year rollover');
  for (const year of [-1, 0, 1, 2]) {
    const start = date(year, 1, 1);
    const next = start.add({years: 1});
    same(next.year, year + 1, 'signed year balance');
    same(start.until(next).days, year === -1 || year === 2 ? 355 : 354, 'signed lunar length');
    parts(start.until(next, {largestUnit: 'month'}), 0, 12, 0, 0, 'signed month count');
    same(next.subtract({months: 12}).equals(start), true, 'signed inverse');
  }
  const longEnd = date(1392, 1, 30);
  const shortEnd = longEnd.add({months: 1});
  same(shortEnd.month, 2, 'short destination month');
  same(shortEnd.day, 29, 'short destination day');
  parts(longEnd.until(shortEnd, {largestUnit: 'month'}), 0, 0, 0, 29, 'positive virtual anchor');
  parts(shortEnd.until(longEnd, {largestUnit: 'month'}), 0, 0, 0, -29, 'negative virtual anchor');
  parts(date(1390, 11, 30).until(leapEnd, {largestUnit: 'month'}), 0, 1, 0, 0, 'valid leap anchor');
  parts(date(1391, 11, 30).until(date(1391, 12, 29), {largestUnit: 'month'}), 0, 0, 0, 29, 'common last virtual anchor');
  const start = date(1392, 1, 1);
  parts(start.until(date(1392, 12, 16), {largestUnit: 'year', smallestUnit: 'month', roundingMode: 'trunc'}), 0, 11, 0, 0, 'residual lunar months');
  parts(start.until(date(1392, 12, 16), {largestUnit: 'year', smallestUnit: 'month', roundingMode: 'halfExpand'}), 1, 0, 0, 0, 'twelve-month expanded carry');
  const ym = start.toPlainYearMonth();
  same(ym.add({months: 12}).year, 1393, 'YearMonth year carry');
  same(ym.add({months: 12}).month, 1, 'YearMonth carried month');
  same(ym.until(ym.add({months: 12}), {largestUnit: 'month'}).months, 12, 'YearMonth month difference');
  parts(ym.until(ym.add({months: 24}), {largestUnit: 'year'}), 2, 0, 0, 0, 'YearMonth whole years');
  parts(ym.until(ym.add({months: 23}), {largestUnit: 'year', smallestUnit: 'month', roundingIncrement: 2, roundingMode: 'ceil'}), 2, 0, 0, 0, 'YearMonth increment carry');
  parts(ym.until(ym.add({months: 5}), {largestUnit: 'year', smallestUnit: 'year', roundingMode: 'halfExpand'}), 0, 0, 0, 0, 'below lunar year half');
  parts(ym.until(ym.add({months: 7}), {largestUnit: 'year', smallestUnit: 'year', roundingMode: 'halfExpand'}), 1, 0, 0, 0, 'above lunar year half');
  const time = date(1392, 11, 30).toPlainDateTime({hour: 23}).add({months: 1, hours: 2});
  same(time.year, 1393, 'datetime carried year');
  same(time.month, 1, 'datetime carried month');
  same(time.day, 1, 'datetime carried day');
  same(time.hour, 1, 'datetime carried hour');
  const zoned = Temporal.ZonedDateTime.from({calendar, year: 1390, month: 1, day: 1, hour: 12, timeZone: 'UTC'});
  const end = zoned.add({months: 12});
  same(end.year, 1391, 'zoned year');
  same(zoned.until(end, {largestUnit: 'month'}).months, 12, 'zoned month count');
  same(zoned.until(end, {largestUnit: 'day'}).days, 355, 'zoned lunar length');
}
same(new Temporal.PlainDate(2020, 1, 31).add({months: 1}).toString(), '2020-02-29', 'retained ISO arithmetic');
same(Temporal.PlainDate.from({calendar: 'coptic', year: 1739, month: 1, day: 1}).add({months: 13}).year, 1740, 'retained thirteen-month count');
same(Temporal.PlainDate.from({calendar: 'indian', year: 1942, month: 1, day: 1}).add({years: 1}).withCalendar('iso8601').toString(), '2021-03-22', 'retained Indian arithmetic');
same(Temporal.PlainDate.from({calendar: 'persian', year: 1403, month: 1, day: 1}).add({months: 12}).withCalendar('iso8601').toString(), '2025-03-21', 'retained Persian arithmetic');
print('islamic-tabular-arithmetic:ok');
262;
