function callbackfn(val, idx, obj) {
  if (idx === 0) return typeof val === "undefined";
  return false;
}
Object.defineProperty(Array.prototype, "0", {
  set: function() {},
  configurable: true
});
[, ].map(callbackfn)[0];
