var arr = [];

Object.defineProperty(arr, "a", {
  get: function () {},
  enumerable: true,
  configurable: true,
});

arr.b = 2;

Object.defineProperty(arr, "a", {
  get: function () {},
});

var keys = [];
for (var key in arr) {
  keys.push(key);
}

if (keys.length !== 2) {
  throw keys.length;
}

if (keys[0] !== "a") {
  throw keys[0];
}

if (keys[1] !== "b") {
  throw keys[1];
}

true;
