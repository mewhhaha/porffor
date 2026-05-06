let object = {};
object.foo = 101;

let desc = Object.getOwnPropertyDescriptor(object, "foo");

let mixedRejected = false;
try {
  Object.defineProperty(object, "bar", {
    get: function() {
      return 1;
    },
    value: 2
  });
} catch (error) {
  mixedRejected = error instanceof TypeError;
}

Object.defineProperty(object, "undefData", { value: undefined });
let undefDataDesc = Object.getOwnPropertyDescriptor(object, "undefData");

Object.defineProperty(object, "undefGetter", { get: undefined });
let undefGetterDesc = Object.getOwnPropertyDescriptor(object, "undefGetter");

let mixedUndefinedRejected = false;
try {
  Object.defineProperty(object, "mixedUndefined", {
    get: function() {
      return 3;
    },
    value: undefined
  });
} catch (error) {
  mixedUndefinedRejected = error instanceof TypeError;
}

Object.defineProperties(object, {
  propsUndefData: { value: undefined },
  propsUndefGetter: { get: undefined },
  propsOne: { value: 11, writable: true },
  propsTwo: { value: 22, enumerable: true, configurable: true }
});
let propsUndefDataDesc = Object.getOwnPropertyDescriptor(object, "propsUndefData");
let propsUndefGetterDesc = Object.getOwnPropertyDescriptor(object, "propsUndefGetter");
let propsOneDesc = Object.getOwnPropertyDescriptor(object, "propsOne");
let propsTwoDesc = Object.getOwnPropertyDescriptor(object, "propsTwo");

let propsMixedRejected = false;
let propsMixedTarget = {};
try {
  Object.defineProperties(propsMixedTarget, {
    propsMixed: {
      get: function() {
        return 4;
      },
      value: 1
    }
  });
} catch (error) {
  propsMixedRejected = error instanceof TypeError;
}

let redefineData = {};
Object.defineProperty(redefineData, "x", {
  value: undefined,
  writable: true,
  enumerable: true,
  configurable: true
});
Object.defineProperties(redefineData, {
  x: { value: 200 }
});
let redefineDataDesc = Object.getOwnPropertyDescriptor(redefineData, "x");

let accessorValue = 10;
function sameGetter() {
  return accessorValue;
}
function sameSetter(value) {
  accessorValue = value;
}
let redefineAccessor = {};
Object.defineProperty(redefineAccessor, "x", {
  get: sameGetter,
  set: sameSetter,
  enumerable: false,
  configurable: true
});
Object.defineProperties(redefineAccessor, {
  x: {
    get: sameGetter,
    set: sameSetter,
    enumerable: true,
    configurable: true
  }
});
redefineAccessor.x = 30;
let redefineAccessorDesc = Object.getOwnPropertyDescriptor(redefineAccessor, "x");

let undefinedGetterSetterHit = 0;
function undefinedGetterSetter(value) {
  undefinedGetterSetterHit = value;
}
let undefinedGetterTarget = {};
Object.defineProperty(undefinedGetterTarget, "x", {
  get: undefined,
  set: undefinedGetterSetter,
  enumerable: true,
  configurable: false
});
Object.defineProperties(undefinedGetterTarget, {
  x: { get: undefined }
});
undefinedGetterTarget.x = 40;
let undefinedGetterDesc = Object.getOwnPropertyDescriptor(undefinedGetterTarget, "x");

let writableOnly = {};
Object.defineProperty(writableOnly, "x", {
  value: 55,
  writable: true,
  enumerable: true,
  configurable: true
});
Object.defineProperty(writableOnly, "x", { writable: false });
let writableOnlyDesc = Object.getOwnPropertyDescriptor(writableOnly, "x");

let genericAccessorValue = 70;
let genericAccessor = {};
Object.defineProperty(genericAccessor, "x", {
  get: function() {
    return genericAccessorValue;
  },
  set: function(value) {
    genericAccessorValue = value;
  },
  enumerable: true,
  configurable: true
});
Object.defineProperty(genericAccessor, "x", { enumerable: false });
genericAccessor.x = 80;
let genericAccessorDesc = Object.getOwnPropertyDescriptor(genericAccessor, "x");

let frozenData = {};
Object.defineProperty(frozenData, "x", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: false
});
let rejectConfigurable = false;
try {
  Object.defineProperty(frozenData, "x", { configurable: true });
} catch (error) {
  rejectConfigurable = error instanceof TypeError;
}
let rejectEnumerable = false;
try {
  Object.defineProperty(frozenData, "x", { enumerable: true });
} catch (error) {
  rejectEnumerable = error instanceof TypeError;
}
let rejectValue = false;
try {
  Object.defineProperty(frozenData, "x", { value: 2 });
} catch (error) {
  rejectValue = error instanceof TypeError;
}
let rejectWritable = false;
try {
  Object.defineProperty(frozenData, "x", { writable: true });
} catch (error) {
  rejectWritable = error instanceof TypeError;
}
Object.defineProperty(frozenData, "x", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: false
});
let frozenDataDesc = Object.getOwnPropertyDescriptor(frozenData, "x");

