use super::*;
use porffor_ir::{OptionalChainOperationIr, RegExpProgram};

#[derive(Debug)]
struct StringRef {
    offset: u32,
    len: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RegExpProgramRef {
    pub(crate) ptr: u32,
    pub(crate) instruction_count: u32,
}

#[derive(Debug, Default)]
pub(crate) struct StringPool {
    pub(crate) bytes: Vec<u8>,
    refs: BTreeMap<String, StringRef>,
    regexp_programs: BTreeMap<Vec<u8>, RegExpProgramRef>,
    pending_regexp_programs: Vec<(Vec<u8>, u32)>,
    pub(crate) uses_heap: bool,
}

impl StringPool {
    pub(crate) fn collect(
        script: &ScriptIr,
        function_metas: &BTreeMap<FunctionId, WasmFunctionMeta>,
    ) -> Self {
        let mut pool = Self::default();
        for value in [
            "",
            " ",
            "          ",
            "\n",
            ": ",
            ",",
            "undefined",
            "null",
            "true",
            "false",
            "0",
            "\"",
            "{",
            "}",
            "{}",
            "[",
            "]",
            "[]",
            ":",
            "0.0",
            "1",
            "1.0",
            "NaN",
            "Infinity",
            "-Infinity",
            "1e-7",
            "-1e-7",
            "100000000000000000000",
            "-100000000000000000000",
            "10203040506070809000",
            "-10203040506070809000",
            "1e+22",
            "-1e+22",
            "Symbol()",
            "-1e+21",
            "\\d",
            "\\d{1}",
            "\\d{2}",
            "\\D{2}",
            "0.",
            ".",
            "\u{20BB7}",
            "\\p{Script=Han}",
            "b.",
            "c.",
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
            "[\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}]",
            "\u{1F468}",
            "\u{1F469}",
            "\u{1F467}",
            "\u{1F466}",
            "\u{200D}",
            "\u{F0000}D842",
            "\u{F0000}DFB7",
            "\u{F0000}D83D",
            "\u{F0000}DC68",
            "\u{F0000}DC69",
            "\u{F0000}DC67",
            "\u{F0000}DC66",
            "([\\d]{5})([-\\ ]?[\\d]{4})?$",
            "(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))",
            "(?:(?:(?<x>a)|(?<x>b)|c)\\k<x>){2}",
            "groups",
            "indices",
            "x",
            "y",
            "z",
            "a",
            "b",
            "c",
            "d",
            "abc",
            "ad",
            "aac",
            "\\ud834",
            "\\udf06",
            "[object Object]",
            "[object Arguments]",
            "[object ",
            "prototype",
            "lastIndex",
            "index",
            "input",
            "dynamic eval unsupported",
            "$_",
            "lastMatch",
            "$&",
            "lastParen",
            "$+",
            "leftContext",
            "$`",
            "rightContext",
            "$'",
            "$1",
            "$2",
            "$3",
            "$4",
            "$5",
            "$6",
            "$7",
            "$8",
            "$9",
            "g",
            "l",
            "77",
            "\\u0037\\u0037",
            "\\s",
            "\\w",
            "\\d+",
            "[a-z]",
            "RegExp legacy static accessor receiver must be RegExp",
            "%ThrowTypeError%",
            "constructor",
            "callee",
            "arguments",
            "caller",
            "valueOf",
            "toString",
            "toUpperCase",
            "padStart",
            "padEnd",
            "repeat",
            "exec",
            "object",
            "boolean",
            "number",
            "string",
            "default",
            "function",
            "function(handle@",
            ")",
            "length",
            "name",
            "message",
            "error",
            "suppressed",
            "cause",
            "errors",
            "global",
            FUNCTION_NAME,
            OBJECT_NAME,
            ARRAY_NAME,
            ARRAY_BUFFER_NAME,
            DATA_VIEW_NAME,
            DATE_NAME,
            REGEXP_NAME,
            MATH_NAME,
            JSON_NAME,
            FLOAT64_ARRAY_NAME,
            FLOAT32_ARRAY_NAME,
            INT32_ARRAY_NAME,
            INT16_ARRAY_NAME,
            INT8_ARRAY_NAME,
            UINT32_ARRAY_NAME,
            UINT16_ARRAY_NAME,
            UINT8_ARRAY_NAME,
            UINT8_CLAMPED_ARRAY_NAME,
            "BigInt64Array",
            "BigUint64Array",
            "$Realm.Float64Array.prototype",
            "$Realm.Float32Array.prototype",
            "$Realm.Int32Array.prototype",
            "$Realm.Int16Array.prototype",
            "$Realm.Int8Array.prototype",
            "$Realm.Uint32Array.prototype",
            "$Realm.Uint16Array.prototype",
            "$Realm.Uint8Array.prototype",
            "$Realm.Uint8ClampedArray.prototype",
            "$Realm.BigInt64Array.prototype",
            "$Realm.BigUint64Array.prototype",
            "BYTES_PER_ELEMENT",
            TYPED_ARRAY_ELEMENT_KIND_SLOT,
            REFLECT_NAME,
            NUMBER_NAME,
            STRING_NAME,
            BOOLEAN_NAME,
            ERROR_NAME,
            EVAL_ERROR_NAME,
            AGGREGATE_ERROR_NAME,
            SUPPRESSED_ERROR_NAME,
            RANGE_ERROR_NAME,
            SYNTAX_ERROR_NAME,
            TYPE_ERROR_NAME,
            URI_ERROR_NAME,
            REFERENCE_ERROR_NAME,
            "call",
            "apply",
            "bind",
            "anchor",
            "big",
            "blink",
            "bold",
            "fixed",
            "fontcolor",
            "fontsize",
            "italics",
            "link",
            "small",
            "strike",
            "sub",
            "substr",
            "substring",
            "sup",
            "trim",
            "trimStart",
            "trimLeft",
            "trimEnd",
            "trimRight",
            "match",
            "matchAll",
            "replace",
            "replaceAll",
            "search",
            "split",
            "concat",
            "[object Undefined]",
            "[object Null]",
            "[object Boolean]",
            "[object Number]",
            "[object String]",
            "[object Symbol]",
            "[object Object]",
            "[object Array]",
            "[object Function]",
            "[object Arguments]",
            "[object BigInt]",
            "[object Error]",
            "from",
            "find",
            "findIndex",
            "findLast",
            "findLastIndex",
            "flat",
            "includes",
            "pop",
            "push",
            "shift",
            "splice",
            "entries",
            "values",
            "create",
            "getPrototypeOf",
            "setPrototypeOf",
            "defineProperty",
            "defineProperties",
            "getOwnPropertyDescriptor",
            "getOwnPropertyNames",
            "getOwnPropertySymbols",
            "keys",
            "hasOwn",
            "is",
            "isSealed",
            "isFrozen",
            "freeze",
            "isExtensible",
            "hasOwnProperty",
            "propertyIsEnumerable",
            "Object.prototype.propertyIsEnumerable called on null or undefined",
            "toString",
            "$IsHTMLDDA",
            "Symbol.iterator",
            "for-of target is not iterable",
            "for-of iterator method must be callable",
            "for-of iterator method must return object",
            "for-of iterator next must be callable",
            "for-of iterator next result must be object",
            "return",
            "IteratorClose return method must be callable",
            "IteratorClose return result must be object",
            "$ArrayIterator.array",
            "$ArrayIterator.index",
            "$ArrayIterator.done",
            "$ArrayIterator.kind",
            PORFFOR_STATIC_GENERATOR_VALUES_METHOD,
            PORFFOR_STATIC_GENERATOR_ITERATOR_SLOT,
            "$RegExpStringIterator.regexp",
            "$RegExpStringIterator.string",
            "$RegExpStringIterator.global",
            "$RegExpStringIterator.unicode",
            "$RegExpStringIterator.done",
            "Array Iterator",
            "Array.prototype iterator method called on null or undefined",
            "Array Iterator next called on incompatible receiver",
            "Array Iterator next called on out-of-bounds TypedArray",
            "RegExp String Iterator next called on incompatible receiver",
            "RegExp String Iterator exec returned non-object",
            "Iterator",
            "toArray",
            "forEach",
            "every",
            "some",
            "find",
            "reduce",
            "map",
            "filter",
            "take",
            "drop",
            "Iterator constructor cannot be called",
            "Iterator.from called on null or undefined",
            "Iterator.from iterator method must return object",
            "Iterator.from iterator method must be callable",
            "Iterator.from next method must be callable",
            "Iterator.prototype.forEach called on null or undefined",
            "Iterator.prototype.forEach callback must be callable",
            "Iterator.prototype.forEach next method must be callable",
            "Iterator.prototype.forEach next result must be object",
            "Iterator.prototype.every called on null or undefined",
            "Iterator.prototype.every callback must be callable",
            "Iterator.prototype.every next method must be callable",
            "Iterator.prototype.every next result must be object",
            "Iterator.prototype.some called on null or undefined",
            "Iterator.prototype.some callback must be callable",
            "Iterator.prototype.some next method must be callable",
            "Iterator.prototype.some next result must be object",
            "Iterator.prototype.find called on null or undefined",
            "Iterator.prototype.find callback must be callable",
            "Iterator.prototype.find next method must be callable",
            "Iterator.prototype.find next result must be object",
            "Iterator.prototype.reduce called on null or undefined",
            "Iterator.prototype.reduce reducer must be callable",
            "Iterator.prototype.reduce next method must be callable",
            "Iterator.prototype.reduce next result must be object",
            "Iterator.prototype.reduce of empty iterator with no initial value",
            "Iterator.prototype.map called on null or undefined",
            "Iterator.prototype.map mapper must be callable",
            "Iterator.prototype.map next method must be callable",
            "Iterator map helper next called on incompatible receiver",
            "Iterator map helper is already running",
            "Iterator map helper next result must be object",
            "Iterator map helper return called on incompatible receiver",
            "Iterator map helper return method must be callable",
            "Iterator map helper return result must be object",
            "$PorfforIteratorMapHelper",
            "$IteratorMapIterator",
            "$IteratorMapNext",
            "$IteratorMapMapper",
            "$IteratorMapIndex",
            "$IteratorMapDone",
            "$IteratorMapExecuting",
            "Iterator.prototype.filter called on null or undefined",
            "Iterator.prototype.filter predicate must be callable",
            "Iterator.prototype.filter next method must be callable",
            "Iterator filter helper next called on incompatible receiver",
            "Iterator filter helper is already running",
            "Iterator filter helper next result must be object",
            "Iterator filter helper return called on incompatible receiver",
            "Iterator filter helper return method must be callable",
            "Iterator filter helper return result must be object",
            "$PorfforIteratorFilterHelper",
            "$IteratorFilterIterator",
            "$IteratorFilterNext",
            "$IteratorFilterPredicate",
            "$IteratorFilterIndex",
            "$IteratorFilterDone",
            "$IteratorFilterExecuting",
            "flatMap",
            "Iterator.prototype.flatMap called on null or undefined",
            "Iterator.prototype.flatMap mapper must be callable",
            "Iterator.prototype.flatMap next method must be callable",
            "Iterator.prototype.flatMap mapper result must be object",
            "Iterator.prototype.flatMap inner iterator method must be callable",
            "Iterator.prototype.flatMap inner iterator method must return object",
            "Iterator.prototype.flatMap inner iterator next method must be callable",
            "Iterator flatMap helper next called on incompatible receiver",
            "Iterator flatMap helper is already running",
            "Iterator flatMap helper next result must be object",
            "Iterator flatMap helper return called on incompatible receiver",
            "Iterator flatMap helper return method must be callable",
            "Iterator flatMap helper return result must be object",
            "$PorfforIteratorFlatMapHelper",
            "$IteratorFlatMapIterator",
            "$IteratorFlatMapNext",
            "$IteratorFlatMapMapper",
            "$IteratorFlatMapIndex",
            "$IteratorFlatMapDone",
            "$IteratorFlatMapExecuting",
            "$IteratorFlatMapInnerIterator",
            "$IteratorFlatMapInnerNext",
            "$IteratorFlatMapInnerActive",
            "Iterator.prototype.take called on null or undefined",
            "Iterator.prototype.take next method must be callable",
            "Iterator.prototype.take limit must be a non-negative number",
            "$PorfforIteratorTakeHelper",
            "$IteratorTakeIterator",
            "$IteratorTakeNext",
            "$IteratorTakeRemaining",
            "$IteratorTakeDone",
            "$IteratorTakeExecuting",
            "Iterator take helper next called on incompatible receiver",
            "Iterator take helper is already running",
            "Iterator take helper next result must be object",
            "Iterator take helper return called on incompatible receiver",
            "Iterator take helper return method must be callable",
            "Iterator take helper return result must be object",
            "Iterator.prototype.drop called on null or undefined",
            "Iterator.prototype.drop next method must be callable",
            "Iterator.prototype.drop limit must be a non-negative number",
            "$PorfforIteratorDropHelper",
            "$IteratorDropIterator",
            "$IteratorDropNext",
            "$IteratorDropRemaining",
            "$IteratorDropDone",
            "$IteratorDropExecuting",
            "Iterator drop helper next called on incompatible receiver",
            "Iterator drop helper is already running",
            "Iterator drop helper next result must be object",
            "Iterator drop helper return called on incompatible receiver",
            "Iterator drop helper return method must be callable",
            "Iterator drop helper return result must be object",
            "Iterator.prototype.toArray called on null or undefined",
            "Iterator.prototype.toArray called on incompatible receiver",
            "Iterator.prototype.toArray next method must be callable",
            "Iterator.prototype.toArray next result must be object",
            "Iterator.prototype[Symbol.dispose] return method must be callable",
            "Iterator.prototype.constructor setter called on incompatible receiver",
            "Iterator.prototype[Symbol.toStringTag] setter called on incompatible receiver",
            "Iterator.from wrapper next called on incompatible receiver",
            "Iterator.from wrapper next method must be callable",
            "Iterator.from wrapper next result must be object",
            "Iterator.from wrapper return called on incompatible receiver",
            "Iterator.from wrapper return method must be callable",
            "Iterator.from wrapper return result must be object",
            "$PorfforIteratorFromWrapper",
            "$IteratorFromIterator",
            "$IteratorFromNext",
            "Array.prototype.values called on null or undefined",
            "Array.from iterator method must return object",
            "Array.from iterator method must be callable",
            "Array.from iterator next must be callable",
            "Array.from iterator next result must be object",
            "Array.from mapper is not callable",
            "Array.from called on null or undefined",
            "Array.from index property is non-configurable",
            "Array.from target is not extensible",
            "Array.of index property is non-configurable",
            "Array.of target is not extensible",
            "TypedArray.from receiver is not a constructor",
            "TypedArray.from mapper is not callable",
            "TypedArray.from constructed target is not a typed array",
            "TypedArray.from constructed target is too small",
            "TypedArray constructor requires new",
            "TypedArray byteOffset out of range",
            "TypedArray byteOffset must be aligned",
            "TypedArray byteLength out of range",
            "TypedArray byteLength must be aligned",
            "TypedArray backing buffer is detached",
            "TypedArray byteLength out of bounds",
            "TypedArray length out of range",
            "TypedArray.prototype.toString requires TypedArray",
            "TypedArray.prototype.toLocaleString requires TypedArray",
            "construct",
            "ownKeys",
            "has",
            "isArray",
            "isView",
            "isInteger",
            "isSafeInteger",
            "isFinite",
            "isNaN",
            "isError",
            "escape",
            "unescape",
            "asIntN",
            "asUintN",
            "E",
            "LN10",
            "LN2",
            "LOG10E",
            "LOG2E",
            "PI",
            "SQRT1_2",
            "SQRT2",
            "abs",
            "acos",
            "acosh",
            "asin",
            "asinh",
            "atan",
            "atan2",
            "atanh",
            "cbrt",
            "ceil",
            "clz32",
            "cos",
            "cosh",
            "exp",
            "expm1",
            "f16round",
            "floor",
            "fround",
            "hypot",
            "imul",
            "log",
            "log10",
            "log1p",
            "log2",
            "max",
            "min",
            "pow",
            "random",
            "round",
            "sign",
            "sin",
            "sinh",
            "sqrt",
            "sumPrecise",
            "tan",
            "tanh",
            "trunc",
            "toExponential",
            "toFixed",
            "toPrecision",
            "MAX_VALUE",
            "MIN_VALUE",
            "EPSILON",
            "MAX_SAFE_INTEGER",
            "MIN_SAFE_INTEGER",
            "POSITIVE_INFINITY",
            "NEGATIVE_INFINITY",
            "slice",
            "detached",
            "resize",
            "transfer",
            "transferToFixedLength",
            "transferToImmutable",
            "sliceToImmutable",
            "getUint8",
            "setUint8",
            "getInt8",
            "setInt8",
            "getUint16",
            "setUint16",
            "getInt16",
            "setInt16",
            "getUint32",
            "setUint32",
            "getInt32",
            "setInt32",
            "getFloat16",
            "setFloat16",
            "getFloat32",
            "setFloat32",
            "getFloat64",
            "setFloat64",
            "now",
            "UTC",
            "getTime",
            "setTime",
            "getFullYear",
            "getUTCFullYear",
            "getMonth",
            "getUTCMonth",
            "getDate",
            "getUTCDate",
            "getDay",
            "getUTCDay",
            "getHours",
            "getUTCHours",
            "getMinutes",
            "getUTCMinutes",
            "getSeconds",
            "getUTCSeconds",
            "getMilliseconds",
            "getUTCMilliseconds",
            "getTimezoneOffset",
            "getYear",
            "setYear",
            "toUTCString",
            "toGMTString",
            "buffer",
            "byteOffset",
            "byteLength",
            "maxByteLength",
            "resizable",
            "growable",
            "grow",
            "Symbol.dispose",
            "Symbol.species",
            "Symbol.isConcatSpreadable",
            "Symbol.match",
            "Symbol.matchAll",
            "Symbol.replace",
            "Symbol.search",
            "Symbol.split",
            "Symbol.toStringTag",
            "Symbol.toPrimitive",
            "Symbol is not a constructor",
            "Symbol.keyFor argument must be a symbol",
            "Symbol.asyncIterator",
            "Symbol.hasInstance",
            "Symbol.unscopables",
            "Symbol.asyncDispose",
            "Symbol",
            "Symbol(",
            "Symbol.prototype.description requires that 'this' be a Symbol",
            "Symbol.prototype.toString requires that 'this' be a Symbol",
            "Symbol.prototype.valueOf requires that 'this' be a Symbol",
            "Symbol.prototype[Symbol.toPrimitive] requires that 'this' be a Symbol",
            "Cannot create property on symbol",
            "iterator",
            "asyncIterator",
            "hasInstance",
            "isConcatSpreadable",
            "species",
            "toStringTag",
            "toPrimitive",
            "unscopables",
            "dispose",
            "asyncDispose",
            "for",
            "keyFor",
            "description",
            "(?:)",
            "source",
            "flags",
            "gc requires a real collector in wasm-aot",
            "parse",
            "stringify",
            "rawJSON",
            "isRawJSON",
            "add",
            "and",
            "compareExchange",
            "exchange",
            "load",
            "notify",
            "or",
            "pause",
            "store",
            "sub",
            "wait",
            "waitAsync",
            "xor",
            "Atomics.add requires an integer typed array",
            "Atomics.and requires an integer typed array",
            "Atomics.compareExchange requires an integer typed array",
            "Atomics.exchange requires an integer typed array",
            "Atomics.load requires an integer typed array",
            "Atomics.notify requires an Int32Array",
            "Atomics.or requires an integer typed array",
            "Atomics.pause iterationNumber must be a finite integral Number",
            "Atomics.store requires an integer typed array",
            "Atomics.sub requires an integer typed array",
            "Atomics.wait requires a shared Int32Array or BigInt64Array",
            "Atomics.waitAsync requires a shared Int32Array or BigInt64Array",
            "Atomics.xor requires an integer typed array",
            "Atomics.add index out of range",
            "Atomics.and index out of range",
            "Atomics.compareExchange index out of range",
            "Atomics.exchange index out of range",
            "Atomics.load index out of range",
            "Atomics.notify index out of range",
            "Atomics.or index out of range",
            "Atomics.store index out of range",
            "Atomics.sub index out of range",
            "Atomics.wait blocking wait queues unsupported in wasm-aot",
            "Atomics.wait index out of range",
            "Atomics.waitAsync blocking wait queues unsupported in wasm-aot",
            "Atomics.waitAsync index out of range",
            "not-equal",
            "timed-out",
            "Atomics.xor index out of range",
            "toJSON",
            "hasIndices",
            "unicodeSets",
            "ignoreCase",
            "multiline",
            "dotAll",
            "unicode",
            "sticky",
            "i",
            "m",
            "s",
            "u",
            "RegExp constructor is unsupported in wasm-aot",
            "Invalid regular expression flags",
            "Invalid regular expression pattern",
            "RegExp.prototype.flags getter receiver is not an object",
            "RegExp.prototype.exec receiver is not RegExp",
            "RegExp.prototype.exec source is not string",
            "RegExp.prototype.exec unsupported pattern",
            "RegExp.prototype[Symbol.match] flags is not string",
            "RegExp.prototype[Symbol.match] exec result is not object or null",
            "RegExp.prototype[Symbol.match] is unsupported in wasm-aot",
            ".(.).",
            "^|\\udf06",
            "RegExp.prototype[Symbol.matchAll] receiver is not RegExp",
            "RegExp.prototype[Symbol.matchAll] source is not string",
            "RegExp.prototype[Symbol.matchAll] flags is not string",
            "RegExp.prototype[Symbol.matchAll] species is not a constructor",
            "RegExp.prototype[Symbol.matchAll] is unsupported in wasm-aot",
            "RegExp.prototype[Symbol.search] receiver is not RegExp",
            "RegExp.prototype[Symbol.search] source is not string",
            "RegExp.prototype[Symbol.search] flags is not string",
            "RegExp.prototype[Symbol.search] exec result is not object or null",
            "RegExp.prototype[Symbol.search] is unsupported in wasm-aot",
            "\u{20BB7}",
            "\u{10FFFF}",
            "\u{20BB7}a\u{20BB7}b\u{20BB7}",
            "a\u{20BB7}b\u{10FFFF}c",
            "\\p{Script=Han}",
            "\\P{ASCII}",
            "String.prototype.matchAll RegExp flags must contain g",
            "String.prototype.matchAll RegExp @@matchAll is not callable",
            "Invalid JSON.parse text",
            "Invalid JSON.rawJSON text",
            "standard builtin body is not emitted unless referenced directly",
            "Cannot redefine JSON reviver property",
            "Cannot add JSON reviver property",
            "ArrayBuffer",
            SHARED_ARRAY_BUFFER_NAME,
            "DataView",
            "get",
            "set",
            "async",
            "value",
            "next",
            "done",
            "proxy",
            "revoke",
            "writable",
            "enumerable",
            "configurable",
            "$Proxy.target",
            "$Proxy.handler",
            "Proxy target must be object",
            "Proxy handler must be object",
            "Proxy get trap is not callable",
            "Proxy has trap is not callable",
            "Proxy getPrototypeOf trap is not callable",
            "Proxy getPrototypeOf trap result must be object or null",
            "Proxy getPrototypeOf trap result does not match target",
            "Proxy isExtensible trap is not callable",
            "Proxy isExtensible trap result does not match target",
            "Proxy setPrototypeOf trap is not callable",
            "Proxy setPrototypeOf trap result incompatible with non-extensible target",
            "Proxy handler is null",
            "Proxy set trap returned false",
            "Proxy set trap is not callable",
            "Proxy set trap result is incompatible with target descriptor",
            "Proxy get trap returned inconsistent frozen data property",
            "Proxy get trap returned value for accessor without getter",
            "Proxy getOwnPropertyDescriptor trap is not callable",
            "Proxy getOwnPropertyDescriptor trap result must be object or undefined",
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-configurable target property",
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-extensible target",
            "Proxy getOwnPropertyDescriptor trap result incompatible with non-extensible target",
            "Proxy getOwnPropertyDescriptor trap result cannot report non-configurable target property",
            "Proxy getOwnPropertyDescriptor trap result cannot report non-writable target property",
            "Proxy preventExtensions trap is not callable",
            "Proxy preventExtensions trap returned false",
            "Proxy preventExtensions trap returned true for extensible target",
            "Proxy defineProperty trap is not callable",
            "Proxy defineProperty trap returned false",
            "Proxy defineProperty trap cannot add property to non-extensible target",
            "Proxy defineProperty trap cannot define non-configurable target property",
            "Proxy defineProperty trap result is incompatible with target descriptor",
            "Proxy defineProperty trap cannot define non-writable target property",
            "Reflect.defineProperty target must be object",
            "Reflect.getPrototypeOf target must be object",
            "Reflect.getOwnPropertyDescriptor target must be object",
            "Reflect.set target must be object",
            "deleteProperty",
            "Reflect.deleteProperty target must be object",
            "Reflect.isExtensible target must be object",
            "Reflect.preventExtensions target must be object",
            "Reflect.ownKeys target must be object",
            "Proxy deleteProperty trap is not callable",
            "Proxy deleteProperty trap returned true for non-configurable target property",
            "Proxy deleteProperty trap returned true for non-extensible target property",
            "Proxy apply trap is not callable",
            "Proxy construct trap returned non-object",
            "Proxy construct trap is not callable",
            "Proxy has trap returned false for non-configurable target property",
            "Proxy has trap returned false for non-extensible target property",
            "Proxy ownKeys trap is not callable",
            "Proxy ownKeys trap result omitted target property",
            "2",
            "3",
            "4",
            "5",
            "10000",
            "prop",
            "f",
            "v",
            ARRAY_BUFFER_DATA_PTR_SLOT,
            ARRAY_BUFFER_BYTE_LENGTH_SLOT,
            ARRAY_BUFFER_IMMUTABLE_SLOT,
            ARRAY_BUFFER_MAX_BYTE_LENGTH_SLOT,
            ARRAY_BUFFER_RESIZABLE_SLOT,
            ARRAY_BUFFER_SHARED_SLOT,
            DATA_VIEW_DATA_PTR_SLOT,
            DATA_VIEW_BYTE_OFFSET_SLOT,
            DATA_VIEW_BYTE_LENGTH_SLOT,
            DATA_VIEW_LENGTH_TRACKING_SLOT,
            TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT,
            TYPED_ARRAY_BYTE_OFFSET_SLOT,
            TYPED_ARRAY_BYTE_LENGTH_SLOT,
            TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT,
            TYPED_ARRAY_LENGTH_TRACKING_SLOT,
            PORFFOR_GENERATOR_THROW_SLOT,
            DATE_VALUE_SLOT,
            "EvalError",
            "AggregateError",
            "RangeError",
            "SyntaxError",
            "TypeError",
            "URIError",
            "ReferenceError",
            "class constructor cannot be invoked without `new`",
            "dynamic Function constructor unsupported",
            "Math.sumPrecise non-number element",
            "Function.prototype.call receiver is not callable",
            "Function.prototype.apply receiver is not callable",
            "Function.prototype.bind receiver is not callable",
            "Function.prototype.toString receiver is not callable",
            "value is not callable",
            "Function.prototype.call primitive thisArg boxing unsupported",
            "Function.prototype.apply primitive thisArg boxing unsupported",
            "Function.prototype.call/apply thisArg adaptation failed",
            "Function.prototype.apply argument list must be array or arguments",
            "Object.prototype.toLocaleString called on null or undefined",
            "Object.prototype.toLocaleString target is not callable",
            "Array.prototype.concat receiver is not array",
            "Array.prototype.concat called on null or undefined",
            "Array.prototype.toLocaleString called on null or undefined",
            "Array.prototype.toLocaleString element method is not callable",
            "Array.prototype.flat receiver is not array",
            "Array.prototype.flat called on null or undefined",
            "Array.prototype.flat constructor is not object",
            "Array.prototype.flatMap receiver is not array",
            "Array.prototype.flatMap called on null or undefined",
            "Array.prototype.flatMap mapper is not callable",
            "Array.prototype.flatMap constructor is not object",
            "Array.prototype.flatMap cannot add property to non-extensible target",
            "Array.prototype.flatMap cannot define non-configurable target property",
            "Array.prototype.at called on null or undefined",
            "Array.prototype.includes receiver is not array",
            "Array.prototype.includes called on null or undefined",
            "Array.prototype.indexOf called on null or undefined",
            "Array.prototype.lastIndexOf called on null or undefined",
            "Array.prototype.find called on null or undefined",
            "Array.prototype.findIndex called on null or undefined",
            "Array.prototype.findLast called on null or undefined",
            "Array.prototype.findLastIndex called on null or undefined",
            "Array.prototype.find predicate is not callable",
            "Array.prototype.findIndex predicate is not callable",
            "Array.prototype.findLast predicate is not callable",
            "Array.prototype.findLastIndex predicate is not callable",
            "Array.prototype.reduce callback is not callable",
            "Array.prototype.reduceRight callback is not callable",
            "Reduce of empty array with no initial value",
            "First argument to String.prototype.endsWith must not be a RegExp",
            "First argument to String.prototype.includes must not be a RegExp",
            "String.prototype.match RegExp @@match is not callable",
            "Array.prototype.concat constructor is not object",
            "Array.prototype.concat cannot add property to non-extensible target",
            "Array.prototype.concat cannot define non-configurable target property",
            "Array.prototype.map receiver is not array",
            "Array.prototype.map called on null or undefined",
            "Array.prototype.map mapper is not callable",
            "Array.prototype.map constructor is not object",
            "Array.prototype.every receiver is not array",
            "Array.prototype.every called on null or undefined",
            "Array.prototype.every callback is not callable",
            "Array.prototype.every constructor is not object",
            "Array.prototype.some receiver is not array",
            "Array.prototype.some called on null or undefined",
            "Array.prototype.some callback is not callable",
            "Array.prototype.some constructor is not object",
            "Array.prototype.filter receiver is not array",
            "Array.prototype.filter called on null or undefined",
            "Array.prototype.filter callback is not callable",
            "Array.prototype.filter constructor is not object",
            "Array.prototype.filter cannot add property to non-extensible target",
            "Array.prototype.filter cannot define non-configurable target property",
            "Invalid array length",
            "Object.prototype.valueOf called on null or undefined",
            "Cannot convert undefined or null to object",
            "Object.setPrototypeOf target must be object",
            "Object.setPrototypeOf prototype must be object or null",
            "Object.setPrototypeOf returned false",
            "Object.hasOwn called on null or undefined",
            "Object.getOwnPropertyDescriptor called on null or undefined",
            "Object.getOwnPropertyNames called on null or undefined",
            "Object.getOwnPropertySymbols called on null or undefined",
            "Object.keys requires object",
            "Object.values requires object",
            "String.prototype method requires a String receiver",
            "Number.prototype method requires a Number receiver",
            "Boolean.prototype method requires a Boolean receiver",
            "Number.prototype.toString radix out of range",
            "BigInt.prototype.toString radix out of range",
            "Number.prototype.toFixed fraction digits out of range",
            "Number.prototype.toPrecision precision out of range",
            "Cannot convert a Symbol value to a number",
            "1000000000000000128",
            "4294967295",
            "Array.prototype.pop receiver is not array",
            "Array.prototype.push receiver is not array",
            "Array.prototype.push length exceeds safe integer",
            "Array.prototype.push length is not writable",
            "Cannot assign to read only property",
            "Cannot delete property",
            "Cannot add property to non-extensible object",
            "Array.prototype.push index write failed",
            "Array.prototype.shift receiver is not array",
            "Array.prototype.splice receiver is not array",
            "TypedArray accessor requires TypedArray",
            "Date constructor requires new",
            "Date method receiver is not Date",
            "RegExp.escape input must be a string",
            "Array.prototype.forEach receiver is not array",
            "Array.prototype.forEach called on null or undefined",
            "Array.prototype.forEach callback is not callable",
            "Cannot convert object to number",
            "Error.prototype.toString receiver is not object",
            "String.prototype HTML method receiver is null or undefined",
            "String.prototype method receiver is null or undefined",
            "repeat count must be non-negative and finite",
            "repeat result would exceed maximum string length",
            "First argument to String.prototype.startsWith must not be a RegExp",
            "String.prototype RegExp/string fallback is unsupported in wasm-aot",
            "String.prototype symbol hook is not callable",
            "AggregateError errors input must be array or arguments",
            "AggregateError errors input must be iterable",
            "AggregateError iterator method must return object",
            "AggregateError iterator next must be callable",
            "AggregateError iterator next result must be object",
            "assert.throws expected a throw",
            "assert.throws expected an error object",
            "assert.throws received the wrong error constructor",
            "assert.throws requires a function callback",
            "assert.throws callback is a class constructor",
            "target is not a constructor",
            "Reflect.construct target is not a constructor",
            "Reflect.construct newTarget is not a constructor",
            "Reflect.construct argumentsList must be an array",
            "Reflect.apply argumentsList must be an array",
            "Cannot convert object to primitive value",
            "ArrayBuffer byteLength getter requires ArrayBuffer",
            "ArrayBuffer detached getter requires ArrayBuffer",
            "ArrayBuffer slice receiver is not ArrayBuffer",
            "ArrayBuffer slice receiver is detached",
            "ArrayBuffer species constructor returned invalid ArrayBuffer",
            "ArrayBuffer constructor requires new",
            "ArrayBuffer maxByteLength getter requires ArrayBuffer",
            "ArrayBuffer resizable getter requires ArrayBuffer",
            "ArrayBuffer resize receiver is not resizable ArrayBuffer",
            "ArrayBuffer resize length is out of range",
            "ArrayBuffer resize is not supported by this host",
            "ArrayBuffer transfer receiver is not ArrayBuffer",
            "ArrayBuffer transfer receiver is detached",
            "ArrayBuffer transfer length is out of range",
            "ArrayBuffer receiver is immutable",
            "ArrayBuffer receiver is SharedArrayBuffer",
            "ArrayBuffer allocation size is too large",
            "SharedArrayBuffer getter requires SharedArrayBuffer",
            "SharedArrayBuffer grow receiver is not growable SharedArrayBuffer",
            "SharedArrayBuffer grow length is out of range",
            "detachArrayBuffer expects an ArrayBuffer",
            "DataView accessor requires DataView",
            "DataView backing buffer is detached",
            "DataView backing buffer is immutable",
            "DataView constructor requires new",
            "DataView constructor requires ArrayBuffer",
            "DataView byteOffset out of bounds",
            "DataView byteLength out of bounds",
            "DataView getUint8 index out of bounds",
            "DataView getUint16 index out of bounds",
            "DataView setUint16 index out of bounds",
            "DataView getUint32 index out of bounds",
            "DataView setUint32 index out of bounds",
            "DataView getFloat16 index out of bounds",
            "DataView setFloat16 index out of bounds",
            "DataView getFloat32 index out of bounds",
            "DataView setFloat32 index out of bounds",
            "DataView getFloat64 index out of bounds",
            "DataView setFloat64 index out of bounds",
            "DataView getBigInt64 index out of bounds",
            "DataView setBigInt64 index out of bounds",
            "cannot convert Number to BigInt",
            "cannot convert non-integer Number to BigInt",
            "cannot convert value to BigInt",
            "Cannot convert BigInt to number",
            "Cannot mix BigInt and other types",
            "BigInt exponent must be non-negative",
            "Do not know how to serialize a BigInt",
            "Converting circular structure to JSON",
            "Cannot convert Symbol to number",
            "Cannot convert a Symbol value to a string",
            "BigInt is not a constructor",
            "right-hand side of `in` is not an object",
            "Right-hand side of 'instanceof' is not callable",
            "must call super() before accessing `this`",
            "derived constructor must call super() before returning",
            "derived constructor may only return object or undefined",
            "super() called twice in derived constructor",
            "super() invalid in class extending null",
            "super property access on null base",
            "private field access on wrong object",
            GLOBAL_THIS_NAME,
            PRINT_NAME,
            IS_CONSTRUCTOR_NAME,
            "<a name=\"",
            "\">",
            "</a>",
            "<big>",
            "</big>",
            "<blink>",
            "</blink>",
            "<b>",
            "</b>",
            "<tt>",
            "</tt>",
            "<font color=\"",
            "<font size=\"",
            "</font>",
            "<i>",
            "</i>",
            "<a href=\"",
            "<small>",
            "</small>",
            "<strike>",
            "</strike>",
            "<sub>",
            "</sub>",
            "<sup>",
            "</sup>",
            "&quot;",
        ] {
            pool.intern_string(value);
        }
        for index in 0..=31 {
            pool.intern_string(&index.to_string());
        }
        for (_, _, value) in NUMBER_TO_PRECISION_CASES {
            pool.intern_string(value);
        }
        for binding in &script.global_bindings {
            pool.intern_string(&binding.name);
        }
        for meta in function_metas.values() {
            pool.intern_string(&meta.name);
            pool.intern_string(&meta.to_string_value);
        }
        for builtin in StandardBuiltinId::all_functions() {
            pool.intern_string(&format!(
                "standard builtin body is not emitted unless referenced directly: {}",
                builtin.debug_name()
            ));
        }
        for function in &script.functions {
            for param in &function.params {
                pool.intern_string(&param.name);
                if let Some(default_init) = &param.default_init {
                    pool.collect_expr(default_init);
                }
            }
            for binding in &function.owned_env_bindings {
                pool.intern_string(&binding.name);
            }
            for binding in &function.captured_bindings {
                pool.intern_string(&binding.name);
            }
            pool.collect_block(&function.body);
        }
        pool.collect_block(&script.body);
        pool.append_regexp_programs();
        pool
    }

