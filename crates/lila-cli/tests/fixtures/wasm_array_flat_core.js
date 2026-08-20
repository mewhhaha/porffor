function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

function throwsTypeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof TypeError;
  }
  return false;
}

let objectValue = { ok: true };
let descriptor = Object.getOwnPropertyDescriptor(Array.prototype, "flat");
let flat = Array.prototype.flat;
let a = [1, [2]];
let deep = [1, [2, [3, [4]]]];
let withNullish = [1, [null, void 0], objectValue];
let boundFlat = Array.prototype.flat.bind([1, [2]]);

typeof flat === "function"
  && flat.name === "flat"
  && flat.length === 0
  && descriptor.value === flat
  && descriptor.writable === true
  && descriptor.enumerable === false
  && descriptor.configurable === true
  && sameArray([].flat(), [])
  && sameArray([[], []].flat(), [])
  && sameArray([[], [1, objectValue]].flat(), [1, objectValue])
  && sameArray(withNullish.flat(), [1, null, void 0, objectValue])
  && sameArray(a.flat(), [1, 2])
  && sameArray(a.flat(undefined), [1, 2])
  && sameArray(a.flat("TestString"), a)
  && sameArray(a.flat({}), a)
  && sameArray(a.flat(+0), a)
  && sameArray(a.flat(-0), a)
  && sameArray(a.flat(Number.NEGATIVE_INFINITY), a)
  && sameArray(a.flat("1"), [1, 2])
  && sameArray([1, [2, [3]]].flat(2), [1, 2, 3])
  && sameArray(deep.flat(Number.POSITIVE_INFINITY), [1, 2, 3, 4])
  && throwsTypeError(function () { flat.call(null); })
  && throwsTypeError(function () { flat.call(undefined); })
  && throwsTypeError(function () { new flat(); })
  && sameArray(boundFlat(), [1, 2]);
