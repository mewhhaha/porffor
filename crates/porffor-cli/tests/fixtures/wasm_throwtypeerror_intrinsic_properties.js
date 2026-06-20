function strictArgs() {
  "use strict";
  return arguments;
}

function findIndex(array, value) {
  for (var i = 0; i < array.length; i = i + 1) {
    if (array[i] === value) return i;
  }
  return -1;
}

var calleeDesc = Object.getOwnPropertyDescriptor(strictArgs(), "callee");
if (calleeDesc === undefined) throw "callee descriptor missing";

var thrower = calleeDesc.get;
if (typeof thrower !== "function") throw "thrower function";
if (calleeDesc.set !== thrower) throw "callee get/set identity";
if (calleeDesc.enumerable !== false) throw "callee enumerable";
if (calleeDesc.configurable !== false) throw "callee configurable";

var threw = false;
try {
  thrower();
} catch (error) {
  if (!(error instanceof TypeError)) throw error;
  threw = true;
}
if (!threw) throw "thrower did not throw";

if (Object.getPrototypeOf(thrower) !== Function.prototype) throw "thrower prototype";

var lengthDesc = Object.getOwnPropertyDescriptor(thrower, "length");
if (thrower.length !== 0) throw "length value";
if (lengthDesc.value !== 0) throw "length descriptor value";
if (lengthDesc.writable !== false) throw "length writable";
if (lengthDesc.enumerable !== false) throw "length enumerable";
if (lengthDesc.configurable !== false) throw "length configurable";

var nameDesc = Object.getOwnPropertyDescriptor(thrower, "name");
if (thrower.name !== "") throw "name value";
if (nameDesc.value !== "") throw "name descriptor value";
if (nameDesc.writable !== false) throw "name writable";
if (nameDesc.enumerable !== false) throw "name enumerable";
if (nameDesc.configurable !== false) throw "name configurable";

var names = Object.getOwnPropertyNames(thrower);
var lengthIndex = findIndex(names, "length");
var nameIndex = findIndex(names, "name");
if (lengthIndex < 0 || nameIndex !== lengthIndex + 1) throw "property order";

if (Object.isExtensible(thrower) !== false) throw "extensible";
if (Object.isFrozen(thrower) !== true) throw "frozen";

var functionArguments = Object.getOwnPropertyDescriptor(Function.prototype, "arguments");
var functionCaller = Object.getOwnPropertyDescriptor(Function.prototype, "caller");
if (functionArguments.get !== thrower) throw "Function.prototype.arguments get";
if (functionArguments.set !== thrower) throw "Function.prototype.arguments set";
if (functionCaller.get !== thrower) throw "Function.prototype.caller get";
if (functionCaller.set !== thrower) throw "Function.prototype.caller set";

threw = false;
try {
  strictArgs().callee;
} catch (error) {
  if (!(error instanceof TypeError)) throw error;
  threw = true;
}
if (!threw) throw "callee read did not throw";

true;
