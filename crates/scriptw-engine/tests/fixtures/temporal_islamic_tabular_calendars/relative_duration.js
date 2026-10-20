function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
for (const [calendar, leapStart, autumnStart, autumnEnd] of [
  ['islamic-civil', '1970-03-09', '2023-10-16', '2023-11-14'],
  ['islamic-tbla', '1970-03-08', '2023-10-15', '2023-11-13']
]) {
  const plain = Temporal.PlainDate.from({calendar, year: 1390, monthCode: 'M01', day: 1});
  same(plain.withCalendar('iso8601').toString(), leapStart, 'relative anchor');
  const time = plain.toPlainDateTime({hour: 12});
  const fixed = Temporal.ZonedDateTime.from(leapStart + 'T12:00[+05:30][u-ca=' + calendar + ']');
  const named = Temporal.ZonedDateTime.from(leapStart + 'T12:00[America/New_York][u-ca=' + calendar + ']');
  const month = Temporal.Duration.from({months: 1});
  const year = Temporal.Duration.from({years: 1});
  const twelve = Temporal.Duration.from({months: 12});
  for (const relativeTo of [plain, time, fixed, named, leapStart + '[u-ca=' + calendar + ']', leapStart + 'T12:00[UTC][u-ca=' + calendar + ']', {calendar, year: 1390, monthCode: 'M01', day: 1}]) {
    same(month.total({unit: 'days', relativeTo}), 30, 'odd month days');
    same(year.total({unit: 'days', relativeTo}), 355, 'lunar leap year days');
    same(twelve.total({unit: 'years', relativeTo}), 1, 'twelve months per year');
    same(Temporal.Duration.compare(twelve, year, {relativeTo}), 0, 'year versus twelve months');
    same(Temporal.Duration.compare({months: 11}, year, {relativeTo}), -1, 'eleven months below year');
    same(Temporal.Duration.from({days: 355}).round({largestUnit: 'years', smallestUnit: 'years', relativeTo}).years, 1, 'relative lunar year round');
    same(Temporal.Duration.from({days: 30}).round({largestUnit: 'months', smallestUnit: 'months', relativeTo}).months, 1, 'relative month round');
  }
  const common = plain.add({years: 1});
  same(year.total({unit: 'days', relativeTo: common}), 354, 'lunar common year days');
  same(Temporal.Duration.from({months: -12}).total({unit: 'years', relativeTo: common}), -1, 'negative relative year');
  same(Temporal.Duration.from({months: -12}).total({unit: 'days', relativeTo: common}), -355, 'negative lunar leap days');
  same(month.total({unit: 'days', relativeTo: plain.add({months: 1})}), 29, 'even month days');
  const leapLast = Temporal.PlainDate.from({calendar, year: 1390, month: 12, day: 1});
  same(month.total({unit: 'days', relativeTo: leapLast}), 30, 'leap last month');
  same(month.total({unit: 'days', relativeTo: leapLast.add({years: 1})}), 29, 'common last month');
  const dst = Temporal.ZonedDateTime.from(autumnStart + 'T12:00[America/New_York][u-ca=' + calendar + ']');
  same(dst.year, 1445, 'autumn calendar year');
  same(dst.month, 4, 'autumn calendar month');
  same(dst.day, 1, 'autumn calendar day');
  same(month.total({unit: 'days', relativeTo: dst}), 29, 'zoned even month');
  same(month.total({unit: 'hours', relativeTo: dst}), 697, 'retained autumn transition');
  same(dst.add({months: 1}).toPlainDate().withCalendar('iso8601').toString(), autumnEnd, 'zoned lunar endpoint');
  for (const relativeTo of [plain, time, fixed, named]) {
    for (const name of ['year', 'month', 'monthCode', 'day', 'calendarId', 'timeZoneId']) {
      Object.defineProperty(relativeTo, name, {get() {throw new Error('public relative field read');}});
    }
    same(twelve.total({unit: 'years', relativeTo}), 1, 'private relative calendar');
    same(Temporal.Duration.compare(twelve, year, {relativeTo}), 0, 'private relative comparison');
  }
  let log = [];
  same(month.total({get relativeTo() {log.push('relative'); return plain;}, get unit() {log.push('unit'); return 'days';}}), 30, 'observed relative total');
  same(log.join('|'), 'relative|unit', 'relative total order');
  const marker = {};
  log = [];
  try {
    month.round({get relativeTo() {throw marker;}, get roundingIncrement() {log.push('late'); return 1;}});
    throw new Error('missing relative abrupt');
  } catch (caught) {if (caught !== marker) throw new Error('relative abrupt identity');}
  same(log.length, 0, 'no late relative options');
}
print('islamic-tabular-relative-duration:ok');
262;
