const result = new Temporal.Instant(0n).until(new Temporal.Instant(18446744073709551616n), {largestUnit:'microseconds'});
if (result.microseconds !== 18446744073709552 || result.nanoseconds !== 616) throw 'Duration Number precision';
if (result.toString() !== 'PT18446744073.709552616S') throw 'observable stored precision';
if (Temporal.Duration.compare(result.add({microseconds:1}), result) !== 0) throw 'subsequent Duration precision';
print('ok');
