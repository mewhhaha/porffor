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
  same(value.monthsInYear, 12, 'fixed month count');
  same(value.daysInYear, leap ? 355 : 354, 'lunar year length');
  same(value.inLeapYear, leap, 'leap phase');
  same(value.era, year > 0 ? 'ah' : 'bh', 'reported era');
  same(value.eraYear, year > 0 ? year : 1 - year, 'reported era year');
  same(value.weekOfYear, undefined, 'non-ISO week');
  same(value.yearOfWeek, undefined, 'non-ISO week year');
}
// The first two pairs are literal pinned Test262 values, not a runtime oracle.
for (const [calendar, offset, epoch, secondYear, thirdYear, reference] of [
  ['islamic-civil', 0, '0622-07-19', '0623-07-08', '0624-06-27', '1972-02-16'],
  ['islamic-tbla', 1, '0622-07-18', '0623-07-07', '0624-06-26', '1972-02-15']
]) {
  for (const [iso, year, month, day, ordinal, length, leap] of [
    ['2000-01-01', 1420, 9, 24 + offset, 260 + offset, 30, true],
    ['0001-01-01', -640, 5, 18 + offset, 136 + offset, 30, false],
    [epoch, 1, 1, 1, 1, 30, false],
    [secondYear, 2, 1, 1, 1, 30, true],
    [thirdYear, 3, 1, 1, 1, 30, false],
    [reference, 1392, 1, 1, 1, 30, false]
  ]) {
    const plain = Temporal.PlainDate.from(iso + '[u-ca=' + calendar.toUpperCase() + ']');
    const time = Temporal.PlainDateTime.from(iso + 'T12:34:56[u-ca=' + calendar + ']');
    const zoned = Temporal.ZonedDateTime.from(iso + 'T12:34:56[UTC][u-ca=' + calendar + ']');
    for (const value of [plain, time, zoned]) fields(value, calendar, year, month, day, ordinal, length, leap);
    same(plain.dayOfWeek, plain.withCalendar('iso8601').dayOfWeek, 'retained ISO weekday');
    same(zoned.toPlainDate().equals(plain), true, 'zoned projection');
    same(time.toPlainDate().equals(plain), true, 'datetime projection');
    same(Temporal.PlainDate.from({calendar, year, monthCode: plain.monthCode, day}, {overflow: 'reject'}).equals(plain), true, 'inverse full date');
    const ym = plain.toPlainYearMonth();
    same(ym.year, year, 'YearMonth projection');
    same(ym.month, month, 'YearMonth ordinal');
    same(ym.monthsInYear, 12, 'YearMonth count');
    same(ym.daysInYear, leap ? 355 : 354, 'YearMonth lunar length');
    same(plain.toPlainMonthDay().day, day, 'MonthDay projection');
  }
  const first = Temporal.PlainDate.from(epoch + '[u-ca=' + calendar + ']');
  fields(first.subtract({days: 1}), calendar, 0, 12, 29, 354, 29, false);
  fields(first.add({days: 354}), calendar, 2, 1, 1, 1, 30, true);
  fields(Temporal.PlainDate.from(secondYear + '[u-ca=' + calendar + ']').subtract({days: 1}), calendar, 1, 12, 29, 354, 29, false);
  fields(Temporal.PlainDate.from(thirdYear + '[u-ca=' + calendar + ']').subtract({days: 1}), calendar, 2, 12, 30, 355, 30, true);
  for (const [era, eraYear, year] of [['ah', -1, -1], ['ah', 0, 0], ['ah', 1, 1], ['bh', -1, 2], ['bh', 0, 1], ['bh', 1, 0], ['bh', 2, -1]]) {
    const date = Temporal.PlainDate.from({calendar, era, eraYear, monthCode: 'M01', day: 1}, {overflow: 'reject'});
    fields(date, calendar, year, 1, 1, 1, 30, year === -1 || year === 2);
    same(Temporal.PlainDate.from(date.toString()).equals(date), true, 'signed annotated round trip');
  }
}
same(new Temporal.PlainDate(2000, 1, 1, 'ISLAMICC').calendarId, 'islamic-civil', 'civil alias');
same(Temporal.PlainDate.from({calendar: 'islamicc', year: 1420, month: 9, day: 24}).withCalendar('iso8601').toString(), '2000-01-01', 'alias coordinates');
print('islamic-tabular-projection:ok');
262;
