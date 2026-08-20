let parent = { x: 1 };
let objectValue = Object.create(parent);
let arrayValue = Array(1, 2);
let errorValue = Error("x");
let rangeErrorValue = RangeError("range");
let syntaxErrorValue = new SyntaxError("syntax");
let evalErrorValue = EvalError("eval");
let uriErrorValue = new URIError("uri");

function f() {}
class C {}

typeof Function === "function"
  && Function === globalThis.Function
  && f instanceof Function
  && C instanceof Function
  && Object.getPrototypeOf(f) === Function.prototype
  && rangeErrorValue.name === "RangeError"
  && syntaxErrorValue instanceof Error
  && evalErrorValue.name === "EvalError"
  && uriErrorValue.message === "uri"
  && new RangeError("x") instanceof Error
  && new SyntaxError("x") instanceof Error
  && new EvalError("x") instanceof Error
  && new URIError("x") instanceof Error
  && new TypeError("y") instanceof Error
  && new ReferenceError("z").name === "ReferenceError"
  && errorValue.message === "x"
  && Object.getPrototypeOf(objectValue) === parent
  && objectValue.x === 1
  && ({}) instanceof Object
  && [] instanceof Array
  && Array.isArray(arrayValue)
  && !Array.isArray({})
  && arrayValue[1] === 2;
