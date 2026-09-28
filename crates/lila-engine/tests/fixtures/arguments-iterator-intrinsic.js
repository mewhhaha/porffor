var intrinsic = Array.prototype.values;
function mapped(x) { return arguments; }
function unmapped(x) { 'use strict'; return arguments; }
function defaults(x = 1) { return arguments; }
var sources = [mapped(1, 2), unmapped(3, 4), defaults(5, 6)];
Array.prototype.values = function replacedValues() {};
Array.prototype[Symbol.iterator] = function replacedIterator() {};
for (var i = 0; i < sources.length; i++) {
  var source = sources[i];
  var descriptor = Object.getOwnPropertyDescriptor(source, Symbol.iterator);
  print(source[Symbol.iterator] === intrinsic, descriptor.value === intrinsic,
    descriptor.writable, descriptor.enumerable, descriptor.configurable,
    Object.getOwnPropertySymbols(source).length);
  delete source[Symbol.iterator];
  print(source[Symbol.iterator] === undefined,
    Object.getOwnPropertyDescriptor(source, Symbol.iterator) === undefined,
    Symbol.iterator in source, Object.getOwnPropertySymbols(source).length);
}
function deferred(value) { return deferred.arguments; }
print(deferred(7)[Symbol.iterator] === intrinsic);
