function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function throwsKind(action, kind) {
  try {action();} catch (caught) {if (!(caught instanceof kind)) throw caught; return;}
  throw new Error('missing expected error');
}
const calendar = 'hebrew';
const make = (monthCode, day, year) => Temporal.PlainMonthDay.from({calendar, monthCode, day, year});
function check(md, code, day, referenceYear) {
  same(md.calendarId, calendar, 'MonthDay calendar');
  same(md.monthCode, code, 'MonthDay canonical code');
  same(md.day, day, 'MonthDay regulated day');
  same(Number(md.toString().slice(0,4)), referenceYear, 'latest full ISO reference');
  same(Temporal.PlainMonthDay.from(md.toString()).equals(md), true, 'reference string round trip');
}
// Pinned latest references cover every code, variable long months and Adar I.
const codes = ['M01','M02','M03','M04','M05','M06','M07','M08','M09','M10','M11','M12'];
for (const code of codes) {
  check(make(code,1), code,1,1972);
  check(make(code,29), code,29,1972);
  const long = ['M01','M02','M03','M05','M07','M09','M11'].includes(code);
  check(make(code,30), code,long ? 30 : 29, long && (code === 'M02' || code === 'M03') ? 1971 : 1972);
  if (!long) throwsKind(() => Temporal.PlainMonthDay.from({calendar,monthCode:code,day:30},{overflow:'reject'}), RangeError);
  throwsKind(() => Temporal.PlainMonthDay.from({calendar,monthCode:code,day:31},{overflow:'reject'}), RangeError);
}
for (const day of [1,29,30]) check(make('M05L',day), 'M05L',day,1970);
check(make('M05L',31), 'M05L',30,1970);
throwsKind(() => Temporal.PlainMonthDay.from({calendar,monthCode:'M05L',day:31},{overflow:'reject'}), RangeError);
same(make('M04',26).toString(), '1972-12-31[u-ca=hebrew]', 'later candidate in same ISO year');
check(make('M02',30,5781), 'M02',29,1972);
check(make('M02',30), 'M02',30,1971);
throwsKind(() => Temporal.PlainMonthDay.from({calendar,year:5781,monthCode:'M02',day:30},{overflow:'reject'}), RangeError);
// Supplied-year regulation publishes the constrained code; yearless Adar I
// retains its leap code and seeks a suitable reference year.
check(make('M05L',30,5783), 'M06',29,1972);
throwsKind(() => Temporal.PlainMonthDay.from({calendar,year:5783,monthCode:'M05L',day:30},{overflow:'reject'}), RangeError);
check(make('M05L',30), 'M05L',30,1970);
const supplied = Temporal.PlainMonthDay.from({calendar,year:5784,month:7,monthCode:'M06',day:1});
check(supplied,'M06',1,1972);
same(supplied.toPlainDate({year:5784}).month,7,'reference code re-resolves in leap argument year');
same(supplied.toPlainDate({year:5783}).month,6,'reference code re-resolves in common argument year');
same(make('M12',1).toPlainDate({year:5784}).month,13,'last regular code shifts in leap year');
same(make('M05L',30).toPlainDate({year:5783}).monthCode,'M06','toPlainDate constrains missing leap');
same(make('M05L',30).toPlainDate({year:5783}).day,29,'toPlainDate regulates target day');
check(supplied.with({day:29}),'M06',29,1972);
for (const code of ['M13','M06L','M04L']) throwsKind(() => make(code,1),RangeError);
throwsKind(() => Temporal.PlainMonthDay.from({calendar,month:5,monthCode:'M04',day:1}),TypeError);
throwsKind(() => Temporal.PlainMonthDay.from({calendar,year:5784,month:5,monthCode:'M04'}),TypeError);
throwsKind(() => Temporal.PlainMonthDay.from({calendar,year:5784,month:5,monthCode:'M04',day:1}),RangeError);
// Shared resolver must keep the already-supported yearless ISO agreement.
throwsKind(() => Temporal.PlainMonthDay.from({month:11,monthCode:'M12',day:18}),RangeError);
same(Temporal.PlainMonthDay.from({month:12,monthCode:'M12',day:18}).monthCode,'M12','ISO yearless agreement');
const month = Temporal.PlainYearMonth.from({calendar,year:5784,monthCode:'M05L'});
same(month.toPlainDate({day:30}).monthCode,'M05L','completed YearMonth carrier');
same(month.toPlainDate({day:30}).day,30,'YearMonth native long month');
print('hebrew-partial-dates:ok');
262;
