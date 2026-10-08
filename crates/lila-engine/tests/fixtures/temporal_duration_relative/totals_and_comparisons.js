function same(actual, expected, message) {
  if (actual !== expected) throw new Error(message);
}

const month = new Temporal.Duration(0, 1);
same(month.total({unit: 'day', relativeTo: '2020-01-31'}), 29, 'leap month');
same(month.total({unit: 'day', relativeTo: '2019-01-31'}), 28, 'ordinary month');
same(new Temporal.Duration(0, -1).total({unit: 'day', relativeTo: '2020-03-31'}),
  -31, 'negative constrained month');
same(new Temporal.Duration(1).total({unit: 'day', relativeTo: '2020-01-01'}),
  366, 'leap year');

// Calendar annotations are not time-zone annotations. The plain relative
// date may include a time and offset, which do not change its date slots.
same(month.total({unit: 'day', relativeTo: '2020-01-31[u-ca=iso8601]'}),
  29, 'calendar-only annotation');
same(month.total({unit: 'day', relativeTo: '2020-01-31[!u-ca=iso8601]'}),
  29, 'critical calendar annotation');
same(month.total({unit: 'day', relativeTo: '2020-01-31T23:45-03:00[u-ca=iso8601]'}),
  29, 'plain offset and calendar');
same(month.total({unit: 'day', relativeTo: '2020-01-31T12:00+05:30[+05:30][u-ca=iso8601]'}),
  29, 'zone plus calendar');
same(month.total({unit: 'day', relativeTo: new Temporal.PlainDateTime(2020, 1, 31, 23, 59)}),
  29, 'datetime relative uses date slots');

same(Temporal.Duration.compare(month, new Temporal.Duration(0, 0, 0, 29),
  {relativeTo: '2020-01-31[u-ca=iso8601]'}), 0, 'equal constrained month');
same(Temporal.Duration.compare(month, new Temporal.Duration(0, 0, 0, 29),
  {relativeTo: '2019-01-31'}), -1, 'shorter month');
same(Temporal.Duration.compare(new Temporal.Duration(1), new Temporal.Duration(0, 0, 0, 365),
  {relativeTo: '2020-01-01T12:00-03:00[-03:00]'}), 1, 'fixed-zone leap year');
same(month.round({largestUnit: 'day', smallestUnit: 'day',
  relativeTo: '2020-01-31[u-ca=iso8601]'}).days,
  29, 'round accepts calendar annotation');

true;
