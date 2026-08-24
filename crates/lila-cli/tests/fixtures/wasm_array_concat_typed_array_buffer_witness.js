function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

function prepareSpreadable(view, length) {
  Object.defineProperty(view, "length", {
    configurable: true,
    value: length
  });
  view[Symbol.isConcatSpreadable] = true;
  return view;
}

function assertSpreadShape(view, expectedPresence, expectedValues, label) {
  var result = [].concat(view);
  assertSame(result.length, expectedPresence.length, label + " length");
  for (var i = 0; i < expectedPresence.length; i++) {
    var present = Object.prototype.hasOwnProperty.call(result, i);
    assertSame(present, expectedPresence[i], label + " presence " + i);
    if (present && expectedValues !== undefined) {
      assertSame(result[i], expectedValues[i], label + " value " + i);
    }
  }
}

var detachedBuffer = new ArrayBuffer(4);
var detached = prepareSpreadable(new Uint16Array(detachedBuffer), 3);
detached[0] = 11;
detached[1] = 22;
__lilaDetachArrayBuffer(detachedBuffer);
assertSpreadShape(detached, [false, false, false], undefined, "detached");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var fixed = prepareSpreadable(new Uint16Array(fixedBuffer, 0, 2), 3);
fixed[0] = 31;
fixed[1] = 32;
assertSpreadShape(fixed, [true, true, false], [31, 32], "fixed in bounds");
fixedBuffer.resize(1);
assertSpreadShape(fixed, [false, false, false], undefined, "fixed out of bounds");
fixedBuffer.resize(4);
assertSpreadShape(fixed, [true, true, false], undefined, "fixed regrown");

var trackingShrinkBuffer = new ArrayBuffer(4, { maxByteLength: 5 });
var trackingShrink = prepareSpreadable(
  new Uint16Array(trackingShrinkBuffer),
  3
);
trackingShrink[0] = 41;
trackingShrink[1] = 42;
trackingShrinkBuffer.resize(3);
assertSpreadShape(
  trackingShrink,
  [true, false, false],
  [41, undefined, undefined],
  "tracking partial shrink"
);

var trackingGrowBuffer = new ArrayBuffer(4, { maxByteLength: 5 });
var trackingGrow = prepareSpreadable(new Uint16Array(trackingGrowBuffer), 3);
trackingGrow[0] = 51;
trackingGrow[1] = 52;
trackingGrowBuffer.resize(5);
assertSpreadShape(
  trackingGrow,
  [true, true, false],
  [51, 52, undefined],
  "tracking partial growth"
);

var offsetBuffer = new ArrayBuffer(6, { maxByteLength: 6 });
var offsetTracking = prepareSpreadable(new Uint16Array(offsetBuffer, 4), 2);
offsetTracking[0] = 61;
offsetBuffer.resize(3);
assertSpreadShape(
  offsetTracking,
  [false, false],
  undefined,
  "tracking offset out of bounds"
);

944;
