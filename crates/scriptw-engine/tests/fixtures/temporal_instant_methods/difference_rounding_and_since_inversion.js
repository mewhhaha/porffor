const modes = ['ceil','floor','expand','trunc','halfCeil','halfFloor','halfExpand','halfTrunc','halfEven'];
const positive = [2,1,2,1,2,1,2,1,2];
const negative = [-1,-2,-2,-1,-1,-2,-2,-1,-2];
const zero = new Temporal.Instant(0n);
for (let i=0;i<modes.length;i++) {
  const options = {largestUnit:'microsecond',smallestUnit:'microsecond',roundingMode:modes[i]};
  if (zero.until(new Temporal.Instant(1500n), options).microseconds!==positive[i]) throw 'positive until';
  if (zero.until(new Temporal.Instant(-1500n), options).microseconds!==negative[i]) throw 'negative until';
  if (zero.since(new Temporal.Instant(1500n), options).microseconds!==negative[i]) throw 'negative since';
  if (zero.since(new Temporal.Instant(-1500n), options).microseconds!==positive[i]) throw 'positive since';
}
let caught = false;
try { zero.until(zero, {smallestUnit:'hour',roundingIncrement:24}); } catch (error) { if (!(error instanceof RangeError)) throw error; caught=true; }
if (!caught) throw 'difference uses exclusive next-unit bound';
print('ok');
