function assert(condition, message) {
  if (!condition) throw message;
}

function capture(operation) {
  try {
    operation();
  } catch (error) {
    return error;
  }
  return undefined;
}

var getTarget = {};
var getReceiver = {};
var getProxy;
getProxy = new Proxy(getTarget, {
  get: function (target, key, receiver) {
    assert(target === getTarget, "Reflect.get trap target");
    assert(key === "value", "Reflect.get trap key");
    getReceiver = receiver;
    return 17;
  },
});

assert(Reflect.get(getProxy, "value", undefined) === 17, "explicit get result");
assert(getReceiver === undefined, "explicit get receiver");
assert(Reflect.get(getProxy, "value") === 17, "omitted get result");
assert(getReceiver === getProxy, "omitted get receiver");

var setTarget = {};
var setReceiver = {};
var setProxy;
setProxy = new Proxy(setTarget, {
  set: function (target, key, value, receiver) {
    assert(target === setTarget, "Reflect.set trap target");
    assert(key === "value", "Reflect.set trap key");
    assert(value === 23, "Reflect.set trap value");
    setReceiver = receiver;
    return true;
  },
});

assert(Reflect.set(setProxy, "value", 23, undefined), "explicit set result");
assert(setReceiver === undefined, "explicit set receiver");
assert(Reflect.set(setProxy, "value", 23), "omitted set result");
assert(setReceiver === setProxy, "omitted set receiver");

var ordinaryExplicitUndefinedTarget = {};
assert(
  Reflect.set(ordinaryExplicitUndefinedTarget, "value", 29, undefined) === false,
  "explicit undefined ordinary receiver result"
);
assert(
  !Object.prototype.hasOwnProperty.call(ordinaryExplicitUndefinedTarget, "value"),
  "explicit undefined ordinary receiver mutation"
);
assert(
  Reflect.set(ordinaryExplicitUndefinedTarget, "value", 31) === true,
  "omitted ordinary receiver result"
);
assert(
  ordinaryExplicitUndefinedTarget.value === 31,
  "omitted ordinary receiver mutation"
);

var constructTrapCalls = 0;
var constructNewTarget;
function ConstructTarget() {}
var constructProxy;
constructProxy = new Proxy(ConstructTarget, {
  construct: function (target, argumentsList, newTarget) {
    constructTrapCalls += 1;
    assert(target === ConstructTarget, "Reflect.construct trap target");
    assert(argumentsList.length === 0, "Reflect.construct trap arguments");
    constructNewTarget = newTarget;
    return {};
  },
});

var explicitUndefinedNewTargetError = capture(function () {
  Reflect.construct(constructProxy, [], undefined);
});
assert(
  explicitUndefinedNewTargetError instanceof TypeError,
  "explicit undefined newTarget error"
);
assert(constructTrapCalls === 0, "explicit undefined invoked construct trap");

Reflect.construct(constructProxy, []);
assert(constructTrapCalls === 1, "omitted newTarget construct count");
assert(constructNewTarget === constructProxy, "omitted newTarget identity");

true;