    fn collect_block(&mut self, block: &BlockIr) {
        for statement in &block.statements {
            self.collect_statement(statement);
        }
    }

    fn collect_statement(&mut self, statement: &StatementIr) {
        match statement {
            StatementIr::Empty
            | StatementIr::Debugger
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. } => {}
            StatementIr::Lexical { name, init, .. } => {
                self.intern_string(name);
                self.collect_expr(init);
            }
            StatementIr::Expression(init) => self.collect_expr(init),
            StatementIr::Return(value) => self.collect_expr(value),
            StatementIr::Throw(value) => self.collect_expr(value),
            StatementIr::Var(declarators) => self.collect_var_declarators(declarators),
            StatementIr::LexicalBlock(statements) => {
                for statement in statements {
                    self.collect_statement(statement);
                }
            }
            StatementIr::Block(block) => self.collect_block(block),
            StatementIr::TryCatch {
                try_block,
                catch_block,
                catch_name,
                catch_source_name,
                ..
            } => {
                self.intern_string(catch_name);
                self.intern_string(catch_source_name);
                self.collect_block(try_block);
                self.collect_block(catch_block);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
            } => {
                self.collect_block(try_block);
                self.collect_block(finally_block);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                catch_name,
                catch_source_name,
                ..
            } => {
                self.intern_string(catch_name);
                self.intern_string(catch_source_name);
                self.collect_block(try_block);
                self.collect_block(catch_block);
                self.collect_block(finally_block);
            }
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_expr(condition);
                self.collect_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.collect_statement(else_branch);
                }
            }
            StatementIr::While { condition, body } => {
                self.collect_expr(condition);
                self.collect_statement(body);
            }
            StatementIr::DoWhile { body, condition } => {
                self.collect_statement(body);
                self.collect_expr(condition);
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
            } => {
                if let Some(init) = init {
                    self.collect_for_init(init);
                }
                if let Some(test) = test {
                    self.collect_expr(test);
                }
                if let Some(update) = update {
                    self.collect_expr(update);
                }
                self.collect_statement(body);
            }
            StatementIr::ForOfArray { iterable, body, .. }
            | StatementIr::ForOfString { iterable, body, .. }
            | StatementIr::ForOfIterator { iterable, body, .. }
            | StatementIr::ForInArray {
                target: iterable,
                body,
                ..
            }
            | StatementIr::ForInString {
                target: iterable,
                body,
                ..
            }
            | StatementIr::ForInObject {
                target: iterable,
                body,
                ..
            } => {
                self.collect_expr(iterable);
                self.collect_statement(body);
            }
            StatementIr::Switch {
                discriminant,
                cases,
            } => {
                self.collect_expr(discriminant);
                for case in cases {
                    if let Some(condition) = &case.condition {
                        self.collect_expr(condition);
                    }
                    self.collect_block(&case.body);
                }
            }
            StatementIr::Labelled { statement, .. } => self.collect_statement(statement),
        }
    }

    fn collect_for_init(&mut self, init: &ForInitIr) {
        match init {
            ForInitIr::Lexical { init, .. } => self.collect_expr(init),
            ForInitIr::LexicalBlock(bindings) => {
                for binding in bindings {
                    self.collect_expr(&binding.init);
                }
            }
            ForInitIr::Var(declarators) => self.collect_var_declarators(declarators),
            ForInitIr::Expression(expr) => self.collect_expr(expr),
        }
    }

    fn collect_var_declarators(&mut self, declarators: &[VarDeclaratorIr]) {
        for declarator in declarators {
            self.intern_string(&declarator.name);
            if let Some(init) = &declarator.init {
                self.collect_expr(init);
            }
        }
    }

    fn collect_expr(&mut self, expr: &TypedExpr) {
        match &expr.expr {
            ExprIr::Symbol { description } => {
                self.uses_heap = true;
                if let Some(description) = description {
                    self.collect_expr(description);
                }
            }
            ExprIr::String(value) => {
                self.intern_string(value);
            }
            ExprIr::RegExpLiteral {
                source,
                flags,
                program,
            } => {
                self.uses_heap = true;
                self.intern_string(source);
                self.intern_string(flags);
                if let Some(program) = program {
                    self.queue_regexp_program(program);
                }
            }
            ExprIr::BigInt(_) => {}
            ExprIr::ObjectLiteral(properties) => {
                self.uses_heap = true;
                for property in properties {
                    match property {
                        ObjectPropertyIr::PrototypeSetter { value } => {
                            self.collect_expr(value);
                        }
                        ObjectPropertyIr::Data { key, value, .. }
                        | ObjectPropertyIr::NonEnumerableData { key, value } => {
                            self.intern_string(key);
                            self.collect_expr(value);
                        }
                        ObjectPropertyIr::ComputedData { key, value } => {
                            self.collect_expr(key);
                            self.collect_expr(value);
                        }
                        ObjectPropertyIr::ComputedMethod { key, function }
                        | ObjectPropertyIr::ComputedGetter { key, function }
                        | ObjectPropertyIr::ComputedSetter { key, function } => {
                            self.collect_expr(key);
                            self.collect_expr(function);
                        }
                        ObjectPropertyIr::Method { key, function }
                        | ObjectPropertyIr::Getter { key, function }
                        | ObjectPropertyIr::Setter { key, function } => {
                            self.intern_string(key);
                            self.collect_expr(function);
                        }
                    }
                }
            }
            ExprIr::ArrayLiteral(elements) => {
                self.uses_heap = true;
                for element in elements {
                    self.collect_expr(element);
                }
            }
            ExprIr::PropertyRead { target, key } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
            }
            ExprIr::OptionalPropertyChain { target, chain } => {
                self.uses_heap = true;
                self.collect_expr(target);
                for operation in chain {
                    match operation {
                        OptionalChainOperationIr::Property { key, .. } => {
                            self.collect_property_key(key);
                        }
                        OptionalChainOperationIr::Call { args, .. } => {
                            for arg in args {
                                self.collect_expr(arg);
                            }
                        }
                    }
                }
            }
            ExprIr::PropertyWrite { target, key, value } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
                self.collect_expr(value);
            }
            ExprIr::PropertyUpdate { target, key, .. } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
            }
            ExprIr::AssignIdentifier { name, value } => {
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::CompoundAssignIdentifier { name, value, .. } => {
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::UnaryNumber { expr: value, .. }
            | ExprIr::LogicalNot { expr: value }
            | ExprIr::Void { expr: value }
            | ExprIr::DeleteValue { expr: value } => self.collect_expr(value),
            ExprIr::SpecOperation {
                operation,
                operands,
            } => {
                if matches!(operation, SpecOperationIr::ToIndex) {
                    self.intern_string("ToIndex out of range");
                }
                if matches!(operation, SpecOperationIr::CreateDataPropertyOrThrow) {
                    self.uses_heap = true;
                    self.intern_string("value");
                    self.intern_string("writable");
                    self.intern_string("enumerable");
                    self.intern_string("configurable");
                    self.intern_string(
                        "CreateDataPropertyOrThrow symbol property keys are not supported",
                    );
                    self.intern_string("CreateDataPropertyOrThrow target is not an object");
                    self.intern_string("Cannot redefine non-configurable property");
                    self.intern_string("Cannot define property on non-extensible object");
                }
                if matches!(operation, SpecOperationIr::Set) {
                    self.uses_heap = true;
                    self.intern_string("Set symbol property keys are not supported");
                    self.intern_string("Set target is not an object");
                }
                if matches!(operation, SpecOperationIr::HasOwnProperty) {
                    self.uses_heap = true;
                    self.intern_string("HasOwnProperty target is not an object");
                }
                if matches!(operation, SpecOperationIr::GetMethod) {
                    self.uses_heap = true;
                    self.intern_string("GetMethod target is not callable");
                }
                if matches!(operation, SpecOperationIr::Construct) {
                    self.uses_heap = true;
                    self.intern_string("target is not a constructor");
                    self.intern_string("Spread argument is not an array");
                }
                if matches!(operation, SpecOperationIr::DeletePropertyOrThrow) {
                    self.uses_heap = true;
                    self.intern_string(
                        "DeletePropertyOrThrow symbol property keys are not supported",
                    );
                    self.intern_string("DeletePropertyOrThrow target is not an object");
                    self.intern_string("Cannot delete property");
                }
                for operand in operands {
                    self.collect_expr(operand);
                }
            }
            ExprIr::DeleteIdentifier { name, .. } | ExprIr::DeleteGlobalProperty { name } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::DeleteProperty { target, key, .. } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
            }
            ExprIr::TypeOf { expr } => {
                self.intern_string("undefined");
                self.intern_string("object");
                self.intern_string("boolean");
                self.intern_string("number");
                self.intern_string("bigint");
                self.intern_string("symbol");
                self.intern_string("string");
                self.intern_string("function");
                self.collect_expr(expr);
            }
            ExprIr::TypeOfUnresolvedIdentifier { .. } => {
                self.intern_string("undefined");
            }
            ExprIr::StringFromCharCode { code } => {
                self.uses_heap = true;
                self.collect_expr(code);
            }
            ExprIr::StringCharCodeAt { target, index } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_expr(index);
            }
            ExprIr::NewTarget => {}
            ExprIr::UpdateIdentifier { name, .. } => self.intern_string(name),
            ExprIr::BinaryNumber { lhs, rhs, .. }
            | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
            | ExprIr::BitwiseNumber { lhs, rhs, .. }
            | ExprIr::CompareNumber { lhs, rhs, .. }
            | ExprIr::CompareValue { lhs, rhs, .. }
            | ExprIr::StrictEquality { lhs, rhs, .. }
            | ExprIr::LooseEquality { lhs, rhs, .. }
            | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
            | ExprIr::Comma { lhs, rhs } => {
                for value in [
                    "",
                    ",",
                    "[object Object]",
                    "[object Arguments]",
                    "valueOf",
                    "toString",
                ] {
                    self.intern_string(value);
                }
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.collect_expr(condition);
                self.collect_expr(then_expr);
                self.collect_expr(else_expr);
            }
            ExprIr::StringConcat { lhs, rhs } => {
                self.uses_heap = true;
                self.intern_string("undefined");
                self.intern_string("null");
                self.intern_string("true");
                self.intern_string("false");
                self.intern_string("NaN");
                self.intern_string("Infinity");
                self.intern_string("-Infinity");
                self.intern_string("[object Object]");
                self.intern_string("[object Arguments]");
                self.intern_string("");
                self.intern_string(",");
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::CoerciveAdd { lhs, rhs } => {
                self.uses_heap = true;
                for value in [
                    "",
                    ",",
                    "undefined",
                    "null",
                    "true",
                    "false",
                    "NaN",
                    "Infinity",
                    "-Infinity",
                    "[object Object]",
                    "[object Arguments]",
                    "valueOf",
                    "toString",
                ] {
                    self.intern_string(value);
                }
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::CallNamed { name, args } => {
                self.uses_heap = true;
                self.intern_string(name);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::SpreadArgument(value) => {
                self.uses_heap = true;
                self.intern_string("Spread argument is not an array");
                self.collect_expr(value);
            }
            ExprIr::AssertSameValue {
                actual,
                expected,
                message,
            } => {
                self.uses_heap = true;
                self.intern_string(ERROR_NAME);
                self.intern_string(message);
                self.collect_expr(actual);
                self.collect_expr(expected);
            }
            ExprIr::RuntimeThrow { name, message } => {
                self.uses_heap = true;
                self.intern_string(name);
                self.intern_string(message);
            }
            ExprIr::GlobalPropertyRead { name } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::GlobalPropertyWrite { name, value, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::GlobalPropertyUpdate { name, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::GlobalPropertyCompoundAssign { name, value, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
            } => {
                self.uses_heap = true;
                self.collect_expr(callee);
                if let Some(this_arg) = this_arg {
                    self.collect_expr(this_arg);
                }
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::JsonParseStaticReviver { value, reviver } => {
                self.uses_heap = true;
                self.collect_json_static_value(value);
                self.collect_expr(reviver);
                self.intern_string("");
                self.intern_string("source");
            }
            ExprIr::Construct { callee, args } => {
                self.uses_heap = true;
                self.intern_string("prototype");
                self.collect_expr(callee);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                self.uses_heap = true;
                self.collect_expr(receiver);
                self.collect_property_key(key);
                if matches!(key, PropertyKeyIr::StaticString(name) if name == "join")
                    || matches!(key, PropertyKeyIr::StaticString(name) if name == "toString")
                        && receiver
                            .possible_kinds
                            .is_subset_of(KindSet::from_kind(ValueKind::Array))
                {
                    self.intern_string("");
                    self.intern_string(",");
                    self.intern_string("Array.prototype.join receiver is not array");
                }
                if matches!(key, PropertyKeyIr::StaticString(name) if name == "reverse") {
                    self.intern_string("Array.prototype.reverse receiver is not array");
                }
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::InstanceOf { lhs, rhs } => {
                self.uses_heap = true;
                self.intern_string("prototype");
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::In { lhs, rhs } => {
                self.uses_heap = true;
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::SuperConstruct { args } => {
                self.uses_heap = true;
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::SuperPropertyRead { key } => {
                self.uses_heap = true;
                self.collect_property_key(key);
            }
            ExprIr::SuperPropertyWrite { key, value } => {
                self.uses_heap = true;
                self.collect_property_key(key);
                self.collect_expr(value);
            }
            ExprIr::ClassDefinition(_)
            | ExprIr::PrivateRead { .. }
            | ExprIr::PrivateWrite { .. }
            | ExprIr::PrivateIn { .. } => {
                self.uses_heap = true;
                if let ExprIr::ClassDefinition(class) = &expr.expr {
                    self.intern_string("prototype");
                    self.intern_string("constructor");
                    self.intern_string("$IsHTMLDDA");
                    self.intern_string("class extends value is not a constructor or null");
                    for method in &class.public_methods {
                        self.collect_property_key(&method.key);
                    }
                    for method in &class.private_methods {
                        self.intern_string(&private_data_key(method.private_name_id));
                        self.intern_string(&private_brand_key(method.private_name_id));
                    }
                    for field in &class.fields {
                        if let Some(key) = &field.key {
                            self.intern_string(key);
                        } else if let Some(private_name_id) = field.private_name_id {
                            self.intern_string(&private_data_key(private_name_id));
                            self.intern_string(&private_brand_key(private_name_id));
                        }
                    }
                    if let Some(heritage) = &class.heritage {
                        self.collect_expr(heritage);
                    }
                }
            }
            ExprIr::Undefined
            | ExprIr::ArrayHole
            | ExprIr::Null
            | ExprIr::Boolean(_)
            | ExprIr::Number(_)
            | ExprIr::FunctionValue(_)
            | ExprIr::This
            | ExprIr::Arguments
            | ExprIr::Identifier(_) => {}
        }
    }

    fn collect_property_key(&mut self, key: &PropertyKeyIr) {
        match key {
            PropertyKeyIr::StaticString(value) => self.intern_string(value),
            PropertyKeyIr::ArrayLength => {}
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                self.collect_expr(expr)
            }
        }
    }

    fn collect_json_static_value(&mut self, value: &JsonStaticValueIr) {
        match value {
            JsonStaticValueIr::Null { source }
            | JsonStaticValueIr::Boolean { source, .. }
            | JsonStaticValueIr::Number { source, .. } => {
                self.intern_string(source);
            }
            JsonStaticValueIr::String {
                value: string_value,
                source,
            } => {
                self.intern_string(string_value);
                self.intern_string(source);
            }
            JsonStaticValueIr::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    self.intern_string(&index.to_string());
                    self.collect_json_static_value(value);
                }
            }
            JsonStaticValueIr::Object(properties) => {
                for (key, value) in properties {
                    self.intern_string(key);
                    self.collect_json_static_value(value);
                }
            }
        }
    }

    fn intern_string(&mut self, value: &str) {
        if self.refs.contains_key(value) {
            return;
        }
        let offset = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        let bytes = Self::runtime_bytes_for_string(value);
        self.bytes.extend_from_slice(&bytes);
        self.refs.insert(
            value.to_string(),
            StringRef {
                offset,
                len: bytes.len() as u32,
            },
        );
    }

    fn queue_regexp_program(&mut self, program: &RegExpProgram) {
        let encoded = program.encode();
        if self.regexp_programs.contains_key(&encoded)
            || self
                .pending_regexp_programs
                .iter()
                .any(|(pending, _)| pending == &encoded)
        {
            return;
        }
        self.pending_regexp_programs
            .push((encoded, program.instructions.len() as u32));
    }

    fn append_regexp_programs(&mut self) {
        if self.pending_regexp_programs.is_empty() {
            return;
        }
        let padding = (8 - self.bytes.len() % 8) % 8;
        self.bytes.resize(self.bytes.len() + padding, 0);
        for (encoded, instruction_count) in self.pending_regexp_programs.drain(..) {
            let ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
            self.bytes.extend_from_slice(&encoded);
            self.regexp_programs.insert(
                encoded,
                RegExpProgramRef {
                    ptr,
                    instruction_count,
                },
            );
        }
    }

    pub(crate) fn runtime_bytes_for_string(value: &str) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(value.len());
        let mut chars = value.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch != JS_STRING_SURROGATE_SENTINEL {
                Self::push_utf8_char(&mut bytes, ch);
                continue;
            }

            if chars.peek().copied() == Some(JS_STRING_SURROGATE_SENTINEL) {
                chars.next();
                Self::push_utf8_char(&mut bytes, JS_STRING_SURROGATE_SENTINEL);
                continue;
            }

            let mut consumed = String::new();
            let mut code_unit = 0_u16;
            let mut is_marker = true;
            for _ in 0..4 {
                let Some(hex) = chars.next() else {
                    is_marker = false;
                    break;
                };
                consumed.push(hex);
                let Some(value) = hex.to_digit(16) else {
                    is_marker = false;
                    break;
                };
                code_unit = (code_unit << 4) | value as u16;
            }

            if is_marker && (0xD800..=0xDFFF).contains(&code_unit) {
                Self::push_wtf8_code_unit(&mut bytes, code_unit);
            } else {
                Self::push_utf8_char(&mut bytes, JS_STRING_SURROGATE_SENTINEL);
                bytes.extend_from_slice(consumed.as_bytes());
            }
        }
        bytes
    }

    fn push_utf8_char(bytes: &mut Vec<u8>, ch: char) {
        let mut buffer = [0; 4];
        bytes.extend_from_slice(ch.encode_utf8(&mut buffer).as_bytes());
    }

    fn push_wtf8_code_unit(bytes: &mut Vec<u8>, code_unit: u16) {
        let code = code_unit as u32;
        if code < 0x80 {
            bytes.push(code as u8);
        } else if code < 0x800 {
            bytes.push((0xC0 | (code >> 6)) as u8);
            bytes.push((0x80 | (code & 0x3F)) as u8);
        } else {
            bytes.push((0xE0 | (code >> 12)) as u8);
            bytes.push((0x80 | ((code >> 6) & 0x3F)) as u8);
            bytes.push((0x80 | (code & 0x3F)) as u8);
        }
    }

    pub(crate) fn payload(&self, value: &str) -> i64 {
        let string = self
            .refs
            .get(value)
            .unwrap_or_else(|| panic!("string `{value}` must exist in pool"));
        (((string.offset as u64) << 32) | string.len as u64) as i64
    }

    pub(crate) fn regexp_program(&self, program: &RegExpProgram) -> RegExpProgramRef {
        *self
            .regexp_programs
            .get(&program.encode())
            .expect("collected RegExp literal program must have static data")
    }
}

pub(crate) fn align_heap_start(bytes: usize) -> u64 {
    ((STATIC_DATA_OFFSET as u64 + bytes as u64) + 7) & !7
}

pub(crate) fn initial_memory_pages(static_data_bytes: usize, uses_heap: bool) -> u64 {
    let required = if uses_heap {
        align_heap_start(static_data_bytes) + (WASM_PAGE_SIZE * 16)
    } else {
        STATIC_DATA_OFFSET as u64 + static_data_bytes as u64
    };
    required.div_ceil(WASM_PAGE_SIZE).max(1)
}
