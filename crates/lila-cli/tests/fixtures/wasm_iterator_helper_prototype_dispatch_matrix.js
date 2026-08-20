function assertSameValue(actual, expected, label) {
  if (actual !== expected) throw label;
}

function identity(value) {
  return value;
}

function keep() {
  return true;
}

function singleton(value) {
  return [value];
}

var prototypeProbe = Iterator.from([0]).map(identity);
var helperPrototype = Object.getPrototypeOf(prototypeProbe);
var next = helperPrototype.next;
var returnMethod = helperPrototype.return;

function check(name, nextHelper, returnHelper, expected, zipped) {
  assertSameValue(Object.getPrototypeOf(nextHelper), helperPrototype, name + " next prototype");
  assertSameValue(Object.getPrototypeOf(returnHelper), helperPrototype, name + " return prototype");

  var nextResult = next.call(nextHelper);
  assertSameValue(nextResult.done, false, name + " next done");
  var nextValue = zipped ? nextResult.value[0] : nextResult.value;
  assertSameValue(nextValue, expected, name + " next value");

  var returnResult = returnMethod.call(returnHelper);
  assertSameValue(returnResult.done, true, name + " return done");
  assertSameValue(returnResult.value, undefined, name + " return value");

  var afterReturn = next.call(returnHelper);
  assertSameValue(afterReturn.done, true, name + " next after return done");
  assertSameValue(afterReturn.value, undefined, name + " next after return value");
}

check("concat", Iterator.concat([1]), Iterator.concat([1]), 1, false);
check("zip", Iterator.zip([[2]]), Iterator.zip([[2]]), 2, true);
check("map", Iterator.from([3]).map(identity), Iterator.from([3]).map(identity), 3, false);
check("filter", Iterator.from([4]).filter(keep), Iterator.from([4]).filter(keep), 4, false);
check(
  "flatMap",
  Iterator.from([5]).flatMap(singleton),
  Iterator.from([5]).flatMap(singleton),
  5,
  false
);
check("take", Iterator.from([6]).take(1), Iterator.from([6]).take(1), 6, false);
check("drop", Iterator.from([7]).drop(0), Iterator.from([7]).drop(0), 7, false);

true;
