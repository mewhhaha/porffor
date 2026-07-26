function asyncThrowScenario() {
  let scenario = {
    expectedTrace: "inNqchtdvqchtdv",
    iterable: undefined,
    secondArgument: "throw-2",
    trace: "",
  };
  let throwCount = 0;
  let iterator = {
    get next() {
      scenario.trace += "n";
      return function () {
        scenario.trace += this === iterator ? "N" : "x";
        return { value: "next", done: false };
      };
    },
    get throw() {
      scenario.trace += this === iterator ? "q" : "x";
      return function (argument) {
        throwCount += 1;
        let expectedArgument = throwCount === 1 ? "throw-1" : "throw-2";
        scenario.trace +=
          this === iterator && argument === expectedArgument ? "c" : "x";

        let done = throwCount === 2;
        let result = {
          get done() {
            scenario.trace += this === result ? "d" : "x";
            return done;
          },
          get value() {
            scenario.trace += this === result ? "v" : "x";
            return done ? "throw-2" : "throw-1";
          },
        };
        let thenable = {
          get then() {
            scenario.trace += this === thenable ? "h" : "x";
            return function (resolve) {
              scenario.trace += this === thenable ? "t" : "x";
              resolve(result);
            };
          },
        };
        return thenable;
      };
    },
  };
  let iterable = {
    [Symbol.asyncIterator]() {
      scenario.trace += this === iterable ? "i" : "x";
      return iterator;
    },
  };
  scenario.iterable = iterable;
  return scenario;
}

function synchronousThrowScenario() {
  let scenario = {
    expectedTrace: "inNqcdvqcdv",
    iterable: undefined,
    secondArgument: undefined,
    trace: "",
  };
  let throwCount = 0;
  let iterator = {
    get next() {
      scenario.trace += "n";
      return function () {
        scenario.trace += this === iterator ? "N" : "x";
        return { value: "next", done: false };
      };
    },
    get throw() {
      scenario.trace += this === iterator ? "q" : "x";
      return function (argument) {
        throwCount += 1;
        let expectedArgument = throwCount === 1 ? "throw-1" : undefined;
        scenario.trace +=
          this === iterator && argument === expectedArgument ? "c" : "x";

        let done = throwCount === 2;
        let result = {
          get done() {
            scenario.trace += this === result ? "d" : "x";
            return done;
          },
          get value() {
            scenario.trace += this === result ? "v" : "x";
            return done ? "throw-2" : "throw-1";
          },
        };
        return result;
      };
    },
  };
  let iterable = {
    [Symbol.iterator]() {
      scenario.trace += this === iterable ? "i" : "x";
      return iterator;
    },
  };
  scenario.iterable = iterable;
  return scenario;
}

function validatesThrow(delegate, scenario) {
  let iterator = delegate(scenario.iterable);
  return iterator.next().then(function (initialResult) {
    if (initialResult.value !== "next" || initialResult.done !== false) return false;

    return iterator.throw("throw-1").then(function (firstThrow) {
      if (firstThrow.value !== "throw-1" || firstThrow.done !== false) return false;

      return iterator.throw(scenario.secondArgument).then(function (secondThrow) {
        return (
          secondThrow.value === "outer:throw-2" &&
          secondThrow.done === true &&
          scenario.trace === scenario.expectedTrace
        );
      });
    });
  });
}

class PrivateStaticDelegate {
  static async *#delegate(source) {
    let result = yield* source;
    return "outer:" + result;
  }

  static get delegate() {
    return this.#delegate;
  }
}

let objectDelegate = {
  async *delegate(source) {
    let result = yield* source;
    return "outer:" + result;
  },
}.delegate;

let validations = [
  validatesThrow(PrivateStaticDelegate.delegate, asyncThrowScenario()),
  validatesThrow(PrivateStaticDelegate.delegate, synchronousThrowScenario()),
  validatesThrow(objectDelegate, asyncThrowScenario()),
  validatesThrow(objectDelegate, synchronousThrowScenario()),
];
Promise.all(validations).then(function (results) {
  let passed = 0;
  for (let i = 0; i < results.length; i += 1) {
    if (results[i]) passed += 1;
  }
  print("async-generator-throw-wrapper-validation:" + passed);
});

0;
