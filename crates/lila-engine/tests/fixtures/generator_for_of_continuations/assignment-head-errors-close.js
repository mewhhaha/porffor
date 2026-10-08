function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + ' value');
  same(result.done, done, label + ' done');
}
const SavedTypeErrorPrototype = TypeError.prototype;
const SavedReferenceErrorPrototype = ReferenceError.prototype;
const getPrototypeOf = Object.getPrototypeOf;
const hasOwn = Object.prototype.hasOwnProperty;
const foreign = $262.createRealm();
const closeMarker = new foreign.global.Object();
function source(state, failClose) {
  const iterator = {
    next() {
      ++state.next;
      state.log.push('next:' + state.next);
      return state.next === 1 ? { value: 9, done: false } : { done: true };
    },
    get return() {
      ++state.returnGets;
      state.log.push('get:return');
      return function () {
        same(this, iterator, 'head close receiver');
        same(arguments.length, 0, 'head close arguments');
        ++state.close;
        state.log.push('close');
        if (failClose) throw closeMarker;
        return {};
      };
    }
  };
  return { [Symbol.iterator]() { return iterator; } };
}
const immutableTarget = 7;
function* immutable(iterable, state) {
  try {
    for (immutableTarget of iterable) { ++state.body; yield 'unreached'; }
  } catch (error) {
    state.error = error;
    state.log.push('catch');
  } finally {
    state.log.push('finally');
    yield 'cleanup';
    state.log.push('finalized');
  }
  return 'done';
}
function* temporalDeadZone(iterable, state) {
  try {
    for (uninitializedTarget of iterable) { ++state.body; yield 'unreached'; }
  } catch (error) {
    state.error = error;
    state.log.push('catch');
  } finally {
    state.log.push('finally');
    yield 'cleanup';
    state.log.push('finalized');
  }
  let uninitializedTarget;
  return 'done';
}
function* unresolved(iterable, state) {
  'use strict';
  try {
    for (generatorMissingHeadTarget of iterable) { ++state.body; yield 'unreached'; }
  } catch (error) {
    state.error = error;
    state.log.push('catch');
  } finally {
    state.log.push('finally');
    yield 'cleanup';
    state.log.push('finalized');
  }
  return 'done';
}
function checkFailure(factory, expectedPrototype) {
  const state = { next: 0, returnGets: 0, close: 0, body: 0, log: [], error: undefined };
  const consumer = factory(source(state, true), state);
  step(consumer.next(), 'cleanup', false, 'head failure reaches outer finalizer');
  same(getPrototypeOf(state.error), expectedPrototype, 'original native head error');
  same(state.error === closeMarker, false, 'head Throw wins over close Throw');
  same(state.log.join(','), 'next:1,get:return,close,catch,finally', 'head failure close order');
  same(state.next, 1, 'head failure no next iteration');
  same(state.body, 0, 'head failure no body entry');
  same(state.returnGets, 1, 'head failure return Get once');
  same(state.close, 1, 'head failure close once before outer finally');
  step(consumer.next(), 'done', true, 'outer finalizer completes');
  same(state.log.join(','), 'next:1,get:return,close,catch,finally,finalized', 'finalizer continuation once');
  step(consumer.next(), undefined, true, 'failed head remains completed');
  same(state.next, 1, 'failed completed head never reassigns');
  same(state.close, 1, 'failed completed head never recloses');
}
checkFailure(immutable, SavedTypeErrorPrototype);
same(immutableTarget, 7, 'const target unchanged');
checkFailure(temporalDeadZone, SavedReferenceErrorPrototype);
checkFailure(unresolved, SavedReferenceErrorPrototype);
same(hasOwn.call(globalThis, 'generatorMissingHeadTarget'), false, 'strict head creates no global');

const strictMode = (function () { return this; })() === undefined;
const named = function* self(iterable, state) {
  try {
    for (self of iterable) {
      ++state.body;
      yield self;
      yield self;
    }
  } finally {
    ++state.finalized;
  }
  return self;
};
const namedState = { next: 0, returnGets: 0, close: 0, body: 0, finalized: 0, log: [] };
const namedConsumer = named(source(namedState, false), namedState);
if (strictMode) {
  let caught;
  try { namedConsumer.next(); } catch (error) { caught = error; }
  same(getPrototypeOf(caught), SavedTypeErrorPrototype, 'strict self head error');
  same(namedState.body, 0, 'strict self head precedes body');
} else {
  step(namedConsumer.next(), named, false, 'sloppy self assignment is ignored');
  step(namedConsumer.next(), named, false, 'sloppy self identity survives resume');
  same(namedState.next, 1, 'sloppy self head is not replayed');
  same(namedState.body, 1, 'sloppy self enters body once');
  step(namedConsumer.return('stop'), 'stop', true, 'sloppy self abrupt completion');
}
same(namedState.next, 1, 'self binding head steps once');
same(namedState.returnGets, 1, 'self binding close acquired once');
same(namedState.close, 1, 'self binding closes once');
same(namedState.finalized, 1, 'self binding outer finally once');
step(namedConsumer.next(), undefined, true, 'self binding consumer remains completed');
same(namedState.close, 1, 'self binding cannot close twice');
print('ok');
262;
