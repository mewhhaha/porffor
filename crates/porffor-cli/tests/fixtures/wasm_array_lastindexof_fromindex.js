function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

var a = new Array(1, 2, 1);
check(a.lastIndexOf(2, undefined), -1, "explicit undefined fromIndex");
check(a.lastIndexOf(1, undefined), 0, "explicit undefined start at zero");
check(a.lastIndexOf(1), 2, "omitted fromIndex starts at end");

var sparse = [0, , 2, , 2];
check(sparse.lastIndexOf(2), 4, "sparse omitted fromIndex");
check(sparse.lastIndexOf(2, undefined), -1, "sparse explicit undefined fromIndex");
check(sparse.lastIndexOf(0, undefined), 0, "sparse explicit undefined hit zero");

true;
