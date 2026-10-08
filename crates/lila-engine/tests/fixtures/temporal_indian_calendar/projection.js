function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function fields(value, year, month, day, dayOfYear, daysInMonth, leap) {
  same(value.calendarId, 'indian', 'calendar');
  same(value.year, year, 'year');
  same(value.month, month, 'month');
  same(value.monthCode, 'M' + (month < 10 ? '0' : '') + month, 'month code');
  same(value.day, day, 'day');
  same(value.dayOfYear, dayOfYear, 'ordinal');
  same(value.daysInMonth, daysInMonth, 'month length');
  same(value.daysInYear, leap ? 366 : 365, 'year length');
  same(value.monthsInYear, 12, 'month count');
  same(value.inLeapYear, leap, 'leap');
  same(value.era, 'shaka', 'era');
  same(value.eraYear, year, 'era year');
  same(value.weekOfYear, undefined, 'non-ISO week');
  same(value.yearOfWeek, undefined, 'non-ISO week year');
}
const boundaries = [
  ['2020-03-20', 1941, 12, 30, 365, 30, false],
  ['2020-03-21', 1942, 1, 1, 1, 31, true],
  ['2021-03-21', 1942, 12, 30, 366, 30, true],
  ['2021-03-22', 1943, 1, 1, 1, 30, false],
  ['2020-04-20', 1942, 1, 31, 31, 31, true],
  ['2020-04-21', 1942, 2, 1, 32, 31, true],
  ['2020-09-23', 1942, 7, 1, 187, 30, true]
];
for (const row of boundaries) {
  const date = Temporal.PlainDate.from(row[0] + '[u-ca=INDIAN]');
  const time = Temporal.PlainDateTime.from(row[0] + 'T12:34:56[u-ca=indian]');
  const zoned = Temporal.ZonedDateTime.from(row[0] + 'T12:34:56[UTC][u-ca=indian]');
  for (const carrier of [date, time, zoned]) {
    fields(carrier, row[1], row[2], row[3], row[4], row[5], row[6]);
    same(carrier.dayOfWeek, date.withCalendar('iso8601').dayOfWeek, 'epoch weekday');
  }
  same(time.toPlainDate().equals(date), true, 'datetime date');
  same(zoned.toPlainDate().equals(date), true, 'zoned date');
  same(zoned.toPlainDateTime().equals(time), true, 'zoned datetime');
  const ym = date.toPlainYearMonth();
  same(ym.year, row[1], 'partial year');
  same(ym.month, row[2], 'partial month');
  same(ym.daysInMonth, row[5], 'partial month length');
  same(ym.era, 'shaka', 'partial era');
  const md = date.toPlainMonthDay();
  same(md.monthCode, date.monthCode, 'partial month code');
  same(md.day, row[3], 'partial day');
}
for (const year of [-1, 0, 1]) {
  const date = Temporal.PlainDate.from({calendar: 'indian', era: 'shaka', eraYear: year, monthCode: 'M01', day: 1});
  same(date.year, year, 'signed calendar year');
  same(date.eraYear, year, 'signed era year');
  const iso = date.withCalendar('iso8601');
  same(iso.year, year + 78, 'signed ISO year');
  same(iso.month, 3, 'signed ISO month');
  same(iso.day, 22, 'signed ISO day');
}
const constructor = new Temporal.PlainDate(2020, 3, 21, 'indian');
same(constructor.year, 1942, 'constructor keeps ISO arguments');
same(constructor.equals(Temporal.PlainDate.from({calendar: 'indian', year: 1942, month: 1, day: 1})), true, 'bag uses calendar arguments');
print('indian-projection:ok');
262;
