// Direct identifier calls selected by a with Object Environment Record retain
// that record's binding object as `this`. Callee Reference evaluation must
// finish before argument evaluation, and a fallback call must not inherit a
// with receiver.
var globalObject = this;
var selectedThis;
var selectedGetterThis;
var selectedResult;
var trace = "";

var selectedTarget = {};
Object.defineProperty(selectedTarget, "method", {
  configurable: true,
  get: function() {
    trace += "g";
    selectedGetterThis = this;
    delete this.method;
    return function(value) {
      trace += "c";
      selectedThis = this;
      return value;
    };
  }
});
var selectedProxy = new Proxy(selectedTarget, {
  has: function(target, key) {
    if (key === "method") trace += "h";
    return Reflect.has(target, key);
  },
  get: function(target, key, receiver) {
    if (key === Symbol.unscopables) trace += "u";
    if (key === "method") trace += "r";
    return Reflect.get(target, key, receiver);
  }
});

function selectedArgument() {
  trace += "a";
  return 41;
}

with (selectedProxy) {
  selectedResult = method(selectedArgument());
}

function strictFallback() {
  "use strict";
  return this;
}
function sloppyFallback() {
  return this;
}
var strictFallbackThis;
var sloppyFallbackThis;
with ({}) {
  strictFallbackThis = strictFallback();
  sloppyFallbackThis = sloppyFallback();
}

var outerThis;
var innerMethodCalls = 0;
var outerBinding = {
  nestedMethod: function() {
    outerThis = this;
    return 73;
  }
};
var innerBinding = {
  nestedMethod: function() {
    innerMethodCalls += 1;
  }
};
innerBinding[Symbol.unscopables] = { nestedMethod: true };
var nestedResult;
with (outerBinding) {
  with (innerBinding) {
    nestedResult = nestedMethod();
  }
}

var builtinShadowThis;
var builtinShadowResult;
var builtinShadow = {
  Boolean: function(value) {
    builtinShadowThis = this;
    return "shadow:" + value;
  }
};
with (builtinShadow) {
  builtinShadowResult = Boolean(1);
}
var builtinFallbackResult;
with ({}) {
  builtinFallbackResult = Boolean(0);
}

function mutableFallback() {
  return "old";
}
var fallbackMutationHasCalls = 0;
var mutatedFallbackThis = globalObject;
var fallbackMutationProxy = new Proxy({}, {
  has: function(_target, key) {
    if (key === "mutableFallback") {
      fallbackMutationHasCalls += 1;
      mutableFallback = function() {
        "use strict";
        mutatedFallbackThis = this;
        return "new";
      };
    }
    return false;
  }
});
var mutatedFallbackResult;
with (fallbackMutationProxy) {
  mutatedFallbackResult = mutableFallback();
}

trace === "huhrgac" &&
  selectedResult === 41 &&
  selectedThis === selectedProxy &&
  selectedGetterThis === selectedProxy &&
  !Reflect.has(selectedTarget, "method") &&
  strictFallbackThis === undefined &&
  sloppyFallbackThis === globalObject &&
  nestedResult === 73 &&
  outerThis === outerBinding &&
  innerMethodCalls === 0 &&
  builtinShadowResult === "shadow:1" &&
  builtinShadowThis === builtinShadow &&
  builtinFallbackResult === false &&
  fallbackMutationHasCalls === 1 &&
  mutatedFallbackResult === "new" &&
  mutatedFallbackThis === undefined;
