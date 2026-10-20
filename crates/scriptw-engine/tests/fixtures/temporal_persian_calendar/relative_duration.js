function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const month = Temporal.Duration.from({months: 1});
const days = Temporal.Duration.from({days: 31});
const plain = Temporal.PlainDate.from({calendar: 'persian', year: 1400, month: 2, day: 1});
const time = plain.toPlainDateTime({hour: 23});
const fixed = Temporal.ZonedDateTime.from('2021-04-21T12:00[+05:30][u-ca=persian]');
const named = Temporal.ZonedDateTime.from('2021-04-21T12:00[America/New_York][u-ca=persian]');
for (const relativeTo of [plain, time, fixed, named, '2021-04-21[u-ca=persian]', '2021-04-21T12:00[UTC][u-ca=persian]', {calendar: 'persian', year: 1400, monthCode: 'M02', day: 1}]) {
  same(month.total({unit: 'days', relativeTo}), 31, 'calendar month days');
  same(month.total({unit: 'hours', relativeTo}), 744, 'actual month hours');
  same(Temporal.Duration.compare(month, days, {relativeTo}), 0, 'equal calendar month');
  same(Temporal.Duration.compare(month, {days: 30}, {relativeTo}), 1, 'longer calendar month');
  same(month.round({largestUnit: 'days', smallestUnit: 'days', relativeTo}).days, 31, 'calendar balancing');
  same(days.round({largestUnit: 'months', smallestUnit: 'months', roundingMode: 'trunc', relativeTo}).months, 1, 'exact month rounding');
  same(Temporal.Duration.from({days: 30}).round({largestUnit: 'months', smallestUnit: 'months', roundingMode: 'ceil', relativeTo}).months, 1, 'fractional month rounding');
}
const leapStart = Temporal.PlainDate.from({calendar: 'persian', year: 1403, month: 1, day: 1});
same(Temporal.Duration.from({years: 1}).total({unit: 'days', relativeTo: leapStart}), 366, 'calendar leap year');
same(Temporal.Duration.from({months: -1}).total({unit: 'days', relativeTo: plain.add({months: 1})}), -31, 'negative month');
const dst = Temporal.ZonedDateTime.from('2025-02-19T12:00[America/New_York][u-ca=persian]');
same(month.total({unit: 'days', relativeTo: dst}), 30, 'leap Esfand calendar days');
same(month.total({unit: 'hours', relativeTo: dst}), 719, 'retained zone spring transition');
same(dst.add({months: 1}).toPlainDate().withCalendar('iso8601').toString(), '2025-03-21', 'zoned Nowruz endpoint');
for (const relativeTo of [plain, time, fixed, named]) {
  for (const name of ['year', 'month', 'monthCode', 'day', 'calendarId', 'timeZoneId']) {
    Object.defineProperty(relativeTo, name, {get() {throw new Error('public relative field read');}});
  }
  same(month.total({unit: 'days', relativeTo}), 31, 'private relative calendar');
  same(Temporal.Duration.compare(month, days, {relativeTo}), 0, 'private comparison calendar');
}
let log = [];
same(month.total({get relativeTo() {log.push('relative'); return plain;}, get unit() {log.push('unit'); return 'days';}}), 31, 'observed total');
same(log.join('|'), 'relative|unit', 'relative total order');
const marker = {};
log = [];
try {
  month.round({get relativeTo() {throw marker;}, get roundingIncrement() {log.push('late'); return 1;}});
  throw new Error('missing relative abrupt');
} catch (error) {
  if (error !== marker) throw new Error('relative abrupt identity');
}
same(log.length, 0, 'no late relative option');
print('persian-relative-duration:ok');
262;
