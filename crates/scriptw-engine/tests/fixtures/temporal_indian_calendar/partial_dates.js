function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const references = [
  ['M01', 31, '1972-04-20[u-ca=indian]'],
  ['M10', 10, '1972-12-31[u-ca=indian]'],
  ['M10', 11, '1972-01-01[u-ca=indian]'],
  ['M10', 22, '1972-01-12[u-ca=indian]'],
  ['M12', 30, '1972-03-20[u-ca=indian]']
];
for (const row of references) {
  const md = Temporal.PlainMonthDay.from({calendar: 'indian', monthCode: row[0], day: row[1]});
  same(md.toString(), row[2], 'latest calendar reference');
  same(md.monthCode, row[0], 'reference month code');
  same(md.day, row[1], 'reference day');
  same(Temporal.PlainMonthDay.from(row[2]).equals(md), true, 'parsed reference');
  const date = md.toPlainDate({year: 1942});
  same(date.year, 1942, 'partial supplied calendar year');
  same(date.monthCode, row[0], 'partial supplied month');
  same(date.day, row[1], 'partial supplied day');
  same(date.toPlainMonthDay().equals(md), true, 'date to partial reference');
  same(date.toPlainDateTime({hour: 12}).toPlainDate().toPlainMonthDay().equals(md), true, 'datetime to partial reference');
}
const leap = Temporal.PlainMonthDay.from({calendar: 'indian', monthCode: 'M01', day: 31});
const constrained = Temporal.PlainMonthDay.from({calendar: 'indian', year: 1943, monthCode: 'M01', day: 31});
same(constrained.day, 30, 'supplied year regulation precedes reference');
same(leap.with({year: 1943}).day, 30, 'partial with supplied year');
same(leap.toPlainDate({era: 'shaka', eraYear: 1943}).day, 30, 'partial era conversion');
same(Temporal.PlainMonthDay.from('2020-04-20[u-ca=indian]').equals(leap), true, 'parsed full date before reference');
const ym = Temporal.PlainYearMonth.from({calendar: 'indian', year: 1942, monthCode: 'M01'});
same(ym.toString(), '2020-03-21[u-ca=indian]', 'calendar day-one reference');
same(ym.toPlainDate({day: 31}).withCalendar('iso8601').toString(), '2020-04-20', 'partial calendar day');
same(Temporal.PlainYearMonth.from('2020-04-20[u-ca=indian]').equals(ym), true, 'parsed year-month normalization');
same(ym.with({year: 1943}).toString(), '2021-03-22[u-ca=indian]', 'partial year change day-one');
same(ym.with({monthCode: 'M02'}).toString(), '2020-04-21[u-ca=indian]', 'partial month change day-one');
same(ym.subtract({months: 1}).toString(), '2020-02-20[u-ca=indian]', 'negative partial calendar anchor');
same(ym.add({months: 1}).toString(), '2020-04-21[u-ca=indian]', 'positive partial calendar anchor');
const zoned = Temporal.ZonedDateTime.from('2020-04-20T12:00[UTC][u-ca=indian]');
same(zoned.toPlainDate().toPlainMonthDay().equals(leap), true, 'zoned date partial reference');
same(zoned.toPlainDateTime().toPlainDate().toPlainYearMonth().equals(ym), true, 'zoned datetime partial reference');
print('indian-partial-dates:ok');
262;
