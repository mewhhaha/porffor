function throwsTypeError(callback) {
  try {
    callback();
    return false;
  } catch (error) {
    return error instanceof TypeError;
  }
}

var symbol = Symbol("target");
var failures = 0;

if (Object.setPrototypeOf(true, null) !== true) failures |= 1;
if (Object.setPrototypeOf(3, null) !== 3) failures |= 2;
if (Object.setPrototypeOf("string", null) !== "string") failures |= 4;
if (Object.setPrototypeOf(symbol, null) !== symbol) failures |= 8;
if (Object.setPrototypeOf(0n, null) !== 0n) failures |= 16;
if (!throwsTypeError(function() { Object.setPrototypeOf(null, null); })) failures |= 32;
if (!throwsTypeError(function() { Object.setPrototypeOf(undefined, null); })) failures |= 64;
if (!throwsTypeError(function() { Object.setPrototypeOf(true); })) failures |= 128;
if (!throwsTypeError(function() { Object.setPrototypeOf(true, 1); })) failures |= 256;
if (!throwsTypeError(function() { Object.setPrototypeOf({}, "prototype"); })) failures |= 512;

failures === 0;
