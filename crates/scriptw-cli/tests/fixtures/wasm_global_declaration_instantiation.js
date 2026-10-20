var Infinity;
var NaN;
var undefined;

var duplicate;
function duplicate() {
  return 1;
}
function duplicate() {
  return 2;
}

if (Infinity !== 1 / 0) throw "var replaced Infinity";
if (NaN === NaN) throw "var replaced NaN";
if (undefined !== void 0) throw "var replaced undefined";
if (duplicate() !== 2) throw "last duplicate function did not win";

var hardened = 1;
Object.defineProperty(globalThis, "hardened", { writable: false });
hardened = 2;
if (hardened !== 1 || globalThis.hardened !== 1) {
  throw "script-global cache retained a rejected Set value";
}

if (true) {
  function annexB() {
    return 3;
  }
}
if (annexB() !== 3 || globalThis.annexB !== annexB) {
  throw "Annex B copy missed its script-global target";
}

let infinityDesc = Object.getOwnPropertyDescriptor(globalThis, "Infinity");
let nanDesc = Object.getOwnPropertyDescriptor(globalThis, "NaN");
let undefinedDesc = Object.getOwnPropertyDescriptor(globalThis, "undefined");

infinityDesc.writable === false
  && infinityDesc.enumerable === false
  && infinityDesc.configurable === false
  && nanDesc.writable === false
  && nanDesc.enumerable === false
  && nanDesc.configurable === false
  && undefinedDesc.writable === false
  && undefinedDesc.enumerable === false
  && undefinedDesc.configurable === false;
