for (let radix = 11; radix <= 36; radix++) {
  for (let digit = 10n; digit < radix; digit++) {
    let expected = String.fromCharCode(Number(digit + 87n));
    if (digit.toString(radix) !== expected) {
      throw "BigInt radix digit";
    }
  }
}

if (!(10n < 11) || !(11 > 10n) || !(10n <= 10) || !(10 >= 10n)) {
  throw "integral Number comparison";
}
if (!(10n < 10.5) || !(10.5 > 10n) || !(10n > 9.5) || !(9.5 < 10n)) {
  throw "fractional Number comparison";
}
if (10n < NaN || NaN < 10n || 10n >= NaN || NaN >= 10n) {
  throw "NaN comparison";
}
if (!(10n < Infinity) || !(-Infinity < 10n)) {
  throw "infinite Number comparison";
}
if (
  !(9007199254740995n < 9007199254740996) ||
  !(9007199254740996 > 9007199254740995n)
) {
  throw "exact integral Number comparison";
}

1;
