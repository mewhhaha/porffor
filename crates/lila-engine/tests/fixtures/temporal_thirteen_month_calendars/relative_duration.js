function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
for (const [calendar, leapYear] of [['coptic', 1739], ['ethiopic', 2015], ['ethioaa', 7515]]) {
  const plain = Temporal.PlainDate.from({calendar, year: leapYear, monthCode: 'M01', day: 1});
  const time = plain.toPlainDateTime({hour: 12});
  const fixed = Temporal.ZonedDateTime.from('2022-09-11T12:00[+05:30][u-ca=' + calendar + ']');
  const named = Temporal.ZonedDateTime.from('2022-09-11T12:00[America/New_York][u-ca=' + calendar + ']');
  const month = Temporal.Duration.from({months: 1});
  const year = Temporal.Duration.from({years: 1});
  const thirteen = Temporal.Duration.from({months: 13});
  for (const relativeTo of [plain, time, fixed, named, '2022-09-11[u-ca=' + calendar + ']', '2022-09-11T12:00[UTC][u-ca=' + calendar + ']', {calendar, year: leapYear, monthCode: 'M01', day: 1}]) {
    same(month.total({unit: 'days', relativeTo}), 30, 'regular month days');
    same(year.total({unit: 'days', relativeTo}), 366, 'leap-year days');
    same(thirteen.total({unit: 'years', relativeTo}), 1, 'thirteen months per year');
    same(Temporal.Duration.compare(thirteen, year, {relativeTo}), 0, 'year versus thirteen months');
    same(Temporal.Duration.compare({months: 12}, year, {relativeTo}), -1, 'twelve months below year');
    same(Temporal.Duration.from({days: 366}).round({largestUnit: 'years', smallestUnit: 'years', relativeTo}).years, 1, 'relative year round');
    same(Temporal.Duration.from({days: 30}).round({largestUnit: 'months', smallestUnit: 'months', relativeTo}).months, 1, 'relative month round');
  }
  const end = plain.add({years: 1});
  same(Temporal.Duration.from({months: -13}).total({unit: 'years', relativeTo: end}), -1, 'negative relative year');
  same(Temporal.Duration.from({months: -13}).total({unit: 'days', relativeTo: end}), -366, 'negative leap-year days');
  const short = Temporal.PlainDate.from({calendar, year: leapYear, month: 13, day: 1});
  const commonShort = short.add({years: 1});
  same(month.total({unit: 'days', relativeTo: short}), 6, 'leap short month');
  same(month.total({unit: 'days', relativeTo: commonShort}), 5, 'common short month');
  const dst = Temporal.ZonedDateTime.from('2023-10-12T12:00[America/New_York][u-ca=' + calendar + ']');
  same(dst.month, 2, 'DST calendar month');
  same(dst.day, 1, 'DST calendar day');
  same(month.total({unit: 'days', relativeTo: dst}), 30, 'zoned calendar month days');
  same(month.total({unit: 'hours', relativeTo: dst}), 721, 'retained autumn transition');
  same(dst.add({months: 1}).toPlainDate().withCalendar('iso8601').toString(), '2023-11-11', 'zoned month endpoint');
  for (const relativeTo of [plain, time, fixed, named]) {
    for (const name of ['year', 'month', 'monthCode', 'day', 'calendarId', 'timeZoneId']) {
      Object.defineProperty(relativeTo, name, {get() {throw new Error('public relative field read');}});
    }
    same(thirteen.total({unit: 'years', relativeTo}), 1, 'private relative calendar');
    same(Temporal.Duration.compare(thirteen, year, {relativeTo}), 0, 'private relative comparison');
  }
  let log = [];
  same(month.total({get relativeTo() {log.push('relative'); return plain;}, get unit() {log.push('unit'); return 'days';}}), 30, 'observed relative total');
  same(log.join('|'), 'relative|unit', 'relative total order');
  const marker = {};
  log = [];
  try {
    month.round({get relativeTo() {throw marker;}, get roundingIncrement() {log.push('late'); return 1;}});
    throw new Error('missing relative abrupt');
  } catch (caught) {
    if (caught !== marker) throw new Error('relative abrupt identity');
  }
  same(log.length, 0, 'no late relative options');
}
print('thirteen-month-relative-duration:ok');
262;
