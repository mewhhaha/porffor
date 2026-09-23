function range(action) { let caught; try { action(); } catch (error) { caught=error; } if (!(caught instanceof RangeError)) throw 'expected RangeError'; }
for (const field of ['years','months','weeks']) {
  const bag={}; bag[field]=4294967295;
  if (Temporal.Duration.from(bag)[field] !== bag[field]) throw 'calendar upper bound';
  bag[field]=4294967296; range(() => Temporal.Duration.from(bag));
}
for (const [field,bound] of [['seconds',9007199254740992],['milliseconds',9007199254740992000],['microseconds',9007199254740992000000],['nanoseconds',9007199254740992000000000]]) {
  for (const sign of [-1,1]) { const bag={}; bag[field]=sign*bound; range(() => Temporal.Duration.from(bag)); }
}
const maximum = Temporal.Duration.from({seconds:9007199254740991,nanoseconds:999999999});
if (maximum.toString() !== 'PT9007199254740991.999999999S') throw 'strict canonical time bound';
range(() => maximum.add({nanoseconds:1}));
range(() => maximum.round({largestUnit:'nanosecond',smallestUnit:'nanosecond'}));
range(() => maximum.round({largestUnit:'microsecond',smallestUnit:'nanosecond'}));
range(() => Temporal.Duration.from({seconds:9007199254740989,nanoseconds:3000000000}));
if (Temporal.Duration.from({seconds:9007199254740989,nanoseconds:2000000000}).seconds !== 9007199254740989) throw 'do not balance input storage';
range(() => Temporal.Duration.from({microseconds:17280000000000000000,nanoseconds:-1}));
range(() => Temporal.Duration.from({nanoseconds:Infinity}));
range(() => Temporal.Duration.from({nanoseconds:NaN}));
print('ok');
