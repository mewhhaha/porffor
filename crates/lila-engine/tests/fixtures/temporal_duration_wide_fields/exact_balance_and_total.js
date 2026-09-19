const source = Temporal.Duration.from({seconds:18446744073,nanoseconds:709551616});
const micro = source.round({largestUnit:'microsecond',smallestUnit:'nanosecond'});
if (micro.microseconds !== 18446744073709552 || micro.nanoseconds !== 616) throw 'single Number rounding';
if (micro.toString() !== 'PT18446744073.709552616S') throw 'stored precision';
if (Temporal.Duration.compare(micro.add({microseconds:1}), micro) !== 0) throw 'balanced Number precision';
const halfway = Temporal.Duration.from({seconds:9007199254,nanoseconds:740993001});
const balanced = halfway.round({largestUnit:'microsecond',smallestUnit:'nanosecond'});
if (balanced.microseconds !== 9007199254740992 || balanced.nanoseconds !== 1) throw 'integer tie to even';
if (halfway.total('microseconds') !== 9007199254740994) throw 'fractional sticky bit';
for (const [seconds,nanoseconds,unit,expected] of [
  [377110899344747,842292143,'nanoseconds',377110899344747842292143],
  [1235718035423963,836744502,'microseconds',1235718035423963836744.502],
  [1577035655772815,753554182,'milliseconds',1577035655772815753.554182],
  [0,1,'microseconds',0.001],
  [0,1,'milliseconds',0.000001]
]) {
  const duration = Temporal.Duration.from({seconds,nanoseconds});
  if (duration.total(unit) !== expected || duration.negated().total(unit) !== -expected) throw 'single rational rounding';
}
print('ok');
