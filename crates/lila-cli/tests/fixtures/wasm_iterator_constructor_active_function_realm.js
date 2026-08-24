let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

let entryActiveError;
try {
  Reflect.construct(Iterator, [], Iterator);
} catch (error) {
  entryActiveError = error;
}
if (entryActiveError === undefined) {
  throw "entry-realm active Iterator must throw";
}
if (Object.getPrototypeOf(entryActiveError) !== TypeError.prototype) {
  throw "entry-realm active Iterator TypeError realm";
}

let activeError;
try {
  Reflect.construct(other.Iterator, [], other.Iterator);
} catch (error) {
  activeError = error;
}
if (activeError === undefined) {
  throw "created-realm active Iterator must throw";
}
if (Object.getPrototypeOf(activeError) !== other.TypeError.prototype) {
  throw "created-realm active Iterator TypeError realm";
}

let entryActiveProxyEvents = [];
let entryActiveProxy = new Proxy(Iterator, {
  get: function (target, key, receiver) {
    if (key === "prototype") entryActiveProxyEvents.push("prototype");
    return Reflect.get(target, key, receiver);
  },
});
let entryActiveProxyResult = Reflect.construct(Iterator, [], entryActiveProxy);
entryActiveProxyEvents.push("return");
if (!(entryActiveProxyEvents.join(",") === "prototype,return" &&
      Object.getPrototypeOf(entryActiveProxyResult) === Iterator.prototype)) {
  throw "entry-realm active Iterator Proxy identity or prototype Get order";
}

let entryActiveBoundEvents = [];
let entryActiveBound = Iterator.bind(null);
Object.defineProperty(entryActiveBound, "prototype", {
  configurable: true,
  get: function () {
    entryActiveBoundEvents.push("prototype");
    return undefined;
  },
});
let entryActiveBoundResult = Reflect.construct(Iterator, [], entryActiveBound);
entryActiveBoundEvents.push("return");
if (!(entryActiveBoundEvents.join(",") === "prototype,return" &&
      Object.getPrototypeOf(entryActiveBoundResult) === Iterator.prototype)) {
  throw "entry-realm active Iterator bound identity or prototype Get order";
}

let otherActiveProxyEvents = [];
let otherActiveProxy = new Proxy(other.Iterator, {
  get: function (target, key, receiver) {
    if (key === "prototype") otherActiveProxyEvents.push("prototype");
    return Reflect.get(target, key, receiver);
  },
});
let otherActiveProxyResult = Reflect.construct(
  other.Iterator,
  [],
  otherActiveProxy,
);
otherActiveProxyEvents.push("return");
if (!(otherActiveProxyEvents.join(",") === "prototype,return" &&
      Object.getPrototypeOf(otherActiveProxyResult) === other.Iterator.prototype)) {
  throw "created-realm active Iterator Proxy identity or prototype Get order";
}

let otherActiveBoundEvents = [];
let otherActiveBound = other.Iterator.bind(null);
Object.defineProperty(otherActiveBound, "prototype", {
  configurable: true,
  get: function () {
    otherActiveBoundEvents.push("prototype");
    return undefined;
  },
});
let otherActiveBoundResult = Reflect.construct(
  other.Iterator,
  [],
  otherActiveBound,
);
otherActiveBoundEvents.push("return");
if (!(otherActiveBoundEvents.join(",") === "prototype,return" &&
      Object.getPrototypeOf(otherActiveBoundResult) === other.Iterator.prototype)) {
  throw "created-realm active Iterator bound identity or prototype Get order";
}

let entryNewTarget = Reflect.construct(other.Iterator, [], Iterator);
if (Object.getPrototypeOf(entryNewTarget) !== Iterator.prototype) {
  throw "distinct entry Iterator NewTarget rejected or wrong prototype";
}

let entryPrototypeReads = 0;
let observedEntryIterator = new Proxy(Iterator, {
  get: function (target, key, receiver) {
    if (key === "prototype") entryPrototypeReads = entryPrototypeReads + 1;
    return Reflect.get(target, key, receiver);
  },
});
let observedEntryNewTarget = Reflect.construct(
  other.Iterator,
  [],
  observedEntryIterator,
);
if (!(entryPrototypeReads === 1 &&
      Object.getPrototypeOf(observedEntryNewTarget) === Iterator.prototype)) {
  throw "distinct entry Iterator prototype Get count";
}

let otherNewTarget = Reflect.construct(Iterator, [], other.Iterator);
if (Object.getPrototypeOf(otherNewTarget) !== other.Iterator.prototype) {
  throw "distinct created Iterator NewTarget rejected or wrong prototype";
}

let otherPrototypeReads = 0;
let observedOtherIterator = new Proxy(other.Iterator, {
  get: function (target, key, receiver) {
    if (key === "prototype") otherPrototypeReads = otherPrototypeReads + 1;
    return Reflect.get(target, key, receiver);
  },
});
let observedOtherNewTarget = Reflect.construct(
  Iterator,
  [],
  observedOtherIterator,
);
if (!(otherPrototypeReads === 1 &&
      Object.getPrototypeOf(observedOtherNewTarget) === other.Iterator.prototype)) {
  throw "distinct created Iterator prototype Get count";
}

1515;
