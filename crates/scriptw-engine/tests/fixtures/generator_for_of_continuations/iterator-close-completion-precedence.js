// IteratorClose propagates an existing Throw ahead of GetMethod/Call/result
// errors. Those errors replace a pending Return; a successful close does not.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function* consume(source) {
  for (var value of source) {
    yield value;
  }
  return "unreached tail";
}
function makeSource(mode, state, closeError) {
  var iterator = {
    next: function () { return { value: 1, done: false }; },
    get return() {
      state.reads++;
      if (mode === "getter throw") throw closeError;
      if (mode === "missing") return undefined;
      if (mode === "not callable") return 5;
      return function () {
        same(this, iterator, "close receiver");
        same(arguments.length, 0, "close arguments");
        state.calls++;
        if (mode === "call throw") throw closeError;
        if (mode === "primitive result") return 5;
        return { value: "ignored", done: false };
      };
    },
  };
  return { [Symbol.iterator]: function () { return iterator; } };
}
function check(mode, resumeKind, expectedKind) {
  var original = {};
  var closeError = {};
  var state = { reads: 0, calls: 0 };
  var consumer = consume(makeSource(mode, state, closeError));
  same(consumer.next().value, 1, mode + " initial value");
  var observed;
  var threw = false;
  try {
    if (resumeKind === "return") observed = consumer.return(original);
    else observed = consumer.throw(original);
  } catch (error) {
    threw = true;
    observed = error;
  }
  if (expectedKind === "return") {
    same(threw, false, mode + " return succeeds");
    same(observed.done, true, mode + " completed return");
    same(observed.value, original, mode + " original return payload");
  } else {
    same(threw, true, mode + " must throw");
    if (expectedKind === "original") same(observed, original, mode + " original Throw wins");
    if (expectedKind === "close") same(observed, closeError, mode + " close error replaces Return");
    if (expectedKind === "type") same(observed instanceof TypeError, true, mode + " protocol TypeError");
  }
  same(state.reads, 1, mode + " return read once");
  var expectedCalls = mode === "getter throw" || mode === "missing" || mode === "not callable" ? 0 : 1;
  same(state.calls, expectedCalls, mode + " close call count");
  same(consumer.next().done, true, mode + " remains completed");
  same(state.reads, 1, mode + " completed next does not repeat close");
}

check("getter throw", "return", "close");
check("call throw", "return", "close");
check("not callable", "return", "type");
check("primitive result", "return", "type");
check("missing", "return", "return");
check("success", "return", "return");
check("getter throw", "throw", "original");
check("call throw", "throw", "original");
check("not callable", "throw", "original");
check("primitive result", "throw", "original");
check("missing", "throw", "original");
check("success", "throw", "original");
print("ok");
262;
