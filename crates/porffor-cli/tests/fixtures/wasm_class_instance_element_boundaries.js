let baseOrder = "";

class BaseOrder {
  value = (baseOrder += "f", 1);

  constructor(argument = (baseOrder += "p", this.value)) {
    baseOrder += "b";
    if (argument !== 1) throw "base field unavailable to parameter default";
  }
}

new BaseOrder();
if (baseOrder !== "fpb") throw "base instance element boundary";

let derivedOrder = "";

class Parent {
  constructor() {
    derivedOrder += "s";
  }
}

class Derived extends Parent {
  value = (derivedOrder += "f", 1);

  constructor(argument = (derivedOrder += "p", 1)) {
    if (argument !== 1) throw "derived parameter default";
    derivedOrder += "b";
    super();
    derivedOrder += "a";
  }
}

new Derived();
if (derivedOrder !== "pbsfa") throw "derived instance element boundary";

let arrowOrder = "";

class ArrowParent {
  constructor() {
    arrowOrder += "s";
  }
}

class ArrowDerived extends ArrowParent {
  value = (arrowOrder += "f", 1);

  constructor() {
    arrowOrder += "b";
    (() => super())();
    arrowOrder += "a";
  }
}

new ArrowDerived();
if (arrowOrder !== "bsfa") throw "arrow super instance element boundary";

let defaultOrder = "";

class DefaultParent {
  constructor() {
    defaultOrder += "s";
  }
}

class DefaultDerived extends DefaultParent {
  value = (defaultOrder += "f", 1);
}

new DefaultDerived();
if (defaultOrder !== "sf") throw "default derived instance element boundary";

let failedSuperFieldRan = false;

class ThrowingParent {
  constructor() {
    throw "parent";
  }
}

class FailedSuper extends ThrowingParent {
  value = (failedSuperFieldRan = true);
}

try {
  new FailedSuper();
  throw "failed super returned";
} catch (error) {
  if (error !== "parent") throw error;
}

if (failedSuperFieldRan) throw "field ran after failed super";

let baseCalls = 0;
let fieldCalls = 0;

class TwiceParent {
  constructor() {
    baseCalls += 1;
  }
}

class TwiceDerived extends TwiceParent {
  value = (fieldCalls += 1);

  constructor() {
    super();
    super();
  }
}

let secondSuperThrew = false;
try {
  new TwiceDerived();
} catch (error) {
  secondSuperThrew = true;
}

if (!secondSuperThrew) throw "second super returned";
if (baseCalls !== 2) throw "second super skipped base construction";
if (fieldCalls !== 1) throw "second super repeated instance elements";

let bodyAfterFieldThrow = false;

class FieldThrowParent {}

class FieldThrowDerived extends FieldThrowParent {
  value = (() => {
    throw "field";
  })();

  constructor() {
    super();
    bodyAfterFieldThrow = true;
  }
}

try {
  new FieldThrowDerived();
  throw "throwing field returned";
} catch (error) {
  if (error !== "field") throw error;
}

if (bodyAfterFieldThrow) throw "derived body continued after field throw";

let inheritedSetterCalled = false;

class SetterParent {
  set value(next) {
    inheritedSetterCalled = true;
  }
}

class FieldDefinesOwnProperty extends SetterParent {
  value = 1;
}

const ownField = new FieldDefinesOwnProperty();
if (inheritedSetterCalled) throw "instance field called inherited setter";
if (!Object.prototype.hasOwnProperty.call(ownField, "value")) throw "missing own field";
if (ownField.value !== 1) throw "wrong own field value";

true;
