function hasOwn(object, key) {
  return Object.prototype.hasOwnProperty.call(object, key);
}

let inherited = [, "b"];
Array.prototype[0] = "a";
let inheritedResult = inherited.sort();
delete Array.prototype[0];

let log = [];
let proxyTarget = { 0: "b", 2: "a", length: 3 };
let proxy = new Proxy(proxyTarget, {
  get: function (target, key, receiver) {
    log.push("get:" + key);
    return Reflect.get(target, key, receiver);
  },
  has: function (target, key) {
    log.push("has:" + key);
    return Reflect.has(target, key);
  },
  set: function (target, key, value) {
    log.push("set:" + key + ":" + value);
    return Reflect.set(target, key, value, target);
  },
  deleteProperty: function (target, key) {
    log.push("delete:" + key);
    return Reflect.deleteProperty(target, key);
  }
});
let proxyResult = Array.prototype.sort.call(proxy);

let accessorLog = [];
Object.defineProperty(Object.prototype, "2", {
  get: function () {
    accessorLog.push("get");
    return 4;
  },
  set: function (value) {
    accessorLog.push("set with " + value);
  },
  configurable: true
});
let accessorArray = [undefined, 3, , 2, undefined, , 1];
accessorArray.sort();
let accessorResult = accessorLog.join(",") === "get,set with 3"
  && !hasOwn(accessorArray, "2")
  && accessorArray[0] === 1
  && accessorArray[1] === 2
  && accessorArray[3] === 4
  && accessorArray[4] === undefined
  && accessorArray[5] === undefined
  && !("6" in accessorArray)
  && accessorArray.length === 7;
delete Object.prototype[2];

inheritedResult === inherited
  && inherited[0] === "a"
  && inherited[1] === "b"
  && proxyResult === proxy
  && proxyTarget[0] === "a"
  && proxyTarget[1] === "b"
  && !hasOwn(proxyTarget, "2")
  && log.join(",") === "get:length,has:0,get:0,has:1,has:2,get:2,set:0:a,set:1:b,delete:2"
  && accessorResult;
