let zeroWidthInput = 0xabcdef0123456789abcdef0123456789n;
if (BigInt.asUintN(0, zeroWidthInput) !== 0n) throw "zero-bit unsigned";
if (BigInt.asIntN(0, zeroWidthInput) !== 0n) throw "zero-bit signed";
if (zeroWidthInput !== 0xabcdef0123456789abcdef0123456789n) {
  throw "zero-bit input mutated";
}

if (BigInt.asUintN(1, -1n) !== 1n) throw "one-bit unsigned";
if (BigInt.asIntN(1, 1n) !== -1n) throw "one-bit signed";
if (BigInt.asUintN(63, -1n) !== 0x7fffffffffffffffn) {
  throw "63-bit unsigned";
}
if (BigInt.asIntN(63, 0x4000000000000000n) !== -0x4000000000000000n) {
  throw "63-bit signed minimum";
}
if (BigInt.asUintN(64, -1n) !== 0xffffffffffffffffn) {
  throw "64-bit unsigned";
}
if (BigInt.asIntN(64, 0x8000000000000000n) !== -0x8000000000000000n) {
  throw "64-bit signed minimum";
}

let positive = 0xabcdef0123456789abcdefn;
if (BigInt.asUintN(65, positive) !== 0x10123456789abcdefn) {
  throw "65-bit unsigned positive";
}
if (BigInt.asIntN(65, positive) !== -0xfedcba9876543211n) {
  throw "65-bit signed positive";
}
if (positive !== 0xabcdef0123456789abcdefn) throw "positive input mutated";

let negative = -0x10000000000000001n;
if (BigInt.asUintN(65, negative) !== 0xffffffffffffffffn) {
  throw "65-bit unsigned negative";
}
if (BigInt.asIntN(65, negative) !== 0xffffffffffffffffn) {
  throw "65-bit signed negative wrap";
}
if (negative !== -0x10000000000000001n) throw "negative input mutated";

if (BigInt.asUintN(65, -1n) !== 0x1ffffffffffffffffn) {
  throw "65-bit unsigned minus one";
}
if (BigInt.asIntN(65, 0x10000000000000000n) !== -0x10000000000000000n) {
  throw "65-bit signed minimum";
}

let wide = 0xc89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n;
if (
  BigInt.asIntN(200, wide) !==
  -0x761f7e209749a0124cd3001599f1aa2069fa9af59fc52a03acn
) {
  throw "200-bit signed";
}
if (
  BigInt.asIntN(201, wide) !==
  0x89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n
) {
  throw "201-bit signed";
}

let unsignedWide = 0xb89e081df68b65fedb32cffea660e55df9605650a603ad5fc54n;
if (
  BigInt.asUintN(200, unsignedWide) !==
  0x089e081df68b65fedb32cffea660e55df9605650a603ad5fc54n
) {
  throw "200-bit unsigned";
}
if (
  BigInt.asUintN(201, unsignedWide) !==
  0x189e081df68b65fedb32cffea660e55df9605650a603ad5fc54n
) {
  throw "201-bit unsigned";
}

if (
  BigInt.asUintN(200, "-1606938044258990275541962092341162602522202993782792835301377") !==
  0xffffffffffffffffffffffffffffffffffffffffffffffffffn
) {
  throw "200-bit unsigned parsed negative";
}

let conversionOrder = "";
let ordered = BigInt.asUintN(
  {
    valueOf() {
      conversionOrder += "bits";
      return 65;
    },
  },
  {
    valueOf() {
      conversionOrder += ",bigint";
      return 0xabcdef0123456789abcdefn;
    },
  },
);
if (conversionOrder !== "bits,bigint") throw "conversion order";
if (ordered !== 0x10123456789abcdefn) throw "ordered conversion result";

let bigintConversionReached = false;
try {
  BigInt.asIntN(
    {
      valueOf() {
        throw "bits abrupt";
      },
    },
    {
      valueOf() {
        bigintConversionReached = true;
        return 1n;
      },
    },
  );
  throw "missing bits abrupt";
} catch (error) {
  if (error !== "bits abrupt") throw error;
}
if (bigintConversionReached) throw "bigint converted after bits abrupt";

1;
