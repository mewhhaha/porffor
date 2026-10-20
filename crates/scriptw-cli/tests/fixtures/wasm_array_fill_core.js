let empty = [];
let emptyResult = empty.fill("value");

let defaultBounds = [0, 1, 2];
let defaultResult = defaultBounds.fill("value");

let negativeBounds = [0, 1, 2, 3, 4];
negativeBounds.fill("value", -4, -1);

let infiniteBounds = [0, 1, 2];
infiniteBounds.fill("value", Infinity);
let negativeInfiniteBounds = [0, 1, 2];
negativeInfiniteBounds.fill("value", -Infinity);

let arrayLike = { 0: "zero", 1: "one", 2: "two", length: 3 };
let arrayLikeResult = Array.prototype.fill.call(arrayLike, "value", 1);

let explicitUndefinedEnd = [0, 0];
explicitUndefinedEnd.fill(1, 0, undefined);

let resizableBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let fixedView = new Uint8Array(resizableBuffer, 0, 4);
Array.prototype.fill.call(fixedView, {
  valueOf: function () {
    resizableBuffer.resize(2);
    return 3;
  }
}, 1, 2);
let resizedView = new Uint8Array(resizableBuffer);

let coercionOrder = "";
let coerced = [0, 1, 2];
coerced.fill(
  "value",
  {
    valueOf: function () {
      coercionOrder = coercionOrder + "start";
      return 1;
    }
  },
  {
    valueOf: function () {
      coercionOrder = coercionOrder + "end";
      return 2;
    }
  }
);

let setOrder = "";
let abrupt = new Proxy({ length: 3 }, {
  set: function (target, key, value) {
    if (key === "0") {
      setOrder = setOrder + value;
      return true;
    }
    if (key === "1") throw "stop";
    target[key] = value;
    return true;
  }
});
let propagated = false;
try {
  Array.prototype.fill.call(abrupt, "value");
} catch (error) {
  propagated = error === "stop";
}

typeof Array(0).fill === "function"
  && emptyResult === empty
  && empty.length === 0
  && defaultResult === defaultBounds
  && defaultBounds[0] === "value"
  && defaultBounds[1] === "value"
  && defaultBounds[2] === "value"
  && negativeBounds[0] === 0
  && negativeBounds[1] === "value"
  && negativeBounds[2] === "value"
  && negativeBounds[3] === "value"
  && negativeBounds[4] === 4
  && infiniteBounds[0] === 0
  && infiniteBounds[1] === 1
  && infiniteBounds[2] === 2
  && negativeInfiniteBounds[0] === "value"
  && negativeInfiniteBounds[1] === "value"
  && negativeInfiniteBounds[2] === "value"
  && arrayLikeResult === arrayLike
  && arrayLike[0] === "zero"
  && arrayLike[1] === "value"
  && arrayLike[2] === "value"
  && explicitUndefinedEnd[0] === 1
  && explicitUndefinedEnd[1] === 1
  && resizedView[0] === 0
  && resizedView[1] === 0
  && coercionOrder === "startend"
  && coerced[0] === 0
  && coerced[1] === "value"
  && coerced[2] === 2
  && propagated
  && setOrder === "value";
