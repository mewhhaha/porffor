use super::*;

pub(crate) const RESULT_TAG_EXPORT: &str = "result_tag";
pub(crate) const COMPLETION_KIND_EXPORT: &str = "completion_kind";
pub(crate) const COMPLETION_AUX_EXPORT: &str = "completion_aux";
pub(crate) const THROW_ERROR_NAME_EXPORT: &str = "throw_error_name";
/// Companion export to `THROW_ERROR_NAME_EXPORT`. The host reads both at an
/// uncaught throw so a failure detail can name the defect
/// (`TypeError: RegExp.prototype.exec unsupported pattern`) instead of printing
/// a raw linear-memory address (`TypeError: object(handle@5397552)`), which is
/// neither stable across builds nor resolvable to an allocation site.
pub(crate) const THROW_ERROR_MESSAGE_EXPORT: &str = "throw_error_message";

pub(crate) const HOST_IMPORT_MODULE: &str = "porf_host";
pub(crate) const HOST_IMPORT_AGENT_CAN_SUSPEND: &str = "agent_can_suspend";
pub(crate) const HOST_IMPORT_PRINT_LINE_UTF8: &str = "print_line_utf8";
pub(crate) const HOST_IMPORT_NUMBER_POW: &str = "number_pow";
pub(crate) const HOST_IMPORT_PRIVATE_MEMORY: &str = "private_memory";
pub(crate) const HOST_IMPORT_SHARED_MEMORY: &str = "shared_memory";
pub(crate) const HOST_IMPORT_SHARED_MEMORY_ALLOC: &str = "shared_memory_alloc";
pub(crate) const HOST_IMPORT_WALL_CLOCK_MILLIS: &str = "wall_clock_millis";
pub(crate) const HOST_IMPORT_MONOTONIC_CLOCK_NANOS: &str = "monotonic_clock_nanos";
pub(crate) const HOST_IMPORT_SLEEP_NANOS: &str = "sleep_nanos";
pub(crate) const HOST_IMPORT_AGENT_CALL: &str = "agent_call";

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
// Canonical per-realm `parseInt`/`parseFloat` function objects. The spec
// requires `Number.parseInt` and the global `parseInt` to be the *same*
// function object (likewise `parseFloat`). These mutable globals hold that
// object during realm construction so every install site reads one identity;
// they are reset to 0 at the start of each realm build so createRealm mints
// fresh objects per realm rather than aliasing the previous realm's.
pub(crate) const PARSE_INT_FUNCTION_GLOBAL_INDEX: u32 = 78;
pub(crate) const PARSE_FLOAT_FUNCTION_GLOBAL_INDEX: u32 = 79;
// The main realm's original RegExp.prototype.exec function object. The
// RegExp @@match and @@search compact paths require this exact identity.
pub(crate) const REGEXP_PROTOTYPE_EXEC_FUNCTION_GLOBAL_INDEX: u32 = 80;
pub(crate) const ITERATOR_HELPER_PROTOTYPE_GLOBAL_INDEX: u32 = 81;
pub(crate) const STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX: u32 = 82;
pub(crate) const PROMISE_PROTOTYPE_GLOBAL_INDEX: u32 = 83;
pub(crate) const PROMISE_CONSTRUCTOR_GLOBAL_INDEX: u32 = 84;
pub(crate) const PROMISE_JOB_QUEUE_HEAD_GLOBAL_INDEX: u32 = 85;
pub(crate) const PROMISE_JOB_QUEUE_TAIL_GLOBAL_INDEX: u32 = 86;
pub(crate) const MAP_PROTOTYPE_GLOBAL_INDEX: u32 = 87;
pub(crate) const MAP_CONSTRUCTOR_GLOBAL_INDEX: u32 = 88;
pub(crate) const SET_PROTOTYPE_GLOBAL_INDEX: u32 = 89;
pub(crate) const SET_CONSTRUCTOR_GLOBAL_INDEX: u32 = 90;
pub(crate) const MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX: u32 = 91;
pub(crate) const SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX: u32 = 92;
pub(crate) const GENERATOR_PROTOTYPE_GLOBAL_INDEX: u32 = 93;
pub(crate) const GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX: u32 = 94;
pub(crate) const GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX: u32 = 95;
pub(crate) const ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX: u32 = 96;
pub(crate) const ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX: u32 = 97;
pub(crate) const ASYNC_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX: u32 = 98;
pub(crate) const ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX: u32 = 99;
pub(crate) const ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX: u32 = 100;
pub(crate) const ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX: u32 = 101;
pub(crate) const WEAK_MAP_PROTOTYPE_GLOBAL_INDEX: u32 = 102;
pub(crate) const WEAK_MAP_CONSTRUCTOR_GLOBAL_INDEX: u32 = 103;
pub(crate) const ATOMICS_ASYNC_WAITER_ACTIVE_LIST_HEAD_GLOBAL_INDEX: u32 = 104;
pub(crate) const TEMPORAL_OBJECT_GLOBAL_INDEX: u32 = 105;
pub(crate) const TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX: u32 = 106;
pub(crate) const TEMPORAL_INSTANT_CONSTRUCTOR_GLOBAL_INDEX: u32 = 107;
pub(crate) const WEAK_REF_PROTOTYPE_GLOBAL_INDEX: u32 = 108;
pub(crate) const WEAK_REF_CONSTRUCTOR_GLOBAL_INDEX: u32 = 109;
pub(crate) const FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX: u32 = 110;
pub(crate) const FINALIZATION_REGISTRY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 111;
pub(crate) const WEAK_SET_PROTOTYPE_GLOBAL_INDEX: u32 = 112;
pub(crate) const WEAK_SET_CONSTRUCTOR_GLOBAL_INDEX: u32 = 113;
pub(crate) const TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX: u32 = 114;
pub(crate) const TEMPORAL_ZONED_DATE_TIME_CONSTRUCTOR_GLOBAL_INDEX: u32 = 115;
pub(crate) const INTL_OBJECT_GLOBAL_INDEX: u32 = 116;
pub(crate) const INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX: u32 = 117;
pub(crate) const INTL_LOCALE_CONSTRUCTOR_GLOBAL_INDEX: u32 = 118;
// HostPromiseRejectionTracker bookkeeping: a FIFO of promise records that were
// rejected while [[IsHandled]] was false. The list is walked once, after the
// job queue drains, and entries whose [[IsHandled]] became true in the meantime
// are skipped - so a `.catch` attached from a later job still suppresses the
// report.
pub(crate) const PROMISE_UNHANDLED_REJECTION_HEAD_GLOBAL_INDEX: u32 = 119;
pub(crate) const PROMISE_UNHANDLED_REJECTION_TAIL_GLOBAL_INDEX: u32 = 120;
pub(crate) const TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX: u32 = 121;
pub(crate) const TEMPORAL_PLAIN_DATE_CONSTRUCTOR_GLOBAL_INDEX: u32 = 122;
pub(crate) const TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX: u32 = 123;
pub(crate) const TEMPORAL_DURATION_CONSTRUCTOR_GLOBAL_INDEX: u32 = 124;
pub(crate) const TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX: u32 = 125;
pub(crate) const TEMPORAL_PLAIN_TIME_CONSTRUCTOR_GLOBAL_INDEX: u32 = 126;
pub(crate) const TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX: u32 = 127;
pub(crate) const TEMPORAL_PLAIN_DATE_TIME_CONSTRUCTOR_GLOBAL_INDEX: u32 = 128;
pub(crate) const TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_GLOBAL_INDEX: u32 = 129;
pub(crate) const TEMPORAL_PLAIN_YEAR_MONTH_CONSTRUCTOR_GLOBAL_INDEX: u32 = 130;
pub(crate) const TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_GLOBAL_INDEX: u32 = 131;
pub(crate) const TEMPORAL_PLAIN_MONTH_DAY_CONSTRUCTOR_GLOBAL_INDEX: u32 = 132;
pub(crate) const INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX: u32 = 133;
pub(crate) const INTL_DATE_TIME_FORMAT_CONSTRUCTOR_GLOBAL_INDEX: u32 = 134;
// Appended after the previous maximum so no existing index moves. The registry
// is dense and position-indexed (`global_index_registry_is_unique_and_dense`),
// so the matching `GlobalIndexSlot` row goes at the END of
// `GLOBAL_INDEX_REGISTRY`, not next to `throw_error_name_heap`: a row inserted
// beside its sibling would renumber every global after it.
pub(crate) const THROW_ERROR_MESSAGE_HEAP_GLOBAL_INDEX: u32 = 135;

