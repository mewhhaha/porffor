function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function checkKeys(actual, expected, label) {
  check(actual.length, expected.length, label + " length");
  for (var i = 0; i < expected.length; i++) {
    check(actual[i], expected[i], label + " key " + i);
  }
}

var getterCalls = 0;
var array = [];
array[2] = "indexed";
array.visibleFirst = 1;
Object.defineProperty(array, "hidden", {
  get: function () {
    getterCalls++;
    return 2;
  },
  enumerable: false,
  configurable: true,
});
array.visibleLast = 3;
array[Symbol("symbol key")] = 4;

checkKeys(
  Object.getOwnPropertyNames(array),
  ["2", "length", "visibleFirst", "hidden", "visibleLast"],
  "all string keys",
);
checkKeys(
  Object.keys(array),
  ["2", "visibleFirst", "visibleLast"],
  "enumerable string keys",
);
check(getterCalls, 0, "key selection does not invoke accessors");

262;
