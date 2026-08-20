function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function checkIndices(indices, full, x, y, z, label) {
  check(indices[0][0], 0, label + " full start");
  check(indices[0][1], full, label + " full end");
  if (x === undefined) check(indices.groups.x, undefined, label + " x");
  else {
    check(indices.groups.x[0], x[0], label + " x start");
    check(indices.groups.x[1], x[1], label + " x end");
  }
  if (y === undefined) check(indices.groups.y, undefined, label + " y");
  else {
    check(indices.groups.y[0], y[0], label + " y start");
    check(indices.groups.y[1], y[1], label + " y end");
  }
  if (z === undefined) check(indices.groups.z, undefined, label + " z");
  else {
    check(indices.groups.z[0], z[0], label + " z start");
    check(indices.groups.z[1], z[1], label + " z end");
  }
  check(JSON.stringify(Object.keys(indices.groups)), JSON.stringify(["x", "y", "z"]), label + " keys");
  check(JSON.stringify(Object.getOwnPropertyNames(indices.groups)), JSON.stringify(["x", "y", "z"]), label + " own names");
  if (typeof Object.getPrototypeOf === "function") {
    check(Object.getPrototypeOf(indices.groups), null, label + " null prototype");
  }
}

var duplicate = /(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))/;
var abc = duplicate.exec("abc");
check(abc[0], "abc", "abc full");
check(abc.groups.x, "b", "abc x");
check(abc.groups.y, "a", "abc y");
check(abc.groups.z, "c", "abc z");
check(JSON.stringify(Object.keys(abc.groups)), JSON.stringify(["x", "y", "z"]), "abc keys");
check(JSON.stringify(Object.getOwnPropertyNames(abc.groups)), JSON.stringify(["x", "y", "z"]), "abc own names");

var ad = duplicate.exec("ad");
check(ad[0], "ad", "ad full");
check(ad.groups.x, "a", "ad x");
check(ad.groups.y, undefined, "ad y");
check(ad.groups.z, "d", "ad z");
check(JSON.stringify(Object.keys(ad.groups)), JSON.stringify(["x", "y", "z"]), "ad keys");
check(JSON.stringify(Object.getOwnPropertyNames(ad.groups)), JSON.stringify(["x", "y", "z"]), "ad own names");

var dynamicOwnNames = {};
dynamicOwnNames.answer = 42;
check(Object.getOwnPropertyNames(dynamicOwnNames)[0], "answer", "dynamic own name");
check(dynamicOwnNames[Object.getOwnPropertyNames(dynamicOwnNames)[0]], 42, "dynamic own-name value");

var repeated = /(?:(?:(?<x>a)|(?<x>b)|c)\k<x>){2}/.exec("aac");
check(repeated[0], "aac", "repeated full");
check(repeated.groups.x, undefined, "repeated final x");

var duplicateIndices = /(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))/d;
var abcIndices = duplicateIndices.exec("abc");
checkIndices(abcIndices.indices, 3, [1, 2], [0, 1], [2, 3], "abc indices");
check(abcIndices.indices.groups.x, abcIndices.indices[3], "abc selected x pair identity");
check(abcIndices.indices.groups.y, abcIndices.indices[2], "abc selected y pair identity");
check(abcIndices.indices.groups.z, abcIndices.indices[4], "abc selected z pair identity");
var adIndices = duplicateIndices.exec("ad");
checkIndices(adIndices.indices, 2, [0, 1], undefined, [1, 2], "ad indices");
check(adIndices.indices.groups.x, adIndices.indices[1], "ad selected x pair identity");
check(adIndices.indices.groups.z, adIndices.indices[5], "ad selected z pair identity");

var direct = /(?<x>a)|(?<x>b)\k<x>/;
var directMatch = direct.exec("bb");
check(directMatch[0], "bb", "direct backref full");
check(directMatch.groups.x, "b", "direct backref x");

var fallback = /(?:(?<x>a)\k<x>|ab)/.exec("ab");
check(fallback[0], "ab", "backref mismatch fallback full");
check(fallback.groups.x, undefined, "backref mismatch fallback capture");

var codeUnitBackref = /(?<x>..)\k<x>/.exec("𝌆𝌆");
check(codeUnitBackref[0], "𝌆𝌆", "code-unit backref full");
check(codeUnitBackref.groups.x, "𝌆", "code-unit backref capture");
var unicodeBackref = /(?<x>.)\k<x>/u.exec("𝌆𝌆");
check(unicodeBackref[0], "𝌆𝌆", "unicode backref full");
check(unicodeBackref.groups.x, "𝌆", "unicode backref capture");

var noCaptureIndices = /a/d.exec("ba");
check(noCaptureIndices.indices[0][0], 1, "no-capture indices start");
check(noCaptureIndices.indices[0][1], 2, "no-capture indices end");
check(noCaptureIndices.indices.groups, undefined, "no-capture indices groups");

true;
