var ok = true;

function captureArraySpreadError(iterable) {
  var caught;
  try {
    [...iterable];
  } catch (error) {
    caught = error;
  }
  return caught;
}

function isExpectedTypeError(error, message) {
  return error instanceof TypeError && error.message === message;
}

var nonCallableIterator = {};
nonCallableIterator[Symbol.iterator] = 0;
var nonCallableIteratorError = captureArraySpreadError(nonCallableIterator);
ok = ok && isExpectedTypeError(
  nonCallableIteratorError,
  "array spread value is not iterable"
);

var primitiveIteratorResult = {};
primitiveIteratorResult[Symbol.iterator] = function() {
  return 1;
};
var primitiveIteratorResultError = captureArraySpreadError(primitiveIteratorResult);
ok = ok && isExpectedTypeError(
  primitiveIteratorResultError,
  "array spread iterator method must return object"
);

var nonCallableNextClosed = 0;
var nonCallableNext = {};
nonCallableNext[Symbol.iterator] = function() {
  return {
    next: 0,
    return: function() {
      nonCallableNextClosed++;
      return {};
    }
  };
};
var nonCallableNextError = captureArraySpreadError(nonCallableNext);
ok = ok && isExpectedTypeError(
  nonCallableNextError,
  "array spread iterator next must be callable"
);
ok = ok && nonCallableNextClosed === 0;

var primitiveNextResultClosed = 0;
var primitiveNextResult = {};
primitiveNextResult[Symbol.iterator] = function() {
  return {
    next: function() {
      return 1;
    },
    return: function() {
      primitiveNextResultClosed++;
      return {};
    }
  };
};
var primitiveNextResultError = captureArraySpreadError(primitiveNextResult);
ok = ok && isExpectedTypeError(
  primitiveNextResultError,
  "array spread iterator next result must be object"
);
ok = ok && primitiveNextResultClosed === 0;

var doneError = {};
var doneErrorClosed = 0;
var doneErrorIterable = {};
doneErrorIterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        get done() {
          throw doneError;
        }
      };
    },
    return: function() {
      doneErrorClosed++;
      return {};
    }
  };
};
ok = ok && captureArraySpreadError(doneErrorIterable) === doneError;
ok = ok && doneErrorClosed === 0;

var valueError = {};
var valueErrorClosed = 0;
var valueErrorIterable = {};
valueErrorIterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        done: false,
        get value() {
          throw valueError;
        }
      };
    },
    return: function() {
      valueErrorClosed++;
      return {};
    }
  };
};
ok = ok && captureArraySpreadError(valueErrorIterable) === valueError;
ok = ok && valueErrorClosed === 0;

var originalStringIteratorDescriptor = Object.getOwnPropertyDescriptor(
  String.prototype,
  Symbol.iterator
);
var stringIteratorCalls = 0;
var stringIteratorReceiver;
Object.defineProperty(String.prototype, Symbol.iterator, {
  configurable: true,
  writable: true,
  value: function() {
    "use strict";
    stringIteratorCalls++;
    stringIteratorReceiver = this;
    var done = false;
    return {
      next: function() {
        if (done) return { done: true };
        done = true;
        return { value: 17, done: false };
      }
    };
  }
});
var spreadString;
try {
  spreadString = [..."ab"];
} finally {
  Object.defineProperty(
    String.prototype,
    Symbol.iterator,
    originalStringIteratorDescriptor
  );
}
ok = ok && stringIteratorCalls === 1;
ok = ok && stringIteratorReceiver === "ab";
ok = ok && spreadString.length === 1 && spreadString[0] === 17;

ok;
