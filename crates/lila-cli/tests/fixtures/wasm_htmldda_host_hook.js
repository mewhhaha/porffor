function __lilaIsHTMLDDA() {
  return "ordinary";
}

if (typeof __lilaIsHTMLDDA !== "function") throw "ordinary typeof";
if (!__lilaIsHTMLDDA) throw "ordinary truthy";
if (__lilaIsHTMLDDA == null) throw "ordinary loose null";
if (__lilaIsHTMLDDA() !== "ordinary") throw "ordinary call";
if (new __lilaIsHTMLDDA() instanceof __lilaIsHTMLDDA !== true) {
  throw "ordinary construct";
}

var $262 = {
  IsHTMLDDA: __lilaCreateHTMLDDA()
};

if ($262.IsHTMLDDA === undefined) throw "strict undefined";
if (typeof $262.IsHTMLDDA !== "undefined") throw "typeof";
if (!!$262.IsHTMLDDA !== false) throw "truthy";
if (!($262.IsHTMLDDA == null)) throw "loose null";
if (!($262.IsHTMLDDA == undefined)) throw "loose undefined";
if (Object.is($262.IsHTMLDDA, undefined) !== false) throw "object is undefined";
if ($262.IsHTMLDDA() !== null) throw "call result";
let coercionCalls = 0;
Object.defineProperty($262.IsHTMLDDA, Symbol.toPrimitive, {
  configurable: true,
  value() {
    coercionCalls++;
    throw "same-type equality coerced";
  }
});
if (!($262.IsHTMLDDA == $262.IsHTMLDDA) || coercionCalls !== 0) {
  throw "same-type identity";
}

let threw = false;
for (let construct of [
  function () { return new $262.IsHTMLDDA(); },
  function () { return Reflect.construct($262.IsHTMLDDA, []); },
  function () { return new ($262.IsHTMLDDA.bind(null))(); }
]) {
  threw = false;
  try {
    construct();
  } catch (error) {
    threw = error instanceof TypeError;
  }
  if (!threw) throw "constructable";
}

let items = {};
items[Symbol.iterator] = $262.IsHTMLDDA;
threw = false;
try {
  Array.from(items);
} catch (error) {
  threw = error instanceof TypeError;
}
if (!threw) throw "Array.from iterator";

let typedArrayConstructors = [
  Int8Array,
  Uint8Array,
  Uint8ClampedArray,
  Int16Array,
  Uint16Array,
  Int32Array,
  Uint32Array,
  Float32Array,
  Float64Array
];
for (let i = 0; i < typedArrayConstructors.length; i++) {
  let TypedArray = typedArrayConstructors[i];
  threw = false;
  try {
    TypedArray.from(items);
  } catch (error) {
    threw = error instanceof TypeError;
  }
  if (!threw) throw "TypedArray.from iterator";
}

let prototypeGetterCalled = false;
Object.defineProperty($262.IsHTMLDDA, "prototype", {
  get() {
    prototypeGetterCalled = true;
    return {};
  },
  configurable: true
});
threw = false;
try {
  class C extends $262.IsHTMLDDA {}
} catch (error) {
  threw = error instanceof TypeError;
}
if (!threw) throw "class heritage";
if (prototypeGetterCalled) throw "class heritage prototype";

262;
