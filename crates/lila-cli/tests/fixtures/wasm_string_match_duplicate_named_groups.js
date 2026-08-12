function check(actual, expected) {
  if (actual !== expected) {
    throw "check failed";
  }
}

function checkKeys(object, expected) {
  var keys = Object.keys(object);
  check(keys.length, expected.length);
  for (var i = 0; i < expected.length; i++) {
    check(keys[i], expected[i]);
  }
}

function checkPair(actual, start, end) {
  check(actual.length, 2);
  check(actual[0], start);
  check(actual[1], end);
}

var matcher = /(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))/;

var three = "abc".match(matcher);
check(three[0], "abc");
check(three.index, 0);
check(three.input, "abc");
check(three.groups.x, "b");
check(three.groups.y, "a");
check(three.groups.z, "c");
checkKeys(three.groups, ["x", "y", "z"]);

var two = "ad".match(matcher);
check(two[0], "ad");
check(two.groups.x, "a");
check(two.groups.y, undefined);
check(two.groups.z, "d");
checkKeys(two.groups, ["x", "y", "z"]);

var indexedMatcher = /(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))/d;
var indexed = "abc".match(indexedMatcher);
checkPair(indexed.indices[0], 0, 3);
checkPair(indexed.indices.groups.x, 1, 2);
checkPair(indexed.indices.groups.y, 0, 1);
checkPair(indexed.indices.groups.z, 2, 3);
checkKeys(indexed.indices.groups, ["x", "y", "z"]);

var indexedTwo = "ad".match(indexedMatcher);
checkPair(indexedTwo.indices.groups.x, 0, 1);
check(indexedTwo.indices.groups.y, undefined);
checkPair(indexedTwo.indices.groups.z, 1, 2);

var iterated = /(?:(?:(?<x>a)|(?<x>b)|c)\k<x>){2}/;
var iteratedMatch = "aac".match(iterated);
check(iteratedMatch.groups.x, undefined);

var iteratedIndexed = /(?:(?:(?<x>a)|(?<x>b)|c)\k<x>){2}/d;
var iteratedIndexedMatch = "aac".match(iteratedIndexed);
check(iteratedIndexedMatch.indices.groups.x, undefined);

true;
