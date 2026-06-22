function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

var arrayValues = Array.from(Iterator.from([0, 1, 2, 3]));
check(arrayValues.length, 4, "array length");
check(arrayValues[0], 0, "array first");
check(arrayValues[3], 3, "array last");

var boxedStringValues = Array.from(Iterator.from(new String("str")));
check(boxedStringValues.length, 3, "boxed string length");
check(boxedStringValues[0], "s", "boxed string first");
check(boxedStringValues[2], "r", "boxed string last");

var iterator = Iterator.from([7, 8]);
var first = iterator.next();
check(first.value, 7, "direct next value");
check(first.done, false, "direct next done");

true;
