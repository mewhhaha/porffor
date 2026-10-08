function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function error(kind, action) {
  try { action(); } catch (caught) {
    if (!(caught instanceof kind)) throw new Error('wrong field error');
    return;
  }
  throw new Error('missing field error');
}
const make = fields => Temporal.PlainDate.from(Object.assign({calendar: 'indian'}, fields));
const leap = make({year: 1942, monthCode: 'M01', day: 31});
same(leap.withCalendar('iso8601').toString(), '2020-04-20', 'calendar leap fields');
const common = make({year: 1943, month: 1, day: 31});
same(common.day, 30, 'common constrain');
error(RangeError, () => Temporal.PlainDate.from({calendar: 'indian', year: 1943, month: 1, day: 31}, {overflow: 'reject'}));
same(make({year: 1943, month: 99, day: 99}).month, 12, 'numeric month constrain');
same(make({year: 1943, month: 99, day: 99}).day, 30, 'numeric day constrain');
for (const code of ['M13', 'M00', 'M01L']) {
  error(RangeError, () => make({year: 1942, monthCode: code, day: 1}));
}
error(RangeError, () => make({year: 1942, era: 'shaka', eraYear: 1943, month: 1, day: 1}));
error(RangeError, () => make({year: 1942, era: 'saka', eraYear: 1942, month: 1, day: 1}));
error(RangeError, () => make({year: 1942, month: 2, monthCode: 'M01', day: 1}));
error(TypeError, () => make({era: 'shaka', month: 1, day: 1}));
for (const receiver of [leap, leap.toPlainDateTime({hour: 12}), leap.toPlainYearMonth()]) {
  const changedMonth = receiver.with({monthCode: 'M02'});
  same(changedMonth.year, 1942, 'with calendar year default');
  same(changedMonth.month, 2, 'with month-code exclusion');
  const changedYear = receiver.with({era: 'shaka', eraYear: 1943});
  same(changedYear.year, 1943, 'with era exclusion');
}
same(leap.with({year: 1943}).day, 30, 'with calendar leap constrain');
same(leap.toPlainDateTime({hour: 12}).with({hour: 7}).day, 31, 'time-only with keeps calendar date');
const zoned = Temporal.ZonedDateTime.from({calendar: 'indian', year: 1942, monthCode: 'M01', day: 31, hour: 12, timeZone: 'UTC'});
same(zoned.with({year: 1943}).day, 30, 'zoned with calendar regulation');
same(zoned.with({monthCode: 'M02'}).month, 2, 'zoned with month-code exclusion');
let log = [];
const bag = {
  get calendar() { log.push('calendar'); return 'indian'; },
  get day() { log.push('day'); return 1; },
  get era() { log.push('era'); return 'shaka'; },
  get eraYear() { log.push('eraYear'); return 1942; },
  get month() { log.push('month'); return 1; },
  get monthCode() { log.push('monthCode'); return 'M01'; },
  get year() { log.push('year'); return 1943; }
};
error(RangeError, () => Temporal.PlainDate.from(bag, {get overflow() {log.push('overflow'); return 'reject';}}));
same(log.join('|'), 'calendar|day|era|eraYear|month|monthCode|year|overflow', 'field and option reads');
const marker = {};
log = [];
let result = 'before';
let finallyRuns = 0;
try {
  try {
    result = Temporal.PlainDate.from({calendar: 'indian', get day() {throw marker;}, get year() {log.push('late year'); return 1942;}}, {get overflow() {log.push('late overflow'); return 'reject';}});
  } finally {finallyRuns++;}
  throw new Error('missing original field throw');
} catch (caught) {
  if (caught !== marker) throw new Error('field abrupt identity');
}
same(result, 'before', 'no field result publication');
same(finallyRuns, 1, 'field finally');
same(log.length, 0, 'field abrupt stops later reads');
print('indian-fields:ok');
262;
