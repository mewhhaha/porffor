let nextGetterReason = {};
let nextCallReason = {};
let doneGetterReason = {};
let valueGetterReason = {};
let numberThenCalled = false;

Number.prototype.then = function () {
  numberThenCalled = true;
};

function nextGetterAbruptSource() {
  return {
    [Symbol.asyncIterator]() {
      return {
        get next() {
          throw nextGetterReason;
        },
      };
    },
  };
}

function nonCallableNextSource() {
  return {
    [Symbol.asyncIterator]() {
      return { next: 0 };
    },
  };
}

function nextCallAbruptSource() {
  return {
    [Symbol.asyncIterator]() {
      return {
        next() {
          throw nextCallReason;
        },
      };
    },
  };
}

function nonObjectResultSource() {
  return {
    [Symbol.asyncIterator]() {
      return {
        next() {
          return 42;
        },
      };
    },
  };
}

function doneGetterAbruptSource(observation) {
  return {
    [Symbol.asyncIterator]() {
      return {
        next() {
          return {
            get done() {
              throw doneGetterReason;
            },
            get value() {
              observation.valueRead = true;
              return 0;
            },
          };
        },
      };
    },
  };
}

function valueGetterAbruptSource() {
  return {
    [Symbol.asyncIterator]() {
      return {
        next() {
          return {
            done: false,
            get value() {
              throw valueGetterReason;
            },
          };
        },
      };
    },
  };
}

function rejectionMatches(promise, expectedReason) {
  return promise.then(
    function () {
      return false;
    },
    function (reason) {
      return reason === expectedReason;
    }
  );
}

function rejectsWithTypeError(promise) {
  return promise.then(
    function () {
      return false;
    },
    function (reason) {
      return reason.constructor === TypeError;
    }
  );
}

function validationsFor(delegate) {
  let doneObservation = { valueRead: false };
  let doneValidation = rejectionMatches(
    delegate(doneGetterAbruptSource(doneObservation)).next(),
    doneGetterReason
  ).then(function (matches) {
    return matches && !doneObservation.valueRead;
  });

  return [
    rejectionMatches(delegate(nextGetterAbruptSource()).next(), nextGetterReason),
    rejectsWithTypeError(delegate(nonCallableNextSource()).next()),
    rejectionMatches(delegate(nextCallAbruptSource()).next(), nextCallReason),
    rejectsWithTypeError(delegate(nonObjectResultSource()).next()),
    doneValidation,
    rejectionMatches(delegate(valueGetterAbruptSource()).next(), valueGetterReason),
  ];
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

let validations = validationsFor(PrivateStaticDelegate.delegate).concat(
  validationsFor(objectDelegate)
);
Promise.all(validations).then(function (results) {
  let passed = 0;
  for (let i = 0; i < results.length; i += 1) {
    if (results[i]) passed += 1;
  }

  let observedNumberThen = numberThenCalled;
  print(
    "async-generator-next-wrapper-validation:" +
      passed +
      ":" +
      observedNumberThen
  );
});

0;
