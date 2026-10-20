function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function readJoinedSize(useMap) {
  var map = new Map();
  map.set("entry", 1);
  var set = new Set();
  set.add("left");
  set.add("right");
  return (useMap ? map : set).size;
}

function setJoinedPrototype(useMap) {
  var map = new Map();
  var set = new Set();
  (useMap ? map : set).__proto__ = null;
  return Object.getPrototypeOf(useMap ? map : set);
}

function clearJoinedPrototype(useMap) {
  var map = new Map();
  var set = new Set();
  var result = ((useMap ? map : set).__proto__ &&= null);
  check(result, null, "logical assignment result");
  return Object.getPrototypeOf(useMap ? map : set);
}

function writeNestedConditional(first, second) {
  var left = { p: 1, leftOnly: 0 };
  var middle = { p: 1, middleOnly: 0 };
  var right = { p: 1, rightOnly: 0 };
  (first ? left : second ? middle : right).p = "s";
  return left.p + 1 + ":" + (middle.p + 1) + ":" + (right.p + 1);
}

function writeConditionalLogical(flag) {
  var left = { p: 0, leftOnly: 0 };
  var right = { p: 0, rightOnly: 0 };
  (flag ? left : right).p ||= "s";
  return left.p + 1 + ":" + (right.p + 1);
}

function writeConditionalPrototype(flag) {
  var leftPrototype = { p: 1, leftOnly: 0 };
  var rightPrototype = { p: 1, rightOnly: 0 };
  var left = Object.create(leftPrototype);
  var right = Object.create(rightPrototype);
  (flag ? leftPrototype : rightPrototype).p = "s";
  return left.p + 1 + ":" + (right.p + 1);
}

check(readJoinedSize(true), 1, "getter selected Map size");
check(readJoinedSize(false), 2, "getter selected Set size");
check(setJoinedPrototype(true), null, "setter selected Map prototype");
check(setJoinedPrototype(false), null, "setter selected Set prototype");
check(clearJoinedPrototype(true), null, "combined selection changed Map prototype");
check(clearJoinedPrototype(false), null, "combined selection changed Set prototype");
check(writeNestedConditional(true, false), "s1:2:2", "nested left receiver write");
check(writeNestedConditional(false, true), "2:s1:2", "nested middle receiver write");
check(writeNestedConditional(false, false), "2:2:s1", "nested right receiver write");
check(writeConditionalLogical(true), "s1:1", "logical left receiver write");
check(writeConditionalLogical(false), "1:s1", "logical right receiver write");
check(writeConditionalPrototype(true), "s1:2", "left prototype write");
check(writeConditionalPrototype(false), "2:s1", "right prototype write");

262;
