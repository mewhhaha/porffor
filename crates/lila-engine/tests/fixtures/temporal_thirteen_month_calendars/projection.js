function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function fields(value, calendar, year, month, day, ordinal, length, leap) {
  same(value.calendarId, calendar, 'canonical calendar');
  same(value.year, year, 'arithmetic year');
  same(value.month, month, 'month');
  same(value.monthCode, 'M' + (month < 10 ? '0' : '') + month, 'month code');
  same(value.day, day, 'day');
  same(value.dayOfYear, ordinal, 'ordinal');
  same(value.daysInMonth, length, 'month length');
  same(value.daysInYear, leap ? 366 : 365, 'year length');
  same(value.monthsInYear, 13, 'month count');
  same(value.inLeapYear, leap, 'leap');
  same(value.era, calendar === 'coptic' ? 'am' : calendar === 'ethioaa' || year <= 0 ? 'aa' : 'am', 'era');
  same(value.eraYear, calendar === 'ethiopic' && year <= 0 ? year + 5500 : year, 'era year');
  same(value.weekOfYear, undefined, 'non-ISO week');
  same(value.yearOfWeek, undefined, 'non-ISO week year');
}
// The first and last rows are pinned Test262 full-date roundtrip vectors.
const rows = [
  ['2000-01-01', 1716, 4, 22, 112, 30, false],
  ['2023-09-11', 1739, 13, 6, 366, 6, true],
  ['2023-09-12', 1740, 1, 1, 1, 30, false],
  ['2024-09-10', 1740, 13, 5, 365, 5, false],
  ['2024-09-11', 1741, 1, 1, 1, 30, false],
  ['0001-01-01', -283, 5, 8, 128, 30, false]
];
const families = [['coptic', 0], ['ethiopic', 276], ['ethioaa', 5776]];
for (const [calendar, offset] of families) {
  for (const row of rows) {
    const date = Temporal.PlainDate.from(row[0] + '[u-ca=' + calendar.toUpperCase() + ']');
    const time = Temporal.PlainDateTime.from(row[0] + 'T12:34:56[u-ca=' + calendar + ']');
    const zoned = Temporal.ZonedDateTime.from(row[0] + 'T12:34:56[UTC][u-ca=' + calendar + ']');
    for (const carrier of [date, time, zoned]) {
      fields(carrier, calendar, row[1] + offset, row[2], row[3], row[4], row[5], row[6]);
      same(carrier.dayOfWeek, date.withCalendar('iso8601').dayOfWeek, 'epoch weekday');
    }
    same(time.toPlainDate().equals(date), true, 'datetime date');
    same(zoned.toPlainDateTime().equals(time), true, 'zoned datetime');
    const ym = date.toPlainYearMonth();
    same(ym.year, row[1] + offset, 'YearMonth year');
    same(ym.month, row[2], 'YearMonth month');
    same(ym.monthCode, date.monthCode, 'YearMonth code');
    same(ym.monthsInYear, 13, 'YearMonth count');
    same(ym.daysInMonth, row[5], 'YearMonth length');
    const md = date.toPlainMonthDay();
    same(md.monthCode, date.monthCode, 'MonthDay code');
    same(md.day, row[3], 'MonthDay day');
  }
  for (const year of [-1, 0, 1]) {
    const era = calendar === 'coptic' || calendar === 'ethiopic' ? 'am' : 'aa';
    const date = Temporal.PlainDate.from({calendar, era, eraYear: year, monthCode: 'M01', day: 1});
    fields(date, calendar, year, 1, 1, 1, 30, year === -1);
    same(Temporal.PlainDate.from(date.toString()).equals(date), true, 'signed inverse');
  }
}
for (const [eraYear, year] of [[-1, -5501], [0, -5500], [1, -5499], [5500, 0], [5501, 1]]) {
  const date = Temporal.PlainDate.from({calendar: 'ethiopic', era: 'aa', eraYear, monthCode: 'M01', day: 1});
  same(date.year, year, 'Ethiopic AA conversion');
  same(date.era, year <= 0 ? 'aa' : 'am', 'Ethiopic era remapping');
  same(date.eraYear, year <= 0 ? eraYear : year, 'Ethiopic era-year remapping');
}
for (const [calendar, epochYear] of [['coptic', 283], ['ethiopic', 7], ['ethioaa', -5493]]) {
  const start = new Temporal.PlainDate(epochYear, 12, 31).withCalendar(calendar).with({monthCode: 'M01', day: 1});
  same(start.year, 0, 'arithmetic epoch year');
}
const alias = new Temporal.PlainDate(2024, 9, 11, 'ETHIOPIC-AMETE-ALEM');
same(alias.calendarId, 'ethioaa', 'Amete Alem alias');
same(alias.year, 7517, 'alias projection');
same(alias.equals(Temporal.PlainDate.from({calendar: 'ethioaa', year: 7517, month: 1, day: 1})), true, 'ISO constructor versus calendar bag');
print('thirteen-month-projection:ok');
262;
