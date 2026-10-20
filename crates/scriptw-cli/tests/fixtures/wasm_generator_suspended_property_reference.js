function fail(label) {
  throw "suspended property Reference fixture failed: " + label;
}

function check(actual, expected, label) {
  if (actual !== expected) {
    fail(label + ": " + actual + " !== " + expected);
  }
}

var trace = "";
var target = {};
function base() {
  trace += "b";
  return target;
}
function key() {
  trace += "k";
  return "value";
}
function yielded() {
  trace += "y";
  return 1;
}
function* ordered() {
  base()[key()] = yield yielded();
}

var iterator = ordered();
var first = iterator.next();
check(first.value, 1, "plain yield value");
check(first.done, false, "plain yield suspension");
check(trace, "bky", "Reference precedes yielded expression");
var second = iterator.next(7);
check(second.done, true, "plain yield completion");
check(trace, "bky", "Reference operands are not re-evaluated");
check(target.value, 7, "plain resume writes through captured Reference");

var frozen = Object.freeze({ value: 1 });
var sloppyThrew = false;
function* sloppy() {
  frozen.value = yield 2;
}
iterator = sloppy();
iterator.next();
try {
  iterator.next(8);
} catch (error) {
  sloppyThrew = true;
}
check(sloppyThrew, false, "sloppy resume ignores failed Set");
check(frozen.value, 1, "sloppy failed Set does not mutate");

var strictThrew = false;
function* strict() {
  "use strict";
  frozen.value = yield 3;
}
iterator = strict();
iterator.next();
try {
  iterator.next(9);
} catch (error) {
  strictThrew = error instanceof TypeError;
}
check(strictThrew, true, "strict resume throws for failed Set");
check(frozen.value, 1, "strict failed Set does not mutate");

var abruptTarget = {};
function* abrupt() {
  abruptTarget.value = yield 4;
}
iterator = abrupt();
iterator.next();
try {
  iterator.throw("stop");
} catch (error) {}
check("value" in abruptTarget, false, "throw resume suppresses PutValue");

var delegatedTarget = {};
var delegatedTrace = "";
function delegatedBase() {
  delegatedTrace += "b";
  return delegatedTarget;
}
function delegatedKey() {
  delegatedTrace += "k";
  return "value";
}
function* source() {
  yield 5;
  return 6;
}
function* delegated() {
  delegatedBase()[delegatedKey()] = yield* source();
}
iterator = delegated();
first = iterator.next();
check(first.value, 5, "delegated yield value");
check(delegatedTrace, "bk", "delegated Reference precedes iteration");
second = iterator.next();
check(second.done, true, "delegated completion");
check(delegatedTrace, "bk", "delegated Reference operands are not re-evaluated");
check(delegatedTarget.value, 6, "yield star writes completion through captured Reference");

true;
