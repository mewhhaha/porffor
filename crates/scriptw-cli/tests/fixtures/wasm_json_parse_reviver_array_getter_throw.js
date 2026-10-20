let sentinel = { thrown: true };
let ok = false;

try {
  JSON.parse("[0,0]", function () {
    Object.defineProperty(this, "1", {
      get: function () {
        throw sentinel;
      }
    });
  });
} catch (err) {
  ok = err === sentinel;
}

ok;
