Object.defineProperty(Array.prototype, "0", {
  set: function() {},
  configurable: true
});
typeof [, ][0];
