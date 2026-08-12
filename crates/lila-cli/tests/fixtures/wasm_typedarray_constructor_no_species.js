let grossBufferConstructionCount = 0;
let speciesGetterCount = 0;
let throwOnGrossBufferConstruction = false;

let prototypeTarget = {};
let prototypeValue = { marker: 1 };
prototypeTarget.__proto__ = prototypeValue;
if (Object.getPrototypeOf(prototypeTarget) !== prototypeValue) {
  throw "prototype setter did not update prototype";
}
prototypeTarget.__proto__ = 1;
if (Object.getPrototypeOf(prototypeTarget) !== prototypeValue) {
  throw "prototype setter accepted a primitive prototype";
}

class GrossBuffer extends ArrayBuffer {
  constructor() {
    grossBufferConstructionCount = grossBufferConstructionCount + 1;
    super(...arguments);
    if (throwOnGrossBufferConstruction) throw "gross buffer reconstructed";
  }

  static get [Symbol.species]() {
    speciesGetterCount = speciesGetterCount + 1;
    throw "species getter called";
  }
}

let grossBuffer = new GrossBuffer(4);
throwOnGrossBufferConstruction = true;
let grossTypedArray = new Uint8Array(grossBuffer);
grossTypedArray[0] = 23;
let clonedTypedArray = new Int8Array(grossTypedArray);

if (grossBufferConstructionCount !== 1) throw "unexpected buffer construction count";
if (speciesGetterCount !== 0) throw "unexpected species getter count";
if (clonedTypedArray.length !== 4) throw "unexpected clone length";
if (clonedTypedArray[0] !== 23) throw "unexpected clone contents";
if (Object.getPrototypeOf(clonedTypedArray.buffer) !== ArrayBuffer.prototype) {
  throw "unexpected direct clone buffer prototype";
}
if (clonedTypedArray.buffer.__proto__ !== ArrayBuffer.prototype) {
  throw "unexpected clone buffer prototype";
}
if (clonedTypedArray.buffer.constructor !== ArrayBuffer) {
  throw "unexpected clone buffer constructor";
}

123;
