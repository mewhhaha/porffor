// Iterator acquisition, next, done, and value errors propagate before the
// body-owned IteratorClose region. They must not invoke the return method.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function* consume(source) {
  for (var value of source) {
    yield value;
    throw "unreached body";
  }
}
function check(mode, expectedLog) {
  var marker = {};
  var log = [];
  var closes = 0;
  var iterator = {
    get next() {
      log.push("get next");
      if (mode === "get next") throw marker;
      return function () {
        log.push("next");
        if (mode === "next") throw marker;
        return {
          get done() {
            log.push("done");
            if (mode === "done") throw marker;
            return false;
          },
          get value() {
            log.push("value");
            throw marker;
          },
        };
      };
    },
    get return() {
      closes++;
      return function () { throw "return must not be called"; };
    },
  };
  var source = {
    get [Symbol.iterator]() {
      log.push("get iterator");
      if (mode === "get iterator") throw marker;
      return function () {
        log.push("call iterator");
        if (mode === "call iterator") throw marker;
        return iterator;
      };
    },
  };
  var consumer = consume(source);
  var threw = false;
  try { consumer.next(); }
  catch (error) {
    threw = true;
    same(error, marker, mode + " original error identity");
  }
  same(threw, true, mode + " throws");
  same(closes, 0, mode + " does not retrieve return");
  same(log.join("|"), expectedLog, mode + " operations stop at the error");
  same(consumer.next().done, true, mode + " abrupt generator is completed");
  same(closes, 0, mode + " completed next does not close");
}

check("get iterator", "get iterator");
check("call iterator", "get iterator|call iterator");
check("get next", "get iterator|call iterator|get next");
check("next", "get iterator|call iterator|get next|next");
check("done", "get iterator|call iterator|get next|next|done");
check("value", "get iterator|call iterator|get next|next|done|value");
print("ok");
262;
