function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function checkKeys(object, expected, label) {
  var keys = Object.keys(object);
  check(keys.length, expected.length, label + " length");
  for (var i = 0; i < expected.length; i++) {
    check(keys[i], expected[i], label + " key " + i);
  }
}

function checkPair(actual, start, end, label) {
  check(actual.length, 2, label + " length");
  check(actual[0], start, label + " start");
  check(actual[1], end, label + " end");
}

var matcher = /(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))/;

var three = "abc".match(matcher);
check(three[0], "abc", "three match");
check(three.index, 0, "three index");
check(three.input, "abc", "three input");
check(three.groups.x, "b", "three group x");
check(three.groups.y, "a", "three group y");
check(three.groups.z, "c", "three group z");
checkKeys(three.groups, ["x", "y", "z"], "three group keys");

var two = "ad".match(matcher);
check(two[0], "ad", "two match");
check(two.groups.x, "a", "two group x");
check(two.groups.y, undefined, "two group y");
check(two.groups.z, "d", "two group z");
checkKeys(two.groups, ["x", "y", "z"], "two group keys");

var indexedMatcher = /(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))/d;
var indexed = "abc".match(indexedMatcher);
checkPair(indexed.indices[0], 0, 3, "full indices");
checkPair(indexed.indices.groups.x, 1, 2, "indices group x");
checkPair(indexed.indices.groups.y, 0, 1, "indices group y");
checkPair(indexed.indices.groups.z, 2, 3, "indices group z");
checkKeys(indexed.indices.groups, ["x", "y", "z"], "indices group keys");

var indexedTwo = "ad".match(indexedMatcher);
checkPair(indexedTwo.indices.groups.x, 0, 1, "indices two group x");
check(indexedTwo.indices.groups.y, undefined, "indices two group y");
checkPair(indexedTwo.indices.groups.z, 1, 2, "indices two group z");

var iterated = /(?:(?:(?<x>a)|(?<x>b)|c)\k<x>){2}/;
var iteratedMatch = "aac".match(iterated);
check(iteratedMatch.groups.x, undefined, "iterated group x");

var iteratedIndexed = /(?:(?:(?<x>a)|(?<x>b)|c)\k<x>){2}/d;
var iteratedIndexedMatch = "aac".match(iteratedIndexed);
check(iteratedIndexedMatch.indices.groups.x, undefined, "iterated indices group x");

true;
