const events = [];
const thenable = { then(resolve) { events.push('adopt'); resolve(42); } };
async function task() {
  await 0;
  for (using value of [{ [Symbol.dispose]() { events.push('dispose'); } }]) {
    try { return thenable; }
    finally { events.push('finally'); }
  }
  throw 'return was lost';
}
task().then(value => {
  if (value !== 42 || events.join(',') !== 'finally,dispose,adopt') throw events.join(',');
  print('ok');
});
