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

var coercionToken = {};
var coercionTrace = "";
var throwingElement = {
  toString: function () {
    coercionTrace += "toString";
    throw coercionToken;
  },
  valueOf: function () {
    coercionTrace += "valueOf";
    return 1;
  },
};
var coercionIdentityPreserved = false;
try {
  [throwingElement].join();
} catch (error) {
  coercionIdentityPreserved = error === coercionToken;
}
if (!coercionIdentityPreserved || coercionTrace !== "toString") {
  throw "array element ToPrimitive abrupt routing";
}

true;
