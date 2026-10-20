// GeneratorResumeAbrupt supplies Return at the suspended yield. The try's
// finalizer can yield again before ForIn/OfBodyEvaluation performs IteratorClose.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + " value");
  same(result.done, done, label + " done");
}
var log = [];
var nextCalls = 0;
var returnReads = 0;
var closeCalls = 0;
var iterator = {
  next: function () {
    nextCalls++;
    log.push("next:" + nextCalls);
    return { value: 5, done: false };
  },
  get return() {
    returnReads++;
    log.push("get return");
    return function () {
      same(this, iterator, "IteratorClose receiver");
      same(arguments.length, 0, "IteratorClose receives no generator return argument");
      closeCalls++;
      log.push("close");
      return { value: "ignored", done: false };
    };
  },
};
var source = {
  [Symbol.iterator]: function () { return iterator; },
};
function* walk() {
  for (var value of source) {
    try {
      log.push("body:" + value);
      yield "body";
      log.push("unreached body continuation");
    } finally {
      log.push("finally start");
      yield "cleanup";
      log.push("finally complete");
    }
  }
  log.push("unreached tail");
  return 77;
}

var consumer = walk();
step(consumer.next(), "body", false, "body yield");
step(consumer.return(91), "cleanup", false, "return enters yielding finalizer");
same(log.join("|"), "next:1|body:5|finally start", "finalizer precedes close");
same(returnReads, 0, "return property is not read while cleanup is suspended");
same(closeCalls, 0, "suspension is not an abrupt loop exit");
step(consumer.next("ignored"), 91, true, "pending return restored after cleanup");
same(nextCalls, 1, "return skips the next iteration");
same(returnReads, 1, "close method acquired once");
same(closeCalls, 1, "source closed once");
same(log.join("|"), "next:1|body:5|finally start|finally complete|get return|close", "finalizer and close order");
step(consumer.next(), undefined, true, "completed generator remains completed");
same(closeCalls, 1, "completed next cannot close again");

// An explicit return also waits for the finalizer. The value supplied to the
// yielding return must survive cleanup and the outer iterator must close once.
function* returnYield() {
  for (var value of source) {
    try {
      log.push("return yield:" + value);
      return yield "explicit";
    } finally {
      log.push("explicit finally start");
      yield "explicit cleanup";
      log.push("explicit finally complete");
    }
  }
  throw "explicit return must skip the tail";
}
log = [];
nextCalls = 0;
returnReads = 0;
closeCalls = 0;
var explicitReturnValue = {};
var explicitConsumer = returnYield();
step(explicitConsumer.next(), "explicit", false, "explicit return yield");
step(explicitConsumer.next(explicitReturnValue), "explicit cleanup", false, "explicit return enters finalizer");
same(log.join("|"), "next:1|return yield:5|explicit finally start", "explicit finalizer precedes close");
same(returnReads, 0, "explicit return does not read close while cleanup is suspended");
same(closeCalls, 0, "explicit cleanup suspension keeps the source open");
step(explicitConsumer.next("ignored"), explicitReturnValue, true, "explicit return survives cleanup");
same(nextCalls, 1, "explicit return skips the next iteration");
same(returnReads, 1, "explicit return acquires close once");
same(closeCalls, 1, "explicit return closes once");
same(log.join("|"), "next:1|return yield:5|explicit finally start|explicit finally complete|get return|close", "explicit finalizer and close order");
step(explicitConsumer.next(), undefined, true, "explicit return remains completed");
same(closeCalls, 1, "completed explicit return cannot close again");

// Normal exhaustion of the delegate provides the return value; its enclosing
// finalizer still finishes before the outer for-of iterator closes.
var delegatedReturnValue = {};
function* returnDelegate() {
  yield "delegate";
  return delegatedReturnValue;
}
function* returnYieldStar() {
  for (var value of source) {
    try {
      log.push("return yield star:" + value);
      return yield* returnDelegate();
    } finally {
      log.push("delegated finally start");
      yield "delegated cleanup";
      log.push("delegated finally complete");
    }
  }
  throw "delegated return must skip the tail";
}
log = [];
nextCalls = 0;
returnReads = 0;
closeCalls = 0;
var delegatedConsumer = returnYieldStar();
step(delegatedConsumer.next(), "delegate", false, "delegated return yield");
step(delegatedConsumer.next("ignored"), "delegated cleanup", false, "delegated return enters finalizer");
same(log.join("|"), "next:1|return yield star:5|delegated finally start", "delegated finalizer precedes close");
same(returnReads, 0, "delegated return does not read close while cleanup is suspended");
same(closeCalls, 0, "delegated cleanup suspension keeps the source open");
step(delegatedConsumer.next("ignored"), delegatedReturnValue, true, "delegate return survives cleanup");
same(nextCalls, 1, "delegated return skips the next iteration");
same(returnReads, 1, "delegated return acquires close once");
same(closeCalls, 1, "delegated return closes once");
same(log.join("|"), "next:1|return yield star:5|delegated finally start|delegated finally complete|get return|close", "delegated finalizer and close order");
step(delegatedConsumer.next(), undefined, true, "delegated return remains completed");
same(closeCalls, 1, "completed delegated return cannot close again");
print("ok");
262;
