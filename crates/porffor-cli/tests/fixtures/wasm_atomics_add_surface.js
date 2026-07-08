function assertTypeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof TypeError)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

function verifyProperty(object, key, expected) {
  let actual = Object.getOwnPropertyDescriptor(object, key);
  if (actual === undefined) throw key + " descriptor missing";
  if (actual.value !== expected.value) throw key + " descriptor value";
  if (actual.writable !== expected.writable) throw key + " descriptor writable";
  if (actual.enumerable !== expected.enumerable) throw key + " descriptor enumerable";
  if (actual.configurable !== expected.configurable) throw key + " descriptor configurable";
}

if (typeof Atomics.add !== "function") throw "add function";
if (Atomics.add.length !== 3) throw "add length";
if (Atomics.add.name !== "add") throw "add name";

verifyProperty(Atomics, "add", {
  value: Atomics.add,
  writable: true,
  enumerable: false,
  configurable: true,
});
verifyProperty(Atomics.add, "length", {
  value: 3,
  writable: false,
  enumerable: false,
  configurable: true,
});
verifyProperty(Atomics.add, "name", {
  value: "add",
  writable: false,
  enumerable: false,
  configurable: true,
});

let savedAdd = Atomics.add;
if (delete Atomics.add !== true) throw "add delete result";
if ("add" in Atomics) throw "add still present after delete";
Object.defineProperty(Atomics, "add", {
  value: savedAdd,
  writable: true,
  enumerable: false,
  configurable: true,
});
if (Atomics.add !== savedAdd) throw "add restore";

assertTypeError(function () { new Atomics.add(undefined, 0, 0); }, "constructor");
assertTypeError(function () { Atomics.add(undefined, 0, 0); }, "undefined view");
assertTypeError(function () { Atomics.add(null, 0, 0); }, "null view");
assertTypeError(function () { Atomics.add({}, 0, 0); }, "plain object view");
assertTypeError(function () { Atomics.add([], 0, 0); }, "array view");
assertTypeError(function () { Atomics.add(new DataView(new ArrayBuffer(8)), 0, 0); }, "dataview view");

234;
