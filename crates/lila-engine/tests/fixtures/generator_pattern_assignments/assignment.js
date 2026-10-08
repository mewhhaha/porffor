function check(value, label) { if (!value) throw new Error(label); }
function step(iterator, input, expected, done, label) {
  const result = iterator.next(input);
  check(result.value === expected && result.done === done, label);
}
let trace = '';
let selected;
let copied;
const sink = {
  set selected(value) {
    trace += 'S';
    gc();
    check(value.self === value, 'default whole value remains rooted during Put');
    selected = value;
  },
  set rest(value) { trace += 'R'; copied = value; }
};
function target() { trace += 'T'; return sink; }
function key() { trace += 'K'; return 'selected'; }
function fallback() {
  trace += 'D';
  const value = {};
  value.self = value;
  return value;
}
const source = {
  get selected() { trace += 'G'; gc(); return undefined; },
  other: 17
};
source.self = source;
function* objectAssignment() {
  return ({[key()]: target().selected = fallback(), ...target().rest} =
    (yield 'first', yield 'source'));
}
let iterator = objectAssignment();
step(iterator, undefined, 'first', false, 'complete RHS first suspension');
step(iterator, 0, 'source', false, 'complete RHS second suspension');
check(trace === '', 'no pattern target/key/Get before Normal RHS');
gc();
let result = iterator.next(source);
check(result.done && result.value === source && result.value.self === source, 'same whole RHS result');
check(trace === 'KTGDSTR', 'key then target then Get/default/Put then rest target/Put');
check(selected.self === selected && copied.other === 17 && copied.self === source,
  'whole default and object rest');
check(!Object.prototype.hasOwnProperty.call(copied, 'selected'), 'rest excludes selected key');

let nextCalls = 0;
let closeCalls = 0;
const iterable = {
  [Symbol.iterator]() {
    trace += 'I';
    return {
      next() { nextCalls++; trace += 'N'; return {value: undefined, done: false}; },
      return() { closeCalls++; trace += 'C'; return {}; }
    };
  }
};
function* arrayAssignment() { return ([target().selected = fallback()] = yield 'array'); }
trace = '';
iterator = arrayAssignment();
step(iterator, undefined, 'array', false, 'array RHS');
check(nextCalls === 0 && closeCalls === 0 && trace === '', 'no iterator before Normal RHS');
gc();
result = iterator.next(iterable);
check(result.done && result.value === iterable, 'array assignment returns original iterable');
check(trace === 'ITNDSC' && nextCalls === 1 && closeCalls === 1,
  'real assignment Reference precedes Step and normal IteratorClose occurs once');

const thrown = {};
thrown.self = thrown;
const closeFailure = {};
const throwingSink = { set selected(value) { gc(); throw thrown; } };
let abruptCloseCalls = 0;
const abruptIterable = {
  [Symbol.iterator]() {
    return {
      next() { return {value: 23, done: false}; },
      return() { abruptCloseCalls++; throw closeFailure; }
    };
  }
};
function* abruptPattern() { return ([throwingSink.selected] = yield 'abrupt'); }
iterator = abruptPattern();
step(iterator, undefined, 'abrupt', false, 'abrupt RHS');
let caught;
try { iterator.next(abruptIterable); } catch (error) { caught = error; }
check(caught === thrown && caught.self === caught && abruptCloseCalls === 1,
  'target Throw identity wins IteratorClose Throw');

function held() { return 29; }
let functionRest;
function* functionAssignment() { return ({...functionRest} = (yield 'function', held)); }
iterator = functionAssignment();
step(iterator, undefined, 'function', false, 'function RHS prefix');
gc();
result = iterator.next();
check(result.done && result.value === held && result.value() === 29, 'callable RHS identity survives pattern effects');

function* discardedAssignment() { [sink.selected] = yield 'discarded'; return selected; }
iterator = discardedAssignment();
step(iterator, undefined, 'discarded', false, 'discarded assignment RHS');
const discardedValue = {};
discardedValue.self = discardedValue;
gc();
step(iterator, [discardedValue], discardedValue, true, 'discarded direct Yield uses real assignment consumer');
const templateSink = {};
function* discardedTemplate() { [templateSink.value] = `${yield 'part'}`; return templateSink.value; }
iterator = discardedTemplate();
step(iterator, undefined, 'part', false, 'discarded template RHS');
gc();
step(iterator, 'abc', 'a', true, 'discarded template uses complete RHS staging');

let untouched = 0;
const untouchedSink = { set value(value) { untouched++; } };
function* injected() {
  try { return ([untouchedSink.value] = yield 'pending'); }
  finally { yield 'finally'; }
}
const returned = {};
returned.self = returned;
iterator = injected();
step(iterator, undefined, 'pending', false, 'injected Return RHS');
result = iterator.return(returned);
check(!result.done && result.value === 'finally', 'pending Return enters yielding finalizer');
gc();
step(iterator, undefined, returned, true, 'same whole pending Return');
iterator = injected();
step(iterator, undefined, 'pending', false, 'injected Throw RHS');
result = iterator.throw(thrown);
check(!result.done && result.value === 'finally', 'pending Throw enters yielding finalizer');
gc();
caught = undefined;
try { iterator.next(); } catch (error) { caught = error; }
check(caught === thrown && untouched === 0, 'abrupt RHS skips all pattern target writes');

function* capturedHead() {
  for (let [i] = yield (() => i); i < 2; i++) { yield (() => i); }
}
iterator = capturedHead();
result = iterator.next();
const initialCell = result.value;
let tdz = false;
try { initialCell(); } catch (error) { tdz = error instanceof ReferenceError; }
check(tdz, 'captured pattern head remains TDZ while initializer suspends');
gc();
result = iterator.next([0]);
const firstCell = result.value;
check(!result.done && firstCell() === 0, 'first pattern iteration');
gc();
result = iterator.next();
const secondCell = result.value;
check(!result.done && secondCell() === 1, 'second pattern iteration');
result = iterator.next();
check(result.done && initialCell() === 0 && firstCell() === 0 && secondCell() === 1,
  'initial head and each mutable iteration retain distinct cells');

function* eagerHead() {
  for (let {i} = {i: 0}; i < 2; i++) { yield (() => i); }
}
iterator = eagerHead();
const eagerFirst = iterator.next().value;
const eagerSecond = iterator.next().value;
check(iterator.next().done && eagerFirst() === 0 && eagerSecond() === 1,
  'eager pattern heads use the same actual per-iteration binding census');

function* constHead() {
  for (const {i} = yield (() => i); i; ) { yield (() => i); break; }
}
iterator = constHead();
const constInitial = iterator.next().value;
result = iterator.next({i: 31});
const constBody = result.value;
gc();
check(!result.done && constInitial() === 31 && constBody() === 31 && iterator.next().done,
  'const pattern head uses its initialized cell without mutable iteration cloning');

function* storedHead() {
  for (let {i} = yield 'head'; yield i; i++) { yield i; break; }
}
iterator = storedHead();
step(iterator, undefined, 'head', false, 'uncaptured pattern initializer');
gc();
step(iterator, {i: 37}, 37, false, 'uncaptured pattern head survives into suspended test');
gc();
step(iterator, true, 37, false, 'uncaptured head survives into body');
check(iterator.next().done, 'uncaptured pattern loop exits');
print('generator-pattern-assignments:ok');
