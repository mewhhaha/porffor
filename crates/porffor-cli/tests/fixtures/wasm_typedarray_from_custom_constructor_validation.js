let TA = Uint8Array;
let caught = false;

try {
  TA.from.call(function () {}, []);
} catch (error) {
  caught = error instanceof TypeError;
}
if (!caught) throw "TypedArray.from custom constructor plain object";

caught = false;
try {
  TA.from.call(function () { return new TA(1); }, [1, 2]);
} catch (error) {
  caught = error instanceof TypeError;
}
if (!caught) throw "TypedArray.from custom constructor small array source";

caught = false;
try {
  TA.from.call(function () { return new TA(1); }, { 0: 1, 1: 2, length: 2 });
} catch (error) {
  caught = error instanceof TypeError;
}
if (!caught) throw "TypedArray.from custom constructor small array-like source";

262;
