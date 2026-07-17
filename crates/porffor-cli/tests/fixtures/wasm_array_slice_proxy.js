function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let index = 0; index < actual.length; index++) {
    if (actual[index] !== expected[index]) return false;
  }
  return true;
}

let sourceLog = [];
let source = new Proxy({ 0: "zero", 2: "two", length: 3 }, {
  get: function (target, key, receiver) {
    sourceLog.push("get:" + key);
    return Reflect.get(target, key, receiver);
  },
  has: function (target, key) {
    sourceLog.push("has:" + key);
    return Reflect.has(target, key);
  }
});
let sourceResult = Array.prototype.slice.call(source);

let targetLog = [];
function ProxyTarget(length) {
  return new Proxy({}, {
    defineProperty: function (target, key, descriptor) {
      targetLog.push("define:" + key);
      return Reflect.defineProperty(target, key, descriptor);
    },
    set: function (target, key, value, receiver) {
      targetLog.push("set:" + key);
      return Reflect.set(target, key, value, target);
    }
  });
}

let speciesSource = ["zero", , "two"];
speciesSource.constructor = {
  get [Symbol.species]() {
    return ProxyTarget;
  }
};
let speciesResult = speciesSource.slice();

let highIndexSource = [];
highIndexSource["9007199254740989"] = "9007199254740989";
highIndexSource["9007199254740990"] = "9007199254740990";
let highIndexProxy = new Proxy(highIndexSource, {
  get: function (target, key, receiver) {
    if (key === "length") return 2 ** 53 + 2;
    return Reflect.get(target, key, receiver);
  }
});
let highIndexResult = Array.prototype.slice.call(highIndexProxy, 9007199254740989);

sameArray(sourceResult, ["zero", undefined, "two"])
  && sourceLog.join(",") === "get:length,has:0,get:0,has:1,has:2,get:2"
  && speciesResult[0] === "zero"
  && speciesResult[2] === "two"
  && speciesResult.length === 3
  && targetLog.join(",") === "define:0,define:2,set:length"
  && Reflect.get(highIndexSource, "9007199254740989", highIndexProxy) === "9007199254740989"
  && sameArray(highIndexResult, ["9007199254740989", "9007199254740990"]);
