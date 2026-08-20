let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

if (typeof other.RegExp !== "function") {
  throw "cross-realm RegExp constructor";
}

if (typeof other.RegExp.escape !== "function") {
  throw "cross-realm RegExp.escape";
}

let value = other.RegExp.escape.call(RegExp, "oi+hello");
if (value !== "\\x6fi\\+hello") {
  throw "cross-realm RegExp.escape result";
}

true;
