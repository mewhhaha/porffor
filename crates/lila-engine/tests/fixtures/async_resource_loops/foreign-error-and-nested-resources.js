const foreign = __lilaCreateRealm().global;
const marker = new foreign.TypeError('foreign');
const events = [];
async function task() {
  using outer = { [Symbol.dispose]() { events.push('outer'); } };
  await 0;
  for (using loop = { [Symbol.dispose]() { events.push('loop'); } }; true;) {
    using inner = { [Symbol.dispose]() { events.push('inner'); } };
    try { throw marker; }
    finally { events.push('finally'); }
  }
}
task().then(() => { throw 'unexpected fulfillment'; }, error => {
  if (error !== marker || !(error instanceof foreign.TypeError) || error instanceof TypeError) throw 'foreign identity';
  if (events.join(',') !== 'finally,inner,loop,outer') throw events.join(',');
  print('ok');
});
