function catchesReferenceError(fn) {
  try {
    fn();
  } catch (error) {
    return error.name === "ReferenceError";
  }
  return false;
}

if (!catchesReferenceError(function () {
  let x = 1;
  for (let x in { x }) {}
})) {
  throw "for-in let head";
}

if (!catchesReferenceError(function () {
  let x = 1;
  for (const x in { x }) {}
})) {
  throw "for-in const head";
}

var probeExpr;
let x = "outside";
for (let x in { i: probeExpr = function () { typeof x; } }) {}

if (!catchesReferenceError(probeExpr)) {
  throw "for-in delayed head";
}

if (!catchesReferenceError(function () {
  let y = 1;
  for (let y of [y]) {}
})) {
  throw "for-of let head";
}

true;
