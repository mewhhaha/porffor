use super::*;

pub(crate) const RESULT_TAG_EXPORT: &str = "result_tag";
pub(crate) const COMPLETION_KIND_EXPORT: &str = "completion_kind";
pub(crate) const COMPLETION_AUX_EXPORT: &str = "completion_aux";
pub(crate) const THROW_ERROR_NAME_EXPORT: &str = "throw_error_name";

pub(crate) const HOST_IMPORT_MODULE: &str = "porf_host";
pub(crate) const HOST_IMPORT_PRINT_LINE_UTF8: &str = "print_line_utf8";

pub(crate) const RESULT_TAG_GLOBAL_INDEX: u32 = 0;
pub(crate) const COMPLETION_KIND_GLOBAL_INDEX: u32 = 1;
pub(crate) const COMPLETION_AUX_GLOBAL_INDEX: u32 = 2;
pub(crate) const HEAP_PTR_GLOBAL_INDEX: u32 = 3;
pub(crate) const SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX: u32 = 4;
pub(crate) const OBJECT_PROTOTYPE_GLOBAL_INDEX: u32 = 5;
pub(crate) const FUNCTION_PROTOTYPE_GLOBAL_INDEX: u32 = 6;
pub(crate) const ARRAY_PROTOTYPE_GLOBAL_INDEX: u32 = 7;
pub(crate) const NUMBER_PROTOTYPE_GLOBAL_INDEX: u32 = 8;
pub(crate) const STRING_PROTOTYPE_GLOBAL_INDEX: u32 = 9;
pub(crate) const BOOLEAN_PROTOTYPE_GLOBAL_INDEX: u32 = 10;
pub(crate) const ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 11;
pub(crate) const TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 12;
pub(crate) const REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 13;
pub(crate) const EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 14;
pub(crate) const RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 15;
pub(crate) const SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 16;
pub(crate) const URI_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 17;
pub(crate) const AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 18;
pub(crate) const FUNCTION_CONSTRUCTOR_GLOBAL_INDEX: u32 = 19;
pub(crate) const OBJECT_CONSTRUCTOR_GLOBAL_INDEX: u32 = 20;
pub(crate) const ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 21;
pub(crate) const NUMBER_CONSTRUCTOR_GLOBAL_INDEX: u32 = 22;
pub(crate) const STRING_CONSTRUCTOR_GLOBAL_INDEX: u32 = 23;
pub(crate) const BOOLEAN_CONSTRUCTOR_GLOBAL_INDEX: u32 = 24;
pub(crate) const ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 25;
pub(crate) const TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 26;
pub(crate) const REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 27;
pub(crate) const EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 28;
pub(crate) const AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 29;
pub(crate) const RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 30;
pub(crate) const SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 31;
pub(crate) const URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 32;
pub(crate) const ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX: u32 = 33;
pub(crate) const SHARED_ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX: u32 = 34;
pub(crate) const DATA_VIEW_PROTOTYPE_GLOBAL_INDEX: u32 = 35;
pub(crate) const TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX: u32 = 36;
pub(crate) const TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 37;
pub(crate) const ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX: u32 = 38;
pub(crate) const SHARED_ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX: u32 = 39;
pub(crate) const DATA_VIEW_CONSTRUCTOR_GLOBAL_INDEX: u32 = 40;
pub(crate) const REFLECT_OBJECT_GLOBAL_INDEX: u32 = 41;
pub(crate) const FLOAT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 42;
pub(crate) const FLOAT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 43;
pub(crate) const INT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 44;
pub(crate) const INT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 45;
pub(crate) const INT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 46;
pub(crate) const UINT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 47;
pub(crate) const UINT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 48;
pub(crate) const UINT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 49;
pub(crate) const UINT8_CLAMPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 50;
pub(crate) const BIGINT_CONSTRUCTOR_GLOBAL_INDEX: u32 = 51;
pub(crate) const PROXY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 52;
pub(crate) const MATH_OBJECT_GLOBAL_INDEX: u32 = 53;
pub(crate) const DATE_PROTOTYPE_GLOBAL_INDEX: u32 = 54;
pub(crate) const DATE_CONSTRUCTOR_GLOBAL_INDEX: u32 = 55;
pub(crate) const REGEXP_PROTOTYPE_GLOBAL_INDEX: u32 = 56;
pub(crate) const REGEXP_CONSTRUCTOR_GLOBAL_INDEX: u32 = 57;
pub(crate) const REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX: u32 = 58;
pub(crate) const JSON_OBJECT_GLOBAL_INDEX: u32 = 59;
pub(crate) const ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX: u32 = 60;
pub(crate) const ITERATOR_PROTOTYPE_GLOBAL_INDEX: u32 = 61;
pub(crate) const ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX: u32 = 62;
pub(crate) const ITERATOR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 63;
pub(crate) const THROW_ERROR_NAME_HEAP_GLOBAL_INDEX: u32 = 64;
pub(crate) const SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX: u32 = 65;
pub(crate) const SUPPRESSED_ERROR_CONSTRUCTOR_GLOBAL_INDEX: u32 = 66;
pub(crate) const THROW_TYPE_ERROR_GLOBAL_INDEX: u32 = 67;
pub(crate) const BIGINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 68;
pub(crate) const BIGUINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 69;
pub(crate) const REGEXP_PROTOTYPE_SYMBOL_SEARCH_GLOBAL_INDEX: u32 = 70;
pub(crate) const REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_GLOBAL_INDEX: u32 = 71;
pub(crate) const ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX: u32 = 72;
pub(crate) const CURRENT_REALM_GLOBAL_INDEX: u32 = 73;
pub(crate) const ATOMICS_OBJECT_GLOBAL_INDEX: u32 = 74;
pub(crate) const SYMBOL_CONSTRUCTOR_GLOBAL_INDEX: u32 = 75;
pub(crate) const SYMBOL_PROTOTYPE_GLOBAL_INDEX: u32 = 76;
pub(crate) const SYMBOL_REGISTRY_GLOBAL_INDEX: u32 = 77;

