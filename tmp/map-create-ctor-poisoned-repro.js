var a = [];
var callCount = 0;
var cb = function() {
  callCount += 1;
};
Object.defineProperty(a, "constructor", {
  get: function() {
    throw new Test262Error();
  }
});
try {
  a.map(cb);
  "no throw";
} catch (e) {
  e.constructor === Test262Error && callCount === 0;
}
