const names = ['days','hours','microseconds','milliseconds','minutes','months','nanoseconds','seconds','weeks','years'];
const log = [];
const bag = {};
for (const name of names) {
  Object.defineProperty(bag, name, {get(){log.push(name);return name === 'days' ? 1 : 0;}});
}
for (const method of [Temporal.Instant.prototype.add, Temporal.Instant.prototype.subtract]) {
  log.length = 0;
  let caught = false;
  try { method.call(new Temporal.Instant(0n), bag); } catch (error) { if (!(error instanceof RangeError)) throw error; caught = true; }
  if (!caught || log.join(',') !== names.join(',')) throw 'full duration read before date rejection';
  log.length = 0;
  caught = false;
  try { method.call({}, bag); } catch (error) { if (!(error instanceof TypeError)) throw error; caught = true; }
  if (!caught || log.length !== 0) throw 'brand before duration';
}
const marker = {};
const dateBag = {days:1, get years(){throw marker;}};
let seen = false;
try { new Temporal.Instant(0n).add(dateBag); } catch (error) { if (error !== marker) throw 'duration completion identity'; seen = true; }
if (!seen) throw 'missing final getter';
for (const name of ['years','months','weeks','days']) {
  const value = {}; value[name] = -1;
  let caught = false;
  try { new Temporal.Instant(0n).add(value); } catch (error) { if (!(error instanceof RangeError)) throw error; caught = true; }
  if (!caught) throw 'date unit';
}
print('ok');
