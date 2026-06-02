var a = [];
Object.defineProperty(a, "constructor", {
  get: function() {
    throw new Test262Error();
  }
});
try {
  a.constructor;
  "no throw";
} catch (e) {
  typeof e;
}
