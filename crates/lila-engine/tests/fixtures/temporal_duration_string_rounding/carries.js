function same(actual, expected) {
  if (actual !== expected) throw `${actual} !== ${expected}`;
}
const expand = { fractionalSecondDigits: 0, roundingMode: 'expand' };
for (const sign of [1, -1]) {
  const prefix = sign < 0 ? '-' : '';
  same(new Temporal.Duration(0, 0, 0, 0, sign, 59 * sign, 59 * sign, 900 * sign)
    .toString(expand), prefix + 'PT2H0S');
  same(new Temporal.Duration(sign, 11 * sign, 0, 30 * sign, 23 * sign,
    59 * sign, 59 * sign, 999 * sign, 999 * sign, 999 * sign)
    .toString({ fractionalSecondDigits: 8, roundingMode: 'expand' }),
    prefix + 'P1Y11M31DT0.00000000S');
  same(new Temporal.Duration(0, 0, 2 * sign, 0, 23 * sign, 59 * sign, 59 * sign, 999 * sign)
    .toString(expand), prefix + 'P2W1DT0S');
  same(new Temporal.Duration(0, 0, 0, 0, 0, 0, 59 * sign, 900 * sign)
    .toString(expand), prefix + 'PT60S');
  same(new Temporal.Duration(0, 0, 0, 0, 23 * sign, 59 * sign, 59 * sign, 999 * sign)
    .toString(expand), prefix + 'PT24H0S');
}
const unbalanced = new Temporal.Duration(0, 0, 0, 1, 24, 60, 60, 999, 999, 999);
same(unbalanced.toString(), 'P1DT24H60M60.999999999S');
same(unbalanced.toString({ fractionalSecondDigits: 9 }), 'P1DT24H60M60.999999999S');
same(unbalanced.toString({ fractionalSecondDigits: 0 }), 'P2DT1H1M0S');
same(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -1).toString({ fractionalSecondDigits: 0 }), 'PT0S');
same(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -1).toString({ fractionalSecondDigits: 1 }), 'PT0.0S');
print('ok');
