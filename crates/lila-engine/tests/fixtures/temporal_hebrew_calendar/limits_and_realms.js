function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function errorPrototype(action,prototype,label) {
  try {action();} catch(caught) {same(Object.getPrototypeOf(caught),prototype,label);return;}
  throw new Error('missing '+label);
}
const calendar = 'hebrew';
// Literal pinned full-date and YearMonth extreme rows; distinct carrier limits.
const minimum = new Temporal.PlainDate(-271821,4,19,calendar);
const maximum = new Temporal.PlainDate(275760,9,13,calendar);
for (const [date,year,month,code,day] of [[minimum,-268058,11,'M11',4],[maximum,279517,10,'M09',11]]) {
  same(date.year,year,'literal native extreme year');
  same(date.month,month,'literal native extreme ordinal');
  same(date.monthCode,code,'literal native extreme code');
  same(date.day,day,'literal native extreme day');
  same(date.era,'am','signed extreme AM');
  same(date.eraYear,year,'signed extreme era year');
  same(Temporal.PlainDate.from({calendar,year,monthCode:code,day},{overflow:'reject'}).equals(date),true,'full extreme round trip');
}
errorPrototype(()=>minimum.subtract({days:1}),RangeError.prototype,'minimum full carrier');
errorPrototype(()=>maximum.add({days:1}),RangeError.prototype,'maximum full carrier');
errorPrototype(()=>minimum.toPlainYearMonth(),RangeError.prototype,'minimum forbidden partial month');
const first = Temporal.PlainYearMonth.from({calendar,year:-268058,monthCode:'M12'});
same(first.month,12,'first native YearMonth');
same(first.toString(),'-271821-05-16[u-ca=hebrew]','first allowed partial reference');
const last = Temporal.PlainYearMonth.from({calendar,year:279517,monthCode:'M09'});
same(last.month,10,'final native YearMonth');
same(last.toString(),'+275760-09-03[u-ca=hebrew]','last allowed partial reference');
errorPrototype(()=>Temporal.PlainYearMonth.from({calendar,year:279517,monthCode:'M10'}),RangeError.prototype,'following partial ISO month');
// MonthDay admits a native year whose whole interval intersects the ISO
// carrier, independently of the chosen month's converted year or full date.
same(Temporal.PlainMonthDay.from({calendar,year:-268058,monthCode:'M01',day:1}).monthCode,'M01','admitted native year despite chosen prior ISO year');
errorPrototype(()=>Temporal.PlainMonthDay.from({calendar,year:-268059,monthCode:'M01',day:1}),RangeError.prototype,'prior native year has no carrier intersection');
errorPrototype(()=>Temporal.PlainMonthDay.from({calendar,year:279518,monthCode:'M01',day:1}),RangeError.prototype,'following native year has no carrier intersection');
same(Temporal.PlainMonthDay.from({calendar,year:-268058,monthCode:'M11',day:1}).day,1,'before minimum full date within ISO year');
same(Temporal.PlainMonthDay.from({calendar,year:279517,monthCode:'M12',day:29}).day,29,'after maximum full date within ISO year');
for (const year of [-999999,999999]) {
  errorPrototype(()=>Temporal.PlainMonthDay.from({calendar,year,monthCode:'M05L',day:30}),RangeError.prototype,'native envelope before month information');
}
const foreign = __lilaCreateRealm().global;
const LocalDate = Temporal.PlainDate;
const ForeignDate = foreign.Temporal.PlainDate;
const LocalMonthDay = Temporal.PlainMonthDay;
const ForeignMonthDay = foreign.Temporal.PlainMonthDay;
const localFrom = LocalDate.from;
const foreignFrom = ForeignDate.from;
const localMDFrom = LocalMonthDay.from;
const foreignMDFrom = ForeignMonthDay.from;
const localWith = LocalDate.prototype.with;
const foreignWith = ForeignDate.prototype.with;
const localAdd = LocalDate.prototype.add;
const foreignAdd = ForeignDate.prototype.add;
const localRange = RangeError.prototype;
const foreignRange = foreign.RangeError.prototype;
const localType = TypeError.prototype;
const foreignType = foreign.TypeError.prototype;
const marker = foreign.Object();
const receivers = [localFrom({calendar,year:5784,monthCode:'M05L',day:30}),foreignFrom({calendar,year:5784,monthCode:'M05L',day:30})];
foreign.RangeError = foreign.TypeError = function wrongError(){throw 'mutable foreign errors';};
foreign.Temporal = {};
globalThis.RangeError = globalThis.TypeError = function wrongError(){throw 'mutable local errors';};
for (let direction=0;direction<2;direction++) {
  const from = direction===0 ? foreignFrom : localFrom;
  const nestedFrom = direction===0 ? localFrom : foreignFrom;
  const mdFrom = direction===0 ? foreignMDFrom : localMDFrom;
  const withMethod = direction===0 ? foreignWith : localWith;
  const addMethod = direction===0 ? foreignAdd : localAdd;
  const receiver = receivers[direction];
  const resultPrototype = direction===0 ? ForeignDate.prototype : LocalDate.prototype;
  const mdPrototype = direction===0 ? ForeignMonthDay.prototype : LocalMonthDay.prototype;
  const rangePrototype = direction===0 ? foreignRange : localRange;
  const typePrototype = direction===0 ? foreignType : localType;
  const result = from({calendar,year:5760,monthCode:'M04',day:23});
  same(Object.getPrototypeOf(result),resultPrototype,'called Realm result');
  same(result.withCalendar('iso8601').toString(),'2000-01-01','called Realm literal conversion');
  const md = mdFrom({calendar,monthCode:'M05L',day:30});
  same(Object.getPrototypeOf(md),mdPrototype,'called Realm partial result');
  let reads = 0;
  errorPrototype(()=>from({calendar,year:5783,monthCode:'M05L',get day(){reads++;nestedFrom({calendar,year:5784,monthCode:'M01',day:1});return 1;}},{overflow:'reject'}),rangePrototype,'nested original-code reject Realm');
  same(reads,1,'nested field once');
  errorPrototype(()=>from({calendar,year:5784,monthCode:'M05L',get day(){nestedFrom({calendar,year:5784,monthCode:'M01',day:1});return Symbol('day');}}),typePrototype,'nested numeric TypeError Realm');
  errorPrototype(()=>mdFrom({calendar,year:5783,monthCode:'M05L',day:30},{overflow:'reject'}),rangePrototype,'MonthDay original-code Realm');
  errorPrototype(()=>withMethod.call(receiver,{year:5785},{overflow:'reject'}),rangePrototype,'borrowed with error Realm');
  errorPrototype(()=>addMethod.call(receiver,{years:1},{overflow:'reject'}),rangePrototype,'borrowed add error Realm');
  let published='before';
  let finalizers=0;
  try {
    try {published=from({calendar,get day(){throw marker;}});} finally {finalizers++;}
    throw new Error('missing foreign hook throw');
  } catch(caught) {if(caught!==marker)throw new Error('foreign abrupt identity');}
  same(published,'before','no abrupt publication');
  same(finalizers,1,'abrupt finally');
}
print('hebrew-limits-and-realms:ok');
262;
