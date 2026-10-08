function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
function descriptor(object, key, value, writable, enumerable, configurable) {
  var d = Object.getOwnPropertyDescriptor(object, key);
  same(d.value, value, String(key) + ' value');
  same(d.writable, writable, String(key) + ' writable');
  same(d.enumerable, enumerable, String(key) + ' enumerable');
  same(d.configurable, configurable, String(key) + ' configurable');
}
var C = Intl.ListFormat;
same(typeof C, 'function', 'constructor installed');
same(Object.getPrototypeOf(C), Function.prototype, 'constructor prototype');
descriptor(Intl, 'ListFormat', C, true, false, true);
descriptor(C, 'name', 'ListFormat', false, false, true);
descriptor(C, 'length', 0, false, false, true);
descriptor(C, 'prototype', C.prototype, false, false, false);
same(Object.getPrototypeOf(C.prototype), Object.prototype, 'ordinary prototype');
descriptor(C.prototype, 'constructor', C, true, false, true);
descriptor(C.prototype, Symbol.toStringTag, 'Intl.ListFormat', false, false, true);
for (var row of [[C, 'supportedLocalesOf', 1], [C.prototype, 'resolvedOptions', 0], [C.prototype, 'format', 1], [C.prototype, 'formatToParts', 1]]) {
  var method = row[0][row[1]];
  descriptor(row[0], row[1], method, true, false, true);
  descriptor(method, 'name', row[1], false, false, true);
  descriptor(method, 'length', row[2], false, false, true);
  check(!Object.hasOwn(method, 'prototype'), row[1] + ' nonconstructor');
}
var lf = new C('en-US');
same(Object.getPrototypeOf(lf), C.prototype, 'instance prototype');
same(Object.prototype.toString.call(lf), '[object Intl.ListFormat]', 'tag');
same(Reflect.ownKeys(lf).length, 0, 'private slots have no own keys');
check(Object.isExtensible(lf), 'extensible instance');
var first = lf.resolvedOptions();
same(Object.getPrototypeOf(first), Object.prototype, 'resolved ordinary object');
same(Object.keys(first).join(','), 'locale,type,style', 'resolved property order');
descriptor(first, 'locale', 'en-US', true, true, true);
descriptor(first, 'type', 'conjunction', true, true, true);
descriptor(first, 'style', 'long', true, true, true);
first.type = 'unit'; delete first.locale;
var second = lf.resolvedOptions();
check(first !== second, 'fresh options');
same(second.type, 'conjunction', 'options cannot change private record');
same(second.locale, 'en-US', 'private locale retained');
same(new C('en-US-u-ca-buddhist').resolvedOptions().locale, 'en-US', 'no relevant extensions');
class Derived extends C {}
var derived = new Derived('en-US', {type: 'unit', style: 'narrow'});
same(Object.getPrototypeOf(derived), Derived.prototype, 'actual subclass prototype');
same(derived.format(['A', 'B']), 'A B', 'actual subclass branded configuration');
print('ok metadata_and_resolved');
262;
