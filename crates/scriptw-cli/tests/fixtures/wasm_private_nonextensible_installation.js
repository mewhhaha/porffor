function assert(value, message) {
  if (!value) throw message;
}

function assertTypeError(callback, message) {
  try {
    callback();
  } catch (error) {
    if (error.name === "TypeError") return;
  }
  throw message;
}

class ReturnReceiver {
  constructor(receiver) {
    return receiver;
  }
}

class PrivateField extends ReturnReceiver {
  #value = 17;
  read() { return this.#value; }
  write(value) { this.#value = value; }
}

class PrivateMethod extends ReturnReceiver {
  #method() { return this; }
  read() { return this.#method(); }
}

class PrivateAccessor extends ReturnReceiver {
  #value = 23;
  get #accessor() { return this.#value; }
  set #accessor(value) { this.#value = value; }
  read() { return this.#accessor; }
  write(value) { this.#accessor = value; }
}

// Extensibility governs ordinary properties, not [[PrivateElements]].
for (const restrict of [Object.preventExtensions, Object.seal, Object.freeze]) {
  for (const Constructor of [PrivateField, PrivateMethod, PrivateAccessor]) {
    const receiver = restrict({});
    assert(new Constructor(receiver) === receiver, "private installation changed receiver");
    assert(!Object.isExtensible(receiver), "private installation changed extensibility");
    assert(Reflect.ownKeys(receiver).length === 0, "private element became an ordinary property");
    if (Constructor === PrivateMethod) {
      assert(Constructor.prototype.read.call(receiver) === receiver, "private method receiver");
    } else {
      assert(Constructor.prototype.read.call(receiver) === (Constructor === PrivateField ? 17 : 23),
        "private element initial value");
      Constructor.prototype.write.call(receiver, 29);
      assert(Constructor.prototype.read.call(receiver) === 29, "private write on restricted receiver");
    }
    assertTypeError(() => new Constructor(receiver), "duplicate private installation accepted");
  }
}

class SelfSealingField {
  #value = (Object.preventExtensions(this), 42);
  read() { return this.#value; }
}
const selfSealed = new SelfSealingField();
assert(!Object.isExtensible(selfSealed) && selfSealed.read() === 42,
  "private field addition after initializer sealed receiver");

class SelfSealingStaticField {
  static #value = (Object.preventExtensions(SelfSealingStaticField), 43);
  static read() { return this.#value; }
}
assert(!Object.isExtensible(SelfSealingStaticField) && SelfSealingStaticField.read() === 43,
  "private static field addition after initializer sealed class");

true;
