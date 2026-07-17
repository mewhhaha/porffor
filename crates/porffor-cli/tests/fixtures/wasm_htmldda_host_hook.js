function __porfIsHTMLDDA() {
  return null;
}

var $262 = {
  IsHTMLDDA: __porfIsHTMLDDA
};

if ($262.IsHTMLDDA === undefined) throw "strict undefined";
if (typeof $262.IsHTMLDDA !== "undefined") throw "typeof";
if (!!$262.IsHTMLDDA !== false) throw "truthy";
if (!($262.IsHTMLDDA == null)) throw "loose null";
if (!($262.IsHTMLDDA == undefined)) throw "loose undefined";
if (Object.is($262.IsHTMLDDA, undefined) !== false) throw "object is undefined";
if ($262.IsHTMLDDA() !== null) throw "call result";

let items = {};
items[Symbol.iterator] = $262.IsHTMLDDA;
let threw = false;
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
