const events = [];
const bodyError = {};
const disposalError = {};
const closeError = {};
const resource = { [Symbol.dispose]() { events.push('dispose'); throw disposalError; } };
const iterable = { [Symbol.iterator]() { return {
  next() { return { value: resource, done: false }; },
  return() { events.push('close'); throw closeError; }
}; } };
async function task() {
  await 0;
  try {
    for (using value of iterable) {
      try { events.push('body'); throw bodyError; }
      finally { events.push('finally'); }
    }
  } catch (error) {
    if (!(error instanceof SuppressedError) || error.error !== disposalError || error.suppressed !== bodyError) throw 'suppression or close precedence';
    events.push('caught');
  }
  await 0;
  if (events.join(',') !== 'body,finally,dispose,close,caught') throw events.join(',');
}
task().then(() => print('ok'));
