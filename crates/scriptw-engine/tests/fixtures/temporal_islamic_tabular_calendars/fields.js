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
for (const [calendar, leapEnd] of [['islamic-civil', '1971-02-26'], ['islamic-tbla', '1971-02-25']]) {
  const make = bag => Temporal.PlainDate.from(Object.assign({calendar}, bag));
  const leap = make({year: 1390, monthCode: 'M12', day: 30});
  same(leap.withCalendar('iso8601').toString(), leapEnd, 'leap month12');
  same(make({year: 1391, month: 12, day: 30}).day, 29, 'common constrain');
  error(RangeError, () => Temporal.PlainDate.from({calendar, year: 1391, monthCode: 'M12', day: 30}, {overflow: 'reject'}));
  same(make({year: 1392, month: 2, day: 30}).day, 29, 'even month constrain');
  same(make({year: 1392, month: 99, day: 99}).month, 12, 'numeric month constrain');
  same(make({year: 1392, month: 99, day: 99}).day, 29, 'common last month');
  for (const monthCode of ['M00', 'M13', 'M01L', 'M12L', 'M13L']) {
    error(RangeError, () => make({year: 1392, monthCode, day: 1}));
    error(RangeError, () => Temporal.PlainYearMonth.from({calendar, year: 1392, monthCode}));
    error(RangeError, () => Temporal.PlainMonthDay.from({calendar, monthCode, day: 1}));
    error(RangeError, () => Temporal.ZonedDateTime.from({calendar, year: 1392, monthCode, day: 1, timeZone: 'UTC'}));
  }
  same(Temporal.PlainYearMonth.from({calendar, year: 1392, month: 12, monthCode: 'M12'}).month, 12, 'YearMonth suitability');
  same(Temporal.PlainMonthDay.from({calendar, year: 1390, month: 12, monthCode: 'M12', day: 30}).day, 30, 'MonthDay suitability');
  same(Temporal.ZonedDateTime.from({calendar, year: 1392, month: 12, monthCode: 'M12', day: 29, timeZone: 'UTC'}).month, 12, 'zoned suitability');
  error(RangeError, () => make({year: 1392, month: 11, monthCode: 'M12', day: 1}));
  error(RangeError, () => make({year: 0, era: 'bh', eraYear: 2, month: 1, day: 1}));
  same(make({year: 0, era: 'bh', eraYear: 1, month: 1, day: 1}).year, 0, 'negative era agreement');
  error(TypeError, () => make({era: 'ah', month: 1, day: 1}));
  for (const era of ['AH', 'BH', 'anno-hegirae']) error(RangeError, () => make({era, eraYear: 1392, month: 1, day: 1}));
  for (const receiver of [leap, leap.toPlainDateTime({hour: 12}), leap.toPlainYearMonth()]) {
    same(receiver.with({monthCode: 'M01'}).month, 1, 'with month exclusion');
    same(receiver.with({era: 'bh', eraYear: 1}).year, 0, 'with era exclusion');
  }
  same(leap.with({year: 1391}).day, 29, 'with common year');
  const zoned = Temporal.ZonedDateTime.from({calendar, year: 1390, month: 12, day: 30, timeZone: 'UTC'});
  same(zoned.with({year: 1391}).day, 29, 'zoned regulation');
  same(zoned.with({monthCode: 'M01'}).month, 1, 'zoned month exclusion');
  let log = [];
  const bag = {
    get calendar() {log.push('calendar'); return calendar;},
    get day() {log.push('day'); return 1;},
    get era() {log.push('era'); return 'ah';},
    get eraYear() {log.push('eraYear'); return 1392;},
    get month() {log.push('month'); return 12;},
    get monthCode() {log.push('monthCode'); return 'M12';},
    get year() {log.push('year'); return 1393;}
  };
  error(RangeError, () => Temporal.PlainDate.from(bag, {get overflow() {log.push('overflow'); return 'reject';}}));
  same(log.join('|'), 'calendar|day|era|eraYear|month|monthCode|year|overflow', 'field and option order');
  log = [];
  // Preserve the accessor until Temporal reads the original bag.
  error(RangeError, () => Temporal.PlainDate.from({calendar, day: 1, monthCode: 'L12M', get year() {log.push('late year'); return 1392;}}));
  same(log.length, 0, 'syntax before later field');
  error(TypeError, () => make({day: 1, monthCode: 'M13', year: Symbol('year')}));
  const marker = {};
  let published = 'before';
  let finallyRuns = 0;
  try {
    try {
      const value = Temporal.PlainDate.from({calendar, get day() {throw marker;}, get year() {log.push('late year'); return 1392;}}, {get overflow() {log.push('late option'); return 'reject';}});
      published = value;
    } finally {finallyRuns++;}
    throw new Error('missing original field throw');
  } catch (caught) {if (caught !== marker) throw new Error('field throw identity');}
  same(published, 'before', 'no abrupt publication');
  same(finallyRuns, 1, 'field finally');
  same(log.length, 0, 'no late fields or options');
}
for (const calendar of ['islamic']) {
  error(RangeError, () => Temporal.PlainDate.from({calendar, year: 1392, month: 1, day: 1}));
}
same(Temporal.PlainDate.from({calendar: 'coptic', year: 1739, monthCode: 'M13', day: 6}).day, 6, 'retained thirteen-month domain');
print('islamic-tabular-fields:ok');
262;
