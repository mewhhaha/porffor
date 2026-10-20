const log = [];
const receiver = new Temporal.Instant(0n);
const other = {toString(){log.push('other');return '1970-01-01T00:00:00.0000015Z';}};
const options = {
  get largestUnit(){log.push('largest');return 'microsecond';},
  get roundingIncrement(){log.push('increment');return 1;},
  get roundingMode(){log.push('mode');return 'halfExpand';},
  get smallestUnit(){log.push('smallest');return 'microsecond';}
};
const from = Temporal.Instant.from;
const durationFrom = Temporal.Duration.from;
Temporal.Instant.from = function(){throw 'public Instant.from';};
Temporal.Duration.from = function(){throw 'public Duration.from';};
if (receiver.until(other, options).microseconds!==2) throw 'canonical conversion';
if (log.join(',')!=='other,largest,increment,mode,smallest') throw 'difference order';
Temporal.Instant.from = from;
Temporal.Duration.from = durationFrom;
log.length=0;
let caught=false;
try { receiver.until(other, {get largestUnit(){log.push('largest');return 'day';},get roundingIncrement(){log.push('increment');throw 'late getter';}}); }
catch(error){if(error!=='late getter')throw error;caught=true;}
if(!caught||log.join(',')!=='other,largest,increment')throw 'option abrupt before category validation';
log.length=0;
caught=false;
try { receiver.until(other, null); } catch(error){if(!(error instanceof TypeError))throw error;caught=true;}
if(!caught||log.join(',')!=='other')throw 'other conversion before options';
log.length=0;
caught=false;
try { Temporal.Instant.prototype.since.call({}, other, options); } catch(error){if(!(error instanceof TypeError))throw error;caught=true;}
if(!caught||log.length!==0)throw 'difference brand before conversion';
function argumentOptions() {
  arguments.smallestUnit = 'microsecond';
  arguments.roundingMode = 'halfExpand';
  if (receiver.until(new Temporal.Instant(1500n), arguments).microseconds !== 2) throw 'Arguments options';
}
argumentOptions();
print('ok');
