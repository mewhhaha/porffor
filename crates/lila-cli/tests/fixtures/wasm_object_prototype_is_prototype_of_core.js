let isPrototypeOf = Object.prototype.isPrototypeOf;
let receiver = {};
let child = Object.create(receiver);

let primitiveVDoesNotConvertThis = isPrototypeOf.call(null, 1) === false;
let nullThisThrowsForObjectV = false;
let undefinedThisThrowsForObjectV = false;
try {
  isPrototypeOf.call(null, {});
} catch (error) {
  nullThisThrowsForObjectV = error instanceof TypeError;
}
try {
  isPrototypeOf.call(undefined, {});
} catch (error) {
  undefinedThisThrowsForObjectV = error instanceof TypeError;
}

let primitiveThisIsBoxed = isPrototypeOf.call(1, {}) === false;
let ordinaryChain = isPrototypeOf.call(receiver, child)
  && isPrototypeOf.call(Object.prototype, child)
  && !isPrototypeOf.call(child, receiver);

let trapCount = 0;
let proxy = new Proxy({}, {
  getPrototypeOf() {
    trapCount++;
    return receiver;
  },
});
let proxyResult = isPrototypeOf.call(receiver, proxy);

let abruptTrapThrows = false;
let abruptProxy = new Proxy({}, {
  getPrototypeOf() {
    throw new Error("abrupt getPrototypeOf");
  },
});
try {
  isPrototypeOf.call(receiver, abruptProxy);
} catch (error) {
  abruptTrapThrows = error instanceof Error;
}

let fixedTarget = Object.create(receiver);
Object.preventExtensions(fixedTarget);
let fixedProxy = new Proxy(fixedTarget, {
  getPrototypeOf() {
    return receiver;
  },
});
let fixedInvariantMatch = isPrototypeOf.call(receiver, fixedProxy);

let invariantMismatchThrows = false;
let wrongPrototype = {};
let mismatchedProxy = new Proxy(fixedTarget, {
  getPrototypeOf() {
    return wrongPrototype;
  },
});
try {
  isPrototypeOf.call(receiver, mismatchedProxy);
} catch (error) {
  invariantMismatchThrows = error instanceof TypeError;
}

let nestedTrapOrder = "";
let innerProxy = new Proxy(fixedTarget, {
  getPrototypeOf() {
    nestedTrapOrder += "inner;";
    return receiver;
  },
});
let outerProxy = new Proxy(innerProxy, {
  getPrototypeOf(target) {
    nestedTrapOrder += "outer;";
    return Reflect.getPrototypeOf(target);
  },
});
let nestedProxyResult = isPrototypeOf.call(receiver, outerProxy);

primitiveVDoesNotConvertThis
  && nullThisThrowsForObjectV
  && undefinedThisThrowsForObjectV
  && primitiveThisIsBoxed
  && ordinaryChain
  && proxyResult
  && trapCount === 1
  && abruptTrapThrows
  && fixedInvariantMatch
  && invariantMismatchThrows
  && nestedProxyResult
  && nestedTrapOrder === "outer;inner;inner;";
