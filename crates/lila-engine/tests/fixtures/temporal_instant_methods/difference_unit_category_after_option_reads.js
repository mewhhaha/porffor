const receiver = new Temporal.Instant(0n);
const other = new Temporal.Instant(1000n);
const names = ['largestUnit', 'roundingIncrement', 'roundingMode', 'smallestUnit'];
const expected = [];
for (const name of names) {
  const hook = name === 'roundingIncrement' ? 'valueOf' : 'toString';
  expected.push('get ' + name, 'get ' + name + '.' + hook, 'call ' + name + '.' + hook);
}
for (const difference of [Temporal.Instant.prototype.until, Temporal.Instant.prototype.since]) {
  for (const largestUnit of ['year', 'years', 'month', 'months', 'week', 'weeks', 'day', 'days']) {
    const log = [];
    function observed(name, value) {
      log.push('get ' + name);
      if (name === 'roundingIncrement') return {
        get valueOf() {
          log.push('get ' + name + '.valueOf');
          return function () { log.push('call ' + name + '.valueOf'); return value; };
        }
      };
      return {
        get toString() {
          log.push('get ' + name + '.toString');
          return function () { log.push('call ' + name + '.toString'); return value; };
        }
      };
    }
    const options = {
      get largestUnit() { return observed('largestUnit', largestUnit); },
      get roundingIncrement() { return observed('roundingIncrement', 1); },
      get roundingMode() { return observed('roundingMode', 'halfFloor'); },
      get smallestUnit() { return observed('smallestUnit', 'nanosecond'); }
    };
    let caught;
    try { difference.call(receiver, other, options); } catch (error) { caught = error; }
    if (!(caught instanceof RangeError)) throw 'recognized date unit must fail';
    if (log.join(',') !== expected.join(',')) throw 'date category before complete option reads: ' + log.join(',');
  }
  for (const units of [['hour', 'day'], ['nanosecond', 'hour']]) {
    const log = [];
    const options = {
      get largestUnit() { log.push('largestUnit'); return units[0]; },
      get roundingIncrement() { log.push('roundingIncrement'); return 1; },
      get roundingMode() { log.push('roundingMode'); return 'trunc'; },
      get smallestUnit() { log.push('smallestUnit'); return units[1]; }
    };
    let caught;
    try { difference.call(receiver, other, options); } catch (error) { caught = error; }
    if (!(caught instanceof RangeError) || log.join(',') !== names.join(',')) throw 'unit relationship validation order';
  }
}
print('ok');
