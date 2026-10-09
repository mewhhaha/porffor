// The completion selected after a yielding finalizer owns whether this loop
// continues, closes for break/return, or preserves an original Throw.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + " value");
  same(result.done, done, label + " done");
}
function source(state, closeError, failClose) {
  var iterator = {
    next: function () {
      state.next++;
      return { value: 1, done: state.next > 1 };
    },
    return: function () {
      state.close++;
      if (failClose) throw closeError;
      return {};
    },
  };
  return { [Symbol.iterator]: function () { return iterator; } };
}
function* replace(iterable, pending, replacement, marker) {
  for (var value of iterable) {
    try {
      yield "body";
      if (pending === "continue") continue;
      break;
    } finally {
      yield "cleanup-1";
      yield "cleanup-2";
      if (replacement === "continue") continue;
      if (replacement === "break") break;
      if (replacement === "return") return marker;
      if (replacement === "throw") throw marker;
    }
  }
  return "tail";
}
function check(pending, replacement, failClose) {
  var marker = {};
  var closeError = {};
  var state = { next: 0, close: 0 };
  var consumer = replace(source(state, closeError, failClose), pending, replacement, marker);
  step(consumer.next(), "body", false, "replacement body");
  step(consumer.next(), "cleanup-1", false, "first cleanup");
  step(consumer.next(), "cleanup-2", false, "second cleanup");
  same(state.next, 1, "pending completion does not advance iterator");
  same(state.close, 0, "pending completion does not close iterator");
  var observed;
  var threw = false;
  try { observed = consumer.next(); }
  catch (error) { threw = true; observed = error; }
  if (replacement === "throw") {
    same(threw, true, "replacement Throw escapes");
    same(observed, marker, "replacement Throw wins over close error");
  } else if (failClose && replacement !== "continue") {
    same(threw, true, "close error replaces selected break or return");
    same(observed, closeError, "close error identity");
  } else {
    same(threw, false, "resolved replacement succeeds");
    step(observed, replacement === "return" ? marker : "tail", true, "replacement completion");
  }
  same(state.next, replacement === "continue" ? 2 : 1, "selected continuation stepping");
  same(state.close, replacement === "continue" ? 0 : 1, "selected close count");
  step(consumer.next(), undefined, true, "replacement remains completed");
  same(state.close, replacement === "continue" ? 0 : 1, "no repeated close");
}
check("break", "continue", true);
check("continue", "break", false);
check("continue", "break", true);
check("break", "return", false);
check("continue", "return", true);
check("break", "throw", true);
check("continue", "throw", true);

var returnMarker = {};
var returnState = { next: 0, close: 0 };
var returning = replace(source(returnState, {}, false), "continue", "continue", {});
step(returning.next(), "body", false, "injected Return body");
step(returning.next(), "cleanup-1", false, "pending Continue cleanup");
step(returning.return(returnMarker), returnMarker, true, "injected Return replaces pending Continue");
same(returnState.next, 1, "injected Return does not step");
same(returnState.close, 1, "injected Return closes once");

var foreign = __lilaCreateRealm();
var throwMarker = new foreign.global.Object();
var throwState = { next: 0, close: 0 };
var throwing = replace(source(throwState, {}, true), "break", "continue", {});
step(throwing.next(), "body", false, "injected Throw body");
step(throwing.next(), "cleanup-1", false, "pending Break cleanup");
var caught;
try { throwing.throw(throwMarker); }
catch (error) { caught = error; }
same(caught, throwMarker, "foreign injected Throw identity wins over close error");
same(throwState.next, 1, "injected Throw does not step");
same(throwState.close, 1, "injected Throw closes once");
step(throwing.next(), undefined, true, "injected Throw remains completed");
same(throwState.close, 1, "completed injected Throw does not repeat close");
print("ok");
262;
