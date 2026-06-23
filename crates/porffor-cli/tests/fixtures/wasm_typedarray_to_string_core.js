"use strict";

let failures = 0;

function expectTypeError(bit, thunk) {
  let threw = false;
  try {
    thunk();
  } catch (e) {
    threw = e instanceof TypeError;
  }
  if (!threw) failures |= bit;
}

let sample = new Uint8Array([1, 2, 3]);
if (sample.toString() !== "1,2,3") failures |= 1;

let empty = new Uint8Array(0);
if (empty.toString() !== "") failures |= 2;

let detached = new Uint8Array(0);
__porfDetachArrayBuffer(detached.buffer);
expectTypeError(4, function () { detached.toString(); });

let typedArrayPrototype = Object.getPrototypeOf(Uint8Array.prototype);
let fn = typedArrayPrototype.toString;
if (typeof fn !== "function") failures |= 8;
expectTypeError(16, function () { new fn(); });
if (fn !== Array.prototype.toString) failures |= 32;
if (Uint8Array.prototype.toString !== fn) failures |= 64;
if (Array.prototype.toString.call(true) !== "[object Boolean]") failures |= 128;
if (Array.prototype.toString.call({ join: null }) !== "[object Object]") failures |= 256;

Array.prototype[1] = 9;
let inherited = [1];
inherited.length = 2;
if (inherited.toString() !== "1,9") failures |= 512;
delete Array.prototype[1];

failures === 0;
