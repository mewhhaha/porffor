let symbolThrew = false;
try {
  Uint8Array.from([Symbol()]);
} catch (err) {
  symbolThrew = err instanceof TypeError;
}
if (!symbolThrew) throw "TypedArray.from symbol value";

let getterThrew = false;
let source = { length: 1 };
Object.defineProperty(source, "0", {
  get: function () {
    throw new RangeError("from getter");
  }
});
try {
  Float32Array.from(source);
} catch (err) {
  getterThrew = err instanceof RangeError;
}
if (!getterThrew) throw "TypedArray.from getter abrupt";

262;
