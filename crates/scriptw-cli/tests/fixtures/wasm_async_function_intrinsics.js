async function value() {}

let AsyncFunction = value.constructor;

Object.getPrototypeOf(AsyncFunction) === Function
  && Object.getPrototypeOf(AsyncFunction.prototype) === Function.prototype
  && value instanceof Function
  ? 123
  : 0;
