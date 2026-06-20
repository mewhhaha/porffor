var arrayTarget = new Proxy([42], {});
var arrayProxy = new Proxy(arrayTarget, {
  getOwnPropertyDescriptor: undefined
});

var plainTarget = new Proxy({ foo: 1 }, {});
var plainProxy = new Proxy(plainTarget, {
  getOwnPropertyDescriptor: null
});

var customDescriptor = {
  get: function() { return 7; },
  set: function(_value) {},
  enumerable: false,
  configurable: true
};
var customTarget = new Proxy({}, {
  getOwnPropertyDescriptor: function(_target, key) {
    if (key === "foo") return customDescriptor;
  }
});
var customProxy = new Proxy(customTarget, {
  getOwnPropertyDescriptor: null
});
var regExpTarget = new Proxy(/(?:)/, {});
var regExpProxy = new Proxy(regExpTarget, {
  getOwnPropertyDescriptor: undefined
});
var stringTarget = new Proxy(new String("str"), {});
var stringProxy = new Proxy(stringTarget, {});
var functionTarget = new Proxy(function() {}, {});
var functionProxy = new Proxy(functionTarget, {});

var arrayIndex = Object.getOwnPropertyDescriptor(arrayProxy, "0");
var arrayLength = Object.getOwnPropertyDescriptor(arrayProxy, "length");
var plainMissing = Object.getOwnPropertyDescriptor(plainProxy, "bar");
var plainFoo = Object.getOwnPropertyDescriptor(plainProxy, "foo");
var customFoo = Object.getOwnPropertyDescriptor(customProxy, "foo");
var regExpLastIndex = Object.getOwnPropertyDescriptor(regExpProxy, "lastIndex");
var stringIndex = Object.getOwnPropertyDescriptor(stringProxy, "0");
var stringLength = Object.getOwnPropertyDescriptor(stringProxy, "length");
var functionPrototype = Object.getOwnPropertyDescriptor(functionProxy, "prototype");

arrayIndex.value === 42 &&
  arrayProxy["0"] === 42 &&
  arrayIndex.writable === true &&
  arrayIndex.enumerable === true &&
  arrayIndex.configurable === true &&
  arrayLength.value === 1 &&
  arrayProxy.length === 1 &&
  arrayLength.enumerable === false &&
  arrayLength.configurable === false &&
  plainMissing === undefined &&
  plainFoo.value === 1 &&
  plainProxy.foo === 1 &&
  plainFoo.writable === true &&
  plainFoo.enumerable === true &&
  plainFoo.configurable === true &&
  customFoo.get === customDescriptor.get &&
  customFoo.set === customDescriptor.set &&
  customFoo.enumerable === false &&
  customFoo.configurable === true &&
  regExpLastIndex.value === 0 &&
  regExpProxy.lastIndex === 0 &&
  regExpLastIndex.writable === true &&
  regExpLastIndex.enumerable === false &&
  regExpLastIndex.configurable === false &&
  stringIndex.value === "s" &&
  stringProxy["0"] === "s" &&
  stringIndex.writable === false &&
  stringIndex.enumerable === true &&
  stringIndex.configurable === false &&
  stringLength.value === 3 &&
  stringProxy.length === 3 &&
  stringLength.writable === false &&
  stringLength.enumerable === false &&
  stringLength.configurable === false &&
  functionPrototype !== undefined &&
  functionPrototype.writable === true &&
  functionPrototype.enumerable === false &&
  functionPrototype.configurable === false;
