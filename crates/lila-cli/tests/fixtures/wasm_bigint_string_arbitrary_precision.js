let maximumImmediate = BigInt("9223372036854775807");
if (maximumImmediate !== 9223372036854775807n) throw "maximum immediate";

let minimumImmediate = BigInt("-9223372036854775808");
if (minimumImmediate !== BigInt(-9223372036854775808)) throw "minimum immediate";

let positiveHeapBoundary = BigInt("9223372036854775808");
if (positiveHeapBoundary !== 9223372036854775808n) throw "positive heap boundary";

let negativeHeapBoundary = BigInt("-9223372036854775809");
if (negativeHeapBoundary !== -9223372036854775809n) throw "negative heap boundary";

let hugePositive = BigInt("340282366920938463463374607431768211457");
if (hugePositive !== 340282366920938463463374607431768211457n) {
  throw "huge positive decimal";
}

let hugeNegative = BigInt("-340282366920938463463374607431768211457");
if (hugeNegative !== -340282366920938463463374607431768211457n) {
  throw "huge negative decimal";
}

let trimmed = BigInt("\u00a0+00000000000000000000000000042\ufeff");
if (trimmed !== 42n) throw "trimmed signed decimal";

if (BigInt("") !== 0n) throw "empty string";
if (BigInt(" \t\r\n") !== 0n) throw "whitespace string";

let hexadecimal = BigInt("0x100000000000000010000000000000001");
if (hexadecimal !== 0x100000000000000010000000000000001n) throw "hexadecimal";

let octal = BigInt("0o2000000000000000000000000000000000000000001");
if (octal !== 0o2000000000000000000000000000000000000000001n) throw "octal";

let binary = BigInt("0b10000000000000000000000000000000000000000000000000000000000000001");
if (binary !== 0b10000000000000000000000000000000000000000000000000000000000000001n) {
  throw "binary";
}

__lilaAssertThrows(SyntaxError, function () {
  BigInt("+0x1");
});

__lilaAssertThrows(SyntaxError, function () {
  BigInt("0x");
});

if (BigInt.asIntN(64, "18446744073709551618") !== 2n) throw "asIntN string";
if (BigInt.asUintN(64, "-18446744073709551618") !== 18446744073709551614n) {
  throw "asUintN string";
}

1;
