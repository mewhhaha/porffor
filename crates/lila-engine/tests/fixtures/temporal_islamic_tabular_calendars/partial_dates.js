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
for (const [calendar, start, evenStart, late, commonEnd, leapEnd, finalMonth, followingYear] of [
  ['islamic-civil', '1972-02-16', '1972-03-17', '1972-01-14', '1972-02-15', '1971-02-26', '1973-01-06', '1973-02-04'],
  ['islamic-tbla', '1972-02-15', '1972-03-16', '1972-01-13', '1972-02-14', '1971-02-25', '1973-01-05', '1973-02-03']
]) {
  for (const [monthCode, day, reference] of [
    ['M01', 1, start], ['M02', 1, evenStart], ['M11', 27, late],
    ['M12', 29, commonEnd], ['M12', 30, leapEnd]
  ]) {
    const md = Temporal.PlainMonthDay.from({calendar, monthCode, day});
    same(md.toString(), reference + '[u-ca=' + calendar + ']', 'latest reference');
    same(md.monthCode, monthCode, 'reference code');
    same(md.day, day, 'reference day');
    same(Temporal.PlainMonthDay.from(md.toString()).equals(md), true, 'parsed reference');
    const date = md.toPlainDate({year: 1390});
    same(date.year, 1390, 'supplied calendar year');
    same(date.monthCode, monthCode, 'supplied code');
    same(date.day, day, 'supplied day');
    same(date.toPlainMonthDay().equals(md), true, 'date reference join');
    same(date.toPlainDateTime({hour: 12}).toPlainDate().toPlainMonthDay().equals(md), true, 'datetime reference join');
  }
  const lastAtCutoff = calendar === 'islamic-civil' ? 25 : 26;
  same(Temporal.PlainMonthDay.from({calendar, monthCode: 'M11', day: lastAtCutoff}).toString(), '1972-12-31[u-ca=' + calendar + ']', 'exact cutoff');
  same(Temporal.PlainMonthDay.from({calendar, monthCode: 'M12', day: 31}).day, 30, 'missing year uses leap reference');
  range(() => Temporal.PlainMonthDay.from({calendar, monthCode: 'M12', day: 31}, {overflow: 'reject'}));
  const leap = Temporal.PlainMonthDay.from({calendar, monthCode: 'M12', day: 30});
  same(Temporal.PlainMonthDay.from({calendar, year: 1391, monthCode: 'M12', day: 30}).day, 29, 'supplied common regulation');
  same(leap.with({year: 1391}).toString(), commonEnd + '[u-ca=' + calendar + ']', 'with year before reference');
  same(leap.toPlainDate({era: 'ah', eraYear: 1391}).day, 29, 'era supplied common year');
  const ym = Temporal.PlainYearMonth.from({calendar, year: 1392, monthCode: 'M12'});
  same(ym.toString(), finalMonth + '[u-ca=' + calendar + ']', 'calendar day-one reference');
  same(ym.with({monthCode: 'M01'}).toString(), start + '[u-ca=' + calendar + ']', 'partial month reference');
  same(ym.add({months: 1}).toString(), followingYear + '[u-ca=' + calendar + ']', 'positive partial anchor');
  same(Temporal.PlainYearMonth.from(ym.toPlainDate({day: 29}).toString()).equals(ym), true, 'parsed YearMonth normalization');
  const explicitYearMonth = new Temporal.PlainYearMonth(1971, 2, calendar, calendar === 'islamic-civil' ? 26 : 25);
  same(explicitYearMonth.year, 1390, 'explicit reference year');
  same(Temporal.PlainYearMonth.from(explicitYearMonth).equals(explicitYearMonth), true, 'explicit YearMonth clone');
  const explicitMonthDay = new Temporal.PlainMonthDay(2, calendar === 'islamic-civil' ? 26 : 25, calendar, 1971);
  same(explicitMonthDay.day, 30, 'explicit leap reference');
  same(Temporal.PlainMonthDay.from(explicitMonthDay).equals(explicitMonthDay), true, 'explicit MonthDay clone');
  const zoned = Temporal.ZonedDateTime.from(leapEnd + 'T12:00[UTC][u-ca=' + calendar + ']');
  same(zoned.toPlainDate().toPlainMonthDay().equals(leap), true, 'zoned reference join');
}
same(Temporal.PlainMonthDay.from({calendar: 'islamicc', monthCode: 'M12', day: 30}).toString(), '1971-02-26[u-ca=islamic-civil]', 'alias reference');
print('islamic-tabular-partial-dates:ok');
262;
