function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function captureThrow(action, label) {
  try {
    action();
  } catch (error) {
    return error;
  }
  throw new Error(label + " did not throw");
}

function expectEntryTypeError(action, expectedMessage, label) {
  var caught = captureThrow(action, label);
  assert(
    Object.getPrototypeOf(caught) === TypeError.prototype,
    label + " wrong TypeError prototype: " + caught.name + ": " + caught.message
  );
  assert(caught.message === expectedMessage, label + " wrong message");
}

// Thirteen initialized captured bindings put lexical-environment slot 12 at
// the same byte offset as a function object's cached TypeError prototype.
var realmCapture0 = {};
var realmCapture1 = {};
var realmCapture2 = {};
var realmCapture3 = {};
var realmCapture4 = {};
var realmCapture5 = {};
var realmCapture6 = {};
var realmCapture7 = {};
var realmCapture8 = {};
var realmCapture9 = {};
var realmCapture10 = {};
var realmCapture11 = {};
var realmCapture12 = {};

function retainWideLexicalEnvironment() {
  return (
    realmCapture0 !== realmCapture1 &&
    realmCapture1 !== realmCapture2 &&
    realmCapture2 !== realmCapture3 &&
    realmCapture3 !== realmCapture4 &&
    realmCapture4 !== realmCapture5 &&
    realmCapture5 !== realmCapture6 &&
    realmCapture6 !== realmCapture7 &&
    realmCapture7 !== realmCapture8 &&
    realmCapture8 !== realmCapture9 &&
    realmCapture9 !== realmCapture10 &&
    realmCapture10 !== realmCapture11 &&
    realmCapture11 !== realmCapture12
  );
}

assert(retainWideLexicalEnvironment(), "wide lexical environment capture");

var iteratorMethodReceiver;
var iteratorMethodArgumentCount = -1;
var nextReceiver;
var nextArgumentCount = -1;
var nextLoadCount = 0;
var nextCallCount = 0;
var bodyCount = 0;
var closeCount = 0;
var nextSentinel = { source: "next apply trap" };
var proxyIterator = {
  return: function () {
    closeCount += 1;
    return {};
  },
};
var proxyNext = new Proxy(
  function () {
    throw new Error("callable Proxy next target ran directly");
  },
  {
    apply: function (target, receiver, argumentsList) {
      nextReceiver = receiver;
      nextArgumentCount = argumentsList.length;
      nextCallCount += 1;
      if (nextCallCount === 1) return { value: 41, done: false };
      throw nextSentinel;
    },
  }
);
Object.defineProperty(proxyIterator, "next", {
  configurable: true,
  get: function () {
    nextLoadCount += 1;
    return proxyNext;
  },
});

var proxyIterable = {};
proxyIterable[Symbol.iterator] = new Proxy(
  function () {
    throw new Error("callable Proxy iterator target ran directly");
  },
  {
    apply: function (target, receiver, argumentsList) {
      iteratorMethodReceiver = receiver;
      iteratorMethodArgumentCount = argumentsList.length;
      return proxyIterator;
    },
  }
);

var nextError = captureThrow(function () {
  for (var value of proxyIterable) {
    assert(value === 41, "callable Proxy next value");
    bodyCount += 1;
  }
}, "callable Proxy next");
assert(nextError === nextSentinel, "callable Proxy next throw identity");
assert(iteratorMethodReceiver === proxyIterable, "callable Proxy iterator receiver");
assert(iteratorMethodArgumentCount === 0, "callable Proxy iterator argument count");
assert(nextReceiver === proxyIterator, "callable Proxy next receiver");
assert(nextArgumentCount === 0, "callable Proxy next argument count");
assert(nextLoadCount === 1, "callable Proxy next load count");
assert(nextCallCount === 2, "callable Proxy next call count");
assert(bodyCount === 1, "callable Proxy body count");
assert(closeCount === 0, "abrupt callable Proxy next must not close");

var iteratorMethodSentinel = { source: "iterator apply trap" };
var throwingIteratorMethod = {};
throwingIteratorMethod[Symbol.iterator] = new Proxy(function () {}, {
  apply: function () {
    throw iteratorMethodSentinel;
  },
});
var iteratorMethodError = captureThrow(function () {
  for (var value of throwingIteratorMethod) {
    throw new Error("throwing iterator method entered its body");
  }
}, "callable Proxy iterator method");
assert(
  iteratorMethodError === iteratorMethodSentinel,
  "callable Proxy iterator throw identity"
);

var primitiveNonCallableIteratorMethod = {};
primitiveNonCallableIteratorMethod[Symbol.iterator] = 0;
expectEntryTypeError(
  function () {
    for (var value of primitiveNonCallableIteratorMethod) {
      throw new Error("primitive non-callable iterator entered its body");
    }
  },
  "for-of target is not iterable",
  "primitive non-callable iterator method"
);

var nonCallableIteratorMethod = {};
nonCallableIteratorMethod[Symbol.iterator] = new Proxy({}, {});
expectEntryTypeError(
  function () {
    for (var value of nonCallableIteratorMethod) {
      throw new Error("non-callable Proxy iterator entered its body");
    }
  },
  "for-of target is not iterable",
  "non-callable Proxy iterator method"
);

var nonCallableNextCloseCount = 0;
var nonCallableNextIterable = {};
nonCallableNextIterable[Symbol.iterator] = function () {
  return {
    next: new Proxy({}, {}),
    return: function () {
      nonCallableNextCloseCount += 1;
      return {};
    },
  };
};
expectEntryTypeError(
  function () {
    for (var value of nonCallableNextIterable) {
      throw new Error("non-callable Proxy next entered its body");
    }
  },
  "for-of iterator next must be callable",
  "non-callable Proxy next"
);
assert(nonCallableNextCloseCount === 0, "non-callable Proxy next must not close");

var revokedIteratorMethod = Proxy.revocable(function () {
  throw new Error("revoked iterator method target ran");
}, {});
revokedIteratorMethod.revoke();
assert(typeof revokedIteratorMethod.proxy === "function", "revoked iterator method is callable");
var revokedMethodIterable = {};
revokedMethodIterable[Symbol.iterator] = revokedIteratorMethod.proxy;
var revokedMethodError = captureThrow(function () {
  for (var value of revokedMethodIterable) {
    throw new Error("revoked iterator method entered its body");
  }
}, "revoked iterator method");
assert(revokedMethodError instanceof TypeError, "revoked iterator method TypeError");
assert(
  revokedMethodError.message !== "for-of target is not iterable",
  "revoked iterator method was replaced by the protocol diagnostic"
);

var revokedNext = Proxy.revocable(function () {
  throw new Error("revoked next target ran");
}, {});
revokedNext.revoke();
assert(typeof revokedNext.proxy === "function", "revoked next is callable");
var revokedNextCloseCount = 0;
var revokedNextIterable = {};
revokedNextIterable[Symbol.iterator] = function () {
  return {
    next: revokedNext.proxy,
    return: function () {
      revokedNextCloseCount += 1;
      return {};
    },
  };
};
var revokedNextError = captureThrow(function () {
  for (var value of revokedNextIterable) {
    throw new Error("revoked next entered its body");
  }
}, "revoked next");
assert(revokedNextError instanceof TypeError, "revoked next TypeError");
assert(
  revokedNextError.message !== "for-of iterator next must be callable",
  "revoked next was replaced by the protocol diagnostic"
);
assert(revokedNextCloseCount === 0, "revoked next must not close");

true;
