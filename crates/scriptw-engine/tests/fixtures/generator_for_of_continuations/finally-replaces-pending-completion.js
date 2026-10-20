// The finalizer decides the completion that reaches IteratorClose. A Throw
// produced there wins over close failure; a Return produced there does not.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + " value");
  same(result.done, done, label + " done");
}
function source(state, closeError, shouldThrow) {
  var iterator = {
    next: function () {
      state.next++;
      return { value: 1, done: false };
    },
    return: function () {
      state.close++;
      if (shouldThrow) throw closeError;
      return {};
    },
  };
  return { [Symbol.iterator]: function () { return iterator; } };
}
function* finalizerThrows(iterable, replacement) {
  for (var value of iterable) {
    try {
      yield "body";
    } finally {
      yield "cleanup";
      throw replacement;
    }
  }
}
function* finalizerReturns(iterable) {
  for (var value of iterable) {
    try {
      yield "body";
    } finally {
      yield "cleanup";
      return 99;
    }
  }
}
function expectThrow(callback, expected, label) {
  var threw = false;
  try { callback(); }
  catch (error) {
    threw = true;
    same(error, expected, label + " identity");
  }
  same(threw, true, label + " throws");
}

var replacement = {};
var closeError = {};
var returningState = { next: 0, close: 0 };
var returning = finalizerThrows(source(returningState, closeError, true), replacement);
step(returning.next(), "body", false, "pending Return body");
step(returning.return(40), "cleanup", false, "pending Return cleanup");
same(returningState.close, 0, "Return cleanup remains suspended");
expectThrow(function () { returning.next(); }, replacement, "finalizer Throw replaces Return and wins over close failure");
same(returningState.next, 1, "replacement Throw skips stepping");
same(returningState.close, 1, "replacement Throw closes once");

var originalThrow = {};
var throwingState = { next: 0, close: 0 };
var throwing = finalizerThrows(source(throwingState, closeError, true), replacement);
step(throwing.next(), "body", false, "pending Throw body");
step(throwing.throw(originalThrow), "cleanup", false, "pending Throw cleanup");
same(throwingState.close, 0, "Throw cleanup remains suspended");
expectThrow(function () { throwing.next(); }, replacement, "finalizer Throw replaces original Throw");
same(throwingState.close, 1, "replacement Throw close count");

var replacedState = { next: 0, close: 0 };
var replaced = finalizerReturns(source(replacedState, closeError, false));
step(replaced.next(), "body", false, "finalizer Return body");
step(replaced.throw(originalThrow), "cleanup", false, "finalizer Return cleanup");
same(replacedState.close, 0, "replacement Return waits for cleanup");
step(replaced.next(), 99, true, "finalizer Return replaces original Throw");
same(replacedState.close, 1, "replacement Return closes once");

var failedState = { next: 0, close: 0 };
var failed = finalizerReturns(source(failedState, closeError, true));
step(failed.next(), "body", false, "failing replacement Return body");
step(failed.throw(originalThrow), "cleanup", false, "failing replacement Return cleanup");
expectThrow(function () { failed.next(); }, closeError, "close error replaces finalizer Return rather than retaining the old Throw");
same(failedState.close, 1, "failing replacement Return closes once");
print("ok");
262;
