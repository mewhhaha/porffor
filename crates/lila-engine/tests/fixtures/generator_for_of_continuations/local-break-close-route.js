// A failure in IteratorClose after local Break belongs to the surrounding
// generator catch/finally, after the loop has stopped owning its continuation.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + " value");
  same(result.done, done, label + " done");
}
function source(mode, state, closeError) {
  var iterator = {
    next: function () { state.next++; return { value: 1, done: false }; },
    get return() {
      state.reads++;
      if (mode === "getter throw") throw closeError;
      if (mode === "missing") return undefined;
      if (mode === "not callable") return 1;
      return function () {
        same(this, iterator, "close receiver");
        same(arguments.length, 0, "close arguments");
        state.calls++;
        if (mode === "call throw") throw closeError;
        if (mode === "primitive result") return 1;
        return {};
      };
    },
  };
  return { [Symbol.iterator]: function () { return iterator; } };
}
function* route(iterable, state) {
  try {
    for (var value of iterable) {
      yield "body";
      break;
    }
    state.path += "tail,";
  } catch (error) {
    state.error = error;
    state.path += "catch,";
    yield "caught";
  } finally {
    state.path += "finally,";
    yield "outer-cleanup";
  }
  return "finished";
}
function check(mode, kind) {
  var closeError = {};
  var state = { next: 0, reads: 0, calls: 0, path: "", error: undefined };
  var consumer = route(source(mode, state, closeError), state);
  step(consumer.next(), "body", false, mode + " body");
  same(state.reads, 0, mode + " no early close");
  if (kind === "success") {
    step(consumer.next(), "outer-cleanup", false, mode + " outer cleanup");
    same(state.path, "tail,finally,", mode + " successful break tail");
  } else {
    step(consumer.next(), "caught", false, mode + " outer catch");
    same(state.path, "catch,", mode + " catch ordering");
    if (kind === "identity") same(state.error, closeError, mode + " original close error");
    else same(state.error instanceof TypeError, true, mode + " native protocol TypeError");
    step(consumer.next(), "outer-cleanup", false, mode + " caught outer cleanup");
    same(state.path, "catch,finally,", mode + " finally follows catch");
  }
  step(consumer.next(), "finished", true, mode + " surrounding completion");
  same(state.next, 1, mode + " local break does not step again");
  same(state.reads, 1, mode + " return getter runs once");
  var calls = mode === "getter throw" || mode === "missing" || mode === "not callable" ? 0 : 1;
  same(state.calls, calls, mode + " return call count");
  step(consumer.next(), undefined, true, mode + " completed next");
  same(state.reads, 1, mode + " completed next does not repeat close");
}
check("getter throw", "identity");
check("call throw", "identity");
check("not callable", "type");
check("primitive result", "type");
check("missing", "success");
check("success", "success");
print("ok");
262;
