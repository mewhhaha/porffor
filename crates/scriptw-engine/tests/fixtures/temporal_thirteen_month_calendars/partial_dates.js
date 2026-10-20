function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function range(action) {
  try {action();} catch (caught) {
    if (!(caught instanceof RangeError)) throw new Error('wrong partial error');
    return;
  }
  throw new Error('missing partial RangeError');
}
for (const [calendar, commonYear, era] of [['coptic', 1740, 'am'], ['ethiopic', 2016, 'am'], ['ethioaa', 7516, 'aa']]) {
  const references = [
    ['M01', 1, '1972-09-11'],
    ['M04', 22, '1972-12-31'],
    ['M04', 23, '1972-01-02'],
    ['M13', 5, '1972-09-10'],
    ['M13', 6, '1971-09-11']
  ];
  for (const [monthCode, day, iso] of references) {
    const md = Temporal.PlainMonthDay.from({calendar, monthCode, day});
    same(md.toString(), iso + '[u-ca=' + calendar + ']', 'latest reference');
    same(md.monthCode, monthCode, 'reference code');
    same(md.day, day, 'reference day');
    same(Temporal.PlainMonthDay.from(md.toString()).equals(md), true, 'parsed reference');
    const date = md.toPlainDate({year: commonYear - 1});
    same(date.year, commonYear - 1, 'supplied calendar year');
    same(date.monthCode, monthCode, 'supplied calendar month');
    same(date.day, day, 'supplied calendar day');
    same(date.toPlainMonthDay().equals(md), true, 'date reference');
    same(date.toPlainDateTime({hour: 12}).toPlainDate().toPlainMonthDay().equals(md), true, 'datetime reference');
  }
  const leap = Temporal.PlainMonthDay.from({calendar, monthCode: 'M13', day: 6});
  same(Temporal.PlainMonthDay.from({calendar, monthCode: 'M13', day: 7}).equals(leap), true, 'missing-year constrain uses a leap year');
  range(() => Temporal.PlainMonthDay.from({calendar, monthCode: 'M13', day: 7}, {overflow: 'reject'}));
  same(Temporal.PlainMonthDay.from({calendar, year: commonYear, monthCode: 'M13', day: 6}).day, 5, 'supplied-year regulation');
  same(leap.with({year: commonYear}).toString(), '1972-09-10[u-ca=' + calendar + ']', 'with year before reference');
  same(leap.toPlainDate({era, eraYear: commonYear}).withCalendar('iso8601').toString(), '2024-09-10', 'era supplied year');
  const ym = Temporal.PlainYearMonth.from({calendar, year: commonYear, monthCode: 'M13'});
  same(ym.toString(), '2024-09-06[u-ca=' + calendar + ']', 'calendar day-one reference');
  same(ym.monthsInYear, 13, 'partial month count');
  same(ym.toPlainDate({day: 5}).withCalendar('iso8601').toString(), '2024-09-10', 'partial day completion');
  same(Temporal.PlainYearMonth.from('2024-09-10[u-ca=' + calendar + ']').equals(ym), true, 'parsed YearMonth normalization');
  same(ym.with({monthCode: 'M01'}).toString(), '2023-09-12[u-ca=' + calendar + ']', 'with month reference');
  same(ym.subtract({months: 1}).toString(), '2024-08-07[u-ca=' + calendar + ']', 'preceding month reference');
  same(ym.add({months: 1}).toString(), '2024-09-11[u-ca=' + calendar + ']', 'following year reference');
  const explicitYm = new Temporal.PlainYearMonth(2024, 9, calendar, 10);
  same(explicitYm.toString(), '2024-09-10[u-ca=' + calendar + ']', 'explicit YearMonth ISO reference');
  same(Temporal.PlainYearMonth.from(explicitYm).toString(), explicitYm.toString(), 'YearMonth clone preserves explicit reference');
  const explicitMd = new Temporal.PlainMonthDay(9, 11, calendar, 2023);
  same(explicitMd.toString(), '2023-09-11[u-ca=' + calendar + ']', 'explicit MonthDay ISO reference');
  same(explicitMd.monthCode, 'M13', 'explicit MonthDay projected month');
  same(explicitMd.day, 6, 'explicit MonthDay projected day');
  same(Temporal.PlainMonthDay.from(explicitMd).toString(), explicitMd.toString(), 'MonthDay clone preserves explicit reference');
  const zoned = Temporal.ZonedDateTime.from('2023-09-11T12:00[UTC][u-ca=' + calendar + ']');
  same(zoned.toPlainDate().toPlainMonthDay().equals(leap), true, 'zoned leap reference');
}
const alias = Temporal.PlainMonthDay.from({calendar: 'ethiopic-amete-alem', monthCode: 'M13', day: 6});
same(alias.calendarId, 'ethioaa', 'partial canonical alias');
same(alias.toString(), '1971-09-11[u-ca=ethioaa]', 'alias leap reference');
print('thirteen-month-partial-dates:ok');
262;
