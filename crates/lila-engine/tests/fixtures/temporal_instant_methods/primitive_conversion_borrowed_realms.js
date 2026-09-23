const foreign = __lilaCreateRealm().global;
const I = foreign.Temporal.Instant;
const D = foreign.Temporal.Duration;
const instantPrototype = I.prototype;
const durationPrototype = D.prototype;
const typeErrorPrototype = foreign.TypeError.prototype;
const from = I.from;
const until = instantPrototype.until;
const since = instantPrototype.since;
const receiver = new Temporal.Instant(0n);
foreign.Temporal.Instant = function(){throw 'public foreign Instant';};
foreign.Temporal.Duration = function(){throw 'public foreign Duration';};
foreign.TypeError = function(){throw 'public foreign TypeError';};
I.from = function(){throw 'public foreign from';};
const input = {[Symbol.toPrimitive](hint) {
  if (hint !== 'string') throw 'hint';
  // A nested entry-Realm call cannot replace the borrowed builtin's owner.
  if (Object.getPrototypeOf(Temporal.Instant.from('1970-01-01T00:00Z')) !== Temporal.Instant.prototype) throw 'nested entry Realm';
  return '1970-01-01T00:00Z';
}};
if (Object.getPrototypeOf(from(input)) !== instantPrototype) throw 'borrowed from result Realm';
for (const difference of [until, since]) {
  const result = difference.call(receiver, input);
  if (Object.getPrototypeOf(result) !== durationPrototype) throw 'borrowed difference result Realm';
  for (const field of ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds']) {
    if (!Object.is(result[field], 0)) throw 'balanced fields must be positive zero';
  }
}
for (const convert of [input => from(input), input => until.call(receiver, input), input => since.call(receiver, input)]) {
  for (const primitive of [1, {}]) {
    let caught;
    try { convert({[Symbol.toPrimitive](){return primitive;}}); } catch (error) { caught = error; }
    if (!caught || Object.getPrototypeOf(caught) !== typeErrorPrototype) throw 'conversion error Realm';
  }
  const marker = {};
  let caught;
  try { convert({get [Symbol.toPrimitive](){throw marker;}}); } catch (error) { caught = error; }
  if (caught !== marker) throw 'foreign conversion abrupt identity';
}
for (const branded of [receiver, new Temporal.ZonedDateTime(0n, 'UTC')]) {
  Object.defineProperty(branded, Symbol.toPrimitive, {get(){throw 'branded primitive lookup';}});
  const result = from(branded);
  if (Object.getPrototypeOf(result) !== instantPrototype || result.epochNanoseconds !== 0n) throw 'branded copy';
  if (!until.call(receiver, branded).blank) throw 'branded other';
}
print('ok');
