let effects = [];
function* values() {
  yield "a";
  yield "b";
  yield "c";
}

const expectedThis = function () {
  return this;
}.call(undefined);

let iterator = values();
let result = iterator.some(function (value, index) {
  if (this !== expectedThis) {
    throw "callback this";
  }
  effects.push(value);
  effects.push(index);
  return index === 1;
});

if (result !== true) {
  throw "truthy result";
}
if (
  effects.length !== 4 ||
  effects[0] !== "a" ||
  effects[1] !== 0 ||
  effects[2] !== "b" ||
  effects[3] !== 1
) {
  throw "truthy effects";
}
let afterClose = iterator.next();
if (afterClose.done !== true || afterClose.value !== undefined) {
  throw "truthy close";
}

let closed = false;
const closable = {
  __proto__: Iterator.prototype,
  get next() {
    throw "next should not be read";
  },
  return: function () {
    closed = true;
    return {};
  },
};

let invalidCallbackThrew = false;
try {
  closable.some();
} catch (error) {
  invalidCallbackThrew = error instanceof TypeError;
}

if (!invalidCallbackThrew) {
  throw "invalid callback throw";
}
if (!closed) {
  throw "invalid callback close";
}

let returnCalls = 0;
let callbackCalls = 0;
let callbackThrew = false;
let callbackConstructorMatched = false;
function SomeSentinelError() {}
class ClosingIterator extends Iterator {
  next() {
    return { done: false, value: 1 };
  }
  return() {
    ++returnCalls;
    return { closeResult: true };
  }
}

try {
  new ClosingIterator().some(function () {
    ++callbackCalls;
    throw new SomeSentinelError();
  });
} catch (error) {
  callbackThrew = true;
  if (error.closeResult === true) {
    throw "threw close result";
  }
  if (error.constructor === Object) {
    throw "callback constructor object";
  }
  if (error.constructor === undefined) {
    throw "callback constructor undefined";
  }
  callbackConstructorMatched = error.constructor === SomeSentinelError;
}

if (!callbackThrew) {
  throw "callback throw";
}
if (!callbackConstructorMatched) {
  throw "callback constructor";
}
if (callbackCalls !== 1) {
  throw "callback calls";
}
if (returnCalls !== 1) {
  throw "callback close";
}

let arrayIterator = [1, 2, 3, 4, 5][Symbol.iterator]();
if (arrayIterator.return !== undefined) {
  throw "array iterator return";
}
let arrayResult = arrayIterator.some(function (value) {
  return value > 3;
});
if (arrayResult !== true) {
  throw "array iterator result";
}
let remaining = arrayIterator.next();
if (remaining.done !== false || remaining.value !== 5) {
  throw "array iterator remaining";
}

true;
