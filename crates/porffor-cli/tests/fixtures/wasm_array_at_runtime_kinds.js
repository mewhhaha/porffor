let arrayGetterReceiver;
let inheritedArray = Array(7);
Object.defineProperty(Array.prototype, "6", {
  get: function () {
    arrayGetterReceiver = this;
    return "inherited";
  },
  configurable: true
});
let inheritedValue = inheritedArray.at(6);
delete Array.prototype[6];

let inheritedSentinel = {};
let inheritedThrow;
Object.defineProperty(Array.prototype, "6", {
  get: function () {
    throw inheritedSentinel;
  },
  configurable: true
});
try {
  inheritedArray.at(6);
} catch (error) {
  inheritedThrow = error;
}
delete Array.prototype[6];

function readArgumentsAtSix() {
  return Array.prototype.at.call(arguments, 6);
}

let ordinaryObject = { 6: "object", length: 7 };
function ordinaryFunction() {}
Object.defineProperty(ordinaryFunction, "length", { value: 7 });
ordinaryFunction[6] = "function";

let proxyLog = [];
let proxy = new Proxy({ 6: "proxy", length: 7 }, {
  get: function (target, key, receiver) {
    proxyLog.push(String(key));
    return Reflect.get(target, key, receiver);
  }
});
let proxyValue = Array.prototype.at.call(proxy, -1);

let outOfBoundsLog = [];
let outOfBoundsProxy = new Proxy({ 6: "unread", length: 7 }, {
  get: function (target, key, receiver) {
    outOfBoundsLog.push(String(key));
    return Reflect.get(target, key, receiver);
  }
});
let outOfBoundsValue = Array.prototype.at.call(outOfBoundsProxy, 7);

let proxySentinel = {};
let proxyThrow;
let throwingProxy = new Proxy({ length: 7 }, {
  get: function (target, key, receiver) {
    if (key === "6") throw proxySentinel;
    return Reflect.get(target, key, receiver);
  }
});
try {
  Array.prototype.at.call(throwingProxy, 6);
} catch (error) {
  proxyThrow = error;
}

let argumentsValue = readArgumentsAtSix("zero", "one", "two", "three", "four", "five", "arguments");
let objectValue = Array.prototype.at.call(ordinaryObject, 6);
let functionValue = Array.prototype.at.call(ordinaryFunction, 6);
let stringValue = Array.prototype.at.call("strings", 6);
let uint8Value = new Uint8Array([3, 5, 8]).at(-1);
inheritedValue === "inherited"
  && arrayGetterReceiver === inheritedArray
  && inheritedThrow === inheritedSentinel
  && argumentsValue === "arguments"
  && objectValue === "object"
  && functionValue === "function"
  && stringValue === "s"
  && uint8Value === 8
  && proxyValue === "proxy"
  && proxyLog.join(",") === "length,6"
  && outOfBoundsValue === undefined
  && outOfBoundsLog.join(",") === "length"
  && proxyThrow === proxySentinel;