pub(crate) const THROW_ERROR_NAME_NO_HEAP_GLOBAL_INDEX: u32 = HEAP_PTR_GLOBAL_INDEX;
/// The no-heap alias, mirroring `THROW_ERROR_NAME_NO_HEAP_GLOBAL_INDEX`.
///
/// A module compiled without a heap emits four globals, so both throw-diagnostic
/// exports land on the same i64 slot and clobber each other. That is
/// unobservable rather than merely tolerated: the host reads either global only
/// when the completion value is a heap object (`Object`/`Array`/`Function`/
/// `Arguments`), and a module with no heap cannot produce one.
pub(crate) const THROW_ERROR_MESSAGE_NO_HEAP_GLOBAL_INDEX: u32 = HEAP_PTR_GLOBAL_INDEX;
pub(crate) const JS_FUNCTION_TYPE_INDEX: u32 = 1;
pub(crate) const HEAP_ALLOC_TYPE_INDEX: u32 = 2;
pub(crate) const OBJECT_APPEND_DATA_PROPERTY_TYPE_INDEX: u32 = 3;
pub(crate) const OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX: u32 = 4;
pub(crate) const FUNCTION_OBJECT_ALLOC_TYPE_INDEX: u32 = 5;
pub(crate) const PLAIN_OBJECT_ALLOC_TYPE_INDEX: u32 = 6;
pub(crate) const ARRAY_ALLOC_TYPE_INDEX: u32 = 7;
pub(crate) const HOST_PRINT_IMPORT_TYPE_INDEX: u32 = 8;
pub(crate) const HOST_NUMBER_POW_IMPORT_TYPE_INDEX: u32 = 9;
pub(crate) const HOST_AGENT_CAN_SUSPEND_IMPORT_TYPE_INDEX: u32 = 10;
pub(crate) const HOST_MONOTONIC_CLOCK_NANOS_IMPORT_TYPE_INDEX: u32 = 11;
pub(crate) const HOST_SLEEP_NANOS_IMPORT_TYPE_INDEX: u32 = 12;
pub(crate) const HOST_AGENT_CALL_IMPORT_TYPE_INDEX: u32 = 13;
pub(crate) const HOST_WALL_CLOCK_MILLIS_IMPORT_TYPE_INDEX: u32 = 14;
pub(crate) const HOST_AGENT_CAN_SUSPEND_IMPORT_FUNCTION_INDEX: u32 = 0;
pub(crate) const HOST_PRINT_IMPORT_FUNCTION_INDEX: u32 = 1;

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
    GlobalIndexSlot {
        name: "%parseInt%",
        index: PARSE_INT_FUNCTION_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%parseFloat%",
        index: PARSE_FLOAT_FUNCTION_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%RegExp.prototype.exec%",
        index: REGEXP_PROTOTYPE_EXEC_FUNCTION_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%IteratorHelperPrototype%",
        index: ITERATOR_HELPER_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%StringIteratorPrototype%",
        index: STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Promise.prototype",
        index: PROMISE_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Promise",
        index: PROMISE_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "[[PromiseJobQueueHead]]",
        index: PROMISE_JOB_QUEUE_HEAD_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "[[PromiseJobQueueTail]]",
        index: PROMISE_JOB_QUEUE_TAIL_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Map.prototype",
        index: MAP_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Map",
        index: MAP_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Set.prototype",
        index: SET_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Set",
        index: SET_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%MapIteratorPrototype%",
        index: MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%SetIteratorPrototype%",
        index: SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%GeneratorPrototype%",
        index: GENERATOR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%GeneratorFunction.prototype%",
        index: GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%GeneratorFunction%",
        index: GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%AsyncIteratorPrototype%",
        index: ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%AsyncFunction.prototype%",
        index: ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%AsyncFunction%",
        index: ASYNC_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%AsyncGeneratorPrototype%",
        index: ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%AsyncGeneratorFunction.prototype%",
        index: ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "%AsyncGeneratorFunction%",
        index: ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "WeakMap.prototype",
        index: WEAK_MAP_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "WeakMap",
        index: WEAK_MAP_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "[[AtomicsAsyncWaiterActiveListHead]]",
        index: ATOMICS_ASYNC_WAITER_ACTIVE_LIST_HEAD_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal",
        index: TEMPORAL_OBJECT_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.Instant.prototype",
        index: TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.Instant",
        index: TEMPORAL_INSTANT_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "WeakRef.prototype",
        index: WEAK_REF_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "WeakRef",
        index: WEAK_REF_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "FinalizationRegistry.prototype",
        index: FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "FinalizationRegistry",
        index: FINALIZATION_REGISTRY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "WeakSet.prototype",
        index: WEAK_SET_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "WeakSet",
        index: WEAK_SET_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.ZonedDateTime.prototype",
        index: TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.ZonedDateTime",
        index: TEMPORAL_ZONED_DATE_TIME_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Intl",
        index: INTL_OBJECT_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Intl.Locale.prototype",
        index: INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Intl.Locale",
        index: INTL_LOCALE_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "unhandled_rejection_head",
        index: PROMISE_UNHANDLED_REJECTION_HEAD_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "unhandled_rejection_tail",
        index: PROMISE_UNHANDLED_REJECTION_TAIL_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainDate.prototype",
        index: TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainDate",
        index: TEMPORAL_PLAIN_DATE_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.Duration.prototype",
        index: TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.Duration",
        index: TEMPORAL_DURATION_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainTime.prototype",
        index: TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainTime",
        index: TEMPORAL_PLAIN_TIME_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainDateTime.prototype",
        index: TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainDateTime",
        index: TEMPORAL_PLAIN_DATE_TIME_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainYearMonth.prototype",
        index: TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainYearMonth",
        index: TEMPORAL_PLAIN_YEAR_MONTH_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainMonthDay.prototype",
        index: TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Temporal.PlainMonthDay",
        index: TEMPORAL_PLAIN_MONTH_DAY_CONSTRUCTOR_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Intl.DateTimeFormat.prototype",
        index: INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
    },
    GlobalIndexSlot {
        name: "Intl.DateTimeFormat",
        index: INTL_DATE_TIME_FORMAT_CONSTRUCTOR_GLOBAL_INDEX,
    },
    // Last, and it must stay last while it holds the highest index: this
    // registry is asserted to be dense with `index == position`, and
    // `emit.rs` emits `GLOBAL_INDEX_REGISTRY.len()` globals. Its sibling
    // `throw_error_name_heap` sits at index 64 and cannot be joined here
    // without renumbering 70 globals.
    GlobalIndexSlot {
        name: "throw_error_message_heap",
        index: THROW_ERROR_MESSAGE_HEAP_GLOBAL_INDEX,
    },
];

