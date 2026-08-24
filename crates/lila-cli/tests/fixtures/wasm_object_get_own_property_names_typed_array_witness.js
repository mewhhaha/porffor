function assertNames(view, expected, label) {
  var actual = Object.getOwnPropertyNames(view);
  if (actual.length !== expected.length) {
    throw label + " length: " + actual.join(",");
  }
  for (var i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) {
      throw label + " key " + i + ": " + actual[i];
    }
  }
}

function addOrdinaryKeys(view, label) {
  view.visible = label;
  Object.defineProperty(view, "hidden", {
    configurable: true,
    value: label
  });
  view[Symbol(label)] = label;
  return view;
}

var detachedBuffer = new ArrayBuffer(4);
var detached = addOrdinaryKeys(new Uint16Array(detachedBuffer), "detached");
__lilaDetachArrayBuffer(detachedBuffer);
assertNames(detached, ["visible", "hidden"], "detached");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var fixed = addOrdinaryKeys(new Uint16Array(fixedBuffer, 0, 2), "fixed");
assertNames(fixed, ["0", "1", "visible", "hidden"], "fixed in bounds");
fixedBuffer.resize(1);
assertNames(fixed, ["visible", "hidden"], "fixed out of bounds");
fixedBuffer.resize(4);
assertNames(fixed, ["0", "1", "visible", "hidden"], "fixed regrown");

var trackingBuffer = new ArrayBuffer(4, { maxByteLength: 5 });
var tracking = addOrdinaryKeys(new Uint16Array(trackingBuffer), "tracking");
trackingBuffer.resize(3);
assertNames(
  tracking,
  ["0", "visible", "hidden"],
  "tracking partial shrink"
);
trackingBuffer.resize(5);
assertNames(
  tracking,
  ["0", "1", "visible", "hidden"],
  "tracking partial growth"
);

var offsetBuffer = new ArrayBuffer(6, { maxByteLength: 6 });
var offsetTracking = addOrdinaryKeys(
  new Uint16Array(offsetBuffer, 4),
  "offset"
);
offsetBuffer.resize(3);
assertNames(
  offsetTracking,
  ["visible", "hidden"],
  "tracking offset out of bounds"
);

951;
