function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function throwsKind(action, kind, label) {
  try { action(); } catch (caught) { if (!(caught instanceof kind)) throw caught; return; }
  throw new Error('missing ' + label);
}
const calendar = 'hebrew';
const from = Temporal.PlainDate.from;
const reject = {overflow: 'reject'};
for (const [year, month, code] of [[5783,6,'M06'], [5784,6,'M05L'], [5784,7,'M06'], [5784,13,'M12']]) {
  const date = from({calendar, year, month, monthCode: code, day: 1}, reject);
  same(date.month, month, 'agreed ordinal');
  same(date.monthCode, code, 'agreed code');
  same(Temporal.PlainYearMonth.from({calendar, year, monthCode: code}).month, month, 'YearMonth agreement');
  same(Temporal.PlainDateTime.from({calendar, year, monthCode: code, day: 1}).month, month, 'DateTime agreement');
  same(Temporal.ZonedDateTime.from({calendar, year, monthCode: code, day: 1, timeZone: 'UTC'}).month, month, 'zoned agreement');
}
for (const code of ['M13', 'M01L', 'M06L', 'M12L', 'M00', 'M14']) {
  for (const year of [5783, 5784]) throwsKind(() => from({calendar, year, monthCode: code, day: 1}), RangeError, 'invalid canonical code');
}
same(from({calendar, year: 5783, month: 6, monthCode: 'M05L', day: 30}).monthCode, 'M06', 'missing leap constrains forward');
same(from({calendar, year: 5783, month: 6, monthCode: 'M05L', day: 30}).day, 29, 'day clamps after code');
throwsKind(() => from({calendar, year: 5783, month: 6, monthCode: 'M05L', day: 1}, reject), RangeError, 'retained original missing leap rejects');
throwsKind(() => from({calendar, year: 5783, month: 5, monthCode: 'M05L', day: 1}), RangeError, 'constrained agreement mismatch');
throwsKind(() => from({calendar, year: 5784, month: 6, monthCode: 'M06', day: 1}), RangeError, 'leap ordinal mismatch');
// All four .with owners retain canonical receiver code across a year change.
const common = from({calendar, year: 5786, monthCode: 'M12', day: 1});
const leap = from({calendar, year: 5784, monthCode: 'M05L', day: 1});
for (const original of [common, common.toPlainYearMonth(), common.toPlainDateTime({hour: 12}), Temporal.ZonedDateTime.from({calendar, year: 5786, monthCode: 'M12', day: 1, timeZone: 'UTC'})]) {
  const changed = original.with({year: 5784}, reject);
  same(changed.monthCode, 'M12', 'with keeps code');
  same(changed.month, 13, 'with resolves target ordinal');
  same(original.with({era: 'am', eraYear: 5784}, reject).month, 13, 'with era keeps code');
}
for (const original of [leap, leap.toPlainYearMonth(), leap.toPlainDateTime({hour: 12}), Temporal.ZonedDateTime.from({calendar, year: 5784, monthCode: 'M05L', day: 1, timeZone: 'UTC'})]) {
  same(original.with({year: 5785}).monthCode, 'M06', 'with missing leap constrain');
  throwsKind(() => original.with({year: 5785}, reject), RangeError, 'with keeps original code for reject');
  same(original.with({year: 5785, monthCode: 'M07'}, reject).monthCode, 'M07', 'incoming code replaces snapshot');
}
let log = [];
const code = { [Symbol.toPrimitive](hint) { log.push('code:' + hint); return 'M05L'; } };
const bag = {
  get calendar() {log.push('calendar'); return calendar;},
  get day() {log.push('day'); return 1;},
  get era() {log.push('era'); return undefined;},
  get eraYear() {log.push('eraYear'); return undefined;},
  get month() {log.push('month'); return 6;},
  get monthCode() {log.push('monthCode'); return code;},
  get year() {log.push('year'); return 5783;}
};
throwsKind(() => from(bag, {get overflow() {log.push('overflow'); return 'reject';}}), RangeError, 'late original-code reject');
same(log.join('|'), 'calendar|day|era|eraYear|month|monthCode|code:string|year|overflow', 'complete acquisition before suitability/overflow');
log = [];
const marker = {};
try {
  from({calendar, get day() {log.push('day'); return 1;}, get monthCode() {log.push('code'); return {[Symbol.toPrimitive]() {throw marker;}};}, get year() {log.push('year'); return 5784;}}, {get overflow() {log.push('overflow'); return 'constrain';}});
  throw new Error('missing hook throw');
} catch (caught) {if (caught !== marker) throw new Error('original hook identity');}
same(log.join('|'), 'day|code', 'syntax hook prevents later fields/options');
for (const year of [Number.MAX_SAFE_INTEGER, -Number.MAX_SAFE_INTEGER]) {
  throwsKind(() => from({calendar, year, month: 1, monthCode: 'M12', day: 1}), RangeError, 'agreement before unsafe native multiplication');
}
print('hebrew-fields:ok');
262;
