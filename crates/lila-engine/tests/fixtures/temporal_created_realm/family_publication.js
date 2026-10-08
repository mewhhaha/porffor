var foreign = __lilaCreateRealm().global;
var rows = [
  ['Instant', [1234n], 'epochNanoseconds', 1234n, 1],
  ['ZonedDateTime', [1234n, 'UTC'], 'epochNanoseconds', 1234n, 2],
  ['PlainDate', [2000, 5, 2], 'year', 2000, 3],
  ['PlainTime', [3, 4, 5], 'hour', 3, 0],
  ['PlainDateTime', [2000, 5, 2, 3, 4, 5], 'year', 2000, 3],
  ['PlainMonthDay', [5, 2], 'monthCode', 'M05', 2],
  ['PlainYearMonth', [2000, 5], 'year', 2000, 2],
  ['Duration', [0, 0, 0, 2], 'days', 2, 0]
];
if (Object.getPrototypeOf(foreign.Temporal) !== foreign.Object.prototype) throw 'Temporal namespace Realm';
if (Object.prototype.toString.call(foreign.Temporal) !== '[object Temporal]') throw 'Temporal namespace tag';
for (var row of rows) {
  var name = row[0], constructor = foreign.Temporal[name];
  var member = Object.getOwnPropertyDescriptor(foreign.Temporal, name);
  if (!member || member.value !== constructor || !member.writable || member.enumerable || !member.configurable) throw name + ' namespace descriptor';
  if (typeof constructor !== 'function' || constructor === Temporal[name] || constructor.name !== name || constructor.length !== row[4]) throw name + ' constructor';
  if (Object.getPrototypeOf(constructor) !== foreign.Function.prototype) throw name + ' function Realm';
  var prototype = constructor.prototype;
  if (prototype === Temporal[name].prototype || Object.getPrototypeOf(prototype) !== foreign.Object.prototype) throw name + ' prototype Realm';
  var prototypeProperty = Object.getOwnPropertyDescriptor(constructor, 'prototype');
  if (prototypeProperty.writable || prototypeProperty.enumerable || prototypeProperty.configurable) throw name + ' prototype descriptor';
  var constructorProperty = Object.getOwnPropertyDescriptor(prototype, 'constructor');
  if (constructorProperty.value !== constructor || !constructorProperty.writable || constructorProperty.enumerable || !constructorProperty.configurable) throw name + ' constructor backlink';
  var receiver = Reflect.construct(constructor, row[1]);
  if (Object.getPrototypeOf(receiver) !== prototype || receiver[row[2]] !== row[3]) throw name + ' constructor result';
  for (var home of [Temporal[name], constructor]) {
    var expectedFunctionPrototype = home === constructor ? foreign.Function.prototype : Function.prototype;
    for (var property of Object.getOwnPropertyNames(home.prototype)) {
      var descriptor = Object.getOwnPropertyDescriptor(home.prototype, property);
      if (descriptor.get !== undefined) {
        if (typeof descriptor.get !== 'function' || descriptor.set !== undefined || descriptor.enumerable || !descriptor.configurable) throw name + ' published accessor descriptor: ' + property;
        if (descriptor.get.name !== 'get ' + property || descriptor.get.length !== 0 || descriptor.get.hasOwnProperty('prototype') || Object.getPrototypeOf(descriptor.get) !== expectedFunctionPrototype) throw name + ' published accessor metadata: ' + property;
      }
    }
  }
  var getter = Object.getOwnPropertyDescriptor(prototype, row[2]);
  if (typeof getter.get !== 'function' || getter.set !== undefined || getter.enumerable || !getter.configurable) throw name + ' accessor descriptor';
  if (getter.get.name !== 'get ' + row[2] || getter.get.length !== 0 || Object.getPrototypeOf(getter.get) !== foreign.Function.prototype) throw name + ' accessor metadata';
  if (getter.get.call(Reflect.construct(Temporal[name], row[1])) !== row[3]) throw name + ' borrowed accessor';
  var caught = undefined;
  try { getter.get.call(prototype); } catch (error) { caught = error; }
  if (!caught || Object.getPrototypeOf(caught) !== foreign.TypeError.prototype) throw name + ' prototype must not have instance brand';
  var method = Object.getOwnPropertyDescriptor(prototype, 'toString');
  if (!method || typeof method.value !== 'function' || !method.writable || method.enumerable || !method.configurable) throw name + ' method descriptor';
  if (method.value.name !== 'toString' || method.value.hasOwnProperty('prototype') || Object.getPrototypeOf(method.value) !== foreign.Function.prototype) throw name + ' method metadata';
}
print('ok');
262;
