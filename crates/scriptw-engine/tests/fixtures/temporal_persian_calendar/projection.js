function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function fields(value, year, month, day, ordinal, length, leap) {
  same(value.calendarId, 'persian', 'calendar');
  same(value.year, year, 'year');
  same(value.month, month, 'month');
  same(value.monthCode, 'M' + (month < 10 ? '0' : '') + month, 'month code');
  same(value.day, day, 'day');
  same(value.dayOfYear, ordinal, 'ordinal');
  same(value.daysInMonth, length, 'month length');
  same(value.daysInYear, leap ? 366 : 365, 'year length');
  same(value.monthsInYear, 12, 'month count');
  same(value.inLeapYear, leap, 'leap');
  same(value.era, 'ap', 'era');
  same(value.eraYear, year, 'era year');
  same(value.weekOfYear, undefined, 'non-ISO week');
  same(value.yearOfWeek, undefined, 'non-ISO week year');
}
// ICU's pinned university leap table and independent RD test vectors.
const boundaries = [
  ['2020-03-20', 1399, 1, 1, 1, 31, true],
  ['2021-03-20', 1399, 12, 30, 366, 30, true],
  ['2021-03-21', 1400, 1, 1, 1, 31, false],
  ['2021-09-22', 1400, 6, 31, 186, 31, false],
  ['2021-09-23', 1400, 7, 1, 187, 30, false],
  ['2024-03-20', 1403, 1, 1, 1, 31, true],
  ['2025-03-20', 1403, 12, 30, 366, 30, true],
  ['2025-03-21', 1404, 1, 1, 1, 31, false],
  ['2124-03-19', 1502, 12, 29, 365, 29, false],
  ['2124-03-20', 1503, 1, 1, 1, 31, true]
];
for (const row of boundaries) {
  const date = Temporal.PlainDate.from(row[0] + '[u-ca=PERSIAN]');
  const time = Temporal.PlainDateTime.from(row[0] + 'T12:34:56[u-ca=persian]');
  const zoned = Temporal.ZonedDateTime.from(row[0] + 'T12:34:56[UTC][u-ca=persian]');
  for (const carrier of [date, time, zoned]) {
    fields(carrier, row[1], row[2], row[3], row[4], row[5], row[6]);
    same(carrier.dayOfWeek, date.withCalendar('iso8601').dayOfWeek, 'epoch weekday');
  }
  same(time.toPlainDate().equals(date), true, 'datetime date');
  same(zoned.toPlainDateTime().equals(time), true, 'zoned datetime');
  const ym = date.toPlainYearMonth();
  same(ym.year, row[1], 'partial year');
  same(ym.month, row[2], 'partial month');
  same(ym.daysInMonth, row[5], 'partial length');
  same(ym.era, 'ap', 'partial era');
  const md = date.toPlainMonthDay();
  same(md.monthCode, date.monthCode, 'partial month code');
  same(md.day, row[3], 'partial day');
}
const signed = [[-1, '0620-03-21', false], [0, '0621-03-21', false], [1, '0622-03-21', true]];
for (const row of signed) {
  const date = Temporal.PlainDate.from({calendar: 'persian', era: 'ap', eraYear: row[0], monthCode: 'M01', day: 1});
  fields(date, row[0], 1, 1, 1, 31, row[2]);
  same(date.withCalendar('iso8601').toString(), row[1], 'signed ISO conversion');
  same(Temporal.PlainDate.from(row[1] + '[u-ca=persian]').equals(date), true, 'signed inverse');
}
const constructor = new Temporal.PlainDate(2025, 3, 20, 'persian');
same(constructor.year, 1403, 'constructor ISO coordinates');
same(constructor.equals(Temporal.PlainDate.from({calendar: 'persian', year: 1403, month: 12, day: 30})), true, 'bag calendar coordinates');
print('persian-projection:ok');
262;
