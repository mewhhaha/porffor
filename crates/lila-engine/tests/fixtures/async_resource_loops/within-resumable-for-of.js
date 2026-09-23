const events = [];
async function task() {
  for (const id of [1, 2]) {
    await 0;
    for (using resource of [{ [Symbol.dispose]() { events.push('d' + id); } }]) {
      try { events.push('b' + id); }
      finally { events.push('f' + id); }
    }
    await 0;
  }
}
task().then(() => {
  if (events.join(',') !== 'b1,f1,d1,b2,f2,d2') throw events.join(',');
  print('ok');
});
