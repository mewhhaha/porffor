function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function* walk() {
  for (const value of [7, 8]) {
    const reader = () => value;
    yield reader;
    yield reader;
  }
}

var consumer = walk();
var earlier = consumer.next().value;
same(earlier(), 7, "first const cell");
same(consumer.next().value, earlier, "same iteration keeps its reader");
var later = consumer.next().value;
same(later === earlier, false, "next iteration creates its own reader");
same(later(), 8, "second const cell");
same(earlier(), 7, "earlier const capture is retained");
same(consumer.next().value, later, "resumed second const cell");
var completed = consumer.next();
same(completed.done, true, "const loop exhaustion");
same(completed.value, undefined, "implicit generator completion");
same(earlier(), 7, "earlier capture after completion");
same(later(), 8, "later capture after completion");
print("ok");
262;
