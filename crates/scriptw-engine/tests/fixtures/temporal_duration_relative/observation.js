function same(actual, expected, message) {
  if (actual !== expected) throw new Error(message);
}

const month = new Temporal.Duration(0, 1);
const date = new Temporal.PlainDate(2020, 1, 31);
const marker = {};
Object.defineProperty(date, 'year', {get() { throw marker; }});
Object.defineProperty(date, 'calendar', {get() { throw marker; }});
Object.defineProperty(date, 'overflow', {get() { throw marker; }});
let trace = '';
const rounded = month.round({
  get largestUnit() { trace += 'L'; return 'month'; },
  get relativeTo() { trace += 'R'; return date; },
  get roundingIncrement() { trace += 'I'; return 1; },
  get roundingMode() { trace += 'M'; return 'trunc'; },
  get smallestUnit() { trace += 'S'; return 'day'; }
});
same(trace, 'LRIMS', 'round option order');
// ISODateSurpasses compares the original day before constraining it:
// https://tc39.es/proposal-temporal/#sec-temporal-isodatesurpasses
same(rounded.months, 0, 'end-of-month difference keeps no whole month');
same(rounded.days, 29, 'branded relative slots');

trace = '';
same(month.total({
  get relativeTo() { trace += 'R'; return date; },
  get unit() { trace += 'U'; return 'day'; }
}), 29, 'total value');
same(trace, 'RU', 'total option order');

let laterReads = 0;
let caught;
try {
  month.round({largestUnit: 'month',
    get relativeTo() { throw marker; },
    get roundingIncrement() { laterReads++; return 1; }});
} catch (error) { caught = error; }
same(caught, marker, 'relative getter abrupt identity');
same(laterReads, 0, 'round stops after abrupt relative read');

caught = undefined;
try {
  month.total({relativeTo: {get year() { throw marker; }},
    get unit() { laterReads++; return 'day'; }});
} catch (error) { caught = error; }
same(caught, marker, 'relative bag abrupt identity');
same(laterReads, 0, 'total converts relative before unit');

trace = '';
const bag = {
  get calendar() { trace += 'c'; return 'iso8601'; },
  get day() { trace += 'd'; return 29; },
  get month() { trace += 'm'; return 2; },
  get monthCode() { trace += 'k'; return 'M02'; },
  get year() { trace += 'y'; return 2024; }
};
const converted = Temporal.PlainDate.from(bag, {
  get overflow() { trace += 'o'; return 'reject'; }
});
same(trace, 'cdmkyo', 'shared PlainDate field acquisition precedes overflow');
same(converted.year, 2024, 'converted year');
same(converted.month, 2, 'converted month');
same(converted.day, 29, 'converted day');

trace = '';
caught = undefined;
try {
  Temporal.PlainDate.from({
    get day() { trace += 'd'; return 1; },
    get month() { trace += 'm'; return 1; },
    get monthCode() { trace += 'k'; return 'invalid'; },
    get year() { trace += 'y'; return 2024; }
  }, {get overflow() { trace += 'o'; return 'reject'; }});
} catch (error) { caught = error; }
same(caught instanceof RangeError, true, 'monthCode syntax failure');
same(trace, 'dmk', 'monthCode rejection stops before year and overflow');

let overflowReads = 0;
const copied = Temporal.PlainDate.from(date, {
  get overflow() { overflowReads++; return undefined; }
});
same(overflowReads, 1, 'present overflow options remain observable on branded input');
same(copied === date, false, 'PlainDate.from still returns its own fresh result');
same(Temporal.PlainDate.compare(date, copied), 0, 'Omit uses slots without input overflow reads');

true;
