"use strict";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function values(set) {
  return [...set].join(",");
}

var small = new Set([1, 2]);
var large = new Set([2, 3, 4]);

assert(
  values(small.difference(large)) === "1",
  "difference receiver iteration",
);
assert(values(large.difference(small)) === "3,4", "difference other iteration");
assert(
  values(small.intersection(large)) === "2",
  "intersection receiver iteration",
);
assert(
  values(large.intersection(small)) === "2",
  "intersection other iteration",
);
assert(
  values(small.symmetricDifference(large)) === "1,3,4",
  "symmetric difference other iteration",
);
assert(values(small.union(large)) === "1,2,3,4", "union other iteration");

assert(!small.isDisjointFrom(large), "disjoint receiver overlap");
assert(!large.isDisjointFrom(small), "disjoint other overlap");
assert(new Set([5]).isDisjointFrom(large), "disjoint receiver full scan");
assert(large.isDisjointFrom(new Set([5, 6])), "disjoint other full scan");
assert(new Set([2]).isSubsetOf(large), "subset receiver success");
assert(!small.isSubsetOf(large), "subset receiver failure");
assert(large.isSupersetOf(new Set([2])), "superset other success");
assert(!large.isSupersetOf(new Set([1])), "superset other failure");

true;
