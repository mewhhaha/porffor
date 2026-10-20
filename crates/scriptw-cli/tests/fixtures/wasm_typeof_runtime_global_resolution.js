function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function checkBuiltinTypes() {
  check(typeof Number, "function", "Number type");
  check(typeof Symbol, "function", "Symbol type");
  check(typeof BigInt, "function", "BigInt type");
}

let createdDuringWithTypeof;
let createdDuringWithTypeofName = "createdDuringWithTypeofValue";
with ({
  createdDuringWithTypeofValue: 0,
  get [Symbol.unscopables]() {
    globalThis[createdDuringWithTypeofName] = 1;
    return { createdDuringWithTypeofValue: true };
  }
}) {
  createdDuringWithTypeof = typeof createdDuringWithTypeofValue;
}
check(createdDuringWithTypeof, "number", "global created during unscopables lookup");
delete globalThis.createdDuringWithTypeofValue;

checkBuiltinTypes();

let receiverTypeError = false;
try {
  BigInt.prototype.toString(1);
} catch (error) {
  receiverTypeError = error instanceof TypeError;
}
check(receiverTypeError, true, "BigInt prototype receiver");

globalThis.createdTypeofValue = 1;
function createdType() {
  return typeof createdTypeofValue;
}
check(createdType(), "number", "created global type");

let getterCalls = 0;
Object.defineProperty(globalThis, "accessorTypeofValue", {
  configurable: true,
  get: function() {
    getterCalls += 1;
    return 1;
  }
});
check(typeof accessorTypeofValue, "number", "accessor global type");
check(getterCalls, 1, "accessor global calls");

delete globalThis.accessorTypeofValue;
delete globalThis.createdTypeofValue;
check(typeof accessorTypeofValue, "undefined", "deleted accessor type");
check(createdType(), "undefined", "deleted created global type");

true;
