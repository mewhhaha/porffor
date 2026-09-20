const events = [];
const readers = [];
let index = 0;
const values = [1, 2, 3].map(id => ({ id, [Symbol.dispose]() { events.push('d' + id); } }));
const iterable = { [Symbol.iterator]() { return {
  next() { return index < values.length ? { value: values[index++], done: false } : { done: true }; },
  return() { events.push('close'); return { done: true }; }
}; } };
async function task() {
  await 0;
  for (using value of iterable) {
    const local = value.id * 10;
    readers.push(() => value.id + local);
    try { events.push('b' + value.id); if (value.id === 1) continue; break; }
    finally { events.push('f' + value.id); }
  }
  await 0;
  if (readers[0]() !== 11 || readers[1]() !== 22 || readers.length !== 2) throw 'iteration environments';
}
task().then(() => {
  if (events.join(',') !== 'b1,f1,d1,b2,f2,d2,close') throw events.join(',');
  print('ok');
});
