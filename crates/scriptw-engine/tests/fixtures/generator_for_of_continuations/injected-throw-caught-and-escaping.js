function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + " value");
  same(result.done, done, label + " done");
}

var marker = {};
var log = [];
var nextCalls = 0;
var closeCalls = 0;
var iterator = {
  next: function () {
    nextCalls++;
    return { value: nextCalls, done: nextCalls > 2 };
  },
  return: function () {
    closeCalls++;
    log.push("close");
    return {};
  },
};
var source = { [Symbol.iterator]: function () { return iterator; } };
function* caught() {
  for (let value of source) {
    try {
      yield "body:" + value;
    } catch (error) {
      same(error, marker, "injected throw identity in catch");
      log.push("catch:" + value);
      yield error;
      log.push("after catch:" + value);
    } finally {
      log.push("finally:" + value);
    }
    yield "after:" + value;
  }
}

var consumer = caught();
step(consumer.next(), "body:1", false, "initial caught body");
step(consumer.throw(marker), marker, false, "catch can suspend with exact thrown object");
same(closeCalls, 0, "handled throw does not leave the loop");
same(nextCalls, 1, "catch resumption remains in the same iteration");
step(consumer.next(), "after:1", false, "caught body continues");
step(consumer.next(), "body:2", false, "next iteration after catch");
step(consumer.return(88), 88, true, "return from second iteration");
same(closeCalls, 1, "later return closes exactly once");
same(log.join("|"), "catch:1|after catch:1|finally:1|finally:2|close", "catch/finally/close order");

// An uncaught Throw(undefined) remains a Throw and wins even when retrieving
// the source iterator's return method throws another arbitrary value.
var closeError = {};
var escapingReturnReads = 0;
var escapingIterator = {
  next: function () { return { value: 3, done: false }; },
  get return() {
    escapingReturnReads++;
    throw closeError;
  },
};
var escapingSource = { [Symbol.iterator]: function () { return escapingIterator; } };
function* escaping() {
  for (var value of escapingSource) {
    yield value;
    throw "unreached escaping continuation";
  }
}
var escapingConsumer = escaping();
step(escapingConsumer.next(), 3, false, "escaping body");
var threw = false;
try {
  escapingConsumer.throw(undefined);
} catch (error) {
  threw = true;
  same(error, undefined, "original undefined throw wins over close error");
}
same(threw, true, "undefined is an abrupt throw rather than missing completion");
same(escapingReturnReads, 1, "escaping throw attempts close exactly once");
step(escapingConsumer.next(), undefined, true, "escaping generator completed");
print("ok");
262;
