function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function errorPrototype(action, prototype, label) {
  try {action();} catch (error) {
    same(Object.getPrototypeOf(error), prototype, label);
    return;
  }
  throw new Error('missing ' + label);
}
const minimum = new Temporal.PlainDate(-271821, 4, 19, 'indian');
const maximum = new Temporal.PlainDate(275760, 9, 13, 'indian');
for (const date of [minimum, maximum]) {
  const fields = {calendar: 'indian', year: date.year, monthCode: date.monthCode, day: date.day};
  same(Temporal.PlainDate.from(fields, {overflow: 'reject'}).equals(date), true, 'full carrier field round trip');
}
same(minimum.year, -271899, 'minimum calendar year');
same(minimum.month, 1, 'minimum calendar month');
same(minimum.day, 29, 'minimum calendar day');
same(maximum.year, 275682, 'maximum calendar year');
same(maximum.month, 6, 'maximum calendar month');
same(maximum.day, 22, 'maximum calendar day');
errorPrototype(() => minimum.subtract({days: 1}), RangeError.prototype, 'minimum date range');
errorPrototype(() => maximum.add({days: 1}), RangeError.prototype, 'maximum date range');
errorPrototype(() => Temporal.PlainDate.from({calendar: 'indian', year: 275682, month: 6, day: 23}), RangeError.prototype, 'calendar converted date range');
// YearMonth checks the ISO year/month, independently of PlainDate's day bound.
const lastMonth = Temporal.PlainYearMonth.from({calendar: 'indian', year: 275682, monthCode: 'M07'});
same(lastMonth.toString(), '+275760-09-23[u-ca=indian]', 'partial range beyond final ISO day');
errorPrototype(() => lastMonth.toPlainDate({day: 1}), RangeError.prototype, 'partial to full date range');
errorPrototype(() => Temporal.PlainYearMonth.from({calendar: 'indian', year: 275682, monthCode: 'M08'}), RangeError.prototype, 'partial maximum month');
const firstMonth = Temporal.PlainYearMonth.from({calendar: 'indian', year: -271899, monthCode: 'M02'});
same(firstMonth.toString(), '-271821-04-21[u-ca=indian]', 'partial first calendar day');
errorPrototype(() => Temporal.PlainYearMonth.from({calendar: 'indian', year: -271899, monthCode: 'M01'}), RangeError.prototype, 'partial minimum month');
// Admission uses intersection of the whole native year with the full carrier.
// A chosen month may fall outside that carrier or in the next ISO year.
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'indian', year: -271900, monthCode: 'M10', day: 11}, {overflow: 'constrain'}), RangeError.prototype, 'preceding native year cannot be constrained into range');
same(Temporal.PlainMonthDay.from({calendar: 'indian', year: -271899, monthCode: 'M01', day: 1}).toString(), '1972-03-21[u-ca=indian]', 'MonthDay minimum year outside full carrier');
same(Temporal.PlainMonthDay.from({calendar: 'indian', year: 275682, monthCode: 'M07', day: 1}).toString(), '1972-09-23[u-ca=indian]', 'MonthDay maximum year outside full carrier');
same(Temporal.PlainMonthDay.from({calendar: 'indian', year: 275682, monthCode: 'M10', day: 10}).toString(), '1972-12-31[u-ca=indian]', 'MonthDay last requested ISO-year day');
same(Temporal.PlainMonthDay.from({calendar: 'indian', year: 275682, monthCode: 'M10', day: 11}).toString(), '1972-01-01[u-ca=indian]', 'same native year admits chosen next ISO year');
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'indian', year: -271900, monthCode: 'M10', day: 10}), RangeError.prototype, 'whole preceding native year outside carrier');
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'indian', year: 275683, monthCode: 'M01', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'whole following native year outside carrier');
same(Temporal.PlainMonthDay.from({calendar: 'indian', era: 'shaka', eraYear: -271899, monthCode: 'M01', day: 1}).monthCode, 'M01', 'era-resolved extreme year admission');
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'indian', year: -271900, era: 'shaka', monthCode: 'M01', day: 1}), TypeError.prototype, 'incomplete era precedes native-year range');
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'indian', era: 'shaka', eraYear: 275683, monthCode: 'M12', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'era-resolved adjacent year rejects');
// ISO uses the supplied year only for overflow, without native-year admission.
for (const year of [-999999, 999999]) {
  const monthDay = Temporal.PlainMonthDay.from({calendar: 'iso8601', year, monthCode: 'M01', day: 1});
  same(monthDay.monthCode, 'M01', 'ISO distant year keeps admission exemption');
  same(monthDay.day, 1, 'ISO distant year keeps day');
}
// These calendars share Gregorian year starts with their native year offsets.
for (const [calendar, offset] of [
  ['gregory', 0], ['gregorian', 0], ['buddhist', 543], ['roc', -1911], ['japanese', 0]
]) {
  const minimumYear = -271821 + offset;
  const maximumYear = 275760 + offset;
  const first = Temporal.PlainMonthDay.from({calendar, year: minimumYear, monthCode: 'M01', day: 1});
  same(first.monthCode, 'M01', 'Gregorian arithmetic admits first native-year month');
  same(first.day, 1, 'Gregorian arithmetic admits day before full carrier minimum');
  const last = Temporal.PlainMonthDay.from({calendar, year: maximumYear, monthCode: 'M12', day: 31});
  same(last.monthCode, 'M12', 'Gregorian arithmetic admits last native-year month');
  same(last.day, 31, 'Gregorian arithmetic admits day after full carrier maximum');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: minimumYear - 1, monthCode: 'M12', day: 31}, {overflow: 'constrain'}), RangeError.prototype, 'Gregorian arithmetic rejects whole preceding native year');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: maximumYear + 1, monthCode: 'M01', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'Gregorian arithmetic rejects whole following native year');
}
const foreign = __lilaCreateRealm().global;
const LocalDate = Temporal.PlainDate;
const ForeignDate = foreign.Temporal.PlainDate;
const localFrom = LocalDate.from;
const foreignFrom = ForeignDate.from;
const localWith = LocalDate.prototype.with;
const foreignWith = ForeignDate.prototype.with;
const localAdd = LocalDate.prototype.add;
const foreignAdd = ForeignDate.prototype.add;
const LocalMonthDay = Temporal.PlainMonthDay;
const ForeignMonthDay = foreign.Temporal.PlainMonthDay;
const localMonthDayFrom = LocalMonthDay.from;
const foreignMonthDayFrom = ForeignMonthDay.from;
const localRange = RangeError.prototype;
const foreignRange = foreign.RangeError.prototype;
const localType = TypeError.prototype;
const foreignType = foreign.TypeError.prototype;
const localReceiver = localFrom({calendar: 'indian', year: 1942, month: 1, day: 31});
const foreignReceiver = foreignFrom({calendar: 'indian', year: 1942, month: 1, day: 31});
foreign.RangeError = foreign.TypeError = function wrongError() {throw 'mutable intrinsic constructor';};
foreign.Temporal = {};
for (let direction = 0; direction < 2; direction++) {
  const from = direction === 0 ? foreignFrom : localFrom;
  const nestedFrom = direction === 0 ? localFrom : foreignFrom;
  const withMethod = direction === 0 ? foreignWith : localWith;
  const addMethod = direction === 0 ? foreignAdd : localAdd;
  const monthDayFrom = direction === 0 ? foreignMonthDayFrom : localMonthDayFrom;
  const receiver = direction === 0 ? localReceiver : foreignReceiver;
  const resultPrototype = direction === 0 ? ForeignDate.prototype : LocalDate.prototype;
  const monthDayPrototype = direction === 0 ? ForeignMonthDay.prototype : LocalMonthDay.prototype;
  const rangePrototype = direction === 0 ? foreignRange : localRange;
  const typePrototype = direction === 0 ? foreignType : localType;
  const result = from({calendar: 'indian', year: 1943, month: 2, day: 1});
  same(Object.getPrototypeOf(result), resultPrototype, 'called Realm calendar result');
  same(result.withCalendar('iso8601').toString(), '2021-04-21', 'called Realm calendar conversion');
  const monthDay = monthDayFrom({calendar: 'indian', year: 275682, monthCode: 'M07', day: 1});
  same(Object.getPrototypeOf(monthDay), monthDayPrototype, 'called Realm MonthDay reference result');
  same(monthDay.toString(), '1972-09-23[u-ca=indian]', 'called Realm MonthDay reference conversion');
  errorPrototype(() => monthDayFrom({calendar: 'indian', year: 275683, monthCode: 'M10', get day() {
    nestedFrom({calendar: 'indian', year: 1942, month: 2, day: 1});
    return 11;
  }}), rangePrototype, 'nested whole-native-year admission Realm');
  let reads = 0;
  errorPrototype(() => from({calendar: 'indian', year: 1943, month: 1, get day() {
    reads++;
    nestedFrom({calendar: 'indian', year: 1942, month: 2, day: 1});
    return 31;
  }}, {overflow: 'reject'}), rangePrototype, 'nested calendar field RangeError Realm');
  same(reads, 1, 'nested field read');
  errorPrototype(() => from({calendar: 'indian', year: 1942, month: 1, get day() {
    nestedFrom({calendar: 'indian', year: 1942, month: 2, day: 1});
    return Symbol('invalid day');
  }}), typePrototype, 'nested calendar field TypeError Realm');
  errorPrototype(() => withMethod.call(receiver, {year: 1943}, {overflow: 'reject'}), rangePrototype, 'borrowed with calendar error Realm');
  errorPrototype(() => addMethod.call(receiver, {years: 1}, {overflow: 'reject'}), rangePrototype, 'borrowed add calendar error Realm');
  const marker = {};
  try {
    from({calendar: 'indian', get day() {throw marker;}});
    throw new Error('missing original Realm field throw');
  } catch (error) {
    if (error !== marker) throw new Error('Realm field abrupt identity');
  }
}
print('indian-limits-and-realms:ok');
262;
