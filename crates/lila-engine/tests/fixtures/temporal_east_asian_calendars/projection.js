function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
for (const calendar of ['chinese', 'dangi']) {
  const make = (year, monthCode, day = 1) => Temporal.PlainDate.from({calendar, year, monthCode, day}, {overflow: 'reject'});
  const millennium = Temporal.PlainDate.from('2000-01-01[u-ca=' + calendar + ']');
  same(millennium.year, 1999, 'related Gregorian year');
  same(millennium.month, 11, 'pinned millennium ordinal');
  same(millennium.monthCode, 'M11', 'pinned millennium code');
  same(millennium.day, 25, 'pinned millennium day');
  same(make(1999, 'M11', 25).withCalendar('iso8601').toString(), '2000-01-01', 'literal field conversion');
  for (const [year, days, count] of [[1969,354,12], [1970,355,12], [1993,383,13], [2001,384,13], [2006,385,13], [1987,calendar === 'chinese' ? 384 : 385,13]]) {
    const date = make(year, 'M01');
    const next = make(year + 1, 'M01');
    const carriers = [date, date.toPlainDateTime({hour: 12}), date.toPlainYearMonth(), Temporal.ZonedDateTime.from({calendar, year, monthCode: 'M01', day: 1, timeZone: 'UTC'})];
    for (const value of carriers) {
      same(value.calendarId, calendar, 'carrier keeps calendar');
      same(value.year, year, 'carrier related year');
      same(value.monthsInYear, count, 'actual variable count');
      same(value.daysInYear, days, 'actual lunar year length');
      same(value.inLeapYear, count === 13, 'actual inserted month');
      same(value.era, undefined, 'calendar has no era');
      same(value.eraYear, undefined, 'calendar has no era year');
    }
    same(date.until(next).days, days, 'year projection and full-date interval');
    same(date.add({days}).equals(next), true, 'year endpoint');
    same(date.add({months: count}).equals(next), true, 'year month serial');
    let month = date;
    let prefix = 1;
    for (let ordinal = 1; ordinal <= count; ordinal++) {
      same(month.month, ordinal, 'month ordinal progression');
      same(month.dayOfYear, prefix, 'native day prefix');
      same(month.daysInMonth === 29 || month.daysInMonth === 30, true, 'lunar month length');
      const restored = make(year, month.monthCode);
      same(restored.equals(month), true, 'canonical code round trip');
      prefix += month.daysInMonth;
      month = month.add({months: 1});
    }
    same(prefix, days + 1, 'whole native year');
    same(month.equals(next), true, 'last month ends at next year');
    same(date.weekOfYear, undefined, 'no ISO week');
  }
  // Pinned vendor public-Date examples distinguish both observations.
  for (const [iso, chinese, dangi] of [['2012-04-23','M04','M03L'], ['2012-05-23','M04L','M04']]) {
    const date = Temporal.PlainDate.from(iso + '[u-ca=' + calendar + ']');
    same(date.year, 2012, 'modern differing related year');
    same(date.monthCode, calendar === 'chinese' ? chinese : dangi, 'modern differing leap placement');
    same(make(date.year, date.monthCode, date.day).withCalendar('iso8601').toString(), iso, 'differing literal round trip');
  }
  // RD literals are the pinned two exact model joins; 719163 is ISO1970Jan1 RD.
  for (const [rd, year] of [[-1334565,-3653], [1717776,4704]]) {
    const iso = Temporal.PlainDate.from('1970-01-01').add({days: rd - 719163});
    const first = iso.withCalendar(calendar);
    same(first.year, year, 'model join year');
    same(first.monthCode, 'M01', 'model join first code');
    same(first.month, 1, 'model join first ordinal');
    same(first.day, 1, 'model join first day');
    same(make(year, 'M01').equals(first), true, 'native join conversion');
    const before = first.subtract({days: 1});
    same(before.year, year - 1, 'model join predecessor year');
    same(before.month, before.monthsInYear, 'model join predecessor last ordinal');
    same(before.day, before.daysInMonth, 'model join predecessor last day');
    same(before.add({days: 1}).equals(first), true, 'model day continuity');
  }
  for (const year of [-10000, -3654, -3653, -1, 0, 1, 4703, 4704, 10000]) {
    const date = make(year, 'M01');
    same(date.year, year, 'signed native year');
    same(date.withCalendar('iso8601').year, year, 'related year owns New Year');
    same(Temporal.PlainDate.from(date.toString()).equals(date), true, 'signed full-date round trip');
    const next = make(year + 1, 'M01');
    same(date.add({months: date.monthsInYear}).equals(next), true, 'signed/table/model serial boundary');
    same(next.subtract({months: date.monthsInYear}).equals(date), true, 'signed inverse serial boundary');
  }
}
print('east-asian-projection:ok');
262;
