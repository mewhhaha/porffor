let order = "";
let keyCalls = 0;

function key(name) {
  keyCalls += 1;
  order += name;
  return name;
}

let laterPrivateSeen;

class Ordered {
  [key("a")]() {}
  static first = (order += "1", Ordered.later());
  static {
    order += "2";
    laterPrivateSeen = #laterPrivate in this;
  }
  [key("b")]() {}
  static #middle = (order += "3", 3);
  static {
    order += "4";
    if (!(#middle in this)) throw "middle brand";
  }
  static #laterPrivate = (order += "5", 5);
  static later() {
    return 1;
  }
}

if (keyCalls !== 2) throw "computed key count";
if (order !== "ab12345") throw "definition and static order";
if (laterPrivateSeen !== false) throw "future static private brand";

class Overwrite {
  static clash = "field";
  static clash() {
    return "method";
  }
}

if (Overwrite.clash !== "field") throw "method installation order";

let instanceOrder = "";
class InstanceOrder {
  before = (instanceOrder += "a", #later in this);
  #middle = (instanceOrder += "b", 1);
  after = (instanceOrder += "c", #middle in this);
  #later = (instanceOrder += "d", 2);
  #callable = () => 42;
  called = this.#callable();
  hasLater() {
    return #later in this;
  }
}

const instance = new InstanceOrder();
if (instanceOrder !== "abcd") throw "instance field order";
if (instance.before !== false) throw "future instance private brand";
if (instance.after !== true) throw "prior instance private brand";
if (instance.called !== 42) throw "earlier private field shape";
if (!instance.hasLater()) throw "final instance private brand";

function makePrivateReader() {
  return class PrivateReader {
    #value = 1;
    static read(target) {
      return target.#value;
    }
  };
}

function makePrivateOther() {
  return class PrivateOther {
    #value = 2;
  };
}

const PrivateReader = makePrivateReader();
const PrivateOther = makePrivateOther();
let distinctPrivateIdentity = false;
try {
  PrivateReader.read(new PrivateOther());
} catch (error) {
  distinctPrivateIdentity = error.name === "TypeError";
}
if (!distinctPrivateIdentity) throw "private identity crossed class boundary";

function makeRepeatedPrivateClass() {
  return class RepeatedPrivateClass {
    #value = 3;
    read(target) {
      return target.#value;
    }
  };
}

const RepeatedPrivateFirst = makeRepeatedPrivateClass();
const RepeatedPrivateSecond = makeRepeatedPrivateClass();
const repeatedPrivateFirst = new RepeatedPrivateFirst();
const repeatedPrivateSecond = new RepeatedPrivateSecond();
let repeatedPrivateIdentity = false;
try {
  repeatedPrivateFirst.read(repeatedPrivateSecond);
} catch (error) {
  repeatedPrivateIdentity = error.name === "TypeError";
}
if (!repeatedPrivateIdentity) throw "private identity reused across class evaluations";

let stopped = false;
try {
  class Stops {
    static {
      throw "stop";
    }
    static later = (stopped = true);
  }
} catch (error) {
  if (error !== "stop") throw error;
}
if (stopped) throw "static execution continued after throw";

let computedTdz = false;
try {
  class ComputedTdz {
    [ComputedTdz]() {}
  }
} catch (error) {
  computedTdz = error.name === "ReferenceError";
}
if (!computedTdz) throw "computed class name TDZ";

let computedFieldOrder = "";
let computedFieldKeyCalls = 0;

function fieldKey(name) {
  computedFieldKeyCalls += 1;
  computedFieldOrder += name;
  return name;
}

class ComputedFields {
  [fieldKey("a")] = 1;
  [fieldKey("b")]() {}
  static [fieldKey("c")] = (computedFieldOrder += "i", 3);
  [fieldKey("d")]() {}
}

const firstComputedInstance = new ComputedFields();
const secondComputedInstance = new ComputedFields();
if (computedFieldOrder !== "abcdi") throw "computed field definition order";
if (computedFieldKeyCalls !== 4) throw "computed field key repeated per instance";
if (firstComputedInstance.a !== 1 || secondComputedInstance.a !== 1) {
  throw "cached instance field key";
}
if (ComputedFields.c !== 3) throw "cached static field key";

let inheritedStaticSetterCalls = 0;
class StaticFieldBase {
  static set shadowed(value) {
    inheritedStaticSetterCalls += 1;
  }
}
class StaticFieldDerived extends StaticFieldBase {
  static ["shadowed"] = 5;
}
if (inheritedStaticSetterCalls !== 0 || StaticFieldDerived.shadowed !== 5) {
  throw "static field invoked inherited setter";
}

let computedFieldTdz = false;
try {
  class ComputedFieldTdz {
    [ComputedFieldTdz] = 1;
  }
} catch (error) {
  computedFieldTdz = error.name === "ReferenceError";
}
if (!computedFieldTdz) throw "computed field class name TDZ";

let missingComputedFieldName = false;
try {
  function defineMissingComputedFieldName() {
    class MissingComputedFieldName {
      [unboundComputedFieldName] = 1;
    }
  }
  defineMissingComputedFieldName();
} catch (error) {
  missingComputedFieldName = error.name === "ReferenceError";
}
if (!missingComputedFieldName) throw "missing computed field name";

const symbolFieldKey = Symbol("field");
class SymbolField {
  [symbolFieldKey] = 7;
}
if (new SymbolField()[symbolFieldKey] !== 7) throw "computed symbol field key";

let computedFieldCoercions = 0;
const objectFieldKey = {
  toString() {
    computedFieldCoercions += 1;
    return "coerced";
  },
};
class CoercedField {
  [objectFieldKey] = 9;
}
if (new CoercedField().coerced !== 9) throw "coerced computed field key";
if (new CoercedField().coerced !== 9) throw "reused coerced computed field key";
if (computedFieldCoercions !== 1) throw "computed field key coerced per instance";

const symbolFromObjectKey = Symbol("object-field");
const objectSymbolFieldKey = {
  [Symbol.toPrimitive]() {
    return symbolFromObjectKey;
  },
};
class ObjectSymbolField {
  [objectSymbolFieldKey] = 11;
}
if (new ObjectSymbolField()[symbolFromObjectKey] !== 11) {
  throw "object-coerced symbol field key";
}

true;
