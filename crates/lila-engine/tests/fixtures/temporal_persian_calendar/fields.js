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
const make = fields => Temporal.PlainDate.from(Object.assign({calendar: 'persian'}, fields));
const leap = make({year: 1403, monthCode: 'M12', day: 30});
same(leap.withCalendar('iso8601').toString(), '2025-03-20', 'calendar leap fields');
same(make({year: 1404, month: 12, day: 30}).day, 29, 'common constrain');
error(RangeError, () => Temporal.PlainDate.from({calendar: 'persian', year: 1404, month: 12, day: 30}, {overflow: 'reject'}));
same(make({year: 1404, month: 99, day: 99}).month, 12, 'numeric month constrain');
same(make({year: 1404, month: 99, day: 99}).day, 29, 'numeric day constrain');
for (const code of ['M13', 'M00', 'M01L']) error(RangeError, () => make({year: 1403, monthCode: code, day: 1}));
for (const era of ['sh', 'hs', 'AP']) error(RangeError, () => make({era, eraYear: 1403, month: 1, day: 1}));
error(RangeError, () => make({year: 1403, era: 'ap', eraYear: 1404, month: 1, day: 1}));
error(RangeError, () => make({year: 1403, month: 2, monthCode: 'M01', day: 1}));
error(TypeError, () => make({era: 'ap', month: 1, day: 1}));
for (const receiver of [leap, leap.toPlainDateTime({hour: 12}), leap.toPlainYearMonth()]) {
  same(receiver.with({monthCode: 'M02'}).month, 2, 'with month-code exclusion');
  same(receiver.with({era: 'ap', eraYear: 1404}).year, 1404, 'with era exclusion');
}
same(leap.with({year: 1404}).day, 29, 'with leap constrain');
same(leap.toPlainDateTime({hour: 12}).with({hour: 7}).day, 30, 'time-only with');
const zoned = Temporal.ZonedDateTime.from({calendar: 'persian', year: 1403, month: 12, day: 30, hour: 12, timeZone: 'UTC'});
same(zoned.with({year: 1404}).day, 29, 'zoned leap regulation');
same(zoned.with({monthCode: 'M02'}).month, 2, 'zoned field exclusion');
let log = [];
const bag = {
  get calendar() {log.push('calendar'); return 'persian';},
  get day() {log.push('day'); return 1;},
  get era() {log.push('era'); return 'ap';},
  get eraYear() {log.push('eraYear'); return 1403;},
  get month() {log.push('month'); return 1;},
  get monthCode() {log.push('monthCode'); return 'M01';},
  get year() {log.push('year'); return 1404;}
};
error(RangeError, () => Temporal.PlainDate.from(bag, {get overflow() {log.push('overflow'); return 'reject';}}));
same(log.join('|'), 'calendar|day|era|eraYear|month|monthCode|year|overflow', 'field and option order');
const marker = {};
log = [];
let published = 'before';
let finallyRuns = 0;
try {
  try {
    const result = Temporal.PlainDate.from({calendar: 'persian', get day() {throw marker;}, get year() {log.push('late year'); return 1403;}}, {get overflow() {log.push('late option'); return 'reject';}});
    published = result;
  } finally {finallyRuns++;}
  throw new Error('missing field throw');
} catch (caught) {
  if (caught !== marker) throw new Error('field throw identity');
}
same(published, 'before', 'no abrupt publication');
same(finallyRuns, 1, 'field finally');
same(log.length, 0, 'no late field or option reads');
print('persian-fields:ok');
262;
