use super::*;
use icu_normalizer::{
    properties::{CanonicalCombiningClassMapBorrowed, CanonicalCompositionBorrowed},
    properties::{CanonicalDecompositionBorrowed, Decomposed},
    DecomposingNormalizerBorrowed,
};
use icu_properties::{props, CodePointSetData};
use porffor_ir::{
    OptionalChainOperationIr, RegExpProgram, StaticRegExpCompilation, TemplateObjectIr,
    BUILTIN_REGEXP_FUNCTION_ID, BUILTIN_REGEXP_PROTOTYPE_COMPILE_FUNCTION_ID, REGEXP_OPCODE_ACCEPT,
    REGEXP_OPCODE_DOT, REGEXP_OPCODE_JUMP, REGEXP_OPCODE_LITERAL_ASCII,
    REGEXP_OPCODE_LITERAL_CODE_POINT, REGEXP_OPCODE_NEGATIVE_ASCII_CLASS,
    REGEXP_OPCODE_NOT_WHITESPACE, REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
    REGEXP_OPCODE_POSITIVE_ASCII_CLASS, REGEXP_OPCODE_SPLIT, REGEXP_OPCODE_UNICODE_PROPERTY,
    REGEXP_OPCODE_WHITESPACE,
};
use std::sync::OnceLock;

/// Packed magic/version word at the start of an immutable named-group table.
/// The high 32 bits are the format version and the low 32 bits are `NRGT`.
pub(crate) const REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION: u64 =
    (1_u64 << 32) | u32::from_le_bytes(*b"NRGT") as u64;

#[derive(Debug)]
struct StringRef {
    offset: u32,
    len: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RegExpProgramRef {
    pub(crate) ptr: u32,
    pub(crate) instruction_count: u32,
    pub(crate) capture_count: u32,
    pub(crate) split_count: u32,
    pub(crate) repeatable_split_count: u32,
    pub(crate) named_group_table_ptr: u32,
}

/// Semantic identity for immutable static RegExp programs. The bytecode blob
/// contains instructions only, so capture metadata must participate here
/// instead of being appended to the static program bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RegExpProgramStaticKey {
    encoded_instructions: Vec<u8>,
    capture_count: u32,
    named_groups: Vec<(String, Vec<u32>)>,
}

struct LowercaseTables {
    mappings: Vec<u8>,
    mapping_count: u32,
    cased_ranges: Vec<std::ops::RangeInclusive<u32>>,
    case_ignorable_ranges: Vec<std::ops::RangeInclusive<u32>>,
}

static LOWERCASE_TABLES: OnceLock<LowercaseTables> = OnceLock::new();

struct UppercaseTables {
    mappings: Vec<u8>,
    mapping_count: u32,
}

static UPPERCASE_TABLES: OnceLock<UppercaseTables> = OnceLock::new();

struct NormalizationMapping {
    codepoint: u32,
    sequence_index: u32,
    sequence_len: u32,
}

struct NormalizationTables {
    canonical_mappings: Vec<NormalizationMapping>,
    canonical_sequences: Vec<u32>,
    compatibility_mappings: Vec<NormalizationMapping>,
    compatibility_sequences: Vec<u32>,
    combining_classes: Vec<(u32, u8)>,
    compositions: Vec<(u32, u32, u32)>,
}

static NORMALIZATION_TABLES: OnceLock<NormalizationTables> = OnceLock::new();

