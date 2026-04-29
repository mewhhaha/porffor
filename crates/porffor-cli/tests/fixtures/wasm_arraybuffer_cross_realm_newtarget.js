let $262 = { createRealm: __porfCreateRealm };
let other = $262.createRealm().global;
let C = new other.Function();
C.prototype = null;

let result = Reflect.construct(ArrayBuffer, [], C);
if (Object.getPrototypeOf(result) !== other.ArrayBuffer.prototype) {
  throw "cross-realm ArrayBuffer prototype";
}
if (other.ArrayBuffer.prototype === ArrayBuffer.prototype) {
  throw "realm ArrayBuffer prototype identity";
}

123;
