let hasZero = false;
let getZero = false;
let receiverSeen = false;
let seen = [];
let proxy = new Proxy({}, {
  has: function (_target, key) {
    if (key === "0") {
      hasZero = true;
      return true;
    }
    return false;
  },
  get: function (_target, key) {
    seen.push(key);
    if (key === "length") return 1;
    if (key === "0") {
      getZero = true;
      return 7;
    }
    return undefined;
  }
});

let result = Array.prototype.reduce.call(proxy, function (accumulator, value, index, receiver) {
  receiverSeen = receiver === proxy && index === 0;
  return accumulator + value;
}, 5);

result === 12 && hasZero && getZero && receiverSeen && seen.join(",") === "length,0";
