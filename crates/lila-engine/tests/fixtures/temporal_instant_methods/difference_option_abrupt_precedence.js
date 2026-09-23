const receiver = new Temporal.Instant(0n);
const other = new Temporal.Instant(1000n);
const names = ['largestUnit', 'roundingIncrement', 'roundingMode', 'smallestUnit'];
for (const difference of [Temporal.Instant.prototype.until, Temporal.Instant.prototype.since]) {
  for (const stop of [1, 2, 3]) {
    for (const phase of ['get', 'coerce']) {
      for (const marker of [undefined, {}]) {
        const log = [];
        const expected = [];
        const options = {};
        for (let index = 0; index < names.length; index++) {
          const name = names[index];
          const value = ['week', 1, 'halfFloor', 'nanosecond'][index];
          Object.defineProperty(options, name, {
            get() {
              log.push('get ' + name);
              if (index === stop && phase === 'get') throw marker;
              return {
                [Symbol.toPrimitive](hint) {
                  if (hint !== (index === 1 ? 'number' : 'string')) throw 'option conversion hint';
                  log.push('coerce ' + name);
                  if (index === stop && phase === 'coerce') throw marker;
                  return value;
                }
              };
            }
          });
          if (index <= stop) {
            expected.push('get ' + name);
            if (index !== stop || phase === 'coerce') expected.push('coerce ' + name);
          }
        }
        let caught = false;
        try { difference.call(receiver, other, options); }
        catch (error) { if (error !== marker) throw 'option abrupt identity'; caught = true; }
        if (!caught || log.join(',') !== expected.join(',')) throw 'later abrupt must precede date category rejection';
      }
    }
  }
  for (const invalid of [
    ['unknown-unit', 1, 'trunc', 'nanosecond', 0],
    ['week', 0, 'trunc', 'nanosecond', 1],
    ['week', 1, 'invalid-mode', 'nanosecond', 2]
  ]) {
    const log = [];
    const options = {};
    for (let index = 0; index < names.length; index++) {
      const name = names[index];
      Object.defineProperty(options, name, { get() { log.push(name); return invalid[index]; } });
    }
    let caught;
    try { difference.call(receiver, other, options); } catch (error) { caught = error; }
    if (!(caught instanceof RangeError)) throw 'independent option validation';
    if (log.join(',') !== names.slice(0, invalid[4] + 1).join(',')) throw 'independent validation must stop later reads';
  }
}
print('ok');
