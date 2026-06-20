let $262 = { createRealm: __porfCreateRealm };
let other = $262.createRealm().global;
let C = other.Proxy;
C.prototype = null;

let result = Reflect.construct(Boolean, [], C);
if (Object.getPrototypeOf(result) !== other.Boolean.prototype) {
  throw "cross-realm Boolean prototype";
}

123;
