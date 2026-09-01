function assert(condition, label) {
  if (!condition) throw new Error(label);
}

function captureThrow(action, label) {
  try {
    action();
  } catch (error) {
    return error;
  }
  throw new Error(label + " did not throw");
}

function assertTypeErrorPrototype(action, expectedPrototype, label) {
  var error = captureThrow(action, label);
  assert(Object.getPrototypeOf(error) === expectedPrototype, label + " prototype");
}

var invokeRevokedFromWideEntryEnvironment = (function () {
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

  return function (proxy) {
    assert(
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
        realmCapture11 !== realmCapture12,
      "wide entry lexical environment"
    );
    try {
      proxy();
    } catch (error) {
      return Object.getPrototypeOf(error);
    }
    return null;
  };
})();

var other = __lilaCreateRealm().global;
assert(TypeError.prototype !== other.TypeError.prototype, "distinct TypeError prototypes");
assert(Array.prototype !== other.Array.prototype, "distinct Array prototypes");

var callableRevocation = Proxy.revocable(function () {}, {});
var revokedCallableProxy = callableRevocation.proxy;
callableRevocation.revoke();

assertTypeErrorPrototype(
  function () {
    revokedCallableProxy();
  },
  TypeError.prototype,
  "entry revoked call"
);
assertTypeErrorPrototype(
  function () {
    other.Reflect.apply(revokedCallableProxy, null, []);
  },
  other.TypeError.prototype,
  "created Realm revoked call"
);
assert(
  invokeRevokedFromWideEntryEnvironment(revokedCallableProxy) === TypeError.prototype,
  "direct wide lexical environment Realm"
);
assert(
  other.Reflect.apply(invokeRevokedFromWideEntryEnvironment, null, [revokedCallableProxy]) ===
    TypeError.prototype,
  "borrowed call into wide lexical environment Realm"
);

function RevokedConstructorTarget() {}

var constructorRevocation = Proxy.revocable(RevokedConstructorTarget, {});
var revokedConstructableProxy = constructorRevocation.proxy;
constructorRevocation.revoke();

assertTypeErrorPrototype(
  function () {
    new revokedConstructableProxy();
  },
  TypeError.prototype,
  "entry revoked construct"
);
assertTypeErrorPrototype(
  function () {
    other.Reflect.construct(revokedConstructableProxy, []);
  },
  other.TypeError.prototype,
  "created Realm revoked construct"
);

var callableHandlerRevocation = Proxy.revocable({}, {});
var callableWithRevokedHandler = new Proxy(function () {}, callableHandlerRevocation.proxy);
callableHandlerRevocation.revoke();

assertTypeErrorPrototype(
  function () {
    callableWithRevokedHandler();
  },
  TypeError.prototype,
  "entry revoked call handler"
);
assertTypeErrorPrototype(
  function () {
    other.Reflect.apply(callableWithRevokedHandler, null, []);
  },
  other.TypeError.prototype,
  "created Realm revoked call handler"
);

var constructorHandlerRevocation = Proxy.revocable({}, {});
var constructableWithRevokedHandler = new Proxy(
  function () {},
  constructorHandlerRevocation.proxy
);
constructorHandlerRevocation.revoke();

assertTypeErrorPrototype(
  function () {
    new constructableWithRevokedHandler();
  },
  TypeError.prototype,
  "entry revoked construct handler"
);
assertTypeErrorPrototype(
  function () {
    other.Reflect.construct(constructableWithRevokedHandler, []);
  },
  other.TypeError.prototype,
  "created Realm revoked construct handler"
);

var accessorRevocation = Proxy.revocable(function () {}, {});
var callableAccessorHandler = {};
var constructorAccessorHandler = {};
Object.defineProperty(callableAccessorHandler, "apply", {
  get: accessorRevocation.proxy,
});
Object.defineProperty(constructorAccessorHandler, "construct", {
  get: accessorRevocation.proxy,
});
var callableWithRevokedAccessor = new Proxy(function () {}, callableAccessorHandler);
var constructableWithRevokedAccessor = new Proxy(function () {}, constructorAccessorHandler);
accessorRevocation.revoke();

assertTypeErrorPrototype(
  function () {
    callableWithRevokedAccessor();
  },
  TypeError.prototype,
  "entry revoked apply accessor"
);
assertTypeErrorPrototype(
  function () {
    other.Reflect.apply(callableWithRevokedAccessor, null, []);
  },
  other.TypeError.prototype,
  "created Realm revoked apply accessor"
);
assertTypeErrorPrototype(
  function () {
    new constructableWithRevokedAccessor();
  },
  TypeError.prototype,
  "entry revoked construct accessor"
);
assertTypeErrorPrototype(
  function () {
    other.Reflect.construct(constructableWithRevokedAccessor, []);
  },
  other.TypeError.prototype,
  "created Realm revoked construct accessor"
);

