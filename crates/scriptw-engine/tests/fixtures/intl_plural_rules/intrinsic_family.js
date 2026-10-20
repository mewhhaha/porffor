// Source/data-derived expectations; authored control, not an executed result.
function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { check(Object.is(actual, expected), label); }
function throws(ctor, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  check(caught instanceof ctor && caught.constructor === ctor, label);
}
function abrupt(marker, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  same(caught, marker, label);
}

var PR = Intl.PluralRules, proto = PR.prototype;
same(PR.name, 'PluralRules', 'constructor name'); same(PR.length, 0, 'constructor length');
var pd = Object.getOwnPropertyDescriptor(PR, 'prototype');
check(!pd.writable && !pd.enumerable && !pd.configurable, 'prototype flags');
same(Object.getPrototypeOf(proto), Object.prototype, 'ordinary prototype');
same(proto.constructor, PR, 'prototype constructor');
var nd = Object.getOwnPropertyDescriptor(Intl, 'PluralRules');
check(nd.writable && !nd.enumerable && nd.configurable, 'namespace flags');
for (var row of [[PR,'supportedLocalesOf',1], [proto,'resolvedOptions',0], [proto,'select',1], [proto,'selectRange',2]]) {
  var descriptor = Object.getOwnPropertyDescriptor(row[0], row[1]), method = descriptor.value;
  same(method.name, row[1], 'method name'); same(method.length, row[2], 'method length');
  check(descriptor.writable && !descriptor.enumerable && descriptor.configurable, 'method flags');
  throws(TypeError, function () { Reflect.construct(method, []); }, 'nonconstructor');
}
var tag = Object.getOwnPropertyDescriptor(proto, Symbol.toStringTag);
same(tag.value, 'Intl.PluralRules', 'tag'); check(!tag.writable && !tag.enumerable && tag.configurable, 'tag flags');
same(Object.prototype.toString.call(new PR('en')), '[object Intl.PluralRules]', 'instance tag');
throws(TypeError, function () { proto.select.call(proto, 1); }, 'prototype is unbranded');
throws(TypeError, function () { proto.resolvedOptions.call(new Proxy(new PR('en'), {})); }, 'proxy is unbranded');
class Sub extends PR {}
var sub = new Sub('en'); same(Object.getPrototypeOf(sub), Sub.prototype, 'subclass prototype'); same(sub.select(1), 'one', 'subclass brand');

print("ok intrinsic_family");
262;
