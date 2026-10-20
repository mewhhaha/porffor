function asyncReturnScenario() {
  let scenario = {
    expectedTrace: "inNrchtdvrchtdv",
    iterable: undefined,
    secondArgument: "return-2",
    trace: "",
  };
  let returnCount = 0;
  let iterator = {
    get next() {
      scenario.trace += "n";
      return function () {
        scenario.trace += this === iterator ? "N" : "x";
        return { value: "next", done: false };
      };
    },
    get return() {
      scenario.trace += this === iterator ? "r" : "x";
      return function (argument) {
        returnCount += 1;
        let expectedArgument = returnCount === 1 ? "return-1" : "return-2";
        scenario.trace +=
          this === iterator && argument === expectedArgument ? "c" : "x";

        let done = returnCount === 2;
        let result = {
          get done() {
            scenario.trace += this === result ? "d" : "x";
            return done;
          },
          get value() {
            scenario.trace += this === result ? "v" : "x";
            return done ? "return-2" : "return-1";
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

function synchronousReturnScenario() {
  let scenario = {
    expectedTrace: "inNrcdvrcdv",
    iterable: undefined,
    secondArgument: undefined,
    trace: "",
  };
  let returnCount = 0;
  let iterator = {
    get next() {
      scenario.trace += "n";
      return function () {
        scenario.trace += this === iterator ? "N" : "x";
        return { value: "next", done: false };
      };
    },
    get return() {
      scenario.trace += this === iterator ? "r" : "x";
      return function (argument) {
        returnCount += 1;
        let expectedArgument = returnCount === 1 ? "return-1" : undefined;
        scenario.trace +=
          this === iterator && argument === expectedArgument ? "c" : "x";

        let done = returnCount === 2;
        let result = {
          get done() {
            scenario.trace += this === result ? "d" : "x";
            return done;
          },
          get value() {
            scenario.trace += this === result ? "v" : "x";
            return done ? "return-2" : "return-1";
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

function validatesReturn(delegate, scenario) {
  let iterator = delegate(scenario.iterable);
  return iterator.next().then(function (initialResult) {
    if (initialResult.value !== "next" || initialResult.done !== false) return false;

    return iterator.return("return-1").then(function (firstReturn) {
      if (firstReturn.value !== "return-1" || firstReturn.done !== false) return false;

      return iterator.return(scenario.secondArgument).then(function (secondReturn) {
        return (
          secondReturn.value === "return-2" &&
          secondReturn.done === true &&
          scenario.trace === scenario.expectedTrace
        );
      });
    });
  });
}

class PrivateStaticDelegate {
  static async *#delegate(source) {
    yield* source;
  }

  static get delegate() {
    return this.#delegate;
  }
}

let objectDelegate = {
  async *delegate(source) {
    yield* source;
  },
}.delegate;

let validations = [
  validatesReturn(PrivateStaticDelegate.delegate, asyncReturnScenario()),
  validatesReturn(PrivateStaticDelegate.delegate, synchronousReturnScenario()),
  validatesReturn(objectDelegate, asyncReturnScenario()),
  validatesReturn(objectDelegate, synchronousReturnScenario()),
];
Promise.all(validations).then(function (results) {
  let passed = 0;
  for (let i = 0; i < results.length; i += 1) {
    if (results[i]) passed += 1;
  }
  print("async-generator-return-wrapper-validation:" + passed);
});

0;
