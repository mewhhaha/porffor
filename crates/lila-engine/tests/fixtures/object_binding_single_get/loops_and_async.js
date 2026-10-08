function check(condition, name) { if (!condition) throw name; }
const define = Object.defineProperty;
const old = Object.getOwnPropertyDescriptor(String.prototype, 'watch');
let stringGets = 0, stringDefaults = 0;
try {
  define(String.prototype, 'watch', {configurable: true, get() {
    'use strict';
    check(typeof this === 'string' && (this === 'a' || this === 'bb'), 'for-in GetV retains primitive key receiver');
    stringGets++; return this.length === 1 ? 23 : undefined;
  }});
  const cells = [];
  for (const {watch = (stringDefaults++, 13)} in {a: 0, bb: 0}) cells.push(() => watch);
  check(stringGets === 2 && stringDefaults === 1 && cells.length === 2 &&
    cells[0]() === 23 && cells[1]() === 13, 'for-in primitive keys retain one Get and fresh lexical cells');
} finally {
  if (old === undefined) delete String.prototype.watch;
  else define(String.prototype, 'watch', old);
}

let varGets = 0, varDefaults = 0;
function varLoop() {
  const values = [{get item() { varGets++; return 4; }}, {get item() { varGets++; return undefined; }}];
  for (var {item = (varDefaults++, 8)} of values) {}
  return item;
}
check(varLoop() === 8 && varGets === 2 && varDefaults === 1, 'var head keeps one Get and hoisted final binding');

function makeState(owner) {
  const state = {owner, gets: 0, defaults: 0, nexts: 0, closes: 0, cells: [], trace: []};
  const values = [{get item() { state.gets++; state.trace.push('get1'); return owner; }},
    {get item() { state.gets++; state.trace.push('get2'); return undefined; }}];
  const iterator = {next() {
    state.nexts++; state.trace.push('next' + state.nexts);
    return state.nexts <= 2 ? {done: false, value: values[state.nexts - 1]} : {done: true};
  }, return() { state.closes++; return {}; }};
  state.iterator = iterator;
  state.iterable = {[Symbol.iterator]() { return iterator; }};
  return state;
}
async function consume(state) {
  for (let {item = (state.defaults++, state.trace.push('default'), state.owner + 10)} of state.iterable) {
    state.cells.push(() => item);
    state.trace.push('body');
    state.iterator.next = function() { throw 'iterator next was not cached'; };
    await 0;
    item += 1;
    await 0;
    state.trace.push('resume');
  }
  return state;
}
async function run() {
  const left = makeState(3), right = makeState(20);
  await Promise.all([consume(left), consume(right)]);
  for (const state of [left, right]) {
    check(state.gets === 2 && state.defaults === 1 && state.nexts === 3 && state.closes === 0,
      'async resume never repeats Get/default or closes a completed iterator');
    check(state.cells[0]() === state.owner + 1 && state.cells[1]() === state.owner + 11,
      'interleaved async activation and iteration cells stay distinct');
    check(state.trace.join(',') === 'next1,get1,body,resume,next2,get2,default,body,resume,next3',
      'head initialization precedes body suspension once');
  }
}
run().then(() => print('object-binding-loops:ok'), error => print('object-binding-loops:failed:' + String(error)));
262;
