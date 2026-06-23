let effects = [];
function* values() {
  yield "a";
  yield "b";
  yield "c";
}

const expectedThis = function () {
  return this;
}.call(undefined);

let result = values().reduce(function (memo, value, index) {
  if (this !== expectedThis) {
    throw "callback this";
  }
  effects.push(memo);
  effects.push(value);
  effects.push(index);
  return value;
});

if (result !== "c") {
  throw "no initial result";
}
if (
  effects.length !== 6 ||
  effects[0] !== "a" ||
  effects[1] !== "b" ||
  effects[2] !== 1 ||
  effects[3] !== "b" ||
  effects[4] !== "c" ||
  effects[5] !== 2
) {
  throw "no initial effects";
}

let initialValue = { seed: true };
let oneCallCount = 0;
result = (function* () {
  yield "x";
})().reduce(function (memo, value, index) {
  ++oneCallCount;
  if (memo !== initialValue || value !== "x" || index !== 0) {
    throw "initial callback args";
  }
  return value;
}, initialValue);

if (result !== "x" || oneCallCount !== 1) {
  throw "initial result";
}

let noInitialCalls = 0;
result = (function* () {
  yield "only";
})().reduce(function () {
  ++noInitialCalls;
  return "bad";
});
if (result !== "only" || noInitialCalls !== 0) {
  throw "single no initial";
}

let emptyInitial = {};
result = (function* () {})().reduce(function () {
  throw "empty reducer";
}, emptyInitial);
if (result !== emptyInitial) {
  throw "empty initial";
}

let emptyNoInitialThrew = false;
try {
  (function* () {})().reduce(function () {});
} catch (error) {
  emptyNoInitialThrew = error instanceof TypeError;
}
if (!emptyNoInitialThrew) {
  throw "empty no initial";
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

let invalidReducerThrew = false;
try {
  closable.reduce();
} catch (error) {
  invalidReducerThrew = error instanceof TypeError;
}

if (!invalidReducerThrew) {
  throw "invalid reducer throw";
}
if (!closed) {
  throw "invalid reducer close";
}

let returnCalls = 0;
let reducerCalls = 0;
let reducerThrew = false;
let reducerConstructorMatched = false;
function ReduceSentinelError() {}
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
  new ClosingIterator().reduce(function () {
    ++reducerCalls;
    throw new ReduceSentinelError();
  }, 0);
} catch (error) {
  reducerThrew = true;
  if (error.closeResult === true) {
    throw "threw close result";
  }
  if (error.constructor === Object) {
    throw "reducer constructor object";
  }
  if (error.constructor === undefined) {
    throw "reducer constructor undefined";
  }
  reducerConstructorMatched = error.constructor === ReduceSentinelError;
}

if (!reducerThrew) {
  throw "reducer throw";
}
if (!reducerConstructorMatched) {
  throw "reducer constructor";
}
if (reducerCalls !== 1) {
  throw "reducer calls";
}
if (returnCalls !== 1) {
  throw "reducer close";
}

let nonObjectThrew = false;
let nonObjectIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    return 0;
  },
};
try {
  nonObjectIterator.reduce(function (memo, value) {
    return value;
  });
} catch (error) {
  nonObjectThrew = error instanceof TypeError;
}
if (!nonObjectThrew) {
  throw "non-object next result";
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
  nextThrowIterator.reduce(function (memo, value) {
    return value;
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
  doneThrowIterator.reduce(function (memo, value) {
    return value;
  }, 0);
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
  valueThrowIterator.reduce(function (memo, value) {
    return value;
  }, 0);
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
