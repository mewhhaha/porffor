function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function source(asynchronous, owner) {
  const state = { owner, asynchronous, events: [], nextGets: 0, nexts: 0,
    asyncGets: 0, syncGets: 0, closes: 0, cells: [] };
  const iterator = {
    get next() { state.nextGets++; state.events.push('get:next'); return next; },
    return() { state.closes++; throw 'natural completion must not close'; }
  };
  const next = new Proxy(function () {}, {
    apply(target, receiver, args) {
      same(receiver, iterator, 'cached next raw receiver');
      same(args.length, 0, 'next argument count');
      state.nexts++;
      const index = state.nexts;
      state.events.push('next:' + index);
      const result = {
        get done() { state.events.push('done:' + index); return index > 2; },
        get value() {
          if (index > 2 && asynchronous) throw 'true async done result value observed';
          state.events.push('value:' + index);
          if (index > 2) return Promise.resolve(0).then(function () {
            state.events.push('terminal:awaited'); return 0;
          });
          return asynchronous ? index : Promise.resolve(index);
        }
      };
      return asynchronous ? Promise.resolve(result) : result;
    }
  });
  state.iterator = iterator;
  state.iterable = {
    get [Symbol.asyncIterator]() {
      state.asyncGets++; state.events.push('get:async');
      return asynchronous ? function () {
        same(this, state.iterable, 'async iterator source receiver');
        state.events.push('acquire:async'); return iterator;
      } : null;
    },
    get [Symbol.iterator]() {
      state.syncGets++; state.events.push('get:sync');
      return function () {
        same(this, state.iterable, 'sync iterator source receiver');
        state.events.push('acquire:sync'); return iterator;
      };
    }
  };
  return state;
}
async function consume(state) {
  for await (let value of state.iterable) {
    state.cells.push(function () { return value; });
    state.events.push('body:' + value);
    await Promise.resolve(value).then(function (original) {
      if (original === 1) {
        Object.defineProperty(state.iterator, 'next', {
          value() { throw 'cached next was replaced'; }, configurable: true
        });
      }
      return original;
    });
    state.events.push('after:' + value);
    value += 10;
    await 0;
    state.events.push('second:' + value);
  }
  await 0;
  state.events.push('tail');
  return state.owner;
}
function check(state) {
  const acquisition = state.asynchronous
    ? 'get:async,acquire:async,get:next,'
    : 'get:async,get:sync,acquire:sync,get:next,';
  same(state.events.join(','), acquisition +
    'next:1,done:1,value:1,body:1,after:1,second:11,' +
    'next:2,done:2,value:2,body:2,after:2,second:12,' +
    (state.asynchronous ? 'next:3,done:3,tail' :
      'next:3,done:3,value:3,terminal:awaited,tail'),
    'next/body/continuation order');
  same(state.nextGets, 1, 'next is cached once');
  same(state.nexts, 3, 'body resumes never request another next');
  same(state.asyncGets, 1, 'async symbol observed once');
  same(state.syncGets, state.asynchronous ? 0 : 1, 'fallback symbol domain');
  same(state.closes, 0, 'natural done skips return');
  same(state.cells.length, 2, 'one captured head cell per iteration');
  same(state.cells[0](), 11, 'first head survives both body awaits');
  same(state.cells[1](), 12, 'second head has a distinct cell');
}
async function varHead() {
  let total = 0;
  for await (var value of [Promise.resolve(4), 5]) {
    await 0;
    total += value;
    await 0;
  }
  await 0;
  same(value, 5, 'var head value remains after loop');
  same(total, 9, 'var head retained through body resumes');
}
async function constHead() {
  let total = 0;
  for await (const value of [7, 8]) {
    await 0;
    total += value;
    await Promise.resolve(value);
  }
  same(total, 15, 'const head is retained across multiple awaits');
}
async function run() {
  const left = source(true, 'left');
  const right = source(false, 'right');
  const results = await Promise.all([consume(left), consume(right)]);
  same(results[0], 'left', 'first activation result');
  same(results[1], 'right', 'second activation result');
  check(left); check(right);
  await varHead();
  await constHead();
}
run().then(function () { print('for-await-next-body:ok'); },
  function (error) { print('unexpected:' + String(error)); });
262;
