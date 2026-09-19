const modes = ['ceil','floor','expand','trunc','halfCeil','halfFloor','halfExpand','halfTrunc','halfEven'];
const positive = [2n,1n,2n,1n,2n,1n,2n,1n,2n];
const negative = [-1n,-2n,-1n,-2n,-1n,-2n,-1n,-2n,-2n];
for (let i=0;i<modes.length;i++) {
  const options = {smallestUnit:'microsecond', roundingMode:modes[i]};
  if (new Temporal.Instant(1500n).round(options).epochNanoseconds !== positive[i] * 1000n) throw 'positive tie';
  if (new Temporal.Instant(-1500n).round(options).epochNanoseconds !== negative[i] * 1000n) throw 'negative tie';
}
const day = 86400000000000n;
for (const row of [[day/2n,0n],[day+day/2n,2n*day],[-day/2n,0n],[-day-day/2n,-2n*day]]) {
  if (new Temporal.Instant(row[0]).round({smallestUnit:'hour',roundingIncrement:24,roundingMode:'halfEven'}).epochNanoseconds !== row[1]) throw 'global day parity';
}
for (const epoch of [18446744073709551616n,-18446744073709551616n,8639999999999999999999n,-8639999999999999999999n]) {
  if (new Temporal.Instant(epoch).round('nanosecond').epochNanoseconds !== epoch) throw 'wide round exact';
}
if (new Temporal.Instant(-1n).round({smallestUnit:'microsecond',roundingMode:'trunc'}).epochNanoseconds !== -1000n) throw 'as-if-positive trunc';
print('ok');
