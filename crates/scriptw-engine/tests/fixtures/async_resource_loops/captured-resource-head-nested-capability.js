const events = [];
const readers = [];
async function consume() {
  await 0;
  for (using head of [{ id: 7, [Symbol.dispose]() { events.push('head'); } }]) {
    const read = () => head.id;
    readers.push(read);
    using inner = { [Symbol.dispose]() { events.push('inner'); } };
    if (read() !== 7) throw 'captured head changed inside nested using';
    events.push('body');
  }
  await 0;
  if (readers.length !== 1 || readers[0]() !== 7) throw 'captured head changed after nested using';
  if (events.join(',') !== 'body,inner,head') throw events.join(',');
}
consume().then(() => print('ok'), error => print('FAIL: ' + String(error)));
