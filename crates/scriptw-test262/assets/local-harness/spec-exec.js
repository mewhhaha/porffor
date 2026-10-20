// Portions of this file are adapted from Test262 (https://github.com/tc39/test262)
// Test262 is BSD-3-Clause licensed; see the upstream LICENSE file

/// sta.js
// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Provides both:

    - An error class to avoid false positives when testing for thrown exceptions
    - A function to explicitly throw an exception using the Test262Error class
defines: [Test262Error, $DONOTEVALUATE]
---*/


function Test262Error(message) {
  if (!(this instanceof Test262Error)) return new Test262Error(message);
  this.message = message || "";
}

Test262Error.prototype.toString = function () {
  return "Test262Error: " + this.message;
};

Test262Error.thrower = function (message) {
  throw new Test262Error(message);
};

function $DONOTEVALUATE() {
  throw "Test262: This statement should not be evaluated.";
}

// Scriptw SpecExec host adapter; separate from the upstream sta.js resource helper.
function __scriptwUnsupportedHost(name) {
  return __scriptwUnsupportedHostCapability(name);
}

function __scriptwUnsupportedAgentMethod(name) {
  return function () {
    __scriptwUnsupportedHost('agent.' + name);
  };
}

var __scriptwHost262 = typeof globalThis.$262 === 'object' && globalThis.$262 ? globalThis.$262 : null;

var $262 = {
  global: globalThis,
  AbstractModuleSource: __scriptwHost262 && __scriptwHost262.AbstractModuleSource
    ? __scriptwHost262.AbstractModuleSource
    : function AbstractModuleSource() {
        __scriptwUnsupportedHost('AbstractModuleSource');
      },
  IsHTMLDDA: __scriptwHost262 && typeof __scriptwHost262.IsHTMLDDA === 'function'
    ? __scriptwHost262.IsHTMLDDA
    : function IsHTMLDDA() {
        __scriptwUnsupportedHost('IsHTMLDDA');
      },
  gc() {
    if (typeof gc === 'function') {
      return gc();
    }
    __scriptwUnsupportedHost('gc');
  },
  detachArrayBuffer(buffer) {
    if (typeof __scriptwDetachArrayBuffer === 'function') {
      return __scriptwDetachArrayBuffer(buffer);
    }
    __scriptwUnsupportedHost('detachArrayBuffer');
  },
  getGlobal(name) {
    return globalThis[name];
  },
  evalScript(code) {
    if (typeof __scriptwEvalScript === 'function') {
      return __scriptwEvalScript(code);
    }
    __scriptwUnsupportedHost('evalScript');
  },
  createRealm() {
    if (typeof __scriptwCreateRealm === 'function') {
      return __scriptwCreateRealm();
    }
    __scriptwUnsupportedHost('createRealm');
  },
  destroy() {},
  agent: __scriptwHost262 && __scriptwHost262.agent ? __scriptwHost262.agent : {
    start: __scriptwUnsupportedAgentMethod('start'),
    broadcast: __scriptwUnsupportedAgentMethod('broadcast'),
    receiveBroadcast: __scriptwUnsupportedAgentMethod('receiveBroadcast'),
    report: __scriptwUnsupportedAgentMethod('report'),
    getReport: __scriptwUnsupportedAgentMethod('getReport'),
    sleep: __scriptwUnsupportedAgentMethod('sleep'),
    monotonicNow: __scriptwUnsupportedAgentMethod('monotonicNow'),
    leaving: __scriptwUnsupportedAgentMethod('leaving')
  }
};
