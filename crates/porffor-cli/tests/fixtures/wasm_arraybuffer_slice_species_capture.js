let resultBuffer;
let speciesConstructor = {};
speciesConstructor[Symbol.species] = function(length) {
  resultBuffer = new ArrayBuffer(length);
  return resultBuffer;
};

let arrayBuffer = new ArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

let result = arrayBuffer.slice();
if (result !== resultBuffer) throw "species result identity";
if (result.byteLength !== 8) throw "species result length";

123;
