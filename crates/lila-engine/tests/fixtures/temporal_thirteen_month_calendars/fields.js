function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function error(kind, action) {
  try {action();} catch (caught) {
    if (!(caught instanceof kind)) throw new Error('wrong field error');
    return;
  }
  throw new Error('missing field error');
}
for (const [calendar, commonYear, era] of [['coptic', 1740, 'am'], ['ethiopic', 2016, 'am'], ['ethioaa', 7516, 'aa']]) {
  const make = bag => Temporal.PlainDate.from(Object.assign({calendar}, bag));
  const leap = make({year: commonYear - 1, monthCode: 'M13', day: 6});
  same(leap.withCalendar('iso8601').toString(), '2023-09-11', 'leap fields');
  same(make({year: commonYear, month: 13, day: 6}).day, 5, 'common constrain');
  error(RangeError, () => Temporal.PlainDate.from({calendar, year: commonYear, monthCode: 'M13', day: 6}, {overflow: 'reject'}));
  const constrained = make({year: commonYear, month: 99, day: 99});
  same(constrained.month, 13, 'numeric month constrain');
  same(constrained.day, 5, 'short month constrain');
  for (const monthCode of ['M00', 'M14', 'M01L', 'M13L']) {
    error(RangeError, () => make({year: commonYear, monthCode, day: 1}));
    error(RangeError, () => Temporal.PlainYearMonth.from({calendar, year: commonYear, monthCode}));
    error(RangeError, () => Temporal.PlainMonthDay.from({calendar, monthCode, day: 1}));
    error(RangeError, () => Temporal.ZonedDateTime.from({calendar, year: commonYear, monthCode, day: 1, timeZone: 'UTC'}));
  }
  same(Temporal.PlainYearMonth.from({calendar, year: commonYear, month: 13, monthCode: 'M13'}).month, 13, 'YearMonth suitability');
  same(Temporal.PlainMonthDay.from({calendar, year: commonYear, month: 13, monthCode: 'M13', day: 5}).monthCode, 'M13', 'MonthDay suitability');
  same(Temporal.ZonedDateTime.from({calendar, year: commonYear, month: 13, monthCode: 'M13', day: 5, timeZone: 'UTC'}).month, 13, 'zoned suitability');
  error(RangeError, () => make({year: commonYear, month: 12, monthCode: 'M13', day: 1}));
  error(RangeError, () => make({year: commonYear, era, eraYear: commonYear + 1, month: 1, day: 1}));
  error(TypeError, () => make({era, month: 1, day: 1}));
  for (const invalidEra of ['AM', 'AA', 'incar', 'mundi']) {
    error(RangeError, () => make({era: invalidEra, eraYear: commonYear, month: 1, day: 1}));
  }
  for (const receiver of [leap, leap.toPlainDateTime({hour: 12}), leap.toPlainYearMonth()]) {
    same(receiver.with({monthCode: 'M01'}).month, 1, 'with month exclusion');
    same(receiver.with({era, eraYear: commonYear}).year, commonYear, 'with era exclusion');
  }
  same(leap.with({year: commonYear}).day, 5, 'with supplied common year');
  const zoned = Temporal.ZonedDateTime.from({calendar, year: commonYear - 1, month: 13, day: 6, hour: 12, timeZone: 'UTC'});
  same(zoned.with({year: commonYear}).day, 5, 'zoned regulation');
  same(zoned.with({monthCode: 'M01'}).month, 1, 'zoned month exclusion');
  let log = [];
  const bag = {
    get calendar() {log.push('calendar'); return calendar;},
    get day() {log.push('day'); return 1;},
    get era() {log.push('era'); return era;},
    get eraYear() {log.push('eraYear'); return commonYear;},
    get month() {log.push('month'); return 13;},
    get monthCode() {log.push('monthCode'); return 'M13';},
    get year() {log.push('year'); return commonYear + 1;}
  };
  error(RangeError, () => Temporal.PlainDate.from(bag, {get overflow() {log.push('overflow'); return 'reject';}}));
  same(log.join('|'), 'calendar|day|era|eraYear|month|monthCode|year|overflow', 'field and option order');
  log = [];
  // Preserve the accessor until Temporal reads the original bag.
  error(RangeError, () => Temporal.PlainDate.from({calendar, day: 1, monthCode: 'L13M', get year() {log.push('late year'); return commonYear;}}));
  same(log.length, 0, 'month-code syntax before later year');
  error(TypeError, () => make({day: 1, monthCode: 'M13L', year: Symbol('year')}));
  const marker = {};
  log = [];
  let published = 'before';
  let finallyRuns = 0;
  try {
    try {
      const result = Temporal.PlainDate.from({calendar, get day() {throw marker;}, get year() {log.push('late year'); return commonYear;}}, {get overflow() {log.push('late option'); return 'reject';}});
      published = result;
    } finally {finallyRuns++;}
    throw new Error('missing original field throw');
  } catch (caught) {
    if (caught !== marker) throw new Error('field throw identity');
  }
  same(published, 'before', 'no abrupt publication');
  same(finallyRuns, 1, 'field finally');
  same(log.length, 0, 'no late field or option');
}
same(Temporal.PlainDate.from({calendar: 'ethiopic', year: 0, era: 'aa', eraYear: 5500, month: 13, day: 5}).year, 0, 'Ethiopic AA year agreement');
error(RangeError, () => Temporal.PlainDate.from({calendar: 'ethiopic', year: 0, era: 'aa', eraYear: 5499, month: 13, day: 5}));
for (const calendar of ['iso8601', 'gregory', 'indian', 'persian']) {
  error(RangeError, () => Temporal.PlainDate.from({calendar, year: 2000, monthCode: 'M13', day: 1}));
  error(RangeError, () => Temporal.PlainYearMonth.from({calendar, year: 2000, monthCode: 'M13'}));
  error(RangeError, () => Temporal.PlainMonthDay.from({calendar, monthCode: 'M13', day: 1}));
}
print('thirteen-month-fields:ok');
262;
