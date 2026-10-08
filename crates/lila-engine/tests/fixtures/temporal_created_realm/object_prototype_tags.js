var rows = [
  ['Instant', [1234n], 'epochNanoseconds', 1234n],
  ['ZonedDateTime', [1234n, 'UTC'], 'epochNanoseconds', 1234n],
  ['PlainDate', [2000, 5, 2], 'year', 2000],
  ['PlainTime', [3, 4, 5], 'hour', 3],
  ['PlainDateTime', [2000, 5, 2, 3, 4, 5], 'year', 2000],
  ['PlainMonthDay', [5, 2], 'monthCode', 'M05'],
  ['PlainYearMonth', [2000, 5], 'year', 2000],
  ['Duration', [0, 0, 0, 2], 'days', 2]
];
var current;
var functionPrototype = function inheritedFunction() {};
var arrayPrototype = ['array index'];
var proxyReads = 0;
var proxyPrototype = new Proxy({}, {
  get: function(target, key, receiver) {
    if (key === 'inherited') {
      proxyReads++;
      if (receiver !== current) throw 'Proxy inherited receiver';
      return 'Proxy getter';
    }
    return Reflect.get(target, key, receiver);
  }
});
for (var prototype of [functionPrototype, arrayPrototype]) {
  Object.defineProperty(prototype, 'inherited', {
    get: function() {
      if (this !== current) throw 'inherited accessor receiver';
      return 'ordinary getter';
    }, configurable: true
  });
}
for (var row of rows) {
  var constructor = Temporal[row[0]];
  var getter = Object.getOwnPropertyDescriptor(constructor.prototype, row[2]).get;
  for (var prototype of [functionPrototype, arrayPrototype, proxyPrototype]) {
    function NewTarget() { throw 'NewTarget must not be invoked'; }
    NewTarget.prototype = prototype;
    current = Reflect.construct(constructor, row[1], NewTarget);
    if (Object.getPrototypeOf(current) !== prototype) throw row[0] + ' object prototype identity';
    if (getter.call(current) !== row[3]) throw row[0] + ' object prototype keeps Temporal brand';
    if (current.inherited !== (prototype === proxyPrototype ? 'Proxy getter' : 'ordinary getter')) throw row[0] + ' inherited access';
    if (prototype === arrayPrototype && current[0] !== 'array index') throw row[0] + ' Array prototype indexed access';
  }
}
if (proxyReads !== rows.length) throw 'one inherited Proxy read per instance';
print('ok');
262;
