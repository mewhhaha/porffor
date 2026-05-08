let log = "";

let iterable = {};
iterable[Symbol.iterator] = function() {
  log += "i";
  let step = 0;
  return {
    next: function() {
      log += "n";
      step = step + 1;
      if (step === 1) return { done: false, value: "a" };
      if (step === 2) {
        let result = { done: false };
        Object.defineProperty(result, "value", {
          get: function() {
            log += "v";
            return "b";
          }
        });
        return result;
      }
      let result = { done: true };
      Object.defineProperty(result, "value", {
        get: function() {
          throw "done value observed";
        }
      });
      return result;
    }
  };
};

let aggregate = new AggregateError(iterable, "m");
if (aggregate.errors.length !== 2) throw "iterable length";
if (aggregate.errors[0] !== "a") throw "first value";
if (aggregate.errors[1] !== "b") throw "second value";
if (log !== "innvn") throw log;

function assertTypeError(callback, label) {
  try {
    callback();
  } catch (error) {
    if (error instanceof TypeError) return;
    throw label + " wrong error";
  }
  throw label + " missing error";
}

function assertThrowsValue(callback, expected, label) {
  try {
    callback();
  } catch (error) {
    if (error === expected) return;
    throw label + " wrong value";
  }
  throw label + " missing error";
}

assertTypeError(function() { new AggregateError(undefined); }, "undefined");
assertTypeError(function() { new AggregateError({}); }, "missing iterator");
assertTypeError(function() { new AggregateError({ "Symbol.iterator": 1 }); }, "non-callable iterator");

let badIterator = {};
badIterator[Symbol.iterator] = function() { return 7; };
assertTypeError(function() { new AggregateError(badIterator); }, "bad iterator");

let badNext = {};
badNext[Symbol.iterator] = function() { return { next: 1 }; };
assertTypeError(function() { new AggregateError(badNext); }, "bad next");

let badResult = {};
badResult[Symbol.iterator] = function() {
  return {
    next: function() { return 1; }
  };
};
assertTypeError(function() { new AggregateError(badResult); }, "bad result");

let throwingIterator = {};
Object.defineProperty(throwingIterator, "Symbol.iterator", {
  get: function() { throw "iterator getter"; }
});
assertThrowsValue(function() { new AggregateError(throwingIterator); }, "iterator getter", "iterator getter");

let throwingNextGetter = {};
throwingNextGetter[Symbol.iterator] = function() {
  return Object.defineProperty({}, "next", {
    get: function() { throw "next getter"; }
  });
};
assertThrowsValue(function() { new AggregateError(throwingNextGetter); }, "next getter", "next getter");

let throwingNext = {};
throwingNext[Symbol.iterator] = function() {
  return {
    next: function() { throw "next call"; }
  };
};
assertThrowsValue(function() { new AggregateError(throwingNext); }, "next call", "next call");

let throwingDone = {};
throwingDone[Symbol.iterator] = function() {
  return {
    next: function() {
      return Object.defineProperty({}, "done", {
        get: function() { throw "done getter"; }
      });
    }
  };
};
assertThrowsValue(function() { new AggregateError(throwingDone); }, "done getter", "done getter");

let throwingValue = {};
throwingValue[Symbol.iterator] = function() {
  return {
    next: function() {
      let result = { done: false };
      Object.defineProperty(result, "value", {
        get: function() { throw "value getter"; }
      });
      return result;
    }
  };
};
assertThrowsValue(function() { new AggregateError(throwingValue); }, "value getter", "value getter");

123;
