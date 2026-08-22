let computedCalls = 0;

class Base {
  accessor publicValue = 1;
  accessor #privateValue = 2;
  static accessor staticValue = 3;
  static accessor #privateStatic = 4;
  accessor [++computedCalls] = 5;

  readPrivate() { return this.#privateValue; }
  writePrivate(value) { this.#privateValue = value; }
  static readPrivate() { return this.#privateStatic; }
  static writePrivate(value) { this.#privateStatic = value; }
}

class Derived extends Base {}

class ReplacementBase {
  constructor() { return Object.preventExtensions({}); }
}
class NonExtensibleDerived extends ReplacementBase {
  accessor value = 15;
}

let symbolKey = Symbol.iterator;
class Keyed {
  accessor "text" = 16;
  accessor 17 = 18;
  accessor [symbolKey] = 19;
}

class OverwrittenByGetter {
  accessor value = 20;
  get value() { return 21; }
}
class OverwritesGetter {
  get value() { return 22; }
  accessor value = 23;
}

let base = new Base();
let derived = new Derived();
base.publicValue = 11;
base.writePrivate(12);
Base.staticValue = 13;
Base.writePrivate(14);
let descriptor = Object.getOwnPropertyDescriptor(Base.prototype, "publicValue");
let getter = descriptor.get;
let setter = descriptor.set;
let detached = getter.call(base) === 11;
setter.call(base, 24);

let publicWrongReceiver = false;
try {
  getter.call({});
} catch (error) {
  publicWrongReceiver = error instanceof TypeError;
}

let privateWrongReceiver = false;
try {
  Base.prototype.readPrivate.call({});
} catch (error) {
  privateWrongReceiver = error instanceof TypeError;
}

let nonExtensibleRejected = false;
try {
  new NonExtensibleDerived();
} catch (error) {
  nonExtensibleRejected = error instanceof TypeError;
}
let keyed = new Keyed();

function assert(value, label) {
  if (!value) throw label;
}

assert(computedCalls === 1, "computed name evaluated once");
assert(detached, "detached public getter");
assert(base.publicValue === 24, "detached public setter");
assert(base.readPrivate() === 12, "private instance accessor");
assert(Base.staticValue === 13, "public static accessor");
assert(Base.readPrivate() === 14, "private static accessor");
assert(base[1] === 5, "computed accessor");
assert(derived.publicValue === 1, "inherited public accessor");
assert(derived.readPrivate() === 2, "inherited private accessor");
assert(descriptor.enumerable === false, "descriptor enumerable");
assert(descriptor.configurable === true, "descriptor configurable");
assert(descriptor.get.length === 0, "getter length");
assert(descriptor.set.length === 1, "setter length");
assert(publicWrongReceiver, "public wrong receiver");
assert(privateWrongReceiver, "private wrong receiver");
assert(nonExtensibleRejected, "non-extensible receiver");
assert(keyed.text === 16, "string key");
assert(keyed[17] === 18, "numeric key");
assert(keyed[symbolKey] === 19, "symbol key");
assert(new OverwrittenByGetter().value === 21, "ordinary getter overwrites auto-accessor");
assert(new OverwritesGetter().value === 23, "auto-accessor overwrites ordinary getter");

true;
