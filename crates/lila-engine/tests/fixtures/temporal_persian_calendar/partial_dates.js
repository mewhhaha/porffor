function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const references = [
  ['M01', 31, '1972-04-20[u-ca=persian]'],
  ['M10', 10, '1972-12-31[u-ca=persian]'],
  ['M10', 11, '1972-01-01[u-ca=persian]'],
  ['M12', 29, '1972-03-19[u-ca=persian]'],
  ['M12', 30, '1972-03-20[u-ca=persian]']
];
for (const row of references) {
  const md = Temporal.PlainMonthDay.from({calendar: 'persian', monthCode: row[0], day: row[1]});
  same(md.toString(), row[2], 'latest reference');
  same(md.monthCode, row[0], 'reference month code');
  same(md.day, row[1], 'reference day');
  same(Temporal.PlainMonthDay.from(row[2]).equals(md), true, 'parsed reference');
  const date = md.toPlainDate({year: 1403});
  same(date.year, 1403, 'supplied calendar year');
  same(date.monthCode, row[0], 'supplied month');
  same(date.day, row[1], 'supplied day');
  same(date.toPlainMonthDay().equals(md), true, 'date reference');
  same(date.toPlainDateTime({hour: 12}).toPlainDate().toPlainMonthDay().equals(md), true, 'datetime reference');
}
const leap = Temporal.PlainMonthDay.from({calendar: 'persian', monthCode: 'M12', day: 30});
same(Temporal.PlainMonthDay.from({calendar: 'persian', year: 1404, monthCode: 'M12', day: 30}).day, 29, 'year regulation before reference');
same(leap.with({year: 1404}).day, 29, 'with supplied year');
same(leap.toPlainDate({era: 'ap', eraYear: 1404}).day, 29, 'era supplied year');
same(Temporal.PlainMonthDay.from('2025-03-20[u-ca=persian]').equals(leap), true, 'parsed full-date reference');
const ym = Temporal.PlainYearMonth.from({calendar: 'persian', year: 1403, monthCode: 'M12'});
same(ym.toString(), '2025-02-19[u-ca=persian]', 'calendar day-one reference');
same(ym.toPlainDate({day: 30}).withCalendar('iso8601').toString(), '2025-03-20', 'partial calendar day');
same(Temporal.PlainYearMonth.from('2025-03-20[u-ca=persian]').equals(ym), true, 'parsed year-month normalization');
same(ym.with({year: 1404}).daysInMonth, 29, 'partial year regulation');
same(ym.with({monthCode: 'M01'}).toString(), '2024-03-20[u-ca=persian]', 'partial month reference');
same(ym.subtract({months: 1}).toString(), '2025-01-20[u-ca=persian]', 'negative partial day-one anchor');
same(ym.add({months: 1}).toString(), '2025-03-21[u-ca=persian]', 'positive partial day-one anchor');
const signed = Temporal.PlainYearMonth.from({calendar: 'persian', era: 'ap', eraYear: 0, monthCode: 'M01'});
same(signed.toString(), '0621-03-21[u-ca=persian]', 'signed partial reference');
same(signed.year, 0, 'signed partial projection');
const zoned = Temporal.ZonedDateTime.from('2025-03-20T12:00[UTC][u-ca=persian]');
same(zoned.toPlainDate().toPlainMonthDay().equals(leap), true, 'zoned date reference');
same(zoned.toPlainDateTime().toPlainDate().toPlainYearMonth().equals(ym), true, 'zoned partial reference');
print('persian-partial-dates:ok');
262;
