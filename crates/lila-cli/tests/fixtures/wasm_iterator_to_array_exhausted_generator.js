function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

var iterator = (function* () {})();
var next = iterator.next();
check(next.value, undefined, "next value");
check(next.done, true, "next done");

var result = iterator.toArray();
check(result.length, 0, "first toArray");

result = iterator.toArray();
check(result.length, 0, "second toArray");

true;
