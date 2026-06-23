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
let result = iterator.every(function (value, index) {
  if (this !== expectedThis) {
    throw "callback this";
  }
  effects.push(value);
  effects.push(index);
  return index < 1;
});

if (result !== false) {
  throw "falsey result";
}
if (
  effects.length !== 4 ||
  effects[0] !== "a" ||
  effects[1] !== 0 ||
  effects[2] !== "b" ||
  effects[3] !== 1
) {
  throw "falsey effects";
}
let afterClose = iterator.next();
if (afterClose.done !== true || afterClose.value !== undefined) {
  throw "falsey close";
}

function* allTruthyValues() {
  yield 1;
  yield 2;
}

if (
  allTruthyValues().every(function (value) {
    return value;
  }) !== true
) {
  throw "all truthy result";
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
  closable.every();
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
function EverySentinelError() {}
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
  new ClosingIterator().every(function () {
    ++callbackCalls;
    throw new EverySentinelError();
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
  callbackConstructorMatched = error.constructor === EverySentinelError;
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
  nextThrowIterator.every(function () {
    return true;
  });
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
  valueThrowIterator.every(function () {
    return true;
  });
} catch (error) {
  valueThrowMatched = error instanceof ValueSentinelError;
}
if (!valueThrowMatched) {
  throw "value throw constructor";
}
if (valueClosed) {
  throw "value throw close";
}

let arrayIterator = [1, 2, 3, 4, 5][Symbol.iterator]();
if (arrayIterator.return !== undefined) {
  throw "array iterator return";
}
let arrayResult = arrayIterator.every(function (value) {
  return value < 4;
});
if (arrayResult !== false) {
  throw "array iterator result";
}
let remaining = arrayIterator.next();
if (remaining.done !== false || remaining.value !== 5) {
  throw "array iterator remaining";
}

true;
