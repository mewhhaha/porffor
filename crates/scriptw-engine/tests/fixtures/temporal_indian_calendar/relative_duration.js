function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const month = Temporal.Duration.from({months: 1});
const days = Temporal.Duration.from({days: 31});
const plain = Temporal.PlainDate.from({calendar: 'indian', year: 1943, month: 2, day: 1});
const time = plain.toPlainDateTime({hour: 23});
const fixed = Temporal.ZonedDateTime.from('2021-04-21T12:00[+05:30][u-ca=indian]');
const named = Temporal.ZonedDateTime.from('2021-04-21T12:00[America/New_York][u-ca=indian]');
for (const relativeTo of [plain, time, fixed, named, '2021-04-21[u-ca=indian]', '2021-04-21T12:00[UTC][u-ca=indian]', {calendar: 'indian', year: 1943, monthCode: 'M02', day: 1}]) {
  same(month.total({unit: 'days', relativeTo}), 31, 'relative calendar month length');
  same(month.total({unit: 'hours', relativeTo}), 744, 'relative actual hours');
  same(Temporal.Duration.compare(month, days, {relativeTo}), 0, 'relative equal month');
  same(Temporal.Duration.compare(month, {days: 30}, {relativeTo}), 1, 'relative longer month');
  same(month.round({largestUnit: 'days', smallestUnit: 'days', relativeTo}).days, 31, 'relative month balancing');
  same(days.round({largestUnit: 'months', smallestUnit: 'months', roundingMode: 'trunc', relativeTo}).months, 1, 'relative exact month rounding');
  same(Temporal.Duration.from({days: 30}).round({largestUnit: 'months', smallestUnit: 'months', roundingMode: 'ceil', relativeTo}).months, 1, 'relative fractional month rounding');
}
const leapStart = Temporal.PlainDate.from({calendar: 'indian', year: 1942, month: 1, day: 1});
same(Temporal.Duration.from({years: 1}).total({unit: 'days', relativeTo: leapStart}), 366, 'relative calendar leap year');
same(Temporal.Duration.from({months: -1}).total({unit: 'days', relativeTo: plain.add({months: 1})}), -31, 'negative relative month');
// Branded conversion consumes private date/calendar/zone slots, not public getters.
for (const relativeTo of [plain, time, fixed, named]) {
  for (const name of ['year', 'month', 'monthCode', 'day', 'calendarId', 'timeZoneId']) {
    Object.defineProperty(relativeTo, name, {get() {throw new Error('public relative field read');}});
  }
  same(month.total({unit: 'days', relativeTo}), 31, 'retained private calendar');
  same(Temporal.Duration.compare(month, days, {relativeTo}), 0, 'retained comparison calendar');
}
let log = [];
same(month.total({get relativeTo() {log.push('relative'); return plain;}, get unit() {log.push('unit'); return 'days';}}), 31, 'observed relative total');
same(log.join('|'), 'relative|unit', 'relative total reads');
const marker = {};
log = [];
try {
  month.round({get relativeTo() {throw marker;}, get roundingIncrement() {log.push('late'); return 1;}});
  throw new Error('missing relative abrupt');
} catch (error) {
  if (error !== marker) throw new Error('relative abrupt identity');
}
same(log.length, 0, 'relative abrupt stops options');
print('indian-relative-duration:ok');
262;
