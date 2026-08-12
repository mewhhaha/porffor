"use strict";

var obj = {};
var threw = false;

Object.preventExtensions(obj);

try {
  obj.x = 1;
} catch (err) {
  threw = err instanceof TypeError;
}

threw === true && obj.x === undefined;
