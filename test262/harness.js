// Portions of this file are adapted from Test262 (https://github.com/tc39/test262)
// Test262 is BSD-3-Clause licensed; see the upstream LICENSE file

/// sta.js
function Test262Error(message) {
  this.message = message || '';
  this.name = 'Test262Error';
}

Test262Error.prototype.toString = function () {
  return 'Test262Error: ' + this.message;
};

Test262Error.thrower = function (message) {
  throw new Test262Error(message);
};

function $DONOTEVALUATE() {
  throw 'Test262: This statement should not be evaluated.';
}

function __porfUnsupportedHost(name) {
  throw new Test262Error('local harness host ' + name + ' unsupported');
}

function __porfUnsupportedAgentMethod(name) {
  return function () {
    __porfUnsupportedHost('agent.' + name);
  };
}

var __porfHost262 = typeof globalThis.$262 === 'object' && globalThis.$262 ? globalThis.$262 : null;

var $262 = {
  global: globalThis,
  AbstractModuleSource: __porfHost262 && __porfHost262.AbstractModuleSource
    ? __porfHost262.AbstractModuleSource
    : function AbstractModuleSource() {
        __porfUnsupportedHost('AbstractModuleSource');
      },
  IsHTMLDDA: __porfHost262 && typeof __porfHost262.IsHTMLDDA === 'function'
    ? __porfHost262.IsHTMLDDA
    : function IsHTMLDDA() {
        __porfUnsupportedHost('IsHTMLDDA');
      },
  gc() {
    if (typeof gc === 'function') {
      return gc();
    }
    __porfUnsupportedHost('gc');
  },
  detachArrayBuffer(buffer) {
    if (typeof __porfDetachArrayBuffer === 'function') {
      return __porfDetachArrayBuffer(buffer);
    }
    __porfUnsupportedHost('detachArrayBuffer');
  },
  getGlobal(name) {
    return globalThis[name];
  },
  evalScript(code) {
    if (typeof __porfEvalScript === 'function') {
      return __porfEvalScript(code);
    }
    __porfUnsupportedHost('evalScript');
  },
  createRealm() {
    if (typeof __porfCreateRealm === 'function') {
      return __porfCreateRealm();
    }
    __porfUnsupportedHost('createRealm');
  },
  destroy() {},
  agent: __porfHost262 && __porfHost262.agent ? __porfHost262.agent : {
    start: __porfUnsupportedAgentMethod('start'),
    broadcast: __porfUnsupportedAgentMethod('broadcast'),
    receiveBroadcast: __porfUnsupportedAgentMethod('receiveBroadcast'),
    report: __porfUnsupportedAgentMethod('report'),
    getReport: __porfUnsupportedAgentMethod('getReport'),
    sleep: __porfUnsupportedAgentMethod('sleep'),
    monotonicNow: __porfUnsupportedAgentMethod('monotonicNow'),
    leaving: __porfUnsupportedAgentMethod('leaving')
  }
};
