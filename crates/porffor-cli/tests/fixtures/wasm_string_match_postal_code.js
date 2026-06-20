function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

var basic = "Boston, Mass. 02134".match(/([\d]{5})([-\ ]?[\d]{4})?$/);
check(basic.length, 3, "basic length");
check(basic[0], "02134", "basic match");
check(basic[1], "02134", "basic capture1");
check(basic[2], undefined, "basic capture2");
check(basic.index, 14, "basic index");
check(basic.input, "Boston, Mass. 02134", "basic input");

var hyphen = "Zip 02134-1234".match(/([\d]{5})([-\ ]?[\d]{4})?$/);
check(hyphen[0], "02134-1234", "hyphen match");
check(hyphen[1], "02134", "hyphen capture1");
check(hyphen[2], "-1234", "hyphen capture2");
check(hyphen.index, 4, "hyphen index");

var space = "Zip 02134 1234".match(/([\d]{5})([-\ ]?[\d]{4})?$/);
check(space[0], "02134 1234", "space match");
check(space[2], " 1234", "space capture2");

var noSep = "Zip 021341234".match(/([\d]{5})([-\ ]?[\d]{4})?$/);
check(noSep[0], "021341234", "nosep match");
check(noSep[1], "02134", "nosep capture1");
check(noSep[2], "1234", "nosep capture2");

var global = "Boston, Mass. 02134".match(/([\d]{5})([-\ ]?[\d]{4})?$/g);
check(global.length, 1, "global length");
check(global[0], "02134", "global match");

check("abc".match(/([\d]{5})([-\ ]?[\d]{4})?$/), null, "no match");

true;
