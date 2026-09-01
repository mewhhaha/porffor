function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

function assertDataDescriptor(
  object,
  key,
  expectedValue,
  expectedWritable,
  expectedConfigurable,
  label
) {
  var descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (descriptor === undefined) throw label + " missing";
  assertSame(descriptor.value, expectedValue, label + " value");
  assertSame(descriptor.writable, expectedWritable, label + " writable");
  assertSame(descriptor.enumerable, false, label + " enumerable");
  assertSame(
    descriptor.configurable,
    expectedConfigurable,
    label + " configurable"
  );
}

function assertRealmTypeError(callback, expectedPrototype, label) {
  var thrown;
  try {
    callback();
  } catch (error) {
    thrown = error;
  }
  if (thrown === undefined) throw label + " did not throw";
  assertSame(Object.getPrototypeOf(thrown), expectedPrototype, label + " realm");
}

function assertTypedArrayIntrinsic(realmGlobal, constructors, label) {
  var typedArrayConstructor = Object.getPrototypeOf(constructors[0]);
  var typedArrayPrototype = typedArrayConstructor.prototype;

  assertSame(typeof typedArrayConstructor, "function", label + " typeof");
  assertSame(
    typedArrayConstructor === realmGlobal.Function,
    false,
    label + " dedicated identity"
  );
  assertSame(typedArrayConstructor.name, "TypedArray", label + " name");
  assertSame(typedArrayConstructor.length, 0, label + " length");
  assertSame(
    realmGlobal.Function.prototype.toString.call(typedArrayConstructor),
    "function TypedArray() { [native code] }",
    label + " native source"
  );
  assertSame(
    Object.getPrototypeOf(typedArrayConstructor),
    realmGlobal.Function.prototype,
    label + " function prototype"
  );
  assertSame(
    Object.getOwnPropertyDescriptor(realmGlobal, "TypedArray"),
    undefined,
    label + " hidden global"
  );

  assertDataDescriptor(
    typedArrayConstructor,
    "name",
    "TypedArray",
    false,
    true,
    label + " name descriptor"
  );
  assertDataDescriptor(
    typedArrayConstructor,
    "length",
    0,
    false,
    true,
    label + " length descriptor"
  );
  assertDataDescriptor(
    typedArrayConstructor,
    "prototype",
    typedArrayPrototype,
    false,
    false,
    label + " prototype descriptor"
  );
  assertDataDescriptor(
    typedArrayPrototype,
    "constructor",
    typedArrayConstructor,
    true,
    true,
    label + " constructor descriptor"
  );

  for (var i = 0; i < constructors.length; i = i + 1) {
    assertSame(
      Object.getPrototypeOf(constructors[i]),
      typedArrayConstructor,
      label + " constructor identity"
    );
    assertSame(
      Object.getPrototypeOf(constructors[i].prototype),
      typedArrayPrototype,
      label + " prototype identity"
    );
  }

  assertRealmTypeError(
    function () {
      typedArrayConstructor();
    },
    realmGlobal.TypeError.prototype,
    label + " direct call"
  );
  assertRealmTypeError(
    function () {
      new typedArrayConstructor();
    },
    realmGlobal.TypeError.prototype,
    label + " direct construct"
  );
  assertRealmTypeError(
    function () {
      realmGlobal.Reflect.construct(
        typedArrayConstructor,
        [],
        realmGlobal.Object
      );
    },
    realmGlobal.TypeError.prototype,
    label + " reflected target"
  );

  var constructed = realmGlobal.Reflect.construct(
    realmGlobal.Object,
    [],
    typedArrayConstructor
  );
  assertSame(
    Object.getPrototypeOf(constructed),
    typedArrayPrototype,
    label + " IsConstructor newTarget prototype"
  );

  return typedArrayConstructor;
}

var entryConstructors = [
  Float64Array,
  Float32Array,
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array,
  Uint8ClampedArray,
  BigInt64Array,
  BigUint64Array,
];
var other = __lilaCreateRealm().global;
var otherConstructors = [
  other.Float64Array,
  other.Float32Array,
  other.Int32Array,
  other.Int16Array,
  other.Int8Array,
  other.Uint32Array,
  other.Uint16Array,
  other.Uint8Array,
  other.Uint8ClampedArray,
  other.BigInt64Array,
  other.BigUint64Array,
];

var entryTypedArray = assertTypedArrayIntrinsic(
  globalThis,
  entryConstructors,
  "entry realm"
);
var otherTypedArray = assertTypedArrayIntrinsic(
  other,
  otherConstructors,
  "created realm"
);
assertSame(
  entryTypedArray === otherTypedArray,
  false,
  "per-realm TypedArray identity"
);
assertSame(
  entryTypedArray.prototype === otherTypedArray.prototype,
  false,
  "per-realm TypedArray prototype identity"
);

true;
