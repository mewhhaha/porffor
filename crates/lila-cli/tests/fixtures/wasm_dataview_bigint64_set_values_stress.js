let values = [
  127,
  128,
  32767,
  32768,
  2147483647,
  2147483648,
  4294967295,
  4294967296,
  -1,
  -0,
  0,
];

let buffer = new ArrayBuffer(8);
let view = new DataView(buffer);
let passed = 0;

values.forEach(function (value, i) {
  let result = view.setBigInt64(0, BigInt(value), false);
  passed = passed + Number.isInteger(value);
  passed = passed + (result === undefined);
});

passed;
