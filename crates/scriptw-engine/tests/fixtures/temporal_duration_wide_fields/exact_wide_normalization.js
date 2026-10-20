for (const [name,value,expected] of [
  ['milliseconds',9007199254740990976,'PT9007199254740990.976S'],
  ['microseconds',9007199254740990951424,'PT9007199254740990.951424S'],
  ['nanoseconds',9007199254740990926258176,'PT9007199254740990.926258176S'],
  ['nanoseconds',18446744073709551616,'PT18446744073.709551616S']
]) {
  const bag = {}; bag[name] = value;
  const duration = Temporal.Duration.from(bag);
  if (duration[name] !== value || duration.toString() !== expected) throw 'exact positive normalization';
  if (duration.negated().toString() !== '-' + expected) throw 'exact negative normalization';
  if (Temporal.Duration.compare(duration, Temporal.Duration.from(expected)) !== 0) throw 'string normalization equality';
}
if (Temporal.Duration.from('PT0000000000000000000000000000001.000000001S').toString() !== 'PT1.000000001S') throw 'leading zero duration';
if (Temporal.Duration.from('PT46H66M71.50040904S').toString() !== 'PT46H66M71.50040904S') throw 'exact fractional string';
print('ok');
