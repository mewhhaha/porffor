const names = ['add', 'subtract', 'round', 'until', 'since'];
for (const name of names) {
  const d = Object.getOwnPropertyDescriptor(Temporal.Instant.prototype, name);
  if (!d || typeof d.value !== 'function' || !d.writable || d.enumerable || !d.configurable) throw 'method descriptor';
  const method = d.value;
  if (method.name !== name || method.length !== 1 || method.hasOwnProperty('prototype')) throw 'method metadata';
  for (const key of ['name', 'length']) {
    const md = Object.getOwnPropertyDescriptor(method, key);
    if (md.writable || md.enumerable || !md.configurable) throw 'function descriptor';
  }
  let caught = false;
  try { Reflect.construct(method, []); } catch (error) { if (!(error instanceof TypeError)) throw error; caught = true; }
  if (!caught) throw 'method is constructable';
}
class Sub extends Temporal.Instant {}
const receiver = new Sub(5n);
const outputs = [receiver.add({nanoseconds:0}), receiver.subtract({seconds:0}), receiver.round('nanosecond')];
for (const result of outputs) {
  if (result === receiver || Object.getPrototypeOf(result) !== Temporal.Instant.prototype || result.epochNanoseconds !== 5n) throw 'intrinsic result';
}
if (Object.getPrototypeOf(receiver.until(receiver)) !== Temporal.Duration.prototype) throw 'Duration result';
if (Object.getPrototypeOf(receiver.since(receiver)) !== Temporal.Duration.prototype) throw 'Duration result';
print('ok');
