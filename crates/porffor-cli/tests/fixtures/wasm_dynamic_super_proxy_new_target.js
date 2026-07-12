class Base {
  constructor(value) {
    this.value = value;
    this.observedNewTarget = new.target;
  }
}
class Direct extends Base {}
class Deeper extends Direct {}
class Explicit extends Base { constructor(value) { super(value); } }

let direct = new Direct(1);
let deeper = new Deeper(2);
let explicit = new Explicit(3);

function ReturningBase(value) { return { value: value, observedNewTarget: new.target }; }
Object.setPrototypeOf(Explicit, ReturningBase);
let changed = new Explicit(4);

let proxyBase = new Proxy(ReturningBase, {});
Object.setPrototypeOf(Direct, proxyBase);
let proxied = new Direct(5);

let nonconstructorThrows = false;
Object.setPrototypeOf(Direct, {});
try { new Direct(6); } catch (error) { nonconstructorThrows = error instanceof TypeError; }
let nullThrows = false;
Object.setPrototypeOf(Direct, null);
try { new Direct(7); } catch (error) { nullThrows = error instanceof TypeError; }

class NullExplicit extends null { constructor() { super(); } }
Object.setPrototypeOf(NullExplicit, Object);
let nullExplicit = new NullExplicit();

class NullDefault extends null {}
Object.setPrototypeOf(NullDefault, Array);
let nullDefault = new NullDefault(1, 2);

class DerivedArray extends Array {}
class DerivedU8 extends Uint8Array {}
let array = new DerivedArray(1, 2);
let typed = new DerivedU8(2);

let gets = 0;
let proxyPrototype = { marker: true };
let proxyNewTarget = new Proxy(function () {}, {
  get(target, key, receiver) {
    if (key === 'prototype') gets++;
    return key === 'prototype' ? proxyPrototype : Reflect.get(target, key, receiver);
  }
});
let proxyArray = Reflect.construct(Array, [1, 2], proxyNewTarget);

let sentinel = false;
let throwingNewTarget = new Proxy(function () {}, {
  get(target, key) { if (key === 'prototype') throw 'sentinel'; }
});
try { Reflect.construct(Array, [], throwingNewTarget); } catch (error) { sentinel = error === 'sentinel'; }

let revokedThrows = false;
let revocable = Proxy.revocable(function () {}, {});
revocable.revoke();
try { Reflect.construct(Array, [], revocable.proxy); } catch (error) { revokedThrows = error instanceof TypeError; }

function PrimitiveNewTarget() {}
PrimitiveNewTarget.prototype = 1;
let primitiveArray = Reflect.construct(Array, [1], PrimitiveNewTarget);

let existing = { existing: true };
let sameNew = new Object(existing) === existing;
let sameReflect = Reflect.construct(Object, [existing], Object) === existing;
function Different() {}
Different.prototype = { different: true };
let freshObject = Reflect.construct(Object, [existing], Different);
let freshPrimitive = Reflect.construct(Object, [1], Different);

explicit.value === 3 && explicit.observedNewTarget === Explicit &&
  changed.value === 4 && changed.observedNewTarget === Explicit &&
  proxied.value === 5 && proxied.observedNewTarget === Direct &&
  nonconstructorThrows && nullThrows &&
  Object.getPrototypeOf(nullExplicit) === NullExplicit.prototype &&
  Array.isArray(nullDefault) && nullDefault.length === 2 &&
  Array.isArray(array) && array.length === 2 && Object.getPrototypeOf(array) === DerivedArray.prototype &&
  typed.length === 2 && Object.getPrototypeOf(typed) === DerivedU8.prototype &&
  gets === 1 && Object.getPrototypeOf(proxyArray) === proxyPrototype &&
  Array.isArray(proxyArray) && proxyArray.length === 2 && proxyArray[0] === 1 && proxyArray[1] === 2 &&
  sentinel && revokedThrows &&
  Object.getPrototypeOf(primitiveArray) === Array.prototype &&
  sameNew && sameReflect && freshObject !== existing &&
  Object.getPrototypeOf(freshObject) === Different.prototype &&
  Object.getPrototypeOf(freshPrimitive) === Different.prototype;
