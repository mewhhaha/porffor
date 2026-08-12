let privateGetterCalls = 0;

class PrivateCallableBase {
  methodReceiver() {
    return this;
  }

  static staticReceiver() {
    return this;
  }
}

function makePrivateCallableClass() {
  return class PrivateCallableClass extends PrivateCallableBase {
    #value = 1;

    #method() {
      return super.methodReceiver();
    }

    get #paired() {
      privateGetterCalls += 1;
      return this.#value;
    }

    set #paired(value) {
      this.#value = value;
    }

    get #getterOnly() {
      return this;
    }

    set #setterOnly(value) {
      this.#value = value;
    }

    get #throwing() {
      throw "private getter throw";
    }

    static #staticMethod() {
      return super.staticReceiver();
    }

    static get #staticGetter() {
      return this;
    }

    methodReference() {
      return this.#method;
    }

    callMethod() {
      return this.#method();
    }

    readPaired() {
      return this.#paired;
    }

    writePaired(value) {
      this.#paired = value;
      return this.#value;
    }

    readSetterOnly() {
      return this.#setterOnly;
    }

    writeGetterOnly() {
      this.#getterOnly = 2;
    }

    writeMethod() {
      this.#method = 2;
    }

    readThrowing() {
      return this.#throwing;
    }

    static callStaticMethod() {
      return this.#staticMethod();
    }

    static staticMethodReference() {
      return this.#staticMethod;
    }

    static readStaticGetter() {
      return this.#staticGetter;
    }
  };
}

const FirstPrivateCallable = makePrivateCallableClass();
const SecondPrivateCallable = makePrivateCallableClass();
const first = new FirstPrivateCallable();
const second = new FirstPrivateCallable();
const otherClass = new SecondPrivateCallable();

if (first.methodReference() !== second.methodReference()) {
  throw "private method identity changed between instances";
}
if (first.methodReference() === otherClass.methodReference()) {
  throw "private method identity crossed class evaluation";
}
if (first.methodReference().name !== "#method") throw "private method name";
if (first.callMethod() !== first) throw "private method receiver";

if (first.readPaired() !== 1 || privateGetterCalls !== 1) {
  throw "private getter invocation";
}
if (first.writePaired(2) !== 2 || first.readPaired() !== 2) {
  throw "paired private accessor";
}
if (privateGetterCalls !== 2) throw "private getter invocation count";

if (FirstPrivateCallable.callStaticMethod() !== FirstPrivateCallable) {
  throw "static private method placement";
}
if (FirstPrivateCallable.staticMethodReference().name !== "#staticMethod") {
  throw "static private method name";
}
if (FirstPrivateCallable.readStaticGetter() !== FirstPrivateCallable) {
  throw "static private getter placement";
}

let setterOnlyReadRejected = false;
try {
  first.readSetterOnly();
} catch (error) {
  setterOnlyReadRejected = error.name === "TypeError";
}
if (!setterOnlyReadRejected) throw "setter-only private read";

let getterOnlyWriteRejected = false;
try {
  first.writeGetterOnly();
} catch (error) {
  getterOnlyWriteRejected = error.name === "TypeError";
}
if (!getterOnlyWriteRejected) throw "getter-only private write";

let methodWriteRejected = false;
try {
  first.writeMethod();
} catch (error) {
  methodWriteRejected = error.name === "TypeError";
}
if (!methodWriteRejected) throw "private method write";

let getterThrowPropagated = false;
let continuedAfterGetterThrow = false;
try {
  first.readThrowing();
  continuedAfterGetterThrow = true;
} catch (error) {
  getterThrowPropagated = error === "private getter throw";
}
if (!getterThrowPropagated || continuedAfterGetterThrow) {
  throw "private getter abrupt completion";
}

true;
