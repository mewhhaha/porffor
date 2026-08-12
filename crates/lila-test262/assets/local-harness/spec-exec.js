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

function __lilaUnsupportedHost(name) {
  throw new Test262Error('local harness host ' + name + ' unsupported');
}

function __lilaUnsupportedAgentMethod(name) {
  return function () {
    __lilaUnsupportedHost('agent.' + name);
  };
}

var __lilaHost262 = typeof globalThis.$262 === 'object' && globalThis.$262 ? globalThis.$262 : null;

var $262 = {
  global: globalThis,
  AbstractModuleSource: __lilaHost262 && __lilaHost262.AbstractModuleSource
    ? __lilaHost262.AbstractModuleSource
    : function AbstractModuleSource() {
        __lilaUnsupportedHost('AbstractModuleSource');
      },
  IsHTMLDDA: __lilaHost262 && typeof __lilaHost262.IsHTMLDDA === 'function'
    ? __lilaHost262.IsHTMLDDA
    : function IsHTMLDDA() {
        __lilaUnsupportedHost('IsHTMLDDA');
      },
  gc() {
    if (typeof gc === 'function') {
      return gc();
    }
    __lilaUnsupportedHost('gc');
  },
  detachArrayBuffer(buffer) {
    if (typeof __lilaDetachArrayBuffer === 'function') {
      return __lilaDetachArrayBuffer(buffer);
    }
    __lilaUnsupportedHost('detachArrayBuffer');
  },
  getGlobal(name) {
    return globalThis[name];
  },
  evalScript(code) {
    if (typeof __lilaEvalScript === 'function') {
      return __lilaEvalScript(code);
    }
    __lilaUnsupportedHost('evalScript');
  },
  createRealm() {
    if (typeof __lilaCreateRealm === 'function') {
      return __lilaCreateRealm();
    }
    __lilaUnsupportedHost('createRealm');
  },
  destroy() {},
  agent: __lilaHost262 && __lilaHost262.agent ? __lilaHost262.agent : {
    start: __lilaUnsupportedAgentMethod('start'),
    broadcast: __lilaUnsupportedAgentMethod('broadcast'),
    receiveBroadcast: __lilaUnsupportedAgentMethod('receiveBroadcast'),
    report: __lilaUnsupportedAgentMethod('report'),
    getReport: __lilaUnsupportedAgentMethod('getReport'),
    sleep: __lilaUnsupportedAgentMethod('sleep'),
    monotonicNow: __lilaUnsupportedAgentMethod('monotonicNow'),
    leaving: __lilaUnsupportedAgentMethod('leaving')
  }
};
