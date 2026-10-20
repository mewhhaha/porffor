function same(actual, expected, message) {
  if (actual !== expected) throw new Error(message);
}
function range(action, message) {
  let caught;
  try { action(); } catch (error) { caught = error; }
  if (!(caught instanceof RangeError)) throw new Error(message);
}

const relativeTo = '2020-02-29T23:59:59.999999999+05:30[+05:30]';
const nanosecond = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 1);
same(nanosecond.total({unit: 'nanosecond', relativeTo}), 1, 'midnight nanosecond');
same(nanosecond.round({smallestUnit: 'nanosecond', relativeTo}).nanoseconds,
  1, 'round keeps nanosecond');
same(Temporal.Duration.compare(nanosecond, new Temporal.Duration(), {relativeTo}),
  1, 'compare nanosecond');

const wide = new Temporal.Duration(0, 0, 0, 0, 4000, 0, 0, 0, 0, 1);
same(wide.total({unit: 'hour', relativeTo: '2020-01-01'}),
  4000.0000000000005, 'one final rounding for wide total');
same(wide.negated().total({unit: 'hour', relativeTo: '2020-01-01'}),
  -4000.0000000000005, 'negative exact total');

const maximum = new Temporal.ZonedDateTime(8640000000000000000000n, 'UTC');
const minimum = new Temporal.ZonedDateTime(-8640000000000000000000n, 'UTC');
range(() => nanosecond.total({unit: 'nanosecond', relativeTo: maximum}),
  'total rejects upper target');
range(() => nanosecond.round({smallestUnit: 'nanosecond', relativeTo: maximum}),
  'round rejects upper target');
range(() => nanosecond.negated().total({unit: 'nanosecond', relativeTo: minimum}),
  'total rejects lower target');
range(() => Temporal.Duration.compare(new Temporal.Duration(1), new Temporal.Duration(0, 0, 0, 365),
  {relativeTo: maximum}), 'compare validates calendar target');
same(Temporal.Duration.compare(new Temporal.Duration(1), new Temporal.Duration(1),
  {relativeTo: maximum}), 0, 'equal fields compare before target range');

true;
