// ForIn/OfBodyEvaluation caches [[NextMethod]], reads done before value,
// and resumes the existing iteration without reacquiring or stepping.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + " value");
  same(result.done, done, label + " done");
}

var log = [];
var nextCalls = 0;
var closeCalls = 0;
var iterator = {
  get next() {
    log.push("get next");
    return function () {
      nextCalls++;
      var selected = nextCalls;
      log.push("next:" + selected);
      return {
        get done() {
          log.push("done:" + selected);
          return selected > 2;
        },
        get value() {
          log.push("value:" + selected);
          if (selected > 2) throw "exhausting value must not be read";
          return selected;
        },
      };
    };
  },
  return: function () {
    closeCalls++;
    return {};
  },
};
var source = {
  get [Symbol.iterator]() {
    log.push("get iterator");
    return function () {
      log.push("call iterator");
      return iterator;
    };
  },
};
function sourceOnce() {
  log.push("iterable");
  return source;
}
function* walk() {
  for (var value of sourceOnce()) {
    log.push("body:" + value);
    yield "first:" + value;
    log.push("after first:" + value);
    Object.defineProperty(iterator, "next", {
      value: function () { throw "cached next must survive replacement"; },
      configurable: true,
    });
    yield "second:" + value;
    log.push("after second:" + value);
  }
  log.push("tail");
  return 99;
}

var unstarted = walk();
var prestartReturn = {};
step(unstarted.return(prestartReturn), prestartReturn, true, "return before start");
step(unstarted.next(), undefined, true, "returned generator stays completed");
same(log.length, 0, "pre-start return does not acquire the iterator");
same(nextCalls, 0, "pre-start return does not step");
same(closeCalls, 0, "pre-start return does not close");

var consumer = walk();
same(log.length, 0, "calling a generator does not acquire the iterator");
step(consumer.next(), "first:1", false, "first suspension");
same(nextCalls, 1, "first step");
step(consumer.next("ignored"), "second:1", false, "second suspension");
same(nextCalls, 1, "resumption does not step");
step(consumer.next(), "first:2", false, "second iteration");
step(consumer.next(), "second:2", false, "second iteration continuation");
step(consumer.next(), 99, true, "loop exit");
same(nextCalls, 3, "two values plus exhaustion");
same(closeCalls, 0, "normal exhaustion does not close");
same(log.join("|"), [
  "iterable", "get iterator", "call iterator", "get next",
  "next:1", "done:1", "value:1", "body:1",
  "after first:1", "after second:1",
  "next:2", "done:2", "value:2", "body:2",
  "after first:2", "after second:2", "next:3", "done:3", "tail",
].join("|"), "protocol and segment order");
print("ok");
262;
