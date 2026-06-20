function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

check("Boston, MA 02134".lastIndexOf("0"), 11, "postal code zero");
check("ababa".lastIndexOf("ba"), 3, "omitted position starts at end");
check("ababa".lastIndexOf("ba", 2), 1, "position clamps candidate start");
check("ababa".lastIndexOf("ba", 0), -1, "position before match");
check("abc".lastIndexOf(""), 3, "empty search omitted position");
check("abc".lastIndexOf("", 1), 1, "empty search explicit position");
check("abc".lastIndexOf("", undefined), 0, "empty search explicit undefined");
check("abc".lastIndexOf("a", undefined), 0, "explicit undefined starts at zero hit");
check("abc".lastIndexOf("b", undefined), -1, "explicit undefined starts at zero miss");
check("aaaa".lastIndexOf("aa", 2), 2, "overlapping reverse match at position");
check("aaaa".lastIndexOf("aa", 1), 1, "overlapping reverse match below end");
check("abc".lastIndexOf("d"), -1, "missing search");
check("abc".lastIndexOf("abc", 100), 0, "large position clamps to length");
check("\ud834\udf06a\ud834\udf06".lastIndexOf("\ud834\udf06"), 3, "utf16 code-unit index");

true;
