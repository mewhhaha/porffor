const marker = {};
const receiver = new Temporal.Instant(0n);
let trace = '';
const getter = {get [Symbol.toPrimitive]() { trace += 'get'; throw marker; }, toString() { throw 'late toString'; }};
const method = {get [Symbol.toPrimitive]() { trace += 'get,'; return function(hint) { trace += hint; throw marker; }; }, valueOf() { throw 'late valueOf'; }};
const options = {get largestUnit(){throw 'options after abrupt';}};
for (const convert of [input => Temporal.Instant.from(input), input => receiver.until(input, options), input => receiver.since(input, options)]) {
  for (const input of [getter, method]) {
    trace = '';
    let caught;
    try { convert(input); } catch (error) { caught = error; }
    if (caught !== marker || trace !== (input === getter ? 'get' : 'get,string')) throw 'primitive abrupt identity or order';
  }
}
for (const difference of [Temporal.Instant.prototype.until, Temporal.Instant.prototype.since]) {
  trace = '';
  let caught;
  try { difference.call({}, getter, options); } catch (error) { caught = error; }
  if (!(caught instanceof TypeError) || trace !== '') throw 'brand before primitive hook';
}
trace = '';
const input = {
  get [Symbol.toPrimitive]() { trace += 'get,'; return undefined; },
  toString() { trace += 'toString,'; return {}; },
  valueOf() { trace += 'valueOf'; return '1970-01-01T00:00:00.000000123Z'; }
};
if (Temporal.Instant.from(input).epochNanoseconds !== 123n || trace !== 'get,toString,valueOf') throw 'ordinary primitive order';
trace = '';
const observedOptions = {get largestUnit(){trace += ',largest';return 'nanosecond';}};
if (receiver.until(input, observedOptions).nanoseconds !== 123 || trace !== 'get,toString,valueOf,largest') throw 'options after conversion';
print('ok');