let frozenAccessorValue = 5;
function frozenGetter() {
  return frozenAccessorValue;
}
function frozenSetter(value) {
  frozenAccessorValue = value;
}
function otherFrozenGetter() {
  return 9;
}
function otherFrozenSetter(value) {
  frozenAccessorValue = value + 1;
}
let frozenAccessor = {};
Object.defineProperty(frozenAccessor, "x", {
  get: frozenGetter,
  set: frozenSetter,
  enumerable: true,
  configurable: false
});
let rejectAccessorToData = false;
try {
  Object.defineProperty(frozenAccessor, "x", { value: 10 });
} catch (error) {
  rejectAccessorToData = error instanceof TypeError;
}
let rejectGetterChange = false;
try {
  Object.defineProperty(frozenAccessor, "x", { get: otherFrozenGetter });
} catch (error) {
  rejectGetterChange = error instanceof TypeError;
}
let rejectSetterChange = false;
try {
  Object.defineProperty(frozenAccessor, "x", { set: otherFrozenSetter });
} catch (error) {
  rejectSetterChange = error instanceof TypeError;
}
Object.defineProperty(frozenAccessor, "x", {
  get: frozenGetter,
  set: frozenSetter,
  enumerable: true,
  configurable: false
});
frozenAccessor.x = 6;
let frozenAccessorDesc = Object.getOwnPropertyDescriptor(frozenAccessor, "x");

let ok = true;
if (desc.value !== 101) ok = false;
if (desc.writable !== true) ok = false;
if (desc.enumerable !== true) ok = false;
if (desc.configurable !== true) ok = false;
if (desc.get !== undefined) ok = false;
if (desc.set !== undefined) ok = false;
if (mixedRejected !== true) ok = false;
if (Object.getOwnPropertyDescriptor(object, "bar") !== undefined) ok = false;
if (undefDataDesc.value !== undefined) ok = false;
if (undefDataDesc.writable !== false) ok = false;
if (undefDataDesc.enumerable !== false) ok = false;
if (undefDataDesc.configurable !== false) ok = false;
if (undefGetterDesc.value !== undefined) ok = false;
if (undefGetterDesc.writable !== undefined) ok = false;
if (undefGetterDesc.get !== undefined) ok = false;
if (mixedUndefinedRejected !== true) ok = false;
if (Object.getOwnPropertyDescriptor(object, "mixedUndefined") !== undefined) ok = false;
if (propsUndefDataDesc.value !== undefined) ok = false;
if (propsUndefDataDesc.writable !== false) ok = false;
if (propsUndefDataDesc.enumerable !== false) ok = false;
if (propsUndefDataDesc.configurable !== false) ok = false;
if (propsUndefGetterDesc.value !== undefined) ok = false;
if (propsUndefGetterDesc.writable !== undefined) ok = false;
if (propsUndefGetterDesc.get !== undefined) ok = false;
if (propsOneDesc.value !== 11) ok = false;
if (propsOneDesc.writable !== true) ok = false;
if (propsOneDesc.enumerable !== false) ok = false;
if (propsOneDesc.configurable !== false) ok = false;
if (propsTwoDesc.value !== 22) ok = false;
if (propsTwoDesc.writable !== false) ok = false;
if (propsTwoDesc.enumerable !== true) ok = false;
if (propsTwoDesc.configurable !== true) ok = false;
if (propsMixedRejected !== true) ok = false;
if (Object.getOwnPropertyDescriptor(propsMixedTarget, "propsMixed") !== undefined) ok = false;
if (redefineDataDesc.value !== 200) ok = false;
if (redefineDataDesc.writable !== true) ok = false;
if (redefineDataDesc.enumerable !== true) ok = false;
if (redefineDataDesc.configurable !== true) ok = false;
if (redefineAccessor.x !== 30) ok = false;
if (redefineAccessorDesc.get !== sameGetter) ok = false;
if (redefineAccessorDesc.set !== sameSetter) ok = false;
if (redefineAccessorDesc.enumerable !== true) ok = false;
if (redefineAccessorDesc.configurable !== true) ok = false;
if (undefinedGetterTarget.x !== undefined) ok = false;
if (undefinedGetterSetterHit !== 40) ok = false;
if (undefinedGetterDesc.get !== undefined) ok = false;
if (undefinedGetterDesc.set !== undefinedGetterSetter) ok = false;
if (undefinedGetterDesc.enumerable !== true) ok = false;
if (undefinedGetterDesc.configurable !== false) ok = false;
if (writableOnlyDesc.value !== 55) ok = false;
if (writableOnlyDesc.writable !== false) ok = false;
if (writableOnlyDesc.enumerable !== true) ok = false;
if (writableOnlyDesc.configurable !== true) ok = false;
if (genericAccessor.x !== 80) ok = false;
if (genericAccessorDesc.get === undefined) ok = false;
if (genericAccessorDesc.set === undefined) ok = false;
if (genericAccessorDesc.enumerable !== false) ok = false;
if (genericAccessorDesc.configurable !== true) ok = false;
if (rejectConfigurable !== true) ok = false;
if (rejectEnumerable !== true) ok = false;
if (rejectValue !== true) ok = false;
if (rejectWritable !== true) ok = false;
if (frozenDataDesc.value !== 1) ok = false;
if (frozenDataDesc.writable !== false) ok = false;
if (frozenDataDesc.enumerable !== false) ok = false;
if (frozenDataDesc.configurable !== false) ok = false;
if (rejectAccessorToData !== true) ok = false;
if (rejectGetterChange !== true) ok = false;
if (rejectSetterChange !== true) ok = false;
if (frozenAccessor.x !== 6) ok = false;
if (frozenAccessorDesc.get !== frozenGetter) ok = false;
if (frozenAccessorDesc.set !== frozenSetter) ok = false;
if (frozenAccessorDesc.enumerable !== true) ok = false;
if (frozenAccessorDesc.configurable !== false) ok = false;
ok;
