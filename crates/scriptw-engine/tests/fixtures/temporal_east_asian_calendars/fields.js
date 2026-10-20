function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function throwsKind(action, kind, label) {
  try {action();} catch (caught) {if (!(caught instanceof kind)) throw caught; return;}
  throw new Error('missing ' + label);
}
for (const calendar of ['chinese', 'dangi']) {
  const from = Temporal.PlainDate.from;
  const reject = {overflow: 'reject'};
  for (const [year, ordinal, code] of [[2001,5,'M04L'], [2004,3,'M02L'], [2020,5,'M04L'], [2020,6,'M05'], [2020,13,'M12'], [2021,12,'M12']]) {
    const bag = {calendar, year, month: ordinal, monthCode: code, day: 1};
    for (const value of [from(bag, reject), Temporal.PlainDateTime.from(bag, reject), Temporal.PlainYearMonth.from(bag, reject), Temporal.ZonedDateTime.from({...bag, timeZone: 'UTC'}, reject)]) {
      same(value.year, year, 'resolved year');
      same(value.month, ordinal, 'constrained code agreement');
      same(value.monthCode, code, 'canonical code');
    }
  }
  for (const code of ['M13', 'M00', 'M14', 'M13L', 'm04', 'M4L']) {
    throwsKind(() => from({calendar, year: 2020, monthCode: code, day: 1}), RangeError, 'code suitability');
  }
  const missing = {calendar, year: 2021, month: 4, monthCode: 'M04L', day: 1};
  same(from(missing).monthCode, 'M04', 'missing leap constrains backward');
  same(from(missing).month, 4, 'missing leap agreement ordinal');
  throwsKind(() => from(missing, reject), RangeError, 'original missing leap retained for reject');
  throwsKind(() => from({...missing, month: 5}), RangeError, 'agreement precedes late leap regulation');
  throwsKind(() => from({calendar, month: 6, monthCode: 'M05', day: 1}), TypeError, 'missing year precedes agreement');
  throwsKind(() => from({calendar, year: 2020, day: 32}, reject), TypeError, 'missing month precedes overflow');
  throwsKind(() => from({calendar, year: 2020, month: 12, monthCode: 'M05'}), TypeError, 'missing day precedes agreement');
  const noEra = from({calendar, year: 2020, era: 'unused', eraYear: 100, monthCode: 'M01', day: 1});
  same(noEra.year, 2020, 'from ignores era for calendar without eras');
  throwsKind(() => from({calendar, era: 'unused', eraYear: 2020, monthCode: 'M01', day: 1}), TypeError, 'era cannot replace native year');
  const common = from({calendar, year: 2000, monthCode: 'M08', day: 2});
  const leap = from({calendar, year: 2020, monthCode: 'M04L', day: 1});
  for (const value of [common, common.toPlainYearMonth(), common.toPlainMonthDay(), common.toPlainDateTime({hour: 12}), Temporal.ZonedDateTime.from({calendar, year: 2000, monthCode: 'M08', day: 2, timeZone: 'UTC'})]) {
    const changed = value.with({year: 2001}, reject);
    same(changed.monthCode, 'M08', 'with preserves receiver canonical code');
    if (changed.month !== undefined) same(changed.month, 9, 'with resolves target leap ordinal');
    same(value.with({year: 2001, month: 3}, reject).monthCode, 'M03', 'incoming month replaces code');
  }
  for (const value of [leap, leap.toPlainYearMonth(), leap.toPlainDateTime({hour: 12}), Temporal.ZonedDateTime.from({calendar, year: 2020, monthCode: 'M04L', day: 1, timeZone: 'UTC'})]) {
    same(value.with({year: 2021}).monthCode, 'M04', 'with missing leap constrains backward');
    throwsKind(() => value.with({year: 2021}, reject), RangeError, 'with original leap reject');
    throwsKind(() => value.with({era: 'ce', eraYear: 2021}), TypeError, 'with era-only bag has no calendar fields');
  }
  let log = [];
  const code = {[Symbol.toPrimitive](hint) {log.push('code:' + hint); return 'M04L';}};
  const bag = {
    get calendar() {log.push('calendar'); return calendar;},
    get day() {log.push('day'); return 1;},
    get era() {throw new Error('era read');},
    get eraYear() {throw new Error('eraYear read');},
    get month() {log.push('month'); return 4;},
    get monthCode() {log.push('monthCode'); return code;},
    get year() {log.push('year'); return 2021;}
  };
  throwsKind(() => from(bag, {get overflow() {log.push('overflow'); return 'reject';}}), RangeError, 'late original-code rejection');
  same(log.join('|'), 'calendar|day|month|monthCode|code:string|year|overflow', 'complete no-era field acquisition');
  log = [];
  const marker = {};
  try {
    from({calendar, get day() {log.push('day'); return 1;}, get monthCode() {log.push('code'); return {[Symbol.toPrimitive]() {throw marker;}};}, get year() {log.push('year'); return 2020;}}, {get overflow() {log.push('overflow'); return 'constrain';}});
    throw new Error('missing original hook throw');
  } catch (caught) {if (caught !== marker) throw new Error('hook abrupt identity');}
  same(log.join('|'), 'day|code', 'abrupt syntax hook stops later acquisition');
  for (const year of [Number.MAX_SAFE_INTEGER, -Number.MAX_SAFE_INTEGER]) {
    throwsKind(() => from({calendar, year, monthCode: 'M01', day: 1}), RangeError, 'envelope before model arithmetic');
  }
}
print('east-asian-fields:ok');
262;
