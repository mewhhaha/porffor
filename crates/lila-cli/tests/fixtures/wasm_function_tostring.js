function demo(x) { return x + 1; }
let arrow = y => y + 2;
let o = { m(z) { return z + 3; } };
let regexpSpeciesGetter = Object.getOwnPropertyDescriptor(RegExp, Symbol.species).get;
let proxyFunction = new Proxy(function proxied() {}, {});
let proxyApply = new Proxy(function proxiedApply() {}, { apply() {} }).apply;

demo.toString() === "function demo(x) { return x + 1; }"
  && arrow.toString() === "y => y + 2"
  && o.m.toString() === "m(z) { return z + 3; }"
  && ("" + Array) === "function Array() { [native code] }"
  && ("" + Function.prototype.call) === "function call() { [native code] }"
  && ("" + RegExp.prototype[Symbol.match]) === "function [Symbol.match]() { [native code] }"
  && ("" + regexpSpeciesGetter) === "function get [Symbol.species]() { [native code] }"
  && ("" + proxyFunction) === "function () { [native code] }"
  && Function.prototype.toString.call(proxyFunction) === "function () { [native code] }"
  && ("" + proxyApply) === "function apply() { [native code] }"
  && ("" + function named() {}.bind(null)) === "function () { [native code] }";
