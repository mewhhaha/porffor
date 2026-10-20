const events = [];
let reader;
async function consume() {
  await 0;
  {
    const captured = 19;
    reader = () => captured;
    using resource = { [Symbol.dispose]() { events.push('dispose'); } };
    if (reader() !== 19) throw 'captured block changed inside using';
    events.push('body');
  }
  await 0;
  if (reader() !== 19) throw 'captured block changed after using';
  if (events.join(',') !== 'body,dispose') throw events.join(',');
}
consume().then(() => print('ok'), error => print('FAIL: ' + String(error)));