/// Maps a global-object property name to the canonical function-object global
/// that must back both it and the matching `Number.*` static, so the two share
/// one identity within a realm.
pub(crate) fn canonical_host_function_global_index_by_name(name: &str) -> Option<u32> {
    match name {
        "parseInt" => Some(PARSE_INT_FUNCTION_GLOBAL_INDEX),
        "parseFloat" => Some(PARSE_FLOAT_FUNCTION_GLOBAL_INDEX),
        _ => None,
    }
}

pub(crate) fn standard_builtin_constructor_global_index(builtin: StandardBuiltinId) -> Option<u32> {
    match builtin {
        StandardBuiltinId::FunctionConstructor => Some(FUNCTION_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::PromiseConstructor => Some(PROMISE_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::MapConstructor => Some(MAP_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::WeakMapConstructor => Some(WEAK_MAP_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::WeakSetConstructor => Some(WEAK_SET_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::WeakRefConstructor => Some(WEAK_REF_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::FinalizationRegistryConstructor => {
            Some(FINALIZATION_REGISTRY_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::SetConstructor => Some(SET_CONSTRUCTOR_GLOBAL_INDEX),
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
        StandardBuiltinId::TemporalInstantConstructor => {
            Some(TEMPORAL_INSTANT_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainDateConstructor => {
            Some(TEMPORAL_PLAIN_DATE_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalDurationConstructor => {
            Some(TEMPORAL_DURATION_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainTimeConstructor => {
            Some(TEMPORAL_PLAIN_TIME_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainDateTimeConstructor => {
            Some(TEMPORAL_PLAIN_DATE_TIME_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainYearMonthConstructor => {
            Some(TEMPORAL_PLAIN_YEAR_MONTH_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainMonthDayConstructor => {
            Some(TEMPORAL_PLAIN_MONTH_DAY_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalZonedDateTimeConstructor => {
            Some(TEMPORAL_ZONED_DATE_TIME_CONSTRUCTOR_GLOBAL_INDEX)
        }
        StandardBuiltinId::IntlLocaleConstructor => Some(INTL_LOCALE_CONSTRUCTOR_GLOBAL_INDEX),
        StandardBuiltinId::IntlDateTimeFormatConstructor => {
            Some(INTL_DATE_TIME_FORMAT_CONSTRUCTOR_GLOBAL_INDEX)
        }
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
        | StandardBuiltinId::RegExpPrototypeCompile
        | StandardBuiltinId::RegExpPrototypeExec
        | StandardBuiltinId::RegExpPrototypeTest
        | StandardBuiltinId::RegExpPrototypeToString
        | StandardBuiltinId::EvalFunction
        | StandardBuiltinId::StringFromCharCode
        | StandardBuiltinId::StringFromCodePoint
        | StandardBuiltinId::StringRaw
        | StandardBuiltinId::StringPrototypeToString
        | StandardBuiltinId::StringPrototypeValueOf
        | StandardBuiltinId::StringPrototypeCharAt
        | StandardBuiltinId::StringPrototypeConcat
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
        | StandardBuiltinId::StringPrototypeNormalize
        | StandardBuiltinId::StringPrototypeLocaleCompare
        | StandardBuiltinId::StringPrototypeIterator
        | StandardBuiltinId::StringPrototypeToLocaleLowerCase
        | StandardBuiltinId::StringPrototypeToLocaleUpperCase
        | StandardBuiltinId::StringPrototypeToLowerCase
        | StandardBuiltinId::StringPrototypeToUpperCase
        | StandardBuiltinId::StringPrototypeTrim
        | StandardBuiltinId::StringPrototypeTrimStart
        | StandardBuiltinId::StringPrototypeTrimEnd
        | StandardBuiltinId::StringPrototypeIsWellFormed
        | StandardBuiltinId::StringPrototypeToWellFormed
        | StandardBuiltinId::DateNow
        | StandardBuiltinId::DateParse
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
        | StandardBuiltinId::DatePrototypeToIsoString
        | StandardBuiltinId::DatePrototypeToJson
        | StandardBuiltinId::DatePrototypeToPrimitive
        | StandardBuiltinId::DatePrototypeToDateString
        | StandardBuiltinId::DatePrototypeToLocaleDateString
        | StandardBuiltinId::DatePrototypeToLocaleString
        | StandardBuiltinId::DatePrototypeToLocaleTimeString
        | StandardBuiltinId::DatePrototypeToTemporalInstant
        | StandardBuiltinId::DatePrototypeToTimeString
        | StandardBuiltinId::DatePrototypeToString
        | StandardBuiltinId::DatePrototypeToUtcString
        | StandardBuiltinId::RegExpLegacyStaticGetter
        | StandardBuiltinId::RegExpLegacyStaticSetter
        | StandardBuiltinId::RegExpSpeciesGetter
        | StandardBuiltinId::RegExpPrototypeFlagsGetter
        | StandardBuiltinId::RegExpPrototypeSourceGetter
        | StandardBuiltinId::RegExpPrototypeHasIndicesGetter
        | StandardBuiltinId::RegExpPrototypeGlobalGetter
        | StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter
        | StandardBuiltinId::RegExpPrototypeMultilineGetter
        | StandardBuiltinId::RegExpPrototypeDotAllGetter
        | StandardBuiltinId::RegExpPrototypeUnicodeGetter
        | StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter
        | StandardBuiltinId::RegExpPrototypeStickyGetter
        | StandardBuiltinId::RegExpPrototypeSymbolMatch
        | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
        | StandardBuiltinId::RegExpPrototypeSymbolReplace
        | StandardBuiltinId::RegExpPrototypeSymbolSearch
        | StandardBuiltinId::RegExpPrototypeSymbolSplit
        | StandardBuiltinId::RegExpEscape
        | StandardBuiltinId::ObjectCreate
        | StandardBuiltinId::ObjectGetPrototypeOf
        | StandardBuiltinId::ObjectSetPrototypeOf
        | StandardBuiltinId::ObjectDefineProperty
        | StandardBuiltinId::ObjectDefineProperties
        | StandardBuiltinId::ObjectGetOwnPropertyDescriptor
        | StandardBuiltinId::ObjectGetOwnPropertyDescriptors
        | StandardBuiltinId::ObjectAssign
        | StandardBuiltinId::ObjectGetOwnPropertyNames
        | StandardBuiltinId::ObjectGetOwnPropertySymbols
        | StandardBuiltinId::ObjectKeys
        | StandardBuiltinId::ObjectValues
        | StandardBuiltinId::ObjectEntries
        | StandardBuiltinId::ObjectHasOwn
        | StandardBuiltinId::ObjectIs
        | StandardBuiltinId::ObjectIsSealed
        | StandardBuiltinId::ObjectIsFrozen
        | StandardBuiltinId::ObjectSeal
        | StandardBuiltinId::ObjectFreeze
        | StandardBuiltinId::ObjectIsExtensible
        | StandardBuiltinId::ObjectPreventExtensions
        | StandardBuiltinId::ObjectPrototypeHasOwnProperty
        | StandardBuiltinId::ObjectPrototypeLookupGetter
        | StandardBuiltinId::ObjectPrototypeLookupSetter
        | StandardBuiltinId::ObjectPrototypeProtoGetter
        | StandardBuiltinId::ObjectPrototypeProtoSetter
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
        | StandardBuiltinId::ArrayFromAsync
        | StandardBuiltinId::ArrayFromAsyncFulfilled
        | StandardBuiltinId::ArrayFromAsyncRejected
        | StandardBuiltinId::ArrayOf
        | StandardBuiltinId::ArrayIsArray
        | StandardBuiltinId::ArraySpeciesGetter
        | StandardBuiltinId::TypedArraySpeciesGetter
        | StandardBuiltinId::ArrayPrototypeConcat
        | StandardBuiltinId::ArrayPrototypeJoin
        | StandardBuiltinId::ArrayPrototypeSlice
        | StandardBuiltinId::ArrayPrototypeSplice
        | StandardBuiltinId::ArrayPrototypeSort
        | StandardBuiltinId::ArrayPrototypeToLocaleString
        | StandardBuiltinId::ArrayPrototypeFlat
        | StandardBuiltinId::ArrayPrototypeFlatMap
        | StandardBuiltinId::ArrayPrototypeAt
        | StandardBuiltinId::ArrayPrototypeToReversed
        | StandardBuiltinId::ArrayPrototypeToSpliced
        | StandardBuiltinId::ArrayPrototypeToSorted
        | StandardBuiltinId::ArrayPrototypeWith
        | StandardBuiltinId::ArrayPrototypeReverse
        | StandardBuiltinId::ArrayPrototypeCopyWithin
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
        | StandardBuiltinId::ArrayPrototypeReduce
        | StandardBuiltinId::ArrayPrototypeReduceRight
        | StandardBuiltinId::ArrayPrototypePop
        | StandardBuiltinId::ArrayPrototypePush
        | StandardBuiltinId::ArrayPrototypeShift
        | StandardBuiltinId::ArrayPrototypeUnshift
        | StandardBuiltinId::ArrayPrototypeFill
        | StandardBuiltinId::ArrayPrototypeKeys
        | StandardBuiltinId::ArrayPrototypeEntries
        | StandardBuiltinId::ArrayPrototypeValues
        | StandardBuiltinId::ArrayIteratorNext
        | StandardBuiltinId::ArrayIteratorIdentity
        | StandardBuiltinId::StringIteratorNext
        | StandardBuiltinId::GeneratorPrototypeNext
        | StandardBuiltinId::GeneratorPrototypeReturn
        | StandardBuiltinId::GeneratorPrototypeThrow
        | StandardBuiltinId::AsyncGeneratorPrototypeNext
        | StandardBuiltinId::AsyncGeneratorPrototypeReturn
        | StandardBuiltinId::AsyncGeneratorPrototypeThrow
        | StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose
        | StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeFulfilled
        | StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeRejected
        | StandardBuiltinId::IteratorFrom
        | StandardBuiltinId::IteratorConcat
        | StandardBuiltinId::IteratorConcatNext
        | StandardBuiltinId::IteratorConcatReturn
        | StandardBuiltinId::IteratorZip
        | StandardBuiltinId::IteratorZipKeyed
        | StandardBuiltinId::IteratorZipNext
        | StandardBuiltinId::IteratorZipReturn
        | StandardBuiltinId::IteratorHelperNext
        | StandardBuiltinId::IteratorHelperReturn
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
        | StandardBuiltinId::TypedArrayPrototypeToStringTagGetter
        | StandardBuiltinId::TypedArrayPrototypeToString
        | StandardBuiltinId::TypedArrayPrototypeAt
        | StandardBuiltinId::TypedArrayPrototypeIncludes
        | StandardBuiltinId::TypedArrayPrototypeIndexOf
        | StandardBuiltinId::TypedArrayPrototypeLastIndexOf
        | StandardBuiltinId::TypedArrayPrototypeFind
        | StandardBuiltinId::TypedArrayPrototypeFindIndex
        | StandardBuiltinId::TypedArrayPrototypeFindLast
        | StandardBuiltinId::TypedArrayPrototypeFindLastIndex
        | StandardBuiltinId::TypedArrayPrototypeEvery
        | StandardBuiltinId::TypedArrayPrototypeSome
        | StandardBuiltinId::TypedArrayPrototypeMap
        | StandardBuiltinId::TypedArrayPrototypeFilter
        | StandardBuiltinId::TypedArrayPrototypeForEach
        | StandardBuiltinId::TypedArrayPrototypeReduce
        | StandardBuiltinId::TypedArrayPrototypeReduceRight
        | StandardBuiltinId::TypedArrayPrototypeValues
        | StandardBuiltinId::TypedArrayPrototypeKeys
        | StandardBuiltinId::TypedArrayPrototypeEntries
        | StandardBuiltinId::TypedArrayPrototypeJoin
        | StandardBuiltinId::TypedArrayPrototypeToLocaleString
        | StandardBuiltinId::TypedArrayPrototypeSubarray
        | StandardBuiltinId::TypedArrayPrototypeSlice
        | StandardBuiltinId::TypedArrayPrototypeSet
        | StandardBuiltinId::TypedArrayPrototypeReverse
        | StandardBuiltinId::TypedArrayPrototypeCopyWithin
        | StandardBuiltinId::TypedArrayPrototypeSort
        | StandardBuiltinId::TypedArrayPrototypeToReversed
        | StandardBuiltinId::TypedArrayPrototypeToSorted
        | StandardBuiltinId::TypedArrayPrototypeWith
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
        | StandardBuiltinId::EncodeUri
        | StandardBuiltinId::EncodeUriComponent
        | StandardBuiltinId::DecodeUri
        | StandardBuiltinId::DecodeUriComponent
        | StandardBuiltinId::SymbolFor
        | StandardBuiltinId::SymbolKeyFor
        | StandardBuiltinId::SymbolPrototypeDescriptionGetter
        | StandardBuiltinId::SymbolPrototypeToString
        | StandardBuiltinId::SymbolPrototypeValueOf
        | StandardBuiltinId::SymbolPrototypeToPrimitive
        | StandardBuiltinId::PromisePrototypeThen
        | StandardBuiltinId::PromisePrototypeCatch
        | StandardBuiltinId::PromisePrototypeFinally
        | StandardBuiltinId::PromiseThenFinally
        | StandardBuiltinId::PromiseCatchFinally
        | StandardBuiltinId::PromiseValueThunk
        | StandardBuiltinId::PromiseThrower
        | StandardBuiltinId::PromiseSpeciesGetter
        | StandardBuiltinId::MapSpeciesGetter
        | StandardBuiltinId::SetSpeciesGetter
        | StandardBuiltinId::PromiseResolve
        | StandardBuiltinId::PromiseWithResolvers
        | StandardBuiltinId::PromiseTry
        | StandardBuiltinId::PromiseReject
        | StandardBuiltinId::PromiseAll
        | StandardBuiltinId::PromiseAllSettled
        | StandardBuiltinId::PromiseAllKeyed
        | StandardBuiltinId::PromiseAllSettledKeyed
        | StandardBuiltinId::PromiseAny
        | StandardBuiltinId::PromiseRace
        | StandardBuiltinId::PromiseAllResolveElement
        | StandardBuiltinId::PromiseAllSettledResolveElement
        | StandardBuiltinId::PromiseAllSettledRejectElement
        | StandardBuiltinId::PromiseAnyRejectElement
        | StandardBuiltinId::PromiseAllKeyedResolveElement
        | StandardBuiltinId::PromiseAllSettledKeyedResolveElement
        | StandardBuiltinId::PromiseAllSettledKeyedRejectElement
        | StandardBuiltinId::PromiseCapabilityExecutor
        | StandardBuiltinId::PromiseResolveFunction
        | StandardBuiltinId::PromiseRejectFunction
        | StandardBuiltinId::MapGroupBy
        | StandardBuiltinId::ObjectGroupBy
        | StandardBuiltinId::ObjectFromEntries
        | StandardBuiltinId::MapPrototypeClear
        | StandardBuiltinId::MapPrototypeDelete
        | StandardBuiltinId::MapPrototypeForEach
        | StandardBuiltinId::MapPrototypeKeys
        | StandardBuiltinId::MapPrototypeValues
        | StandardBuiltinId::MapPrototypeEntries
        | StandardBuiltinId::MapIteratorNext
        | StandardBuiltinId::MapPrototypeGet
        | StandardBuiltinId::MapPrototypeGetOrInsert
        | StandardBuiltinId::MapPrototypeGetOrInsertComputed
        | StandardBuiltinId::MapPrototypeHas
        | StandardBuiltinId::MapPrototypeSet
        | StandardBuiltinId::MapPrototypeSizeGetter
        | StandardBuiltinId::WeakMapPrototypeDelete
        | StandardBuiltinId::WeakMapPrototypeGet
        | StandardBuiltinId::WeakMapPrototypeGetOrInsert
        | StandardBuiltinId::WeakMapPrototypeGetOrInsertComputed
        | StandardBuiltinId::WeakMapPrototypeHas
        | StandardBuiltinId::WeakMapPrototypeSet
        | StandardBuiltinId::WeakSetPrototypeAdd
        | StandardBuiltinId::WeakSetPrototypeDelete
        | StandardBuiltinId::WeakSetPrototypeHas
        | StandardBuiltinId::SetPrototypeAdd
        | StandardBuiltinId::SetPrototypeClear
        | StandardBuiltinId::SetPrototypeDelete
        | StandardBuiltinId::SetPrototypeDifference
        | StandardBuiltinId::SetPrototypeForEach
        | StandardBuiltinId::SetPrototypeIntersection
        | StandardBuiltinId::SetPrototypeIsDisjointFrom
        | StandardBuiltinId::SetPrototypeIsSubsetOf
        | StandardBuiltinId::SetPrototypeIsSupersetOf
        | StandardBuiltinId::SetPrototypeSymmetricDifference
        | StandardBuiltinId::SetPrototypeUnion
        | StandardBuiltinId::SetPrototypeValues
        | StandardBuiltinId::SetPrototypeEntries
        | StandardBuiltinId::SetIteratorNext
        | StandardBuiltinId::SetPrototypeHas
        | StandardBuiltinId::SetPrototypeSizeGetter
        | StandardBuiltinId::TemporalPlainDateFrom
        | StandardBuiltinId::TemporalPlainDateCompare
        | StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeEraGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDayGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeWith
        | StandardBuiltinId::TemporalPlainDatePrototypeWithCalendar
        | StandardBuiltinId::TemporalPlainDatePrototypeEquals
        | StandardBuiltinId::TemporalPlainDatePrototypeToString
        | StandardBuiltinId::TemporalPlainDatePrototypeToJson
        | StandardBuiltinId::TemporalPlainDatePrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainDatePrototypeValueOf
        | StandardBuiltinId::TemporalPlainDatePrototypeAdd
        | StandardBuiltinId::TemporalPlainDatePrototypeSubtract
        | StandardBuiltinId::TemporalPlainDatePrototypeUntil
        | StandardBuiltinId::TemporalPlainDatePrototypeSince
        | StandardBuiltinId::TemporalPlainDatePrototypeToPlainDateTime
        | StandardBuiltinId::TemporalPlainDatePrototypeToPlainYearMonth
        | StandardBuiltinId::TemporalPlainDatePrototypeToPlainMonthDay
        | StandardBuiltinId::TemporalPlainYearMonthFrom
        | StandardBuiltinId::TemporalPlainYearMonthCompare
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInMonthGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthsInYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeInLeapYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeWith
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeAdd
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeSubtract
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeUntil
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeSince
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeEquals
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeToString
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeToJson
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeValueOf
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeToPlainDate
        | StandardBuiltinId::TemporalPlainMonthDayFrom
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeDayGetter
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeWith
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeEquals
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeToString
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeToJson
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeValueOf
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeToPlainDate
        | StandardBuiltinId::TemporalDurationFrom
        | StandardBuiltinId::TemporalDurationCompare
        | StandardBuiltinId::TemporalDurationPrototypeYearsGetter
        | StandardBuiltinId::TemporalDurationPrototypeMonthsGetter
        | StandardBuiltinId::TemporalDurationPrototypeWeeksGetter
        | StandardBuiltinId::TemporalDurationPrototypeDaysGetter
        | StandardBuiltinId::TemporalDurationPrototypeHoursGetter
        | StandardBuiltinId::TemporalDurationPrototypeMinutesGetter
        | StandardBuiltinId::TemporalDurationPrototypeSecondsGetter
        | StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter
        | StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter
        | StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter
        | StandardBuiltinId::TemporalDurationPrototypeSignGetter
        | StandardBuiltinId::TemporalDurationPrototypeBlankGetter
        | StandardBuiltinId::TemporalDurationPrototypeWith
        | StandardBuiltinId::TemporalDurationPrototypeNegated
        | StandardBuiltinId::TemporalDurationPrototypeAbs
        | StandardBuiltinId::TemporalDurationPrototypeAdd
        | StandardBuiltinId::TemporalDurationPrototypeSubtract
        | StandardBuiltinId::TemporalDurationPrototypeRound
        | StandardBuiltinId::TemporalDurationPrototypeTotal
        | StandardBuiltinId::TemporalDurationPrototypeToString
        | StandardBuiltinId::TemporalDurationPrototypeToJson
        | StandardBuiltinId::TemporalDurationPrototypeToLocaleString
        | StandardBuiltinId::TemporalDurationPrototypeValueOf
        | StandardBuiltinId::TemporalPlainTimeFrom
        | StandardBuiltinId::TemporalPlainTimeCompare
        | StandardBuiltinId::TemporalPlainTimePrototypeHourGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeMinuteGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeSecondGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeMillisecondGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeMicrosecondGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeNanosecondGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeWith
        | StandardBuiltinId::TemporalPlainTimePrototypeAdd
        | StandardBuiltinId::TemporalPlainTimePrototypeSubtract
        | StandardBuiltinId::TemporalPlainTimePrototypeUntil
        | StandardBuiltinId::TemporalPlainTimePrototypeSince
        | StandardBuiltinId::TemporalPlainTimePrototypeRound
        | StandardBuiltinId::TemporalPlainTimePrototypeEquals
        | StandardBuiltinId::TemporalPlainTimePrototypeToString
        | StandardBuiltinId::TemporalPlainTimePrototypeToJson
        | StandardBuiltinId::TemporalPlainTimePrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainTimePrototypeValueOf
        | StandardBuiltinId::TemporalPlainDateTimeFrom
        | StandardBuiltinId::TemporalPlainDateTimeCompare
        | StandardBuiltinId::TemporalPlainDateTimePrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeEraGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeEraYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDayGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeHourGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMinuteGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeSecondGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMillisecondGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMicrosecondGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeNanosecondGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfWeekGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeWeekOfYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeYearOfWeekGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInWeekGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInMonthGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthsInYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeInLeapYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeWith
        | StandardBuiltinId::TemporalPlainDateTimePrototypeWithPlainTime
        | StandardBuiltinId::TemporalPlainDateTimePrototypeWithCalendar
        | StandardBuiltinId::TemporalPlainDateTimePrototypeAdd
        | StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract
        | StandardBuiltinId::TemporalPlainDateTimePrototypeUntil
        | StandardBuiltinId::TemporalPlainDateTimePrototypeSince
        | StandardBuiltinId::TemporalPlainDateTimePrototypeRound
        | StandardBuiltinId::TemporalPlainDateTimePrototypeEquals
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToString
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToJson
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainDateTimePrototypeValueOf
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainDate
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainTime
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime
        | StandardBuiltinId::TemporalNowInstant
        | StandardBuiltinId::TemporalNowTimeZoneId
        | StandardBuiltinId::TemporalNowZonedDateTimeIso
        | StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter
        | StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter
        | StandardBuiltinId::TemporalInstantPrototypeEquals
        | StandardBuiltinId::TemporalInstantFrom
        | StandardBuiltinId::TemporalInstantCompare
        | StandardBuiltinId::TemporalInstantFromEpochMilliseconds
        | StandardBuiltinId::TemporalInstantFromEpochNanoseconds
        | StandardBuiltinId::TemporalInstantPrototypeToString
        | StandardBuiltinId::TemporalInstantPrototypeToJson
        | StandardBuiltinId::TemporalInstantPrototypeValueOf
        | StandardBuiltinId::TemporalZonedDateTimeFrom
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEraGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEraYearGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeYearGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeDayGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeHourGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMinuteGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeSecondGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMillisecondGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMicrosecondGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeNanosecondGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEquals
        | StandardBuiltinId::TemporalZonedDateTimePrototypeToInstant
        | StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime
        | StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone
        | StandardBuiltinId::IntlGetCanonicalLocales
        | StandardBuiltinId::IntlLocalePrototypeLanguageGetter
        | StandardBuiltinId::IntlLocalePrototypeScriptGetter
        | StandardBuiltinId::IntlLocalePrototypeRegionGetter
        | StandardBuiltinId::IntlLocalePrototypeBaseNameGetter
        | StandardBuiltinId::IntlLocalePrototypeToString
        | StandardBuiltinId::IntlDateTimeFormatSupportedLocalesOf
        | StandardBuiltinId::IntlDateTimeFormatPrototypeResolvedOptions
        | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter
        | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatToParts
        | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRange
        | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRangeToParts
        | StandardBuiltinId::IntlDateTimeFormatBoundFormat
        | StandardBuiltinId::WeakRefPrototypeDeref
        | StandardBuiltinId::FinalizationRegistryPrototypeRegister
        | StandardBuiltinId::FinalizationRegistryPrototypeUnregister => None,
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
        .find(|builtin| *builtin != HostBuiltinId::HTMLDDA && builtin.as_str() == name)
}

pub(crate) fn all_host_builtins() -> &'static [HostBuiltinId] {
    &[
        HostBuiltinId::Print,
        HostBuiltinId::Gc,
        HostBuiltinId::AssertThrows,
        HostBuiltinId::IsConstructor,
        HostBuiltinId::CreateRealm,
        HostBuiltinId::CreateHTMLDDA,
        HostBuiltinId::HTMLDDA,
        HostBuiltinId::ParseInt,
        HostBuiltinId::ParseFloat,
        HostBuiltinId::DetachArrayBuffer,
        HostBuiltinId::AgentStart,
        HostBuiltinId::AgentBroadcast,
        HostBuiltinId::AgentReceiveBroadcast,
        HostBuiltinId::AgentReport,
        HostBuiltinId::AgentGetReport,
        HostBuiltinId::AgentSleep,
        HostBuiltinId::AgentMonotonicNow,
        HostBuiltinId::AgentLeaving,
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

pub(crate) fn typed_array_realm_intrinsics_prototype_offset(
    builtin: StandardBuiltinId,
) -> Option<u64> {
    Some(match builtin {
        StandardBuiltinId::Float64ArrayConstructor => {
            HEAP_REALM_INTRINSICS_FLOAT64_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Float32ArrayConstructor => {
            HEAP_REALM_INTRINSICS_FLOAT32_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Int32ArrayConstructor => {
            HEAP_REALM_INTRINSICS_INT32_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Int16ArrayConstructor => {
            HEAP_REALM_INTRINSICS_INT16_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Int8ArrayConstructor => {
            HEAP_REALM_INTRINSICS_INT8_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Uint32ArrayConstructor => {
            HEAP_REALM_INTRINSICS_UINT32_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Uint16ArrayConstructor => {
            HEAP_REALM_INTRINSICS_UINT16_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Uint8ArrayConstructor => {
            HEAP_REALM_INTRINSICS_UINT8_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::Uint8ClampedArrayConstructor => {
            HEAP_REALM_INTRINSICS_UINT8_CLAMPED_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::BigInt64ArrayConstructor => {
            HEAP_REALM_INTRINSICS_BIGINT64_ARRAY_PROTOTYPE_OFFSET
        }
        StandardBuiltinId::BigUint64ArrayConstructor => {
            HEAP_REALM_INTRINSICS_BIGUINT64_ARRAY_PROTOTYPE_OFFSET
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

pub(crate) const fn throw_error_message_global_index(uses_heap: bool) -> u32 {
    if uses_heap {
        THROW_ERROR_MESSAGE_HEAP_GLOBAL_INDEX
    } else {
        THROW_ERROR_MESSAGE_NO_HEAP_GLOBAL_INDEX
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
        StandardBuiltinId::PromiseConstructor => Some(PROMISE_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::MapConstructor => Some(MAP_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::WeakMapConstructor => Some(WEAK_MAP_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::WeakSetConstructor => Some(WEAK_SET_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::WeakRefConstructor => Some(WEAK_REF_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::FinalizationRegistryConstructor => {
            Some(FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::SetConstructor => Some(SET_PROTOTYPE_GLOBAL_INDEX),
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
        StandardBuiltinId::TemporalInstantConstructor => {
            Some(TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalZonedDateTimeConstructor => {
            Some(TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainDateConstructor => {
            Some(TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalDurationConstructor => {
            Some(TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainTimeConstructor => {
            Some(TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainDateTimeConstructor => {
            Some(TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainYearMonthConstructor => {
            Some(TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::TemporalPlainMonthDayConstructor => {
            Some(TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_GLOBAL_INDEX)
        }
        StandardBuiltinId::IntlLocaleConstructor => Some(INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX),
        StandardBuiltinId::IntlDateTimeFormatConstructor => {
            Some(INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX)
        }
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
    /// Per-function attribution for every body in `bytes`, in code-section
    /// order, always populated.
    ///
    /// Typed rather than parsed back out of `debug_dump`: the dump's full
    /// report is opt-in behind `PORFFOR_EMIT_SIZE_REPORT` and its only printer
    /// lives two crates away, so "how big is `js::probe#f0`?" was a question the
    /// compiler could answer but no test could ask. It is derived from the same
    /// single [`crate::emitted_function::ModuleFunctionTable::summaries`] call
    /// that renders the `largest emitted function:` line, so the two cannot
    /// disagree.
    pub function_sizes: Vec<EmittedFunctionSummary>,
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

    /// A function body exceeded the configured per-function budget.
    ///
    /// The constructor takes the [`FunctionIdentity`] rather than a name or a
    /// bare size, so this diagnostic cannot be produced without knowing which
    /// function it is about — precisely what the `[origin:unknown] ... Code for
    /// function is too large` failure lacks. The budget arrives as a
    /// [`FunctionBodyBudget`], validated once at construction, so there is no
    /// bare `u32` threshold to mis-thread.
    pub(crate) fn function_too_large(
        identity: &FunctionIdentity,
        body_bytes: FunctionBodySize,
        budget: FunctionBodyBudget,
    ) -> Self {
        Self {
            message: format!(
                "emitted function body exceeds the configured budget: {} ({}) is {} against a budget of {}",
                identity.wasm_name(),
                identity.category(),
                body_bytes,
                budget
            ),
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
        assert_eq!(
            THROW_ERROR_MESSAGE_NO_HEAP_GLOBAL_INDEX, HEAP_PTR_GLOBAL_INDEX,
            "no-heap throw-error-message export aliases the same slot as its name sibling; a \
             heap-less module cannot produce an object completion, so neither is ever read"
        );
        assert_eq!(
            GLOBAL_INDEX_REGISTRY.len(),
            THROW_ERROR_MESSAGE_HEAP_GLOBAL_INDEX as usize + 1,
            "the registry length tracks the highest index, and `emit.rs` emits exactly this many \
             globals for a heap module"
        );
        assert!(
            THROW_ERROR_MESSAGE_HEAP_GLOBAL_INDEX > INTL_DATE_TIME_FORMAT_CONSTRUCTOR_GLOBAL_INDEX,
            "the throw-error-message slot was appended after the previous maximum precisely so no \
             existing global index moved"
        );
    }

    #[test]
    fn iterator_concat_builtins_use_runtime_function_objects() {
        for builtin in [
            StandardBuiltinId::IteratorConcat,
            StandardBuiltinId::IteratorConcatNext,
            StandardBuiltinId::IteratorConcatReturn,
        ] {
            assert_eq!(standard_builtin_constructor_global_index(builtin), None);
            assert_eq!(standard_builtin_function_global_index(builtin), None);
        }
    }
}
