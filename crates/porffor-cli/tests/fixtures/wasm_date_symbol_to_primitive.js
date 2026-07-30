const toPrimitive = Date.prototype[Symbol.toPrimitive];
if (typeof toPrimitive !== "function") throw "missing";
if (toPrimitive.name !== "[Symbol.toPrimitive]") throw "name";
if (toPrimitive.length !== 1) throw "length";

const descriptor = Object.getOwnPropertyDescriptor(
  Date.prototype,
  Symbol.toPrimitive
);
if (descriptor.writable !== false) throw "writable";
if (descriptor.enumerable !== false) throw "enumerable";
if (descriptor.configurable !== true) throw "configurable";

let steps = [];
const stringFirst = {
  toString() {
    steps.push("toString");
    return {};
  },
  valueOf() {
    steps.push("valueOf");
    return 7;
  }
};
if (toPrimitive.call(stringFirst, "default") !== 7) throw "default result";
if (steps.join(",") !== "toString,valueOf") throw "default order";

steps = [];
const numberFirst = {
  toString() {
    steps.push("toString");
    return "later";
  },
  valueOf() {
    steps.push("valueOf");
    return 8;
  }
};
if (toPrimitive.call(numberFirst, "number") !== 8) throw "number result";
if (steps.join(",") !== "valueOf") throw "number order";

let threw = false;
try {
  toPrimitive.call({}, "invalid");
} catch (error) {
  threw = error instanceof TypeError;
}
if (!threw) throw "invalid hint";

threw = false;
try {
  toPrimitive.call(null, "string");
} catch (error) {
  threw = error instanceof TypeError;
}
if (!threw) throw "invalid receiver";

threw = false;
try {
  toPrimitive.call({ toString: null, valueOf: null }, "string");
} catch (error) {
  threw = error instanceof TypeError;
}
if (!threw) throw "no primitive";

262;
