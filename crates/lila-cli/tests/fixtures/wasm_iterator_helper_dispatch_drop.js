function assertSameValue(actual, expected, label) {
  if (actual !== expected) throw label;
}

var nextIndex = 0;
var source = {
  __proto__: Iterator.prototype,
  next: function () {
    nextIndex = nextIndex + 1;
    if (nextIndex === 1) return { done: false, value: 1 };
    if (nextIndex === 2) return { done: false, value: 2 };
    return { done: true, value: undefined };
  },
  return: function () {
    return { done: true, value: undefined };
  },
};
var helper = source.drop(1);
var helperPrototype = Object.getPrototypeOf(helper);
var next = helperPrototype.next;
var returnMethod = helperPrototype.return;

var step = next.call(helper);
assertSameValue(step.done, false, "borrowed next done");
assertSameValue(step.value, 2, "borrowed next value");

step = returnMethod.call(helper);
assertSameValue(step.done, true, "borrowed return done");
assertSameValue(step.value, undefined, "borrowed return value");

step = next.call(helper);
assertSameValue(step.done, true, "next after return done");
assertSameValue(step.value, undefined, "next after return value");

true;
