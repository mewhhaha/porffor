const cases = [0n, -1n, 1n, 9223372036854775807n, 9223372036854775808n, -9223372036854775808n, -9223372036854775809n, 18446744073709551616n];
for (const epoch of cases) {
  const instant = new Temporal.Instant(epoch);
  if (instant.add({nanoseconds:3}).epochNanoseconds !== epoch + 3n) throw 'add boundary';
  if (instant.subtract({nanoseconds:3}).epochNanoseconds !== epoch - 3n) throw 'subtract boundary';
  if (instant.add('-PT1.999999999S').epochNanoseconds !== epoch - 1999999999n) throw 'negative string';
  if (instant.subtract(new Temporal.Duration(0,0,0,0,0,0,1,999,999,999)).epochNanoseconds !== epoch - 1999999999n) throw 'Duration instance';
}
if (new Temporal.Instant(-900000000n).add({nanoseconds:-900000000}).epochNanoseconds !== -1800000000n) throw 'negative carry';
if (new Temporal.Instant(900000000n).subtract({nanoseconds:-900000000}).epochNanoseconds !== 1800000000n) throw 'positive carry';
if (new Temporal.Instant(-500000000n).add({seconds:1}).epochNanoseconds !== 500000000n) throw 'cross zero';
if (new Temporal.Instant(500000000n).subtract({seconds:1}).epochNanoseconds !== -500000000n) throw 'cross zero';
const nanos = Number.MAX_SAFE_INTEGER;
if (new Temporal.Instant(10n).add({nanoseconds:nanos}).epochNanoseconds !== 9007199254741001n) throw 'large nanoseconds';
if (new Temporal.Instant(10n).subtract({microseconds:nanos}).epochNanoseconds !== -9007199254740990990n) throw 'large microseconds';
print('ok');
