function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

function throwsTypeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof TypeError;
  }
  return false;
}

function throwsSentinel(fn) {
  let sentinel = { thrown: true };
  try {
    fn(sentinel);
  } catch (err) {
    return err === sentinel;
  }
  return false;
}

function mapperReturnArgs(value, index, source) {
  return [value, index, source.length];
}

function mapperThis(value) {
  return this;
}

let descriptor = Object.getOwnPropertyDescriptor(Array.prototype, "flatMap");
let flatMap = Array.prototype.flatMap;
let objectValue = { value: 1 };
let nestedValue = [2];
let arrayLikeElement = [1];
let arrayLikeReceiver = { length: 2, 0: arrayLikeElement, 1: 2 };
let thisObject = { thisArg: true };
let primitiveThis = 7;
let primitiveThisResult = flatMap.call([0], mapperThis, primitiveThis);
let nullThis = flatMap.call([0], mapperThis, null)[0];
let undefinedThis = flatMap.call([0], mapperThis, undefined)[0];

typeof flatMap === "function"
  && flatMap.name === "flatMap"
  && flatMap.length === 1
  && descriptor.value === flatMap
  && descriptor.writable === true
  && descriptor.enumerable === false
  && descriptor.configurable === true
  && sameArray([1, 2].flatMap(function (value) { return [value, value + 10]; }), [1, 11, 2, 12])
  && sameArray([1].flatMap(function () { return [nestedValue]; }), [nestedValue])
  && sameArray([1, 2].flatMap(function (value) { return value + 1; }), [2, 3])
  && sameArray([objectValue].flatMap(function (value) { return [value]; }), [objectValue])
  && sameArray([3, 4].flatMap(mapperReturnArgs), [3, 0, 2, 4, 1, 2])
  && flatMap.call([0], mapperThis, thisObject)[0] === thisObject
  && primitiveThisResult.length === 1
  && nullThis !== thisObject
  && undefinedThis !== thisObject
  && throwsTypeError(function () { [1].flatMap(); })
  && throwsTypeError(function () { [1].flatMap(null); })
  && throwsTypeError(function () { [1].flatMap(1); })
  && throwsSentinel(function (sentinel) { [1].flatMap(function () { throw sentinel; }); })
  && sameArray(flatMap.call(true, function () { return [1]; }), [])
  && sameArray(flatMap.call(arrayLikeReceiver, function (value) { return [value]; }), [arrayLikeElement, 2]);
