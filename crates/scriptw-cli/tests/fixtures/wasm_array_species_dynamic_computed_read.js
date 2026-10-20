function read(obj, key) {
  return obj[key];
}

function hasExpectedElement(result) {
  let desc = Object.getOwnPropertyDescriptor(result, "0");
  return desc.value === 2
    && desc.writable === true
    && desc.enumerable === true
    && desc.configurable === true
    && read(result, "0") === 2;
}

let flatTarget = function (_length) {
  Object.defineProperty(this, "0", {
    value: 17,
    writable: false,
    enumerable: false,
    configurable: true
  });
};
let flatSource = [[2]];
flatSource.constructor = {
  get [Symbol.species]() {
    return flatTarget;
  }
};
let flatResult = flatSource.flat();

let flatMapTarget = function (_length) {
  Object.defineProperty(this, "0", {
    value: 17,
    writable: false,
    enumerable: false,
    configurable: true
  });
};
let flatMapSource = [2];
flatMapSource.constructor = {
  get [Symbol.species]() {
    return flatMapTarget;
  }
};
let flatMapResult = flatMapSource.flatMap(function (value) {
  return value;
});

let concatTarget = function (_length) {
  Object.defineProperty(this, "0", {
    value: 17,
    writable: false,
    enumerable: false,
    configurable: true
  });
};
let concatSource = [];
concatSource.constructor = {
  get [Symbol.species]() {
    return concatTarget;
  }
};
let concatResult = concatSource.concat([2]);

hasExpectedElement(flatResult)
  && hasExpectedElement(flatMapResult)
  && hasExpectedElement(concatResult);
