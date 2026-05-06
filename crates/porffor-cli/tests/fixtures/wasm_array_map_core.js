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
  return value + index + source.length;
}

function mapperThis() {
  return this;
}

let descriptor = Object.getOwnPropertyDescriptor(Array.prototype, "map");
let map = Array.prototype.map;
let thisObject = { thisArg: true };
let sparse = [1, 2, 3];
delete sparse[1];
sparse[2] = 3;
let callbackCount = 0;
let sparseResult = sparse.map(function (value) {
  callbackCount = callbackCount + 1;
  return value + 10;
});
let primitiveThis = 7;
let primitiveThisResult = map.call([0], mapperThis, primitiveThis);
let nullThis = map.call([0], mapperThis, null)[0];
let undefinedThis = map.call([0], mapperThis, undefined)[0];
let arrayLikeReceiver = { length: 2, 0: 5, 1: 6 };

typeof map === "function"
  && map.name === "map"
  && map.length === 1
  && descriptor.value === map
  && descriptor.writable === true
  && descriptor.enumerable === false
  && descriptor.configurable === true
  && sameArray([1, 2, 3].map(function (value) { return value * 2; }), [2, 4, 6])
  && sameArray([3, 4].map(mapperReturnArgs), [5, 7])
  && [0].map(mapperThis, thisObject)[0] === thisObject
  && primitiveThisResult.length === 1
  && nullThis !== thisObject
  && undefinedThis !== thisObject
  && sparseResult.length === 3
  && sparseResult[0] === 11
  && sparseResult[2] === 13
  && callbackCount === 2
  && Object.prototype.hasOwnProperty.call(sparseResult, "1") === false
  && sameArray(map.call(arrayLikeReceiver, function (value) { return value + 1; }), [6, 7])
  && sameArray(map.call(true, function () { return 1; }), [])
  && throwsTypeError(function () { map.call(null, function (value) { return value; }); })
  && throwsTypeError(function () { map.call(undefined, function (value) { return value; }); })
  && throwsTypeError(function () { [1].map(); })
  && throwsTypeError(function () { [1].map(null); })
  && throwsTypeError(function () { [1].map(1); })
  && throwsSentinel(function (sentinel) { [1].map(function () { throw sentinel; }); });
