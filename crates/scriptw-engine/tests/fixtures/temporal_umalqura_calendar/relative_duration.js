function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const calendar = 'islamic-umalqura';
const plain = Temporal.PlainDate.from({calendar, year: 1390, monthCode: 'M01', day: 1});
same(plain.withCalendar('iso8601').toString(), '1970-03-09', 'pinned relative anchor');
const time = plain.toPlainDateTime({hour: 12});
const fixed = Temporal.ZonedDateTime.from('1970-03-09T12:00[+05:30][u-ca=islamic-umalqura]');
const named = Temporal.ZonedDateTime.from('1970-03-09T12:00[America/New_York][u-ca=islamic-umalqura]');
const month = Temporal.Duration.from({months: 1});
const year = Temporal.Duration.from({years: 1});
const twelve = Temporal.Duration.from({months: 12});
for (const relativeTo of [plain, time, fixed, named, '1970-03-09[u-ca=islamic-umalqura]', '1970-03-09T12:00[UTC][u-ca=islamic-umalqura]', {calendar, year: 1390, monthCode: 'M01', day: 1}]) {
  same(month.total({unit: 'days', relativeTo}), 29, 'table odd month short');
  same(year.total({unit: 'days', relativeTo}), 355, 'table long year');
  same(twelve.total({unit: 'years', relativeTo}), 1, 'count12 total');
  same(Temporal.Duration.compare(twelve, year, {relativeTo}), 0, 'year versus twelve');
  same(Temporal.Duration.compare({months: 11}, year, {relativeTo}), -1, 'eleven months below year');
  same(Temporal.Duration.from({days: 355}).round({largestUnit: 'years', smallestUnit: 'years', relativeTo}).years, 1, 'actual year rounding');
  same(Temporal.Duration.from({days: 29}).round({largestUnit: 'months', smallestUnit: 'months', relativeTo}).months, 1, 'actual month rounding');
}
const common = plain.add({years: 1});
same(year.total({unit: 'days', relativeTo: common}), 354, 'table common year');
same(year.total({unit: 'days', relativeTo: plain.add({years: 2})}), 355, 'table 1392 distinct from civil');
same(Temporal.Duration.from({months: -12}).total({unit: 'years', relativeTo: common}), -1, 'negative relative year');
same(Temporal.Duration.from({months: -12}).total({unit: 'days', relativeTo: common}), -355, 'negative actual year days');
same(month.total({unit: 'days', relativeTo: plain.add({months: 1})}), 30, 'table even month long');
const last = Temporal.PlainDate.from({calendar, year: 1390, month: 12, day: 1});
same(month.total({unit: 'days', relativeTo: last}), 30, 'table last month long');
same(month.total({unit: 'days', relativeTo: last.add({years: 1})}), 29, 'table following last month short');
// The table makes M04 a30-day month across the retained NY autumn transition.
const dst = Temporal.ZonedDateTime.from('2023-10-16T12:00[America/New_York][u-ca=islamic-umalqura]');
same(dst.year, 1445, 'autumn year');
same(dst.month, 4, 'autumn month');
same(dst.day, 1, 'autumn day');
same(month.total({unit: 'days', relativeTo: dst}), 30, 'zoned table month');
same(month.total({unit: 'hours', relativeTo: dst}), 721, 'retained one-hour autumn transition');
same(dst.add({months: 1}).toPlainDate().withCalendar('iso8601').toString(), '2023-11-15', 'zoned table endpoint');
for (const relativeTo of [plain, time, fixed, named]) {
  for (const name of ['year', 'month', 'monthCode', 'day', 'calendarId', 'timeZoneId']) {
    Object.defineProperty(relativeTo, name, {get() {throw new Error('public relative field read');}});
  }
  same(twelve.total({unit: 'years', relativeTo}), 1, 'private relative calendar');
  same(Temporal.Duration.compare(twelve, year, {relativeTo}), 0, 'private relative comparison');
}
let log = [];
same(month.total({get relativeTo() {log.push('relative'); return plain;}, get unit() {log.push('unit'); return 'days';}}), 29, 'observed relative total');
same(log.join('|'), 'relative|unit', 'relative option order');
const marker = {};
try {
  month.round({get relativeTo() {throw marker;}, get roundingIncrement() {log.push('late'); return 1;}});
  throw new Error('missing relative throw');
} catch (caught) {if (caught !== marker) throw new Error('relative throw identity');}
same(log.join('|'), 'relative|unit', 'no later options after throw');
print('umalqura-relative-duration:ok');
262;
