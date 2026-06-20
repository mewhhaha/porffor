"use strict";

var symA = Symbol("A");
var symB = Symbol("B");
var symC = Symbol("C");
var obj = {};
var assignThrew = false;
var defineThrew = false;

obj[symA] = 1;
Object.preventExtensions(obj);
obj[symA] = 2;

try {
  obj[symB] = 1;
} catch (err) {
  assignThrew = err instanceof TypeError;
}

try {
  Object.defineProperty(obj, symC, {
    value: 1
  });
} catch (err) {
  defineThrew = err instanceof TypeError;
}

assignThrew === true
  && defineThrew === true
  && obj[symA] === 2
  && delete obj[symA] === true
  && obj[symA] === undefined
  && obj[symB] === undefined
  && obj[symC] === undefined;
