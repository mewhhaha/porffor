function callbackfn(val, idx, obj) {
  return typeof idx;
}
Object.defineProperty(Array.prototype, "0", {
  set: function() {},
  configurable: true
});
[, ].map(callbackfn)[0];
