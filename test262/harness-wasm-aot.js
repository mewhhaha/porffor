// Portions of this file are adapted from Test262 (https://github.com/tc39/test262)
// Test262 is BSD-3-Clause licensed; see the upstream LICENSE file
//
// wasm-aot compiles all harness functions eagerly today. Keep host shims tiny so
// unsupported host capabilities fail when called instead of blocking unrelated tests.

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
assert.throws = __porfAssertThrows;
assert.compareArray = function () {
    __porfAssertUnsupported('assert.compareArray');
};

/// sta-preamble.js
function Test262Error(message) {
}

function $DONOTEVALUATE() {
  throw 'Test262: This statement should not be evaluated.';
}

/// isConstructor.js
var isConstructor = __porfIsConstructor;

/// sta.js
function Test262Error(message) {
}

function $DONOTEVALUATE() {
  throw 'Test262: This statement should not be evaluated.';
}

function __porfUnsupportedHost(name) {
  throw name + ' unsupported in wasm-aot host harness';
}

var $262 = {
  global: globalThis,
  AbstractModuleSource: undefined,
  IsHTMLDDA: undefined,
  gc: function () {
    gc();
  },
  detachArrayBuffer: function () {
    __porfUnsupportedHost('detachArrayBuffer');
  },
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
    start: function () {
      __porfUnsupportedHost('agent.start');
    },
    broadcast: function () {
      __porfUnsupportedHost('agent.broadcast');
    },
    receiveBroadcast: function () {
      __porfUnsupportedHost('agent.receiveBroadcast');
    },
    report: function () {
      __porfUnsupportedHost('agent.report');
    },
    getReport: function () {
      __porfUnsupportedHost('agent.getReport');
    },
    sleep: function () {},
    monotonicNow: function () {
      return 0;
    },
    leaving: function () {}
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

function verifyNotWritable(obj, name) {
  var desc = Object.getOwnPropertyDescriptor(obj, name);
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
  return true;
}

function verifyWritable(obj, name) {
  var desc = Object.getOwnPropertyDescriptor(obj, name);
  if (desc === undefined || desc.writable !== true) {
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

/// testTypedArray.js
var floatArrayConstructors = [];
var nonClampedIntArrayConstructors = [];
var intArrayConstructors = [];
var typedArrayConstructors = [];
var bigIntArrayConstructors = [];
var allTypedArrayConstructors = typedArrayConstructors;
var TypedArray = Object.getPrototypeOf(Int8Array);
var nonAtomicsFriendlyTypedArrayConstructors = [];

function testWithTypedArrayConstructors(f, selected) {
  f(Float64Array);
  f(Float32Array);
  f(Int32Array);
  f(Int16Array);
  f(Int8Array);
  f(Uint32Array);
  f(Uint16Array);
  f(Uint8Array);
  f(Uint8ClampedArray);
}

function testWithAllTypedArrayConstructors(f, selected) {
  testWithTypedArrayConstructors(f);
}

function testWithBigIntTypedArrayConstructors(f, selected) {
}

function testWithAtomicsFriendlyTypedArrayConstructors(f) {
  testWithTypedArrayConstructors(f, nonClampedIntArrayConstructors);
}

function testWithNonAtomicsFriendlyTypedArrayConstructors(f) {
  testWithTypedArrayConstructors(f, nonAtomicsFriendlyTypedArrayConstructors);
}
