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
