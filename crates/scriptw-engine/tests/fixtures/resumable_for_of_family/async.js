function check(value) { if (!value) throw new Error('async iterator phases'); }
async function run() {
  const whole = {marker: 42}, closeError = {marker: 43}; whole.self = whole;
  for (const protocol of ['sync', 'async', 'fallback']) {
    const events = []; let reads = 0, calls = 0, closed = 0;
    const inner = {[Symbol.iterator]() { return {
      next() { return {done: false, value: undefined}; },
      return() { events.push('inner'); return {}; }
    }; }};
    const iterator = {
      get next() { reads++; return function () {
        check(this === iterator); calls++;
        return protocol === 'async' ? Promise.resolve({done: false, value: inner}) : {done: false, value: inner};
      }; },
      return() { closed++; events.push('outer');
        if (protocol === 'sync') throw closeError;
        const pending=Promise.resolve().then(() => {events.push('close-await'); throw closeError;});
        return protocol==='fallback' ? {done:true,value:pending} : pending;
      }
    };
    const input = {[Symbol.iterator]() { return iterator; }};
    if (protocol === 'async') input[Symbol.asyncIterator] = function () { return iterator; };
    function fail() { events.push('default'); Object.defineProperty(iterator, 'next', {value() {throw 'reacquired';}}); gc(); return Promise.reject(whole); }
    async function synchronous() { for (const [item = await fail()] of await input) { throw 'body'; } }
    async function awaited() { for await (const [item = await fail()] of await input) { throw 'body'; } }
    try { await (protocol === 'sync' ? synchronous() : awaited()); throw 'rejection lost'; }
    catch (error) { check(error === whole); }
    check(reads === 1 && calls === 1 && closed === 1);
    check(events.join(',') === (protocol === 'sync' ? 'default,inner,outer' : 'default,inner,outer,close-await'));
  }

  const readers = [], out = [];
  for await (const row of await [[undefined, 2], [undefined, 3]]) {
    const retained = row; readers.push(() => retained);
    try {
      for await (const [item = await Promise.resolve(9)] of [row]) {
        if (await true) out.push(item);
        await 0; gc();
      }
    } finally { await 0; }
  }
  check(out.join(',') === '9,9' && readers[0]()[1] === 2 && readers[1]()[1] === 3);

  const events = [], target = {};
  let resolve;
  const gate = new Promise(done => {resolve = done;});
  const key = {toString() {events.push('key'); return 'value';}};
  async function assign() {
    for (target[await gate] of [11]) { events.push('body'); await 0; }
  }
  const assigning = assign(); gc(); check(events.length === 0);
  resolve(key); await assigning;
  check(target.value === 11 && events.join(',') === 'key,body');
}
run().then(() => print('ok'), error => {throw error;});
void 0;
