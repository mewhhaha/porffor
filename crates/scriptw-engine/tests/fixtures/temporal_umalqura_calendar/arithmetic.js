function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function range(action) {
  try {action();} catch (caught) {if (caught instanceof RangeError) return; throw caught;}
  throw new Error('missing arithmetic range error');
}
const calendar = 'islamic-umalqura';
const make = (year, month, day) => Temporal.PlainDate.from({calendar, year, month, day});
function duration(value, years, months, days, label) {
  same(value.years, years, label + ' years');
  same(value.months, months, label + ' months');
  same(value.weeks, 0, label + ' weeks');
  same(value.days, days, label + ' days');
}
for (const [year, days] of [[1390,355], [1391,354], [1392,355], [1600,354]]) {
  const start = make(year, 1, 1);
  const next = make(year + 1, 1, 1);
  same(start.add({years: 1}).equals(next), true, 'year forward');
  same(start.add({months: 12}).equals(next), true, 'twelve months forward');
  same(next.subtract({months: 12}).equals(start), true, 'twelve months backward');
  duration(start.until(next, {largestUnit: 'days'}), 0, 0, days, 'actual year days');
  duration(next.until(start, {largestUnit: 'days'}), 0, 0, -days, 'negative actual year days');
  duration(start.until(next, {largestUnit: 'years'}), 1, 0, 0, 'year unit');
  duration(next.until(start, {largestUnit: 'months'}), 0, -12, 0, 'negative months');
}
for (const year of [-1, 0, 1, 2]) {
  const start = make(year, 1, 1);
  const next = start.add({years: 1});
  same(start.add({months: 12}).equals(next), true, 'signed twelve months');
  same(next.subtract({years: 1}).equals(start), true, 'signed year inverse');
  duration(start.until(next, {largestUnit: 'days'}), 0, 0, year === -1 || year === 2 ? 355 : 354, 'signed civil year days');
}
const long = make(1392, 1, 30);
const short = make(1392, 2, 29);
same(long.add({months: 1}).equals(short), true, 'constrained table month');
range(() => long.add({months: 1}, {overflow: 'reject'}));
duration(long.until(short, {largestUnit: 'months'}), 0, 0, 29, 'forward virtual anchor before clamp');
duration(short.until(long, {largestUnit: 'months'}), 0, 0, -29, 'backward virtual anchor before clamp');
const leapEnd = make(1390, 12, 30);
same(leapEnd.add({years: 1}).day, 29, 'table destination year regulation');
range(() => leapEnd.add({years: 1}, {overflow: 'reject'}));
same(leapEnd.add({days: 1}).equals(make(1391, 1, 1)), true, 'long last month rollover');
same(make(1391, 12, 29).add({days: 1}).equals(make(1392, 1, 1)), true, 'short last month rollover');
const start = make(1392, 1, 1);
const nearEnd = make(1392, 12, 16);
same(start.until(nearEnd, {largestUnit: 'years', smallestUnit: 'months', roundingMode: 'trunc'}).months, 11, 'month truncation');
same(start.until(nearEnd, {largestUnit: 'years', smallestUnit: 'years', roundingMode: 'halfExpand'}).years, 1, 'real-year midpoint');
const ym = Temporal.PlainYearMonth.from({calendar, year: 1392, monthCode: 'M12'});
same(ym.add({months: 1}).year, 1393, 'YearMonth carry year');
same(ym.add({months: 1}).month, 1, 'YearMonth carry month');
same(ym.until(ym.add({months: 24}), {largestUnit: 'years'}).years, 2, 'YearMonth count');
same(ym.until(ym.add({months: 23}), {largestUnit: 'years', smallestUnit: 'months', roundingIncrement: 2, roundingMode: 'ceil'}).years, 2, 'YearMonth rounding carry');
const time = make(1392, 11, 30).toPlainDateTime({hour: 23}).add({months: 1, hours: 2});
same(time.year, 1393, 'DateTime next table year');
same(time.month, 1, 'DateTime destination month carry');
same(time.day, 1, 'DateTime day carry');
// A long destination last month reaches the next year after the extra day.
same(time.toPlainDate().equals(make(1393, 1, 1)), true, 'DateTime date carry');
same(time.hour, 1, 'DateTime hour carry');
const zoned = Temporal.ZonedDateTime.from('1972-02-16T12:00[UTC][u-ca=islamic-umalqura]');
same(zoned.add({months: 12}).year, 1393, 'zoned count carry');
same(zoned.until(zoned.add({months: 12}), {largestUnit: 'days'}).days, 355, 'zoned actual year');
same(new Temporal.PlainDate(2020, 1, 31).add({months: 1}).toString(), '2020-02-29', 'retained ISO month');
same(Temporal.PlainDate.from({calendar: 'coptic', year: 1739, month: 1, day: 1}).add({months: 13}).year, 1740, 'retained thirteen count');
print('umalqura-arithmetic:ok');
262;
