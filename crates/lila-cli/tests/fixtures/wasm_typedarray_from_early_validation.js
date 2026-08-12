let caught = false;
let from = Uint8Array.from;
try {
  from([]);
} catch (error) {
  caught = error instanceof TypeError;
}
if (!caught) throw "TypedArray.from bare call";

caught = false;
try {
  Uint8Array.from.call(({ m() {} }).m, []);
} catch (error) {
  caught = error instanceof TypeError;
}
if (!caught) throw "TypedArray.from non-constructor receiver";

let iteratorGetterCalled = false;
let arrayLike = {};
Object.defineProperty(arrayLike, Symbol.iterator, {
  get() {
    iteratorGetterCalled = true;
    return undefined;
  }
});

caught = false;
try {
  Uint8Array.from(arrayLike, null);
} catch (error) {
  caught = error instanceof TypeError;
}
if (!caught) throw "TypedArray.from non-callable mapper";
if (iteratorGetterCalled) throw "TypedArray.from touched iterator before mapper check";

262;
