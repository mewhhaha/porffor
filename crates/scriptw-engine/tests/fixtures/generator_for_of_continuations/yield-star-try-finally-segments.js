// The delegated segment must finish before the iteration's finalizer starts.
// A finalizer yield suspends that same iteration rather than stepping its source.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + " value");
  same(result.done, done, label + " done");
}
var log = [];
function* walk() {
  for (var value of [1, 2]) {
    try {
      log.push("try:" + value);
      yield* [value, value + 10];
      log.push("after delegate:" + value);
    } finally {
      log.push("finally:" + value);
      yield "final:" + value;
      log.push("after final:" + value);
    }
  }
  log.push("tail");
  return "finished";
}

var consumer = walk();
step(consumer.next(), 1, false, "first delegated value");
same(log.join("|"), "try:1", "delegate still suspended");
step(consumer.next(), 11, false, "second delegated value");
same(log.join("|"), "try:1", "finalizer has not started");
step(consumer.next(), "final:1", false, "first finalizer yield");
same(log.join("|"), "try:1|after delegate:1|finally:1", "first finalizer order");
step(consumer.next(), 2, false, "next iteration delegated value");
step(consumer.next(), 12, false, "next iteration delegate continuation");
step(consumer.next(), "final:2", false, "next iteration finalizer");
step(consumer.next(), "finished", true, "loop completion");
same(log.join("|"), [
  "try:1", "after delegate:1", "finally:1", "after final:1",
  "try:2", "after delegate:2", "finally:2", "after final:2", "tail",
].join("|"), "every delegated and finalizer segment runs once");
print("ok");
262;
