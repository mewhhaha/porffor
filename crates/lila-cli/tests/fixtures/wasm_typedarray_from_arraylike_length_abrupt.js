let getterSentinel = { name: "getter sentinel" };
let getterArrayLike = {};
Object.defineProperty(getterArrayLike, "length", {
  get() {
    throw getterSentinel;
  }
});

let caught = false;
try {
  Uint8Array.from(getterArrayLike);
} catch (error) {
  caught = error === getterSentinel;
}
if (!caught) throw "TypedArray.from length getter";

let valueOfSentinel = { name: "valueOf sentinel" };
let valueOfArrayLike = {
  length: {
    valueOf() {
      throw valueOfSentinel;
    }
  }
};

caught = false;
try {
  Float32Array.from(valueOfArrayLike);
} catch (error) {
  caught = error === valueOfSentinel;
}
if (!caught) throw "TypedArray.from length valueOf";

262;
