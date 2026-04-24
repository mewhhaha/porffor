// Portions of this file are adapted from Test262 (https://github.com/tc39/test262)
// Test262 is BSD-3-Clause licensed; see the upstream LICENSE file
//
// wasm-aot compiles all harness functions eagerly today. Keep host shims tiny so
// unsupported host capabilities fail when called instead of blocking unrelated tests.

/// assert.js
function __porfAssertIsSameValue(a, b) {
  if (a === b) {
    return a !== 0 || 1 / a === 1 / b;
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

var assert = {
  _isSameValue: __porfAssertIsSameValue,
  _toString: __porfAssertToString,
  sameValue: function (actual, expected, message) {
    if (actual === expected && (actual !== 0 || 1 / actual === 1 / expected)) {
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
  },
  notSameValue: function (actual, unexpected, message) {
    if (actual === unexpected && (actual !== 0 || 1 / actual === 1 / unexpected)) {
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
  },
  throws: function () {
    __porfAssertUnsupported('assert.throws');
  },
  compareArray: function () {
    __porfAssertUnsupported('assert.compareArray');
  }
};

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
