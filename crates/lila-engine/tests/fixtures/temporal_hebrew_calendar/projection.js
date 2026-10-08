function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const calendar = 'hebrew';
const make = (year, monthCode, day = 1) => Temporal.PlainDate.from({calendar, year, monthCode, day}, {overflow: 'reject'});
function fields(date, year, month, code, day) {
  same(date.calendarId, calendar, 'canonical calendar');
  same(date.year, year, 'native year');
  same(date.month, month, 'native ordinal');
  same(date.monthCode, code, 'canonical code');
  same(date.day, day, 'native day');
  same(date.era, 'am', 'signed AM');
  same(date.eraYear, year, 'signed era year');
}
// Literal pinned Test262 round trips and all six daysInYear cohorts.
fields(Temporal.PlainDate.from('2000-01-01[u-ca=hebrew]'), 5760, 4, 'M04', 23);
fields(Temporal.PlainDate.from('0001-01-01[u-ca=hebrew]'), 3761, 4, 'M04', 18);
same(make(5760, 'M04', 23).withCalendar('iso8601').toString(), '2000-01-01', 'forward literal');
same(make(3761, 'M04', 18).withCalendar('iso8601').toString(), '0001-01-01', 'ancient forward literal');
const rows = [
  [5730, 383, [30,29,29,29,30,30,29,30,29,30,29,30,29]],
  [5731, 354, [30,29,30,29,30,29,30,29,30,29,30,29]],
  [5732, 355, [30,30,30,29,30,29,30,29,30,29,30,29]],
  [5736, 385, [30,30,30,29,30,30,29,30,29,30,29,30,29]],
  [5737, 353, [30,29,29,29,30,29,30,29,30,29,30,29]],
  [5738, 384, [30,29,30,29,30,30,29,30,29,30,29,30,29]]
];
for (const [year, yearDays, lengths] of rows) {
  const leap = lengths.length === 13;
  let prefix = 1;
  for (let month = 1; month <= lengths.length; month++) {
    const code = leap && month === 6 ? 'M05L' : 'M' + String(month - (leap && month > 6 ? 1 : 0)).padStart(2, '0');
    const date = make(year, code);
    const time = date.toPlainDateTime({hour: 12});
    const ym = date.toPlainYearMonth();
    const zoned = Temporal.ZonedDateTime.from({calendar, year, monthCode: code, day: 1, timeZone: 'UTC'});
    for (const carrier of [date, time, ym, zoned]) {
      same(carrier.year, year, 'carrier native year');
      same(carrier.month, month, 'carrier ordinal');
      same(carrier.monthCode, code, 'carrier canonical code');
      same(carrier.daysInMonth, lengths[month - 1], 'actual variable month');
      same(carrier.daysInYear, yearDays, 'actual six-length year');
      same(carrier.monthsInYear, lengths.length, 'actual year count');
      same(carrier.inLeapYear, leap, 'actual Metonic leap');
    }
    same(date.dayOfYear, prefix, 'native month prefix');
    same(date.weekOfYear, undefined, 'no ISO week field');
    prefix += lengths[month - 1];
  }
  same(prefix, yearDays + 1, 'complete native year');
  fields(make(year, 'M01').add({days: yearDays}), year + 1, 1, 'M01', 1);
}
for (const year of [-20, -1, 0, 1, 19, 20]) {
  const date = make(year, 'M01');
  fields(date, year, 1, 'M01', 1);
  fields(Temporal.PlainDate.from(date.toString()), year, 1, 'M01', 1);
  fields(Temporal.PlainDate.from({calendar, era: 'am', eraYear: year, monthCode: 'M01', day: 1}), year, 1, 'M01', 1);
  same(date.add({months: date.monthsInYear}).equals(make(year + 1, 'M01')), true, 'signed serial year boundary');
  same(make(year + 1, 'M01').subtract({months: date.monthsInYear}).equals(date), true, 'signed inverse boundary');
}
print('hebrew-projection:ok');
262;
