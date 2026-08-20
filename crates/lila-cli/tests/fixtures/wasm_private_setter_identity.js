function makePrivateSetterClass() {
  return class PrivateSetterClass {
    get #value() {
      return this.stored;
    }

    set #value(value) {
      this.stored = value;
    }

    write(target, value) {
      return target.#value = value;
    }

    read() {
      return this.#value;
    }
  };
}

const FirstPrivateSetter = makePrivateSetterClass();
const SecondPrivateSetter = makePrivateSetterClass();
const firstPrivateSetter = new FirstPrivateSetter();
const secondPrivateSetter = new SecondPrivateSetter();

if (firstPrivateSetter.write(firstPrivateSetter, 41) !== 41) {
  throw "private setter assignment result";
}
if (firstPrivateSetter.read() !== 41) throw "private setter effect";
if (secondPrivateSetter.write(secondPrivateSetter, 42) !== 42) {
  throw "repeated private setter assignment result";
}
if (secondPrivateSetter.read() !== 42) throw "repeated private setter effect";

let wrongPrivateSetterBrand = false;
try {
  firstPrivateSetter.write(secondPrivateSetter, 43);
} catch (error) {
  wrongPrivateSetterBrand = error.name === "TypeError";
}
if (!wrongPrivateSetterBrand) throw "private setter identity crossed class evaluation";

class ThrowingPrivateSetter {
  set #value(value) {
    throw value;
  }

  write(value) {
    this.#value = value;
  }
}

let privateSetterThrowPropagated = false;
try {
  new ThrowingPrivateSetter().write("private setter throw");
} catch (error) {
  privateSetterThrowPropagated = error === "private setter throw";
}
if (!privateSetterThrowPropagated) throw "private setter swallowed throw";

true;
