const instant = new Temporal.Instant(1n);
for (const row of [['hour',24],['minute',1440],['second',86400],['millisecond',86400000],['second',864],['nanosecond',1000000000]]) {
  const result = instant.round({smallestUnit:row[0],roundingIncrement:row[1]});
  if (result.epochNanoseconds !== 0n) throw 'valid full-day divisor';
}
const log = [];
const options = {
  get roundingIncrement(){log.push('increment'); return {valueOf(){log.push('increment.valueOf');return 25;}};},
  get roundingMode(){log.push('mode');return {toString(){log.push('mode.toString');return 'expand';}};},
  get smallestUnit(){log.push('unit');return {toString(){log.push('unit.toString');return 'hour';}};}
};
let caught = false;
try { instant.round(options); } catch (error) { if (!(error instanceof RangeError)) throw error; caught = true; }
if (!caught || log.join(',') !== 'increment,increment.valueOf,mode,mode.toString,unit,unit.toString') throw 'round option order';
for (const options of [{}, {smallestUnit:'day'}, {smallestUnit:'hour',roundingIncrement:7}, {smallestUnit:'nanosecond',roundingIncrement:1000000001}]) {
  let caught = false;
  try { instant.round(options); } catch (error) { if (!(error instanceof RangeError)) throw error; caught = true; }
  if (!caught) throw 'invalid round option';
}
let reads = 0;
caught = false;
try { Temporal.Instant.prototype.round.call({}, {get smallestUnit(){reads++;return 'hour';}}); } catch (error) { if (!(error instanceof TypeError)) throw error; caught = true; }
if (!caught || reads !== 0) throw 'round receiver brand';
Object.defineProperty(Object.prototype, 'roundingMode', {configurable:true,get(){throw 'shorthand prototype';}});
if (instant.round('microseconds').epochNanoseconds !== 0n) throw 'plural shorthand';
delete Object.prototype.roundingMode;
function argumentOptions() {
  arguments.smallestUnit = 'microsecond';
  if (instant.round(arguments).epochNanoseconds !== 0n) throw 'Arguments options';
}
argumentOptions();
print('ok');
