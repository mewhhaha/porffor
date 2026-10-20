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
let result = iterator.find(function (value, index) {
  if (this !== expectedThis) {
    throw "callback this";
  }
  effects.push(value);
  effects.push(index);
  return index === 1;
});

if (result !== "b") {
  throw "found result";
}
if (
  effects.length !== 4 ||
  effects[0] !== "a" ||
  effects[1] !== 0 ||
  effects[2] !== "b" ||
  effects[3] !== 1
) {
  throw "found effects";
}
let afterClose = iterator.next();
if (afterClose.done !== true || afterClose.value !== undefined) {
  throw "found close";
}

function* allFalseyValues() {
  yield 1;
  yield 2;
}

if (
  allFalseyValues().find(function () {
    return false;
  }) !== undefined
) {
  throw "all falsey result";
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
  closable.find();
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
function FindSentinelError() {}
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
  new ClosingIterator().find(function () {
    ++callbackCalls;
    throw new FindSentinelError();
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
  callbackConstructorMatched = error.constructor === FindSentinelError;
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
  nextThrowIterator.find(function () {
    return false;
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

function DoneSentinelError() {}
let doneClosed = false;
let doneThrowIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    return {
      get done() {
        throw new DoneSentinelError();
      },
      value: 1,
    };
  },
  return: function () {
    doneClosed = true;
    return {};
  },
};
let doneThrowMatched = false;
try {
  doneThrowIterator.find(function () {
    return true;
  });
} catch (error) {
  doneThrowMatched = error instanceof DoneSentinelError;
}
if (!doneThrowMatched) {
  throw "done throw constructor";
}
if (doneClosed) {
  throw "done throw close";
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
  valueThrowIterator.find(function () {
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
let arrayResult = arrayIterator.find(function (value) {
  return value > 3;
});
if (arrayResult !== 4) {
  throw "array iterator result";
}
let remaining = arrayIterator.next();
if (remaining.done !== false || remaining.value !== 5) {
  throw "array iterator remaining";
}

true;
