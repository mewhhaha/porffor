function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function errorPrototype(action, prototype, label) {
  try {action();} catch (caught) {same(Object.getPrototypeOf(caught), prototype, label); return;}
  throw new Error('missing ' + label);
}
for (const calendar of ['chinese', 'dangi']) {
  const minimum = new Temporal.PlainDate(-271821,4,19,calendar);
  const maximum = new Temporal.PlainDate(275760,9,13,calendar);
  same(minimum.year, -271821, 'minimum native year follows January/February New Year');
  same(maximum.year, 275760, 'maximum native year follows January/February New Year');
  for (const date of [minimum, maximum]) {
    const fields = {calendar, year: date.year, monthCode: date.monthCode, day: date.day};
    same(Temporal.PlainDate.from(fields, {overflow: 'reject'}).equals(date), true, 'completed full-carrier extreme round trip');
    const reference = Temporal.PlainMonthDay.from(fields);
    same(reference.day, date.day, 'extreme whole-year reference preserves day');
    same(reference.monthCode === date.monthCode || reference.monthCode === date.monthCode.slice(0, 3), true, 'extreme reference applies only finite-table leap-code constraint');
  }
  errorPrototype(() => minimum.subtract({days: 1}), RangeError.prototype, 'full-date minimum');
  errorPrototype(() => maximum.add({days: 1}), RangeError.prototype, 'full-date maximum');
  // The chosen month may lie outside the full carrier if its year intersects.
  same(Temporal.PlainMonthDay.from({calendar, year: -271821, monthCode: 'M01', day: 1}).monthCode, 'M01', 'minimum whole native year admits early month');
  same(Temporal.PlainMonthDay.from({calendar, year: 275760, monthCode: 'M12', day: 29}).monthCode, 'M12', 'maximum whole native year admits late month');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: -271822, monthCode: 'M12', day: 29}, {overflow: 'constrain'}), RangeError.prototype, 'preceding whole native year rejects');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: 275761, monthCode: 'M01', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'following whole native year rejects');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: -271822, monthCode: 'M12'}), TypeError.prototype, 'missing day precedes native-year range');
  errorPrototype(() => Temporal.PlainYearMonth.from({calendar, year: -271821, monthCode: 'M01'}), RangeError.prototype, 'YearMonth keeps minimum ISO month limit');
  const lastYearFirstMonth = Temporal.PlainYearMonth.from({calendar, year: 275760, monthCode: 'M01'});
  same(lastYearFirstMonth.year, 275760, 'YearMonth final native year early month');
  same(lastYearFirstMonth.toPlainDate({day: 1}).monthCode, 'M01', 'partial to completed full date');
  errorPrototype(() => Temporal.PlainYearMonth.from({calendar, year: 275761, monthCode: 'M01'}), RangeError.prototype, 'YearMonth following ISO year rejects');
}
const foreign = __lilaCreateRealm().global;
const LocalDate = Temporal.PlainDate;
const ForeignDate = foreign.Temporal.PlainDate;
const LocalMonthDay = Temporal.PlainMonthDay;
const ForeignMonthDay = foreign.Temporal.PlainMonthDay;
const localFrom = LocalDate.from;
const foreignFrom = ForeignDate.from;
const localMonthDayFrom = LocalMonthDay.from;
const foreignMonthDayFrom = ForeignMonthDay.from;
const localWith = LocalDate.prototype.with;
const foreignWith = ForeignDate.prototype.with;
const localAdd = LocalDate.prototype.add;
const foreignAdd = ForeignDate.prototype.add;
const localRange = RangeError.prototype;
const foreignRange = foreign.RangeError.prototype;
const localType = TypeError.prototype;
const foreignType = foreign.TypeError.prototype;
foreign.RangeError = foreign.TypeError = function wrongError() {throw 'mutable error constructor';};
foreign.Temporal = {};
for (const calendar of ['chinese', 'dangi']) {
  const localReceiver = localFrom({calendar, year: 2020, monthCode: 'M04L', day: 1});
  const foreignReceiver = foreignFrom({calendar, year: 2020, monthCode: 'M04L', day: 1});
  for (let direction = 0; direction < 2; direction++) {
    const from = direction === 0 ? foreignFrom : localFrom;
    const nestedFrom = direction === 0 ? localFrom : foreignFrom;
    const monthDayFrom = direction === 0 ? foreignMonthDayFrom : localMonthDayFrom;
    const withMethod = direction === 0 ? foreignWith : localWith;
    const addMethod = direction === 0 ? foreignAdd : localAdd;
    const receiver = direction === 0 ? localReceiver : foreignReceiver;
    const datePrototype = direction === 0 ? ForeignDate.prototype : LocalDate.prototype;
    const monthDayPrototype = direction === 0 ? ForeignMonthDay.prototype : LocalMonthDay.prototype;
    const rangePrototype = direction === 0 ? foreignRange : localRange;
    const typePrototype = direction === 0 ? foreignType : localType;
    const result = from({calendar, year: 1999, monthCode: 'M11', day: 25});
    same(Object.getPrototypeOf(result), datePrototype, 'called Realm calendar result');
    same(result.withCalendar('iso8601').toString(), '2000-01-01', 'called Realm literal conversion');
    const monthDay = monthDayFrom({calendar, monthCode: 'M09L', day: 1});
    same(Object.getPrototypeOf(monthDay), monthDayPrototype, 'called Realm reference result');
    same(Number(monthDay.toString().slice(0,4)), 2014, 'called Realm future reference table');
    let reads = 0;
    errorPrototype(() => from({calendar, year: 2021, monthCode: 'M04L', get day() {
      reads++;
      nestedFrom({calendar, year: 2000, monthCode: 'M01', day: 1});
      return 1;
    }}, {overflow: 'reject'}), rangePrototype, 'nested original-code RangeError Realm');
    same(reads, 1, 'nested field read exactly once');
    errorPrototype(() => monthDayFrom({calendar, year: 275761, monthCode: 'M01', get day() {
      nestedFrom({calendar, year: 2000, monthCode: 'M01', day: 1});
      return 1;
    }}), rangePrototype, 'nested whole-native-year RangeError Realm');
    errorPrototype(() => from({calendar, year: 2020, monthCode: 'M01', get day() {
      nestedFrom({calendar, year: 2000, monthCode: 'M01', day: 1});
      return Symbol('invalid day');
    }}), typePrototype, 'nested TypeError Realm');
    errorPrototype(() => withMethod.call(receiver, {year: 2021}, {overflow: 'reject'}), rangePrototype, 'borrowed with original-code Realm');
    errorPrototype(() => addMethod.call(receiver, {years: 1}, {overflow: 'reject'}), rangePrototype, 'borrowed add original-code Realm');
    const marker = {};
    try {
      from({calendar, get day() {throw marker;}});
      throw new Error('missing original field throw');
    } catch (caught) {if (caught !== marker) throw new Error('original Realm field identity');}
  }
}
print('east-asian-limits-and-realms:ok');
262;
