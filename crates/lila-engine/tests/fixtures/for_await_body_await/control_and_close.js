function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
const returned = {};
const replacement = {};
const closeFailure = {};
function source(asynchronous, closeMode) {
  const state = { events: [], nexts: 0, returnGets: 0, closes: 0 };
  const iterator = {
    next() {
      state.nexts++;
      state.events.push('next:' + state.nexts);
      const result = state.nexts <= 3
        ? { value: state.nexts, done: false } : { done: true };
      return asynchronous ? Promise.resolve(result) : result;
    },
    get return() {
      state.returnGets++; state.events.push('get:return');
      return new Proxy(function () {}, {
        apply(target, receiver, args) {
          same(receiver, iterator, 'close raw receiver');
          same(args.length, 0, 'close argument count');
          state.closes++; state.events.push('close');
          if (closeMode === 'reject') return Promise.reject(closeFailure);
          if (closeMode === 'throw') throw closeFailure;
          if (asynchronous) {
            return Promise.resolve(0).then(function () {
              state.events.push('close:awaited');
              return {
                get done() { throw 'async close done must not be read'; },
                get value() { throw 'async close value must not be read'; }
              };
            });
          }
          return {
            get done() { state.events.push('return:done'); return true; },
            get value() {
              state.events.push('return:value');
              return Promise.resolve(0).then(function () {
                state.events.push('return:awaited'); return 0;
              });
            }
          };
        }
      });
    }
  };
  state.iterable = asynchronous
    ? { [Symbol.asyncIterator]() { return iterator; } }
    : { [Symbol.iterator]() { return iterator; } };
  return state;
}
async function breakAfterContinue(state) {
  for await (let value of state.iterable) {
    try {
      await 0;
      state.events.push('body:' + value);
      if (value === 1) continue;
      break;
    } finally {
      state.events.push('finally:' + value);
      await 0;
      state.events.push('middle:' + value);
      await 0;
      state.events.push('finalized:' + value);
    }
    throw 'local control fell through';
  }
  await 0;
  state.events.push('tail');
  return returned;
}
async function returnAfterFinalizers(state) {
  for await (const value of state.iterable) {
    try {
      await value;
      state.events.push('returning');
      return returned;
    } finally {
      state.events.push('finally');
      await 0;
      state.events.push('middle');
      await 0;
      state.events.push('finalized');
    }
  }
  throw 'returning loop reached tail';
}
async function replaceBreakWithContinue(state) {
  for await (const value of state.iterable) {
    try {
      await 0;
      state.events.push('breaking:' + value);
      break;
    } finally {
      await 0;
      state.events.push('continue:' + value);
      await 0;
      continue;
    }
  }
  await 0;
  state.events.push('tail');
  return returned;
}
async function replaceReturnWithRejection(state) {
  for await (const value of state.iterable) {
    try {
      await value;
      state.events.push('returning');
      return returned;
    } finally {
      state.events.push('finally');
      await 0;
      state.events.push('rejecting');
      await Promise.reject(replacement);
      throw 'rejected finalizer resumed normally';
    }
  }
  throw 'rejecting loop reached tail';
}
async function run() {
  const direct = source(true, 'normal');
  same(await breakAfterContinue(direct), returned, 'break selected completion');
  same(direct.events.join(','),
    'next:1,body:1,finally:1,middle:1,finalized:1,' +
    'next:2,body:2,finally:2,middle:2,finalized:2,get:return,close,close:awaited,tail',
    'continue skips close and break waits for both finalizers and close');
  same(direct.nexts, 2, 'break stops before next step');
  same(direct.returnGets, 1, 'break observes return once');
  same(direct.closes, 1, 'break closes once');

  const fallback = source(false, 'normal');
  same(await breakAfterContinue(fallback), returned, 'sync fallback break completion');
  same(fallback.events.join(','),
    'next:1,body:1,finally:1,middle:1,finalized:1,' +
    'next:2,body:2,finally:2,middle:2,finalized:2,' +
    'get:return,close,return:done,return:value,return:awaited,tail',
    'sync fallback close unwraps and awaits its value');
  same(fallback.returnGets, 1, 'fallback return Get once');
  same(fallback.closes, 1, 'fallback close once');

  const returning = source(true, 'normal');
  same(await returnAfterFinalizers(returning), returned, 'Return survives awaited close');
  same(returning.events.join(','),
    'next:1,returning,finally,middle,finalized,get:return,close,close:awaited',
    'Return finalizers precede close completion');
  same(returning.nexts, 1, 'Return cannot request another value');
  same(returning.closes, 1, 'Return closes once');

  const continuing = source(true, 'throw');
  same(await replaceBreakWithContinue(continuing), returned, 'finally Continue replaces Break');
  same(continuing.events.join(','),
    'next:1,breaking:1,continue:1,next:2,breaking:2,continue:2,' +
    'next:3,breaking:3,continue:3,next:4,tail', 'replaced Break never closes');
  same(continuing.returnGets, 0, 'Continue never even Gets return');
  same(continuing.closes, 0, 'natural completion after Continue never closes');

  const replaced = source(true, 'reject');
  let observed = returned;
  try { await replaceReturnWithRejection(replaced); }
  catch (error) { observed = error; }
  same(observed, replacement, 'finalizer rejection replaces Return and wins over close rejection');
  same(replaced.events.join(','),
    'next:1,returning,finally,rejecting,get:return,close', 'replacement close order');
  same(replaced.closes, 1, 'rejected finalizer closes once');

  const failedReturn = source(true, 'reject');
  observed = returned;
  try { await returnAfterFinalizers(failedReturn); }
  catch (error) { observed = error; }
  same(observed, closeFailure, 'close rejection replaces successful Return');
  same(failedReturn.events.join(','),
    'next:1,returning,finally,middle,finalized,get:return,close', 'failed Return close order');
  same(failedReturn.closes, 1, 'failed Return never closes twice');
}
run().then(function () { print('for-await-control-close:ok'); },
  function (error) { print('unexpected:' + String(error)); });
262;
