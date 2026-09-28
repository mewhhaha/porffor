function arrayKeys() {
  var array = [1];
  array['Symbol.isConcatSpreadable'] = false;
  return [array['Symbol.isConcatSpreadable'] === false,
    array[Symbol.isConcatSpreadable] === undefined,
    Object.hasOwn(array, 'Symbol.isConcatSpreadable'),
    Object.hasOwn(array, Symbol.isConcatSpreadable)];
}
print(arrayKeys().join(' '));
print(Symbol('key')['Symbol.toPrimitive'] === undefined);
Symbol.prototype['Symbol.toPrimitive'] = 17;
print(Symbol('key')['Symbol.toPrimitive'] === 17);
delete Symbol.prototype['Symbol.toPrimitive'];
print(Symbol('key')['Symbol.toPrimitive'] === undefined);

var array = [1];
var symbol = Symbol('key');
print(array['Symbol.isConcatSpreadable'] === undefined);
array['Symbol.isConcatSpreadable'] = false;
print(array['Symbol.isConcatSpreadable'] === false, array[Symbol.isConcatSpreadable] === undefined,
  Object.hasOwn(array, 'Symbol.isConcatSpreadable'), Object.hasOwn(array, Symbol.isConcatSpreadable));
print([].concat(array).length === 1, [].concat(array)[0] === 1);
delete array['Symbol.isConcatSpreadable'];
array[Symbol.isConcatSpreadable] = false;
print(array['Symbol.isConcatSpreadable'] === undefined, array[Symbol.isConcatSpreadable] === false);
print(symbol['Symbol.toPrimitive'] === undefined);
Symbol.prototype['Symbol.toPrimitive'] = 17;
print(symbol['Symbol.toPrimitive'] === 17, typeof symbol[Symbol.toPrimitive] === 'function');
delete Symbol.prototype['Symbol.toPrimitive'];
print(symbol['Symbol.toPrimitive'] === undefined);
