let constructorArgumentsAreUnmapped = false;
let instanceSetterArgumentsAreUnmapped = false;
let staticSetterArgumentsAreUnmapped = false;

class StrictClassCallables {
  constructor(value) {
    value = 2;
    constructorArgumentsAreUnmapped = arguments.length === 1 && arguments[0] === 1;
  }

  method(value) {
    value = 2;
    return arguments.length === 1 && arguments[0] === 1;
  }

  static method(value) {
    value = 2;
    return arguments.length === 1 && arguments[0] === 1;
  }

  get value() {
    return arguments.length;
  }

  set value(next) {
    next = 2;
    instanceSetterArgumentsAreUnmapped = arguments[0] === 1;
  }

  static get value() {
    return arguments.length;
  }

  static set value(next) {
    next = 2;
    staticSetterArgumentsAreUnmapped = arguments[0] === 1;
  }
}

new StrictClassCallables(1);

const instanceMethod = StrictClassCallables.prototype.method;
const staticMethod = StrictClassCallables.method;
const instanceValue = Object.getOwnPropertyDescriptor(
  StrictClassCallables.prototype,
  "value",
);
const staticValue = Object.getOwnPropertyDescriptor(StrictClassCallables, "value");

instanceValue.set.call({}, 1);
staticValue.set.call({}, 1);

function ordinary(value) {
  value = 2;
  return arguments[0] === 2;
}

const ordinaryObject = {
  method(value) {
    value = 2;
    return arguments[0] === 2;
  },
};

if (!constructorArgumentsAreUnmapped) throw "constructor arguments mapping";
if (!instanceMethod.call({}, 1)) throw "instance method arguments mapping";
if (!staticMethod.call({}, 1)) throw "static method arguments mapping";
if (instanceValue.get.call({}) !== 0) throw "instance getter arguments";
if (staticValue.get.call({}) !== 0) throw "static getter arguments";
if (!instanceSetterArgumentsAreUnmapped) throw "instance setter arguments mapping";
if (!staticSetterArgumentsAreUnmapped) throw "static setter arguments mapping";
if (!ordinary(1)) throw "ordinary function mapped arguments control";
if (!ordinaryObject.method(1)) throw "object method mapped arguments control";

true;
