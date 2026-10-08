function check(value, message) { if (!value) throw new Error(message); }
async function run() {
  const whole = {marker: 2}; whole.self = whole;
  for (const shape of ['adjacent', 'separated', 'method']) {
    const events = []; let reads = 0;
    const middle = {get [Symbol.dispose]() {
      ++reads; events.push('get-middle');
      return function () {
        check(this === middle, 'sync receiver'); events.push('middle');
        Promise.resolve().then(() => events.push('middle-job'));
        return {get then() { throw new Error('sync result must be ignored'); }};
      };
    }};
    const top = {[Symbol.asyncDispose]() { events.push('async'); return Promise.resolve().then(() => events.push('async-job')); }};
    async function adjacent() {
      { using resource = await middle; await using a = null, b = undefined; await 0; }
      events.push('after'); return whole;
    }
    async function separated() {
      { await using a = null; using resource = await middle; await using b = undefined; await 0; }
      events.push('after'); return whole;
    }
    async function method() {
      { await using a = null; using resource = await middle; await using b = await top; await 0; }
      events.push('after'); return whole;
    }
    const result = await (shape === 'adjacent' ? adjacent() : shape === 'separated' ? separated() : method());
    check(result === whole && reads === 1, 'one initializer and cached method');
    const expected = shape === 'adjacent' ? 'get-middle,middle,after,middle-job'
      : shape === 'separated' ? 'get-middle,middle,middle-job,after'
      : 'get-middle,async,async-job,middle,after,middle-job';
    check(events.join(',') === expected, 'one capability preserves needsAwait/hasAwaited: ' + shape);
  }
  const events = []; const failure = {marker: 3};
  const sync = {[Symbol.dispose]() { events.push('sync'); }};
  const asyncResource = {[Symbol.asyncDispose]() { events.push('async'); return Promise.resolve(); }};
  async function partial() {
    using first = await sync;
    await using second = await asyncResource, third = await Promise.reject(failure);
    throw new Error('unreachable');
  }
  let caught; try { await partial(); } catch (error) { caught = error; }
  check(caught === failure && events.join(',') === 'async,sync', 'rejected initializer closes earlier entries only');
  events.length = 0;
  let resolveDispose, count = 0, reader;
  const held = {[Symbol.asyncDispose]() { events.push('dispose'); return new Promise(resolve => { resolveDispose = resolve; }); }};
  async function loop() {
    for (await using resource = await held; await (count < 2); await ++count) {
      reader = () => resource; events.push('body' + count); await 0; continue;
    }
    events.push('after'); return whole;
  }
  const pending = loop();
  while (!resolveDispose) await 0;
  gc(); check(reader() === held, 'head lexical cell survives pending disposal');
  check(events.join(',') === 'body0,body1,dispose', 'Continue/update retains capability');
  resolveDispose(); check(await pending === whole, 'original async continuation');
  check(events.join(',') === 'body0,body1,dispose,after', 'disposal completes before following source');
  events.length = 0;
  async function caseValues() {
    outer: switch (await 0) {
      case await 0: {
        using first = await sync; events.push('first');
      }
      default: {
        await using second = await asyncResource; events.push('second'); await 0; break outer;
      }
    }
    events.push('after-case'); return whole;
  }
  check(await caseValues() === whole, 'whole switch return');
  check(events.join(',') === 'first,sync,second,async,after-case', 'fallthrough and labelled break close their original blocks');
  print('resumable-async-resource-scopes:ok');
}
run().catch(error => print('resource-error:' + error));
