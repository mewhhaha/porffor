const limit = 8640000000000000000000n;
const earliest = new Temporal.Instant(-limit);
const latest = new Temporal.Instant(limit);
if (earliest.add({seconds:17280000000000}).epochNanoseconds !== limit) throw 'whole range add';
if (latest.subtract({seconds:17280000000000}).epochNanoseconds !== -limit) throw 'whole range subtract';
if (new Temporal.Instant(0n).add('PT2400000000H').epochNanoseconds !== limit) throw 'maximum hour string';
if (new Temporal.Instant(0n).subtract('PT144000000000M').epochNanoseconds !== -limit) throw 'minimum minute string';
if (earliest.add({nanoseconds:1}).epochNanoseconds !== -limit + 1n) throw 'earliest neighbor';
if (latest.subtract({nanoseconds:1}).epochNanoseconds !== limit - 1n) throw 'latest neighbor';
for (const operation of [()=>latest.add({nanoseconds:1}),()=>earliest.subtract({nanoseconds:1}),()=>earliest.add({nanoseconds:-1}),()=>latest.subtract({nanoseconds:-1})]) {
  let caught = false;
  try { operation(); } catch (error) { if (!(error instanceof RangeError)) throw error; caught = true; }
  if (!caught) throw 'one nanosecond outside range';
}
print('ok');
