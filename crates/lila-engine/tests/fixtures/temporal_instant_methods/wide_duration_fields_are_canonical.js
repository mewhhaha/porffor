const limit = 8640000000000000000000n;
const min = new Temporal.Instant(-limit);
const max = new Temporal.Instant(limit);
for (const field of ['nanoseconds','microseconds','milliseconds','seconds']) {
  const scale = field === 'nanoseconds' ? 1 : field === 'microseconds' ? 1000 : field === 'milliseconds' ? 1000000 : 1000000000;
  const bag = {}; bag[field] = 17280000000000000000000 / scale;
  const duration = Temporal.Duration.from(bag);
  if (min.add(bag).epochNanoseconds !== limit || min.add(duration).epochNanoseconds !== limit) throw 'wide addition';
  if (max.subtract(duration).epochNanoseconds !== -limit) throw 'wide subtraction';
  const difference = min.until(max, {largestUnit:field});
  if (difference[field] !== bag[field] || min.add(difference).epochNanoseconds !== limit) throw 'wide result round trip';
  if (difference.negated()[field] !== -bag[field]) throw 'wide Duration representation';
}
print('ok');
