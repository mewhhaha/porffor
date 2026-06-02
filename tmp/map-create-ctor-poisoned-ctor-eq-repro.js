var a = [];
Object.defineProperty(a, "constructor", {
  get: function() {
    throw new Test262Error();
  }
});
try {
  a.map(function() {});
  "no throw";
} catch (e) {
  e.constructor === Test262Error;
}
