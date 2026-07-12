var plain = {};
var array = [];
var fn = function() {};
var err = new Error();
var hasOwn = Function.prototype.call.bind(Object.prototype.hasOwnProperty);

function skipsDescriptorCheck(verifyProp) {
  if (!verifyProp) {
    return false;
  }
  return true;
}

Object.preventExtensions(plain);
Object.preventExtensions(array);
Object.preventExtensions(fn);
Object.preventExtensions(err);

plain.exName = "plain";
array.exName = "array";
fn.exName = "fn";
err.exName = "err";
err[0] = "indexed";
array[0] = "sloppy-index";

var strictArrayIndexThrew = false;
try {
  (function() {
    "use strict";
    array[0] = "strict-index";
  })();
} catch (error) {
  strictArrayIndexThrew = error instanceof TypeError;
}

Object.isExtensible(plain) === false
  && Object.isExtensible(array) === false
  && Object.isExtensible(fn) === false
  && Object.isExtensible(err) === false
  && plain.hasOwnProperty("exName") === false
  && array.hasOwnProperty("exName") === false
  && fn.hasOwnProperty("exName") === false
  && err.hasOwnProperty("exName") === false
  && err.hasOwnProperty("0") === false
  && array.hasOwnProperty("0") === false
  && array.length === 0
  && strictArrayIndexThrew === true
  && hasOwn(plain, "exName") === false
  && hasOwn(array, "exName") === false
  && hasOwn(fn, "exName") === false
  && hasOwn(err, "exName") === false
  && hasOwn(err, "0") === false
  && skipsDescriptorCheck("nocheck") === true;
