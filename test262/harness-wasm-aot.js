// Portions of this file are adapted from Test262 (https://github.com/tc39/test262)
// Test262 is BSD-3-Clause licensed; see the upstream LICENSE file
//
// Materialization activates the realm boundary only for cases that can use it.
// Keep the inactive template fail-loud so an unrecognized access cannot silently
// run with the wrong host semantics.

/// assert.js
function __porfAssertIsSameValue(a, b) {
  if (a === b) {
    return true;
  }
  return a !== a && b !== b;
}

function __porfAssertToString(value) {
  if (value === undefined) {
    return 'undefined';
  }
  if (value === null) {
    return 'null';
  }
  return String(value);
}

function __porfAssertUnsupported(name) {
  throw name + ' unsupported in wasm-aot assert harness';
}

function assert(mustBeTrue, message) {
  if (mustBeTrue) {
    return;
  }
  if (message === undefined) {
    message = 'Expected true but got false';
  }
  throw message;
}

assert._isSameValue = __porfAssertIsSameValue;
assert._toString = __porfAssertToString;
assert.sameValue = function (actual, expected, message) {
    if (actual === expected) {
      return;
    }
    if (actual !== actual && expected !== expected) {
      return;
    }

    if (message === undefined) {
      message = '';
    } else {
      message = message + ' ';
    }

    message = message + 'Expected SameValue(' + __porfAssertToString(actual) + ', ' + __porfAssertToString(expected) + ') to be true';
    throw message;
  };
assert.notSameValue = function (actual, unexpected, message) {
    if (actual === unexpected) {
      // SameValue matched; fall through to failure below.
    } else if (actual !== actual && unexpected !== unexpected) {
      // NaN is SameValue to NaN; fall through to failure below.
    } else {
      return;
    }

    if (message === undefined) {
      message = '';
    } else {
      message = message + ' ';
    }

    message = message + 'Expected SameValue(' + __porfAssertToString(actual) + ', ' + __porfAssertToString(unexpected) + ') to be false';
    throw message;
  };
assert.throws = function (expectedErrorConstructor, func, message) {
  return __porfAssertThrows(expectedErrorConstructor, func, message);
};

function __porfCompareArrayMismatchIndex(actual, expected) {
  if (actual.length !== expected.length) {
    return -2;
  }

  var index = 0;
  while (index < actual.length) {
    if (!__porfAssertIsSameValue(actual[index], expected[index])) {
      return index;
    }
    index = index + 1;
  }
  return -1;
}

function __porfAssertCompareArray(actual, expected, message) {
  var mismatchIndex = __porfCompareArrayMismatchIndex(actual, expected);
  if (mismatchIndex === -1) {
    return;
  }
  if (message) {
    throw message;
  }
  if (mismatchIndex === -2) {
    throw 'Expected arrays to have the same length';
  }

  throw 'Expected arrays to contain the same values at ' + mismatchIndex + ': ' +
    __porfAssertToString(actual[mismatchIndex]) + ' !== ' +
    __porfAssertToString(expected[mismatchIndex]);
}
assert.compareArray = __porfAssertCompareArray;

function compareArray(actual, expected) {
  return __porfCompareArrayMismatchIndex(actual, expected) === -1;
}

/// sta-preamble.js
function Test262Error(message) {
}

Test262Error.thrower = function (message) {
  throw new Test262Error(message);
};

function $DONOTEVALUATE() {
  throw 'Test262: This statement should not be evaluated.';
}

/// isConstructor.js
var isConstructor = __porfIsConstructor;

/// sta.js
function Test262Error(message) {
}

Test262Error.thrower = function (message) {
  throw new Test262Error(message);
};

function $DONOTEVALUATE() {
  throw 'Test262: This statement should not be evaluated.';
}

function __porfUnsupportedHost(name) {
  throw name + ' unsupported in wasm-aot host harness';
}

function AbstractModuleSource() {
  throw new TypeError();
}

function __porfAbstractModuleSourceToStringTag() {
  return undefined;
}

Object.defineProperty(AbstractModuleSource, "prototype", {
  value: AbstractModuleSource.prototype,
  writable: false,
  enumerable: false,
  configurable: false
});

Object.defineProperty(AbstractModuleSource.prototype, Symbol.toStringTag, {
  get: __porfAbstractModuleSourceToStringTag,
  set: undefined,
  enumerable: false,
  configurable: true
});

var $262 = {
  global: undefined,
  AbstractModuleSource: AbstractModuleSource,
  // Must be a real [[IsHTMLDDA]] exotic object, not an ordinary function that
  // returns null: the annexB `emulates-undefined` tests observe ToBoolean,
  // `typeof`, IsLooselyEqual and the absence of an own `prototype` property.
  // `__porfCreateHTMLDDA()` mints one function object carrying the
  // FUNCTION_FLAG_IS_HTMLDDA flag that every one of those paths consults.
  IsHTMLDDA: __porfCreateHTMLDDA(),
  gc: function () {
    gc();
  },
  detachArrayBuffer: __porfDetachArrayBuffer,
  evalScript: function () {
    __porfUnsupportedHost('evalScript');
  },
  createRealm: function () {
    __porfUnsupportedHost('createRealm');
  },
  destroy: function () {},
  getGlobal: function () {
    __porfUnsupportedHost('getGlobal');
  },
  agent: {
    start: function (source) {
      return __porfAgentStart(source);
    },
    broadcast: function (sab) {
      return __porfAgentBroadcast(sab);
    },
    receiveBroadcast: function (callback) {
      return callback(__porfAgentReceiveBroadcast());
    },
    report: function (value) {
      return __porfAgentReport(value);
    },
    getReport: function () {
      return __porfAgentGetReport();
    },
    sleep: function (milliseconds) {
      return __porfAgentSleep(milliseconds);
    },
    monotonicNow: function () {
      return __porfAgentMonotonicNow();
    },
    leaving: function () {
      return __porfAgentLeaving();
    }
  }
};

