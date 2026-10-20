function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
const calendar = 'islamic-umalqura';
const make = (year, month, day) => Temporal.PlainDate.from({calendar, year, month, day}, {overflow: 'reject'});
function dateFields(date, year, month, day) {
  same(date.calendarId, calendar, 'canonical calendar');
  same(date.year, year, 'projected year');
  same(date.month, month, 'projected month');
  same(date.monthCode, 'M' + String(month).padStart(2, '0'), 'projected code');
  same(date.day, day, 'projected day');
  same(date.monthsInYear, 12, 'fixed count');
  same(date.era, year > 0 ? 'ah' : 'bh', 'reported era');
  same(date.eraYear, year > 0 ? year : 1 - year, 'reported era year');
  same(date.weekOfYear, undefined, 'no ISO week projection');
}
// Exact ISO starts and month flags are literal rows from the pinned ICU4X table.
const rows = [
  [1300, '1882-11-12', 354, [30,29,30,29,30,29,30,29,30,29,30,29]],
  [1390, '1970-03-09', 355, [29,30,29,30,30,30,29,30,29,30,29,30]],
  [1391, '1971-02-27', 354, [29,29,30,29,30,30,29,30,30,29,30,29]],
  [1392, '1972-02-16', 355, [30,29,29,30,29,30,29,30,30,29,30,30]],
  [1420, '1999-04-17', 355, [29,30,29,29,30,29,30,30,30,30,29,30]],
  [1600, '2173-12-07', 354, [29,29,30,29,30,29,29,30,30,30,29,30]]
];
for (const [year, iso, days, months] of rows) {
  const start = make(year, 1, 1);
  same(start.withCalendar('iso8601').toString(), iso, 'literal year start');
  dateFields(new Temporal.PlainDate(Number(iso.slice(0, 4)), Number(iso.slice(5, 7)), Number(iso.slice(8, 10)), calendar), year, 1, 1);
  let ordinal = 1;
  for (let month = 1; month <= 12; month++) {
    const date = make(year, month, 1);
    dateFields(date, year, month, 1);
    same(date.dayOfYear, ordinal, 'table prefix ordinal');
    same(date.daysInMonth, months[month - 1], 'nonalternating month length');
    same(date.daysInYear, days, 'actual table year length');
    same(date.inLeapYear, days === 355, 'actual table long year');
    const time = date.toPlainDateTime({hour: 12});
    same(time.daysInMonth, months[month - 1], 'DateTime table projection');
    same(date.toPlainYearMonth().daysInYear, days, 'YearMonth table projection');
    same(Temporal.ZonedDateTime.from({calendar, year, month, day: 1, timeZone: 'UTC'}).daysInMonth, months[month - 1], 'zoned table projection');
    ordinal += months[month - 1];
  }
  same(ordinal, days + 1, 'complete table year');
  dateFields(start.add({days}), year + 1, 1, 1);
}
// Both adjacent civil joins and the required upper table year are explicit.
dateFields(Temporal.PlainDate.from('1882-11-11[u-ca=islamic-umalqura]'), 1299, 12, 29);
dateFields(Temporal.PlainDate.from('1882-11-12[u-ca=islamic-umalqura]'), 1300, 1, 1);
same(make(1299, 12, 29).withCalendar('iso8601').toString(), '1882-11-11', 'lower civil last day');
same(make(1600, 12, 30).withCalendar('iso8601').toString(), '2174-11-25', 'upper table last day');
dateFields(Temporal.PlainDate.from('2174-11-25[u-ca=islamic-umalqura]'), 1600, 12, 30);
dateFields(Temporal.PlainDate.from('2174-11-26[u-ca=islamic-umalqura]'), 1601, 1, 1);
same(make(1601, 1, 1).withCalendar('iso8601').toString(), '2174-11-26', 'upper civil first day');
same(Temporal.PlainDate.from({calendar: 'islamic-civil', year: 1600, month: 1, day: 1}).withCalendar('iso8601').toString(), '2173-12-06', 'table differs from civil inside upper row');
same(make(1392, 1, 1).inLeapYear, true, 'table 1392 long');
same(Temporal.PlainDate.from({calendar: 'islamic-civil', year: 1392, month: 1, day: 1}).inLeapYear, false, 'civil 1392 common');
// Pinned Test262 round-trip and signed-era observations, including the ICU discrepancy.
dateFields(Temporal.PlainDate.from('2000-01-01[u-ca=islamic-umalqura]'), 1420, 9, 24);
dateFields(Temporal.PlainDate.from('2006-07-25[u-ca=islamic-umalqura]'), 1427, 6, 29);
dateFields(Temporal.PlainDate.from('0001-01-01[u-ca=islamic-umalqura]'), -640, 5, 18);
for (const [era, eraYear, year] of [['ah',-1,-1], ['ah',0,0], ['ah',1,1], ['bh',-1,2], ['bh',0,1], ['bh',1,0], ['bh',2,-1]]) {
  const date = Temporal.PlainDate.from({calendar, era, eraYear, monthCode: 'M01', day: 1});
  dateFields(date, year, 1, 1);
  same(Temporal.PlainDate.from(date.toString()).equals(date), true, 'signed annotated round trip');
  same(date.withCalendar('islamic-civil').year, year, 'signed civil fallback');
}
print('umalqura-projection:ok');
262;
