// Portions of this file are adapted from Test262 (https://github.com/tc39/test262)
// Test262 is BSD-3-Clause licensed; see the upstream LICENSE file
//
// Materialization activates the realm boundary only for cases that can use it.
// Keep the inactive template fail-loud so an unrecognized access cannot silently
// run with the wrong host semantics.

function __scriptwUnsupportedHost(name) {
  throw name + ' unsupported in wasm-aot host harness';
}

var $262 = {
  global: undefined,
  AbstractModuleSource: __scriptwGetAbstractModuleSource(),
  // Must be a real [[IsHTMLDDA]] exotic object, not an ordinary function that
  // returns null: the annexB `emulates-undefined` tests observe ToBoolean,
  // `typeof`, IsLooselyEqual and the absence of an own `prototype` property.
  // `__scriptwCreateHTMLDDA()` mints one function object carrying the
  // FUNCTION_FLAG_IS_HTMLDDA flag that every one of those paths consults.
  IsHTMLDDA: __scriptwCreateHTMLDDA(),
  gc: function () {
    gc();
  },
  detachArrayBuffer: __scriptwDetachArrayBuffer,
  // Compiler-only Test262 capability. Lowering resolves this function value
  // to DynamicSourceIntrinsic::RealmEvalScript and rejects its invocation with
  // typed T13 accounting before Wasm planning.
  evalScript: __scriptwRealmEvalScript,
  createRealm: function () {
    __scriptwUnsupportedHost('createRealm');
  },
  destroy: function () {},
  getGlobal: function () {
    __scriptwUnsupportedHost('getGlobal');
  },
  agent: {
    start: function (source) {
      return __scriptwAgentStart(source);
    },
    broadcast: function (sab, id) {
      return __scriptwAgentBroadcast(sab, id);
    },
    receiveBroadcast: function (callback) {
      var message = __scriptwAgentReceiveBroadcast();
      return callback(message[0], message[1]);
    },
    report: function (value) {
      return __scriptwAgentReport(value);
    },
    getReport: function () {
      return __scriptwAgentGetReport();
    },
    sleep: function (milliseconds) {
      return __scriptwAgentSleep(milliseconds);
    },
    monotonicNow: function () {
      return __scriptwAgentMonotonicNow();
    },
    leaving: function () {
      return __scriptwAgentLeaving();
    }
  }
};
