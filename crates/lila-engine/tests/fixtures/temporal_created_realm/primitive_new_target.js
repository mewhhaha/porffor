var foreign = __lilaCreateRealm().global;
var rows = [
  ['Instant', [1234n], 'epochNanoseconds', 1234n],
  ['ZonedDateTime', [1234n, '+01:00'], 'epochNanoseconds', 1234n],
  ['PlainDate', [2000, 5, 2], 'year', 2000],
  ['PlainTime', [3, 4, 5], 'hour', 3],
  ['PlainDateTime', [2000, 5, 2, 3, 4, 5], 'year', 2000],
  ['PlainMonthDay', [5, 2], 'monthCode', 'M05'],
  ['PlainYearMonth', [2000, 5], 'year', 2000],
  ['Duration', [0, 0, 0, 2], 'days', 2]
];
for (var row of rows) {
  // Array is never invoked; its bound constructor contributes its real Realm.
  var NewTarget = foreign.Array.bind(null);
  Object.defineProperty(NewTarget, 'prototype', {value: 42, configurable: true});
  var result = Reflect.construct(Temporal[row[0]], row[1], NewTarget);
  if (Object.getPrototypeOf(result) !== foreign.Temporal[row[0]].prototype) throw row[0] + ' primitive default Realm';
  if (result[row[2]] !== row[3]) throw row[0] + ' default retains slots';
  var UndefinedTarget = foreign.Array.bind(null);
  var defaultResult = Reflect.construct(Temporal[row[0]], row[1], UndefinedTarget);
  if (Object.getPrototypeOf(defaultResult) !== foreign.Temporal[row[0]].prototype || defaultResult[row[2]] !== row[3]) throw row[0] + ' absent prototype default';
}
print('ok');
262;
