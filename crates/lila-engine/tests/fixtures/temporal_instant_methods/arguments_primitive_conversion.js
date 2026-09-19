const receiver = new Temporal.Instant(0n);
let calls = 0;
function checkArguments() {
  const input = arguments;
  input[Symbol.toPrimitive] = function(hint) {
    if (hint !== 'string' || this !== input) throw 'Arguments primitive receiver or hint';
    calls++;
    return '1970-01-01T00:00:00.000000123Z';
  };
  if (Temporal.Instant.from(input).epochNanoseconds !== 123n) throw 'direct Arguments conversion';
  const from = Temporal.Instant.from;
  if (from(input).epochNanoseconds !== 123n) throw 'dynamic Arguments conversion';
  if (receiver.until(input).nanoseconds !== 123) throw 'until Arguments conversion';
  if (receiver.since(input).nanoseconds !== -123) throw 'since Arguments conversion';
}
checkArguments();
if (calls !== 4) throw 'Arguments hook count';
for (const makeInput of [() => [], () => function() {}, () => ({})]) {
  const input = makeInput();
  input[Symbol.toPrimitive] = function(hint) {
    if (hint !== 'string' || this !== input) throw 'object primitive receiver or hint';
    return '1970-01-01T00:00:00.000000123Z';
  };
  if (Temporal.Instant.from(input).epochNanoseconds !== 123n) throw 'ordinary object kind conversion';
}
print('ok');
