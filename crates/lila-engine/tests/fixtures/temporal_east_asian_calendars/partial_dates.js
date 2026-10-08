function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function throwsKind(action, kind, label) {
  try {action();} catch (caught) {if (!(caught instanceof kind)) throw caught; return;}
  throw new Error('missing ' + label);
}
function reference(value, code, day, year) {
  same(value.monthCode, code, 'reference canonical code');
  same(value.day, day, 'reference day');
  same(Number(value.toString().slice(0, 4)), year, 'literal table ISO reference year');
}
for (const calendar of ['chinese', 'dangi']) {
  const md = (code, day, options) => Temporal.PlainMonthDay.from({calendar, monthCode: code, day}, options);
  const reject = {overflow: 'reject'};
  const regular30 = [1970,1972,calendar === 'chinese' ? 1966 : 1968,1970,1972,1971,1972,1971,1972,1972,1970,1972];
  for (let month = 1; month <= 12; month++) {
    const code = 'M' + String(month).padStart(2, '0');
    reference(md(code,1,reject), code,1,1972);
    reference(md(code,29,reject), code,29,1972);
    reference(md(code,30,reject), code,30,regular30[month - 1]);
    reference(md(code,31), code,30,regular30[month - 1]);
    throwsKind(() => md(code,31,reject), RangeError, 'day overflow before reference availability');
  }
  const leapRows = [
    ['M02L',1947,1947], ['M03L',1966,1966], ['M04L',1963,1963],
    ['M05L',1971,1971], ['M06L',1960,1960], ['M07L',1968,1968],
    ['M08L',1957,1957], ['M09L',2014,2014], ['M10L',1984,1984], ['M11L',2033,2034]
  ];
  for (const [code, firstYear, lastYear] of leapRows) {
    reference(md(code,1,reject), code,1,firstYear);
    reference(md(code,29,reject), code,29,lastYear);
  }
  reference(md('M11L',10,reject), 'M11L',10,2033);
  reference(md('M11L',11,reject), 'M11L',11,2034);
  for (const [code, year] of [['M03L',1955], ['M04L',1944], ['M05L',1952], ['M06L',1941], ['M07L',1938]]) {
    reference(md(code,30,reject), code,30,year);
    reference(md(code,31), code,30,year);
    throwsKind(() => md(code,31,reject), RangeError, 'long leap day overflow');
  }
  // These are suitable leap codes, with unavailable reference combinations.
  for (const code of ['M01L','M12L']) {
    const regular = code.slice(0,3);
    reference(md(code,29), regular,29,1972);
    throwsKind(() => md(code,1,reject), RangeError, 'historically unavailable leap reference');
  }
  for (const code of ['M01L','M02L','M08L','M09L','M10L','M11L','M12L']) {
    const regular = code.slice(0,3);
    const expected = md(regular,30,reject);
    for (const day of [30,31]) {
      const constrained = md(code,day);
      same(constrained.monthCode, regular, 'unavailable reference changes only leap code');
      same(constrained.day, 30, 'unavailable reference preserves regulated day30');
      same(constrained.equals(expected), true, 'unavailable reference uses regular table row');
      throwsKind(() => md(code,day,reject), RangeError, 'unavailable day30 rejects');
    }
  }
  throwsKind(() => md('M13',1), RangeError, 'M13 remains unsuitable');
  throwsKind(() => Temporal.PlainMonthDay.from({calendar, month: 5, monthCode: 'M04', day: 1}), TypeError, 'numeric month needs year before agreement');
  throwsKind(() => Temporal.PlainMonthDay.from({calendar, year: 2020, month: 5, monthCode: 'M04', day: 1}), RangeError, 'supplied-year code disagreement');
  const shifted = Temporal.PlainMonthDay.from({calendar, year: 2004, month: 5, monthCode: 'M04', day: 1});
  reference(shifted, 'M04',1,1972);
  same(Temporal.PlainDate.from(shifted.toString()).month, 4, 'reference ordinal differs from supplied ordinal');
  reference(Temporal.PlainMonthDay.from({calendar, year: 2021, monthCode: 'M04L', day: 1}), 'M04',1,1972);
  throwsKind(() => Temporal.PlainMonthDay.from({calendar, year: 2021, monthCode: 'M04L', day: 1}, reject), RangeError, 'supplied-year original missing leap rejects');
  const existingLeap = Temporal.PlainDate.from({calendar, year: 2020, monthCode: 'M04L', day: 1}, reject);
  reference(existingLeap.toPlainMonthDay(), 'M04L',1,1963);
  same(existingLeap.toPlainMonthDay().equals(Temporal.PlainMonthDay.from({calendar, year: 2020, monthCode: 'M04L', day: 1}, reject)), true, 'both actual reference factories');
  // The reference is common-year ordinal6; the requested year has ordinal7.
  const toDate = md('M06',1,reject).toPlainDate({year: 2020});
  same(toDate.year, 2020, 'MonthDay toPlainDate resolves supplied year');
  same(toDate.monthCode, 'M06', 'MonthDay toPlainDate preserves code');
  same(toDate.month, 7, 'MonthDay toPlainDate re-resolves ordinal');
  const ym = Temporal.PlainYearMonth.from({calendar, year: 2020, monthCode: 'M04L'}, reject);
  same(ym.month, 5, 'YearMonth leap ordinal');
  same(ym.monthsInYear, 13, 'YearMonth native count');
  same(ym.toPlainDate({day: 1}).equals(existingLeap), true, 'YearMonth completed full date');
  same(ym.with({year: 2021}).monthCode, 'M04', 'YearMonth missing leap constrain');
  throwsKind(() => ym.with({year: 2021}, reject), RangeError, 'YearMonth original missing leap rejects');
  throwsKind(() => Temporal.PlainMonthDay.from({calendar, year: 2020, monthCode: 'M04L'}), TypeError, 'missing day precedes reference processing');
}
print('east-asian-partial-dates:ok');
262;
