const events = [];
const callbacks = [];
function mark(name) { events.push(name); return name; }
async function task() {
  await 0;
  for (using resource of [{ id: 7, [Symbol.dispose]() { events.push('dispose'); } }]) {
    class Local extends (mark('heritage'), Object) {
      [mark('method')]() { return resource.id; }
      [mark('field')] = async () => { await 0; return resource.id; };
      static { mark('static'); }
      async read() { await 0; return resource.id; }
    }
    const instance = new Local();
    callbacks.push(instance.field, () => instance.read(), async () => { await 0; return resource.id; });
    if (instance.method() !== 7) throw 'method capture';
  }
  const first = await callbacks[0]();
  const second = await callbacks[1]();
  const third = await callbacks[2]();
  if (first !== 7 || second !== 7 || third !== 7) throw 'nested async capture';
}
task().then(() => {
  if (events.join(',') !== 'heritage,method,field,static,dispose') throw events.join(',');
  print('ok');
});