var applyArgumentLists = [];
var callableApplyProxy = new Proxy(
  function () {
    throw new Error("apply target ran");
  },
  {
    apply: function (target, thisArgument, argumentsList) {
      applyArgumentLists.push(argumentsList);
      return argumentsList[0] + argumentsList[1];
    },
  }
);
var entryApplyInput = [19, 23];
var createdApplyInput = [20, 22];

assert(Reflect.apply(callableApplyProxy, null, entryApplyInput) === 42, "entry apply result");
assert(
  other.Reflect.apply(callableApplyProxy, null, createdApplyInput) === 42,
  "created Realm apply result"
);
assert(applyArgumentLists.length === 2, "apply trap call count");
assert(applyArgumentLists[0] !== entryApplyInput, "entry apply arguments are fresh");
assert(applyArgumentLists[1] !== createdApplyInput, "created Realm apply arguments are fresh");
assert(applyArgumentLists[0] !== applyArgumentLists[1], "apply argument lists are distinct");
assert(
  Object.getPrototypeOf(applyArgumentLists[0]) === Array.prototype,
  "entry apply arguments prototype"
);
assert(
  Object.getPrototypeOf(applyArgumentLists[1]) === other.Array.prototype,
  "created Realm apply arguments prototype"
);

function ConstructTarget() {
  throw new Error("construct target ran");
}

var constructArgumentLists = [];
var callableConstructProxy = new Proxy(ConstructTarget, {
  construct: function (target, argumentsList, newTarget) {
    constructArgumentLists.push(argumentsList);
    return { marker: argumentsList[0] };
  },
});
var entryConstructInput = [41];
var createdConstructInput = [42];
var entryConstructed = Reflect.construct(callableConstructProxy, entryConstructInput);
var createdConstructed = other.Reflect.construct(callableConstructProxy, createdConstructInput);

assert(entryConstructed.marker === 41, "entry construct result");
assert(createdConstructed.marker === 42, "created Realm construct result");
assert(constructArgumentLists.length === 2, "construct trap call count");
assert(constructArgumentLists[0] !== entryConstructInput, "entry construct arguments are fresh");
assert(
  constructArgumentLists[1] !== createdConstructInput,
  "created Realm construct arguments are fresh"
);
assert(
  constructArgumentLists[0] !== constructArgumentLists[1],
  "construct argument lists are distinct"
);
assert(
  Object.getPrototypeOf(constructArgumentLists[0]) === Array.prototype,
  "entry construct arguments prototype"
);
assert(
  Object.getPrototypeOf(constructArgumentLists[1]) === other.Array.prototype,
  "created Realm construct arguments prototype"
);

var nonCallableApplyProxy = new Proxy(function () {}, { apply: 0 });
assertTypeErrorPrototype(
  function () {
    nonCallableApplyProxy();
  },
  TypeError.prototype,
  "entry noncallable apply trap"
);
assertTypeErrorPrototype(
  function () {
    other.Reflect.apply(nonCallableApplyProxy, null, []);
  },
  other.TypeError.prototype,
  "created Realm noncallable apply trap"
);

var primitiveConstructResultProxy = new Proxy(function () {}, {
  construct: function () {
    return 0;
  },
});
assertTypeErrorPrototype(
  function () {
    new primitiveConstructResultProxy();
  },
  TypeError.prototype,
  "entry primitive construct result"
);
assertTypeErrorPrototype(
  function () {
    other.Reflect.construct(primitiveConstructResultProxy, []);
  },
  other.TypeError.prototype,
  "created Realm primitive construct result"
);

var nestedTrapArgumentLists = [];
var callableProxyApplyTrap = new Proxy(
  function () {
    throw new Error("nested apply trap target ran");
  },
  {
    apply: function (target, thisArgument, argumentsList) {
      nestedTrapArgumentLists.push(argumentsList);
      return argumentsList[2][0];
    },
  }
);
var nestedDispatchProxy = new Proxy(function () {}, { apply: callableProxyApplyTrap });
var nestedCreatedInput = ["created"];

assert(nestedDispatchProxy("entry") === "entry", "entry nested apply result");
assert(
  other.Reflect.apply(nestedDispatchProxy, null, nestedCreatedInput) === "created",
  "created Realm nested apply result"
);
assert(nestedTrapArgumentLists.length === 2, "nested apply trap call count");
assert(
  Object.getPrototypeOf(nestedTrapArgumentLists[0]) === Array.prototype,
  "entry recursive helper arguments prototype"
);
assert(
  Object.getPrototypeOf(nestedTrapArgumentLists[1]) === other.Array.prototype,
  "created Realm recursive helper arguments prototype"
);
assert(
  Object.getPrototypeOf(nestedTrapArgumentLists[0][2]) === Array.prototype,
  "entry nested trap arguments prototype"
);
assert(
  Object.getPrototypeOf(nestedTrapArgumentLists[1][2]) === other.Array.prototype,
  "created Realm nested trap arguments prototype"
);
assert(
  nestedTrapArgumentLists[1][2] !== nestedCreatedInput,
  "created Realm nested trap arguments are fresh"
);

true;
