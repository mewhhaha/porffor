let signedWords = new BigInt64Array([
  "18446744073709551618",
  "-18446744073709551618",
  "340282366920938463463374607431768211457",
]);
if (signedWords[0] !== 2n) throw "signed positive modulo";
if (signedWords[1] !== -2n) throw "signed negative modulo";
if (signedWords[2] !== 1n) throw "signed huge modulo";

let unsignedWords = new BigUint64Array([
  "18446744073709551618",
  "-18446744073709551618",
]);
if (unsignedWords[0] !== 2n) throw "unsigned positive modulo";
if (unsignedWords[1] !== 18446744073709551614n) throw "unsigned negative modulo";

let buffer = new ArrayBuffer(16);
let view = new DataView(buffer);
view.setBigInt64(0, "-18446744073709551618");
view.setBigUint64(8, "340282366920938463463374607431768211457");
if (view.getBigInt64(0) !== -2n) throw "DataView signed modulo";
if (view.getBigUint64(8) !== 1n) throw "DataView unsigned modulo";

1;
