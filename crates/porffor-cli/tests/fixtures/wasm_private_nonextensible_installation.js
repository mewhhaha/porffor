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
  #value;
}

class PrivateMethod extends ReturnReceiver {
  #method() {}
}

class PrivateAccessor extends ReturnReceiver {
  get #value() {
    return 42;
  }
}

for (const [Constructor, message] of [
  [PrivateField, "private field installed on non-extensible receiver"],
  [PrivateMethod, "private method installed on non-extensible receiver"],
  [PrivateAccessor, "private accessor installed on non-extensible receiver"],
]) {
  const receiver = {};
  Object.preventExtensions(receiver);
  assertTypeError(() => new Constructor(receiver), message);
}

class SelfSealingField {
  #value = (Object.preventExtensions(this), 42);
}

assertTypeError(
  () => new SelfSealingField(),
  "private field installed after initializer made receiver non-extensible",
);

assertTypeError(() => {
  class SelfSealingStaticField {
    static #value = (Object.preventExtensions(SelfSealingStaticField), 42);
  }
}, "static private field installed after initializer made class non-extensible");

true;
