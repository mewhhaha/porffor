let seenThis;
let seenValue;
let seenIndex;
let seenArgsLength;
let thisArg = { marker: 262 };

let doubled = Uint8Array.from([2, 4], function (value, index) {
  seenThis = this;
  seenValue = value;
  seenIndex = index;
  seenArgsLength = arguments.length;
  return value * 2;
}, thisArg);

if (doubled.length !== 2) throw "mapper length";
if (doubled[0] !== 4 || doubled[1] !== 8) throw "mapper doubled values";
if (seenThis !== thisArg) throw "mapper thisArg";
if (seenValue !== 4 || seenIndex !== 1) throw "mapper arguments";
if (seenArgsLength !== 2) throw "mapper arguments length";

let withoutThis = Float32Array.from([3], function (value, index) {
  if (index !== 0) throw "mapper index without thisArg";
  return value + 1;
});
if (withoutThis[0] !== 4) throw "mapper without thisArg value";

let mapperThrew = false;
try {
  Uint8Array.from([1], function () {
    throw new RangeError("mapper abrupt");
  });
} catch (error) {
  mapperThrew = error instanceof RangeError;
}
if (!mapperThrew) throw "mapper abrupt completion";

let conversionThrew = false;
try {
  Uint8Array.from([1], function () {
    return Symbol();
  });
} catch (error) {
  conversionThrew = error instanceof TypeError;
}
if (!conversionThrew) throw "mapper conversion abrupt";

262;
