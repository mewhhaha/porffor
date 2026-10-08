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
function source(state, failClose) {
  const iterator = {
    next() {
      ++state.next;
      state.log.push('next:' + state.next);
      return state.next <= 2 ? { value: { value: state.next }, done: false } : { done: true };
    },
    get return() {
      ++state.returnGets;
      state.log.push('get:return');
      return function () {
        same(this, iterator, 'close receiver');
        ++state.close;
        state.log.push('close');
        if (failClose) throw closeError;
        return {};
      };
    }
  };
  return { [Symbol.iterator]() { return iterator; } };
}
function* walk(iterable, state, pending) {
  for (let { value } of iterable) {
    const readers = [() => value, replacement => value = replacement];
    try {
      state.log.push('body:' + value);
      yield readers;
      if (pending === 'continue') continue;
      if (pending === 'break') break;
      return marker;
    } finally {
      state.log.push('finally:' + value);
      yield 'cleanup-1';
      state.log.push('after-1:' + value);
      yield 'cleanup-2';
      state.log.push('after-2:' + value);
    }
  }
  return 'tail';
}
function checkExit(pending) {
  const state = { next: 0, close: 0, returnGets: 0, log: [] };
  const consumer = walk(source(state, pending === 'injected-throw'), state, pending);
  const readers = consumer.next().value;
  same(readers[0](), 1, 'first initialized cell');
  let entering;
  if (pending === 'injected-return') entering = consumer.return(marker);
  else if (pending === 'injected-throw') entering = consumer.throw(marker);
  else entering = consumer.next();
  step(entering, 'cleanup-1', false, pending + ' enters yielding finalizer');
  same(state.log.join(','), 'next:1,body:1,finally:1', pending + ' finalizer before close');
  same(state.close, 0, 'first suspended cleanup leaves iterator open');
  readers[1](10);
  step(consumer.next(), 'cleanup-2', false, pending + ' second cleanup yield');
  same(readers[0](), 10, 'resume does not repeat BindingInitialization');
  same(state.next, 1, 'pending completion cannot step');
  readers[1](11);
  let result;
  let caught;
  let finalized = 0;
  try { result = consumer.next(); } catch (error) { caught = error; } finally { ++finalized; }
  if (pending === 'injected-throw') {
    same(caught, marker, 'original Throw wins over close Throw');
    same(result, undefined, 'Throw has no normal result');
  } else {
    same(caught, undefined, 'normal selected completion');
    step(result, pending === 'break' ? 'tail' : marker, true, 'selected completion survives cleanup');
  }
  same(finalized, 1, 'caller finally once');
  same(state.log.join(','), 'next:1,body:1,finally:1,after-1:10,after-2:11,get:return,close', 'two finalizer resumes precede close');
  same(readers[0](), 11, 'captured head survives completed iterator');
  same(state.returnGets, 1, 'return method acquired once');
  same(state.close, 1, 'exit closes once');
  step(consumer.next(), undefined, true, 'completed exit');
  same(state.close, 1, 'completed generator cannot close again');
}
checkExit('break');
checkExit('return');
checkExit('injected-return');
checkExit('injected-throw');

const continueState = { next: 0, close: 0, returnGets: 0, log: [] };
const continuing = walk(source(continueState, true), continueState, 'continue');
const first = continuing.next().value;
step(continuing.next(), 'cleanup-1', false, 'Continue first cleanup');
first[1](10);
step(continuing.next(), 'cleanup-2', false, 'Continue first resumed cleanup');
first[1](11);
const second = continuing.next().value;
same(second[0](), 2, 'Continue enters a fresh initialized cell');
same(first[0](), 11, 'earlier iteration cell remains retained');
same(continueState.log.join(','), 'next:1,body:1,finally:1,after-1:10,after-2:11,next:2,body:2', 'Continue initializes only after finalizer');
same(continueState.close, 0, 'Continue does not close');
step(continuing.next(), 'cleanup-1', false, 'Continue second cleanup');
second[1](20);
step(continuing.next(), 'cleanup-2', false, 'Continue second resumed cleanup');
second[1](21);
step(continuing.next(), 'tail', true, 'normal exhaustion after Continue');
same(continueState.close, 0, 'normal exhaustion never closes');
same(first[0](), 11, 'first cell remains independent after exhaustion');
same(second[0](), 21, 'second cell remains independent after exhaustion');
same(continueState.log.join(','), 'next:1,body:1,finally:1,after-1:10,after-2:11,next:2,body:2,finally:2,after-1:20,after-2:21,next:3', 'Continue and terminal done chronology');

function* shadow(iterable) {
  for (const { value } of iterable) {
    let value = 99;
    yield () => value;
  }
}
const shadowConsumer = shadow([{ value: 7 }]);
const shadowReader = shadowConsumer.next().value;
same(shadowReader(), 99, 'body lexical shadow owns separate cell');
step(shadowConsumer.next(), undefined, true, 'shadow consumer completes');
same(shadowReader(), 99, 'body shadow retained after head retired');
print('ok');
262;
