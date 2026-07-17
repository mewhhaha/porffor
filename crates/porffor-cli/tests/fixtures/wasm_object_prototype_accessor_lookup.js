let lookupGetter = Object.prototype.__lookupGetter__;
let lookupSetter = Object.prototype.__lookupSetter__;

if (typeof lookupGetter !== "function") throw "lookup getter missing";
if (typeof lookupSetter !== "function") throw "lookup setter missing";
if (lookupGetter.length !== 1 || lookupSetter.length !== 1) throw "lookup arity";

function getter() {
  throw "getter invoked";
}
function setter() {}

let root = {};
Object.defineProperty(root, "accessor", {
  get: getter,
  set: setter,
  configurable: true,
});
let child = Object.create(root);
if (lookupGetter.call(child, "accessor") !== getter) throw "inherited getter";
if (lookupSetter.call(child, "accessor") !== setter) throw "inherited setter";
if (lookupGetter.call(child, "missing") !== undefined) throw "missing getter";
if (lookupSetter.call(child, "missing") !== undefined) throw "missing setter";

let getterOnly = {};
Object.defineProperty(getterOnly, "accessor", { get: getter });
if (lookupGetter.call(getterOnly, "accessor") !== getter) throw "own getter";
if (lookupSetter.call(getterOnly, "accessor") !== undefined) throw "missing accessor setter";

let setterOnly = {};
Object.defineProperty(setterOnly, "accessor", { set: setter });
if (lookupGetter.call(setterOnly, "accessor") !== undefined) throw "missing accessor getter";
if (lookupSetter.call(setterOnly, "accessor") !== setter) throw "own setter";

let ownData = Object.create(root);
Object.defineProperty(ownData, "accessor", { value: 1 });
if (lookupGetter.call(ownData, "accessor") !== undefined) throw "own data getter";
if (lookupSetter.call(ownData, "accessor") !== undefined) throw "own data setter";

let prototypeData = Object.create(root);
Object.defineProperty(prototypeData, "accessor", { value: 1 });
let dataChild = Object.create(prototypeData);
if (lookupGetter.call(dataChild, "accessor") !== undefined) throw "prototype data getter";
if (lookupSetter.call(dataChild, "accessor") !== undefined) throw "prototype data setter";

let keyCoercions = 0;
let accessorKey = {
  toString: function() {
    keyCoercions += 1;
    return "accessor";
  },
};
if (lookupGetter.call(root, accessorKey) !== getter || keyCoercions !== 1) {
  throw "getter key coercion";
}
keyCoercions = 0;
if (lookupSetter.call(root, accessorKey) !== setter || keyCoercions !== 1) {
  throw "setter key coercion";
}

let nullishKeyCoercions = 0;
let nullishKey = {
  toString: function() {
    nullishKeyCoercions += 1;
    return "accessor";
  },
};
let nullThrew = false;
try {
  lookupGetter.call(null, nullishKey);
} catch (error) {
  nullThrew = error instanceof TypeError;
}
let undefinedThrew = false;
try {
  lookupSetter.call(undefined, nullishKey);
} catch (error) {
  undefinedThrew = error instanceof TypeError;
}
if (!nullThrew || !undefinedThrew || nullishKeyCoercions !== 0) {
  throw "nullish receiver ordering";
}

Object.defineProperty(Number.prototype, "boxedAccessor", {
  get: getter,
  set: setter,
  configurable: true,
});
if (lookupGetter.call(1, "boxedAccessor") !== getter) throw "boxed number getter";
if (lookupSetter.call(1, "boxedAccessor") !== setter) throw "boxed number setter";
delete Number.prototype.boxedAccessor;
if (lookupGetter.call("text", "length") !== undefined) throw "boxed string own data";

let ownDescriptorCalls = 0;
let prototypeCalls = 0;
let proxy = new Proxy(Object.create(root), {
  getOwnPropertyDescriptor: function(target, key) {
    ownDescriptorCalls += 1;
    return Reflect.getOwnPropertyDescriptor(target, key);
  },
  getPrototypeOf: function(target) {
    prototypeCalls += 1;
    return Reflect.getPrototypeOf(target);
  },
});
if (lookupGetter.call(proxy, "accessor") !== getter) throw "proxy getter traversal";
if (ownDescriptorCalls !== 1 || prototypeCalls !== 1) throw "proxy traversal counts";

let marker = {};
let ownDescriptorThrow = new Proxy({}, {
  getOwnPropertyDescriptor: function() {
    throw marker;
  },
});
let ownDescriptorThrew = false;
try {
  lookupGetter.call(ownDescriptorThrow, "accessor");
} catch (error) {
  ownDescriptorThrew = error === marker;
}
if (!ownDescriptorThrew) throw "proxy own descriptor throw";

let prototypeThrow = new Proxy({}, {
  getPrototypeOf: function() {
    throw marker;
  },
});
let prototypeThrew = false;
try {
  lookupSetter.call(prototypeThrow, "accessor");
} catch (error) {
  prototypeThrew = error === marker;
}
if (!prototypeThrew) throw "proxy prototype throw";

true;
