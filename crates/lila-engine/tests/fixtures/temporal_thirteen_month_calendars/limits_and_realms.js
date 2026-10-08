function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function errorPrototype(action, prototype, label) {
  try {action();} catch (caught) {
    same(Object.getPrototypeOf(caught), prototype, label);
    return;
  }
  throw new Error('missing ' + label);
}
// Pinned Test262 full-date extremes differ from ZonedDateTime's first day.
const families = [
  ['coptic', -272099, 275471, 1740, 'am'],
  ['ethiopic', -271823, 275747, 2016, 'am'],
  ['ethioaa', -266323, 281247, 7516, 'aa']
];
for (const [calendar, minYear, maxYear] of families) {
  const minimum = new Temporal.PlainDate(-271821, 4, 19, calendar);
  const maximum = new Temporal.PlainDate(275760, 9, 13, calendar);
  same(minimum.year, minYear, 'minimum projected year');
  same(minimum.month, 3, 'minimum projected month');
  same(minimum.day, 23, 'minimum projected day');
  same(maximum.year, maxYear, 'maximum projected year');
  same(maximum.month, 5, 'maximum projected month');
  same(maximum.day, 22, 'maximum projected day');
  for (const date of [minimum, maximum]) {
    same(Temporal.PlainDate.from({calendar, year: date.year, monthCode: date.monthCode, day: date.day}, {overflow: 'reject'}).equals(date), true, 'full carrier round trip');
  }
  errorPrototype(() => minimum.subtract({days: 1}), RangeError.prototype, 'minimum full-date range');
  errorPrototype(() => maximum.add({days: 1}), RangeError.prototype, 'maximum full-date range');
  errorPrototype(() => Temporal.PlainDate.from({calendar, year: maxYear, month: 5, day: 23}), RangeError.prototype, 'converted full-date range');
  // Calendar day one can lie in a forbidden ISO month even for a valid date.
  errorPrototype(() => minimum.toPlainYearMonth(), RangeError.prototype, 'minimum date to preceding ISO month');
  errorPrototype(() => Temporal.PlainYearMonth.from({calendar, year: minYear, monthCode: 'M03'}), RangeError.prototype, 'minimum YearMonth reference');
  same(Temporal.PlainYearMonth.from({calendar, year: minYear, monthCode: 'M04'}).toString(), '-271821-04-27[u-ca=' + calendar + ']', 'first allowed YearMonth reference');
  const finalMonth = Temporal.PlainYearMonth.from({calendar, year: maxYear, monthCode: 'M06'});
  same(finalMonth.toString(), '+275760-09-22[u-ca=' + calendar + ']', 'YearMonth allowed ISO month');
  errorPrototype(() => finalMonth.toPlainDate({day: 1}), RangeError.prototype, 'YearMonth to out-of-range full date');
  errorPrototype(() => Temporal.PlainYearMonth.from({calendar, year: maxYear, monthCode: 'M07'}), RangeError.prototype, 'following YearMonth ISO month');
  // The whole native year intersects the carrier; its selected month need not.
  const belowFullDate = Temporal.PlainMonthDay.from({calendar, year: minYear, monthCode: 'M01', day: 1});
  same(belowFullDate.toString(), '1972-09-11[u-ca=' + calendar + ']', 'MonthDay before minimum full date');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: minYear - 1, monthCode: 'M12', day: 10}, {overflow: 'constrain'}), RangeError.prototype, 'preceding native year cannot be constrained into range');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: minYear - 1, monthCode: 'M12', day: 9}), RangeError.prototype, 'whole preceding native year outside carrier');
  same(Temporal.PlainMonthDay.from({calendar, year: maxYear, monthCode: 'M06', day: 1}).monthCode, 'M06', 'MonthDay after maximum full date');
  same(Temporal.PlainMonthDay.from({calendar, year: maxYear, monthCode: 'M09', day: 11}).day, 11, 'MonthDay last requested ISO-year day');
  same(Temporal.PlainMonthDay.from({calendar, year: maxYear, monthCode: 'M09', day: 12}).day, 12, 'same native year admits chosen next ISO year');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: maxYear + 1, monthCode: 'M01', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'whole following native year outside carrier');
  const minimumEra = calendar === 'coptic' ? 'am' : 'aa';
  const minimumEraYear = calendar === 'ethiopic' ? minYear + 5500 : minYear;
  same(Temporal.PlainMonthDay.from({calendar, era: minimumEra, eraYear: minimumEraYear, monthCode: 'M01', day: 1}).monthCode, 'M01', 'era-resolved extreme year admission');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: minYear - 1, era: minimumEra, monthCode: 'M01', day: 1}), TypeError.prototype, 'incomplete era precedes native-year range');
  const maximumEra = calendar === 'ethioaa' ? 'aa' : 'am';
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, era: maximumEra, eraYear: maxYear + 1, monthCode: 'M13', day: 5}, {overflow: 'constrain'}), RangeError.prototype, 'era-resolved adjacent year rejects');
  for (const year of [-999999, 999999]) {
    errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year, monthCode: 'M13', day: 6}), RangeError.prototype, 'MonthDay year before month information');
  }
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
const foreignMarker = foreign.Object();
const receiverPairs = families.map(([calendar, minYear, maxYear, commonYear]) => [
  localFrom({calendar, year: commonYear - 1, month: 13, day: 6}),
  foreignFrom({calendar, year: commonYear - 1, month: 13, day: 6})
]);
foreign.RangeError = foreign.TypeError = function wrongError() {throw 'mutable error constructor';};
foreign.Temporal = {};
globalThis.RangeError = globalThis.TypeError = function wrongError() {throw 'mutable local error constructor';};
for (let index = 0; index < families.length; index++) {
  const [calendar, minYear, maxYear, commonYear] = families[index];
  for (let direction = 0; direction < 2; direction++) {
    const from = direction === 0 ? foreignFrom : localFrom;
    const nestedFrom = direction === 0 ? localFrom : foreignFrom;
    const withMethod = direction === 0 ? foreignWith : localWith;
    const addMethod = direction === 0 ? foreignAdd : localAdd;
    const monthDayFrom = direction === 0 ? foreignMonthDayFrom : localMonthDayFrom;
    const receiver = receiverPairs[index][direction];
    const resultPrototype = direction === 0 ? ForeignDate.prototype : LocalDate.prototype;
    const monthDayPrototype = direction === 0 ? ForeignMonthDay.prototype : LocalMonthDay.prototype;
    const rangePrototype = direction === 0 ? foreignRange : localRange;
    const typePrototype = direction === 0 ? foreignType : localType;
    const result = from({calendar, year: commonYear, monthCode: 'M13', day: 5});
    same(Object.getPrototypeOf(result), resultPrototype, 'called Realm full-date result');
    same(result.withCalendar('iso8601').toString(), '2024-09-10', 'called Realm conversion');
    const md = monthDayFrom({calendar, year: maxYear, monthCode: 'M06', day: 1});
    same(Object.getPrototypeOf(md), monthDayPrototype, 'called Realm MonthDay result');
    same(md.monthCode, 'M06', 'called Realm reference projection');
    let reads = 0;
    errorPrototype(() => from({calendar, year: commonYear, month: 13, get day() {
      reads++;
      nestedFrom({calendar, year: commonYear, month: 1, day: 1});
      return 6;
    }}, {overflow: 'reject'}), rangePrototype, 'nested field RangeError Realm');
    same(reads, 1, 'nested field once');
    errorPrototype(() => from({calendar, year: commonYear, month: 1, get day() {
      nestedFrom({calendar, year: commonYear, month: 2, day: 1});
      return Symbol('invalid day');
    }}), typePrototype, 'nested field TypeError Realm');
    errorPrototype(() => monthDayFrom({calendar, year: maxYear + 1, monthCode: 'M09', get day() {
      nestedFrom({calendar, year: commonYear, month: 1, day: 1});
      return 12;
    }}), rangePrototype, 'nested whole-native-year error Realm');
    errorPrototype(() => withMethod.call(receiver, {year: commonYear}, {overflow: 'reject'}), rangePrototype, 'borrowed with error Realm');
    errorPrototype(() => addMethod.call(receiver, {years: 1}, {overflow: 'reject'}), rangePrototype, 'borrowed add error Realm');
    let published = 'before';
    let finallyRuns = 0;
    try {
      try {
        const value = from({calendar, get day() {throw foreignMarker;}});
        published = value;
      } finally {finallyRuns++;}
      throw new Error('missing original Realm field throw');
    } catch (caught) {
      if (caught !== foreignMarker) throw new Error('Realm field throw identity');
    }
    same(published, 'before', 'no Realm abrupt publication');
    same(finallyRuns, 1, 'Realm field finally');
  }
}
print('thirteen-month-limits-and-realms:ok');
262;
