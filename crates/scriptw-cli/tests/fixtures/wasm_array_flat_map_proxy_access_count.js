function sameArray(actual, expected) {
  if (actual.length !== expected.length) {
    return false;
  }
  for (let i = 0; i < actual.length; i++) {
    if (actual[i] !== expected[i]) {
      return false;
    }
  }
  return true;
}

const getCalls = [];
const hasCalls = [];

const handler = {
  get: function (target, property, receiver) {
    getCalls.push(property);
    return Reflect.get(target, property, receiver);
  },
  has: function (target, property) {
    hasCalls.push(property);
    return Reflect.has(target, property);
  }
};

const tier2 = new Proxy([4, 3], handler);
const tier1 = new Proxy([2, [3, 4, 2, 2], 5, tier2, 6], handler);

Array.prototype.flatMap.call(tier1, function (value) {
  return value;
});

sameArray(getCalls, ["length", "constructor", "0", "1", "2", "3", "length", "0", "1", "4"])
  && sameArray(hasCalls, ["0", "1", "2", "3", "0", "1", "4"]);
