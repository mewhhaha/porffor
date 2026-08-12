function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

function throwsSyntaxError(receiver, label) {
  var threw = false;
  try {
    RegExp.prototype[Symbol.matchAll].call(receiver, "input");
  } catch (error) {
    threw = error instanceof SyntaxError;
  }
  check(threw, true, label);
}

throwsSyntaxError({ flags: "z", source: "x" }, "invalid flag");
throwsSyntaxError({ flags: "gg", source: "x" }, "duplicate flag");
throwsSyntaxError(
  { [Symbol.match]: true, flags: "g", source: "(" },
  "invalid source"
);
throwsSyntaxError(
  { [Symbol.match]: true, flags: "g", source: "(?a)" },
  "invalid group"
);

var iterator = RegExp.prototype[Symbol.matchAll].call(
  { [Symbol.match]: true, flags: "g", source: "x" },
  "x"
);
check(typeof iterator.next, "function", "valid iterator");
check(iterator.next().value[0], "x", "valid match");

var sourceReads = 0;
var branded = /x/g;
branded[Symbol.match] = false;
Object.defineProperty(branded, "source", {
  get: function () {
    sourceReads = sourceReads + 1;
    return "y";
  }
});
var brandedIterator = branded[Symbol.matchAll]("x");
check(sourceReads, 0, "branded source reads");
check(brandedIterator.next().value[0], "x", "branded original source");
true;
