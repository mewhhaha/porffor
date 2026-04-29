let buffer = new ArrayBuffer(8);
let $262 = { createRealm: __porfCreateRealm };
let proto = { marker: 1 };
let customNewTarget = function () {};
customNewTarget.prototype = proto;
let custom = Reflect.construct(DataView, [buffer, 1, 2], customNewTarget);
if (Object.getPrototypeOf(custom) !== proto) throw "custom prototype";
if (custom.constructor !== Object) throw "custom constructor";

let nullNewTarget = function () {};
nullNewTarget.prototype = null;
let fallback = Reflect.construct(DataView, [buffer], nullNewTarget);
if (Object.getPrototypeOf(fallback) !== DataView.prototype) throw "null fallback";

let realm = $262.createRealm().global;
let realmFallback = Reflect.construct(DataView, [buffer], realm.DataView);
if (Object.getPrototypeOf(realmFallback) !== realm.DataView.prototype) {
  throw "realm fallback";
}
if (realm.DataView.prototype === DataView.prototype) throw "realm identity";

789;
