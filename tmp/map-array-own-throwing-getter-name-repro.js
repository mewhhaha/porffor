var arr = [];
arr[5] = 10;
arr[10] = 100;
Object.defineProperty(arr, "1", {
  get: function() { throw new RangeError("boom"); },
  configurable: true
});
try {
  arr.map(function(val, idx, obj) {});
  "no throw";
} catch (e) {
  e.name;
}
