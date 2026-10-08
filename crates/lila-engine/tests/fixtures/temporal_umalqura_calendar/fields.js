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
const calendar = 'islamic-umalqura';
const make = bag => Temporal.PlainDate.from(Object.assign({calendar}, bag));
const long = make({year: 1392, monthCode: 'M12', day: 30});
same(long.withCalendar('iso8601').toString(), '1973-02-04', 'table long last month');
same(make({year: 1392, month: 3, day: 30}).day, 29, 'odd table month constrains');
same(make({year: 1390, month: 2, day: 30}).day, 30, 'even table month accepts');
same(make({year: 1392, month: 99, day: 99}).month, 12, 'month regulation');
same(make({year: 1392, month: 99, day: 99}).day, 30, 'day regulation uses table');
error(RangeError, () => Temporal.PlainDate.from({calendar, year: 1392, monthCode: 'M03', day: 30}, {overflow: 'reject'}));
for (const monthCode of ['M00', 'M13', 'M01L', 'M12L', 'M13L']) {
  error(RangeError, () => make({year: 1392, monthCode, day: 1}));
  error(RangeError, () => Temporal.PlainYearMonth.from({calendar, year: 1392, monthCode}));
  error(RangeError, () => Temporal.PlainMonthDay.from({calendar, monthCode, day: 1}));
  error(RangeError, () => Temporal.ZonedDateTime.from({calendar, year: 1392, monthCode, day: 1, timeZone: 'UTC'}));
}
for (let month = 1; month <= 12; month++) {
  const monthCode = 'M' + String(month).padStart(2, '0');
  same(make({year: 1392, month, monthCode, day: 1}).monthCode, monthCode, 'Date code');
  same(Temporal.PlainYearMonth.from({calendar, year: 1392, month, monthCode}).monthCode, monthCode, 'YearMonth code');
  same(Temporal.PlainMonthDay.from({calendar, monthCode, day: 30}).day, 30, 'MonthDay per-month regulation');
  same(Temporal.PlainMonthDay.from({calendar, year: 1392, month, monthCode, day: 1}).monthCode, monthCode, 'MonthDay supplied-year agreement');
  error(TypeError, () => Temporal.PlainMonthDay.from({calendar, month, monthCode, day: 30}));
  same(Temporal.ZonedDateTime.from({calendar, year: 1392, month, monthCode, day: 1, timeZone: 'UTC'}).monthCode, monthCode, 'zoned code');
}
error(RangeError, () => make({year: 1392, month: 11, monthCode: 'M12', day: 1}));
error(RangeError, () => make({year: 0, era: 'bh', eraYear: 2, month: 1, day: 1}));
same(make({year: 0, era: 'bh', eraYear: 1, month: 1, day: 1}).year, 0, 'era agreement');
error(TypeError, () => make({era: 'ah', month: 1, day: 1}));
for (const era of ['AH', 'BH', 'anno-hegirae']) error(RangeError, () => make({era, eraYear: 1392, month: 1, day: 1}));
for (const receiver of [long, long.toPlainDateTime({hour: 12}), long.toPlainYearMonth()]) {
  same(receiver.with({monthCode: 'M01'}).month, 1, 'with excludes old month');
  same(receiver.with({era: 'bh', eraYear: 1}).year, 0, 'with excludes old year');
}
same(long.with({year: 1391}).day, 29, 'with table short destination');
const zoned = Temporal.ZonedDateTime.from({calendar, year: 1392, month: 12, day: 30, timeZone: 'UTC'});
same(zoned.with({year: 1391}).day, 29, 'zoned table regulation');
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
same(log.join('|'), 'calendar|day|era|eraYear|month|monthCode|year|overflow', 'fields before agreement and options');
log = [];
// Pass the bag directly: Object.assign in make() would invoke year before
// Temporal receives the object and invalidate this field-order observation.
error(RangeError, () => Temporal.PlainDate.from({calendar, day: 1, monthCode: 'L12M', get year() {log.push('late'); return 1392;}}));
same(log.length, 0, 'syntax before later field');
error(TypeError, () => make({day: 1, monthCode: 'M13', year: Symbol('year')}));
const marker = {};
let published = 'before';
let finallyRuns = 0;
try {
  try {
    const value = Temporal.PlainDate.from({calendar, get day() {throw marker;}, get year() {log.push('late'); return 1392;}}, {get overflow() {log.push('late option'); return 'reject';}});
    published = value;
  } finally {finallyRuns++;}
  throw new Error('missing field throw');
} catch (caught) {if (caught !== marker) throw new Error('field throw identity');}
same(published, 'before', 'no abrupt publication');
same(finallyRuns, 1, 'field finally');
same(log.length, 0, 'no late fields or options');
error(RangeError, () => make({calendar: 'islamic', year: 1392, month: 1, day: 1}));
same(Temporal.PlainDate.from({calendar: 'coptic', year: 1739, monthCode: 'M13', day: 6}).day, 6, 'retained thirteen-month domain');
print('umalqura-fields:ok');
262;
