function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function range(action) {
  try {action();} catch (caught) {if (caught instanceof RangeError) return; throw caught;}
  throw new Error('missing partial range error');
}
const calendar = 'islamic-umalqura';
// Latest day-30 matches from the pinned rows1389..1392; ISO years also pinned by Test262.
const references = [
  ['M01',1392,'1972-03-16'], ['M02',1390,'1970-05-06'],
  ['M03',1391,'1971-05-25'], ['M04',1392,'1972-06-12'],
  ['M05',1391,'1971-07-23'], ['M06',1392,'1972-08-10'],
  ['M07',1389,'1969-10-12'], ['M08',1392,'1972-10-08'],
  ['M09',1392,'1972-11-07'], ['M10',1390,'1970-12-29'],
  ['M11',1391,'1972-01-17'], ['M12',1390,'1971-02-26']
];
for (const [monthCode, year, iso] of references) {
  const expected = iso + '[u-ca=' + calendar + ']';
  const md = Temporal.PlainMonthDay.from({calendar, monthCode, day: 30});
  same(md.monthCode, monthCode, 'month-specific day30 code');
  same(md.day, 30, 'month-specific day30 preserved');
  same(md.toString(), expected, 'latest full reference');
  same(Temporal.PlainMonthDay.from({calendar, monthCode, day: 31}).toString(), expected, 'absent-year constrain31 uses long month');
  range(() => Temporal.PlainMonthDay.from({calendar, monthCode, day: 31}, {overflow: 'reject'}));
  same(Temporal.PlainMonthDay.from({calendar, year, monthCode, day: 30}, {overflow: 'reject'}).equals(md), true, 'supplied real long year');
  const date = Temporal.PlainDate.from({calendar, year, monthCode, day: 30});
  same(date.toPlainMonthDay().equals(md), true, 'Date partial factory');
  same(date.toPlainDateTime({hour: 12}).toPlainDate().toPlainMonthDay().equals(md), true, 'DateTime partial route');
  same(Temporal.PlainMonthDay.from(md.toString()).equals(md), true, 'annotated MonthDay parse');
}
same(Temporal.PlainMonthDay.from({calendar, monthCode: 'M01', day: 1}).toString(), '1972-02-16[u-ca=islamic-umalqura]', 'day1 reference');
same(Temporal.PlainMonthDay.from({calendar, monthCode: 'M11', day: 25}).toString(), '1972-12-31[u-ca=islamic-umalqura]', 'inclusive cutoff');
same(Temporal.PlainMonthDay.from({calendar, monthCode: 'M11', day: 26}).toString(), '1972-01-13[u-ca=islamic-umalqura]', 'after cutoff chooses prior year');
same(Temporal.PlainMonthDay.from({calendar, year: 1392, monthCode: 'M03', day: 30}).day, 29, 'supplied short month constrains before reference');
range(() => Temporal.PlainMonthDay.from({calendar, year: 1392, monthCode: 'M03', day: 30}, {overflow: 'reject'}));
const month2 = Temporal.PlainMonthDay.from({calendar, monthCode: 'M02', day: 30});
same(month2.toPlainDate({year: 1392}).day, 29, 'completion uses real supplied year');
same(month2.with({day: 31}).day, 30, 'with missing year uses month-specific regulation');
const ym = Temporal.PlainYearMonth.from({calendar, year: 1392, monthCode: 'M12'});
same(ym.toString(), '1973-01-06[u-ca=islamic-umalqura]', 'calendar day-one YearMonth reference');
same(ym.add({months: 1}).toString(), '1973-02-05[u-ca=islamic-umalqura]', 'YearMonth next table year');
same(Temporal.PlainYearMonth.from('1973-02-04[u-ca=islamic-umalqura]').toString(), '1973-01-06[u-ca=islamic-umalqura]', 'parsed YearMonth canonical reference');
const explicitYM = new Temporal.PlainYearMonth(1973, 2, calendar, 4);
same(explicitYM.year, 1392, 'constructor ISO coordinates');
same(explicitYM.month, 12, 'constructor reference projects month');
same(Temporal.PlainYearMonth.from(explicitYM).equals(explicitYM), true, 'clone retains explicit reference');
const explicitMD = new Temporal.PlainMonthDay(2, 26, calendar, 1971);
same(explicitMD.monthCode, 'M12', 'explicit MonthDay ISO reference');
same(explicitMD.day, 30, 'explicit MonthDay day');
same(Temporal.PlainMonthDay.from(explicitMD).equals(explicitMD), true, 'MonthDay clone reference');
const zoned = Temporal.ZonedDateTime.from('1970-05-06T12:00[UTC][u-ca=islamic-umalqura]');
same(zoned.toPlainDate().toPlainMonthDay().equals(month2), true, 'zoned-to-partial calendar retention');
print('umalqura-partial-dates:ok');
262;
