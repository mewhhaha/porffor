let effects = [];
function* values() {
  yield "a";
  yield "b";
}

const expectedThis = function () {
  return this;
}.call(undefined);

values().forEach(function (value, index) {
  if (this !== expectedThis) {
    throw "callback this";
  }
  effects.push(value);
  effects.push(index);
});

if (
  effects.length !== 4 ||
  effects[0] !== "a" ||
  effects[1] !== 0 ||
  effects[2] !== "b" ||
  effects[3] !== 1
) {
  throw "effects";
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

if (typeof closable.return !== "function") {
  throw "return read";
}
closable.return();
if (!closed) {
  throw "direct return call";
}
closed = false;

let invalidCallbackThrew = false;
try {
  closable.forEach();
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
function ForEachSentinelError() {}
if (new ForEachSentinelError().constructor !== ForEachSentinelError) {
  throw "direct constructor";
}
let directThrownConstructorMatched = false;
try {
  ++callbackCalls;
  throw new ForEachSentinelError();
} catch (error) {
  directThrownConstructorMatched = error.constructor === ForEachSentinelError;
}
if (!directThrownConstructorMatched) {
  throw "direct throw constructor";
}
callbackCalls = 0;
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
  new ClosingIterator().forEach(function () {
    ++callbackCalls;
    throw new ForEachSentinelError();
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
  callbackConstructorMatched = error.constructor === ForEachSentinelError;
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

function NextSentinelError() {}
let nextClosed = false;
let nextThrowIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    throw new NextSentinelError();
  },
  return: function () {
    nextClosed = true;
    return {};
  },
};
let nextThrowMatched = false;
try {
  nextThrowIterator.forEach(function () {});
} catch (error) {
  nextThrowMatched = error instanceof NextSentinelError;
}
if (!nextThrowMatched) {
  throw "next throw constructor";
}
if (nextClosed) {
  throw "next throw close";
}

function ValueSentinelError() {}
let valueClosed = false;
let valueThrowIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    return {
      done: false,
      get value() {
        throw new ValueSentinelError();
      },
    };
  },
  return: function () {
    valueClosed = true;
    return {};
  },
};
let valueThrowMatched = false;
try {
  valueThrowIterator.forEach(function () {});
} catch (error) {
  valueThrowMatched = error instanceof ValueSentinelError;
}
if (!valueThrowMatched) {
  throw "value throw constructor";
}
if (valueClosed) {
  throw "value throw close";
}

true;
