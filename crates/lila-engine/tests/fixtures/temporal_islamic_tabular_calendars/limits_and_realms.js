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
const families = [
  ['islamic-civil', 21, 23, '-271821-04-29', '+275760-09-21', 1, 14],
  ['islamic-tbla', 22, 24, '-271821-04-28', '+275760-09-20', 2, 15]
];
for (const [calendar, minDay, maxDay, firstMonth, finalMonth, firstYearDay, finalYearDay] of families) {
  const minYear = -280804;
  const maxYear = 283583;
  const minimum = new Temporal.PlainDate(-271821, 4, 19, calendar);
  const maximum = new Temporal.PlainDate(275760, 9, 13, calendar);
  same(minimum.year, minYear, 'minimum projected year');
  same(minimum.month, 3, 'minimum month');
  same(minimum.day, minDay, 'minimum day');
  same(minimum.era, 'bh', 'minimum negative era');
  same(minimum.eraYear, 280805, 'minimum era year');
  same(maximum.year, maxYear, 'maximum projected year');
  same(maximum.month, 5, 'maximum month');
  same(maximum.day, maxDay, 'maximum day');
  for (const date of [minimum, maximum]) {
    same(Temporal.PlainDate.from({calendar, year: date.year, monthCode: date.monthCode, day: date.day}, {overflow: 'reject'}).equals(date), true, 'full carrier round trip');
  }
  errorPrototype(() => minimum.subtract({days: 1}), RangeError.prototype, 'minimum full date');
  errorPrototype(() => maximum.add({days: 1}), RangeError.prototype, 'maximum full date');
  errorPrototype(() => Temporal.PlainDate.from({calendar, year: maxYear, month: 5, day: maxDay + 1}), RangeError.prototype, 'converted full date');
  // Day-one references use ISO month limits, not full-date boundary days.
  errorPrototype(() => minimum.toPlainYearMonth(), RangeError.prototype, 'minimum to forbidden ISO month');
  errorPrototype(() => Temporal.PlainYearMonth.from({calendar, year: minYear, monthCode: 'M03'}), RangeError.prototype, 'first YearMonth reference');
  same(Temporal.PlainYearMonth.from({calendar, year: minYear, monthCode: 'M04'}).toString(), firstMonth + '[u-ca=' + calendar + ']', 'first allowed ISO month');
  const last = Temporal.PlainYearMonth.from({calendar, year: maxYear, monthCode: 'M06'});
  same(last.toString(), finalMonth + '[u-ca=' + calendar + ']', 'last allowed ISO month');
  errorPrototype(() => last.toPlainDate({day: 1}), RangeError.prototype, 'partial to out-of-range full date');
  errorPrototype(() => Temporal.PlainYearMonth.from({calendar, year: maxYear, monthCode: 'M07'}), RangeError.prototype, 'following ISO month');
  // Admission uses the whole native year, independently of the selected date.
  same(Temporal.PlainMonthDay.from({calendar, year: minYear, monthCode: 'M01', day: 1}).monthCode, 'M01', 'MonthDay before minimum full date');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: minYear - 1, monthCode: 'M12', day: firstYearDay}, {overflow: 'constrain'}), RangeError.prototype, 'preceding native year cannot be constrained into range');
  if (calendar === 'islamic-civil') {
    errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: minYear - 1, monthCode: 'M11', day: 30}), RangeError.prototype, 'whole preceding native year outside carrier');
  } else {
    errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: minYear - 1, monthCode: 'M12', day: 1}), RangeError.prototype, 'whole preceding native year outside carrier');
  }
  same(Temporal.PlainMonthDay.from({calendar, year: maxYear, monthCode: 'M06', day: 1}).monthCode, 'M06', 'MonthDay after maximum full date');
  same(Temporal.PlainMonthDay.from({calendar, year: maxYear, monthCode: 'M09', day: finalYearDay}).day, finalYearDay, 'MonthDay last requested ISO-year day');
  same(Temporal.PlainMonthDay.from({calendar, year: maxYear, monthCode: 'M09', day: finalYearDay + 1}).day, finalYearDay + 1, 'same native year admits chosen next ISO year');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: maxYear + 1, monthCode: 'M01', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'whole following native year outside carrier');
  same(Temporal.PlainMonthDay.from({calendar, era: 'bh', eraYear: 1 - minYear, monthCode: 'M01', day: 1}).monthCode, 'M01', 'era-resolved extreme year admission');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year: minYear - 1, era: 'bh', monthCode: 'M01', day: 1}), TypeError.prototype, 'incomplete era precedes native-year range');
  errorPrototype(() => Temporal.PlainMonthDay.from({calendar, era: 'ah', eraYear: maxYear + 1, monthCode: 'M12', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'era-resolved adjacent year rejects');
  for (const year of [-999999, 999999]) {
    errorPrototype(() => Temporal.PlainMonthDay.from({calendar, year, monthCode: 'M12', day: 30}), RangeError.prototype, 'MonthDay envelope before month information');
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
const marker = foreign.Object();
const pairs = families.map(([calendar]) => [
  localFrom({calendar, year: 1390, month: 12, day: 30}),
  foreignFrom({calendar, year: 1390, month: 12, day: 30})
]);
foreign.RangeError = foreign.TypeError = function wrongError() {throw 'mutable foreign errors';};
foreign.Temporal = {};
globalThis.RangeError = globalThis.TypeError = function wrongError() {throw 'mutable local errors';};
for (let index = 0; index < families.length; index++) {
  const [calendar, minDay, maxDay, firstMonth, finalMonth, firstYearDay, finalYearDay] = families[index];
  for (let direction = 0; direction < 2; direction++) {
    const from = direction === 0 ? foreignFrom : localFrom;
    const nestedFrom = direction === 0 ? localFrom : foreignFrom;
    const withMethod = direction === 0 ? foreignWith : localWith;
    const addMethod = direction === 0 ? foreignAdd : localAdd;
    const mdFrom = direction === 0 ? foreignMonthDayFrom : localMonthDayFrom;
    const receiver = pairs[index][direction];
    const resultPrototype = direction === 0 ? ForeignDate.prototype : LocalDate.prototype;
    const mdPrototype = direction === 0 ? ForeignMonthDay.prototype : LocalMonthDay.prototype;
    const rangePrototype = direction === 0 ? foreignRange : localRange;
    const typePrototype = direction === 0 ? foreignType : localType;
    const result = from({calendar, year: 1392, monthCode: 'M01', day: 1});
    same(Object.getPrototypeOf(result), resultPrototype, 'called Realm date result');
    same(result.withCalendar('iso8601').toString(), calendar === 'islamic-civil' ? '1972-02-16' : '1972-02-15', 'called Realm conversion');
    const md = mdFrom({calendar, year: 283583, monthCode: 'M06', day: 1});
    same(Object.getPrototypeOf(md), mdPrototype, 'called Realm MonthDay result');
    let reads = 0;
    errorPrototype(() => from({calendar, year: 1391, month: 12, get day() {
      reads++;
      nestedFrom({calendar, year: 1392, month: 1, day: 1});
      return 30;
    }}, {overflow: 'reject'}), rangePrototype, 'nested field RangeError Realm');
    same(reads, 1, 'nested field once');
    errorPrototype(() => from({calendar, year: 1392, month: 1, get day() {
      nestedFrom({calendar, year: 1392, month: 2, day: 1});
      return Symbol('day');
    }}), typePrototype, 'nested field TypeError Realm');
    errorPrototype(() => mdFrom({calendar, year: 283584, monthCode: 'M09', get day() {
      nestedFrom({calendar, year: 1392, month: 1, day: 1});
      return finalYearDay + 1;
    }}), rangePrototype, 'nested whole-native-year Realm');
    errorPrototype(() => withMethod.call(receiver, {year: 1391}, {overflow: 'reject'}), rangePrototype, 'borrowed with Realm');
    errorPrototype(() => addMethod.call(receiver, {years: 1}, {overflow: 'reject'}), rangePrototype, 'borrowed add Realm');
    let published = 'before';
    let finallyRuns = 0;
    try {
      try {
        const value = from({calendar, get day() {throw marker;}});
        published = value;
      } finally {finallyRuns++;}
      throw new Error('missing Realm field throw');
    } catch (caught) {if (caught !== marker) throw new Error('original foreign marker');}
    same(published, 'before', 'no Realm abrupt publication');
    same(finallyRuns, 1, 'Realm field finally');
  }
}
print('islamic-tabular-limits-and-realms:ok');
262;
