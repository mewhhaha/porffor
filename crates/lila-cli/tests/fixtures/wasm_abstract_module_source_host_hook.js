function AbstractModuleSource() {
  throw new TypeError();
}

function AbstractModuleSourceToStringTag() {
  return undefined;
}

Object.defineProperty(AbstractModuleSource, "prototype", {
  value: AbstractModuleSource.prototype,
  writable: false,
  enumerable: false,
  configurable: false
});

Object.defineProperty(AbstractModuleSource.prototype, Symbol.toStringTag, {
  get: AbstractModuleSourceToStringTag,
  set: undefined,
  enumerable: false,
  configurable: true
});

var $262 = {
  AbstractModuleSource: AbstractModuleSource
};

if (typeof $262.AbstractModuleSource !== "function") throw "constructor type";

let lengthDesc = Object.getOwnPropertyDescriptor($262.AbstractModuleSource, "length");
if (lengthDesc.value !== 0) throw "length value";
if (lengthDesc.writable !== false) throw "length writable";
if (lengthDesc.enumerable !== false) throw "length enumerable";
if (lengthDesc.configurable !== true) throw "length configurable";

let nameDesc = Object.getOwnPropertyDescriptor($262.AbstractModuleSource, "name");
if (nameDesc.value !== "AbstractModuleSource") throw "name value";
if (nameDesc.writable !== false) throw "name writable";
if (nameDesc.enumerable !== false) throw "name enumerable";
if (nameDesc.configurable !== true) throw "name configurable";

let prototypeDesc = Object.getOwnPropertyDescriptor($262.AbstractModuleSource, "prototype");
if (prototypeDesc.value !== $262.AbstractModuleSource.prototype) throw "prototype value";
if (prototypeDesc.writable !== false) throw "prototype writable";
if (prototypeDesc.enumerable !== false) throw "prototype enumerable";
if (prototypeDesc.configurable !== false) throw "prototype configurable";

let constructorDesc = Object.getOwnPropertyDescriptor(
  $262.AbstractModuleSource.prototype,
  "constructor"
);
if (constructorDesc.value !== $262.AbstractModuleSource) throw "constructor value";
if (constructorDesc.writable !== true) throw "constructor writable";
if (constructorDesc.enumerable !== false) throw "constructor enumerable";
if (constructorDesc.configurable !== true) throw "constructor configurable";

if (Object.getPrototypeOf($262.AbstractModuleSource) !== Function.prototype) {
  throw "constructor prototype chain";
}
if (Object.getPrototypeOf($262.AbstractModuleSource.prototype) !== Object.prototype) {
  throw "prototype prototype chain";
}

let tagDesc = Object.getOwnPropertyDescriptor(
  $262.AbstractModuleSource.prototype,
  Symbol.toStringTag
);
if (typeof tagDesc.get !== "function") throw "toStringTag getter";
if (tagDesc.set !== undefined) throw "toStringTag setter";
if (tagDesc.enumerable !== false) throw "toStringTag enumerable";
if (tagDesc.configurable !== true) throw "toStringTag configurable";
if (tagDesc.get.call(262) !== undefined) throw "toStringTag primitive";
if (tagDesc.get.call($262.AbstractModuleSource.prototype) !== undefined) {
  throw "toStringTag prototype";
}

let threw = false;
try {
  new $262.AbstractModuleSource();
} catch (error) {
  threw = true;
}
if (!threw) throw "constructor did not throw";

262;
