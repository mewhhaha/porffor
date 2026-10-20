// Finite semantic controls for the atomic Temporal GC source. Authored, unrun.
function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { check(Object.is(actual, expected), label); }
function abrupt(operation, expected, label) {
  var caught, threw = false;
  try { operation(); } catch (error) { caught = error; threw = true; }
  check(threw && caught === expected, label);
}
var marker = { retained: Symbol('record poison') };
function poison(record, names) {
  for (var name of names) {
    Object.defineProperty(record, name, {get() { throw marker; }, configurable: true});
  }
}
var date = new Temporal.PlainDate(2000, 2, 29);
var dateTime = new Temporal.PlainDateTime(2000, 2, 29, 12, 34, 56, 123, 456, 789);
var yearMonth = new Temporal.PlainYearMonth(2000, 2);
var monthDay = new Temporal.PlainMonthDay(2, 29);
poison(date, ['calendar', 'calendarId', 'day', 'month', 'monthCode', 'year']);
poison(dateTime, ['calendar', 'calendarId', 'day', 'hour', 'microsecond', 'millisecond', 'minute', 'month', 'monthCode', 'nanosecond', 'second', 'year']);
poison(yearMonth, ['calendar', 'calendarId', 'month', 'monthCode', 'year']);
poison(monthDay, ['calendar', 'calendarId', 'day', 'monthCode']);
gc();
same(Temporal.PlainDate.from(date).toString(), '2000-02-29', 'PlainDate private slots');
same(Temporal.PlainDate.from(dateTime).toString(), '2000-02-29', 'DateTime to Date private slots');
same(Temporal.PlainDateTime.from(dateTime).toString(), '2000-02-29T12:34:56.123456789', 'DateTime full private slots');
same(Temporal.PlainDateTime.from(date).toString(), '2000-02-29T00:00:00', 'Date to DateTime midnight');
same(Temporal.PlainYearMonth.from(yearMonth).toString(), '2000-02', 'YearMonth private slots');
same(Temporal.PlainMonthDay.from(monthDay).toString(), '02-29', 'MonthDay private slots');

var names = ['days', 'hours', 'microseconds', 'milliseconds', 'minutes', 'months', 'nanoseconds', 'seconds', 'weeks', 'years'];
var trace = '';
function durationBag(prefix, days, hours) {
  var bag = {};
  for (let name of names) {
    Object.defineProperty(bag, name, {get() {
      trace += prefix + name + '.get;';
      return {valueOf() {
        trace += prefix + name + '.number;';
        return name === 'days' ? days : name === 'hours' ? hours : 0;
      }};
    }});
  }
  return bag;
}
function durationTrace(prefix) {
  var result = '';
  for (var name of names) result += prefix + name + '.get;' + prefix + name + '.number;';
  return result;
}
var duration = Temporal.Duration.from(durationBag('from.', 1, 2));
same(trace, durationTrace('from.'), 'Duration.from complete alphabetical acquisition');
same(duration.toString(), 'P1DT2H', 'Duration Number fields retain their values');
trace = '';
var changed = duration.with({get days() { trace += 'days;'; return 3; }, get hours() { trace += 'hours;'; return undefined; }});
same(trace, 'days;hours;', 'Duration.with gets before merging absent fields');
check(changed.days === 3 && changed.hours === 2, 'Duration.with retains omitted Number fields');
poison(duration, names);
gc();
same(Temporal.Duration.from(duration).toString(), 'P1DT2H', 'Duration branded conversion ignores public getters');
trace = '';
same(Temporal.Duration.compare(durationBag('left.', 0, 1), durationBag('right.', 0, 2), {
  get relativeTo() { trace += 'relativeTo;'; return undefined; }
}), -1, 'Duration.compare actual operands');
same(trace, durationTrace('left.') + durationTrace('right.') + 'relativeTo;', 'Both duration conversions precede compare options');
trace = '';
var rounded = new Temporal.Duration(0, 0, 0, 0, 1, 2, 3).round({
  get largestUnit() { trace += 'largestUnit;'; return 'hour'; },
  get relativeTo() { trace += 'relativeTo;'; return undefined; },
  get roundingIncrement() { trace += 'roundingIncrement;'; return 1; },
  get roundingMode() { trace += 'roundingMode;'; return 'floor'; },
  get smallestUnit() { trace += 'smallestUnit;'; return 'minute'; }
});
same(trace, 'largestUnit;relativeTo;roundingIncrement;roundingMode;smallestUnit;', 'Duration.round options order');
check(rounded.hours === 1 && rounded.minutes === 2 && rounded.seconds === 0, 'Duration.round canonical fields');
trace = '';
same(new Temporal.Duration(0, 0, 0, 0, 1, 30).total({
  get relativeTo() { trace += 'relativeTo;'; return undefined; },
  get unit() { trace += 'unit;'; return 'minute'; }
}), 90, 'Duration.total complete Number');
same(trace, 'relativeTo;unit;', 'Duration.total options order');
abrupt(function() { Temporal.Duration.from({days: 1e100, get hours() { throw marker; }}); }, marker, 'Later getter abrupt precedes deferred range validation');
abrupt(function() { duration.with({get microseconds() { throw marker; }}); }, marker, 'Duration.with preserves original object throw');
abrupt(function() { duration.round({get largestUnit() { throw marker; }}); }, marker, 'Duration.round preserves original object throw');
abrupt(function() { duration.total({get relativeTo() { throw marker; }}); }, marker, 'Duration.total preserves original object throw');
print('temporal-gc:ok');
262;