/// propertyHelper.js
function verifyProperty(obj, name, desc) {
  var originalDesc = Object.getOwnPropertyDescriptor(obj, name);

  if (desc === undefined) {
    if (originalDesc !== undefined) {
      throw "Expected descriptor to be undefined";
    }
    return true;
  }

  if (originalDesc === undefined) {
    throw "Expected descriptor to exist";
  }
  if (typeof desc !== "object") {
    throw "Expected descriptor object";
  }

  if (desc.value !== undefined) {
    if (originalDesc.value !== desc.value) {
      throw "Expected descriptor value";
    }
    if (obj[name] !== desc.value) {
      throw "Expected property value";
    }
  }

  if (desc.get !== undefined) {
    if (originalDesc.get !== desc.get) {
      throw "Expected descriptor getter";
    }
  }

  if (desc.set !== undefined) {
    if (originalDesc.set !== desc.set) {
      throw "Expected descriptor setter";
    }
  }

  if (desc.writable !== undefined) {
    if (originalDesc.writable !== desc.writable) {
      throw "Expected descriptor writable flag";
    }
  }

  if (desc.enumerable !== undefined) {
    if (originalDesc.enumerable !== desc.enumerable) {
      throw "Expected descriptor enumerable flag";
    }
  }

  if (desc.configurable !== undefined) {
    if (originalDesc.configurable !== desc.configurable) {
      throw "Expected descriptor configurable flag";
    }
  }

  return true;
}

var verifyPrimordialProperty = verifyProperty;

function __porfIsWritable(obj, name, verifyProp, value) {
  var newValue = value || "unlikelyValue";
  var oldValue = obj[name];
  var hadValue = Object.prototype.hasOwnProperty.call(obj, name);
  var writeSucceeded;

  if (arguments.length < 4 && newValue === oldValue) {
    newValue = newValue + "2";
  }

  try {
    obj[name] = newValue;
  } catch (e) {
  }

  writeSucceeded = obj[verifyProp || name] === newValue;

  if (writeSucceeded) {
    if (hadValue) {
      obj[name] = oldValue;
    } else {
      delete obj[name];
    }
  }

  return writeSucceeded;
}

function verifyNotWritable(obj, name, verifyProp, value) {
  var desc;
  if (verifyProp === undefined) {
    desc = Object.getOwnPropertyDescriptor(obj, name);
    if (desc === undefined) {
      throw "Expected descriptor to exist";
    }
    if (desc.set !== undefined) {
      throw "Expected obj[" + String(name) + "] NOT to be writable, but setter exists.";
    }
    if (desc.writable !== undefined) {
      if (desc.writable !== false) {
        throw "Expected obj[" + String(name) + "] NOT to be writable.";
      }
    }
  }
  if (__porfIsWritable(obj, name, verifyProp, value)) {
    throw "Expected obj[" + String(name) + "] NOT to be writable.";
  }
  return true;
}

function verifyWritable(obj, name, verifyProp, value) {
  var desc;
  if (verifyProp === undefined) {
    desc = Object.getOwnPropertyDescriptor(obj, name);
    if (desc === undefined) {
      throw "Expected obj[" + String(name) + "] to be writable.";
    }
    if (desc.writable !== true) {
      throw "Expected obj[" + String(name) + "] to be writable.";
    }
  }
  if (!__porfIsWritable(obj, name, verifyProp, value)) {
    throw "Expected obj[" + String(name) + "] to be writable.";
  }
  return true;
}

function verifyNotEnumerable(obj, name) {
  var desc = Object.getOwnPropertyDescriptor(obj, name);
  if (desc === undefined || desc.enumerable !== false) {
    throw "Expected obj[" + String(name) + "] NOT to be enumerable.";
  }
  return true;
}

function verifyEnumerable(obj, name) {
  var desc = Object.getOwnPropertyDescriptor(obj, name);
  if (desc === undefined || desc.enumerable !== true) {
    throw "Expected obj[" + String(name) + "] to be enumerable.";
  }
  return true;
}

function verifyConfigurable(obj, name) {
  var desc = Object.getOwnPropertyDescriptor(obj, name);
  if (desc === undefined || desc.configurable !== true) {
    throw "Expected obj[" + String(name) + "] to be configurable.";
  }
  return true;
}

function verifyNotConfigurable(obj, name) {
  var desc = Object.getOwnPropertyDescriptor(obj, name);
  if (desc === undefined || desc.configurable !== false) {
    throw "Expected obj[" + String(name) + "] NOT to be configurable.";
  }
  return true;
}

function verifyEqualTo(obj, name, value) {
  if (obj[name] !== value) {
    throw "Expected obj[" + String(name) + "] to equal value.";
  }
}

function verifyCallableProperty(obj, name, functionName, functionLength, desc) {
  var value = obj[name];
  if (typeof value !== "function") {
    throw "Expected callable property";
  }
  verifyProperty(obj, name, desc || {
    value: value,
    writable: true,
    enumerable: false,
    configurable: true
  });
  verifyProperty(value, "length", {
    value: functionLength,
    writable: false,
    enumerable: false,
    configurable: true
  });
}

var verifyPrimordialCallableProperty = verifyCallableProperty;
