const events = [];
const marker = {};
function fail() { events.push('initializer'); throw marker; }
async function task() {
  await 0;
  try {
    for (using first = { [Symbol.dispose]() { events.push('dispose'); } }, second = fail(); false;) {}
  } catch (error) {
    if (error !== marker) throw 'initializer identity';
    events.push('caught');
  }
  await 0;
  if (events.join(',') !== 'initializer,dispose,caught') throw events.join(',');
}
task().then(() => print('ok'));
