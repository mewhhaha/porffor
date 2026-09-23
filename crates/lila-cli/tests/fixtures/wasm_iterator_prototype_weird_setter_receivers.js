// SetterThatIgnoresPrototypeProperties ( thisValue, home, p, v ), the setter
// of both %Iterator.prototype% weird accessors (ECMA-262 27.1.4.1 and
// 27.1.4.14), beyond the ordinary receivers in the two basic fixtures:
//   3. desc = ? thisValue.[[GetOwnProperty]](p)
//   4. desc undefined: ? CreateDataPropertyOrThrow(thisValue, p, v)
//   5. otherwise:      ? Set(thisValue, p, v, true)
// and `home` is the setter's own realm's %Iterator.prototype%.
const results = [];
function check(label, condition) {
  if (!condition) results.push(label);
}
function throwsTypeError(thunk, TypeErrorConstructor = TypeError) {
  try {
    thunk();
  } catch (error) {
    return error instanceof TypeErrorConstructor;
  }
  return false;
}

const constructorAccessor = Object.getOwnPropertyDescriptor(
  Iterator.prototype,
  "constructor",
);
const tagAccessor = Object.getOwnPropertyDescriptor(
  Iterator.prototype,
  Symbol.toStringTag,
);

// Step 5 with an own non-writable data property: Set(..., true) throws.
const readOnly = Object.defineProperty({}, "constructor", {
  value: 1,
  writable: false,
  configurable: true,
});
check(
  "own read-only throws",
  throwsTypeError(() => constructorAccessor.set.call(readOnly, 2)),
);
check("own read-only unchanged", readOnly.constructor === 1);

// Step 5 with an own accessor: Set calls it with the receiver.
let setterReceiver;
let setterValue;
const ownAccessor = {
  set constructor(value) {
    setterReceiver = this;
    setterValue = value;
  },
};
constructorAccessor.set.call(ownAccessor, 7);
check("own accessor receives value", setterValue === 7);
check("own accessor receives receiver", setterReceiver === ownAccessor);

// Step 4 on a non-extensible object: CreateDataPropertyOrThrow throws.
const closed = Object.preventExtensions({});
check(
  "non-extensible throws",
  throwsTypeError(() => tagAccessor.set.call(closed, "x")),
);
check("non-extensible unchanged", !Object.hasOwn(closed, Symbol.toStringTag));

// Step 4 on exotic receivers defines a plain enumerable data property.
function plainFunction() {}
constructorAccessor.set.call(plainFunction, Array);
const functionDescriptor = Object.getOwnPropertyDescriptor(
  plainFunction,
  "constructor",
);
check(
  "function receiver",
  functionDescriptor.value === Array &&
    functionDescriptor.writable &&
    functionDescriptor.enumerable &&
    functionDescriptor.configurable,
);
const array = [];
tagAccessor.set.call(array, "Tagged");
check("array receiver", Object.prototype.toString.call(array) === "[object Tagged]");

// Proxy receivers observe [[GetOwnProperty]] and then [[DefineOwnProperty]]
// (step 4) or [[Set]] (step 5), each exactly once per step.
const trapLog = [];
const proxy = new Proxy(
  {},
  {
    getOwnPropertyDescriptor(target, key) {
      trapLog.push("getOwnPropertyDescriptor");
      return Reflect.getOwnPropertyDescriptor(target, key);
    },
    defineProperty(target, key, descriptor) {
      trapLog.push(
        "defineProperty:" +
          (key === Symbol.toStringTag) +
          ":" +
          descriptor.value +
          ":" +
          descriptor.writable +
          descriptor.enumerable +
          descriptor.configurable,
      );
      return Reflect.defineProperty(target, key, descriptor);
    },
    set(target, key, value, receiver) {
      trapLog.push("set");
      return Reflect.set(target, key, value, receiver);
    },
  },
);
tagAccessor.set.call(proxy, "first");
check(
  "proxy define path",
  trapLog.join(",") ===
    "getOwnPropertyDescriptor,defineProperty:true:first:truetruetrue",
);
trapLog.length = 0;
tagAccessor.set.call(proxy, "second");
check(
  "proxy set path",
  trapLog.join(",") ===
    "getOwnPropertyDescriptor,set,getOwnPropertyDescriptor,defineProperty:true:second:undefinedundefinedundefined",
);

// `home` is the setter's own realm's %Iterator.prototype%.
const other = __lilaCreateRealm().global;
const otherTagAccessor = Object.getOwnPropertyDescriptor(
  other.Iterator.prototype,
  Symbol.toStringTag,
);
const otherConstructorAccessor = Object.getOwnPropertyDescriptor(
  other.Iterator.prototype,
  "constructor",
);
check(
  "other realm home throws its own TypeError",
  throwsTypeError(
    () => otherTagAccessor.set.call(other.Iterator.prototype, "x"),
    other.TypeError,
  ),
);
const foreignHeir = Object.create(Iterator.prototype);
otherConstructorAccessor.set.call(foreignHeir, 3);
check("other realm setter defines on a foreign heir", foreignHeir.constructor === 3);
check("other realm getter", otherConstructorAccessor.get.call() === other.Iterator);
check("main realm getter", constructorAccessor.get.call() === Iterator);
// The main realm's %Iterator.prototype% is not the other setter's home, so
// step 5 runs Set, which reaches the main realm's own accessor; that setter's
// home check throws the main realm's TypeError.
check(
  "other realm Set reaches the main home accessor",
  throwsTypeError(() => otherTagAccessor.set.call(Iterator.prototype, "x")),
);

if (results.length !== 0) throw results.join("; ");
true;
