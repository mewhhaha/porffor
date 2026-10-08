function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + ' value');
  same(result.done, done, label + ' done');
}
const foreign = $262.createRealm();
const marker = new foreign.global.Object();
const closeError = new foreign.global.Object();
var target = 0;
function source(state, failClose) {
  const iterator = {
    next() {
      ++state.next;
      state.log.push('next:' + state.next);
      if (state.next > 3) throw 'extra iteration';
      return state.next <= 2 ? { value: state.next, done: false } : { done: true };
    },
    get return() {
      ++state.returnGets;
      state.log.push('get:return');
      return function () {
        same(this, iterator, 'local head close receiver');
        same(arguments.length, 0, 'local head close arguments');
        ++state.close;
        state.log.push('close:' + target);
        if (failClose) throw closeError;
        return {};
      };
    }
  };
  return { [Symbol.iterator]() { return iterator; } };
}
function* walk(iterable, state, pending) {
  for (target of iterable) {
    try {
      state.log.push('body:' + target);
      yield 'body';
      if (pending === 'continue') continue;
      if (pending === 'break') break;
      return marker;
    } finally {
      state.log.push('finally:' + target);
      yield 'cleanup-1';
      state.log.push('after-1:' + target);
      yield 'cleanup-2';
      state.log.push('after-2:' + target);
    }
  }
  return 'tail';
}
function checkExit(pending) {
  target = -1;
  const state = { next: 0, returnGets: 0, close: 0, log: [] };
  const consumer = walk(source(state, pending === 'injected-throw'), state, pending);
  step(consumer.next(), 'body', false, 'assignment before first body');
  same(target, 1, 'first iteration assigns existing target');
  let entering;
  if (pending === 'injected-return') entering = consumer.return(marker);
  else if (pending === 'injected-throw') entering = consumer.throw(marker);
  else entering = consumer.next();
  step(entering, 'cleanup-1', false, 'selected completion enters finalizer');
  same(state.log.join(','), 'next:1,body:1,finally:1', 'finalizer before close');
  same(state.returnGets, 0, 'suspended finalizer has no return Get');
  same(state.close, 0, 'suspended finalizer keeps iterator open');
  target = 10;
  step(consumer.next(), 'cleanup-2', false, 'second finalizer yield');
  same(target, 10, 'first finalizer resume does not replay assignment');
  same(state.log.join(','), 'next:1,body:1,finally:1,after-1:10', 'first resumed target');
  same(state.next, 1, 'pending completion does not step');
  target = 11;
  const prior = {};
  let completion = prior;
  let caught;
  let finalized = 0;
  try { completion = consumer.next(); }
  catch (error) { caught = error; }
  finally { ++finalized; }
  if (pending === 'injected-throw') {
    same(caught, marker, 'original foreign Throw survives close Throw');
    same(completion, prior, 'abrupt resume leaves prior assignment');
  } else {
    same(caught, undefined, 'selected local or Return completion succeeds');
    step(completion, pending === 'break' ? 'tail' : marker, true, 'selected final completion');
  }
  same(finalized, 1, 'caller finally executes once');
  same(target, 11, 'finalizer target mutation retained through close');
  same(state.log.join(','), 'next:1,body:1,finally:1,after-1:10,after-2:11,get:return,close:11',
    'finalizer completes before single close');
  same(state.next, 1, 'exiting completion skips next iteration assignment');
  same(state.returnGets, 1, 'exiting completion return Get once');
  same(state.close, 1, 'exiting completion close once');
  step(consumer.next(), undefined, true, 'exiting consumer remains completed');
  same(state.next, 1, 'completed exit cannot reassign');
  same(state.close, 1, 'completed exit cannot close again');
}
checkExit('break');
checkExit('return');
checkExit('injected-return');
checkExit('injected-throw');

target = -1;
const continueState = { next: 0, returnGets: 0, close: 0, log: [] };
const continuing = walk(source(continueState, true), continueState, 'continue');
step(continuing.next(), 'body', false, 'Continue first body');
same(target, 1, 'Continue first head assignment');
step(continuing.next(), 'cleanup-1', false, 'Continue first finalizer');
target = 10;
step(continuing.next(), 'cleanup-2', false, 'Continue first resumed finalizer');
same(target, 10, 'Continue resume does not assign');
target = 11;
step(continuing.next(), 'body', false, 'Continue next iteration after finalizer');
same(target, 2, 'Continue performs next head assignment after finalizer');
same(continueState.log.join(','), 'next:1,body:1,finally:1,after-1:10,after-2:11,next:2,body:2',
  'Continue finalizer precedes next assignment');
same(continueState.returnGets, 0, 'local Continue does not acquire close');
same(continueState.close, 0, 'local Continue does not close');
step(continuing.next(), 'cleanup-1', false, 'Continue second finalizer');
target = 20;
step(continuing.next(), 'cleanup-2', false, 'Continue second resumed finalizer');
target = 21;
step(continuing.next(), 'tail', true, 'Continue exhausts normally');
same(target, 21, 'terminal done does not assign a value');
same(continueState.next, 3, 'Continue exact next steps');
same(continueState.returnGets, 0, 'normal exhaustion does not acquire close');
same(continueState.close, 0, 'normal exhaustion does not close');
same(continueState.log.join(','), 'next:1,body:1,finally:1,after-1:10,after-2:11,next:2,body:2,' +
  'finally:2,after-1:20,after-2:21,next:3', 'Continue complete target lifecycle');
step(continuing.next(), undefined, true, 'Continue remains completed');
same(continueState.next, 3, 'completed Continue does not step');
print('ok');
262;
