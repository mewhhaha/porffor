function maxByteLengthOptions() {}
maxByteLengthOptions.maxByteLength = 4;
let shared = new SharedArrayBuffer(2, maxByteLengthOptions);
let initial = new Uint8Array(shared);
initial[0] = 41;
initial[1] = 42;

if (shared.grow(4) !== undefined) throw "grow result";
if (shared.byteLength !== 4) throw "grown byteLength";
if (shared.maxByteLength !== 4) throw "maxByteLength";
if (!shared.growable) throw "growable";

let grown = new Uint8Array(shared);
if (grown[0] !== 41 || grown[1] !== 42) throw "grown prefix";
if (grown[2] !== 0 || grown[3] !== 0) throw "grown zero fill";

let speciesResult;
shared.constructor = {
  [Symbol.species]: function(length) {
    speciesResult = new SharedArrayBuffer(length);
    return speciesResult;
  }
};
let sliced = shared.slice(1, 3);
if (sliced !== speciesResult) throw "species result";
if (sliced.byteLength !== 2) throw "slice byteLength";
let slicedBytes = new Uint8Array(sliced);
if (slicedBytes[0] !== 42 || slicedBytes[1] !== 0) throw "slice bytes";

123;
