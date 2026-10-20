// Local continue reaches iteration cleanup without closing. Local break waits
// through both finalizer yields before closing the cached IteratorRecord once.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + " value");
  same(result.done, done, label + " done");
}
function read(readers) { return readers[0]() + "," + readers[1](); }
function source(base, state) {
  var iterator = {
    get next() {
      state.gets++;
      return function () {
        same(this, iterator, "cached next receiver");
        state.next++;
        Object.defineProperty(iterator, "next", {
          value: function () { throw "replacement next must not run"; },
          configurable: true,
        });
        return { value: base + state.next - 1, done: false };
      };
    },
    get return() {
      state.reads++;
      return function () {
        same(this, iterator, "close receiver");
        same(arguments.length, 0, "close argument count");
        state.close++;
        return {};
      };
    },
  };
  return { [Symbol.iterator]: function () { return iterator; } };
}
function* walk(iterable, base) {
  for (let value of iterable) {
    let ordinal = value - base;
    let body = value * 10;
    let readers = [() => value, () => body];
    try {
      if (ordinal === 0) continue;
      yield readers;
      if (ordinal === 1) {
        value += 10;
        body += 1;
        continue;
      }
      break;
    } finally {
      yield readers;
      yield "cleanup";
    }
  }
  return "tail";
}

var firstState = { gets: 0, next: 0, reads: 0, close: 0 };
var secondState = { gets: 0, next: 0, reads: 0, close: 0 };
var first = walk(source(1, firstState), 1);
var second = walk(source(101, secondState), 101);
var firstInitial = first.next().value;
var secondInitial = second.next().value;
same(read(firstInitial), "1,10", "continue before body yield");
same(read(secondInitial), "101,1010", "second suspended owner");
same(firstState.next, 1, "first pending continue has not advanced");
same(firstState.reads, 0, "continue does not get return");
step(first.next(), "cleanup", false, "first continue second cleanup yield");
step(second.next(), "cleanup", false, "second continue second cleanup yield");
var firstLater = first.next().value;
var secondLater = second.next().value;
same(read(firstLater), "2,20", "fresh first iteration");
same(read(secondLater), "102,1020", "fresh second iteration");
same(firstLater === firstInitial, false, "fresh first readers");
same(secondLater === secondInitial, false, "fresh second readers");
step(first.next(), firstLater, false, "continue after body yield enters finalizer");
same(read(firstLater), "12,21", "mutated first iteration cells");
same(read(secondLater), "102,1020", "other owner remains suspended");
step(second.next(), secondLater, false, "second owner continue enters finalizer");
same(read(secondLater), "112,1021", "mutated second iteration cells");
step(first.next(), "cleanup", false, "first later continue cleanup");
step(second.next(), "cleanup", false, "second later continue cleanup");
var firstLast = first.next().value;
var secondLast = second.next().value;
same(read(firstLast), "3,30", "last first body");
same(read(secondLast), "103,1030", "last second body");
step(first.next(), firstLast, false, "pending break first cleanup");
step(second.next(), secondLast, false, "pending break second cleanup");
step(first.next(), "cleanup", false, "pending break first second cleanup");
step(second.next(), "cleanup", false, "pending break second second cleanup");
same(firstState.reads, 0, "break close waits for finalizer");
same(secondState.reads, 0, "second break close waits for finalizer");
step(first.next(), "tail", true, "resolved first break reaches tail");
step(second.next(), "tail", true, "resolved second break reaches tail");
same(firstState.next, 3, "first local branches step once per iteration");
same(secondState.next, 3, "second local branches step once per iteration");
same(firstState.gets, 1, "first cached next read");
same(secondState.gets, 1, "second cached next read");
same(firstState.close, 1, "first break closes once");
same(secondState.close, 1, "second break closes once");
same(read(firstInitial), "1,10", "first old capture after completion");
same(read(secondLater), "112,1021", "second old capture after completion");
step(first.next(), undefined, true, "completed first next");
step(second.next(), undefined, true, "completed second next");
same(firstState.reads, 1, "completed first does not repeat close");
same(secondState.reads, 1, "completed second does not repeat close");
print("ok");
262;