impl RegExpProgramStaticKey {
    pub(crate) fn from_program(program: &RegExpProgram) -> Self {
        Self {
            encoded_instructions: program.encode(),
            capture_count: program.capture_count,
            named_groups: program
                .named_groups
                .iter()
                .map(|group| (group.name.clone(), group.capture_ids.clone()))
                .collect(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct StringPool {
    pub(crate) bytes: Vec<u8>,
    pub(crate) template_objects: BTreeMap<u64, TemplateObjectIr>,
    refs: BTreeMap<String, StringRef>,
    script_string_literals: BTreeSet<String>,
    runtime_regexp_candidate_literals: BTreeSet<String>,
    regexp_programs: BTreeMap<RegExpProgramStaticKey, RegExpProgramRef>,
    pending_regexp_programs: Vec<(RegExpProgramStaticKey, u32, u32, u32)>,
    runtime_regexp_programs: Vec<(String, String, RegExpProgramRef)>,
    needs_runtime_regexp_programs: bool,
    pub(crate) runtime_regexp_program_table_ptr: u32,
    pub(crate) runtime_regexp_program_count: u32,
    pub(crate) uses_heap: bool,
    pub(crate) lowercase_mapping_table_ptr: u32,
    pub(crate) lowercase_mapping_count: u32,
    pub(crate) uppercase_mapping_table_ptr: u32,
    pub(crate) uppercase_mapping_count: u32,
    pub(crate) cased_range_table_ptr: u32,
    pub(crate) cased_range_count: u32,
    pub(crate) case_ignorable_range_table_ptr: u32,
    pub(crate) case_ignorable_range_count: u32,
    pub(crate) canonical_decomposition_table_ptr: u32,
    pub(crate) canonical_decomposition_count: u32,
    pub(crate) compatibility_decomposition_table_ptr: u32,
    pub(crate) compatibility_decomposition_count: u32,
    pub(crate) combining_class_table_ptr: u32,
    pub(crate) combining_class_count: u32,
    pub(crate) composition_table_ptr: u32,
    pub(crate) composition_count: u32,
}

impl StringPool {
    pub(crate) fn collect(
        script: &ScriptIr,
        function_metas: &BTreeMap<FunctionId, WasmFunctionMeta>,
        compiled_standard_builtins: &[StandardBuiltinId],
    ) -> Self {
        let mut pool = Self::default();
        pool.needs_runtime_regexp_programs = script.functions.iter().any(|function| {
            function.super_constructor_target.as_deref() == Some(BUILTIN_REGEXP_FUNCTION_ID)
        }) || compiled_standard_builtins
            .contains(&StandardBuiltinId::RegExpPrototypeSymbolSplit);
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
            "-0",
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
            "/",
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
            "$",
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
            "withResolvers",
            "try",
            "promise",
            "resolve",
            "reject",
            "callee",
            "arguments",
            "caller",
            "valueOf",
            "toString",
            "toUpperCase",
            "toLowerCase",
            "toLocaleLowerCase",
            "toLocaleUpperCase",
            "fromCodePoint",
            "String.fromCodePoint argument must be an integer from 0 through 0x10FFFF",
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
            "unshift",
            "splice",
            "sort",
            "subarray",
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
            "__proto__",
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
            "destructuring value is not iterable",
            "destructuring iterator method must return object",
            "destructuring iterator next must be callable",
            "destructuring iterator next result must be object",
            "Cannot destructure undefined or null",
            "return",
            "IteratorClose return method must be callable",
            "IteratorClose return result must be object",
            "$ArrayIterator.array",
            "$ArrayIterator.index",
            "$ArrayIterator.done",
            "$ArrayIterator.kind",
            "$StringIterator.string",
            "$StringIterator.index",
            PORFFOR_STATIC_GENERATOR_VALUES_METHOD,
            PORFFOR_STATIC_GENERATOR_ITERATOR_SLOT,
            "$RegExpStringIterator.regexp",
            "$RegExpStringIterator.string",
            "$RegExpStringIterator.global",
            "$RegExpStringIterator.unicode",
            "$RegExpStringIterator.done",
            "Array Iterator",
            "String Iterator",
            "Map Iterator",
            "Set Iterator",
            "Generator",
            "Generator method called on incompatible receiver",
            "Generator is already running",
            "AsyncGenerator method called on incompatible receiver",
            "Promise.resolve receiver is not an object",
            "Promise keyed constructor resolve property is not callable",
            "Promise keyed input must be an object",
            "allKeyed",
            "allSettledKeyed",
            "status",
            "fulfilled",
            "rejected",
            "value",
            "reason",
            "Array.prototype iterator method called on null or undefined",
            "Array Iterator next called on incompatible receiver",
            "Array Iterator next called on out-of-bounds TypedArray",
            "String Iterator next called on incompatible receiver",
            "Map Iterator.prototype.next receiver does not have [[Map]]",
            "Map Iterator.prototype.next receiver is not an object",
            "Map.groupBy items cannot be null or undefined",
            "Map.groupBy callback must be callable",
            "Map.groupBy iterator method must be callable",
            "Map.groupBy iterator method must return an object",
            "Map.groupBy iterator next method must be callable",
            "Map.groupBy iterator produced too many values",
            "Map.groupBy iterator next result must be an object",
            "Map.prototype.getOrInsertComputed callback must be callable",
            "Map method receiver does not have [[MapData]]",
            "Map method receiver is not an object",
            "Object.groupBy items cannot be null or undefined",
            "Object.groupBy callback must be callable",
            "Object.groupBy iterator method must be callable",
            "Object.groupBy iterator method must return an object",
            "Object.groupBy iterator next method must be callable",
            "Object.groupBy iterator produced too many values",
            "Object.groupBy iterator next result must be an object",
            "Set Iterator.prototype.next receiver does not have [[Set]]",
            "Set Iterator.prototype.next receiver is not an object",
            "Set method receiver does not have [[SetData]]",
            "Set method receiver is not an object",
            "Set method argument is not a set-like object",
            "Set-like size is NaN",
            "Set-like size is negative",
            "Set-like has method is not callable",
            "Set-like keys method is not callable",
            "Set-like keys method must return an object",
            "Set-like iterator next method is not callable",
            "Set-like iterator next result must be an object",
            "RegExp String Iterator next called on incompatible receiver",
            "RegExp String Iterator exec returned non-object",
            "Iterator",
            "toArray",
            "forEach",
            "groupBy",
            "every",
            "some",
            "find",
            "reduce",
            "map",
            "filter",
            "take",
            "drop",
            "Iterator constructor cannot be called",
            "Iterator Helper",
            "Iterator helper called on incompatible receiver",
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
            "$IteratorMapIterator",
            "$IteratorMapNext",
            "$IteratorMapMapper",
            "$IteratorMapIndex",
            "$IteratorMapDone",
            "$IteratorMapExecuting",
            "Iterator.zip called with a non-object iterables value",
            "Iterator.zip iterator method must be callable",
            "Iterator.zip iterator method must return object",
            "Iterator.zip next method must be callable",
            "Iterator.zip next result must be object",
            "Iterator.zip options must be an object or undefined",
            "Iterator.zip mode must be a string or undefined",
            "Iterator.zip mode must be shortest, longest, or strict",
            "Iterator.zip padding must be an object or undefined",
            "Iterator.zip strict mode has iterators of different lengths",
            "Iterator zip helper next called on incompatible receiver",
            "Iterator zip helper is already running",
            "Iterator zip helper next result must be object",
            "Iterator zip helper return called on incompatible receiver",
            "$IteratorZipIterators",
            "$IteratorZipNextMethods",
            "$IteratorZipOpen",
            "$IteratorZipMode",
            "$IteratorZipPadding",
            "$IteratorZipDone",
            "$IteratorZipExecuting",
            "$IteratorZipStarted",
            "mode",
            "padding",
            "shortest",
            "longest",
            "strict",
            "Iterator.prototype.filter called on null or undefined",
            "Iterator.prototype.filter predicate must be callable",
            "Iterator.prototype.filter next method must be callable",
            "Iterator filter helper next called on incompatible receiver",
            "Iterator filter helper is already running",
            "Iterator filter helper next result must be object",
            "Iterator filter helper return called on incompatible receiver",
            "Iterator filter helper return method must be callable",
            "Iterator filter helper return result must be object",
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
            "Array.fromAsync mapper is not callable",
            "Array.fromAsync input is null or undefined",
            "Array.fromAsync iterator method is not callable",
            "Array.fromAsync iterator method must return object",
            "Array.fromAsync iterator next is not callable",
            "Array.fromAsync iterator next result must be object",
            "Array.fromAsync iterator produced too many values",
            "Array.of index property is non-configurable",
            "Array.of target is not extensible",
            "TypedArray.from receiver is not a constructor",
            "TypedArray.from mapper is not callable",
            "TypedArray.from iterator next must be callable",
            "TypedArray.from iterator next result must be object",
            "TypedArray.from constructed target is not a typed array",
            "TypedArray.from constructed target is too small",
            "TypedArray.of receiver is not a constructor",
            "TypedArray constructor requires new",
            "TypedArray byteOffset out of range",
            "TypedArray byteOffset must be aligned",
            "TypedArray byteLength out of range",
            "TypedArray byteLength must be aligned",
            "TypedArray backing buffer is detached",
            "TypedArray byteLength out of bounds",
            "TypedArray length out of range",
            "TypedArray iterator method requires a TypedArray",
            "TypedArray find method requires a TypedArray",
            "TypedArray.prototype.includes requires a TypedArray",
            "TypedArray.prototype.indexOf requires a TypedArray",
            "TypedArray.prototype.lastIndexOf requires a TypedArray",
            "TypedArray.prototype.slice requires a TypedArray",
            "TypedArray.prototype.slice has unknown element type",
            "TypedArray.prototype.slice constructor property is not an object",
            "TypedArray.prototype.slice species is not a constructor",
            "TypedArray.prototype.slice species content type differs",
            "TypedArray.prototype.find predicate is not callable",
            "TypedArray.prototype.findIndex predicate is not callable",
            "TypedArray.prototype.findLast predicate is not callable",
            "TypedArray.prototype.findLastIndex predicate is not callable",
            "TypedArray every method requires a TypedArray",
            "TypedArray some method requires a TypedArray",
            "TypedArray.prototype.every callback is not callable",
            "TypedArray.prototype.some callback is not callable",
            "TypedArray.prototype.map requires a TypedArray",
            "TypedArray.prototype.map callback is not callable",
            "TypedArray.prototype.map has unknown element type",
            "TypedArray.prototype.map constructor property is not an object",
            "TypedArray.prototype.map species is not a constructor",
            "TypedArray.prototype.map species content type differs",
            "TypedArray.prototype.filter requires a TypedArray",
            "TypedArray.prototype.filter callback is not callable",
            "TypedArray.prototype.filter has unknown element type",
            "TypedArray.prototype.filter constructor property is not an object",
            "TypedArray.prototype.filter species is not a constructor",
            "TypedArray.prototype.filter species content type differs",
            "TypedArray.prototype.forEach requires a TypedArray",
            "TypedArray.prototype.forEach callback is not callable",
            "TypedArray.prototype.reduce requires a TypedArray",
            "TypedArray.prototype.reduce callback is not callable",
            "TypedArray.prototype.reduceRight requires a TypedArray",
            "TypedArray.prototype.reduceRight callback is not callable",
            "Reduce of empty typed array with no initial value",
            "TypedArray.prototype.toString requires TypedArray",
            "TypedArray.prototype.join requires a TypedArray",
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
            "GeneratorFunction",
            "function GeneratorFunction() { [native code] }",
            "AsyncFunction",
            "function AsyncFunction() { [native code] }",
            "AsyncGenerator",
            "AsyncGeneratorFunction",
            "function AsyncGeneratorFunction() { [native code] }",
            "[Symbol.asyncIterator]",
            "function [Symbol.asyncIterator]() { [native code] }",
            "function next() { [native code] }",
            "function return() { [native code] }",
            "function throw() { [native code] }",
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
            "RegExp.prototype.test receiver is not an object",
            "RegExp.prototype.test exec result is not an object or null",
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
            "RegExp.prototype[Symbol.split] receiver must be an object",
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
            "clear",
            "delete",
            "size",
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
            "Cannot define invalid TypedArray index",
            "Cannot define incompatible TypedArray index descriptor",
            "TypedArray.prototype.subarray requires TypedArray",
            "TypedArray.prototype.subarray has unknown element type",
            "TypedArray.prototype.subarray constructor property is not an object",
            "TypedArray.prototype.subarray species is not a constructor",
            "TypedArray.prototype.subarray species did not return a TypedArray",
            "TypedArray.prototype.subarray species content type differs",
            "Constructed target is not a typed array",
            "Constructed typed array is too small",
            "TypedArray.prototype.set requires TypedArray",
            "TypedArray.prototype.set backing buffer is detached",
            "TypedArray.prototype.set offset is out of range",
            "TypedArray.prototype.set source buffer is detached",
            "TypedArray.prototype.set source and target content types differ",
            "TypedArray.prototype.set source is too large",
            "TypedArray.prototype.reverse requires TypedArray",
            "TypedArray.prototype.sort requires TypedArray",
            "TypedArray.prototype.toReversed requires TypedArray",
            "TypedArray.prototype.toReversed has unknown element type",
            "TypedArray.prototype.toSorted requires TypedArray",
            "TypedArray.prototype.toSorted has unknown element type",
            "TypedArray.prototype.with requires TypedArray",
            "TypedArray.prototype.with index out of range",
            "TypedArray.prototype.with has unknown element type",
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
            "TypedArray.prototype.toLocaleString element method is not callable",
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
            "TypedArray.prototype.at called on incompatible receiver",
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
            "Array species constructor is not a constructor",
            "Cannot add property to non-extensible target",
            "Cannot define non-configurable target property",
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
            "Array.prototype.with index out of range",
            "Array.prototype.toSpliced result exceeds maximum safe length",
            "Array.prototype.concat result exceeds maximum safe length",
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
            "Array.prototype.unshift called on null or undefined",
            "Array.prototype.unshift cannot modify a string",
            "Array.prototype.unshift length exceeds safe integer",
            "Array.prototype.unshift cannot delete destination property",
            "Array.prototype.shift called on null or undefined",
            "Array.prototype.shift cannot modify a string",
            "Array.prototype.shift cannot delete property",
            "Array.prototype.splice receiver is not array",
            "Array.prototype.sort receiver is not array",
            "TypedArray accessor requires TypedArray",
            "Date constructor requires new",
            "Date method receiver is not Date",
            "RegExp.escape input must be a string",
            "RegExp.prototype.compile receiver is not a direct RegExp instance",
            "RegExp.prototype.compile flags must be undefined when pattern is RegExp",
            "invalid regular-expression flag",
            "duplicate regular-expression flag",
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
            "SharedArrayBuffer grow length is smaller than its current byte length",
            "detachArrayBuffer expects an ArrayBuffer",
            "detachArrayBuffer key does not match the ArrayBuffer detach key",
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
            "DataView getBigUint64 index out of bounds",
            "DataView setBigInt64 index out of bounds",
            "DataView setBigUint64 index out of bounds",
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
            pool.intern_string(meta.runtime_name());
            pool.intern_string(&meta.to_string_value);
        }
        for builtin in StandardBuiltinId::all_functions() {
            pool.intern_string(&format!(
                "standard builtin body is not emitted unless referenced directly: {}",
                builtin.debug_name()
            ));
        }
        for function in &script.functions {
            if let Some(plan) = &function.class_instance_element_plan {
                for private_name_id in &plan.private_method_brands {
                    pool.intern_string(&private_brand_key(*private_name_id));
                }
                for field in &plan.fields {
                    match &field.key {
                        ClassFieldKeyIr::Public(key) => pool.intern_string(key),
                        ClassFieldKeyIr::ComputedPublic(_) => {}
                        ClassFieldKeyIr::Private(private_name_id) => {
                            pool.intern_string(&private_data_key(*private_name_id));
                            pool.intern_string(&private_brand_key(*private_name_id));
                        }
                    }
                }
            }
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
        if compiled_standard_builtins.iter().any(|builtin| {
            matches!(
                builtin,
                StandardBuiltinId::StringPrototypeToLowerCase
                    | StandardBuiltinId::StringPrototypeToLocaleLowerCase
            )
        }) {
            pool.intern_string("ς");
        }
        if pool.needs_runtime_regexp_programs {
            pool.queue_runtime_regexp_programs();
        }
        pool.append_regexp_programs();
        if compiled_standard_builtins.iter().any(|builtin| {
            matches!(
                builtin,
                StandardBuiltinId::StringPrototypeToLowerCase
                    | StandardBuiltinId::StringPrototypeToLocaleLowerCase
            )
        }) {
            pool.append_lowercase_tables();
        }
        if compiled_standard_builtins.iter().any(|builtin| {
            matches!(
                builtin,
                StandardBuiltinId::StringPrototypeToUpperCase
                    | StandardBuiltinId::StringPrototypeToLocaleUpperCase
            )
        }) {
            pool.append_uppercase_tables();
        }
        if compiled_standard_builtins.contains(&StandardBuiltinId::StringPrototypeNormalize)
            || compiled_standard_builtins.contains(&StandardBuiltinId::StringPrototypeLocaleCompare)
        {
            for form in ["NFC", "NFD", "NFKC", "NFKD"] {
                pool.intern_string(form);
            }
            pool.intern_string("String.prototype.normalize receiver is null or undefined");
            pool.intern_string("String.prototype.normalize form must be NFC, NFD, NFKC, or NFKD");
            pool.append_normalization_tables();
        }
        pool
    }

    fn append_normalization_tables(&mut self) {
        let tables = NORMALIZATION_TABLES.get_or_init(|| {
            let nfd = DecomposingNormalizerBorrowed::new_nfd();
            let nfkd = DecomposingNormalizerBorrowed::new_nfkd();
            let combining_classes = CanonicalCombiningClassMapBorrowed::new();
            let compositions = CanonicalCompositionBorrowed::new();
            let canonical_decomposition = CanonicalDecompositionBorrowed::new();
            let mut tables = NormalizationTables {
                canonical_mappings: Vec::new(),
                canonical_sequences: Vec::new(),
                compatibility_mappings: Vec::new(),
                compatibility_sequences: Vec::new(),
                combining_classes: Vec::new(),
                compositions: Vec::new(),
            };

            for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
                let source = codepoint.to_string();
                let canonical: Vec<u32> = nfd.normalize(&source).chars().map(u32::from).collect();
                if canonical.as_slice() != [u32::from(codepoint)] {
                    tables.canonical_mappings.push(NormalizationMapping {
                        codepoint: u32::from(codepoint),
                        sequence_index: tables.canonical_sequences.len() as u32,
                        sequence_len: canonical.len() as u32,
                    });
                    tables.canonical_sequences.extend(canonical);
                }

                let compatibility: Vec<u32> =
                    nfkd.normalize(&source).chars().map(u32::from).collect();
                if compatibility.as_slice() != [u32::from(codepoint)] {
                    tables.compatibility_mappings.push(NormalizationMapping {
                        codepoint: u32::from(codepoint),
                        sequence_index: tables.compatibility_sequences.len() as u32,
                        sequence_len: compatibility.len() as u32,
                    });
                    tables.compatibility_sequences.extend(compatibility);
                }

                let combining_class = combining_classes.get_u8(codepoint);
                if combining_class != 0 {
                    tables
                        .combining_classes
                        .push((u32::from(codepoint), combining_class));
                }

                if let Decomposed::Expansion(first, second) =
                    canonical_decomposition.decompose(codepoint)
                {
                    if let Some(composed) = compositions.compose(first, second) {
                        tables.compositions.push((
                            u32::from(first),
                            u32::from(second),
                            u32::from(composed),
                        ));
                    }
                }
            }
            tables.compositions.sort_unstable();
            tables.compositions.dedup();
            tables
        });

        let canonical_sequences_ptr = self.append_codepoints(&tables.canonical_sequences);
        self.canonical_decomposition_table_ptr =
            self.append_normalization_mappings(&tables.canonical_mappings, canonical_sequences_ptr);
        self.canonical_decomposition_count = tables.canonical_mappings.len() as u32;
        let compatibility_sequences_ptr = self.append_codepoints(&tables.compatibility_sequences);
        self.compatibility_decomposition_table_ptr = self.append_normalization_mappings(
            &tables.compatibility_mappings,
            compatibility_sequences_ptr,
        );
        self.compatibility_decomposition_count = tables.compatibility_mappings.len() as u32;

        self.align_bytes(8);
        self.combining_class_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for (codepoint, combining_class) in &tables.combining_classes {
            self.bytes.extend_from_slice(&codepoint.to_le_bytes());
            self.bytes
                .extend_from_slice(&u32::from(*combining_class).to_le_bytes());
        }
        self.combining_class_count = tables.combining_classes.len() as u32;

        self.align_bytes(8);
        self.composition_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for (first, second, composed) in &tables.compositions {
            self.bytes.extend_from_slice(&first.to_le_bytes());
            self.bytes.extend_from_slice(&second.to_le_bytes());
            self.bytes.extend_from_slice(&composed.to_le_bytes());
            self.bytes.extend_from_slice(&0_u32.to_le_bytes());
        }
        self.composition_count = tables.compositions.len() as u32;
    }

    fn append_codepoints(&mut self, codepoints: &[u32]) -> u32 {
        self.align_bytes(8);
        let table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for codepoint in codepoints {
            self.bytes.extend_from_slice(&codepoint.to_le_bytes());
        }
        table_ptr
    }

    fn append_normalization_mappings(
        &mut self,
        mappings: &[NormalizationMapping],
        sequences_ptr: u32,
    ) -> u32 {
        self.align_bytes(8);
        let table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for mapping in mappings {
            self.bytes
                .extend_from_slice(&mapping.codepoint.to_le_bytes());
            self.bytes
                .extend_from_slice(&(sequences_ptr + mapping.sequence_index * 4).to_le_bytes());
            self.bytes
                .extend_from_slice(&mapping.sequence_len.to_le_bytes());
            self.bytes.extend_from_slice(&0_u32.to_le_bytes());
        }
        table_ptr
    }

    fn append_lowercase_tables(&mut self) {
        let tables = LOWERCASE_TABLES.get_or_init(|| {
            let mut mappings = Vec::new();
            let mut mapping_count = 0;
            for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
                let mut lowercase = codepoint.to_lowercase();
                let first = lowercase.next().expect("lowercase mapping is never empty");
                let second = lowercase.next();
                if first == codepoint && second.is_none() {
                    continue;
                }

                let mut lowercase_bytes = Vec::with_capacity(4);
                for lowercase_codepoint in std::iter::once(first).chain(second).chain(lowercase) {
                    let mut encoded = [0; 4];
                    lowercase_bytes.extend_from_slice(
                        lowercase_codepoint.encode_utf8(&mut encoded).as_bytes(),
                    );
                }
                debug_assert!(lowercase_bytes.len() <= 4);
                mappings.extend_from_slice(&(codepoint as u32).to_le_bytes());
                mappings.extend_from_slice(&(lowercase_bytes.len() as u32).to_le_bytes());
                mappings.extend_from_slice(&lowercase_bytes);
                mappings.resize(mappings.len() + 4 - lowercase_bytes.len(), 0);
                mappings.extend_from_slice(&0_u32.to_le_bytes());
                mapping_count += 1;
            }

            LowercaseTables {
                mappings,
                mapping_count,
                cased_ranges: CodePointSetData::new::<props::Cased>()
                    .iter_ranges()
                    .collect(),
                case_ignorable_ranges: CodePointSetData::new::<props::CaseIgnorable>()
                    .iter_ranges()
                    .collect(),
            }
        });

        self.align_bytes(8);
        self.lowercase_mapping_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        self.bytes.extend_from_slice(&tables.mappings);
        self.lowercase_mapping_count = tables.mapping_count;
        self.cased_range_table_ptr = self.append_codepoint_ranges(&tables.cased_ranges);
        self.cased_range_count = tables.cased_ranges.len() as u32;
        self.case_ignorable_range_table_ptr =
            self.append_codepoint_ranges(&tables.case_ignorable_ranges);
        self.case_ignorable_range_count = tables.case_ignorable_ranges.len() as u32;
    }

    fn append_uppercase_tables(&mut self) {
        let tables = UPPERCASE_TABLES.get_or_init(|| {
            let mut mappings = Vec::new();
            let mut mapping_count = 0;
            for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
                let mut uppercase = codepoint.to_uppercase();
                let first = uppercase.next().expect("uppercase mapping is never empty");
                let second = uppercase.next();
                if first == codepoint && second.is_none() {
                    continue;
                }

                let mut uppercase_bytes = Vec::with_capacity(8);
                for uppercase_codepoint in std::iter::once(first).chain(second).chain(uppercase) {
                    let mut encoded = [0; 4];
                    uppercase_bytes.extend_from_slice(
                        uppercase_codepoint.encode_utf8(&mut encoded).as_bytes(),
                    );
                }
                debug_assert!(uppercase_bytes.len() <= 8);
                mappings.extend_from_slice(&(codepoint as u32).to_le_bytes());
                mappings.extend_from_slice(&(uppercase_bytes.len() as u32).to_le_bytes());
                mappings.extend_from_slice(&uppercase_bytes);
                mappings.resize(mappings.len() + 8 - uppercase_bytes.len(), 0);
                mapping_count += 1;
            }

            UppercaseTables {
                mappings,
                mapping_count,
            }
        });

        self.align_bytes(8);
        self.uppercase_mapping_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        self.bytes.extend_from_slice(&tables.mappings);
        self.uppercase_mapping_count = tables.mapping_count;
    }

    fn append_codepoint_ranges(&mut self, ranges: &[std::ops::RangeInclusive<u32>]) -> u32 {
        self.align_bytes(8);
        let table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for range in ranges {
            self.bytes.extend_from_slice(&range.start().to_le_bytes());
            self.bytes.extend_from_slice(&range.end().to_le_bytes());
        }
        table_ptr
    }

    fn align_bytes(&mut self, alignment: usize) {
        let padding = (alignment - self.bytes.len() % alignment) % alignment;
        self.bytes.resize(self.bytes.len() + padding, 0);
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
                collect_finite_string_choices(init, &mut self.runtime_regexp_candidate_literals);
                self.collect_expr(init);
            }
            StatementIr::AnnexBFunctionCopy {
                source_name,
                block_storage_name,
                variable_storage_name,
            } => {
                self.intern_string(source_name);
                self.intern_string(block_storage_name);
                self.intern_string(variable_storage_name);
            }
            StatementIr::Expression(init) => self.collect_expr(init),
            StatementIr::GeneratorYield {
                value,
                delegate,
                resume_mode,
                ..
            } => {
                if *delegate {
                    for key in [
                        "Symbol.iterator",
                        "next",
                        "done",
                        "value",
                        "return",
                        "throw",
                    ] {
                        self.intern_string(key);
                    }
                }
                if let GeneratorResumeModeIr::AssignProperty { target, key } = resume_mode {
                    self.collect_expr(target);
                    self.collect_property_key(key);
                }
                self.collect_expr(value);
            }
            StatementIr::AsyncAwait { value, .. } => self.collect_expr(value),
            StatementIr::Return(value) => self.collect_expr(value),
            StatementIr::Throw(value) => self.collect_expr(value),
            StatementIr::Var(declarators) => self.collect_var_declarators(declarators),
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => {
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
                ..
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
                ..
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
            StatementIr::GeneratorLoop {
                init,
                test,
                update,
                before_yield,
                yield_statement,
                after_yield,
                ..
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
                for statement in before_yield {
                    self.collect_statement(statement);
                }
                self.collect_statement(yield_statement);
                for statement in after_yield {
                    self.collect_statement(statement);
                }
            }
            StatementIr::GeneratorIf {
                condition,
                then_before_yield,
                then_yield_statement,
                then_after_yield,
                else_before_yield,
                else_yield_statement,
                else_after_yield,
                ..
            } => {
                self.collect_expr(condition);
                for statement in then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                {
                    self.collect_statement(statement);
                }
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
                lexical_declarations,
                cases,
                ..
            } => {
                self.collect_expr(discriminant);
                for declaration in lexical_declarations {
                    self.collect_statement(declaration);
                }
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
            ForInitIr::Lexical { init, .. } => {
                collect_finite_string_choices(init, &mut self.runtime_regexp_candidate_literals);
                self.collect_expr(init);
            }
            ForInitIr::LexicalBlock(bindings) => {
                for binding in bindings {
                    collect_finite_string_choices(
                        &binding.init,
                        &mut self.runtime_regexp_candidate_literals,
                    );
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
                collect_finite_string_choices(init, &mut self.runtime_regexp_candidate_literals);
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
                self.script_string_literals.insert(value.clone());
            }
            ExprIr::TemplateObject(template) => {
                self.uses_heap = true;
                for cooked in template.cooked.iter().flatten() {
                    self.intern_string(cooked);
                }
                for raw in &template.raw {
                    self.intern_string(raw);
                }
                self.intern_string("raw");
                if let Some(previous) = self
                    .template_objects
                    .insert(template.site_id, template.clone())
                {
                    debug_assert_eq!(previous, *template);
                }
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
                    collect_finite_string_choices(
                        element,
                        &mut self.runtime_regexp_candidate_literals,
                    );
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
                        OptionalChainOperationIr::PrivateProperty { .. } => {}
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
            ExprIr::PropertyCompoundAssign {
                target, key, value, ..
            } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
                self.collect_expr(value);
            }
            ExprIr::AssignIdentifier { name, value } => {
                self.intern_string(name);
                collect_finite_string_choices(value, &mut self.runtime_regexp_candidate_literals);
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
                if matches!(operation, SpecOperationIr::CopyDataProperties) {
                    self.uses_heap = true;
                    self.intern_string("enumerable");
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
            ExprIr::MaterializeBinding { name, value, body } => {
                self.intern_string(name);
                self.collect_expr(value);
                self.collect_expr(body);
            }
            ExprIr::ArrayDestructure { value, pattern, .. } => {
                self.uses_heap = true;
                for key in ["Symbol.iterator", "next", "done", "value", "return"] {
                    self.intern_string(key);
                }
                self.collect_expr(value);
                pattern.visit_expressions(&mut |expr| self.collect_expr(expr));
                fn collect_static_keys(
                    collector: &mut StringPool,
                    pattern: &ArrayDestructuringPatternIr,
                ) {
                    for element in &pattern.elements {
                        let target = match element {
                            ArrayDestructuringElementIr::Elision => continue,
                            ArrayDestructuringElementIr::Target { target, .. }
                            | ArrayDestructuringElementIr::Rest { target } => target,
                        };
                        match target {
                            DestructuringTargetIr::AssignmentProperty {
                                key: DestructuringPropertyKeyIr::Static(key),
                                ..
                            } => collector.intern_string(key),
                            DestructuringTargetIr::NestedArray(pattern) => {
                                collect_static_keys(collector, pattern)
                            }
                            DestructuringTargetIr::NestedObject(pattern) => {
                                for property in &pattern.properties {
                                    if let DestructuringPropertyKeyIr::Static(key) = &property.key {
                                        collector.intern_string(key);
                                    }
                                    collect_target_static_keys(collector, &property.target);
                                }
                                if let Some(rest) = &pattern.rest {
                                    collect_target_static_keys(collector, rest);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                fn collect_target_static_keys(
                    collector: &mut StringPool,
                    target: &DestructuringTargetIr,
                ) {
                    match target {
                        DestructuringTargetIr::AssignmentProperty {
                            key: DestructuringPropertyKeyIr::Static(key),
                            ..
                        } => collector.intern_string(key),
                        DestructuringTargetIr::NestedArray(pattern) => {
                            collect_static_keys(collector, pattern)
                        }
                        DestructuringTargetIr::NestedObject(pattern) => {
                            for property in &pattern.properties {
                                if let DestructuringPropertyKeyIr::Static(key) = &property.key {
                                    collector.intern_string(key);
                                }
                                collect_target_static_keys(collector, &property.target);
                            }
                            if let Some(rest) = &pattern.rest {
                                collect_target_static_keys(collector, rest);
                            }
                        }
                        _ => {}
                    }
                }
                collect_static_keys(self, pattern);
            }
            ExprIr::ObjectDestructure { value, pattern } => {
                self.uses_heap = true;
                self.intern_string("enumerable");
                self.collect_expr(value);
                pattern.visit_expressions(&mut |expr| self.collect_expr(expr));

                fn collect_target_static_keys(
                    collector: &mut StringPool,
                    target: &DestructuringTargetIr,
                ) {
                    match target {
                        DestructuringTargetIr::AssignmentProperty {
                            key: DestructuringPropertyKeyIr::Static(key),
                            ..
                        } => collector.intern_string(key),
                        DestructuringTargetIr::NestedArray(pattern) => {
                            for element in &pattern.elements {
                                match element {
                                    ArrayDestructuringElementIr::Elision => {}
                                    ArrayDestructuringElementIr::Target { target, .. }
                                    | ArrayDestructuringElementIr::Rest { target } => {
                                        collect_target_static_keys(collector, target);
                                    }
                                }
                            }
                        }
                        DestructuringTargetIr::NestedObject(pattern) => {
                            for property in &pattern.properties {
                                if let DestructuringPropertyKeyIr::Static(key) = &property.key {
                                    collector.intern_string(key);
                                }
                                collect_target_static_keys(collector, &property.target);
                            }
                            if let Some(rest) = &pattern.rest {
                                collect_target_static_keys(collector, rest);
                            }
                        }
                        _ => {}
                    }
                }

                for property in &pattern.properties {
                    if let DestructuringPropertyKeyIr::Static(key) = &property.key {
                        self.intern_string(key);
                    }
                    collect_target_static_keys(self, &property.target);
                }
                if let Some(rest) = &pattern.rest {
                    collect_target_static_keys(self, rest);
                }
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
                for message in [
                    "Spread argument is not iterable",
                    "Spread iterator method must return object",
                    "Spread iterator next must be callable",
                    "Spread iterator next result must be object",
                ] {
                    self.intern_string(message);
                }
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
                static_regexp_compilation,
            } => {
                self.uses_heap = true;
                if static_regexp_compilation.is_none()
                    && (matches!(callee.expr, ExprIr::GlobalPropertyRead { ref name } if name == "RegExp")
                        || callee.function_targets.iter().any(|target| {
                            matches!(
                                target.as_str(),
                                BUILTIN_REGEXP_FUNCTION_ID
                                    | BUILTIN_REGEXP_PROTOTYPE_COMPILE_FUNCTION_ID
                            )
                        }))
                {
                    self.needs_runtime_regexp_programs = true;
                }
                if let Some(compilation) = static_regexp_compilation {
                    match compilation {
                        StaticRegExpCompilation::Program(program) => {
                            self.queue_regexp_program(program)
                        }
                        StaticRegExpCompilation::InvalidSyntax { message } => {
                            self.intern_string(message);
                        }
                    }
                }
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
            ExprIr::Construct {
                callee,
                args,
                static_regexp_compilation,
            } => {
                self.uses_heap = true;
                if static_regexp_compilation.is_none()
                    && (matches!(callee.expr, ExprIr::GlobalPropertyRead { ref name } if name == "RegExp")
                        || callee.function_targets.contains(BUILTIN_REGEXP_FUNCTION_ID))
                {
                    self.needs_runtime_regexp_programs = true;
                }
                self.intern_string("prototype");
                if let Some(compilation) = static_regexp_compilation {
                    match compilation {
                        StaticRegExpCompilation::Program(program) => {
                            self.queue_regexp_program(program)
                        }
                        StaticRegExpCompilation::InvalidSyntax { message } => {
                            self.intern_string(message);
                        }
                    }
                }
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
            ExprIr::ClassDefinition(class) => {
                self.uses_heap = true;
                self.intern_string("prototype");
                self.intern_string("constructor");
                self.intern_string("$IsHTMLDDA");
                self.intern_string("class extends value is not a constructor or null");
                for definition in &class.element_plan.definitions {
                    match definition {
                        ClassElementDefinitionIr::PublicMethod(method) => {
                            self.collect_property_key(&method.key);
                        }
                        ClassElementDefinitionIr::PrivateMethod(method) => {
                            self.intern_string(&private_data_key(method.private_name_id));
                            self.intern_string(&private_brand_key(method.private_name_id));
                        }
                        ClassElementDefinitionIr::ComputedFieldKey { key, .. } => {
                            self.collect_property_key(key);
                        }
                    }
                }
                for static_element in &class.element_plan.static_elements {
                    let ClassStaticElementIr::Field(field) = static_element else {
                        continue;
                    };
                    match &field.key {
                        ClassFieldKeyIr::Public(key) => self.intern_string(key),
                        ClassFieldKeyIr::ComputedPublic(_) => {}
                        ClassFieldKeyIr::Private(private_name_id) => {
                            self.intern_string(&private_data_key(*private_name_id));
                            self.intern_string(&private_brand_key(*private_name_id));
                        }
                    }
                }
                if let Some(heritage) = &class.heritage {
                    self.collect_expr(heritage);
                }
            }
            ExprIr::PrivateRead { target, .. } => {
                self.uses_heap = true;
                self.collect_expr(target);
            }
            ExprIr::PrivateWrite { target, value, .. } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_expr(value);
            }
            ExprIr::PrivateIn { rhs, .. } => {
                self.uses_heap = true;
                self.collect_expr(rhs);
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
        assert!(
            !has_non_consuming_cycle(program),
            "compiler-created RegExp program contains a non-consuming control-flow cycle"
        );
        // Names are part of the immutable metadata table and must be present
        // in the string pool before any program blobs are appended.
        for group in &program.named_groups {
            self.intern_string(&group.name);
        }
        let key = RegExpProgramStaticKey::from_program(program);
        if self.regexp_programs.contains_key(&key)
            || self
                .pending_regexp_programs
                .iter()
                .any(|(pending, _, _, _)| pending == &key)
        {
            return;
        }
        let split_count = program
            .instructions
            .iter()
            .filter(|instruction| instruction.opcode == REGEXP_OPCODE_SPLIT)
            .count() as u32;
        self.pending_regexp_programs.push((
            key,
            program.instructions.len() as u32,
            split_count,
            repeatable_split_count(program),
        ));
    }

    fn queue_runtime_regexp_programs(&mut self) {
        let candidate_literals = if self.runtime_regexp_candidate_literals.is_empty() {
            &self.script_string_literals
        } else {
            &self.runtime_regexp_candidate_literals
        };
        let mut literals = candidate_literals.iter().cloned().collect::<Vec<_>>();
        if !literals.iter().any(|source| source == "(?:)") {
            literals.push("(?:)".to_string());
        }
        if !literals.iter().any(|source| source == "[object Object]") {
            literals.push("[object Object]".to_string());
        }
        let mut flags = self
            .script_string_literals
            .iter()
            .filter(|value| is_regexp_flags_literal(value))
            .cloned()
            .collect::<Vec<_>>();
        if !flags.iter().any(String::is_empty) {
            flags.push(String::new());
        }
        let sticky_flags = flags
            .iter()
            .filter(|flags| !flags.contains('y'))
            .map(|flags| format!("{flags}y"))
            .collect::<Vec<_>>();
        flags.extend(sticky_flags);
        for literal in &literals {
            self.intern_string(literal);
        }
        for flags in &flags {
            self.intern_string(flags);
        }
        let mut candidates = Vec::new();

        for source in &literals {
            let normalized_source = if source.is_empty() { "(?:)" } else { source };
            let compilation_source = if normalized_source == "(?:)" {
                ""
            } else {
                normalized_source
            };
            for flags in &flags {
                let Ok(program) = RegExpProgram::compile(compilation_source, flags) else {
                    continue;
                };
                let key = RegExpProgramStaticKey::from_program(&program);
                self.queue_regexp_program(&program);
                candidates.push((normalized_source.to_string(), flags.clone(), key));
            }
        }

        self.append_regexp_programs();
        self.runtime_regexp_programs = candidates
            .into_iter()
            .map(|(source, flags, key)| {
                let program = *self
                    .regexp_programs
                    .get(&key)
                    .expect("queued runtime RegExp program must have static data");
                (source, flags, program)
            })
            .collect();
        self.append_runtime_regexp_program_table();
    }

    fn append_runtime_regexp_program_table(&mut self) {
        if self.runtime_regexp_programs.is_empty() {
            return;
        }
        let padding = (8 - self.bytes.len() % 8) % 8;
        self.bytes.resize(self.bytes.len() + padding, 0);
        self.runtime_regexp_program_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        self.runtime_regexp_program_count = self.runtime_regexp_programs.len() as u32;
        for (source, flags, program) in &self.runtime_regexp_programs {
            for value in [
                self.payload(source) as u64,
                self.payload(flags) as u64,
                program.ptr as u64,
                program.instruction_count as u64,
                program.capture_count as u64,
                program.split_count as u64,
                program.repeatable_split_count as u64,
                program.named_group_table_ptr as u64,
            ] {
                self.bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    fn append_regexp_programs(&mut self) {
        if self.pending_regexp_programs.is_empty() {
            return;
        }
        let padding = (8 - self.bytes.len() % 8) % 8;
        self.bytes.resize(self.bytes.len() + padding, 0);
        let pending = std::mem::take(&mut self.pending_regexp_programs);
        for (key, instruction_count, split_count, repeatable_split_count) in pending {
            let ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
            let capture_count = key.capture_count;
            self.bytes.extend_from_slice(&key.encoded_instructions);
            let named_group_table_ptr = self.append_named_group_table(&key.named_groups);
            self.regexp_programs.insert(
                key,
                RegExpProgramRef {
                    ptr,
                    instruction_count,
                    capture_count,
                    split_count,
                    repeatable_split_count,
                    named_group_table_ptr,
                },
            );
        }
    }

    fn append_named_group_table(&mut self, named_groups: &[(String, Vec<u32>)]) -> u32 {
        if named_groups.is_empty() {
            return 0;
        }

        let padding = (8 - self.bytes.len() % 8) % 8;
        self.bytes.resize(self.bytes.len() + padding, 0);
        let table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        let records_ptr = table_ptr + 32;
        let total_candidate_count: usize = named_groups
            .iter()
            .map(|(_, candidates)| candidates.len())
            .sum();
        let candidates_base = records_ptr + (named_groups.len() as u32 * 24);

        for value in [
            REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION,
            named_groups.len() as u64,
            total_candidate_count as u64,
            records_ptr as u64,
        ] {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }

        let mut candidate_offset = 0_u32;
        for (name, candidates) in named_groups {
            let name_payload = self.payload(name) as u64;
            let candidates_ptr = candidates_base + candidate_offset;
            for value in [name_payload, candidates_ptr as u64, candidates.len() as u64] {
                self.bytes.extend_from_slice(&value.to_le_bytes());
            }
            candidate_offset += (candidates.len() * 8) as u32;
        }
        for (_, candidates) in named_groups {
            for &capture_id in candidates {
                self.bytes
                    .extend_from_slice(&(capture_id as u64).to_le_bytes());
            }
        }
        table_ptr
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
            .get(&RegExpProgramStaticKey::from_program(program))
            .expect("collected RegExp literal program must have static data")
    }

    #[cfg(test)]
    pub(crate) fn collect_regexp_program_for_test(
        &mut self,
        program: &RegExpProgram,
    ) -> RegExpProgramRef {
        self.queue_regexp_program(program);
        self.append_regexp_programs();
        self.regexp_program(program)
    }
}

fn collect_finite_string_choices(expr: &TypedExpr, choices: &mut BTreeSet<String>) {
    match &expr.expr {
        ExprIr::String(value) => {
            choices.insert(value.clone());
        }
        ExprIr::ArrayLiteral(elements) => {
            for element in elements {
                collect_finite_string_choices(element, choices);
            }
        }
        ExprIr::ObjectLiteral(properties) => {
            for property in properties {
                match property {
                    ObjectPropertyIr::PrototypeSetter { value }
                    | ObjectPropertyIr::Data { value, .. }
                    | ObjectPropertyIr::NonEnumerableData { value, .. }
                    | ObjectPropertyIr::ComputedData { value, .. } => {
                        collect_finite_string_choices(value, choices);
                    }
                    ObjectPropertyIr::ComputedMethod { .. }
                    | ObjectPropertyIr::ComputedGetter { .. }
                    | ObjectPropertyIr::ComputedSetter { .. }
                    | ObjectPropertyIr::Method { .. }
                    | ObjectPropertyIr::Getter { .. }
                    | ObjectPropertyIr::Setter { .. } => {}
                }
            }
        }
        ExprIr::Conditional {
            then_expr,
            else_expr,
            ..
        } => {
            collect_finite_string_choices(then_expr, choices);
            collect_finite_string_choices(else_expr, choices);
        }
        _ => {}
    }
}

fn is_regexp_flags_literal(value: &str) -> bool {
    let mut seen = BTreeSet::new();
    value.chars().all(|flag| {
        matches!(flag, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y') && seen.insert(flag)
    }) && !(seen.contains(&'u') && seen.contains(&'v'))
}

/// Counts `Split`s that can execute again through a control-flow cycle.
///
/// RegExp programs are capped at 4096 instructions, so a small, precise DFS
/// per split is clearer than maintaining a separate SCC representation here.
fn repeatable_split_count(program: &RegExpProgram) -> u32 {
    let instructions = &program.instructions;
    let successors = |pc: usize| -> Vec<usize> {
        let Some(instruction) = instructions.get(pc) else {
            return Vec::new();
        };
        let valid = |target: u64| {
            usize::try_from(target)
                .ok()
                .filter(|target| *target < instructions.len())
        };
        match instruction.opcode {
            REGEXP_OPCODE_SPLIT => [valid(instruction.operand0), valid(instruction.operand1)]
                .into_iter()
                .flatten()
                .collect(),
            REGEXP_OPCODE_JUMP => valid(instruction.operand0).into_iter().collect(),
            REGEXP_OPCODE_ACCEPT => Vec::new(),
            _ if pc + 1 < instructions.len() => vec![pc + 1],
            _ => Vec::new(),
        }
    };

    instructions
        .iter()
        .enumerate()
        .filter(|(_, instruction)| instruction.opcode == REGEXP_OPCODE_SPLIT)
        .filter(|(split_pc, _)| {
            let mut visited = vec![false; instructions.len()];
            let mut stack = successors(*split_pc);
            while let Some(pc) = stack.pop() {
                if pc == *split_pc {
                    return true;
                }
                if visited[pc] {
                    continue;
                }
                visited[pc] = true;
                stack.extend(successors(pc));
            }
            false
        })
        .count() as u32
}

fn has_non_consuming_cycle(program: &RegExpProgram) -> bool {
    fn is_consuming(instruction: &porffor_ir::RegExpInstruction) -> bool {
        matches!(
            instruction.opcode,
            REGEXP_OPCODE_LITERAL_ASCII
                | REGEXP_OPCODE_LITERAL_CODE_POINT
                | REGEXP_OPCODE_NEGATIVE_ASCII_CLASS
                | REGEXP_OPCODE_NOT_WHITESPACE
                | REGEXP_OPCODE_POSITIVE_ASCII_CLASS
                | REGEXP_OPCODE_WHITESPACE
                | REGEXP_OPCODE_DOT
                | REGEXP_OPCODE_UNICODE_PROPERTY
        ) || (instruction.opcode == REGEXP_OPCODE_NUMBERED_BACKREFERENCE
            && instruction.operand1 != 0)
    }

    fn visit(pc: usize, instructions: &[porffor_ir::RegExpInstruction], state: &mut [u8]) -> bool {
        if state[pc] == 1 {
            return true;
        }
        if state[pc] == 2 || is_consuming(&instructions[pc]) {
            return false;
        }
        state[pc] = 1;
        let instruction = instructions[pc];
        let valid_target = |target: u64| {
            usize::try_from(target)
                .ok()
                .filter(|target| *target < instructions.len())
        };
        let successors = match instruction.opcode {
            REGEXP_OPCODE_SPLIT => [
                valid_target(instruction.operand0),
                valid_target(instruction.operand1),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
            REGEXP_OPCODE_JUMP => valid_target(instruction.operand0).into_iter().collect(),
            REGEXP_OPCODE_ACCEPT => Vec::new(),
            _ if pc + 1 < instructions.len() => vec![pc + 1],
            _ => Vec::new(),
        };
        if successors
            .into_iter()
            .any(|successor| visit(successor, instructions, state))
        {
            return true;
        }
        state[pc] = 2;
        false
    }

    let mut state = vec![0; program.instructions.len()];
    (0..program.instructions.len()).any(|pc| visit(pc, &program.instructions, &mut state))
}

#[cfg(test)]
mod regexp_program_validation_tests {
    use super::*;
    use porffor_ir::{RegExpFlags, RegExpInstruction};

    fn program(instructions: Vec<RegExpInstruction>) -> RegExpProgram {
        RegExpProgram {
            flags: RegExpFlags::default(),
            capture_count: 0,
            named_groups: Vec::new(),
            instructions,
        }
    }

    #[test]
    fn static_program_named_group_table_is_serialized_and_no_names_use_zero() {
        let unnamed = RegExpProgram::compile("a", "").expect("program should compile");
        let named = RegExpProgram::compile("(?<x>a)(?<y>b)", "").expect("program should compile");
        let mut pool = StringPool::default();
        let unnamed_ref = pool.collect_regexp_program_for_test(&unnamed);
        let named_ref = pool.collect_regexp_program_for_test(&named);

        assert_eq!(unnamed_ref.named_group_table_ptr, 0);
        assert_ne!(named_ref.named_group_table_ptr, 0);
        let offset = (named_ref.named_group_table_ptr - STATIC_DATA_OFFSET) as usize;
        let read = |at: usize| u64::from_le_bytes(pool.bytes[at..at + 8].try_into().unwrap());
        assert_eq!(read(offset), REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION);
        assert_eq!(read(offset + 8), 2);
        assert_eq!(read(offset + 16), 2);
        assert_eq!(
            read(offset + 24),
            (named_ref.named_group_table_ptr + 32) as u64
        );
        let records = offset + 32;
        assert_eq!(read(records + 16), 1);
        assert_eq!(read(records + 40), 1);
        let candidates = (read(records + 8) - STATIC_DATA_OFFSET as u64) as usize;
        assert_eq!(read(candidates), 1);
        assert_eq!(read(candidates + 8), 2);
        assert_eq!(read(records), pool.payload("x") as u64);
        assert_eq!(read(records + 24), pool.payload("y") as u64);
    }

    #[test]
    fn static_program_dedup_key_includes_named_group_mappings() {
        let base = RegExpProgram::compile("a", "").expect("program should compile");
        let mut first = base.clone();
        first.named_groups.push(porffor_ir::RegExpNamedGroup {
            name: "x".into(),
            capture_ids: vec![1],
        });
        let mut second = first.clone();
        second.named_groups[0].capture_ids = vec![2];
        let mut pool = StringPool::default();
        let first_ref = pool.collect_regexp_program_for_test(&first);
        let second_ref = pool.collect_regexp_program_for_test(&second);
        assert_ne!(first_ref.ptr, second_ref.ptr);
    }

    #[test]
    fn rejects_non_consuming_program_cycles_without_rejecting_valid_repetition() {
        let self_jump = program(vec![RegExpInstruction::jump(0)]);
        assert!(has_non_consuming_cycle(&self_jump));

        let failed_consuming_loop = program(vec![
            RegExpInstruction::split(1, 2),
            RegExpInstruction::literal_ascii(b'a'),
            RegExpInstruction::jump(0),
        ]);
        assert!(has_non_consuming_cycle(&failed_consuming_loop));

        let valid_star = RegExpProgram::compile("a*", "").expect("star should compile");
        assert!(!has_non_consuming_cycle(&valid_star));
        let valid_lazy_star = RegExpProgram::compile("a*?b", "").expect("lazy star should compile");
        assert!(!has_non_consuming_cycle(&valid_lazy_star));
    }

    #[test]
    fn counts_repeatable_splits_inside_lookbehind() {
        let program =
            RegExpProgram::compile(r"(?<=\w+)f", "").expect("lookbehind repetition should compile");
        assert_eq!(repeatable_split_count(&program), 1);
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
