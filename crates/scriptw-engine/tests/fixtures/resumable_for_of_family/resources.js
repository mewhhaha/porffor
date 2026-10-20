function check(value) { if (!value) throw new Error('per-key disposal lifetime'); }
async function run() {
  const events = [], readers = [];
  let calls = 0;
  const input = {[Symbol.iterator]() { return {
    next() { events.push('next' + ++calls); return {done: false, value: {
      value: calls,
      [Symbol.asyncDispose]() { const value = this.value; events.push('dispose' + value); return Promise.resolve().then(() => {gc(); events.push('disposed' + value);}); }
    }}; },
    return() { events.push('close'); return {}; }
  }; }};
  for (await using resource of await input) {
    readers.push(() => resource.value); await 0;
    if (resource.value === 1) continue;
    break;
  }
  check(events.join(',') === 'next1,dispose1,disposed1,next2,dispose2,disposed2,close');
  check(readers[0]() === 1 && readers[1]() === 2);

  const syncEvents = [], whole = {marker: 44};
  const resource = {[Symbol.dispose]() {syncEvents.push('dispose');}};
  const input2 = {[Symbol.iterator]() {return {
    next() {return {done:false,value:resource};},
    return() {syncEvents.push('close');return {};}
  };}};
  function* values() {for (using held of input2) {yield held;}}
  const stream=values(); check(stream.next().value === resource); gc();
  const returned=stream.return(whole); check(returned.done&&returned.value===whole);
  check(syncEvents.join(',')==='dispose,close');

  const awaitedEvents = [];
  const awaited = {[Symbol.asyncIterator]() { return {
    next() {return Promise.resolve({done:false,value:{[Symbol.asyncDispose]() {awaitedEvents.push('dispose'); return Promise.resolve().then(()=>awaitedEvents.push('disposed'));}}});},
    return() {awaitedEvents.push('close');return Promise.resolve({});}
  };}};
  for await (await using held of awaited) { await 0; break; }
  check(awaitedEvents.join(',')==='dispose,disposed,close');
}
run().then(() => print('ok'), error => {throw error;});
void 0;
