function checkArrayToString(array, expected, label) {
  var actual = array.toString();
  var joined = array.join();
  if (actual !== joined) {
    throw label + ": toString !== join: " + actual + " !== " + joined;
  }
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

checkArrayToString(["", "", ""], ",,", "empty strings");
checkArrayToString(["\\", "\\", "\\"], "\\,\\,\\", "backslash strings");
checkArrayToString(["&", "&", "&"], "&,&,&", "ampersand strings");
checkArrayToString([true, true, true], "true,true,true", "booleans");
checkArrayToString([null, null, null], ",,", "nulls");
checkArrayToString([undefined, undefined, undefined], ",,", "undefineds");
checkArrayToString(
  [Infinity, Infinity, Infinity],
  "Infinity,Infinity,Infinity",
  "infinities",
);
checkArrayToString([NaN, NaN, NaN], "NaN,NaN,NaN", "NaNs");

true;
