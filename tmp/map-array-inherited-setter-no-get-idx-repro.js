function callbackfn(val, idx, obj) {
  return idx;
}
Object.defineProperty(Array.prototype, "0", {
  set: function() {},
  configurable: true
});
[, ].map(callbackfn)[0];
