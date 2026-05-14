var called = 0;
var sentinel = { name: "custom constructor sentinel" };

function ctor() {
  called++;
  throw sentinel;
}

var caught = false;
try {
  Uint8Array.from.call(ctor, []);
} catch (error) {
  caught = error === sentinel;
}

if (!caught) throw "custom constructor abrupt";
if (called !== 1) throw "custom constructor called count";

262;