pub(crate) const THROW_ERROR_NAME_NO_HEAP_GLOBAL_INDEX: u32 = HEAP_PTR_GLOBAL_INDEX;
pub(crate) const JS_FUNCTION_TYPE_INDEX: u32 = 1;
pub(crate) const HEAP_ALLOC_TYPE_INDEX: u32 = 2;
pub(crate) const OBJECT_APPEND_DATA_PROPERTY_TYPE_INDEX: u32 = 3;
pub(crate) const OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX: u32 = 4;
pub(crate) const FUNCTION_OBJECT_ALLOC_TYPE_INDEX: u32 = 5;
pub(crate) const PLAIN_OBJECT_ALLOC_TYPE_INDEX: u32 = 6;
pub(crate) const ARRAY_ALLOC_TYPE_INDEX: u32 = 7;
pub(crate) const HOST_PRINT_IMPORT_TYPE_INDEX: u32 = 8;
pub(crate) const HOST_PRINT_IMPORT_FUNCTION_INDEX: u32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlobalIndexSlot {
    pub name: &'static str,
    pub index: u32,
}

pub(crate) const GLOBAL_INDEX_REGISTRY: &[GlobalIndexSlot] = &[
    GlobalIndexSlot {
        name: "result_tag",
        index: RESULT_TAG_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "completion_kind",
        index: COMPLETION_KIND_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "completion_aux",
        index: COMPLETION_AUX_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "heap_ptr",
        index: HEAP_PTR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "script_global_object",
        index: SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Object.prototype",
        index: OBJECT_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Function.prototype",
        index: FUNCTION_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Array.prototype",
        index: ARRAY_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Number.prototype",
        index: NUMBER_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "String.prototype",
        index: STRING_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Boolean.prototype",
        index: BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Error.prototype",
        index: ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "TypeError.prototype",
        index: TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "ReferenceError.prototype",
        index: REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "EvalError.prototype",
        index: EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "RangeError.prototype",
        index: RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "SyntaxError.prototype",
        index: SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "URIError.prototype",
        index: URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "AggregateError.prototype",
        index: AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Function",
        index: FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Object",
        index: OBJECT_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Array",
        index: ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Number",
        index: NUMBER_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "String",
        index: STRING_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Boolean",
        index: BOOLEAN_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Error",
        index: ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "TypeError",
        index: TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "ReferenceError",
        index: REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "EvalError",
        index: EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "AggregateError",
        index: AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "RangeError",
        index: RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "SyntaxError",
        index: SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "URIError",
        index: URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "ArrayBuffer.prototype",
        index: ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "SharedArrayBuffer.prototype",
        index: SHARED_ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "DataView.prototype",
        index: DATA_VIEW_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%TypedArray%.prototype",
        index: TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%TypedArray%",
        index: TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "ArrayBuffer",
        index: ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "SharedArrayBuffer",
        index: SHARED_ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "DataView",
        index: DATA_VIEW_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Reflect",
        index: REFLECT_OBJECT_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Float64Array",
        index: FLOAT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Float32Array",
        index: FLOAT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Int32Array",
        index: INT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Int16Array",
        index: INT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Int8Array",
        index: INT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Uint32Array",
        index: UINT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Uint16Array",
        index: UINT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Uint8Array",
        index: UINT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Uint8ClampedArray",
        index: UINT8_CLAMPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "BigInt",
        index: BIGINT_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Proxy",
        index: PROXY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Math",
        index: MATH_OBJECT_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Date.prototype",
        index: DATE_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Date",
        index: DATE_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "RegExp.prototype",
        index: REGEXP_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "RegExp",
        index: REGEXP_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "RegExp.prototype[Symbol.match]",
        index: REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "JSON",
        index: JSON_OBJECT_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%ArrayIteratorPrototype%",
        index: ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%IteratorPrototype%",
        index: ITERATOR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%IteratorFromWrapperPrototype%",
        index: ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Iterator",
        index: ITERATOR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "throw_error_name_heap",
        index: THROW_ERROR_NAME_HEAP_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "SuppressedError.prototype",
        index: SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "SuppressedError",
        index: SUPPRESSED_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%ThrowTypeError%",
        index: THROW_TYPE_ERROR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "BigInt64Array",
        index: BIGINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "BigUint64Array",
        index: BIGUINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "RegExp.prototype[Symbol.search]",
        index: REGEXP_PROTOTYPE_SYMBOL_SEARCH_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "RegExp.prototype[Symbol.matchAll]",
        index: REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%ArrayTypedArrayToString%",
        index: ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "[[CurrentRealm]]",
        index: CURRENT_REALM_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Atomics",
        index: ATOMICS_OBJECT_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Symbol",
        index: SYMBOL_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Symbol.prototype",
        index: SYMBOL_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "[[SymbolRegistry]]",
        index: SYMBOL_REGISTRY_GLOBAL_INDEX,
    },
];

