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
const minimum = new Temporal.PlainDate(-271821, 4, 19, 'persian');
const maximum = new Temporal.PlainDate(275760, 9, 13, 'persian');
for (const date of [minimum, maximum]) {
  same(Temporal.PlainDate.from({calendar: 'persian', year: date.year, monthCode: date.monthCode, day: date.day}, {overflow: 'reject'}).equals(date), true, 'full carrier round trip');
}
same(minimum.year, -272442, 'minimum year');
same(minimum.month, 1, 'minimum month');
same(minimum.day, 9, 'minimum day');
same(maximum.year, 275139, 'maximum year');
same(maximum.month, 7, 'maximum month');
same(maximum.day, 12, 'maximum day');
errorPrototype(() => minimum.subtract({days: 1}), RangeError.prototype, 'minimum range');
errorPrototype(() => maximum.add({days: 1}), RangeError.prototype, 'maximum range');
errorPrototype(() => Temporal.PlainDate.from({calendar: 'persian', year: 275139, month: 7, day: 13}), RangeError.prototype, 'converted full-date range');
// YearMonth limits use the reference ISO year/month, not the full-date day.
const firstMonth = Temporal.PlainYearMonth.from({calendar: 'persian', year: -272442, monthCode: 'M01'});
same(firstMonth.toString(), '-271821-04-11[u-ca=persian]', 'minimum month reference');
errorPrototype(() => firstMonth.toPlainDate({day: 1}), RangeError.prototype, 'minimum month to full date');
errorPrototype(() => Temporal.PlainYearMonth.from({calendar: 'persian', year: -272443, monthCode: 'M12'}), RangeError.prototype, 'preceding ISO month');
const lastMonth = Temporal.PlainYearMonth.from({calendar: 'persian', year: 275139, monthCode: 'M07'});
same(lastMonth.toString(), '+275760-09-02[u-ca=persian]', 'maximum month reference');
same(lastMonth.toPlainDate({day: 12}).equals(maximum), true, 'maximum month valid day');
errorPrototype(() => lastMonth.toPlainDate({day: 13}), RangeError.prototype, 'maximum month invalid day');
errorPrototype(() => Temporal.PlainYearMonth.from({calendar: 'persian', year: 275139, monthCode: 'M08'}), RangeError.prototype, 'following ISO month');
// Supplied native-year intervals intersect the carrier independently of the
// chosen month/day, including dates in the next ISO year.
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'persian', year: -272443, monthCode: 'M09', day: 21}, {overflow: 'constrain'}), RangeError.prototype, 'preceding native year cannot be constrained into range');
same(Temporal.PlainMonthDay.from({calendar: 'persian', year: -272442, monthCode: 'M01', day: 1}).toString(), '1972-03-21[u-ca=persian]', 'MonthDay outside minimum full date');
same(Temporal.PlainMonthDay.from({calendar: 'persian', year: 275139, monthCode: 'M07', day: 13}).toString(), '1972-10-05[u-ca=persian]', 'MonthDay outside maximum full date');
same(Temporal.PlainMonthDay.from({calendar: 'persian', year: 275139, monthCode: 'M11', day: 1}).toString(), '1972-01-21[u-ca=persian]', 'MonthDay last requested ISO-year day');
same(Temporal.PlainMonthDay.from({calendar: 'persian', year: 275139, monthCode: 'M11', day: 2}).day, 2, 'same native year admits chosen next ISO year');
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'persian', year: -272443, monthCode: 'M09', day: 20}), RangeError.prototype, 'whole preceding native year outside carrier');
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'persian', year: 275140, monthCode: 'M01', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'whole following native year outside carrier');
same(Temporal.PlainMonthDay.from({calendar: 'persian', era: 'ap', eraYear: -272442, monthCode: 'M01', day: 1}).monthCode, 'M01', 'era-resolved extreme year admission');
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'persian', year: -272443, era: 'ap', monthCode: 'M01', day: 1}), TypeError.prototype, 'incomplete era precedes native-year range');
errorPrototype(() => Temporal.PlainMonthDay.from({calendar: 'persian', era: 'ap', eraYear: 275140, monthCode: 'M12', day: 1}, {overflow: 'constrain'}), RangeError.prototype, 'era-resolved adjacent year rejects');
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
const localReceiver = localFrom({calendar: 'persian', year: 1403, month: 12, day: 30});
const foreignReceiver = foreignFrom({calendar: 'persian', year: 1403, month: 12, day: 30});
foreign.RangeError = foreign.TypeError = function wrongError() {throw 'mutable error constructor';};
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
  const result = from({calendar: 'persian', year: 1400, month: 2, day: 1});
  same(Object.getPrototypeOf(result), resultPrototype, 'called Realm result');
  same(result.withCalendar('iso8601').toString(), '2021-04-21', 'called Realm conversion');
  const md = monthDayFrom({calendar: 'persian', year: 275139, monthCode: 'M07', day: 13});
  same(Object.getPrototypeOf(md), monthDayPrototype, 'called Realm MonthDay result');
  same(md.toString(), '1972-10-05[u-ca=persian]', 'called Realm MonthDay reference');
  errorPrototype(() => monthDayFrom({calendar: 'persian', year: 275140, monthCode: 'M11', get day() {
    nestedFrom({calendar: 'persian', year: 1403, month: 1, day: 1});
    return 2;
  }}), rangePrototype, 'nested whole-native-year error Realm');
  let reads = 0;
  errorPrototype(() => from({calendar: 'persian', year: 1404, month: 12, get day() {
    reads++;
    nestedFrom({calendar: 'persian', year: 1403, month: 2, day: 1});
    return 30;
  }}, {overflow: 'reject'}), rangePrototype, 'nested field RangeError Realm');
  same(reads, 1, 'nested field read once');
  errorPrototype(() => from({calendar: 'persian', year: 1403, month: 1, get day() {
    nestedFrom({calendar: 'persian', year: 1403, month: 2, day: 1});
    return Symbol('invalid day');
  }}), typePrototype, 'nested field TypeError Realm');
  errorPrototype(() => withMethod.call(receiver, {year: 1404}, {overflow: 'reject'}), rangePrototype, 'borrowed with error Realm');
  errorPrototype(() => addMethod.call(receiver, {years: 1}, {overflow: 'reject'}), rangePrototype, 'borrowed add error Realm');
  const marker = {};
  try {
    from({calendar: 'persian', get day() {throw marker;}});
    throw new Error('missing original Realm field throw');
  } catch (error) {
    if (error !== marker) throw new Error('Realm field identity');
  }
}
print('persian-limits-and-realms:ok');
262;
