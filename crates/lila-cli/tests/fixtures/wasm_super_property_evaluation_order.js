class Base {}

class BeforeNull extends Base {
  constructor() {
    let directKeySideEffect = 0;
    let directCorrect = false;
    try {
      super[directKeySideEffect++];
    } catch (error) {
      directCorrect = error instanceof ReferenceError && directKeySideEffect === 0;
    }

    let arrowKeySideEffect = 0;
    let arrowCorrect = false;
    const read = () => super[arrowKeySideEffect++];
    try {
      read();
    } catch (error) {
      arrowCorrect = error instanceof ReferenceError && arrowKeySideEffect === 0;
    }

    super();
    if (!directCorrect) throw "before-direct";
    if (!arrowCorrect) throw "before-arrow";
  }
}
Object.setPrototypeOf(BeforeNull.prototype, null);
new BeforeNull();

class AfterNullPrimitive extends Base {
  constructor() {
    super();
    let directKeySideEffect = 0;
    let directCorrect = false;
    try {
      super[directKeySideEffect++];
    } catch (error) {
      directCorrect = error instanceof TypeError && directKeySideEffect === 1;
    }

    let arrowKeySideEffect = 0;
    let arrowCorrect = false;
    const read = () => super[arrowKeySideEffect++];
    try {
      read();
    } catch (error) {
      arrowCorrect = error instanceof TypeError && arrowKeySideEffect === 1;
    }

    if (!directCorrect) throw "after-primitive-direct";
    if (!arrowCorrect) throw "after-primitive-arrow";
  }
}
Object.setPrototypeOf(AfterNullPrimitive.prototype, null);
new AfterNullPrimitive();

class AfterNullObject extends Base {
  constructor() {
    super();
    let coercions = 0;
    const key = { toString() { coercions++; return "marker"; } };
    let correct = false;
    try {
      super[key];
    } catch (error) {
      correct = error instanceof TypeError && coercions === 0;
    }
    if (!correct) throw "after-object";
  }
}
Object.setPrototypeOf(AfterNullObject.prototype, null);
new AfterNullObject();

const inherited = { marker: 42 };
class NonNull extends Base {
  constructor() {
    super();
    let coercions = 0;
    const key = { toString() { coercions++; return "marker"; } };
    if (super[key] !== 42 || coercions !== 1) throw "non-null";
  }
}
Object.setPrototypeOf(NonNull.prototype, inherited);
new NonNull();

class ToPrimitiveGetterAbrupt extends Base {
  constructor() {
    super();
    const marker = {};
    const key = {};
    Object.defineProperty(key, Symbol.toPrimitive, {
      get: function () { throw marker; }
    });
    let caught = false;
    try {
      super[key];
    } catch (error) {
      caught = error === marker;
    }
    if (!caught) throw "to-primitive getter abrupt";
  }
}
Object.setPrototypeOf(ToPrimitiveGetterAbrupt.prototype, inherited);
new ToPrimitiveGetterAbrupt();

class ToStringGetterAbrupt extends Base {
  constructor() {
    super();
    const marker = {};
    const key = {};
    Object.defineProperty(key, "toString", {
      get: function () { throw marker; }
    });
    let caught = false;
    try {
      super[key];
    } catch (error) {
      caught = error === marker;
    }
    if (!caught) throw "to-string getter abrupt";
  }
}
Object.setPrototypeOf(ToStringGetterAbrupt.prototype, inherited);
new ToStringGetterAbrupt();

class ToPrimitiveCallAbrupt extends Base {
  constructor() {
    super();
    const marker = {};
    const key = {};
    Object.defineProperty(key, Symbol.toPrimitive, {
      value: function () { throw marker; }
    });
    let caught = false;
    try {
      super[key];
    } catch (error) {
      caught = error === marker;
    }
    if (!caught) throw "to-primitive call abrupt";
  }
}
Object.setPrototypeOf(ToPrimitiveCallAbrupt.prototype, inherited);
new ToPrimitiveCallAbrupt();

class ProxyToPrimitiveGetterAbrupt extends Base {
  constructor() {
    super();
    const marker = {};
    const key = new Proxy({}, {
      get: function () { throw marker; }
    });
    let caught = false;
    try {
      super[key];
    } catch (error) {
      caught = error === marker;
    }
    if (!caught) throw "proxy to-primitive getter abrupt";
  }
}
Object.setPrototypeOf(ProxyToPrimitiveGetterAbrupt.prototype, inherited);
new ProxyToPrimitiveGetterAbrupt();

let finalizedNullSuperRead = false;
class FinallyNullSuperRead extends Base {
  read() {
    try {
      return super.marker;
    } finally {
      finalizedNullSuperRead = true;
    }
  }
}
Object.setPrototypeOf(FinallyNullSuperRead.prototype, null);
let finalizedNullSuperReadThrew = false;
try {
  new FinallyNullSuperRead().read();
} catch (error) {
  finalizedNullSuperReadThrew = error instanceof TypeError;
}
if (!finalizedNullSuperRead || !finalizedNullSuperReadThrew) {
  throw "null super read skipped finally";
}

true;