pub(crate) fn standard_builtin_constructor_global_index(builtin: StandardBuiltinId) -> Option<u32> {
    match builtin {
        StandardBuiltinId::FunctionConstructor => Some(FUNCTION_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::AggregateErrorConstructor => {
            Some(AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::SuppressedErrorConstructor => {
            Some(SUPPRESSED_ERROR_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::ObjectConstructor => Some(OBJECT_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ProxyConstructor => Some(PROXY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::IteratorConstructor => Some(ITERATOR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ArrayConstructor => Some(ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ArrayBufferConstructor => Some(ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::SharedArrayBufferConstructor => {
            Some(SHARED_ARRAY_BUFFER_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::DataViewConstructor => Some(DATA_VIEW_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::DateConstructor => Some(DATE_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::RegExpConstructor => Some(REGEXP_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Float64ArrayConstructor => Some(FLOAT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Float32ArrayConstructor => Some(FLOAT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Int32ArrayConstructor => Some(INT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Int16ArrayConstructor => Some(INT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Int8ArrayConstructor => Some(INT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Uint32ArrayConstructor => Some(UINT32_ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Uint16ArrayConstructor => Some(UINT16_ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Uint8ArrayConstructor => Some(UINT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::Uint8ClampedArrayConstructor => {
            Some(UINT8_CLAMPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::BigInt64ArrayConstructor => {
            Some(BIGINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::BigUint64ArrayConstructor => {
            Some(BIGUINT64_ARRAY_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::BigIntConstructor => Some(BIGINT_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::NumberConstructor => Some(NUMBER_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::StringConstructor => Some(STRING_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::BooleanConstructor => Some(BOOLEAN_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::SymbolConstructor => Some(SYMBOL_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ErrorConstructor => Some(ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::EvalErrorConstructor => Some(EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::RangeErrorConstructor => Some(RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::SyntaxErrorConstructor => Some(SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::TypeErrorConstructor => Some(TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::URIErrorConstructor => Some(URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::ReferenceErrorConstructor => {
            Some(REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::FunctionPrototypeCall
        | StandardBuiltinId::FunctionPrototypeApply
        | StandardBuiltinId::FunctionPrototypeBind
        | StandardBuiltinId::FunctionPrototypeToString
        | StandardBuiltinId::EvalFunction
        | StandardBuiltinId::StringPrototypeToString
        | StandardBuiltinId::StringPrototypeValueOf
        | StandardBuiltinId::StringPrototypeCharAt
        | StandardBuiltinId::StringPrototypeCharCodeAt
        | StandardBuiltinId::StringPrototypeCodePointAt
        | StandardBuiltinId::StringPrototypeAt
        | StandardBuiltinId::StringPrototypeAnchor
        | StandardBuiltinId::StringPrototypeBig
        | StandardBuiltinId::StringPrototypeBlink
        | StandardBuiltinId::StringPrototypeBold
        | StandardBuiltinId::StringPrototypeFixed
        | StandardBuiltinId::StringPrototypeFontcolor
        | StandardBuiltinId::StringPrototypeFontsize
        | StandardBuiltinId::StringPrototypeItalics
        | StandardBuiltinId::StringPrototypeLink
        | StandardBuiltinId::StringPrototypeSmall
        | StandardBuiltinId::StringPrototypeStrike
        | StandardBuiltinId::StringPrototypeSub
        | StandardBuiltinId::StringPrototypeSubstr
        | StandardBuiltinId::StringPrototypeSubstring
        | StandardBuiltinId::StringPrototypeSup
        | StandardBuiltinId::StringPrototypeMatch
        | StandardBuiltinId::StringPrototypeMatchAll
        | StandardBuiltinId::StringPrototypeReplace
        | StandardBuiltinId::StringPrototypeReplaceAll
        | StandardBuiltinId::StringPrototypeSearch
        | StandardBuiltinId::StringPrototypeIndexOf
        | StandardBuiltinId::StringPrototypeLastIndexOf
        | StandardBuiltinId::StringPrototypeSlice
        | StandardBuiltinId::StringPrototypeSplit
        | StandardBuiltinId::StringPrototypePadStart
        | StandardBuiltinId::StringPrototypePadEnd
        | StandardBuiltinId::StringPrototypeRepeat
        | StandardBuiltinId::StringPrototypeEndsWith
        | StandardBuiltinId::StringPrototypeIncludes
        | StandardBuiltinId::StringPrototypeStartsWith
        | StandardBuiltinId::StringPrototypeToUpperCase
        | StandardBuiltinId::StringPrototypeTrim
        | StandardBuiltinId::StringPrototypeTrimStart
        | StandardBuiltinId::StringPrototypeTrimEnd
        | StandardBuiltinId::StringPrototypeIsWellFormed
        | StandardBuiltinId::StringPrototypeToWellFormed
        | StandardBuiltinId::DateNow
        | StandardBuiltinId::DateUtc
        | StandardBuiltinId::DatePrototypeGetTime
        | StandardBuiltinId::DatePrototypeSetTime
        | StandardBuiltinId::DatePrototypeValueOf
        | StandardBuiltinId::DatePrototypeGetFullYear
        | StandardBuiltinId::DatePrototypeGetUtcFullYear
        | StandardBuiltinId::DatePrototypeGetMonth
        | StandardBuiltinId::DatePrototypeGetUtcMonth
        | StandardBuiltinId::DatePrototypeGetDate
        | StandardBuiltinId::DatePrototypeGetUtcDate
        | StandardBuiltinId::DatePrototypeGetDay
        | StandardBuiltinId::DatePrototypeGetUtcDay
        | StandardBuiltinId::DatePrototypeGetHours
        | StandardBuiltinId::DatePrototypeGetUtcHours
        | StandardBuiltinId::DatePrototypeGetMinutes
        | StandardBuiltinId::DatePrototypeGetUtcMinutes
        | StandardBuiltinId::DatePrototypeGetSeconds
        | StandardBuiltinId::DatePrototypeGetUtcSeconds
        | StandardBuiltinId::DatePrototypeGetMilliseconds
        | StandardBuiltinId::DatePrototypeGetUtcMilliseconds
        | StandardBuiltinId::DatePrototypeGetTimezoneOffset
        | StandardBuiltinId::DatePrototypeGetYear
        | StandardBuiltinId::DatePrototypeSetYear
        | StandardBuiltinId::DatePrototypeSetFullYear
        | StandardBuiltinId::DatePrototypeSetUtcFullYear
        | StandardBuiltinId::DatePrototypeSetMonth
        | StandardBuiltinId::DatePrototypeSetUtcMonth
        | StandardBuiltinId::DatePrototypeSetDate
        | StandardBuiltinId::DatePrototypeSetUtcDate
        | StandardBuiltinId::DatePrototypeSetHours
        | StandardBuiltinId::DatePrototypeSetUtcHours
        | StandardBuiltinId::DatePrototypeSetMinutes
        | StandardBuiltinId::DatePrototypeSetUtcMinutes
        | StandardBuiltinId::DatePrototypeSetSeconds
        | StandardBuiltinId::DatePrototypeSetUtcSeconds
        | StandardBuiltinId::DatePrototypeSetMilliseconds
        | StandardBuiltinId::DatePrototypeSetUtcMilliseconds
        | StandardBuiltinId::DatePrototypeToUtcString
        | StandardBuiltinId::RegExpLegacyStaticGetter
        | StandardBuiltinId::RegExpLegacyStaticSetter
        | StandardBuiltinId::RegExpSpeciesGetter
        | StandardBuiltinId::RegExpPrototypeSymbolMatch
        | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
        | StandardBuiltinId::RegExpPrototypeSymbolSearch
        | StandardBuiltinId::RegExpEscape
        | StandardBuiltinId::ObjectCreate
        | StandardBuiltinId::ObjectGetPrototypeOf
        | StandardBuiltinId::ObjectSetPrototypeOf
        | StandardBuiltinId::ObjectDefineProperty
        | StandardBuiltinId::ObjectDefineProperties
        | StandardBuiltinId::ObjectGetOwnPropertyDescriptor
        | StandardBuiltinId::ObjectGetOwnPropertyNames
        | StandardBuiltinId::ObjectGetOwnPropertySymbols
        | StandardBuiltinId::ObjectKeys
        | StandardBuiltinId::ObjectValues
        | StandardBuiltinId::ObjectHasOwn
        | StandardBuiltinId::ObjectIs
        | StandardBuiltinId::ObjectIsSealed
        | StandardBuiltinId::ObjectIsFrozen
        | StandardBuiltinId::ObjectFreeze
        | StandardBuiltinId::ObjectIsExtensible
        | StandardBuiltinId::ObjectPreventExtensions
        | StandardBuiltinId::ObjectPrototypeHasOwnProperty
        | StandardBuiltinId::ObjectPrototypePropertyIsEnumerable
        | StandardBuiltinId::ObjectPrototypeIsPrototypeOf
        | StandardBuiltinId::ObjectPrototypeToString
        | StandardBuiltinId::ObjectPrototypeToLocaleString
        | StandardBuiltinId::ObjectPrototypeValueOf
        | StandardBuiltinId::ProxyRevocable
        | StandardBuiltinId::ProxyRevoke
        | StandardBuiltinId::ReflectConstruct
        | StandardBuiltinId::ReflectApply
        | StandardBuiltinId::ReflectGet
        | StandardBuiltinId::ReflectGetPrototypeOf
        | StandardBuiltinId::ReflectGetOwnPropertyDescriptor
        | StandardBuiltinId::ReflectSet
        | StandardBuiltinId::ReflectHas
        | StandardBuiltinId::ReflectDefineProperty
        | StandardBuiltinId::ReflectDeleteProperty
        | StandardBuiltinId::ReflectIsExtensible
        | StandardBuiltinId::ReflectPreventExtensions
        | StandardBuiltinId::ReflectSetPrototypeOf
        | StandardBuiltinId::ReflectOwnKeys
        | StandardBuiltinId::ArrayFrom
        | StandardBuiltinId::ArrayOf
        | StandardBuiltinId::ArrayIsArray
        | StandardBuiltinId::ArraySpeciesGetter
        | StandardBuiltinId::ArrayPrototypeConcat
        | StandardBuiltinId::ArrayPrototypeToLocaleString
        | StandardBuiltinId::ArrayPrototypeFlat
        | StandardBuiltinId::ArrayPrototypeFlatMap
        | StandardBuiltinId::ArrayPrototypeAt
        | StandardBuiltinId::ArrayPrototypeIncludes
        | StandardBuiltinId::ArrayPrototypeIndexOf
        | StandardBuiltinId::ArrayPrototypeLastIndexOf
        | StandardBuiltinId::ArrayPrototypeFind
        | StandardBuiltinId::ArrayPrototypeFindIndex
        | StandardBuiltinId::ArrayPrototypeFindLast
        | StandardBuiltinId::ArrayPrototypeFindLastIndex
        | StandardBuiltinId::ArrayPrototypeEvery
        | StandardBuiltinId::ArrayPrototypeSome
        | StandardBuiltinId::ArrayPrototypeForEach
        | StandardBuiltinId::ArrayPrototypeFilter
        | StandardBuiltinId::ArrayPrototypeMap
        | StandardBuiltinId::ArrayPrototypePop
        | StandardBuiltinId::ArrayPrototypePush
        | StandardBuiltinId::ArrayPrototypeKeys
        | StandardBuiltinId::ArrayPrototypeEntries
        | StandardBuiltinId::ArrayPrototypeValues
        | StandardBuiltinId::ArrayIteratorNext
        | StandardBuiltinId::ArrayIteratorIdentity
        | StandardBuiltinId::IteratorFrom
        | StandardBuiltinId::IteratorPrototypeToArray
        | StandardBuiltinId::IteratorPrototypeForEach
        | StandardBuiltinId::IteratorPrototypeEvery
        | StandardBuiltinId::IteratorPrototypeSome
        | StandardBuiltinId::IteratorPrototypeFind
        | StandardBuiltinId::IteratorPrototypeReduce
        | StandardBuiltinId::IteratorPrototypeMap
        | StandardBuiltinId::IteratorMapNext
        | StandardBuiltinId::IteratorMapReturn
        | StandardBuiltinId::IteratorPrototypeFilter
        | StandardBuiltinId::IteratorFilterNext
        | StandardBuiltinId::IteratorFilterReturn
        | StandardBuiltinId::IteratorPrototypeFlatMap
        | StandardBuiltinId::IteratorFlatMapNext
        | StandardBuiltinId::IteratorFlatMapReturn
        | StandardBuiltinId::IteratorPrototypeTake
        | StandardBuiltinId::IteratorTakeNext
        | StandardBuiltinId::IteratorTakeReturn
        | StandardBuiltinId::IteratorPrototypeDrop
        | StandardBuiltinId::IteratorDropNext
        | StandardBuiltinId::IteratorDropReturn
        | StandardBuiltinId::IteratorPrototypeConstructorGetter
        | StandardBuiltinId::IteratorPrototypeConstructorSetter
        | StandardBuiltinId::IteratorPrototypeSymbolDispose
        | StandardBuiltinId::IteratorPrototypeToStringTagGetter
        | StandardBuiltinId::IteratorPrototypeToStringTagSetter
        | StandardBuiltinId::IteratorFromWrapperNext
        | StandardBuiltinId::IteratorFromWrapperReturn
        | StandardBuiltinId::ArrayBufferIsView
        | StandardBuiltinId::BigIntAsIntN
        | StandardBuiltinId::BigIntAsUintN
        | StandardBuiltinId::BigIntPrototypeToString
        | StandardBuiltinId::BigIntPrototypeToLocaleString
        | StandardBuiltinId::BigIntPrototypeValueOf
        | StandardBuiltinId::NumberIsInteger
        | StandardBuiltinId::NumberIsSafeInteger
        | StandardBuiltinId::NumberIsFinite
        | StandardBuiltinId::NumberIsNaN
        | StandardBuiltinId::NumberPrototypeToExponential
        | StandardBuiltinId::NumberPrototypeToFixed
        | StandardBuiltinId::NumberPrototypeToPrecision
        | StandardBuiltinId::NumberPrototypeToString
        | StandardBuiltinId::NumberPrototypeToLocaleString
        | StandardBuiltinId::NumberPrototypeValueOf
        | StandardBuiltinId::BooleanPrototypeToString
        | StandardBuiltinId::BooleanPrototypeValueOf
        | StandardBuiltinId::GlobalIsFinite
        | StandardBuiltinId::GlobalIsNaN
        | StandardBuiltinId::MathAbs
        | StandardBuiltinId::MathAcos
        | StandardBuiltinId::MathAcosh
        | StandardBuiltinId::MathAsin
        | StandardBuiltinId::MathAsinh
        | StandardBuiltinId::MathAtan
        | StandardBuiltinId::MathAtan2
        | StandardBuiltinId::MathAtanh
        | StandardBuiltinId::MathCbrt
        | StandardBuiltinId::MathCeil
        | StandardBuiltinId::MathClz32
        | StandardBuiltinId::MathCos
        | StandardBuiltinId::MathCosh
        | StandardBuiltinId::MathExp
        | StandardBuiltinId::MathExpm1
        | StandardBuiltinId::MathF16Round
        | StandardBuiltinId::MathFloor
        | StandardBuiltinId::MathFround
        | StandardBuiltinId::MathHypot
        | StandardBuiltinId::MathImul
        | StandardBuiltinId::MathLog
        | StandardBuiltinId::MathLog10
        | StandardBuiltinId::MathLog1p
        | StandardBuiltinId::MathLog2
        | StandardBuiltinId::MathPow
        | StandardBuiltinId::MathRandom
        | StandardBuiltinId::MathRound
        | StandardBuiltinId::MathSign
        | StandardBuiltinId::MathSin
        | StandardBuiltinId::MathSinh
        | StandardBuiltinId::MathSqrt
        | StandardBuiltinId::MathSumPrecise
        | StandardBuiltinId::MathTan
        | StandardBuiltinId::MathTanh
        | StandardBuiltinId::MathTrunc
        | StandardBuiltinId::MathMin
        | StandardBuiltinId::MathMax
        | StandardBuiltinId::ErrorIsError
        | StandardBuiltinId::ArrayBufferSpeciesGetter
        | StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
        | StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
        | StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter
        | StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter
        | StandardBuiltinId::SharedArrayBufferPrototypeGrow
        | StandardBuiltinId::ArrayBufferPrototypeDetachedGetter
        | StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter
        | StandardBuiltinId::ArrayBufferPrototypeResizableGetter
        | StandardBuiltinId::ArrayBufferPrototypeResize
        | StandardBuiltinId::ArrayBufferPrototypeSlice
        | StandardBuiltinId::SharedArrayBufferPrototypeSlice
        | StandardBuiltinId::ArrayBufferPrototypeTransfer
        | StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength
        | StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable
        | StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable
        | StandardBuiltinId::DataViewPrototypeBufferGetter
        | StandardBuiltinId::DataViewPrototypeByteLengthGetter
        | StandardBuiltinId::DataViewPrototypeByteOffsetGetter
        | StandardBuiltinId::TypedArrayPrototypeBufferGetter
        | StandardBuiltinId::TypedArrayPrototypeByteLengthGetter
        | StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter
        | StandardBuiltinId::TypedArrayPrototypeLengthGetter
        | StandardBuiltinId::TypedArrayPrototypeToString
        | StandardBuiltinId::TypedArrayPrototypeToLocaleString
        | StandardBuiltinId::TypedArrayFrom
        | StandardBuiltinId::TypedArrayOf
        | StandardBuiltinId::DataViewPrototypeGetUint8
        | StandardBuiltinId::DataViewPrototypeSetUint8
        | StandardBuiltinId::DataViewPrototypeGetInt8
        | StandardBuiltinId::DataViewPrototypeSetInt8
        | StandardBuiltinId::DataViewPrototypeGetUint16
        | StandardBuiltinId::DataViewPrototypeSetUint16
        | StandardBuiltinId::DataViewPrototypeGetInt16
        | StandardBuiltinId::DataViewPrototypeSetInt16
        | StandardBuiltinId::DataViewPrototypeGetUint32
        | StandardBuiltinId::DataViewPrototypeSetUint32
        | StandardBuiltinId::DataViewPrototypeGetInt32
        | StandardBuiltinId::DataViewPrototypeSetInt32
        | StandardBuiltinId::DataViewPrototypeGetFloat16
        | StandardBuiltinId::DataViewPrototypeSetFloat16
        | StandardBuiltinId::DataViewPrototypeGetFloat32
        | StandardBuiltinId::DataViewPrototypeSetFloat32
        | StandardBuiltinId::DataViewPrototypeGetFloat64
        | StandardBuiltinId::DataViewPrototypeSetFloat64
        | StandardBuiltinId::DataViewPrototypeGetBigInt64
        | StandardBuiltinId::DataViewPrototypeSetBigInt64
        | StandardBuiltinId::DataViewPrototypeGetBigUint64
        | StandardBuiltinId::DataViewPrototypeSetBigUint64
        | StandardBuiltinId::ErrorPrototypeToString
        | StandardBuiltinId::ThrowTypeError
        | StandardBuiltinId::BoundFunctionInvoker
        | StandardBuiltinId::JsonParse
        | StandardBuiltinId::JsonStringify
        | StandardBuiltinId::JsonRawJson
        | StandardBuiltinId::JsonIsRawJson
        | StandardBuiltinId::AtomicsAdd
        | StandardBuiltinId::AtomicsAnd
        | StandardBuiltinId::AtomicsCompareExchange
        | StandardBuiltinId::AtomicsExchange
        | StandardBuiltinId::AtomicsLoad
        | StandardBuiltinId::AtomicsNotify
        | StandardBuiltinId::AtomicsOr
        | StandardBuiltinId::AtomicsPause
        | StandardBuiltinId::AtomicsSub
        | StandardBuiltinId::AtomicsStore
        | StandardBuiltinId::AtomicsWait
        | StandardBuiltinId::AtomicsWaitAsync
        | StandardBuiltinId::AtomicsXor
        | StandardBuiltinId::AtomicsIsLockFree
        | StandardBuiltinId::Escape
        | StandardBuiltinId::Unescape
        | StandardBuiltinId::SymbolFor
        | StandardBuiltinId::SymbolKeyFor => None,
    }
}

pub(crate) fn typed_array_constructor_bytes_per_element_entries() -> [(StandardBuiltinId, u64); 11]
{
    [
        (StandardBuiltinId::Float64ArrayConstructor, 8),
        (StandardBuiltinId::Float32ArrayConstructor, 4),
        (StandardBuiltinId::Int32ArrayConstructor, 4),
        (StandardBuiltinId::Int16ArrayConstructor, 2),
        (StandardBuiltinId::Int8ArrayConstructor, 1),
        (StandardBuiltinId::Uint32ArrayConstructor, 4),
        (StandardBuiltinId::Uint16ArrayConstructor, 2),
        (StandardBuiltinId::Uint8ArrayConstructor, 1),
        (StandardBuiltinId::Uint8ClampedArrayConstructor, 1),
        (StandardBuiltinId::BigInt64ArrayConstructor, 8),
        (StandardBuiltinId::BigUint64ArrayConstructor, 8),
    ]
}

pub(crate) fn host_builtin_by_name(name: &str) -> Option<HostBuiltinId> {
    all_host_builtins()
        .iter()
        .copied()
        .find(|builtin| builtin.as_str() == name)
}

pub(crate) fn all_host_builtins() -> &'static [HostBuiltinId] {
    &[
        HostBuiltinId::Print,
        HostBuiltinId::Gc,
        HostBuiltinId::AssertThrows,
        HostBuiltinId::IsConstructor,
        HostBuiltinId::CreateRealm,
        HostBuiltinId::ParseInt,
        HostBuiltinId::ParseFloat,
        HostBuiltinId::DetachArrayBuffer,
    ]
}

pub(crate) fn typed_array_element_kind(builtin: StandardBuiltinId) -> u64 {
    match builtin {
        StandardBuiltinId::Float64ArrayConstructor => 1,
        StandardBuiltinId::Float32ArrayConstructor => 2,
        StandardBuiltinId::Int8ArrayConstructor => 3,
        StandardBuiltinId::Int16ArrayConstructor => 4,
        StandardBuiltinId::Int32ArrayConstructor => 5,
        StandardBuiltinId::Uint8ClampedArrayConstructor => 6,
        StandardBuiltinId::Uint8ArrayConstructor => 7,
        StandardBuiltinId::Uint16ArrayConstructor => 8,
        StandardBuiltinId::Uint32ArrayConstructor => 9,
        StandardBuiltinId::BigInt64ArrayConstructor => 10,
        StandardBuiltinId::BigUint64ArrayConstructor => 11,
        _ => 0,
    }
}

pub(crate) fn typed_array_realm_prototype_offset(builtin: StandardBuiltinId) -> Option<u64> {
    Some(match builtin {
        StandardBuiltinId::Float64ArrayConstructor => {
            HEAP_FUNCTION_REALM_FLOAT64_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Float32ArrayConstructor => {
            HEAP_FUNCTION_REALM_FLOAT32_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Int32ArrayConstructor => {
            HEAP_FUNCTION_REALM_INT32_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Int16ArrayConstructor => {
            HEAP_FUNCTION_REALM_INT16_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Int8ArrayConstructor => HEAP_FUNCTION_REALM_INT8_ARRAY_PROTOTYPE_OFFSET,
        StandardBuiltinId::Uint32ArrayConstructor => {
            HEAP_FUNCTION_REALM_UINT32_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Uint16ArrayConstructor => {
            HEAP_FUNCTION_REALM_UINT16_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Uint8ArrayConstructor => {
            HEAP_FUNCTION_REALM_UINT8_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Uint8ClampedArrayConstructor => {
            HEAP_FUNCTION_REALM_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::BigInt64ArrayConstructor => {
            HEAP_FUNCTION_REALM_BIGINT64_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::BigUint64ArrayConstructor => {
            HEAP_FUNCTION_REALM_BIGUINT64_ARRAY_PROTOTYPE_OFFSET
        }
        _ => return None,
    })
}

pub(crate) fn typed_array_realm_prototype_debug_slot(
    builtin: StandardBuiltinId,
) -> Option<&'static str> {
    Some(match builtin {
        StandardBuiltinId::Float64ArrayConstructor => "$Realm.Float64Array.prototype",
        StandardBuiltinId::Float32ArrayConstructor => "$Realm.Float32Array.prototype",
        StandardBuiltinId::Int32ArrayConstructor => "$Realm.Int32Array.prototype",
        StandardBuiltinId::Int16ArrayConstructor => "$Realm.Int16Array.prototype",
        StandardBuiltinId::Int8ArrayConstructor => "$Realm.Int8Array.prototype",
        StandardBuiltinId::Uint32ArrayConstructor => "$Realm.Uint32Array.prototype",
        StandardBuiltinId::Uint16ArrayConstructor => "$Realm.Uint16Array.prototype",
        StandardBuiltinId::Uint8ArrayConstructor => "$Realm.Uint8Array.prototype",
        StandardBuiltinId::Uint8ClampedArrayConstructor => "$Realm.Uint8ClampedArray.prototype",
        StandardBuiltinId::BigInt64ArrayConstructor => "$Realm.BigInt64Array.prototype",
        StandardBuiltinId::BigUint64ArrayConstructor => "$Realm.BigUint64Array.prototype",
        _ => return None,
    })
}

pub(crate) fn typed_array_bytes_per_element(builtin: StandardBuiltinId) -> u64 {
    typed_array_constructor_bytes_per_element_entries()
        .into_iter()
        .find_map(|(candidate, bytes)| (candidate == builtin).then_some(bytes))
        .unwrap_or(1)
}

pub(crate) fn is_typed_array_constructor(builtin: StandardBuiltinId) -> bool {
    typed_array_constructor_bytes_per_element_entries()
        .into_iter()
        .any(|(candidate, _)| candidate == builtin)
}

pub(crate) const fn throw_error_name_global_index(uses_heap: bool) -> u32 {
    if uses_heap {
        THROW_ERROR_NAME_HEAP_GLOBAL_INDEX
    } else {
        THROW_ERROR_NAME_NO_HEAP_GLOBAL_INDEX
    }
}

pub(crate) fn error_prototype_global_index(name: &str) -> u32 {
    match name {
        ERROR_NAME => ERROR_PROTOTYPE_GLOBAL_INDEX,
        EVAL_ERROR_NAME => EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
        AGGREGATE_ERROR_NAME => AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        SUPPRESSED_ERROR_NAME => SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
        RANGE_ERROR_NAME => RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        SYNTAX_ERROR_NAME => SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
        TYPE_ERROR_NAME => TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        URI_ERROR_NAME => URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
        REFERENCE_ERROR_NAME => REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        _ => OBJECT_PROTOTYPE_GLOBAL_INDEX,
    }
}

pub(crate) fn error_realm_prototype_offset(name: &str) -> Option<u64> {
    match name {
        ERROR_NAME => Some(HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET),
        EVAL_ERROR_NAME => Some(HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET),
        RANGE_ERROR_NAME => Some(HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET),
        REFERENCE_ERROR_NAME => Some(HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET),
        SYNTAX_ERROR_NAME => Some(HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET),
        TYPE_ERROR_NAME => Some(HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET),
        URI_ERROR_NAME => Some(HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET),
        AGGREGATE_ERROR_NAME => Some(HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET),
        SUPPRESSED_ERROR_NAME => Some(HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET),
        _ => None,
    }
}

pub(crate) fn error_realm_prototype_entries() -> [(&'static str, u32, u64); 9] {
    [
        (
            ERROR_NAME,
            ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            EVAL_ERROR_NAME,
            EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            RANGE_ERROR_NAME,
            RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            REFERENCE_ERROR_NAME,
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            SYNTAX_ERROR_NAME,
            SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            TYPE_ERROR_NAME,
            TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            URI_ERROR_NAME,
            URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            AGGREGATE_ERROR_NAME,
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
        ),
        (
            SUPPRESSED_ERROR_NAME,
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
        ),
    ]
}

pub(crate) fn standard_builtin_prototype_global_index(builtin: StandardBuiltinId) -> Option<u32> {
    match builtin {
        StandardBuiltinId::ObjectConstructor => Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::FunctionConstructor => Some(FUNCTION_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::ArrayConstructor => Some(ARRAY_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::IteratorConstructor => Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::NumberConstructor => Some(NUMBER_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::StringConstructor => Some(STRING_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::BooleanConstructor => Some(BOOLEAN_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::SymbolConstructor => Some(SYMBOL_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::ErrorConstructor => Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::EvalErrorConstructor => Some(EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::AggregateErrorConstructor => {
            Some(AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::SuppressedErrorConstructor => {
            Some(SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::RangeErrorConstructor => Some(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::SyntaxErrorConstructor => Some(SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::TypeErrorConstructor => Some(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::URIErrorConstructor => Some(URI_ERROR_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::ReferenceErrorConstructor => {
            Some(REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::RegExpConstructor => Some(REGEXP_PROTOTYPE_GLOBAL_INDEX),
        _ => None,
    }
}

pub(crate) fn standard_builtin_function_global_index(builtin: StandardBuiltinId) -> Option<u32> {
    match builtin {
        StandardBuiltinId::RegExpPrototypeSymbolMatch => {
            Some(REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX)
        }
        StandardBuiltinId::RegExpPrototypeSymbolMatchAll => {
            Some(REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_GLOBAL_INDEX)
        }
        StandardBuiltinId::RegExpPrototypeSymbolSearch => {
            Some(REGEXP_PROTOTYPE_SYMBOL_SEARCH_GLOBAL_INDEX)
        }
        StandardBuiltinId::TypedArrayPrototypeToString => {
            Some(ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX)
        }
        StandardBuiltinId::ThrowTypeError => Some(THROW_TYPE_ERROR_GLOBAL_INDEX),
        _ => standard_builtin_constructor_global_index(builtin),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmArtifact {
    pub bytes: Vec<u8>,
    pub invariant_note: &'static str,
    pub debug_dump: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitError {
    message: String,
}

impl EmitError {
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl core::fmt::Display for EmitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EmitError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn global_index_registry_is_unique_and_dense() {
        let mut indexes = BTreeSet::new();
        let mut names = BTreeSet::new();
        for slot in GLOBAL_INDEX_REGISTRY {
            assert!(indexes.insert(slot.index), "duplicate index {}", slot.index);
            assert!(names.insert(slot.name), "duplicate name {}", slot.name);
        }
        for (expected, slot) in GLOBAL_INDEX_REGISTRY.iter().enumerate() {
            assert_eq!(
                slot.index, expected as u32,
                "global {} should stay at index {}",
                slot.name, expected
            );
        }
        assert_eq!(
            THROW_ERROR_NAME_NO_HEAP_GLOBAL_INDEX, HEAP_PTR_GLOBAL_INDEX,
            "no-heap throw-error-name export intentionally aliases the heap pointer slot"
        );
    }
}
