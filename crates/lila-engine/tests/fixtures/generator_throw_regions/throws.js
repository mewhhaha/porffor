function assert(value, message) { if (!value) throw new Error(message); }
function step(iterator, value, expected, message) {
  var result = iterator.next(value);
  assert(!result.done && result.value === expected, message);
}
function thrownNext(iterator, value, expected, message) {
  var seen = false;
  try { iterator.next(value); } catch (error) { seen = error === expected; }
  assert(seen, message);
  assert(iterator.next().done, 'throw completes generator');
}
var whole = { kind: 'original' };
whole.self = whole;
var injected = { kind: 'injected' };
injected.self = injected;
var trace = [];
function* sequence() {
  try {
    throw (trace.push('first'), yield 'first', trace.push('second'), yield 'second', whole);
  } finally { trace.push('finally'); yield 'cleanup'; }
}
var sequential = sequence();
step(sequential, undefined, 'first', 'first complete operand segment');
gc();
step(sequential, 8, 'second', 'second complete operand segment');
step(sequential, 9, 'cleanup', 'Throw only after entire operand');
gc();
thrownNext(sequential, undefined, whole, 'whole cyclic value survives finalizer');
assert(trace.join(',') === 'first,second,finally', 'operand effects run once in source order');

trace = [];
var receiver = {
  get method() {
    trace.push('get');
    return function (a, b) {
      assert(this === receiver && a === 4 && b === 5, 'retained invocation Reference');
      trace.push('call');
      return whole;
    };
  }
};
function* callThrow() { throw receiver.method(yield 'arg1', yield 'arg2'); }
var calling = callThrow();
step(calling, undefined, 'arg1', 'callee GetValue before first argument');
gc();
step(calling, 4, 'arg2', 'callee and receiver survive next argument');
thrownNext(calling, 5, whole, 'completed Call value becomes exact Throw');
assert(trace.join(',') === 'get,call', 'no callee replay');

function* branches() { throw (yield 'condition') ? yield 'selected' : whole; }
var skipped = branches();
step(skipped, undefined, 'condition', 'conditional Throw selector');
thrownNext(skipped, false, whole, 'skipped branch has no suspension');
var selected = branches();
step(selected, undefined, 'condition', 'selected Throw selector');
step(selected, true, 'selected', 'selected branch suspension');
gc();
thrownNext(selected, whole, whole, 'selected full Value published before Throw');

function* inner() { yield 'delegated'; return whole; }
function* delegatedThrow() { throw yield* inner(); }
var delegating = delegatedThrow();
step(delegating, undefined, 'delegated', 'Throw operand uses real delegation');
gc();
thrownNext(delegating, undefined, whole, 'delegated final Value becomes Throw');

var forbidden = 0;
function* abrupt() {
  try { throw (yield 'gate', forbidden++, whole); }
  finally { yield 'abrupt-cleanup'; }
}
var returning = abrupt();
step(returning, undefined, 'gate', 'injected Return gate');
var pendingReturn = returning.return(injected);
assert(!pendingReturn.done && pendingReturn.value === 'abrupt-cleanup', 'Return bypasses Throw through finalizer');
gc();
var finalReturn = returning.next();
assert(finalReturn.done && finalReturn.value === injected && forbidden === 0, 'whole Return and skipped operand effects');
var throwing = abrupt();
step(throwing, undefined, 'gate', 'injected Throw gate');
var pendingThrow = throwing.throw(injected);
assert(!pendingThrow.done && pendingThrow.value === 'abrupt-cleanup', 'injected Throw enters finalizer');
gc();
thrownNext(throwing, undefined, injected, 'injected Throw identity remains authoritative');
assert(forbidden === 0, 'injected completion bypasses pending operand');

var getterCount = 0;
var getter = { get value() { getterCount++; throw injected; } };
function* getterThrow() { throw (yield 'get-gate', getter.value); }
var getterIterator = getterThrow();
step(getterIterator, undefined, 'get-gate', 'getter operand gate');
thrownNext(getterIterator, undefined, injected, 'GetValue Throw precedes source Throw');
assert(getterCount === 1, 'abrupt getter evaluated exactly once');

function* switchThrow() {
  switch (yield 'switch') { case 1: throw (yield 'case', whole); }
}
var switching = switchThrow();
step(switching, undefined, 'switch', 'Switch Throw discriminant');
step(switching, 1, 'case', 'Switch Throw body segment');
gc();
thrownNext(switching, undefined, whole, 'whole case Throw leaves the actual owner');
function* loopThrow() {
  for (var i = 0; i < 2; i++) { throw (yield i, whole); }
}
var looping = loopThrow();
step(looping, undefined, 0, 'classic loop Throw operand');
thrownNext(looping, undefined, whole, 'Throw exits classic loop without replay');

var nextCount = 0;
var closeCount = 0;
var items = {};
items[Symbol.iterator] = function () {
  return {
    next() { nextCount++; return { done: false, value: 77 }; },
    return() { closeCount++; return { done: true }; }
  };
};
function* iteratorThrow() {
  try { for (var item of items) { throw yield item; } }
  finally { yield 'iterator-cleanup'; }
}
var iterating = iteratorThrow();
step(iterating, undefined, 77, 'actual linear iterator body Throw operand');
gc();
step(iterating, whole, 'iterator-cleanup', 'Throw closes iterator before yielding finalizer');
assert(nextCount === 1 && closeCount === 1, 'source iterator acquisition and close exactly once');
gc();
thrownNext(iterating, undefined, whole, 'whole Throw survives close and finalizer');
assert(closeCount === 1, 'retirement does not close again');
print('generator-throw-regions:ok');
true;
