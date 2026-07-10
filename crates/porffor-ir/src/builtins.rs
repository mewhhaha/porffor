use crate::{
    FunctionId, AGGREGATE_ERROR_NAME, ARRAY_BUFFER_NAME, ARRAY_NAME, ASSERT_THROWS_NAME,
    BIGINT64_ARRAY_NAME, BIGINT_NAME, BIGUINT64_ARRAY_NAME, BOOLEAN_NAME, SYMBOL_NAME,
    BUILTIN_SYMBOL_FUNCTION_ID, BUILTIN_SYMBOL_FOR_FUNCTION_ID, BUILTIN_SYMBOL_KEY_FOR_FUNCTION_ID,
    BUILTIN_AGGREGATE_ERROR_FUNCTION_ID, BUILTIN_ARRAY_BUFFER_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_IS_VIEW_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_DETACHED_GETTER_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZABLE_GETTER_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZE_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_TO_IMMUTABLE_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_FIXED_LENGTH_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_IMMUTABLE_FUNCTION_ID,
    BUILTIN_ARRAY_BUFFER_SPECIES_GETTER_FUNCTION_ID, BUILTIN_ARRAY_FROM_FUNCTION_ID,
    BUILTIN_ARRAY_FUNCTION_ID, BUILTIN_ARRAY_IS_ARRAY_FUNCTION_ID,
    BUILTIN_ARRAY_ITERATOR_IDENTITY_FUNCTION_ID, BUILTIN_ARRAY_ITERATOR_NEXT_FUNCTION_ID,
    BUILTIN_ARRAY_OF_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_AT_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_CONCAT_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_ENTRIES_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_EVERY_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_FILTER_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_FIND_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_FIND_INDEX_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_INDEX_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_FLAT_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_FLAT_MAP_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_FOR_EACH_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_INCLUDES_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_INDEX_OF_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_KEYS_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_MAP_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_POP_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_PUSH_FUNCTION_ID, BUILTIN_ARRAY_PROTOTYPE_SOME_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
    BUILTIN_ARRAY_PROTOTYPE_VALUES_FUNCTION_ID, BUILTIN_ARRAY_SPECIES_GETTER_FUNCTION_ID,
    BUILTIN_ATOMICS_ADD_FUNCTION_ID, BUILTIN_ATOMICS_AND_FUNCTION_ID,
    BUILTIN_ATOMICS_COMPARE_EXCHANGE_FUNCTION_ID, BUILTIN_ATOMICS_EXCHANGE_FUNCTION_ID,
    BUILTIN_ATOMICS_IS_LOCK_FREE_FUNCTION_ID, BUILTIN_ATOMICS_LOAD_FUNCTION_ID,
    BUILTIN_ATOMICS_NOTIFY_FUNCTION_ID, BUILTIN_ATOMICS_OR_FUNCTION_ID,
    BUILTIN_ATOMICS_PAUSE_FUNCTION_ID, BUILTIN_ATOMICS_STORE_FUNCTION_ID,
    BUILTIN_ATOMICS_SUB_FUNCTION_ID, BUILTIN_ATOMICS_WAIT_ASYNC_FUNCTION_ID,
    BUILTIN_ATOMICS_WAIT_FUNCTION_ID, BUILTIN_ATOMICS_XOR_FUNCTION_ID,
    BUILTIN_BIGINT64_ARRAY_FUNCTION_ID, BUILTIN_BIGINT_AS_INT_N_FUNCTION_ID,
    BUILTIN_BIGINT_AS_UINT_N_FUNCTION_ID, BUILTIN_BIGINT_FUNCTION_ID,
    BUILTIN_BIGINT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
    BUILTIN_BIGINT_PROTOTYPE_TO_STRING_FUNCTION_ID, BUILTIN_BIGINT_PROTOTYPE_VALUE_OF_FUNCTION_ID,
    BUILTIN_BIGUINT64_ARRAY_FUNCTION_ID, BUILTIN_BOOLEAN_FUNCTION_ID,
    BUILTIN_BOUND_FUNCTION_INVOKER_FUNCTION_ID, BUILTIN_DATA_VIEW_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGINT64_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGUINT64_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT16_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT32_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT64_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT16_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT32_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT8_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT16_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT32_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT8_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGINT64_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGUINT64_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT16_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT32_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT64_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT16_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT32_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT8_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT16_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT32_FUNCTION_ID,
    BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT8_FUNCTION_ID, BUILTIN_DATE_FUNCTION_ID,
    BUILTIN_DATE_NOW_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_GET_DATE_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_DAY_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_GET_FULL_YEAR_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_HOURS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_MILLISECONDS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_MINUTES_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_GET_MONTH_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_SECONDS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_TIMEZONE_OFFSET_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_TIME_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_GET_UTC_DATE_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_UTC_DAY_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_UTC_FULL_YEAR_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_UTC_HOURS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_UTC_MILLISECONDS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_UTC_MINUTES_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_UTC_MONTH_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_UTC_SECONDS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_GET_YEAR_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_SET_DATE_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_FULL_YEAR_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_SET_HOURS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_MILLISECONDS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_MINUTES_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_SET_MONTH_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_SECONDS_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_SET_TIME_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_UTC_DATE_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_UTC_FULL_YEAR_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_UTC_HOURS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_UTC_MILLISECONDS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_UTC_MINUTES_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_UTC_MONTH_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_UTC_SECONDS_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_SET_YEAR_FUNCTION_ID, BUILTIN_DATE_PROTOTYPE_TO_UTC_STRING_FUNCTION_ID,
    BUILTIN_DATE_PROTOTYPE_VALUE_OF_FUNCTION_ID, BUILTIN_DATE_UTC_FUNCTION_ID,
    BUILTIN_ERROR_FUNCTION_ID, BUILTIN_ERROR_IS_ERROR_FUNCTION_ID,
    BUILTIN_ERROR_PROTOTYPE_TO_STRING_FUNCTION_ID, BUILTIN_ESCAPE_FUNCTION_ID,
    BUILTIN_EVAL_ERROR_FUNCTION_ID, BUILTIN_EVAL_FUNCTION_ID, BUILTIN_FLOAT32_ARRAY_FUNCTION_ID,
    BUILTIN_FLOAT64_ARRAY_FUNCTION_ID, BUILTIN_FUNCTION_FUNCTION_ID,
    BUILTIN_FUNCTION_PROTOTYPE_APPLY_FUNCTION_ID, BUILTIN_FUNCTION_PROTOTYPE_BIND_FUNCTION_ID,
    BUILTIN_FUNCTION_PROTOTYPE_CALL_FUNCTION_ID, BUILTIN_FUNCTION_PROTOTYPE_TO_STRING_FUNCTION_ID,
    BUILTIN_INT16_ARRAY_FUNCTION_ID, BUILTIN_INT32_ARRAY_FUNCTION_ID,
    BUILTIN_INT8_ARRAY_FUNCTION_ID, BUILTIN_ITERATOR_DROP_NEXT_FUNCTION_ID,
    BUILTIN_ITERATOR_DROP_RETURN_FUNCTION_ID, BUILTIN_ITERATOR_FILTER_NEXT_FUNCTION_ID,
    BUILTIN_ITERATOR_FILTER_RETURN_FUNCTION_ID, BUILTIN_ITERATOR_FLAT_MAP_NEXT_FUNCTION_ID,
    BUILTIN_ITERATOR_FLAT_MAP_RETURN_FUNCTION_ID, BUILTIN_ITERATOR_FROM_FUNCTION_ID,
    BUILTIN_ITERATOR_FROM_WRAPPER_NEXT_FUNCTION_ID,
    BUILTIN_ITERATOR_FROM_WRAPPER_RETURN_FUNCTION_ID, BUILTIN_ITERATOR_FUNCTION_ID,
    BUILTIN_ITERATOR_MAP_NEXT_FUNCTION_ID, BUILTIN_ITERATOR_MAP_RETURN_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_GETTER_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_SETTER_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_DROP_FUNCTION_ID, BUILTIN_ITERATOR_PROTOTYPE_EVERY_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_FILTER_FUNCTION_ID, BUILTIN_ITERATOR_PROTOTYPE_FIND_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_FLAT_MAP_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_FOR_EACH_FUNCTION_ID, BUILTIN_ITERATOR_PROTOTYPE_MAP_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_REDUCE_FUNCTION_ID, BUILTIN_ITERATOR_PROTOTYPE_SOME_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_SYMBOL_DISPOSE_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_TAKE_FUNCTION_ID, BUILTIN_ITERATOR_PROTOTYPE_TO_ARRAY_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_GETTER_FUNCTION_ID,
    BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_SETTER_FUNCTION_ID,
    BUILTIN_ITERATOR_TAKE_NEXT_FUNCTION_ID, BUILTIN_ITERATOR_TAKE_RETURN_FUNCTION_ID,
    BUILTIN_JSON_IS_RAW_JSON_FUNCTION_ID, BUILTIN_JSON_PARSE_FUNCTION_ID,
    BUILTIN_JSON_RAW_JSON_FUNCTION_ID, BUILTIN_JSON_STRINGIFY_FUNCTION_ID,
    BUILTIN_NUMBER_FUNCTION_ID, BUILTIN_NUMBER_IS_INTEGER_FUNCTION_ID,
    BUILTIN_OBJECT_CREATE_FUNCTION_ID, BUILTIN_OBJECT_DEFINE_PROPERTIES_FUNCTION_ID,
    BUILTIN_OBJECT_DEFINE_PROPERTY_FUNCTION_ID, BUILTIN_OBJECT_FREEZE_FUNCTION_ID,
    BUILTIN_OBJECT_FUNCTION_ID, BUILTIN_OBJECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID,
    BUILTIN_OBJECT_GET_OWN_PROPERTY_NAMES_FUNCTION_ID,
    BUILTIN_OBJECT_GET_OWN_PROPERTY_SYMBOLS_FUNCTION_ID,
    BUILTIN_OBJECT_GET_PROTOTYPE_OF_FUNCTION_ID, BUILTIN_OBJECT_HAS_OWN_FUNCTION_ID,
    BUILTIN_OBJECT_IS_EXTENSIBLE_FUNCTION_ID, BUILTIN_OBJECT_IS_FROZEN_FUNCTION_ID,
    BUILTIN_OBJECT_IS_FUNCTION_ID, BUILTIN_OBJECT_IS_SEALED_FUNCTION_ID,
    BUILTIN_OBJECT_KEYS_FUNCTION_ID, BUILTIN_OBJECT_PREVENT_EXTENSIONS_FUNCTION_ID,
    BUILTIN_OBJECT_PROTOTYPE_HAS_OWN_PROPERTY_FUNCTION_ID,
    BUILTIN_OBJECT_PROTOTYPE_IS_PROTOTYPE_OF_FUNCTION_ID,
    BUILTIN_OBJECT_PROTOTYPE_PROPERTY_IS_ENUMERABLE_FUNCTION_ID,
    BUILTIN_OBJECT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
    BUILTIN_OBJECT_PROTOTYPE_TO_STRING_FUNCTION_ID, BUILTIN_OBJECT_PROTOTYPE_VALUE_OF_FUNCTION_ID,
    BUILTIN_OBJECT_SET_PROTOTYPE_OF_FUNCTION_ID, BUILTIN_OBJECT_VALUES_FUNCTION_ID,
    BUILTIN_PROXY_FUNCTION_ID, BUILTIN_PROXY_REVOCABLE_FUNCTION_ID,
    BUILTIN_PROXY_REVOKE_FUNCTION_ID, BUILTIN_RANGE_ERROR_FUNCTION_ID,
    BUILTIN_REFERENCE_ERROR_FUNCTION_ID, BUILTIN_REFLECT_APPLY_FUNCTION_ID,
    BUILTIN_REFLECT_CONSTRUCT_FUNCTION_ID, BUILTIN_REFLECT_DEFINE_PROPERTY_FUNCTION_ID,
    BUILTIN_REFLECT_DELETE_PROPERTY_FUNCTION_ID, BUILTIN_REFLECT_GET_FUNCTION_ID,
    BUILTIN_REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID,
    BUILTIN_REFLECT_GET_PROTOTYPE_OF_FUNCTION_ID, BUILTIN_REFLECT_HAS_FUNCTION_ID,
    BUILTIN_REFLECT_IS_EXTENSIBLE_FUNCTION_ID, BUILTIN_REFLECT_OWN_KEYS_FUNCTION_ID,
    BUILTIN_REFLECT_PREVENT_EXTENSIONS_FUNCTION_ID, BUILTIN_REFLECT_SET_FUNCTION_ID,
    BUILTIN_REFLECT_SET_PROTOTYPE_OF_FUNCTION_ID, BUILTIN_REGEXP_ESCAPE_FUNCTION_ID,
    BUILTIN_REGEXP_FUNCTION_ID, BUILTIN_REGEXP_LEGACY_STATIC_GETTER_FUNCTION_ID,
    BUILTIN_REGEXP_LEGACY_STATIC_SETTER_FUNCTION_ID,
    BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_FUNCTION_ID,
    BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_FUNCTION_ID,
    BUILTIN_REGEXP_PROTOTYPE_SYMBOL_SEARCH_FUNCTION_ID, BUILTIN_REGEXP_SPECIES_GETTER_FUNCTION_ID,
    BUILTIN_SHARED_ARRAY_BUFFER_FUNCTION_ID,
    BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID,
    BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROWABLE_GETTER_FUNCTION_ID,
    BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROW_FUNCTION_ID,
    BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID,
    BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID, BUILTIN_STRING_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_ANCHOR_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_AT_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_BIG_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_BLINK_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_BOLD_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_CHAR_AT_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_CHAR_CODE_AT_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_CODE_POINT_AT_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_ENDS_WITH_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_FIXED_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_FONTCOLOR_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_FONTSIZE_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_INCLUDES_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_INDEX_OF_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_IS_WELL_FORMED_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_ITALICS_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_LINK_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_MATCH_ALL_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_MATCH_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_PAD_END_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_PAD_START_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_REPEAT_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_REPLACE_ALL_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_REPLACE_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_SEARCH_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_SLICE_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_SMALL_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_SPLIT_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_STARTS_WITH_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_STRIKE_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_SUBSTRING_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_SUBSTR_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_SUB_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_SUP_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_TO_STRING_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_TO_UPPER_CASE_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_TO_WELL_FORMED_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_TRIM_END_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_TRIM_FUNCTION_ID,
    BUILTIN_STRING_PROTOTYPE_TRIM_START_FUNCTION_ID, BUILTIN_STRING_PROTOTYPE_VALUE_OF_FUNCTION_ID,
    BUILTIN_SUPPRESSED_ERROR_FUNCTION_ID, BUILTIN_SYNTAX_ERROR_FUNCTION_ID,
    BUILTIN_THROW_TYPE_ERROR_FUNCTION_ID, BUILTIN_TYPED_ARRAY_FROM_FUNCTION_ID,
    BUILTIN_TYPED_ARRAY_OF_FUNCTION_ID, BUILTIN_TYPED_ARRAY_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID,
    BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID,
    BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID,
    BUILTIN_TYPED_ARRAY_PROTOTYPE_LENGTH_GETTER_FUNCTION_ID,
    BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
    BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_STRING_FUNCTION_ID, BUILTIN_TYPE_ERROR_FUNCTION_ID,
    BUILTIN_UINT16_ARRAY_FUNCTION_ID, BUILTIN_UINT32_ARRAY_FUNCTION_ID,
    BUILTIN_UINT8_ARRAY_FUNCTION_ID, BUILTIN_UINT8_CLAMPED_ARRAY_FUNCTION_ID,
    BUILTIN_UNESCAPE_FUNCTION_ID, BUILTIN_URI_ERROR_FUNCTION_ID, CREATE_REALM_NAME, DATA_VIEW_NAME,
    DATE_NAME, DETACH_ARRAY_BUFFER_NAME, ERROR_NAME, ESCAPE_NAME, EVAL_ERROR_NAME,
    FLOAT32_ARRAY_NAME, FLOAT64_ARRAY_NAME, FUNCTION_NAME, GC_NAME, HOST_ASSERT_THROWS_FUNCTION_ID,
    HOST_CREATE_REALM_FUNCTION_ID, HOST_DETACH_ARRAY_BUFFER_FUNCTION_ID, HOST_GC_FUNCTION_ID,
    HOST_IS_CONSTRUCTOR_FUNCTION_ID, HOST_PARSE_FLOAT_FUNCTION_ID, HOST_PARSE_INT_FUNCTION_ID,
    HOST_PRINT_FUNCTION_ID, INT16_ARRAY_NAME, INT32_ARRAY_NAME, INT8_ARRAY_NAME,
    IS_CONSTRUCTOR_NAME, NUMBER_NAME, OBJECT_NAME, PARSE_FLOAT_NAME, PARSE_INT_NAME, PRINT_NAME,
    PROXY_NAME, RANGE_ERROR_NAME, REFERENCE_ERROR_NAME, REGEXP_NAME, SHARED_ARRAY_BUFFER_NAME,
    STRING_NAME, SUPPRESSED_ERROR_NAME, SYNTAX_ERROR_NAME, TYPE_ERROR_NAME, UINT16_ARRAY_NAME,
    UINT32_ARRAY_NAME, UINT8_ARRAY_NAME, UINT8_CLAMPED_ARRAY_NAME, UNESCAPE_NAME, URI_ERROR_NAME,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HostBuiltinId {
    Print,
    Gc,
    AssertThrows,
    IsConstructor,
    CreateRealm,
    ParseInt,
    ParseFloat,
    DetachArrayBuffer,
}

impl HostBuiltinId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Print => PRINT_NAME,
            Self::Gc => GC_NAME,
            Self::AssertThrows => ASSERT_THROWS_NAME,
            Self::IsConstructor => IS_CONSTRUCTOR_NAME,
            Self::CreateRealm => CREATE_REALM_NAME,
            Self::ParseInt => PARSE_INT_NAME,
            Self::ParseFloat => PARSE_FLOAT_NAME,
            Self::DetachArrayBuffer => DETACH_ARRAY_BUFFER_NAME,
        }
    }

    pub fn function_id(self) -> FunctionId {
        match self {
            Self::Print => HOST_PRINT_FUNCTION_ID.to_string(),
            Self::Gc => HOST_GC_FUNCTION_ID.to_string(),
            Self::AssertThrows => HOST_ASSERT_THROWS_FUNCTION_ID.to_string(),
            Self::IsConstructor => HOST_IS_CONSTRUCTOR_FUNCTION_ID.to_string(),
            Self::CreateRealm => HOST_CREATE_REALM_FUNCTION_ID.to_string(),
            Self::ParseInt => HOST_PARSE_INT_FUNCTION_ID.to_string(),
            Self::ParseFloat => HOST_PARSE_FLOAT_FUNCTION_ID.to_string(),
            Self::DetachArrayBuffer => HOST_DETACH_ARRAY_BUFFER_FUNCTION_ID.to_string(),
        }
    }

    pub fn from_function_id(function_id: &str) -> Option<Self> {
        match function_id {
            HOST_PRINT_FUNCTION_ID => Some(Self::Print),
            HOST_GC_FUNCTION_ID => Some(Self::Gc),
            HOST_ASSERT_THROWS_FUNCTION_ID => Some(Self::AssertThrows),
            HOST_IS_CONSTRUCTOR_FUNCTION_ID => Some(Self::IsConstructor),
            HOST_CREATE_REALM_FUNCTION_ID => Some(Self::CreateRealm),
            HOST_PARSE_INT_FUNCTION_ID => Some(Self::ParseInt),
            HOST_PARSE_FLOAT_FUNCTION_ID => Some(Self::ParseFloat),
            HOST_DETACH_ARRAY_BUFFER_FUNCTION_ID => Some(Self::DetachArrayBuffer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StandardBuiltinId {
    FunctionConstructor,
    FunctionPrototypeCall,
    FunctionPrototypeApply,
    FunctionPrototypeBind,
    FunctionPrototypeToString,
    EvalFunction,
    ObjectConstructor,
    ObjectCreate,
    ObjectGetPrototypeOf,
    ObjectSetPrototypeOf,
    ObjectDefineProperty,
    ObjectDefineProperties,
    ObjectGetOwnPropertyDescriptor,
    ObjectGetOwnPropertyNames,
    ObjectGetOwnPropertySymbols,
    ObjectKeys,
    ObjectValues,
    ObjectHasOwn,
    ObjectIs,
    ObjectIsSealed,
    ObjectIsFrozen,
    ObjectFreeze,
    ObjectIsExtensible,
    ObjectPreventExtensions,
    ObjectPrototypeHasOwnProperty,
    ObjectPrototypePropertyIsEnumerable,
    ObjectPrototypeIsPrototypeOf,
    ObjectPrototypeToString,
    ObjectPrototypeToLocaleString,
    ObjectPrototypeValueOf,
    ProxyConstructor,
    ProxyRevocable,
    ProxyRevoke,
    ReflectConstruct,
    ReflectApply,
    ReflectGet,
    ReflectGetPrototypeOf,
    ReflectGetOwnPropertyDescriptor,
    ReflectSet,
    ReflectHas,
    ReflectDefineProperty,
    ReflectDeleteProperty,
    ReflectIsExtensible,
    ReflectPreventExtensions,
    ReflectSetPrototypeOf,
    ReflectOwnKeys,
    ArrayConstructor,
    ArrayFrom,
    ArrayOf,
    ArrayIsArray,
    ArraySpeciesGetter,
    ArrayPrototypeConcat,
    ArrayPrototypeToLocaleString,
    ArrayPrototypeFlat,
    ArrayPrototypeFlatMap,
    ArrayPrototypeAt,
    ArrayPrototypeIncludes,
    ArrayPrototypeIndexOf,
    ArrayPrototypeLastIndexOf,
    ArrayPrototypeFind,
    ArrayPrototypeFindIndex,
    ArrayPrototypeFindLast,
    ArrayPrototypeFindLastIndex,
    ArrayPrototypeEvery,
    ArrayPrototypeSome,
    ArrayPrototypeForEach,
    ArrayPrototypeFilter,
    ArrayPrototypeMap,
    ArrayPrototypePop,
    ArrayPrototypePush,
    ArrayPrototypeKeys,
    ArrayPrototypeEntries,
    ArrayPrototypeValues,
    ArrayIteratorNext,
    ArrayIteratorIdentity,
    IteratorConstructor,
    IteratorFrom,
    IteratorPrototypeToArray,
    IteratorPrototypeForEach,
    IteratorPrototypeEvery,
    IteratorPrototypeSome,
    IteratorPrototypeFind,
    IteratorPrototypeReduce,
    IteratorPrototypeMap,
    IteratorMapNext,
    IteratorMapReturn,
    IteratorPrototypeFilter,
    IteratorFilterNext,
    IteratorFilterReturn,
    IteratorPrototypeFlatMap,
    IteratorFlatMapNext,
    IteratorFlatMapReturn,
    IteratorPrototypeTake,
    IteratorTakeNext,
    IteratorTakeReturn,
    IteratorPrototypeDrop,
    IteratorDropNext,
    IteratorDropReturn,
    IteratorPrototypeConstructorGetter,
    IteratorPrototypeConstructorSetter,
    IteratorPrototypeSymbolDispose,
    IteratorPrototypeToStringTagGetter,
    IteratorPrototypeToStringTagSetter,
    IteratorFromWrapperNext,
    IteratorFromWrapperReturn,
    ArrayBufferConstructor,
    SharedArrayBufferConstructor,
    ArrayBufferIsView,
    ArrayBufferSpeciesGetter,
    ArrayBufferPrototypeByteLengthGetter,
    SharedArrayBufferPrototypeByteLengthGetter,
    SharedArrayBufferPrototypeMaxByteLengthGetter,
    SharedArrayBufferPrototypeGrowableGetter,
    SharedArrayBufferPrototypeGrow,
    ArrayBufferPrototypeDetachedGetter,
    ArrayBufferPrototypeMaxByteLengthGetter,
    ArrayBufferPrototypeResizableGetter,
    ArrayBufferPrototypeResize,
    ArrayBufferPrototypeSlice,
    SharedArrayBufferPrototypeSlice,
    ArrayBufferPrototypeTransfer,
    ArrayBufferPrototypeTransferToFixedLength,
    ArrayBufferPrototypeTransferToImmutable,
    ArrayBufferPrototypeSliceToImmutable,
    DataViewConstructor,
    DataViewPrototypeBufferGetter,
    DataViewPrototypeByteLengthGetter,
    DataViewPrototypeByteOffsetGetter,
    TypedArrayPrototypeBufferGetter,
    TypedArrayPrototypeByteLengthGetter,
    TypedArrayPrototypeByteOffsetGetter,
    TypedArrayPrototypeLengthGetter,
    TypedArrayPrototypeToString,
    TypedArrayPrototypeToLocaleString,
    TypedArrayFrom,
    TypedArrayOf,
    DataViewPrototypeGetUint8,
    DataViewPrototypeSetUint8,
    DataViewPrototypeGetInt8,
    DataViewPrototypeSetInt8,
    DataViewPrototypeGetUint16,
    DataViewPrototypeSetUint16,
    DataViewPrototypeGetInt16,
    DataViewPrototypeSetInt16,
    DataViewPrototypeGetUint32,
    DataViewPrototypeSetUint32,
    DataViewPrototypeGetInt32,
    DataViewPrototypeSetInt32,
    DataViewPrototypeGetFloat16,
    DataViewPrototypeSetFloat16,
    DataViewPrototypeGetFloat32,
    DataViewPrototypeSetFloat32,
    DataViewPrototypeGetFloat64,
    DataViewPrototypeSetFloat64,
    DataViewPrototypeGetBigInt64,
    DataViewPrototypeSetBigInt64,
    DataViewPrototypeGetBigUint64,
    DataViewPrototypeSetBigUint64,
    DateConstructor,
    DateNow,
    DateUtc,
    DatePrototypeGetTime,
    DatePrototypeSetTime,
    DatePrototypeValueOf,
    DatePrototypeGetFullYear,
    DatePrototypeGetUtcFullYear,
    DatePrototypeGetMonth,
    DatePrototypeGetUtcMonth,
    DatePrototypeGetDate,
    DatePrototypeGetUtcDate,
    DatePrototypeGetDay,
    DatePrototypeGetUtcDay,
    DatePrototypeGetHours,
    DatePrototypeGetUtcHours,
    DatePrototypeGetMinutes,
    DatePrototypeGetUtcMinutes,
    DatePrototypeGetSeconds,
    DatePrototypeGetUtcSeconds,
    DatePrototypeGetMilliseconds,
    DatePrototypeGetUtcMilliseconds,
    DatePrototypeGetTimezoneOffset,
    DatePrototypeGetYear,
    DatePrototypeSetYear,
    DatePrototypeSetFullYear,
    DatePrototypeSetUtcFullYear,
    DatePrototypeSetMonth,
    DatePrototypeSetUtcMonth,
    DatePrototypeSetDate,
    DatePrototypeSetUtcDate,
    DatePrototypeSetHours,
    DatePrototypeSetUtcHours,
    DatePrototypeSetMinutes,
    DatePrototypeSetUtcMinutes,
    DatePrototypeSetSeconds,
    DatePrototypeSetUtcSeconds,
    DatePrototypeSetMilliseconds,
    DatePrototypeSetUtcMilliseconds,
    DatePrototypeToUtcString,
    RegExpConstructor,
    RegExpSpeciesGetter,
    RegExpLegacyStaticGetter,
    RegExpLegacyStaticSetter,
    RegExpPrototypeSymbolMatch,
    RegExpPrototypeSymbolMatchAll,
    RegExpPrototypeSymbolSearch,
    RegExpEscape,
    JsonParse,
    JsonStringify,
    JsonRawJson,
    JsonIsRawJson,
    AtomicsAdd,
    AtomicsAnd,
    AtomicsCompareExchange,
    AtomicsExchange,
    AtomicsLoad,
    AtomicsNotify,
    AtomicsOr,
    AtomicsPause,
    AtomicsStore,
    AtomicsSub,
    AtomicsWait,
    AtomicsWaitAsync,
    AtomicsXor,
    AtomicsIsLockFree,
    Float64ArrayConstructor,
    Float32ArrayConstructor,
    Int32ArrayConstructor,
    Int16ArrayConstructor,
    Int8ArrayConstructor,
    Uint32ArrayConstructor,
    Uint16ArrayConstructor,
    Uint8ArrayConstructor,
    Uint8ClampedArrayConstructor,
    BigInt64ArrayConstructor,
    BigUint64ArrayConstructor,
    BigIntConstructor,
    BigIntAsIntN,
    BigIntAsUintN,
    BigIntPrototypeToString,
    BigIntPrototypeToLocaleString,
    BigIntPrototypeValueOf,
    NumberConstructor,
    NumberIsInteger,
    NumberIsSafeInteger,
    NumberIsFinite,
    NumberIsNaN,
    NumberPrototypeToExponential,
    NumberPrototypeToFixed,
    NumberPrototypeToPrecision,
    NumberPrototypeToString,
    NumberPrototypeToLocaleString,
    NumberPrototypeValueOf,
    GlobalIsFinite,
    GlobalIsNaN,
    MathAbs,
    MathAcos,
    MathAcosh,
    MathAsin,
    MathAsinh,
    MathAtan,
    MathAtan2,
    MathAtanh,
    MathCbrt,
    MathCeil,
    MathClz32,
    MathCos,
    MathCosh,
    MathExp,
    MathExpm1,
    MathF16Round,
    MathFloor,
    MathFround,
    MathHypot,
    MathImul,
    MathLog,
    MathLog10,
    MathLog1p,
    MathLog2,
    MathPow,
    MathRandom,
    MathRound,
    MathSign,
    MathSin,
    MathSinh,
    MathSqrt,
    MathSumPrecise,
    MathTan,
    MathTanh,
    MathTrunc,
    MathMin,
    MathMax,
    StringConstructor,
    StringPrototypeToString,
    StringPrototypeValueOf,
    StringPrototypeCharAt,
    StringPrototypeCharCodeAt,
    StringPrototypeCodePointAt,
    StringPrototypeAt,
    StringPrototypeAnchor,
    StringPrototypeBig,
    StringPrototypeBlink,
    StringPrototypeBold,
    StringPrototypeFixed,
    StringPrototypeFontcolor,
    StringPrototypeFontsize,
    StringPrototypeItalics,
    StringPrototypeLink,
    StringPrototypeSmall,
    StringPrototypeStrike,
    StringPrototypeSub,
    StringPrototypeSubstr,
    StringPrototypeSubstring,
    StringPrototypeSup,
    StringPrototypeMatch,
    StringPrototypeMatchAll,
    StringPrototypeReplace,
    StringPrototypeReplaceAll,
    StringPrototypeSearch,
    StringPrototypeIndexOf,
    StringPrototypeLastIndexOf,
    StringPrototypeSlice,
    StringPrototypeSplit,
    StringPrototypePadStart,
    StringPrototypePadEnd,
    StringPrototypeRepeat,
    StringPrototypeEndsWith,
    StringPrototypeIncludes,
    StringPrototypeStartsWith,
    StringPrototypeToUpperCase,
    StringPrototypeTrim,
    StringPrototypeTrimStart,
    StringPrototypeTrimEnd,
    StringPrototypeIsWellFormed,
    StringPrototypeToWellFormed,
    BooleanConstructor,
    BooleanPrototypeToString,
    BooleanPrototypeValueOf,
    SymbolConstructor,
    SymbolFor,
    SymbolKeyFor,
    SymbolPrototypeDescriptionGetter,
    SymbolPrototypeToString,
    SymbolPrototypeValueOf,
    SymbolPrototypeToPrimitive,
    ErrorConstructor,
    ErrorIsError,
    EvalErrorConstructor,
    AggregateErrorConstructor,
    SuppressedErrorConstructor,
    RangeErrorConstructor,
    SyntaxErrorConstructor,
    TypeErrorConstructor,
    URIErrorConstructor,
    ReferenceErrorConstructor,
    ErrorPrototypeToString,
    ThrowTypeError,
    BoundFunctionInvoker,
    Escape,
    Unescape,
}

impl StandardBuiltinId {
    pub const fn global_name(self) -> Option<&'static str> {
        match self {
            Self::FunctionConstructor => Some(FUNCTION_NAME),
            Self::AggregateErrorConstructor => Some(AGGREGATE_ERROR_NAME),
            Self::SuppressedErrorConstructor => Some(SUPPRESSED_ERROR_NAME),
            Self::ObjectConstructor => Some(OBJECT_NAME),
            Self::ProxyConstructor => Some(PROXY_NAME),
            Self::IteratorConstructor => Some("Iterator"),
            Self::ArrayConstructor => Some(ARRAY_NAME),
            Self::ArrayBufferConstructor => Some(ARRAY_BUFFER_NAME),
            Self::SharedArrayBufferConstructor => Some(SHARED_ARRAY_BUFFER_NAME),
            Self::DataViewConstructor => Some(DATA_VIEW_NAME),
            Self::DateConstructor => Some(DATE_NAME),
            Self::RegExpConstructor => Some(REGEXP_NAME),
            Self::Float64ArrayConstructor => Some(FLOAT64_ARRAY_NAME),
            Self::Float32ArrayConstructor => Some(FLOAT32_ARRAY_NAME),
            Self::Int32ArrayConstructor => Some(INT32_ARRAY_NAME),
            Self::Int16ArrayConstructor => Some(INT16_ARRAY_NAME),
            Self::Int8ArrayConstructor => Some(INT8_ARRAY_NAME),
            Self::Uint32ArrayConstructor => Some(UINT32_ARRAY_NAME),
            Self::Uint16ArrayConstructor => Some(UINT16_ARRAY_NAME),
            Self::Uint8ArrayConstructor => Some(UINT8_ARRAY_NAME),
            Self::Uint8ClampedArrayConstructor => Some(UINT8_CLAMPED_ARRAY_NAME),
            Self::BigInt64ArrayConstructor => Some(BIGINT64_ARRAY_NAME),
            Self::BigUint64ArrayConstructor => Some(BIGUINT64_ARRAY_NAME),
            Self::BigIntConstructor => Some(BIGINT_NAME),
            Self::BigIntAsIntN => Some(BIGINT_NAME),
            Self::BigIntAsUintN => Some(BIGINT_NAME),
            Self::BigIntPrototypeToString => Some(BIGINT_NAME),
            Self::BigIntPrototypeToLocaleString => Some(BIGINT_NAME),
            Self::BigIntPrototypeValueOf => Some(BIGINT_NAME),
            Self::NumberConstructor => Some(NUMBER_NAME),
            Self::GlobalIsFinite => Some("isFinite"),
            Self::GlobalIsNaN => Some("isNaN"),
            Self::StringConstructor => Some(STRING_NAME),
            Self::StringPrototypeToString
            | Self::StringPrototypeValueOf
            | Self::StringPrototypeCharAt
            | Self::StringPrototypeCharCodeAt
            | Self::StringPrototypeCodePointAt
            | Self::StringPrototypeAt
            | Self::StringPrototypeAnchor
            | Self::StringPrototypeBig
            | Self::StringPrototypeBlink
            | Self::StringPrototypeBold
            | Self::StringPrototypeFixed
            | Self::StringPrototypeFontcolor
            | Self::StringPrototypeFontsize
            | Self::StringPrototypeItalics
            | Self::StringPrototypeLink
            | Self::StringPrototypeSmall
            | Self::StringPrototypeStrike
            | Self::StringPrototypeSub
            | Self::StringPrototypeSubstr
            | Self::StringPrototypeSubstring
            | Self::StringPrototypeSup
            | Self::StringPrototypeMatch
            | Self::StringPrototypeMatchAll
            | Self::StringPrototypeReplace
            | Self::StringPrototypeReplaceAll
            | Self::StringPrototypeSearch
            | Self::StringPrototypeIndexOf
            | Self::StringPrototypeLastIndexOf
            | Self::StringPrototypeSlice
            | Self::StringPrototypeSplit
            | Self::StringPrototypePadStart
            | Self::StringPrototypePadEnd
            | Self::StringPrototypeRepeat
            | Self::StringPrototypeEndsWith
            | Self::StringPrototypeIncludes
            | Self::StringPrototypeStartsWith
            | Self::StringPrototypeToUpperCase
            | Self::StringPrototypeTrim
            | Self::StringPrototypeTrimStart
            | Self::StringPrototypeTrimEnd
            | Self::StringPrototypeIsWellFormed
            | Self::StringPrototypeToWellFormed
            | Self::BooleanPrototypeToString
            | Self::BooleanPrototypeValueOf
            | Self::SymbolFor
            | Self::SymbolKeyFor
            | Self::SymbolPrototypeDescriptionGetter
            | Self::SymbolPrototypeToString
            | Self::SymbolPrototypeValueOf
            | Self::SymbolPrototypeToPrimitive => None,
            Self::BooleanConstructor => Some(BOOLEAN_NAME),
            Self::SymbolConstructor => Some(SYMBOL_NAME),
            Self::ErrorConstructor => Some(ERROR_NAME),
            Self::EvalErrorConstructor => Some(EVAL_ERROR_NAME),
            Self::RangeErrorConstructor => Some(RANGE_ERROR_NAME),
            Self::SyntaxErrorConstructor => Some(SYNTAX_ERROR_NAME),
            Self::TypeErrorConstructor => Some(TYPE_ERROR_NAME),
            Self::URIErrorConstructor => Some(URI_ERROR_NAME),
            Self::ReferenceErrorConstructor => Some(REFERENCE_ERROR_NAME),
            Self::Escape => Some(ESCAPE_NAME),
            Self::Unescape => Some(UNESCAPE_NAME),
            Self::FunctionPrototypeCall
            | Self::FunctionPrototypeApply
            | Self::FunctionPrototypeBind
            | Self::FunctionPrototypeToString
            | Self::ObjectCreate
            | Self::ObjectGetPrototypeOf
            | Self::ObjectSetPrototypeOf
            | Self::ObjectDefineProperty
            | Self::ObjectDefineProperties
            | Self::ObjectGetOwnPropertyDescriptor
            | Self::ObjectGetOwnPropertyNames
            | Self::ObjectGetOwnPropertySymbols
            | Self::ObjectKeys
            | Self::ObjectValues
            | Self::ObjectHasOwn
            | Self::ObjectIs
            | Self::ObjectIsSealed
            | Self::ObjectIsFrozen
            | Self::ObjectFreeze
            | Self::ObjectIsExtensible
            | Self::ObjectPreventExtensions
            | Self::ObjectPrototypeHasOwnProperty
            | Self::ObjectPrototypePropertyIsEnumerable
            | Self::ObjectPrototypeIsPrototypeOf
            | Self::ObjectPrototypeToString
            | Self::ObjectPrototypeToLocaleString
            | Self::ObjectPrototypeValueOf
            | Self::ProxyRevocable
            | Self::ProxyRevoke
            | Self::ReflectConstruct
            | Self::ReflectApply
            | Self::ReflectGet
            | Self::ReflectGetPrototypeOf
            | Self::ReflectGetOwnPropertyDescriptor
            | Self::ReflectSet
            | Self::ReflectHas
            | Self::ReflectDefineProperty
            | Self::ReflectDeleteProperty
            | Self::ReflectIsExtensible
            | Self::ReflectPreventExtensions
            | Self::ReflectSetPrototypeOf
            | Self::ReflectOwnKeys
            | Self::ArrayFrom
            | Self::ArrayOf
            | Self::ArrayIsArray
            | Self::ArraySpeciesGetter
            | Self::ArrayPrototypeConcat
            | Self::ArrayPrototypeToLocaleString
            | Self::ArrayPrototypeFlat
            | Self::ArrayPrototypeFlatMap
            | Self::ArrayPrototypeAt
            | Self::ArrayPrototypeIncludes
            | Self::ArrayPrototypeIndexOf
            | Self::ArrayPrototypeLastIndexOf
            | Self::ArrayPrototypeFind
            | Self::ArrayPrototypeFindIndex
            | Self::ArrayPrototypeFindLast
            | Self::ArrayPrototypeFindLastIndex
            | Self::ArrayPrototypeEvery
            | Self::ArrayPrototypeSome
            | Self::ArrayPrototypeForEach
            | Self::ArrayPrototypeFilter
            | Self::ArrayPrototypeMap
            | Self::ArrayPrototypePop
            | Self::ArrayPrototypePush
            | Self::ArrayPrototypeKeys
            | Self::ArrayPrototypeEntries
            | Self::ArrayPrototypeValues
            | Self::ArrayIteratorNext
            | Self::ArrayIteratorIdentity
            | Self::IteratorFrom
            | Self::IteratorPrototypeToArray
            | Self::IteratorPrototypeForEach
            | Self::IteratorPrototypeEvery
            | Self::IteratorPrototypeSome
            | Self::IteratorPrototypeFind
            | Self::IteratorPrototypeReduce
            | Self::IteratorPrototypeMap
            | Self::IteratorMapNext
            | Self::IteratorMapReturn
            | Self::IteratorPrototypeFilter
            | Self::IteratorFilterNext
            | Self::IteratorFilterReturn
            | Self::IteratorPrototypeFlatMap
            | Self::IteratorFlatMapNext
            | Self::IteratorFlatMapReturn
            | Self::IteratorPrototypeTake
            | Self::IteratorTakeNext
            | Self::IteratorTakeReturn
            | Self::IteratorPrototypeDrop
            | Self::IteratorDropNext
            | Self::IteratorDropReturn
            | Self::IteratorPrototypeConstructorGetter
            | Self::IteratorPrototypeConstructorSetter
            | Self::IteratorPrototypeSymbolDispose
            | Self::IteratorPrototypeToStringTagGetter
            | Self::IteratorPrototypeToStringTagSetter
            | Self::IteratorFromWrapperNext
            | Self::IteratorFromWrapperReturn
            | Self::ArrayBufferIsView
            | Self::NumberIsInteger
            | Self::NumberIsSafeInteger
            | Self::NumberIsFinite
            | Self::NumberIsNaN
            | Self::NumberPrototypeToExponential
            | Self::NumberPrototypeToFixed
            | Self::NumberPrototypeToPrecision
            | Self::NumberPrototypeToString
            | Self::NumberPrototypeToLocaleString
            | Self::NumberPrototypeValueOf
            | Self::MathTrunc
            | Self::MathMin
            | Self::MathMax
            | Self::ErrorIsError
            | Self::ArrayBufferSpeciesGetter
            | Self::ArrayBufferPrototypeByteLengthGetter
            | Self::SharedArrayBufferPrototypeByteLengthGetter
            | Self::SharedArrayBufferPrototypeMaxByteLengthGetter
            | Self::SharedArrayBufferPrototypeGrowableGetter
            | Self::SharedArrayBufferPrototypeGrow
            | Self::ArrayBufferPrototypeDetachedGetter
            | Self::ArrayBufferPrototypeMaxByteLengthGetter
            | Self::ArrayBufferPrototypeResizableGetter
            | Self::ArrayBufferPrototypeResize
            | Self::ArrayBufferPrototypeSlice
            | Self::SharedArrayBufferPrototypeSlice
            | Self::ArrayBufferPrototypeTransfer
            | Self::ArrayBufferPrototypeTransferToFixedLength
            | Self::ArrayBufferPrototypeTransferToImmutable
            | Self::ArrayBufferPrototypeSliceToImmutable
            | Self::DataViewPrototypeBufferGetter
            | Self::DataViewPrototypeByteLengthGetter
            | Self::DataViewPrototypeByteOffsetGetter
            | Self::TypedArrayPrototypeBufferGetter
            | Self::TypedArrayPrototypeByteLengthGetter
            | Self::TypedArrayPrototypeByteOffsetGetter
            | Self::TypedArrayPrototypeLengthGetter
            | Self::TypedArrayPrototypeToString
            | Self::TypedArrayPrototypeToLocaleString
            | Self::TypedArrayFrom
            | Self::TypedArrayOf
            | Self::DataViewPrototypeGetUint8
            | Self::DataViewPrototypeSetUint8
            | Self::DataViewPrototypeGetInt8
            | Self::DataViewPrototypeSetInt8
            | Self::DataViewPrototypeGetUint16
            | Self::DataViewPrototypeSetUint16
            | Self::DataViewPrototypeGetInt16
            | Self::DataViewPrototypeSetInt16
            | Self::DataViewPrototypeGetUint32
            | Self::DataViewPrototypeSetUint32
            | Self::DataViewPrototypeGetInt32
            | Self::DataViewPrototypeSetInt32
            | Self::DataViewPrototypeGetFloat16
            | Self::DataViewPrototypeSetFloat16
            | Self::DataViewPrototypeGetFloat32
            | Self::DataViewPrototypeSetFloat32
            | Self::DataViewPrototypeGetFloat64
            | Self::DataViewPrototypeSetFloat64
            | Self::DataViewPrototypeGetBigInt64
            | Self::DataViewPrototypeSetBigInt64
            | Self::DataViewPrototypeGetBigUint64
            | Self::DataViewPrototypeSetBigUint64
            | Self::DateNow
            | Self::DateUtc
            | Self::DatePrototypeGetTime
            | Self::DatePrototypeSetTime
            | Self::DatePrototypeValueOf
            | Self::DatePrototypeGetFullYear
            | Self::DatePrototypeGetUtcFullYear
            | Self::DatePrototypeGetMonth
            | Self::DatePrototypeGetUtcMonth
            | Self::DatePrototypeGetDate
            | Self::DatePrototypeGetUtcDate
            | Self::DatePrototypeGetDay
            | Self::DatePrototypeGetUtcDay
            | Self::DatePrototypeGetHours
            | Self::DatePrototypeGetUtcHours
            | Self::DatePrototypeGetMinutes
            | Self::DatePrototypeGetUtcMinutes
            | Self::DatePrototypeGetSeconds
            | Self::DatePrototypeGetUtcSeconds
            | Self::DatePrototypeGetMilliseconds
            | Self::DatePrototypeGetUtcMilliseconds
            | Self::DatePrototypeGetTimezoneOffset
            | Self::DatePrototypeGetYear
            | Self::DatePrototypeSetYear
            | Self::DatePrototypeSetFullYear
            | Self::DatePrototypeSetUtcFullYear
            | Self::DatePrototypeSetMonth
            | Self::DatePrototypeSetUtcMonth
            | Self::DatePrototypeSetDate
            | Self::DatePrototypeSetUtcDate
            | Self::DatePrototypeSetHours
            | Self::DatePrototypeSetUtcHours
            | Self::DatePrototypeSetMinutes
            | Self::DatePrototypeSetUtcMinutes
            | Self::DatePrototypeSetSeconds
            | Self::DatePrototypeSetUtcSeconds
            | Self::DatePrototypeSetMilliseconds
            | Self::DatePrototypeSetUtcMilliseconds
            | Self::DatePrototypeToUtcString
            | Self::RegExpSpeciesGetter
            | Self::RegExpLegacyStaticGetter
            | Self::RegExpLegacyStaticSetter
            | Self::RegExpPrototypeSymbolMatch
            | Self::RegExpPrototypeSymbolMatchAll
            | Self::RegExpPrototypeSymbolSearch
            | Self::RegExpEscape
            | Self::JsonParse
            | Self::JsonStringify
            | Self::JsonRawJson
            | Self::JsonIsRawJson
            | Self::AtomicsAdd
            | Self::AtomicsAnd
            | Self::AtomicsCompareExchange
            | Self::AtomicsExchange
            | Self::AtomicsLoad
            | Self::AtomicsNotify
            | Self::AtomicsOr
            | Self::AtomicsPause
            | Self::AtomicsStore
            | Self::AtomicsSub
            | Self::AtomicsWait
            | Self::AtomicsWaitAsync
            | Self::AtomicsXor
            | Self::AtomicsIsLockFree
            | Self::MathAbs
            | Self::MathAcos
            | Self::MathAcosh
            | Self::MathAsin
            | Self::MathAsinh
            | Self::MathAtan
            | Self::MathAtan2
            | Self::MathAtanh
            | Self::MathCbrt
            | Self::MathCeil
            | Self::MathClz32
            | Self::MathCos
            | Self::MathCosh
            | Self::MathExp
            | Self::MathExpm1
            | Self::MathF16Round
            | Self::MathFloor
            | Self::MathFround
            | Self::MathHypot
            | Self::MathImul
            | Self::MathLog
            | Self::MathLog10
            | Self::MathLog1p
            | Self::MathLog2
            | Self::MathPow
            | Self::MathRandom
            | Self::MathRound
            | Self::MathSign
            | Self::MathSin
            | Self::MathSinh
            | Self::MathSqrt
            | Self::MathSumPrecise
            | Self::MathTan
            | Self::MathTanh
            | Self::ErrorPrototypeToString
            | Self::ThrowTypeError
            | Self::BoundFunctionInvoker => None,
            Self::EvalFunction => Some("eval"),
        }
    }

    pub const fn debug_name(self) -> &'static str {
        match self {
            Self::FunctionConstructor => FUNCTION_NAME,
            Self::FunctionPrototypeCall => "Function.prototype.call",
            Self::FunctionPrototypeApply => "Function.prototype.apply",
            Self::FunctionPrototypeBind => "Function.prototype.bind",
            Self::FunctionPrototypeToString => "Function.prototype.toString",
            Self::EvalFunction => "eval",
            Self::ObjectConstructor => OBJECT_NAME,
            Self::ObjectCreate => "Object.create",
            Self::ObjectGetPrototypeOf => "Object.getPrototypeOf",
            Self::ObjectSetPrototypeOf => "Object.setPrototypeOf",
            Self::ObjectDefineProperty => "Object.defineProperty",
            Self::ObjectDefineProperties => "Object.defineProperties",
            Self::ObjectGetOwnPropertyDescriptor => "Object.getOwnPropertyDescriptor",
            Self::ObjectGetOwnPropertyNames => "Object.getOwnPropertyNames",
            Self::ObjectGetOwnPropertySymbols => "Object.getOwnPropertySymbols",
            Self::ObjectKeys => "Object.keys",
            Self::ObjectValues => "Object.values",
            Self::ObjectHasOwn => "Object.hasOwn",
            Self::ObjectIs => "Object.is",
            Self::ObjectIsSealed => "Object.isSealed",
            Self::ObjectIsFrozen => "Object.isFrozen",
            Self::ObjectFreeze => "Object.freeze",
            Self::ObjectIsExtensible => "Object.isExtensible",
            Self::ObjectPreventExtensions => "Object.preventExtensions",
            Self::ObjectPrototypeHasOwnProperty => "Object.prototype.hasOwnProperty",
            Self::ObjectPrototypePropertyIsEnumerable => "Object.prototype.propertyIsEnumerable",
            Self::ObjectPrototypeIsPrototypeOf => "Object.prototype.isPrototypeOf",
            Self::ObjectPrototypeToString => "Object.prototype.toString",
            Self::ObjectPrototypeToLocaleString => "Object.prototype.toLocaleString",
            Self::ObjectPrototypeValueOf => "Object.prototype.valueOf",
            Self::ProxyConstructor => PROXY_NAME,
            Self::ProxyRevocable => "Proxy.revocable",
            Self::ProxyRevoke => "[[ProxyRevoke]]",
            Self::ReflectConstruct => "Reflect.construct",
            Self::ReflectApply => "Reflect.apply",
            Self::ReflectGet => "Reflect.get",
            Self::ReflectGetPrototypeOf => "Reflect.getPrototypeOf",
            Self::ReflectGetOwnPropertyDescriptor => "Reflect.getOwnPropertyDescriptor",
            Self::ReflectSet => "Reflect.set",
            Self::ReflectHas => "Reflect.has",
            Self::ReflectDefineProperty => "Reflect.defineProperty",
            Self::ReflectDeleteProperty => "Reflect.deleteProperty",
            Self::ReflectIsExtensible => "Reflect.isExtensible",
            Self::ReflectPreventExtensions => "Reflect.preventExtensions",
            Self::ReflectSetPrototypeOf => "Reflect.setPrototypeOf",
            Self::ReflectOwnKeys => "Reflect.ownKeys",
            Self::ArrayConstructor => ARRAY_NAME,
            Self::ArrayFrom => "Array.from",
            Self::ArrayOf => "Array.of",
            Self::ArrayIsArray => "Array.isArray",
            Self::ArraySpeciesGetter => "get Array [Symbol.species]",
            Self::ArrayPrototypeConcat => "Array.prototype.concat",
            Self::ArrayPrototypeToLocaleString => "Array.prototype.toLocaleString",
            Self::ArrayPrototypeFlat => "Array.prototype.flat",
            Self::ArrayPrototypeFlatMap => "Array.prototype.flatMap",
            Self::ArrayPrototypeAt => "Array.prototype.at",
            Self::ArrayPrototypeIncludes => "Array.prototype.includes",
            Self::ArrayPrototypeIndexOf => "Array.prototype.indexOf",
            Self::ArrayPrototypeLastIndexOf => "Array.prototype.lastIndexOf",
            Self::ArrayPrototypeFind => "Array.prototype.find",
            Self::ArrayPrototypeFindIndex => "Array.prototype.findIndex",
            Self::ArrayPrototypeFindLast => "Array.prototype.findLast",
            Self::ArrayPrototypeFindLastIndex => "Array.prototype.findLastIndex",
            Self::ArrayPrototypeEvery => "Array.prototype.every",
            Self::ArrayPrototypeSome => "Array.prototype.some",
            Self::ArrayPrototypeForEach => "Array.prototype.forEach",
            Self::ArrayPrototypeFilter => "Array.prototype.filter",
            Self::ArrayPrototypeMap => "Array.prototype.map",
            Self::ArrayPrototypePop => "Array.prototype.pop",
            Self::ArrayPrototypePush => "Array.prototype.push",
            Self::ArrayPrototypeKeys => "Array.prototype.keys",
            Self::ArrayPrototypeEntries => "Array.prototype.entries",
            Self::ArrayPrototypeValues => "Array.prototype.values",
            Self::ArrayIteratorNext => "Array Iterator.prototype.next",
            Self::ArrayIteratorIdentity => "Array Iterator.prototype [Symbol.iterator]",
            Self::IteratorConstructor => "Iterator",
            Self::IteratorFrom => "Iterator.from",
            Self::IteratorPrototypeToArray => "Iterator.prototype.toArray",
            Self::IteratorPrototypeForEach => "Iterator.prototype.forEach",
            Self::IteratorPrototypeEvery => "Iterator.prototype.every",
            Self::IteratorPrototypeSome => "Iterator.prototype.some",
            Self::IteratorPrototypeFind => "Iterator.prototype.find",
            Self::IteratorPrototypeReduce => "Iterator.prototype.reduce",
            Self::IteratorPrototypeMap => "Iterator.prototype.map",
            Self::IteratorMapNext => "Iterator map helper next",
            Self::IteratorMapReturn => "Iterator map helper return",
            Self::IteratorPrototypeFilter => "Iterator.prototype.filter",
            Self::IteratorFilterNext => "Iterator filter helper next",
            Self::IteratorFilterReturn => "Iterator filter helper return",
            Self::IteratorPrototypeFlatMap => "Iterator.prototype.flatMap",
            Self::IteratorFlatMapNext => "Iterator flatMap helper next",
            Self::IteratorFlatMapReturn => "Iterator flatMap helper return",
            Self::IteratorPrototypeTake => "Iterator.prototype.take",
            Self::IteratorTakeNext => "Iterator take helper next",
            Self::IteratorTakeReturn => "Iterator take helper return",
            Self::IteratorPrototypeDrop => "Iterator.prototype.drop",
            Self::IteratorDropNext => "Iterator drop helper next",
            Self::IteratorDropReturn => "Iterator drop helper return",
            Self::IteratorPrototypeConstructorGetter => "get Iterator.prototype.constructor",
            Self::IteratorPrototypeConstructorSetter => "set Iterator.prototype.constructor",
            Self::IteratorPrototypeSymbolDispose => "Iterator.prototype[Symbol.dispose]",
            Self::IteratorPrototypeToStringTagGetter => {
                "get Iterator.prototype[Symbol.toStringTag]"
            }
            Self::IteratorPrototypeToStringTagSetter => {
                "set Iterator.prototype[Symbol.toStringTag]"
            }
            Self::IteratorFromWrapperNext => "%WrapForValidIteratorPrototype%.next",
            Self::IteratorFromWrapperReturn => "%WrapForValidIteratorPrototype%.return",
            Self::ArrayBufferConstructor => ARRAY_BUFFER_NAME,
            Self::SharedArrayBufferConstructor => SHARED_ARRAY_BUFFER_NAME,
            Self::ArrayBufferIsView => "ArrayBuffer.isView",
            Self::ArrayBufferSpeciesGetter => "get ArrayBuffer [Symbol.species]",
            Self::ArrayBufferPrototypeByteLengthGetter => "get ArrayBuffer.prototype.byteLength",
            Self::SharedArrayBufferPrototypeByteLengthGetter => {
                "get SharedArrayBuffer.prototype.byteLength"
            }
            Self::SharedArrayBufferPrototypeMaxByteLengthGetter => {
                "get SharedArrayBuffer.prototype.maxByteLength"
            }
            Self::SharedArrayBufferPrototypeGrowableGetter => {
                "get SharedArrayBuffer.prototype.growable"
            }
            Self::SharedArrayBufferPrototypeGrow => "SharedArrayBuffer.prototype.grow",
            Self::ArrayBufferPrototypeDetachedGetter => "get ArrayBuffer.prototype.detached",
            Self::ArrayBufferPrototypeMaxByteLengthGetter => {
                "get ArrayBuffer.prototype.maxByteLength"
            }
            Self::ArrayBufferPrototypeResizableGetter => "get ArrayBuffer.prototype.resizable",
            Self::ArrayBufferPrototypeResize => "ArrayBuffer.prototype.resize",
            Self::ArrayBufferPrototypeSlice => "ArrayBuffer.prototype.slice",
            Self::SharedArrayBufferPrototypeSlice => "SharedArrayBuffer.prototype.slice",
            Self::ArrayBufferPrototypeTransfer => "ArrayBuffer.prototype.transfer",
            Self::ArrayBufferPrototypeTransferToFixedLength => {
                "ArrayBuffer.prototype.transferToFixedLength"
            }
            Self::ArrayBufferPrototypeTransferToImmutable => {
                "ArrayBuffer.prototype.transferToImmutable"
            }
            Self::ArrayBufferPrototypeSliceToImmutable => "ArrayBuffer.prototype.sliceToImmutable",
            Self::DataViewConstructor => DATA_VIEW_NAME,
            Self::DataViewPrototypeBufferGetter => "get DataView.prototype.buffer",
            Self::DataViewPrototypeByteLengthGetter => "get DataView.prototype.byteLength",
            Self::DataViewPrototypeByteOffsetGetter => "get DataView.prototype.byteOffset",
            Self::TypedArrayPrototypeBufferGetter => "get TypedArray.prototype.buffer",
            Self::TypedArrayPrototypeByteLengthGetter => "get TypedArray.prototype.byteLength",
            Self::TypedArrayPrototypeByteOffsetGetter => "get TypedArray.prototype.byteOffset",
            Self::TypedArrayPrototypeLengthGetter => "get TypedArray.prototype.length",
            Self::TypedArrayPrototypeToString => "TypedArray.prototype.toString",
            Self::TypedArrayPrototypeToLocaleString => "TypedArray.prototype.toLocaleString",
            Self::TypedArrayFrom => "TypedArray.from",
            Self::TypedArrayOf => "TypedArray.of",
            Self::DataViewPrototypeGetUint8 => "DataView.prototype.getUint8",
            Self::DataViewPrototypeSetUint8 => "DataView.prototype.setUint8",
            Self::DataViewPrototypeGetInt8 => "DataView.prototype.getInt8",
            Self::DataViewPrototypeSetInt8 => "DataView.prototype.setInt8",
            Self::DataViewPrototypeGetUint16 => "DataView.prototype.getUint16",
            Self::DataViewPrototypeSetUint16 => "DataView.prototype.setUint16",
            Self::DataViewPrototypeGetInt16 => "DataView.prototype.getInt16",
            Self::DataViewPrototypeSetInt16 => "DataView.prototype.setInt16",
            Self::DataViewPrototypeGetUint32 => "DataView.prototype.getUint32",
            Self::DataViewPrototypeSetUint32 => "DataView.prototype.setUint32",
            Self::DataViewPrototypeGetInt32 => "DataView.prototype.getInt32",
            Self::DataViewPrototypeSetInt32 => "DataView.prototype.setInt32",
            Self::DataViewPrototypeGetFloat16 => "DataView.prototype.getFloat16",
            Self::DataViewPrototypeSetFloat16 => "DataView.prototype.setFloat16",
            Self::DataViewPrototypeGetFloat32 => "DataView.prototype.getFloat32",
            Self::DataViewPrototypeSetFloat32 => "DataView.prototype.setFloat32",
            Self::DataViewPrototypeGetFloat64 => "DataView.prototype.getFloat64",
            Self::DataViewPrototypeSetFloat64 => "DataView.prototype.setFloat64",
            Self::DataViewPrototypeGetBigInt64 => "DataView.prototype.getBigInt64",
            Self::DataViewPrototypeSetBigInt64 => "DataView.prototype.setBigInt64",
            Self::DataViewPrototypeGetBigUint64 => "DataView.prototype.getBigUint64",
            Self::DataViewPrototypeSetBigUint64 => "DataView.prototype.setBigUint64",
            Self::DateConstructor => DATE_NAME,
            Self::DateNow => "Date.now",
            Self::DateUtc => "Date.UTC",
            Self::DatePrototypeGetTime => "Date.prototype.getTime",
            Self::DatePrototypeSetTime => "Date.prototype.setTime",
            Self::DatePrototypeValueOf => "Date.prototype.valueOf",
            Self::DatePrototypeGetFullYear => "Date.prototype.getFullYear",
            Self::DatePrototypeGetUtcFullYear => "Date.prototype.getUTCFullYear",
            Self::DatePrototypeGetMonth => "Date.prototype.getMonth",
            Self::DatePrototypeGetUtcMonth => "Date.prototype.getUTCMonth",
            Self::DatePrototypeGetDate => "Date.prototype.getDate",
            Self::DatePrototypeGetUtcDate => "Date.prototype.getUTCDate",
            Self::DatePrototypeGetDay => "Date.prototype.getDay",
            Self::DatePrototypeGetUtcDay => "Date.prototype.getUTCDay",
            Self::DatePrototypeGetHours => "Date.prototype.getHours",
            Self::DatePrototypeGetUtcHours => "Date.prototype.getUTCHours",
            Self::DatePrototypeGetMinutes => "Date.prototype.getMinutes",
            Self::DatePrototypeGetUtcMinutes => "Date.prototype.getUTCMinutes",
            Self::DatePrototypeGetSeconds => "Date.prototype.getSeconds",
            Self::DatePrototypeGetUtcSeconds => "Date.prototype.getUTCSeconds",
            Self::DatePrototypeGetMilliseconds => "Date.prototype.getMilliseconds",
            Self::DatePrototypeGetUtcMilliseconds => "Date.prototype.getUTCMilliseconds",
            Self::DatePrototypeGetTimezoneOffset => "Date.prototype.getTimezoneOffset",
            Self::DatePrototypeGetYear => "Date.prototype.getYear",
            Self::DatePrototypeSetYear => "Date.prototype.setYear",
            Self::DatePrototypeSetFullYear => "Date.prototype.setFullYear",
            Self::DatePrototypeSetUtcFullYear => "Date.prototype.setUTCFullYear",
            Self::DatePrototypeSetMonth => "Date.prototype.setMonth",
            Self::DatePrototypeSetUtcMonth => "Date.prototype.setUTCMonth",
            Self::DatePrototypeSetDate => "Date.prototype.setDate",
            Self::DatePrototypeSetUtcDate => "Date.prototype.setUTCDate",
            Self::DatePrototypeSetHours => "Date.prototype.setHours",
            Self::DatePrototypeSetUtcHours => "Date.prototype.setUTCHours",
            Self::DatePrototypeSetMinutes => "Date.prototype.setMinutes",
            Self::DatePrototypeSetUtcMinutes => "Date.prototype.setUTCMinutes",
            Self::DatePrototypeSetSeconds => "Date.prototype.setSeconds",
            Self::DatePrototypeSetUtcSeconds => "Date.prototype.setUTCSeconds",
            Self::DatePrototypeSetMilliseconds => "Date.prototype.setMilliseconds",
            Self::DatePrototypeSetUtcMilliseconds => "Date.prototype.setUTCMilliseconds",
            Self::DatePrototypeToUtcString => "Date.prototype.toUTCString",
            Self::RegExpConstructor => REGEXP_NAME,
            Self::RegExpSpeciesGetter => "get RegExp [Symbol.species]",
            Self::RegExpLegacyStaticGetter => "get RegExp legacy static",
            Self::RegExpLegacyStaticSetter => "set RegExp legacy static",
            Self::RegExpPrototypeSymbolMatch => "RegExp.prototype[Symbol.match]",
            Self::RegExpPrototypeSymbolMatchAll => "RegExp.prototype[Symbol.matchAll]",
            Self::RegExpPrototypeSymbolSearch => "RegExp.prototype[Symbol.search]",
            Self::RegExpEscape => "RegExp.escape",
            Self::JsonParse => "JSON.parse",
            Self::JsonStringify => "JSON.stringify",
            Self::JsonRawJson => "JSON.rawJSON",
            Self::JsonIsRawJson => "JSON.isRawJSON",
            Self::AtomicsAdd => "Atomics.add",
            Self::AtomicsAnd => "Atomics.and",
            Self::AtomicsCompareExchange => "Atomics.compareExchange",
            Self::AtomicsExchange => "Atomics.exchange",
            Self::AtomicsLoad => "Atomics.load",
            Self::AtomicsNotify => "Atomics.notify",
            Self::AtomicsOr => "Atomics.or",
            Self::AtomicsPause => "Atomics.pause",
            Self::AtomicsStore => "Atomics.store",
            Self::AtomicsSub => "Atomics.sub",
            Self::AtomicsWait => "Atomics.wait",
            Self::AtomicsWaitAsync => "Atomics.waitAsync",
            Self::AtomicsXor => "Atomics.xor",
            Self::AtomicsIsLockFree => "Atomics.isLockFree",
            Self::Float64ArrayConstructor => FLOAT64_ARRAY_NAME,
            Self::Float32ArrayConstructor => FLOAT32_ARRAY_NAME,
            Self::Int32ArrayConstructor => INT32_ARRAY_NAME,
            Self::Int16ArrayConstructor => INT16_ARRAY_NAME,
            Self::Int8ArrayConstructor => INT8_ARRAY_NAME,
            Self::Uint32ArrayConstructor => UINT32_ARRAY_NAME,
            Self::Uint16ArrayConstructor => UINT16_ARRAY_NAME,
            Self::Uint8ArrayConstructor => UINT8_ARRAY_NAME,
            Self::Uint8ClampedArrayConstructor => UINT8_CLAMPED_ARRAY_NAME,
            Self::BigInt64ArrayConstructor => BIGINT64_ARRAY_NAME,
            Self::BigUint64ArrayConstructor => BIGUINT64_ARRAY_NAME,
            Self::BigIntConstructor => BIGINT_NAME,
            Self::BigIntAsIntN => "BigInt.asIntN",
            Self::BigIntAsUintN => "BigInt.asUintN",
            Self::BigIntPrototypeToString => "BigInt.prototype.toString",
            Self::BigIntPrototypeToLocaleString => "BigInt.prototype.toLocaleString",
            Self::BigIntPrototypeValueOf => "BigInt.prototype.valueOf",
            Self::NumberConstructor => NUMBER_NAME,
            Self::NumberIsInteger => "Number.isInteger",
            Self::NumberIsSafeInteger => "Number.isSafeInteger",
            Self::NumberIsFinite => "Number.isFinite",
            Self::NumberIsNaN => "Number.isNaN",
            Self::NumberPrototypeToExponential => "Number.prototype.toExponential",
            Self::NumberPrototypeToFixed => "Number.prototype.toFixed",
            Self::NumberPrototypeToPrecision => "Number.prototype.toPrecision",
            Self::NumberPrototypeToString => "Number.prototype.toString",
            Self::NumberPrototypeToLocaleString => "Number.prototype.toLocaleString",
            Self::NumberPrototypeValueOf => "Number.prototype.valueOf",
            Self::GlobalIsFinite => "isFinite",
            Self::GlobalIsNaN => "isNaN",
            Self::MathAbs => "Math.abs",
            Self::MathAcos => "Math.acos",
            Self::MathAcosh => "Math.acosh",
            Self::MathAsin => "Math.asin",
            Self::MathAsinh => "Math.asinh",
            Self::MathAtan => "Math.atan",
            Self::MathAtan2 => "Math.atan2",
            Self::MathAtanh => "Math.atanh",
            Self::MathCbrt => "Math.cbrt",
            Self::MathCeil => "Math.ceil",
            Self::MathClz32 => "Math.clz32",
            Self::MathCos => "Math.cos",
            Self::MathCosh => "Math.cosh",
            Self::MathExp => "Math.exp",
            Self::MathExpm1 => "Math.expm1",
            Self::MathF16Round => "Math.f16round",
            Self::MathFloor => "Math.floor",
            Self::MathFround => "Math.fround",
            Self::MathHypot => "Math.hypot",
            Self::MathImul => "Math.imul",
            Self::MathLog => "Math.log",
            Self::MathLog10 => "Math.log10",
            Self::MathLog1p => "Math.log1p",
            Self::MathLog2 => "Math.log2",
            Self::MathPow => "Math.pow",
            Self::MathRandom => "Math.random",
            Self::MathRound => "Math.round",
            Self::MathSign => "Math.sign",
            Self::MathSin => "Math.sin",
            Self::MathSinh => "Math.sinh",
            Self::MathSqrt => "Math.sqrt",
            Self::MathSumPrecise => "Math.sumPrecise",
            Self::MathTan => "Math.tan",
            Self::MathTanh => "Math.tanh",
            Self::MathTrunc => "Math.trunc",
            Self::MathMin => "Math.min",
            Self::MathMax => "Math.max",
            Self::StringConstructor => STRING_NAME,
            Self::StringPrototypeToString => "String.prototype.toString",
            Self::StringPrototypeValueOf => "String.prototype.valueOf",
            Self::StringPrototypeCharAt => "String.prototype.charAt",
            Self::StringPrototypeCharCodeAt => "String.prototype.charCodeAt",
            Self::StringPrototypeCodePointAt => "String.prototype.codePointAt",
            Self::StringPrototypeAt => "String.prototype.at",
            Self::StringPrototypeAnchor => "String.prototype.anchor",
            Self::StringPrototypeBig => "String.prototype.big",
            Self::StringPrototypeBlink => "String.prototype.blink",
            Self::StringPrototypeBold => "String.prototype.bold",
            Self::StringPrototypeFixed => "String.prototype.fixed",
            Self::StringPrototypeFontcolor => "String.prototype.fontcolor",
            Self::StringPrototypeFontsize => "String.prototype.fontsize",
            Self::StringPrototypeItalics => "String.prototype.italics",
            Self::StringPrototypeLink => "String.prototype.link",
            Self::StringPrototypeSmall => "String.prototype.small",
            Self::StringPrototypeStrike => "String.prototype.strike",
            Self::StringPrototypeSub => "String.prototype.sub",
            Self::StringPrototypeSubstr => "String.prototype.substr",
            Self::StringPrototypeSubstring => "String.prototype.substring",
            Self::StringPrototypeSup => "String.prototype.sup",
            Self::StringPrototypeMatch => "String.prototype.match",
            Self::StringPrototypeMatchAll => "String.prototype.matchAll",
            Self::StringPrototypeReplace => "String.prototype.replace",
            Self::StringPrototypeReplaceAll => "String.prototype.replaceAll",
            Self::StringPrototypeSearch => "String.prototype.search",
            Self::StringPrototypeIndexOf => "String.prototype.indexOf",
            Self::StringPrototypeLastIndexOf => "String.prototype.lastIndexOf",
            Self::StringPrototypeSlice => "String.prototype.slice",
            Self::StringPrototypeSplit => "String.prototype.split",
            Self::StringPrototypePadStart => "String.prototype.padStart",
            Self::StringPrototypePadEnd => "String.prototype.padEnd",
            Self::StringPrototypeRepeat => "String.prototype.repeat",
            Self::StringPrototypeEndsWith => "String.prototype.endsWith",
            Self::StringPrototypeIncludes => "String.prototype.includes",
            Self::StringPrototypeStartsWith => "String.prototype.startsWith",
            Self::StringPrototypeToUpperCase => "String.prototype.toUpperCase",
            Self::StringPrototypeTrim => "String.prototype.trim",
            Self::StringPrototypeTrimStart => "String.prototype.trimStart",
            Self::StringPrototypeTrimEnd => "String.prototype.trimEnd",
            Self::StringPrototypeIsWellFormed => "String.prototype.isWellFormed",
            Self::StringPrototypeToWellFormed => "String.prototype.toWellFormed",
            Self::BooleanConstructor => BOOLEAN_NAME,
            Self::SymbolConstructor => SYMBOL_NAME,
            Self::SymbolFor => "Symbol.for",
            Self::SymbolKeyFor => "Symbol.keyFor",
            Self::SymbolPrototypeDescriptionGetter => "get Symbol.prototype.description",
            Self::SymbolPrototypeToString => "Symbol.prototype.toString",
            Self::SymbolPrototypeValueOf => "Symbol.prototype.valueOf",
            Self::SymbolPrototypeToPrimitive => "Symbol.prototype[Symbol.toPrimitive]",
            Self::BooleanPrototypeToString => "Boolean.prototype.toString",
            Self::BooleanPrototypeValueOf => "Boolean.prototype.valueOf",
            Self::ErrorConstructor => ERROR_NAME,
            Self::ErrorIsError => "Error.isError",
            Self::EvalErrorConstructor => EVAL_ERROR_NAME,
            Self::AggregateErrorConstructor => AGGREGATE_ERROR_NAME,
            Self::SuppressedErrorConstructor => SUPPRESSED_ERROR_NAME,
            Self::RangeErrorConstructor => RANGE_ERROR_NAME,
            Self::SyntaxErrorConstructor => SYNTAX_ERROR_NAME,
            Self::TypeErrorConstructor => TYPE_ERROR_NAME,
            Self::URIErrorConstructor => URI_ERROR_NAME,
            Self::ReferenceErrorConstructor => REFERENCE_ERROR_NAME,
            Self::ErrorPrototypeToString => "Error.prototype.toString",
            Self::ThrowTypeError => "%ThrowTypeError%",
            Self::BoundFunctionInvoker => "[[BoundFunctionInvoke]]",
            Self::Escape => ESCAPE_NAME,
            Self::Unescape => UNESCAPE_NAME,
        }
    }

    pub fn function_id(self) -> FunctionId {
        match self {
            Self::FunctionConstructor => BUILTIN_FUNCTION_FUNCTION_ID.to_string(),
            Self::FunctionPrototypeCall => BUILTIN_FUNCTION_PROTOTYPE_CALL_FUNCTION_ID.to_string(),
            Self::FunctionPrototypeApply => {
                BUILTIN_FUNCTION_PROTOTYPE_APPLY_FUNCTION_ID.to_string()
            }
            Self::FunctionPrototypeBind => BUILTIN_FUNCTION_PROTOTYPE_BIND_FUNCTION_ID.to_string(),
            Self::FunctionPrototypeToString => {
                BUILTIN_FUNCTION_PROTOTYPE_TO_STRING_FUNCTION_ID.to_string()
            }
            Self::EvalFunction => BUILTIN_EVAL_FUNCTION_ID.to_string(),
            Self::ObjectConstructor => BUILTIN_OBJECT_FUNCTION_ID.to_string(),
            Self::ObjectCreate => BUILTIN_OBJECT_CREATE_FUNCTION_ID.to_string(),
            Self::ObjectGetPrototypeOf => BUILTIN_OBJECT_GET_PROTOTYPE_OF_FUNCTION_ID.to_string(),
            Self::ObjectSetPrototypeOf => BUILTIN_OBJECT_SET_PROTOTYPE_OF_FUNCTION_ID.to_string(),
            Self::ObjectDefineProperty => BUILTIN_OBJECT_DEFINE_PROPERTY_FUNCTION_ID.to_string(),
            Self::ObjectDefineProperties => {
                BUILTIN_OBJECT_DEFINE_PROPERTIES_FUNCTION_ID.to_string()
            }
            Self::ObjectGetOwnPropertyDescriptor => {
                BUILTIN_OBJECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID.to_string()
            }
            Self::ObjectGetOwnPropertyNames => {
                BUILTIN_OBJECT_GET_OWN_PROPERTY_NAMES_FUNCTION_ID.to_string()
            }
            Self::ObjectGetOwnPropertySymbols => {
                BUILTIN_OBJECT_GET_OWN_PROPERTY_SYMBOLS_FUNCTION_ID.to_string()
            }
            Self::ObjectKeys => BUILTIN_OBJECT_KEYS_FUNCTION_ID.to_string(),
            Self::ObjectValues => BUILTIN_OBJECT_VALUES_FUNCTION_ID.to_string(),
            Self::ObjectHasOwn => BUILTIN_OBJECT_HAS_OWN_FUNCTION_ID.to_string(),
            Self::ObjectIs => BUILTIN_OBJECT_IS_FUNCTION_ID.to_string(),
            Self::ObjectIsSealed => BUILTIN_OBJECT_IS_SEALED_FUNCTION_ID.to_string(),
            Self::ObjectIsFrozen => BUILTIN_OBJECT_IS_FROZEN_FUNCTION_ID.to_string(),
            Self::ObjectFreeze => BUILTIN_OBJECT_FREEZE_FUNCTION_ID.to_string(),
            Self::ObjectIsExtensible => BUILTIN_OBJECT_IS_EXTENSIBLE_FUNCTION_ID.to_string(),
            Self::ObjectPreventExtensions => {
                BUILTIN_OBJECT_PREVENT_EXTENSIONS_FUNCTION_ID.to_string()
            }
            Self::ObjectPrototypeHasOwnProperty => {
                BUILTIN_OBJECT_PROTOTYPE_HAS_OWN_PROPERTY_FUNCTION_ID.to_string()
            }
            Self::ObjectPrototypePropertyIsEnumerable => {
                BUILTIN_OBJECT_PROTOTYPE_PROPERTY_IS_ENUMERABLE_FUNCTION_ID.to_string()
            }
            Self::ObjectPrototypeIsPrototypeOf => {
                BUILTIN_OBJECT_PROTOTYPE_IS_PROTOTYPE_OF_FUNCTION_ID.to_string()
            }
            Self::ObjectPrototypeToString => {
                BUILTIN_OBJECT_PROTOTYPE_TO_STRING_FUNCTION_ID.to_string()
            }
            Self::ObjectPrototypeToLocaleString => {
                BUILTIN_OBJECT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID.to_string()
            }
            Self::ObjectPrototypeValueOf => {
                BUILTIN_OBJECT_PROTOTYPE_VALUE_OF_FUNCTION_ID.to_string()
            }
            Self::ProxyConstructor => BUILTIN_PROXY_FUNCTION_ID.to_string(),
            Self::ProxyRevocable => BUILTIN_PROXY_REVOCABLE_FUNCTION_ID.to_string(),
            Self::ProxyRevoke => BUILTIN_PROXY_REVOKE_FUNCTION_ID.to_string(),
            Self::ReflectConstruct => BUILTIN_REFLECT_CONSTRUCT_FUNCTION_ID.to_string(),
            Self::ReflectApply => BUILTIN_REFLECT_APPLY_FUNCTION_ID.to_string(),
            Self::ReflectGet => BUILTIN_REFLECT_GET_FUNCTION_ID.to_string(),
            Self::ReflectGetPrototypeOf => BUILTIN_REFLECT_GET_PROTOTYPE_OF_FUNCTION_ID.to_string(),
            Self::ReflectGetOwnPropertyDescriptor => {
                BUILTIN_REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID.to_string()
            }
            Self::ReflectSet => BUILTIN_REFLECT_SET_FUNCTION_ID.to_string(),
            Self::ReflectHas => BUILTIN_REFLECT_HAS_FUNCTION_ID.to_string(),
            Self::ReflectDefineProperty => BUILTIN_REFLECT_DEFINE_PROPERTY_FUNCTION_ID.to_string(),
            Self::ReflectDeleteProperty => BUILTIN_REFLECT_DELETE_PROPERTY_FUNCTION_ID.to_string(),
            Self::ReflectIsExtensible => BUILTIN_REFLECT_IS_EXTENSIBLE_FUNCTION_ID.to_string(),
            Self::ReflectPreventExtensions => {
                BUILTIN_REFLECT_PREVENT_EXTENSIONS_FUNCTION_ID.to_string()
            }
            Self::ReflectSetPrototypeOf => BUILTIN_REFLECT_SET_PROTOTYPE_OF_FUNCTION_ID.to_string(),
            Self::ReflectOwnKeys => BUILTIN_REFLECT_OWN_KEYS_FUNCTION_ID.to_string(),
            Self::ArrayConstructor => BUILTIN_ARRAY_FUNCTION_ID.to_string(),
            Self::ArrayFrom => BUILTIN_ARRAY_FROM_FUNCTION_ID.to_string(),
            Self::ArrayOf => BUILTIN_ARRAY_OF_FUNCTION_ID.to_string(),
            Self::ArrayIsArray => BUILTIN_ARRAY_IS_ARRAY_FUNCTION_ID.to_string(),
            Self::ArraySpeciesGetter => BUILTIN_ARRAY_SPECIES_GETTER_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeConcat => BUILTIN_ARRAY_PROTOTYPE_CONCAT_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeToLocaleString => {
                BUILTIN_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID.to_string()
            }
            Self::ArrayPrototypeFlat => BUILTIN_ARRAY_PROTOTYPE_FLAT_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeFlatMap => BUILTIN_ARRAY_PROTOTYPE_FLAT_MAP_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeAt => BUILTIN_ARRAY_PROTOTYPE_AT_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeIncludes => {
                BUILTIN_ARRAY_PROTOTYPE_INCLUDES_FUNCTION_ID.to_string()
            }
            Self::ArrayPrototypeIndexOf => BUILTIN_ARRAY_PROTOTYPE_INDEX_OF_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeLastIndexOf => {
                BUILTIN_ARRAY_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID.to_string()
            }
            Self::ArrayPrototypeFind => BUILTIN_ARRAY_PROTOTYPE_FIND_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeFindIndex => {
                BUILTIN_ARRAY_PROTOTYPE_FIND_INDEX_FUNCTION_ID.to_string()
            }
            Self::ArrayPrototypeFindLast => {
                BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_FUNCTION_ID.to_string()
            }
            Self::ArrayPrototypeFindLastIndex => {
                BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_INDEX_FUNCTION_ID.to_string()
            }
            Self::ArrayPrototypeEvery => BUILTIN_ARRAY_PROTOTYPE_EVERY_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeSome => BUILTIN_ARRAY_PROTOTYPE_SOME_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeForEach => BUILTIN_ARRAY_PROTOTYPE_FOR_EACH_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeFilter => BUILTIN_ARRAY_PROTOTYPE_FILTER_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeMap => BUILTIN_ARRAY_PROTOTYPE_MAP_FUNCTION_ID.to_string(),
            Self::ArrayPrototypePop => BUILTIN_ARRAY_PROTOTYPE_POP_FUNCTION_ID.to_string(),
            Self::ArrayPrototypePush => BUILTIN_ARRAY_PROTOTYPE_PUSH_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeKeys => BUILTIN_ARRAY_PROTOTYPE_KEYS_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeEntries => BUILTIN_ARRAY_PROTOTYPE_ENTRIES_FUNCTION_ID.to_string(),
            Self::ArrayPrototypeValues => BUILTIN_ARRAY_PROTOTYPE_VALUES_FUNCTION_ID.to_string(),
            Self::ArrayIteratorNext => BUILTIN_ARRAY_ITERATOR_NEXT_FUNCTION_ID.to_string(),
            Self::ArrayIteratorIdentity => BUILTIN_ARRAY_ITERATOR_IDENTITY_FUNCTION_ID.to_string(),
            Self::IteratorConstructor => BUILTIN_ITERATOR_FUNCTION_ID.to_string(),
            Self::IteratorFrom => BUILTIN_ITERATOR_FROM_FUNCTION_ID.to_string(),
            Self::IteratorPrototypeToArray => {
                BUILTIN_ITERATOR_PROTOTYPE_TO_ARRAY_FUNCTION_ID.to_string()
            }
            Self::IteratorPrototypeForEach => {
                BUILTIN_ITERATOR_PROTOTYPE_FOR_EACH_FUNCTION_ID.to_string()
            }
            Self::IteratorPrototypeEvery => {
                BUILTIN_ITERATOR_PROTOTYPE_EVERY_FUNCTION_ID.to_string()
            }
            Self::IteratorPrototypeSome => BUILTIN_ITERATOR_PROTOTYPE_SOME_FUNCTION_ID.to_string(),
            Self::IteratorPrototypeFind => BUILTIN_ITERATOR_PROTOTYPE_FIND_FUNCTION_ID.to_string(),
            Self::IteratorPrototypeReduce => {
                BUILTIN_ITERATOR_PROTOTYPE_REDUCE_FUNCTION_ID.to_string()
            }
            Self::IteratorPrototypeMap => BUILTIN_ITERATOR_PROTOTYPE_MAP_FUNCTION_ID.to_string(),
            Self::IteratorMapNext => BUILTIN_ITERATOR_MAP_NEXT_FUNCTION_ID.to_string(),
            Self::IteratorMapReturn => BUILTIN_ITERATOR_MAP_RETURN_FUNCTION_ID.to_string(),
            Self::IteratorPrototypeFilter => {
                BUILTIN_ITERATOR_PROTOTYPE_FILTER_FUNCTION_ID.to_string()
            }
            Self::IteratorFilterNext => BUILTIN_ITERATOR_FILTER_NEXT_FUNCTION_ID.to_string(),
            Self::IteratorFilterReturn => BUILTIN_ITERATOR_FILTER_RETURN_FUNCTION_ID.to_string(),
            Self::IteratorPrototypeFlatMap => {
                BUILTIN_ITERATOR_PROTOTYPE_FLAT_MAP_FUNCTION_ID.to_string()
            }
            Self::IteratorFlatMapNext => BUILTIN_ITERATOR_FLAT_MAP_NEXT_FUNCTION_ID.to_string(),
            Self::IteratorFlatMapReturn => BUILTIN_ITERATOR_FLAT_MAP_RETURN_FUNCTION_ID.to_string(),
            Self::IteratorPrototypeTake => BUILTIN_ITERATOR_PROTOTYPE_TAKE_FUNCTION_ID.to_string(),
            Self::IteratorTakeNext => BUILTIN_ITERATOR_TAKE_NEXT_FUNCTION_ID.to_string(),
            Self::IteratorTakeReturn => BUILTIN_ITERATOR_TAKE_RETURN_FUNCTION_ID.to_string(),
            Self::IteratorPrototypeDrop => BUILTIN_ITERATOR_PROTOTYPE_DROP_FUNCTION_ID.to_string(),
            Self::IteratorDropNext => BUILTIN_ITERATOR_DROP_NEXT_FUNCTION_ID.to_string(),
            Self::IteratorDropReturn => BUILTIN_ITERATOR_DROP_RETURN_FUNCTION_ID.to_string(),
            Self::IteratorPrototypeConstructorGetter => {
                BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_GETTER_FUNCTION_ID.to_string()
            }
            Self::IteratorPrototypeConstructorSetter => {
                BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_SETTER_FUNCTION_ID.to_string()
            }
            Self::IteratorPrototypeSymbolDispose => {
                BUILTIN_ITERATOR_PROTOTYPE_SYMBOL_DISPOSE_FUNCTION_ID.to_string()
            }
            Self::IteratorPrototypeToStringTagGetter => {
                BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_GETTER_FUNCTION_ID.to_string()
            }
            Self::IteratorPrototypeToStringTagSetter => {
                BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_SETTER_FUNCTION_ID.to_string()
            }
            Self::IteratorFromWrapperNext => {
                BUILTIN_ITERATOR_FROM_WRAPPER_NEXT_FUNCTION_ID.to_string()
            }
            Self::IteratorFromWrapperReturn => {
                BUILTIN_ITERATOR_FROM_WRAPPER_RETURN_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferConstructor => BUILTIN_ARRAY_BUFFER_FUNCTION_ID.to_string(),
            Self::SharedArrayBufferConstructor => {
                BUILTIN_SHARED_ARRAY_BUFFER_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferIsView => BUILTIN_ARRAY_BUFFER_IS_VIEW_FUNCTION_ID.to_string(),
            Self::ArrayBufferSpeciesGetter => {
                BUILTIN_ARRAY_BUFFER_SPECIES_GETTER_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeByteLengthGetter => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID.to_string()
            }
            Self::SharedArrayBufferPrototypeByteLengthGetter => {
                BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID.to_string()
            }
            Self::SharedArrayBufferPrototypeMaxByteLengthGetter => {
                BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID.to_string()
            }
            Self::SharedArrayBufferPrototypeGrowableGetter => {
                BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROWABLE_GETTER_FUNCTION_ID.to_string()
            }
            Self::SharedArrayBufferPrototypeGrow => {
                BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROW_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeDetachedGetter => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_DETACHED_GETTER_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeMaxByteLengthGetter => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeResizableGetter => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZABLE_GETTER_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeResize => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZE_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeSlice => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID.to_string()
            }
            Self::SharedArrayBufferPrototypeSlice => {
                BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeTransfer => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeTransferToFixedLength => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_FIXED_LENGTH_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeTransferToImmutable => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_IMMUTABLE_FUNCTION_ID.to_string()
            }
            Self::ArrayBufferPrototypeSliceToImmutable => {
                BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_TO_IMMUTABLE_FUNCTION_ID.to_string()
            }
            Self::DataViewConstructor => BUILTIN_DATA_VIEW_FUNCTION_ID.to_string(),
            Self::DataViewPrototypeBufferGetter => {
                BUILTIN_DATA_VIEW_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeByteLengthGetter => {
                BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeByteOffsetGetter => {
                BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID.to_string()
            }
            Self::TypedArrayPrototypeBufferGetter => {
                BUILTIN_TYPED_ARRAY_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID.to_string()
            }
            Self::TypedArrayPrototypeByteLengthGetter => {
                BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID.to_string()
            }
            Self::TypedArrayPrototypeByteOffsetGetter => {
                BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID.to_string()
            }
            Self::TypedArrayPrototypeLengthGetter => {
                BUILTIN_TYPED_ARRAY_PROTOTYPE_LENGTH_GETTER_FUNCTION_ID.to_string()
            }
            Self::TypedArrayPrototypeToString => {
                BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_STRING_FUNCTION_ID.to_string()
            }
            Self::TypedArrayPrototypeToLocaleString => {
                BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID.to_string()
            }
            Self::TypedArrayFrom => BUILTIN_TYPED_ARRAY_FROM_FUNCTION_ID.to_string(),
            Self::TypedArrayOf => BUILTIN_TYPED_ARRAY_OF_FUNCTION_ID.to_string(),
            Self::DataViewPrototypeGetUint8 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT8_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetUint8 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT8_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetInt8 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT8_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetInt8 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT8_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetUint16 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT16_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetUint16 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT16_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetInt16 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT16_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetInt16 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT16_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetUint32 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT32_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetUint32 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT32_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetInt32 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT32_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetInt32 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT32_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetFloat16 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT16_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetFloat16 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT16_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetFloat32 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT32_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetFloat32 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT32_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetFloat64 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT64_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetFloat64 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT64_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetBigInt64 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGINT64_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetBigInt64 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGINT64_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeGetBigUint64 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGUINT64_FUNCTION_ID.to_string()
            }
            Self::DataViewPrototypeSetBigUint64 => {
                BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGUINT64_FUNCTION_ID.to_string()
            }
            Self::DateConstructor => BUILTIN_DATE_FUNCTION_ID.to_string(),
            Self::DateNow => BUILTIN_DATE_NOW_FUNCTION_ID.to_string(),
            Self::DateUtc => BUILTIN_DATE_UTC_FUNCTION_ID.to_string(),
            Self::DatePrototypeGetTime => BUILTIN_DATE_PROTOTYPE_GET_TIME_FUNCTION_ID.to_string(),
            Self::DatePrototypeSetTime => BUILTIN_DATE_PROTOTYPE_SET_TIME_FUNCTION_ID.to_string(),
            Self::DatePrototypeValueOf => BUILTIN_DATE_PROTOTYPE_VALUE_OF_FUNCTION_ID.to_string(),
            Self::DatePrototypeGetFullYear => {
                BUILTIN_DATE_PROTOTYPE_GET_FULL_YEAR_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetUtcFullYear => {
                BUILTIN_DATE_PROTOTYPE_GET_UTC_FULL_YEAR_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetMonth => BUILTIN_DATE_PROTOTYPE_GET_MONTH_FUNCTION_ID.to_string(),
            Self::DatePrototypeGetUtcMonth => {
                BUILTIN_DATE_PROTOTYPE_GET_UTC_MONTH_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetDate => BUILTIN_DATE_PROTOTYPE_GET_DATE_FUNCTION_ID.to_string(),
            Self::DatePrototypeGetUtcDate => {
                BUILTIN_DATE_PROTOTYPE_GET_UTC_DATE_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetDay => BUILTIN_DATE_PROTOTYPE_GET_DAY_FUNCTION_ID.to_string(),
            Self::DatePrototypeGetUtcDay => {
                BUILTIN_DATE_PROTOTYPE_GET_UTC_DAY_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetHours => BUILTIN_DATE_PROTOTYPE_GET_HOURS_FUNCTION_ID.to_string(),
            Self::DatePrototypeGetUtcHours => {
                BUILTIN_DATE_PROTOTYPE_GET_UTC_HOURS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetMinutes => {
                BUILTIN_DATE_PROTOTYPE_GET_MINUTES_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetUtcMinutes => {
                BUILTIN_DATE_PROTOTYPE_GET_UTC_MINUTES_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetSeconds => {
                BUILTIN_DATE_PROTOTYPE_GET_SECONDS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetUtcSeconds => {
                BUILTIN_DATE_PROTOTYPE_GET_UTC_SECONDS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetMilliseconds => {
                BUILTIN_DATE_PROTOTYPE_GET_MILLISECONDS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetUtcMilliseconds => {
                BUILTIN_DATE_PROTOTYPE_GET_UTC_MILLISECONDS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetTimezoneOffset => {
                BUILTIN_DATE_PROTOTYPE_GET_TIMEZONE_OFFSET_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeGetYear => BUILTIN_DATE_PROTOTYPE_GET_YEAR_FUNCTION_ID.to_string(),
            Self::DatePrototypeSetYear => BUILTIN_DATE_PROTOTYPE_SET_YEAR_FUNCTION_ID.to_string(),
            Self::DatePrototypeSetFullYear => {
                BUILTIN_DATE_PROTOTYPE_SET_FULL_YEAR_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetUtcFullYear => {
                BUILTIN_DATE_PROTOTYPE_SET_UTC_FULL_YEAR_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetMonth => BUILTIN_DATE_PROTOTYPE_SET_MONTH_FUNCTION_ID.to_string(),
            Self::DatePrototypeSetUtcMonth => {
                BUILTIN_DATE_PROTOTYPE_SET_UTC_MONTH_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetDate => BUILTIN_DATE_PROTOTYPE_SET_DATE_FUNCTION_ID.to_string(),
            Self::DatePrototypeSetUtcDate => {
                BUILTIN_DATE_PROTOTYPE_SET_UTC_DATE_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetHours => BUILTIN_DATE_PROTOTYPE_SET_HOURS_FUNCTION_ID.to_string(),
            Self::DatePrototypeSetUtcHours => {
                BUILTIN_DATE_PROTOTYPE_SET_UTC_HOURS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetMinutes => {
                BUILTIN_DATE_PROTOTYPE_SET_MINUTES_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetUtcMinutes => {
                BUILTIN_DATE_PROTOTYPE_SET_UTC_MINUTES_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetSeconds => {
                BUILTIN_DATE_PROTOTYPE_SET_SECONDS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetUtcSeconds => {
                BUILTIN_DATE_PROTOTYPE_SET_UTC_SECONDS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetMilliseconds => {
                BUILTIN_DATE_PROTOTYPE_SET_MILLISECONDS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeSetUtcMilliseconds => {
                BUILTIN_DATE_PROTOTYPE_SET_UTC_MILLISECONDS_FUNCTION_ID.to_string()
            }
            Self::DatePrototypeToUtcString => {
                BUILTIN_DATE_PROTOTYPE_TO_UTC_STRING_FUNCTION_ID.to_string()
            }
            Self::RegExpConstructor => BUILTIN_REGEXP_FUNCTION_ID.to_string(),
            Self::RegExpSpeciesGetter => BUILTIN_REGEXP_SPECIES_GETTER_FUNCTION_ID.to_string(),
            Self::RegExpLegacyStaticGetter => {
                BUILTIN_REGEXP_LEGACY_STATIC_GETTER_FUNCTION_ID.to_string()
            }
            Self::RegExpLegacyStaticSetter => {
                BUILTIN_REGEXP_LEGACY_STATIC_SETTER_FUNCTION_ID.to_string()
            }
            Self::RegExpPrototypeSymbolMatch => {
                BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_FUNCTION_ID.to_string()
            }
            Self::RegExpPrototypeSymbolMatchAll => {
                BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_FUNCTION_ID.to_string()
            }
            Self::RegExpPrototypeSymbolSearch => {
                BUILTIN_REGEXP_PROTOTYPE_SYMBOL_SEARCH_FUNCTION_ID.to_string()
            }
            Self::RegExpEscape => BUILTIN_REGEXP_ESCAPE_FUNCTION_ID.to_string(),
            Self::JsonParse => BUILTIN_JSON_PARSE_FUNCTION_ID.to_string(),
            Self::JsonStringify => BUILTIN_JSON_STRINGIFY_FUNCTION_ID.to_string(),
            Self::JsonRawJson => BUILTIN_JSON_RAW_JSON_FUNCTION_ID.to_string(),
            Self::JsonIsRawJson => BUILTIN_JSON_IS_RAW_JSON_FUNCTION_ID.to_string(),
            Self::AtomicsAdd => BUILTIN_ATOMICS_ADD_FUNCTION_ID.to_string(),
            Self::AtomicsAnd => BUILTIN_ATOMICS_AND_FUNCTION_ID.to_string(),
            Self::AtomicsCompareExchange => {
                BUILTIN_ATOMICS_COMPARE_EXCHANGE_FUNCTION_ID.to_string()
            }
            Self::AtomicsExchange => BUILTIN_ATOMICS_EXCHANGE_FUNCTION_ID.to_string(),
            Self::AtomicsLoad => BUILTIN_ATOMICS_LOAD_FUNCTION_ID.to_string(),
            Self::AtomicsNotify => BUILTIN_ATOMICS_NOTIFY_FUNCTION_ID.to_string(),
            Self::AtomicsOr => BUILTIN_ATOMICS_OR_FUNCTION_ID.to_string(),
            Self::AtomicsPause => BUILTIN_ATOMICS_PAUSE_FUNCTION_ID.to_string(),
            Self::AtomicsStore => BUILTIN_ATOMICS_STORE_FUNCTION_ID.to_string(),
            Self::AtomicsSub => BUILTIN_ATOMICS_SUB_FUNCTION_ID.to_string(),
            Self::AtomicsWait => BUILTIN_ATOMICS_WAIT_FUNCTION_ID.to_string(),
            Self::AtomicsWaitAsync => BUILTIN_ATOMICS_WAIT_ASYNC_FUNCTION_ID.to_string(),
            Self::AtomicsXor => BUILTIN_ATOMICS_XOR_FUNCTION_ID.to_string(),
            Self::AtomicsIsLockFree => BUILTIN_ATOMICS_IS_LOCK_FREE_FUNCTION_ID.to_string(),
            Self::Float64ArrayConstructor => BUILTIN_FLOAT64_ARRAY_FUNCTION_ID.to_string(),
            Self::Float32ArrayConstructor => BUILTIN_FLOAT32_ARRAY_FUNCTION_ID.to_string(),
            Self::Int32ArrayConstructor => BUILTIN_INT32_ARRAY_FUNCTION_ID.to_string(),
            Self::Int16ArrayConstructor => BUILTIN_INT16_ARRAY_FUNCTION_ID.to_string(),
            Self::Int8ArrayConstructor => BUILTIN_INT8_ARRAY_FUNCTION_ID.to_string(),
            Self::Uint32ArrayConstructor => BUILTIN_UINT32_ARRAY_FUNCTION_ID.to_string(),
            Self::Uint16ArrayConstructor => BUILTIN_UINT16_ARRAY_FUNCTION_ID.to_string(),
            Self::Uint8ArrayConstructor => BUILTIN_UINT8_ARRAY_FUNCTION_ID.to_string(),
            Self::Uint8ClampedArrayConstructor => {
                BUILTIN_UINT8_CLAMPED_ARRAY_FUNCTION_ID.to_string()
            }
            Self::BigInt64ArrayConstructor => BUILTIN_BIGINT64_ARRAY_FUNCTION_ID.to_string(),
            Self::BigUint64ArrayConstructor => BUILTIN_BIGUINT64_ARRAY_FUNCTION_ID.to_string(),
            Self::BigIntConstructor => BUILTIN_BIGINT_FUNCTION_ID.to_string(),
            Self::BigIntAsIntN => BUILTIN_BIGINT_AS_INT_N_FUNCTION_ID.to_string(),
            Self::BigIntAsUintN => BUILTIN_BIGINT_AS_UINT_N_FUNCTION_ID.to_string(),
            Self::BigIntPrototypeToString => {
                BUILTIN_BIGINT_PROTOTYPE_TO_STRING_FUNCTION_ID.to_string()
            }
            Self::BigIntPrototypeToLocaleString => {
                BUILTIN_BIGINT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID.to_string()
            }
            Self::BigIntPrototypeValueOf => {
                BUILTIN_BIGINT_PROTOTYPE_VALUE_OF_FUNCTION_ID.to_string()
            }
            Self::NumberConstructor => BUILTIN_NUMBER_FUNCTION_ID.to_string(),
            Self::NumberIsInteger => BUILTIN_NUMBER_IS_INTEGER_FUNCTION_ID.to_string(),
            Self::NumberIsSafeInteger => "$builtin.Number.isSafeInteger".to_string(),
            Self::NumberIsFinite => "$builtin.Number.isFinite".to_string(),
            Self::NumberIsNaN => "$builtin.Number.isNaN".to_string(),
            Self::NumberPrototypeToExponential => {
                "$builtin.Number.prototype.toExponential".to_string()
            }
            Self::NumberPrototypeToFixed => "$builtin.Number.prototype.toFixed".to_string(),
            Self::NumberPrototypeToPrecision => "$builtin.Number.prototype.toPrecision".to_string(),
            Self::NumberPrototypeToString => "$builtin.Number.prototype.toString".to_string(),
            Self::NumberPrototypeToLocaleString => {
                "$builtin.Number.prototype.toLocaleString".to_string()
            }
            Self::NumberPrototypeValueOf => "$builtin.Number.prototype.valueOf".to_string(),
            Self::GlobalIsFinite => "$builtin.isFinite".to_string(),
            Self::GlobalIsNaN => "$builtin.isNaN".to_string(),
            Self::MathAbs => "$builtin.Math.abs".to_string(),
            Self::MathAcos => "$builtin.Math.acos".to_string(),
            Self::MathAcosh => "$builtin.Math.acosh".to_string(),
            Self::MathAsin => "$builtin.Math.asin".to_string(),
            Self::MathAsinh => "$builtin.Math.asinh".to_string(),
            Self::MathAtan => "$builtin.Math.atan".to_string(),
            Self::MathAtan2 => "$builtin.Math.atan2".to_string(),
            Self::MathAtanh => "$builtin.Math.atanh".to_string(),
            Self::MathCbrt => "$builtin.Math.cbrt".to_string(),
            Self::MathCeil => "$builtin.Math.ceil".to_string(),
            Self::MathClz32 => "$builtin.Math.clz32".to_string(),
            Self::MathCos => "$builtin.Math.cos".to_string(),
            Self::MathCosh => "$builtin.Math.cosh".to_string(),
            Self::MathExp => "$builtin.Math.exp".to_string(),
            Self::MathExpm1 => "$builtin.Math.expm1".to_string(),
            Self::MathF16Round => "$builtin.Math.f16round".to_string(),
            Self::MathFloor => "$builtin.Math.floor".to_string(),
            Self::MathFround => "$builtin.Math.fround".to_string(),
            Self::MathHypot => "$builtin.Math.hypot".to_string(),
            Self::MathImul => "$builtin.Math.imul".to_string(),
            Self::MathLog => "$builtin.Math.log".to_string(),
            Self::MathLog10 => "$builtin.Math.log10".to_string(),
            Self::MathLog1p => "$builtin.Math.log1p".to_string(),
            Self::MathLog2 => "$builtin.Math.log2".to_string(),
            Self::MathPow => "$builtin.Math.pow".to_string(),
            Self::MathRandom => "$builtin.Math.random".to_string(),
            Self::MathRound => "$builtin.Math.round".to_string(),
            Self::MathSign => "$builtin.Math.sign".to_string(),
            Self::MathSin => "$builtin.Math.sin".to_string(),
            Self::MathSinh => "$builtin.Math.sinh".to_string(),
            Self::MathSqrt => "$builtin.Math.sqrt".to_string(),
            Self::MathSumPrecise => "$builtin.Math.sumPrecise".to_string(),
            Self::MathTan => "$builtin.Math.tan".to_string(),
            Self::MathTanh => "$builtin.Math.tanh".to_string(),
            Self::MathTrunc => "$builtin.Math.trunc".to_string(),
            Self::MathMin => "$builtin.Math.min".to_string(),
            Self::MathMax => "$builtin.Math.max".to_string(),
            Self::StringConstructor => BUILTIN_STRING_FUNCTION_ID.to_string(),
            Self::StringPrototypeToString => {
                BUILTIN_STRING_PROTOTYPE_TO_STRING_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeValueOf => {
                BUILTIN_STRING_PROTOTYPE_VALUE_OF_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeCharAt => BUILTIN_STRING_PROTOTYPE_CHAR_AT_FUNCTION_ID.to_string(),
            Self::StringPrototypeCharCodeAt => {
                BUILTIN_STRING_PROTOTYPE_CHAR_CODE_AT_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeCodePointAt => {
                BUILTIN_STRING_PROTOTYPE_CODE_POINT_AT_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeAt => BUILTIN_STRING_PROTOTYPE_AT_FUNCTION_ID.to_string(),
            Self::StringPrototypeAnchor => BUILTIN_STRING_PROTOTYPE_ANCHOR_FUNCTION_ID.to_string(),
            Self::StringPrototypeBig => BUILTIN_STRING_PROTOTYPE_BIG_FUNCTION_ID.to_string(),
            Self::StringPrototypeBlink => BUILTIN_STRING_PROTOTYPE_BLINK_FUNCTION_ID.to_string(),
            Self::StringPrototypeBold => BUILTIN_STRING_PROTOTYPE_BOLD_FUNCTION_ID.to_string(),
            Self::StringPrototypeFixed => BUILTIN_STRING_PROTOTYPE_FIXED_FUNCTION_ID.to_string(),
            Self::StringPrototypeFontcolor => {
                BUILTIN_STRING_PROTOTYPE_FONTCOLOR_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeFontsize => {
                BUILTIN_STRING_PROTOTYPE_FONTSIZE_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeItalics => {
                BUILTIN_STRING_PROTOTYPE_ITALICS_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeLink => BUILTIN_STRING_PROTOTYPE_LINK_FUNCTION_ID.to_string(),
            Self::StringPrototypeSmall => BUILTIN_STRING_PROTOTYPE_SMALL_FUNCTION_ID.to_string(),
            Self::StringPrototypeStrike => BUILTIN_STRING_PROTOTYPE_STRIKE_FUNCTION_ID.to_string(),
            Self::StringPrototypeSub => BUILTIN_STRING_PROTOTYPE_SUB_FUNCTION_ID.to_string(),
            Self::StringPrototypeSubstr => BUILTIN_STRING_PROTOTYPE_SUBSTR_FUNCTION_ID.to_string(),
            Self::StringPrototypeSubstring => {
                BUILTIN_STRING_PROTOTYPE_SUBSTRING_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeSup => BUILTIN_STRING_PROTOTYPE_SUP_FUNCTION_ID.to_string(),
            Self::StringPrototypeMatch => BUILTIN_STRING_PROTOTYPE_MATCH_FUNCTION_ID.to_string(),
            Self::StringPrototypeMatchAll => {
                BUILTIN_STRING_PROTOTYPE_MATCH_ALL_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeReplace => {
                BUILTIN_STRING_PROTOTYPE_REPLACE_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeReplaceAll => {
                BUILTIN_STRING_PROTOTYPE_REPLACE_ALL_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeSearch => BUILTIN_STRING_PROTOTYPE_SEARCH_FUNCTION_ID.to_string(),
            Self::StringPrototypeIndexOf => {
                BUILTIN_STRING_PROTOTYPE_INDEX_OF_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeLastIndexOf => {
                BUILTIN_STRING_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeSlice => BUILTIN_STRING_PROTOTYPE_SLICE_FUNCTION_ID.to_string(),
            Self::StringPrototypeSplit => BUILTIN_STRING_PROTOTYPE_SPLIT_FUNCTION_ID.to_string(),
            Self::StringPrototypePadStart => {
                BUILTIN_STRING_PROTOTYPE_PAD_START_FUNCTION_ID.to_string()
            }
            Self::StringPrototypePadEnd => BUILTIN_STRING_PROTOTYPE_PAD_END_FUNCTION_ID.to_string(),
            Self::StringPrototypeRepeat => BUILTIN_STRING_PROTOTYPE_REPEAT_FUNCTION_ID.to_string(),
            Self::StringPrototypeEndsWith => {
                BUILTIN_STRING_PROTOTYPE_ENDS_WITH_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeIncludes => {
                BUILTIN_STRING_PROTOTYPE_INCLUDES_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeStartsWith => {
                BUILTIN_STRING_PROTOTYPE_STARTS_WITH_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeToUpperCase => {
                BUILTIN_STRING_PROTOTYPE_TO_UPPER_CASE_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeTrim => BUILTIN_STRING_PROTOTYPE_TRIM_FUNCTION_ID.to_string(),
            Self::StringPrototypeTrimStart => {
                BUILTIN_STRING_PROTOTYPE_TRIM_START_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeTrimEnd => {
                BUILTIN_STRING_PROTOTYPE_TRIM_END_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeIsWellFormed => {
                BUILTIN_STRING_PROTOTYPE_IS_WELL_FORMED_FUNCTION_ID.to_string()
            }
            Self::StringPrototypeToWellFormed => {
                BUILTIN_STRING_PROTOTYPE_TO_WELL_FORMED_FUNCTION_ID.to_string()
            }
            Self::BooleanConstructor => BUILTIN_BOOLEAN_FUNCTION_ID.to_string(),
            Self::SymbolConstructor => BUILTIN_SYMBOL_FUNCTION_ID.to_string(),
            Self::SymbolFor => BUILTIN_SYMBOL_FOR_FUNCTION_ID.to_string(),
            Self::SymbolKeyFor => BUILTIN_SYMBOL_KEY_FOR_FUNCTION_ID.to_string(),
            Self::SymbolPrototypeDescriptionGetter => {
                "$builtin.Symbol.prototype.description".to_string()
            }
            Self::SymbolPrototypeToString => "$builtin.Symbol.prototype.toString".to_string(),
            Self::SymbolPrototypeValueOf => "$builtin.Symbol.prototype.valueOf".to_string(),
            Self::SymbolPrototypeToPrimitive => {
                "$builtin.Symbol.prototype.toPrimitive".to_string()
            }
            Self::BooleanPrototypeToString => "$builtin.Boolean.prototype.toString".to_string(),
            Self::BooleanPrototypeValueOf => "$builtin.Boolean.prototype.valueOf".to_string(),
            Self::ErrorConstructor => BUILTIN_ERROR_FUNCTION_ID.to_string(),
            Self::ErrorIsError => BUILTIN_ERROR_IS_ERROR_FUNCTION_ID.to_string(),
            Self::EvalErrorConstructor => BUILTIN_EVAL_ERROR_FUNCTION_ID.to_string(),
            Self::AggregateErrorConstructor => BUILTIN_AGGREGATE_ERROR_FUNCTION_ID.to_string(),
            Self::SuppressedErrorConstructor => BUILTIN_SUPPRESSED_ERROR_FUNCTION_ID.to_string(),
            Self::RangeErrorConstructor => BUILTIN_RANGE_ERROR_FUNCTION_ID.to_string(),
            Self::SyntaxErrorConstructor => BUILTIN_SYNTAX_ERROR_FUNCTION_ID.to_string(),
            Self::TypeErrorConstructor => BUILTIN_TYPE_ERROR_FUNCTION_ID.to_string(),
            Self::URIErrorConstructor => BUILTIN_URI_ERROR_FUNCTION_ID.to_string(),
            Self::ReferenceErrorConstructor => BUILTIN_REFERENCE_ERROR_FUNCTION_ID.to_string(),
            Self::ErrorPrototypeToString => {
                BUILTIN_ERROR_PROTOTYPE_TO_STRING_FUNCTION_ID.to_string()
            }
            Self::ThrowTypeError => BUILTIN_THROW_TYPE_ERROR_FUNCTION_ID.to_string(),
            Self::BoundFunctionInvoker => BUILTIN_BOUND_FUNCTION_INVOKER_FUNCTION_ID.to_string(),
            Self::Escape => BUILTIN_ESCAPE_FUNCTION_ID.to_string(),
            Self::Unescape => BUILTIN_UNESCAPE_FUNCTION_ID.to_string(),
        }
    }

    pub fn from_function_id(function_id: &str) -> Option<Self> {
        match function_id {
            BUILTIN_FUNCTION_FUNCTION_ID => Some(Self::FunctionConstructor),
            BUILTIN_FUNCTION_PROTOTYPE_CALL_FUNCTION_ID => Some(Self::FunctionPrototypeCall),
            BUILTIN_FUNCTION_PROTOTYPE_APPLY_FUNCTION_ID => Some(Self::FunctionPrototypeApply),
            BUILTIN_FUNCTION_PROTOTYPE_BIND_FUNCTION_ID => Some(Self::FunctionPrototypeBind),
            BUILTIN_FUNCTION_PROTOTYPE_TO_STRING_FUNCTION_ID => {
                Some(Self::FunctionPrototypeToString)
            }
            BUILTIN_EVAL_FUNCTION_ID => Some(Self::EvalFunction),
            BUILTIN_OBJECT_FUNCTION_ID => Some(Self::ObjectConstructor),
            BUILTIN_OBJECT_CREATE_FUNCTION_ID => Some(Self::ObjectCreate),
            BUILTIN_OBJECT_GET_PROTOTYPE_OF_FUNCTION_ID => Some(Self::ObjectGetPrototypeOf),
            BUILTIN_OBJECT_SET_PROTOTYPE_OF_FUNCTION_ID => Some(Self::ObjectSetPrototypeOf),
            BUILTIN_OBJECT_DEFINE_PROPERTY_FUNCTION_ID => Some(Self::ObjectDefineProperty),
            BUILTIN_OBJECT_DEFINE_PROPERTIES_FUNCTION_ID => Some(Self::ObjectDefineProperties),
            BUILTIN_OBJECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID => {
                Some(Self::ObjectGetOwnPropertyDescriptor)
            }
            BUILTIN_OBJECT_GET_OWN_PROPERTY_NAMES_FUNCTION_ID => {
                Some(Self::ObjectGetOwnPropertyNames)
            }
            BUILTIN_OBJECT_GET_OWN_PROPERTY_SYMBOLS_FUNCTION_ID => {
                Some(Self::ObjectGetOwnPropertySymbols)
            }
            BUILTIN_OBJECT_KEYS_FUNCTION_ID => Some(Self::ObjectKeys),
            BUILTIN_OBJECT_VALUES_FUNCTION_ID => Some(Self::ObjectValues),
            BUILTIN_OBJECT_HAS_OWN_FUNCTION_ID => Some(Self::ObjectHasOwn),
            BUILTIN_OBJECT_IS_FUNCTION_ID => Some(Self::ObjectIs),
            BUILTIN_OBJECT_IS_SEALED_FUNCTION_ID => Some(Self::ObjectIsSealed),
            BUILTIN_OBJECT_IS_FROZEN_FUNCTION_ID => Some(Self::ObjectIsFrozen),
            BUILTIN_OBJECT_FREEZE_FUNCTION_ID => Some(Self::ObjectFreeze),
            BUILTIN_OBJECT_IS_EXTENSIBLE_FUNCTION_ID => Some(Self::ObjectIsExtensible),
            BUILTIN_OBJECT_PREVENT_EXTENSIONS_FUNCTION_ID => Some(Self::ObjectPreventExtensions),
            BUILTIN_OBJECT_PROTOTYPE_HAS_OWN_PROPERTY_FUNCTION_ID => {
                Some(Self::ObjectPrototypeHasOwnProperty)
            }
            BUILTIN_OBJECT_PROTOTYPE_PROPERTY_IS_ENUMERABLE_FUNCTION_ID => {
                Some(Self::ObjectPrototypePropertyIsEnumerable)
            }
            BUILTIN_OBJECT_PROTOTYPE_IS_PROTOTYPE_OF_FUNCTION_ID => {
                Some(Self::ObjectPrototypeIsPrototypeOf)
            }
            BUILTIN_OBJECT_PROTOTYPE_TO_STRING_FUNCTION_ID => Some(Self::ObjectPrototypeToString),
            BUILTIN_OBJECT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID => {
                Some(Self::ObjectPrototypeToLocaleString)
            }
            BUILTIN_OBJECT_PROTOTYPE_VALUE_OF_FUNCTION_ID => Some(Self::ObjectPrototypeValueOf),
            BUILTIN_PROXY_FUNCTION_ID => Some(Self::ProxyConstructor),
            BUILTIN_REFLECT_CONSTRUCT_FUNCTION_ID => Some(Self::ReflectConstruct),
            BUILTIN_REFLECT_APPLY_FUNCTION_ID => Some(Self::ReflectApply),
            BUILTIN_REFLECT_GET_FUNCTION_ID => Some(Self::ReflectGet),
            BUILTIN_REFLECT_GET_PROTOTYPE_OF_FUNCTION_ID => Some(Self::ReflectGetPrototypeOf),
            BUILTIN_REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID => {
                Some(Self::ReflectGetOwnPropertyDescriptor)
            }
            BUILTIN_REFLECT_SET_FUNCTION_ID => Some(Self::ReflectSet),
            BUILTIN_REFLECT_HAS_FUNCTION_ID => Some(Self::ReflectHas),
            BUILTIN_REFLECT_DEFINE_PROPERTY_FUNCTION_ID => Some(Self::ReflectDefineProperty),
            BUILTIN_REFLECT_DELETE_PROPERTY_FUNCTION_ID => Some(Self::ReflectDeleteProperty),
            BUILTIN_REFLECT_IS_EXTENSIBLE_FUNCTION_ID => Some(Self::ReflectIsExtensible),
            BUILTIN_REFLECT_PREVENT_EXTENSIONS_FUNCTION_ID => Some(Self::ReflectPreventExtensions),
            BUILTIN_REFLECT_SET_PROTOTYPE_OF_FUNCTION_ID => Some(Self::ReflectSetPrototypeOf),
            BUILTIN_REFLECT_OWN_KEYS_FUNCTION_ID => Some(Self::ReflectOwnKeys),
            BUILTIN_ARRAY_FUNCTION_ID => Some(Self::ArrayConstructor),
            BUILTIN_ARRAY_FROM_FUNCTION_ID => Some(Self::ArrayFrom),
            BUILTIN_ARRAY_OF_FUNCTION_ID => Some(Self::ArrayOf),
            BUILTIN_ARRAY_IS_ARRAY_FUNCTION_ID => Some(Self::ArrayIsArray),
            BUILTIN_ARRAY_SPECIES_GETTER_FUNCTION_ID => Some(Self::ArraySpeciesGetter),
            BUILTIN_ARRAY_PROTOTYPE_CONCAT_FUNCTION_ID => Some(Self::ArrayPrototypeConcat),
            BUILTIN_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID => {
                Some(Self::ArrayPrototypeToLocaleString)
            }
            BUILTIN_ARRAY_PROTOTYPE_FLAT_FUNCTION_ID => Some(Self::ArrayPrototypeFlat),
            BUILTIN_ARRAY_PROTOTYPE_FLAT_MAP_FUNCTION_ID => Some(Self::ArrayPrototypeFlatMap),
            BUILTIN_ARRAY_PROTOTYPE_AT_FUNCTION_ID => Some(Self::ArrayPrototypeAt),
            BUILTIN_ARRAY_PROTOTYPE_INCLUDES_FUNCTION_ID => Some(Self::ArrayPrototypeIncludes),
            BUILTIN_ARRAY_PROTOTYPE_INDEX_OF_FUNCTION_ID => Some(Self::ArrayPrototypeIndexOf),
            BUILTIN_ARRAY_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID => {
                Some(Self::ArrayPrototypeLastIndexOf)
            }
            BUILTIN_ARRAY_PROTOTYPE_FIND_FUNCTION_ID => Some(Self::ArrayPrototypeFind),
            BUILTIN_ARRAY_PROTOTYPE_FIND_INDEX_FUNCTION_ID => Some(Self::ArrayPrototypeFindIndex),
            BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_FUNCTION_ID => Some(Self::ArrayPrototypeFindLast),
            BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_INDEX_FUNCTION_ID => {
                Some(Self::ArrayPrototypeFindLastIndex)
            }
            BUILTIN_ARRAY_PROTOTYPE_EVERY_FUNCTION_ID => Some(Self::ArrayPrototypeEvery),
            BUILTIN_ARRAY_PROTOTYPE_SOME_FUNCTION_ID => Some(Self::ArrayPrototypeSome),
            BUILTIN_ARRAY_PROTOTYPE_FOR_EACH_FUNCTION_ID => Some(Self::ArrayPrototypeForEach),
            BUILTIN_ARRAY_PROTOTYPE_FILTER_FUNCTION_ID => Some(Self::ArrayPrototypeFilter),
            BUILTIN_ARRAY_PROTOTYPE_MAP_FUNCTION_ID => Some(Self::ArrayPrototypeMap),
            BUILTIN_ARRAY_PROTOTYPE_POP_FUNCTION_ID => Some(Self::ArrayPrototypePop),
            BUILTIN_ARRAY_PROTOTYPE_PUSH_FUNCTION_ID => Some(Self::ArrayPrototypePush),
            BUILTIN_ARRAY_PROTOTYPE_KEYS_FUNCTION_ID => Some(Self::ArrayPrototypeKeys),
            BUILTIN_ARRAY_PROTOTYPE_ENTRIES_FUNCTION_ID => Some(Self::ArrayPrototypeEntries),
            BUILTIN_ARRAY_PROTOTYPE_VALUES_FUNCTION_ID => Some(Self::ArrayPrototypeValues),
            BUILTIN_ARRAY_ITERATOR_NEXT_FUNCTION_ID => Some(Self::ArrayIteratorNext),
            BUILTIN_ARRAY_ITERATOR_IDENTITY_FUNCTION_ID => Some(Self::ArrayIteratorIdentity),
            BUILTIN_ITERATOR_FUNCTION_ID => Some(Self::IteratorConstructor),
            BUILTIN_ITERATOR_FROM_FUNCTION_ID => Some(Self::IteratorFrom),
            BUILTIN_ITERATOR_PROTOTYPE_TO_ARRAY_FUNCTION_ID => Some(Self::IteratorPrototypeToArray),
            BUILTIN_ITERATOR_PROTOTYPE_FOR_EACH_FUNCTION_ID => Some(Self::IteratorPrototypeForEach),
            BUILTIN_ITERATOR_PROTOTYPE_EVERY_FUNCTION_ID => Some(Self::IteratorPrototypeEvery),
            BUILTIN_ITERATOR_PROTOTYPE_SOME_FUNCTION_ID => Some(Self::IteratorPrototypeSome),
            BUILTIN_ITERATOR_PROTOTYPE_FIND_FUNCTION_ID => Some(Self::IteratorPrototypeFind),
            BUILTIN_ITERATOR_PROTOTYPE_REDUCE_FUNCTION_ID => Some(Self::IteratorPrototypeReduce),
            BUILTIN_ITERATOR_PROTOTYPE_MAP_FUNCTION_ID => Some(Self::IteratorPrototypeMap),
            BUILTIN_ITERATOR_MAP_NEXT_FUNCTION_ID => Some(Self::IteratorMapNext),
            BUILTIN_ITERATOR_MAP_RETURN_FUNCTION_ID => Some(Self::IteratorMapReturn),
            BUILTIN_ITERATOR_PROTOTYPE_FILTER_FUNCTION_ID => Some(Self::IteratorPrototypeFilter),
            BUILTIN_ITERATOR_FILTER_NEXT_FUNCTION_ID => Some(Self::IteratorFilterNext),
            BUILTIN_ITERATOR_FILTER_RETURN_FUNCTION_ID => Some(Self::IteratorFilterReturn),
            BUILTIN_ITERATOR_PROTOTYPE_FLAT_MAP_FUNCTION_ID => Some(Self::IteratorPrototypeFlatMap),
            BUILTIN_ITERATOR_FLAT_MAP_NEXT_FUNCTION_ID => Some(Self::IteratorFlatMapNext),
            BUILTIN_ITERATOR_FLAT_MAP_RETURN_FUNCTION_ID => Some(Self::IteratorFlatMapReturn),
            BUILTIN_ITERATOR_PROTOTYPE_TAKE_FUNCTION_ID => Some(Self::IteratorPrototypeTake),
            BUILTIN_ITERATOR_TAKE_NEXT_FUNCTION_ID => Some(Self::IteratorTakeNext),
            BUILTIN_ITERATOR_TAKE_RETURN_FUNCTION_ID => Some(Self::IteratorTakeReturn),
            BUILTIN_ITERATOR_PROTOTYPE_DROP_FUNCTION_ID => Some(Self::IteratorPrototypeDrop),
            BUILTIN_ITERATOR_DROP_NEXT_FUNCTION_ID => Some(Self::IteratorDropNext),
            BUILTIN_ITERATOR_DROP_RETURN_FUNCTION_ID => Some(Self::IteratorDropReturn),
            BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_GETTER_FUNCTION_ID => {
                Some(Self::IteratorPrototypeConstructorGetter)
            }
            BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_SETTER_FUNCTION_ID => {
                Some(Self::IteratorPrototypeConstructorSetter)
            }
            BUILTIN_ITERATOR_PROTOTYPE_SYMBOL_DISPOSE_FUNCTION_ID => {
                Some(Self::IteratorPrototypeSymbolDispose)
            }
            BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_GETTER_FUNCTION_ID => {
                Some(Self::IteratorPrototypeToStringTagGetter)
            }
            BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_SETTER_FUNCTION_ID => {
                Some(Self::IteratorPrototypeToStringTagSetter)
            }
            BUILTIN_ITERATOR_FROM_WRAPPER_NEXT_FUNCTION_ID => Some(Self::IteratorFromWrapperNext),
            BUILTIN_ITERATOR_FROM_WRAPPER_RETURN_FUNCTION_ID => {
                Some(Self::IteratorFromWrapperReturn)
            }
            BUILTIN_ARRAY_BUFFER_FUNCTION_ID => Some(Self::ArrayBufferConstructor),
            BUILTIN_SHARED_ARRAY_BUFFER_FUNCTION_ID => Some(Self::SharedArrayBufferConstructor),
            BUILTIN_ARRAY_BUFFER_IS_VIEW_FUNCTION_ID => Some(Self::ArrayBufferIsView),
            BUILTIN_ARRAY_BUFFER_SPECIES_GETTER_FUNCTION_ID => Some(Self::ArrayBufferSpeciesGetter),
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeByteLengthGetter)
            }
            BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID => {
                Some(Self::SharedArrayBufferPrototypeByteLengthGetter)
            }
            BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID => {
                Some(Self::SharedArrayBufferPrototypeMaxByteLengthGetter)
            }
            BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROWABLE_GETTER_FUNCTION_ID => {
                Some(Self::SharedArrayBufferPrototypeGrowableGetter)
            }
            BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROW_FUNCTION_ID => {
                Some(Self::SharedArrayBufferPrototypeGrow)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_DETACHED_GETTER_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeDetachedGetter)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeMaxByteLengthGetter)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZABLE_GETTER_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeResizableGetter)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZE_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeResize)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeSlice)
            }
            BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID => {
                Some(Self::SharedArrayBufferPrototypeSlice)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeTransfer)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_FIXED_LENGTH_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeTransferToFixedLength)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_IMMUTABLE_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeTransferToImmutable)
            }
            BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_TO_IMMUTABLE_FUNCTION_ID => {
                Some(Self::ArrayBufferPrototypeSliceToImmutable)
            }
            BUILTIN_DATA_VIEW_FUNCTION_ID => Some(Self::DataViewConstructor),
            BUILTIN_DATA_VIEW_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID => {
                Some(Self::DataViewPrototypeBufferGetter)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID => {
                Some(Self::DataViewPrototypeByteLengthGetter)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID => {
                Some(Self::DataViewPrototypeByteOffsetGetter)
            }
            BUILTIN_TYPED_ARRAY_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID => {
                Some(Self::TypedArrayPrototypeBufferGetter)
            }
            BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID => {
                Some(Self::TypedArrayPrototypeByteLengthGetter)
            }
            BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID => {
                Some(Self::TypedArrayPrototypeByteOffsetGetter)
            }
            BUILTIN_TYPED_ARRAY_PROTOTYPE_LENGTH_GETTER_FUNCTION_ID => {
                Some(Self::TypedArrayPrototypeLengthGetter)
            }
            BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_STRING_FUNCTION_ID => {
                Some(Self::TypedArrayPrototypeToString)
            }
            BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID => {
                Some(Self::TypedArrayPrototypeToLocaleString)
            }
            BUILTIN_TYPED_ARRAY_FROM_FUNCTION_ID => Some(Self::TypedArrayFrom),
            BUILTIN_TYPED_ARRAY_OF_FUNCTION_ID => Some(Self::TypedArrayOf),
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT8_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetUint8)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT8_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetUint8)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT8_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetInt8)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT8_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetInt8)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT16_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetUint16)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT16_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetUint16)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT16_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetInt16)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT16_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetInt16)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT32_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetUint32)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT32_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetUint32)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT32_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetInt32)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT32_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetInt32)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT16_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetFloat16)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT16_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetFloat16)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT32_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetFloat32)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT32_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetFloat32)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT64_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetFloat64)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT64_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetFloat64)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGINT64_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetBigInt64)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGINT64_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetBigInt64)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGUINT64_FUNCTION_ID => {
                Some(Self::DataViewPrototypeGetBigUint64)
            }
            BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGUINT64_FUNCTION_ID => {
                Some(Self::DataViewPrototypeSetBigUint64)
            }
            BUILTIN_DATE_FUNCTION_ID => Some(Self::DateConstructor),
            BUILTIN_DATE_NOW_FUNCTION_ID => Some(Self::DateNow),
            BUILTIN_DATE_UTC_FUNCTION_ID => Some(Self::DateUtc),
            BUILTIN_DATE_PROTOTYPE_GET_TIME_FUNCTION_ID => Some(Self::DatePrototypeGetTime),
            BUILTIN_DATE_PROTOTYPE_SET_TIME_FUNCTION_ID => Some(Self::DatePrototypeSetTime),
            BUILTIN_DATE_PROTOTYPE_VALUE_OF_FUNCTION_ID => Some(Self::DatePrototypeValueOf),
            BUILTIN_DATE_PROTOTYPE_GET_FULL_YEAR_FUNCTION_ID => {
                Some(Self::DatePrototypeGetFullYear)
            }
            BUILTIN_DATE_PROTOTYPE_GET_UTC_FULL_YEAR_FUNCTION_ID => {
                Some(Self::DatePrototypeGetUtcFullYear)
            }
            BUILTIN_DATE_PROTOTYPE_GET_MONTH_FUNCTION_ID => Some(Self::DatePrototypeGetMonth),
            BUILTIN_DATE_PROTOTYPE_GET_UTC_MONTH_FUNCTION_ID => {
                Some(Self::DatePrototypeGetUtcMonth)
            }
            BUILTIN_DATE_PROTOTYPE_GET_DATE_FUNCTION_ID => Some(Self::DatePrototypeGetDate),
            BUILTIN_DATE_PROTOTYPE_GET_UTC_DATE_FUNCTION_ID => Some(Self::DatePrototypeGetUtcDate),
            BUILTIN_DATE_PROTOTYPE_GET_DAY_FUNCTION_ID => Some(Self::DatePrototypeGetDay),
            BUILTIN_DATE_PROTOTYPE_GET_UTC_DAY_FUNCTION_ID => Some(Self::DatePrototypeGetUtcDay),
            BUILTIN_DATE_PROTOTYPE_GET_HOURS_FUNCTION_ID => Some(Self::DatePrototypeGetHours),
            BUILTIN_DATE_PROTOTYPE_GET_UTC_HOURS_FUNCTION_ID => {
                Some(Self::DatePrototypeGetUtcHours)
            }
            BUILTIN_DATE_PROTOTYPE_GET_MINUTES_FUNCTION_ID => Some(Self::DatePrototypeGetMinutes),
            BUILTIN_DATE_PROTOTYPE_GET_UTC_MINUTES_FUNCTION_ID => {
                Some(Self::DatePrototypeGetUtcMinutes)
            }
            BUILTIN_DATE_PROTOTYPE_GET_SECONDS_FUNCTION_ID => Some(Self::DatePrototypeGetSeconds),
            BUILTIN_DATE_PROTOTYPE_GET_UTC_SECONDS_FUNCTION_ID => {
                Some(Self::DatePrototypeGetUtcSeconds)
            }
            BUILTIN_DATE_PROTOTYPE_GET_MILLISECONDS_FUNCTION_ID => {
                Some(Self::DatePrototypeGetMilliseconds)
            }
            BUILTIN_DATE_PROTOTYPE_GET_UTC_MILLISECONDS_FUNCTION_ID => {
                Some(Self::DatePrototypeGetUtcMilliseconds)
            }
            BUILTIN_DATE_PROTOTYPE_GET_TIMEZONE_OFFSET_FUNCTION_ID => {
                Some(Self::DatePrototypeGetTimezoneOffset)
            }
            BUILTIN_DATE_PROTOTYPE_GET_YEAR_FUNCTION_ID => Some(Self::DatePrototypeGetYear),
            BUILTIN_DATE_PROTOTYPE_SET_YEAR_FUNCTION_ID => Some(Self::DatePrototypeSetYear),
            BUILTIN_DATE_PROTOTYPE_SET_FULL_YEAR_FUNCTION_ID => {
                Some(Self::DatePrototypeSetFullYear)
            }
            BUILTIN_DATE_PROTOTYPE_SET_UTC_FULL_YEAR_FUNCTION_ID => {
                Some(Self::DatePrototypeSetUtcFullYear)
            }
            BUILTIN_DATE_PROTOTYPE_SET_MONTH_FUNCTION_ID => Some(Self::DatePrototypeSetMonth),
            BUILTIN_DATE_PROTOTYPE_SET_UTC_MONTH_FUNCTION_ID => {
                Some(Self::DatePrototypeSetUtcMonth)
            }
            BUILTIN_DATE_PROTOTYPE_SET_DATE_FUNCTION_ID => Some(Self::DatePrototypeSetDate),
            BUILTIN_DATE_PROTOTYPE_SET_UTC_DATE_FUNCTION_ID => Some(Self::DatePrototypeSetUtcDate),
            BUILTIN_DATE_PROTOTYPE_SET_HOURS_FUNCTION_ID => Some(Self::DatePrototypeSetHours),
            BUILTIN_DATE_PROTOTYPE_SET_UTC_HOURS_FUNCTION_ID => {
                Some(Self::DatePrototypeSetUtcHours)
            }
            BUILTIN_DATE_PROTOTYPE_SET_MINUTES_FUNCTION_ID => Some(Self::DatePrototypeSetMinutes),
            BUILTIN_DATE_PROTOTYPE_SET_UTC_MINUTES_FUNCTION_ID => {
                Some(Self::DatePrototypeSetUtcMinutes)
            }
            BUILTIN_DATE_PROTOTYPE_SET_SECONDS_FUNCTION_ID => Some(Self::DatePrototypeSetSeconds),
            BUILTIN_DATE_PROTOTYPE_SET_UTC_SECONDS_FUNCTION_ID => {
                Some(Self::DatePrototypeSetUtcSeconds)
            }
            BUILTIN_DATE_PROTOTYPE_SET_MILLISECONDS_FUNCTION_ID => {
                Some(Self::DatePrototypeSetMilliseconds)
            }
            BUILTIN_DATE_PROTOTYPE_SET_UTC_MILLISECONDS_FUNCTION_ID => {
                Some(Self::DatePrototypeSetUtcMilliseconds)
            }
            BUILTIN_DATE_PROTOTYPE_TO_UTC_STRING_FUNCTION_ID => {
                Some(Self::DatePrototypeToUtcString)
            }
            BUILTIN_REGEXP_FUNCTION_ID => Some(Self::RegExpConstructor),
            BUILTIN_REGEXP_SPECIES_GETTER_FUNCTION_ID => Some(Self::RegExpSpeciesGetter),
            BUILTIN_REGEXP_LEGACY_STATIC_GETTER_FUNCTION_ID => Some(Self::RegExpLegacyStaticGetter),
            BUILTIN_REGEXP_LEGACY_STATIC_SETTER_FUNCTION_ID => Some(Self::RegExpLegacyStaticSetter),
            BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_FUNCTION_ID => {
                Some(Self::RegExpPrototypeSymbolMatch)
            }
            BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_FUNCTION_ID => {
                Some(Self::RegExpPrototypeSymbolMatchAll)
            }
            BUILTIN_REGEXP_PROTOTYPE_SYMBOL_SEARCH_FUNCTION_ID => {
                Some(Self::RegExpPrototypeSymbolSearch)
            }
            BUILTIN_REGEXP_ESCAPE_FUNCTION_ID => Some(Self::RegExpEscape),
            BUILTIN_JSON_PARSE_FUNCTION_ID => Some(Self::JsonParse),
            BUILTIN_JSON_STRINGIFY_FUNCTION_ID => Some(Self::JsonStringify),
            BUILTIN_JSON_RAW_JSON_FUNCTION_ID => Some(Self::JsonRawJson),
            BUILTIN_JSON_IS_RAW_JSON_FUNCTION_ID => Some(Self::JsonIsRawJson),
            BUILTIN_ATOMICS_ADD_FUNCTION_ID => Some(Self::AtomicsAdd),
            BUILTIN_ATOMICS_AND_FUNCTION_ID => Some(Self::AtomicsAnd),
            BUILTIN_ATOMICS_COMPARE_EXCHANGE_FUNCTION_ID => Some(Self::AtomicsCompareExchange),
            BUILTIN_ATOMICS_EXCHANGE_FUNCTION_ID => Some(Self::AtomicsExchange),
            BUILTIN_ATOMICS_LOAD_FUNCTION_ID => Some(Self::AtomicsLoad),
            BUILTIN_ATOMICS_NOTIFY_FUNCTION_ID => Some(Self::AtomicsNotify),
            BUILTIN_ATOMICS_OR_FUNCTION_ID => Some(Self::AtomicsOr),
            BUILTIN_ATOMICS_PAUSE_FUNCTION_ID => Some(Self::AtomicsPause),
            BUILTIN_ATOMICS_STORE_FUNCTION_ID => Some(Self::AtomicsStore),
            BUILTIN_ATOMICS_SUB_FUNCTION_ID => Some(Self::AtomicsSub),
            BUILTIN_ATOMICS_WAIT_FUNCTION_ID => Some(Self::AtomicsWait),
            BUILTIN_ATOMICS_WAIT_ASYNC_FUNCTION_ID => Some(Self::AtomicsWaitAsync),
            BUILTIN_ATOMICS_XOR_FUNCTION_ID => Some(Self::AtomicsXor),
            BUILTIN_ATOMICS_IS_LOCK_FREE_FUNCTION_ID => Some(Self::AtomicsIsLockFree),
            BUILTIN_FLOAT64_ARRAY_FUNCTION_ID => Some(Self::Float64ArrayConstructor),
            BUILTIN_FLOAT32_ARRAY_FUNCTION_ID => Some(Self::Float32ArrayConstructor),
            BUILTIN_INT32_ARRAY_FUNCTION_ID => Some(Self::Int32ArrayConstructor),
            BUILTIN_INT16_ARRAY_FUNCTION_ID => Some(Self::Int16ArrayConstructor),
            BUILTIN_INT8_ARRAY_FUNCTION_ID => Some(Self::Int8ArrayConstructor),
            BUILTIN_UINT32_ARRAY_FUNCTION_ID => Some(Self::Uint32ArrayConstructor),
            BUILTIN_UINT16_ARRAY_FUNCTION_ID => Some(Self::Uint16ArrayConstructor),
            BUILTIN_UINT8_ARRAY_FUNCTION_ID => Some(Self::Uint8ArrayConstructor),
            BUILTIN_UINT8_CLAMPED_ARRAY_FUNCTION_ID => Some(Self::Uint8ClampedArrayConstructor),
            BUILTIN_BIGINT64_ARRAY_FUNCTION_ID => Some(Self::BigInt64ArrayConstructor),
            BUILTIN_BIGUINT64_ARRAY_FUNCTION_ID => Some(Self::BigUint64ArrayConstructor),
            BUILTIN_BIGINT_FUNCTION_ID => Some(Self::BigIntConstructor),
            BUILTIN_BIGINT_AS_INT_N_FUNCTION_ID => Some(Self::BigIntAsIntN),
            BUILTIN_BIGINT_AS_UINT_N_FUNCTION_ID => Some(Self::BigIntAsUintN),
            BUILTIN_BIGINT_PROTOTYPE_TO_STRING_FUNCTION_ID => Some(Self::BigIntPrototypeToString),
            BUILTIN_BIGINT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID => {
                Some(Self::BigIntPrototypeToLocaleString)
            }
            BUILTIN_BIGINT_PROTOTYPE_VALUE_OF_FUNCTION_ID => Some(Self::BigIntPrototypeValueOf),
            BUILTIN_NUMBER_FUNCTION_ID => Some(Self::NumberConstructor),
            BUILTIN_NUMBER_IS_INTEGER_FUNCTION_ID => Some(Self::NumberIsInteger),
            "$builtin.Number.isSafeInteger" => Some(Self::NumberIsSafeInteger),
            "$builtin.Number.isFinite" => Some(Self::NumberIsFinite),
            "$builtin.Number.isNaN" => Some(Self::NumberIsNaN),
            "$builtin.Number.prototype.toExponential" => Some(Self::NumberPrototypeToExponential),
            "$builtin.Number.prototype.toFixed" => Some(Self::NumberPrototypeToFixed),
            "$builtin.Number.prototype.toPrecision" => Some(Self::NumberPrototypeToPrecision),
            "$builtin.Number.prototype.toString" => Some(Self::NumberPrototypeToString),
            "$builtin.Number.prototype.toLocaleString" => Some(Self::NumberPrototypeToLocaleString),
            "$builtin.Number.prototype.valueOf" => Some(Self::NumberPrototypeValueOf),
            "$builtin.isFinite" => Some(Self::GlobalIsFinite),
            "$builtin.isNaN" => Some(Self::GlobalIsNaN),
            "$builtin.Math.abs" => Some(Self::MathAbs),
            "$builtin.Math.acos" => Some(Self::MathAcos),
            "$builtin.Math.acosh" => Some(Self::MathAcosh),
            "$builtin.Math.asin" => Some(Self::MathAsin),
            "$builtin.Math.asinh" => Some(Self::MathAsinh),
            "$builtin.Math.atan" => Some(Self::MathAtan),
            "$builtin.Math.atan2" => Some(Self::MathAtan2),
            "$builtin.Math.atanh" => Some(Self::MathAtanh),
            "$builtin.Math.cbrt" => Some(Self::MathCbrt),
            "$builtin.Math.ceil" => Some(Self::MathCeil),
            "$builtin.Math.clz32" => Some(Self::MathClz32),
            "$builtin.Math.cos" => Some(Self::MathCos),
            "$builtin.Math.cosh" => Some(Self::MathCosh),
            "$builtin.Math.exp" => Some(Self::MathExp),
            "$builtin.Math.expm1" => Some(Self::MathExpm1),
            "$builtin.Math.f16round" => Some(Self::MathF16Round),
            "$builtin.Math.floor" => Some(Self::MathFloor),
            "$builtin.Math.fround" => Some(Self::MathFround),
            "$builtin.Math.hypot" => Some(Self::MathHypot),
            "$builtin.Math.imul" => Some(Self::MathImul),
            "$builtin.Math.log" => Some(Self::MathLog),
            "$builtin.Math.log10" => Some(Self::MathLog10),
            "$builtin.Math.log1p" => Some(Self::MathLog1p),
            "$builtin.Math.log2" => Some(Self::MathLog2),
            "$builtin.Math.pow" => Some(Self::MathPow),
            "$builtin.Math.random" => Some(Self::MathRandom),
            "$builtin.Math.round" => Some(Self::MathRound),
            "$builtin.Math.sign" => Some(Self::MathSign),
            "$builtin.Math.sin" => Some(Self::MathSin),
            "$builtin.Math.sinh" => Some(Self::MathSinh),
            "$builtin.Math.sqrt" => Some(Self::MathSqrt),
            "$builtin.Math.sumPrecise" => Some(Self::MathSumPrecise),
            "$builtin.Math.tan" => Some(Self::MathTan),
            "$builtin.Math.tanh" => Some(Self::MathTanh),
            "$builtin.Math.trunc" => Some(Self::MathTrunc),
            "$builtin.Math.min" => Some(Self::MathMin),
            "$builtin.Math.max" => Some(Self::MathMax),
            BUILTIN_STRING_FUNCTION_ID => Some(Self::StringConstructor),
            BUILTIN_STRING_PROTOTYPE_TO_STRING_FUNCTION_ID => Some(Self::StringPrototypeToString),
            BUILTIN_STRING_PROTOTYPE_VALUE_OF_FUNCTION_ID => Some(Self::StringPrototypeValueOf),
            BUILTIN_STRING_PROTOTYPE_CHAR_AT_FUNCTION_ID => Some(Self::StringPrototypeCharAt),
            BUILTIN_STRING_PROTOTYPE_CHAR_CODE_AT_FUNCTION_ID => {
                Some(Self::StringPrototypeCharCodeAt)
            }
            BUILTIN_STRING_PROTOTYPE_CODE_POINT_AT_FUNCTION_ID => {
                Some(Self::StringPrototypeCodePointAt)
            }
            BUILTIN_STRING_PROTOTYPE_AT_FUNCTION_ID => Some(Self::StringPrototypeAt),
            BUILTIN_STRING_PROTOTYPE_ANCHOR_FUNCTION_ID => Some(Self::StringPrototypeAnchor),
            BUILTIN_STRING_PROTOTYPE_BIG_FUNCTION_ID => Some(Self::StringPrototypeBig),
            BUILTIN_STRING_PROTOTYPE_BLINK_FUNCTION_ID => Some(Self::StringPrototypeBlink),
            BUILTIN_STRING_PROTOTYPE_BOLD_FUNCTION_ID => Some(Self::StringPrototypeBold),
            BUILTIN_STRING_PROTOTYPE_FIXED_FUNCTION_ID => Some(Self::StringPrototypeFixed),
            BUILTIN_STRING_PROTOTYPE_FONTCOLOR_FUNCTION_ID => Some(Self::StringPrototypeFontcolor),
            BUILTIN_STRING_PROTOTYPE_FONTSIZE_FUNCTION_ID => Some(Self::StringPrototypeFontsize),
            BUILTIN_STRING_PROTOTYPE_ITALICS_FUNCTION_ID => Some(Self::StringPrototypeItalics),
            BUILTIN_STRING_PROTOTYPE_LINK_FUNCTION_ID => Some(Self::StringPrototypeLink),
            BUILTIN_STRING_PROTOTYPE_SMALL_FUNCTION_ID => Some(Self::StringPrototypeSmall),
            BUILTIN_STRING_PROTOTYPE_STRIKE_FUNCTION_ID => Some(Self::StringPrototypeStrike),
            BUILTIN_STRING_PROTOTYPE_SUB_FUNCTION_ID => Some(Self::StringPrototypeSub),
            BUILTIN_STRING_PROTOTYPE_SUBSTR_FUNCTION_ID => Some(Self::StringPrototypeSubstr),
            BUILTIN_STRING_PROTOTYPE_SUBSTRING_FUNCTION_ID => Some(Self::StringPrototypeSubstring),
            BUILTIN_STRING_PROTOTYPE_SUP_FUNCTION_ID => Some(Self::StringPrototypeSup),
            BUILTIN_STRING_PROTOTYPE_MATCH_FUNCTION_ID => Some(Self::StringPrototypeMatch),
            BUILTIN_STRING_PROTOTYPE_MATCH_ALL_FUNCTION_ID => Some(Self::StringPrototypeMatchAll),
            BUILTIN_STRING_PROTOTYPE_REPLACE_FUNCTION_ID => Some(Self::StringPrototypeReplace),
            BUILTIN_STRING_PROTOTYPE_REPLACE_ALL_FUNCTION_ID => {
                Some(Self::StringPrototypeReplaceAll)
            }
            BUILTIN_STRING_PROTOTYPE_SEARCH_FUNCTION_ID => Some(Self::StringPrototypeSearch),
            BUILTIN_STRING_PROTOTYPE_INDEX_OF_FUNCTION_ID => Some(Self::StringPrototypeIndexOf),
            BUILTIN_STRING_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID => {
                Some(Self::StringPrototypeLastIndexOf)
            }
            BUILTIN_STRING_PROTOTYPE_SLICE_FUNCTION_ID => Some(Self::StringPrototypeSlice),
            BUILTIN_STRING_PROTOTYPE_SPLIT_FUNCTION_ID => Some(Self::StringPrototypeSplit),
            BUILTIN_STRING_PROTOTYPE_PAD_START_FUNCTION_ID => Some(Self::StringPrototypePadStart),
            BUILTIN_STRING_PROTOTYPE_PAD_END_FUNCTION_ID => Some(Self::StringPrototypePadEnd),
            BUILTIN_STRING_PROTOTYPE_REPEAT_FUNCTION_ID => Some(Self::StringPrototypeRepeat),
            BUILTIN_STRING_PROTOTYPE_ENDS_WITH_FUNCTION_ID => Some(Self::StringPrototypeEndsWith),
            BUILTIN_STRING_PROTOTYPE_INCLUDES_FUNCTION_ID => Some(Self::StringPrototypeIncludes),
            BUILTIN_STRING_PROTOTYPE_STARTS_WITH_FUNCTION_ID => {
                Some(Self::StringPrototypeStartsWith)
            }
            BUILTIN_STRING_PROTOTYPE_TO_UPPER_CASE_FUNCTION_ID => {
                Some(Self::StringPrototypeToUpperCase)
            }
            BUILTIN_STRING_PROTOTYPE_TRIM_FUNCTION_ID => Some(Self::StringPrototypeTrim),
            BUILTIN_STRING_PROTOTYPE_TRIM_START_FUNCTION_ID => Some(Self::StringPrototypeTrimStart),
            BUILTIN_STRING_PROTOTYPE_TRIM_END_FUNCTION_ID => Some(Self::StringPrototypeTrimEnd),
            BUILTIN_STRING_PROTOTYPE_IS_WELL_FORMED_FUNCTION_ID => {
                Some(Self::StringPrototypeIsWellFormed)
            }
            BUILTIN_STRING_PROTOTYPE_TO_WELL_FORMED_FUNCTION_ID => {
                Some(Self::StringPrototypeToWellFormed)
            }
            BUILTIN_BOOLEAN_FUNCTION_ID => Some(Self::BooleanConstructor),
            BUILTIN_SYMBOL_FUNCTION_ID => Some(Self::SymbolConstructor),
            BUILTIN_SYMBOL_FOR_FUNCTION_ID => Some(Self::SymbolFor),
            BUILTIN_SYMBOL_KEY_FOR_FUNCTION_ID => Some(Self::SymbolKeyFor),
            "$builtin.Symbol.prototype.description" => {
                Some(Self::SymbolPrototypeDescriptionGetter)
            }
            "$builtin.Symbol.prototype.toString" => Some(Self::SymbolPrototypeToString),
            "$builtin.Symbol.prototype.valueOf" => Some(Self::SymbolPrototypeValueOf),
            "$builtin.Symbol.prototype.toPrimitive" => Some(Self::SymbolPrototypeToPrimitive),
            "$builtin.Boolean.prototype.toString" => Some(Self::BooleanPrototypeToString),
            "$builtin.Boolean.prototype.valueOf" => Some(Self::BooleanPrototypeValueOf),
            BUILTIN_ERROR_FUNCTION_ID => Some(Self::ErrorConstructor),
            BUILTIN_ERROR_IS_ERROR_FUNCTION_ID => Some(Self::ErrorIsError),
            BUILTIN_EVAL_ERROR_FUNCTION_ID => Some(Self::EvalErrorConstructor),
            BUILTIN_AGGREGATE_ERROR_FUNCTION_ID => Some(Self::AggregateErrorConstructor),
            BUILTIN_SUPPRESSED_ERROR_FUNCTION_ID => Some(Self::SuppressedErrorConstructor),
            BUILTIN_RANGE_ERROR_FUNCTION_ID => Some(Self::RangeErrorConstructor),
            BUILTIN_SYNTAX_ERROR_FUNCTION_ID => Some(Self::SyntaxErrorConstructor),
            BUILTIN_TYPE_ERROR_FUNCTION_ID => Some(Self::TypeErrorConstructor),
            BUILTIN_URI_ERROR_FUNCTION_ID => Some(Self::URIErrorConstructor),
            BUILTIN_REFERENCE_ERROR_FUNCTION_ID => Some(Self::ReferenceErrorConstructor),
            BUILTIN_ERROR_PROTOTYPE_TO_STRING_FUNCTION_ID => Some(Self::ErrorPrototypeToString),
            BUILTIN_THROW_TYPE_ERROR_FUNCTION_ID => Some(Self::ThrowTypeError),
            BUILTIN_PROXY_REVOCABLE_FUNCTION_ID => Some(Self::ProxyRevocable),
            BUILTIN_PROXY_REVOKE_FUNCTION_ID => Some(Self::ProxyRevoke),
            BUILTIN_BOUND_FUNCTION_INVOKER_FUNCTION_ID => Some(Self::BoundFunctionInvoker),
            BUILTIN_ESCAPE_FUNCTION_ID => Some(Self::Escape),
            BUILTIN_UNESCAPE_FUNCTION_ID => Some(Self::Unescape),
            _ => None,
        }
    }

    pub const fn all_globals() -> &'static [Self] {
        &[
            Self::FunctionConstructor,
            Self::EvalFunction,
            Self::ObjectConstructor,
            Self::ProxyConstructor,
            Self::IteratorConstructor,
            Self::ArrayConstructor,
            Self::ArrayBufferConstructor,
            Self::SharedArrayBufferConstructor,
            Self::DataViewConstructor,
            Self::DateConstructor,
            Self::RegExpConstructor,
            Self::Float64ArrayConstructor,
            Self::Float32ArrayConstructor,
            Self::Int32ArrayConstructor,
            Self::Int16ArrayConstructor,
            Self::Int8ArrayConstructor,
            Self::Uint32ArrayConstructor,
            Self::Uint16ArrayConstructor,
            Self::Uint8ArrayConstructor,
            Self::Uint8ClampedArrayConstructor,
            Self::BigInt64ArrayConstructor,
            Self::BigUint64ArrayConstructor,
            Self::BigIntConstructor,
            Self::NumberConstructor,
            Self::GlobalIsFinite,
            Self::GlobalIsNaN,
            Self::StringConstructor,
            Self::BooleanConstructor,
            Self::SymbolConstructor,
            Self::ErrorConstructor,
            Self::EvalErrorConstructor,
            Self::AggregateErrorConstructor,
            Self::SuppressedErrorConstructor,
            Self::RangeErrorConstructor,
            Self::SyntaxErrorConstructor,
            Self::TypeErrorConstructor,
            Self::URIErrorConstructor,
            Self::ReferenceErrorConstructor,
            Self::Escape,
            Self::Unescape,
        ]
    }

    pub const fn all_functions() -> &'static [Self] {
        &[
            Self::FunctionConstructor,
            Self::FunctionPrototypeCall,
            Self::FunctionPrototypeApply,
            Self::FunctionPrototypeBind,
            Self::FunctionPrototypeToString,
            Self::EvalFunction,
            Self::ObjectConstructor,
            Self::ObjectCreate,
            Self::ObjectGetPrototypeOf,
            Self::ObjectSetPrototypeOf,
            Self::ObjectDefineProperty,
            Self::ObjectDefineProperties,
            Self::ObjectGetOwnPropertyDescriptor,
            Self::ObjectGetOwnPropertyNames,
            Self::ObjectGetOwnPropertySymbols,
            Self::ObjectKeys,
            Self::ObjectValues,
            Self::ObjectHasOwn,
            Self::ObjectIs,
            Self::ObjectIsSealed,
            Self::ObjectIsFrozen,
            Self::ObjectFreeze,
            Self::ObjectIsExtensible,
            Self::ObjectPreventExtensions,
            Self::ObjectPrototypeHasOwnProperty,
            Self::ObjectPrototypePropertyIsEnumerable,
            Self::ObjectPrototypeIsPrototypeOf,
            Self::ObjectPrototypeToString,
            Self::ObjectPrototypeToLocaleString,
            Self::ObjectPrototypeValueOf,
            Self::ProxyConstructor,
            Self::ProxyRevocable,
            Self::ProxyRevoke,
            Self::ReflectConstruct,
            Self::ReflectApply,
            Self::ReflectGet,
            Self::ReflectGetPrototypeOf,
            Self::ReflectGetOwnPropertyDescriptor,
            Self::ReflectSet,
            Self::ReflectHas,
            Self::ReflectDefineProperty,
            Self::ReflectDeleteProperty,
            Self::ReflectIsExtensible,
            Self::ReflectPreventExtensions,
            Self::ReflectSetPrototypeOf,
            Self::ReflectOwnKeys,
            Self::ArrayConstructor,
            Self::ArrayFrom,
            Self::ArrayOf,
            Self::ArrayIsArray,
            Self::ArraySpeciesGetter,
            Self::ArrayPrototypeConcat,
            Self::ArrayPrototypeToLocaleString,
            Self::ArrayPrototypeFlat,
            Self::ArrayPrototypeFlatMap,
            Self::ArrayPrototypeAt,
            Self::ArrayPrototypeIncludes,
            Self::ArrayPrototypeIndexOf,
            Self::ArrayPrototypeLastIndexOf,
            Self::ArrayPrototypeFind,
            Self::ArrayPrototypeFindIndex,
            Self::ArrayPrototypeFindLast,
            Self::ArrayPrototypeFindLastIndex,
            Self::ArrayPrototypeEvery,
            Self::ArrayPrototypeSome,
            Self::ArrayPrototypeForEach,
            Self::ArrayPrototypeFilter,
            Self::ArrayPrototypeMap,
            Self::ArrayPrototypePop,
            Self::ArrayPrototypePush,
            Self::ArrayPrototypeKeys,
            Self::ArrayPrototypeEntries,
            Self::ArrayPrototypeValues,
            Self::ArrayIteratorNext,
            Self::ArrayIteratorIdentity,
            Self::IteratorConstructor,
            Self::IteratorFrom,
            Self::IteratorPrototypeToArray,
            Self::IteratorPrototypeForEach,
            Self::IteratorPrototypeEvery,
            Self::IteratorPrototypeSome,
            Self::IteratorPrototypeFind,
            Self::IteratorPrototypeReduce,
            Self::IteratorPrototypeMap,
            Self::IteratorMapNext,
            Self::IteratorMapReturn,
            Self::IteratorPrototypeFilter,
            Self::IteratorFilterNext,
            Self::IteratorFilterReturn,
            Self::IteratorPrototypeFlatMap,
            Self::IteratorFlatMapNext,
            Self::IteratorFlatMapReturn,
            Self::IteratorPrototypeTake,
            Self::IteratorTakeNext,
            Self::IteratorTakeReturn,
            Self::IteratorPrototypeDrop,
            Self::IteratorDropNext,
            Self::IteratorDropReturn,
            Self::IteratorPrototypeConstructorGetter,
            Self::IteratorPrototypeConstructorSetter,
            Self::IteratorPrototypeSymbolDispose,
            Self::IteratorPrototypeToStringTagGetter,
            Self::IteratorPrototypeToStringTagSetter,
            Self::IteratorFromWrapperNext,
            Self::IteratorFromWrapperReturn,
            Self::ArrayBufferConstructor,
            Self::SharedArrayBufferConstructor,
            Self::ArrayBufferIsView,
            Self::ArrayBufferSpeciesGetter,
            Self::ArrayBufferPrototypeByteLengthGetter,
            Self::SharedArrayBufferPrototypeByteLengthGetter,
            Self::SharedArrayBufferPrototypeMaxByteLengthGetter,
            Self::SharedArrayBufferPrototypeGrowableGetter,
            Self::SharedArrayBufferPrototypeGrow,
            Self::ArrayBufferPrototypeDetachedGetter,
            Self::ArrayBufferPrototypeMaxByteLengthGetter,
            Self::ArrayBufferPrototypeResizableGetter,
            Self::ArrayBufferPrototypeResize,
            Self::ArrayBufferPrototypeSlice,
            Self::SharedArrayBufferPrototypeSlice,
            Self::ArrayBufferPrototypeTransfer,
            Self::ArrayBufferPrototypeTransferToFixedLength,
            Self::ArrayBufferPrototypeTransferToImmutable,
            Self::ArrayBufferPrototypeSliceToImmutable,
            Self::DataViewConstructor,
            Self::DataViewPrototypeBufferGetter,
            Self::DataViewPrototypeByteLengthGetter,
            Self::DataViewPrototypeByteOffsetGetter,
            Self::TypedArrayPrototypeBufferGetter,
            Self::TypedArrayPrototypeByteLengthGetter,
            Self::TypedArrayPrototypeByteOffsetGetter,
            Self::TypedArrayPrototypeLengthGetter,
            Self::TypedArrayPrototypeToString,
            Self::TypedArrayPrototypeToLocaleString,
            Self::TypedArrayFrom,
            Self::TypedArrayOf,
            Self::DataViewPrototypeGetUint8,
            Self::DataViewPrototypeSetUint8,
            Self::DataViewPrototypeGetInt8,
            Self::DataViewPrototypeSetInt8,
            Self::DataViewPrototypeGetUint16,
            Self::DataViewPrototypeSetUint16,
            Self::DataViewPrototypeGetInt16,
            Self::DataViewPrototypeSetInt16,
            Self::DataViewPrototypeGetUint32,
            Self::DataViewPrototypeSetUint32,
            Self::DataViewPrototypeGetInt32,
            Self::DataViewPrototypeSetInt32,
            Self::DataViewPrototypeGetFloat16,
            Self::DataViewPrototypeSetFloat16,
            Self::DataViewPrototypeGetFloat32,
            Self::DataViewPrototypeSetFloat32,
            Self::DataViewPrototypeGetFloat64,
            Self::DataViewPrototypeSetFloat64,
            Self::DataViewPrototypeGetBigInt64,
            Self::DataViewPrototypeSetBigInt64,
            Self::DataViewPrototypeGetBigUint64,
            Self::DataViewPrototypeSetBigUint64,
            Self::DateConstructor,
            Self::DateNow,
            Self::DateUtc,
            Self::DatePrototypeGetTime,
            Self::DatePrototypeSetTime,
            Self::DatePrototypeValueOf,
            Self::DatePrototypeGetFullYear,
            Self::DatePrototypeGetUtcFullYear,
            Self::DatePrototypeGetMonth,
            Self::DatePrototypeGetUtcMonth,
            Self::DatePrototypeGetDate,
            Self::DatePrototypeGetUtcDate,
            Self::DatePrototypeGetDay,
            Self::DatePrototypeGetUtcDay,
            Self::DatePrototypeGetHours,
            Self::DatePrototypeGetUtcHours,
            Self::DatePrototypeGetMinutes,
            Self::DatePrototypeGetUtcMinutes,
            Self::DatePrototypeGetSeconds,
            Self::DatePrototypeGetUtcSeconds,
            Self::DatePrototypeGetMilliseconds,
            Self::DatePrototypeGetUtcMilliseconds,
            Self::DatePrototypeGetTimezoneOffset,
            Self::DatePrototypeGetYear,
            Self::DatePrototypeSetYear,
            Self::DatePrototypeSetFullYear,
            Self::DatePrototypeSetUtcFullYear,
            Self::DatePrototypeSetMonth,
            Self::DatePrototypeSetUtcMonth,
            Self::DatePrototypeSetDate,
            Self::DatePrototypeSetUtcDate,
            Self::DatePrototypeSetHours,
            Self::DatePrototypeSetUtcHours,
            Self::DatePrototypeSetMinutes,
            Self::DatePrototypeSetUtcMinutes,
            Self::DatePrototypeSetSeconds,
            Self::DatePrototypeSetUtcSeconds,
            Self::DatePrototypeSetMilliseconds,
            Self::DatePrototypeSetUtcMilliseconds,
            Self::DatePrototypeToUtcString,
            Self::RegExpConstructor,
            Self::RegExpSpeciesGetter,
            Self::RegExpLegacyStaticGetter,
            Self::RegExpLegacyStaticSetter,
            Self::RegExpPrototypeSymbolMatch,
            Self::RegExpPrototypeSymbolMatchAll,
            Self::RegExpPrototypeSymbolSearch,
            Self::RegExpEscape,
            Self::JsonParse,
            Self::JsonStringify,
            Self::JsonRawJson,
            Self::JsonIsRawJson,
            Self::AtomicsAdd,
            Self::AtomicsAnd,
            Self::AtomicsCompareExchange,
            Self::AtomicsExchange,
            Self::AtomicsLoad,
            Self::AtomicsNotify,
            Self::AtomicsOr,
            Self::AtomicsPause,
            Self::AtomicsStore,
            Self::AtomicsSub,
            Self::AtomicsWait,
            Self::AtomicsWaitAsync,
            Self::AtomicsXor,
            Self::AtomicsIsLockFree,
            Self::Float64ArrayConstructor,
            Self::Float32ArrayConstructor,
            Self::Int32ArrayConstructor,
            Self::Int16ArrayConstructor,
            Self::Int8ArrayConstructor,
            Self::Uint32ArrayConstructor,
            Self::Uint16ArrayConstructor,
            Self::Uint8ArrayConstructor,
            Self::Uint8ClampedArrayConstructor,
            Self::BigInt64ArrayConstructor,
            Self::BigUint64ArrayConstructor,
            Self::BigIntConstructor,
            Self::BigIntAsIntN,
            Self::BigIntAsUintN,
            Self::BigIntPrototypeToString,
            Self::BigIntPrototypeToLocaleString,
            Self::BigIntPrototypeValueOf,
            Self::NumberConstructor,
            Self::NumberIsInteger,
            Self::NumberIsSafeInteger,
            Self::NumberIsFinite,
            Self::NumberIsNaN,
            Self::NumberPrototypeToExponential,
            Self::NumberPrototypeToFixed,
            Self::NumberPrototypeToPrecision,
            Self::NumberPrototypeToString,
            Self::NumberPrototypeToLocaleString,
            Self::NumberPrototypeValueOf,
            Self::GlobalIsFinite,
            Self::GlobalIsNaN,
            Self::MathAbs,
            Self::MathAcos,
            Self::MathAcosh,
            Self::MathAsin,
            Self::MathAsinh,
            Self::MathAtan,
            Self::MathAtan2,
            Self::MathAtanh,
            Self::MathCbrt,
            Self::MathCeil,
            Self::MathClz32,
            Self::MathCos,
            Self::MathCosh,
            Self::MathExp,
            Self::MathExpm1,
            Self::MathF16Round,
            Self::MathFloor,
            Self::MathFround,
            Self::MathHypot,
            Self::MathImul,
            Self::MathLog,
            Self::MathLog10,
            Self::MathLog1p,
            Self::MathLog2,
            Self::MathPow,
            Self::MathRandom,
            Self::MathRound,
            Self::MathSign,
            Self::MathSin,
            Self::MathSinh,
            Self::MathSqrt,
            Self::MathSumPrecise,
            Self::MathTan,
            Self::MathTanh,
            Self::MathTrunc,
            Self::MathMin,
            Self::MathMax,
            Self::StringConstructor,
            Self::StringPrototypeToString,
            Self::StringPrototypeValueOf,
            Self::StringPrototypeCharAt,
            Self::StringPrototypeCharCodeAt,
            Self::StringPrototypeCodePointAt,
            Self::StringPrototypeAt,
            Self::StringPrototypeAnchor,
            Self::StringPrototypeBig,
            Self::StringPrototypeBlink,
            Self::StringPrototypeBold,
            Self::StringPrototypeFixed,
            Self::StringPrototypeFontcolor,
            Self::StringPrototypeFontsize,
            Self::StringPrototypeItalics,
            Self::StringPrototypeLink,
            Self::StringPrototypeSmall,
            Self::StringPrototypeStrike,
            Self::StringPrototypeSub,
            Self::StringPrototypeSubstr,
            Self::StringPrototypeSubstring,
            Self::StringPrototypeSup,
            Self::StringPrototypeMatch,
            Self::StringPrototypeMatchAll,
            Self::StringPrototypeReplace,
            Self::StringPrototypeReplaceAll,
            Self::StringPrototypeSearch,
            Self::StringPrototypeIndexOf,
            Self::StringPrototypeLastIndexOf,
            Self::StringPrototypeSlice,
            Self::StringPrototypeSplit,
            Self::StringPrototypePadStart,
            Self::StringPrototypePadEnd,
            Self::StringPrototypeRepeat,
            Self::StringPrototypeEndsWith,
            Self::StringPrototypeIncludes,
            Self::StringPrototypeStartsWith,
            Self::StringPrototypeToUpperCase,
            Self::StringPrototypeTrim,
            Self::StringPrototypeTrimStart,
            Self::StringPrototypeTrimEnd,
            Self::StringPrototypeIsWellFormed,
            Self::StringPrototypeToWellFormed,
            Self::BooleanConstructor,
            Self::BooleanPrototypeToString,
            Self::BooleanPrototypeValueOf,
            Self::SymbolConstructor,
            Self::SymbolFor,
            Self::SymbolKeyFor,
            Self::SymbolPrototypeDescriptionGetter,
            Self::SymbolPrototypeToString,
            Self::SymbolPrototypeValueOf,
            Self::SymbolPrototypeToPrimitive,
            Self::ErrorConstructor,
            Self::ErrorIsError,
            Self::EvalErrorConstructor,
            Self::AggregateErrorConstructor,
            Self::SuppressedErrorConstructor,
            Self::RangeErrorConstructor,
            Self::SyntaxErrorConstructor,
            Self::TypeErrorConstructor,
            Self::URIErrorConstructor,
            Self::ReferenceErrorConstructor,
            Self::ErrorPrototypeToString,
            Self::ThrowTypeError,
            Self::BoundFunctionInvoker,
            Self::Escape,
            Self::Unescape,
        ]
    }

    pub const fn constructable(self) -> bool {
        matches!(
            self,
            Self::FunctionConstructor
                | Self::IteratorConstructor
                | Self::BoundFunctionInvoker
                | Self::ObjectConstructor
                | Self::ProxyConstructor
                | Self::ArrayConstructor
                | Self::ArrayBufferConstructor
                | Self::SharedArrayBufferConstructor
                | Self::DataViewConstructor
                | Self::DateConstructor
                | Self::RegExpConstructor
                | Self::Float64ArrayConstructor
                | Self::Float32ArrayConstructor
                | Self::Int32ArrayConstructor
                | Self::Int16ArrayConstructor
                | Self::Int8ArrayConstructor
                | Self::Uint32ArrayConstructor
                | Self::Uint16ArrayConstructor
                | Self::Uint8ArrayConstructor
                | Self::Uint8ClampedArrayConstructor
                | Self::BigInt64ArrayConstructor
                | Self::BigUint64ArrayConstructor
                | Self::NumberConstructor
                | Self::StringConstructor
                | Self::BooleanConstructor
                | Self::BigIntConstructor
                | Self::SymbolConstructor
                | Self::ErrorConstructor
                | Self::EvalErrorConstructor
                | Self::AggregateErrorConstructor
                | Self::SuppressedErrorConstructor
                | Self::RangeErrorConstructor
                | Self::SyntaxErrorConstructor
                | Self::TypeErrorConstructor
                | Self::URIErrorConstructor
                | Self::ReferenceErrorConstructor
        )
    }

    pub const fn is_error_constructor(self) -> bool {
        matches!(
            self,
            Self::ErrorConstructor
                | Self::EvalErrorConstructor
                | Self::AggregateErrorConstructor
                | Self::SuppressedErrorConstructor
                | Self::RangeErrorConstructor
                | Self::SyntaxErrorConstructor
                | Self::TypeErrorConstructor
                | Self::URIErrorConstructor
                | Self::ReferenceErrorConstructor
        )
    }

    pub const fn is_static_method(self) -> bool {
        matches!(
            self,
            Self::ObjectCreate
                | Self::ObjectGetPrototypeOf
                | Self::ObjectSetPrototypeOf
                | Self::ObjectDefineProperty
                | Self::ObjectDefineProperties
                | Self::ObjectGetOwnPropertyDescriptor
                | Self::ObjectGetOwnPropertyNames
                | Self::ObjectGetOwnPropertySymbols
                | Self::ObjectKeys
                | Self::ObjectValues
                | Self::ObjectHasOwn
                | Self::ObjectIs
                | Self::ProxyRevocable
                | Self::ReflectConstruct
                | Self::ReflectApply
                | Self::ReflectGet
                | Self::ReflectGetPrototypeOf
                | Self::ReflectSet
                | Self::ReflectHas
                | Self::ReflectDefineProperty
                | Self::ReflectDeleteProperty
                | Self::ReflectIsExtensible
                | Self::ReflectPreventExtensions
                | Self::ReflectSetPrototypeOf
                | Self::ReflectOwnKeys
                | Self::IteratorFrom
                | Self::ArrayFrom
                | Self::ArrayOf
                | Self::ArrayIsArray
                | Self::ArrayBufferIsView
                | Self::DateNow
                | Self::DateUtc
                | Self::BigIntAsIntN
                | Self::BigIntAsUintN
                | Self::NumberIsInteger
                | Self::NumberIsSafeInteger
                | Self::NumberIsFinite
                | Self::NumberIsNaN
                | Self::SymbolFor
                | Self::SymbolKeyFor
                | Self::MathAbs
                | Self::MathAcos
                | Self::MathAcosh
                | Self::MathAsin
                | Self::MathAsinh
                | Self::MathAtan
                | Self::MathAtan2
                | Self::MathAtanh
                | Self::MathCbrt
                | Self::MathCeil
                | Self::MathClz32
                | Self::MathCos
                | Self::MathCosh
                | Self::MathExp
                | Self::MathExpm1
                | Self::MathF16Round
                | Self::MathFloor
                | Self::MathFround
                | Self::MathHypot
                | Self::MathImul
                | Self::MathLog
                | Self::MathLog10
                | Self::MathLog1p
                | Self::MathLog2
                | Self::MathPow
                | Self::MathRandom
                | Self::MathRound
                | Self::MathSign
                | Self::MathSin
                | Self::MathSinh
                | Self::MathSqrt
                | Self::MathSumPrecise
                | Self::MathTan
                | Self::MathTanh
                | Self::MathTrunc
                | Self::MathMin
                | Self::MathMax
                | Self::ErrorIsError
        )
    }

    pub const fn is_boxed_primitive_constructor(self) -> bool {
        matches!(
            self,
            Self::NumberConstructor | Self::StringConstructor | Self::BooleanConstructor
        )
    }

    pub const fn string_html_method_name(self) -> Option<&'static str> {
        match self {
            Self::StringPrototypeAnchor => Some("anchor"),
            Self::StringPrototypeBig => Some("big"),
            Self::StringPrototypeBlink => Some("blink"),
            Self::StringPrototypeBold => Some("bold"),
            Self::StringPrototypeFixed => Some("fixed"),
            Self::StringPrototypeFontcolor => Some("fontcolor"),
            Self::StringPrototypeFontsize => Some("fontsize"),
            Self::StringPrototypeItalics => Some("italics"),
            Self::StringPrototypeLink => Some("link"),
            Self::StringPrototypeSmall => Some("small"),
            Self::StringPrototypeStrike => Some("strike"),
            Self::StringPrototypeSub => Some("sub"),
            Self::StringPrototypeSup => Some("sup"),
            _ => None,
        }
    }

    pub const fn string_prototype_method_name(self) -> Option<&'static str> {
        match self {
            Self::StringPrototypeToString => Some("toString"),
            Self::StringPrototypeValueOf => Some("valueOf"),
            Self::StringPrototypeCharAt => Some("charAt"),
            Self::StringPrototypeCharCodeAt => Some("charCodeAt"),
            Self::StringPrototypeCodePointAt => Some("codePointAt"),
            Self::StringPrototypeAt => Some("at"),
            Self::StringPrototypeSubstr => Some("substr"),
            Self::StringPrototypeSubstring => Some("substring"),
            Self::StringPrototypeMatch => Some("match"),
            Self::StringPrototypeMatchAll => Some("matchAll"),
            Self::StringPrototypeReplace => Some("replace"),
            Self::StringPrototypeReplaceAll => Some("replaceAll"),
            Self::StringPrototypeSearch => Some("search"),
            Self::StringPrototypeIndexOf => Some("indexOf"),
            Self::StringPrototypeLastIndexOf => Some("lastIndexOf"),
            Self::StringPrototypeSlice => Some("slice"),
            Self::StringPrototypeSplit => Some("split"),
            Self::StringPrototypePadStart => Some("padStart"),
            Self::StringPrototypePadEnd => Some("padEnd"),
            Self::StringPrototypeRepeat => Some("repeat"),
            Self::StringPrototypeEndsWith => Some("endsWith"),
            Self::StringPrototypeIncludes => Some("includes"),
            Self::StringPrototypeStartsWith => Some("startsWith"),
            Self::StringPrototypeToUpperCase => Some("toUpperCase"),
            Self::StringPrototypeTrim => Some("trim"),
            Self::StringPrototypeTrimStart => Some("trimStart"),
            Self::StringPrototypeTrimEnd => Some("trimEnd"),
            Self::StringPrototypeIsWellFormed => Some("isWellFormed"),
            Self::StringPrototypeToWellFormed => Some("toWellFormed"),
            _ => self.string_html_method_name(),
        }
    }

    pub const fn native_function_name(self) -> Option<&'static str> {
        match self {
            Self::FunctionConstructor => Some(FUNCTION_NAME),
            Self::FunctionPrototypeCall => Some("call"),
            Self::FunctionPrototypeApply => Some("apply"),
            Self::FunctionPrototypeBind => Some("bind"),
            Self::FunctionPrototypeToString => Some("toString"),
            Self::EvalFunction => Some("eval"),
            Self::ObjectConstructor => Some(OBJECT_NAME),
            Self::ObjectCreate => Some("create"),
            Self::ObjectGetPrototypeOf => Some("getPrototypeOf"),
            Self::ObjectSetPrototypeOf => Some("setPrototypeOf"),
            Self::ObjectDefineProperty => Some("defineProperty"),
            Self::ObjectDefineProperties => Some("defineProperties"),
            Self::ObjectGetOwnPropertyDescriptor => Some("getOwnPropertyDescriptor"),
            Self::ObjectGetOwnPropertyNames => Some("getOwnPropertyNames"),
            Self::ObjectGetOwnPropertySymbols => Some("getOwnPropertySymbols"),
            Self::ObjectKeys => Some("keys"),
            Self::ObjectValues => Some("values"),
            Self::ObjectHasOwn => Some("hasOwn"),
            Self::ObjectIs => Some("is"),
            Self::ObjectIsSealed => Some("isSealed"),
            Self::ObjectIsFrozen => Some("isFrozen"),
            Self::ObjectFreeze => Some("freeze"),
            Self::ObjectIsExtensible => Some("isExtensible"),
            Self::ObjectPreventExtensions => Some("preventExtensions"),
            Self::ObjectPrototypeHasOwnProperty => Some("hasOwnProperty"),
            Self::ObjectPrototypePropertyIsEnumerable => Some("propertyIsEnumerable"),
            Self::ObjectPrototypeIsPrototypeOf => Some("isPrototypeOf"),
            Self::ObjectPrototypeToString => Some("toString"),
            Self::ObjectPrototypeToLocaleString => Some("toLocaleString"),
            Self::ObjectPrototypeValueOf => Some("valueOf"),
            Self::ProxyConstructor => Some(PROXY_NAME),
            Self::ProxyRevocable => Some("revocable"),
            Self::ProxyRevoke => Some("revoke"),
            Self::ReflectConstruct => Some("construct"),
            Self::ReflectApply => Some("apply"),
            Self::ReflectGet => Some("get"),
            Self::ReflectGetPrototypeOf => Some("getPrototypeOf"),
            Self::ReflectGetOwnPropertyDescriptor => Some("getOwnPropertyDescriptor"),
            Self::ReflectSet => Some("set"),
            Self::ReflectHas => Some("has"),
            Self::ReflectDefineProperty => Some("defineProperty"),
            Self::ReflectDeleteProperty => Some("deleteProperty"),
            Self::ReflectIsExtensible => Some("isExtensible"),
            Self::ReflectPreventExtensions => Some("preventExtensions"),
            Self::ReflectSetPrototypeOf => Some("setPrototypeOf"),
            Self::ReflectOwnKeys => Some("ownKeys"),
            Self::ArrayConstructor => Some(ARRAY_NAME),
            Self::ArrayFrom => Some("from"),
            Self::ArrayOf => Some("of"),
            Self::ArrayIsArray => Some("isArray"),
            Self::ArraySpeciesGetter => Some("get [Symbol.species]"),
            Self::ArrayPrototypeConcat => Some("concat"),
            Self::ArrayPrototypeToLocaleString => Some("toLocaleString"),
            Self::ArrayPrototypeFlat => Some("flat"),
            Self::ArrayPrototypeFlatMap => Some("flatMap"),
            Self::ArrayPrototypeAt => Some("at"),
            Self::ArrayPrototypeIncludes => Some("includes"),
            Self::ArrayPrototypeIndexOf => Some("indexOf"),
            Self::ArrayPrototypeLastIndexOf => Some("lastIndexOf"),
            Self::ArrayPrototypeFind => Some("find"),
            Self::ArrayPrototypeFindIndex => Some("findIndex"),
            Self::ArrayPrototypeFindLast => Some("findLast"),
            Self::ArrayPrototypeFindLastIndex => Some("findLastIndex"),
            Self::ArrayPrototypeEvery => Some("every"),
            Self::ArrayPrototypeSome => Some("some"),
            Self::ArrayPrototypeForEach => Some("forEach"),
            Self::ArrayPrototypeFilter => Some("filter"),
            Self::ArrayPrototypeMap => Some("map"),
            Self::ArrayPrototypePop => Some("pop"),
            Self::ArrayPrototypePush => Some("push"),
            Self::ArrayPrototypeKeys => Some("keys"),
            Self::ArrayPrototypeEntries => Some("entries"),
            Self::ArrayPrototypeValues => Some("values"),
            Self::ArrayIteratorNext => Some("next"),
            Self::ArrayIteratorIdentity => Some("[Symbol.iterator]"),
            Self::IteratorConstructor => Some("Iterator"),
            Self::IteratorFrom => Some("from"),
            Self::IteratorPrototypeToArray => Some("toArray"),
            Self::IteratorPrototypeForEach => Some("forEach"),
            Self::IteratorPrototypeEvery => Some("every"),
            Self::IteratorPrototypeSome => Some("some"),
            Self::IteratorPrototypeFind => Some("find"),
            Self::IteratorPrototypeReduce => Some("reduce"),
            Self::IteratorPrototypeMap => Some("map"),
            Self::IteratorMapNext => Some("next"),
            Self::IteratorMapReturn => Some("return"),
            Self::IteratorPrototypeFilter => Some("filter"),
            Self::IteratorFilterNext => Some("next"),
            Self::IteratorFilterReturn => Some("return"),
            Self::IteratorPrototypeFlatMap => Some("flatMap"),
            Self::IteratorFlatMapNext => Some("next"),
            Self::IteratorFlatMapReturn => Some("return"),
            Self::IteratorPrototypeTake => Some("take"),
            Self::IteratorTakeNext => Some("next"),
            Self::IteratorTakeReturn => Some("return"),
            Self::IteratorPrototypeDrop => Some("drop"),
            Self::IteratorDropNext => Some("next"),
            Self::IteratorDropReturn => Some("return"),
            Self::IteratorPrototypeConstructorGetter => Some("get constructor"),
            Self::IteratorPrototypeConstructorSetter => Some("set constructor"),
            Self::IteratorPrototypeSymbolDispose => Some("[Symbol.dispose]"),
            Self::IteratorPrototypeToStringTagGetter => Some("get [Symbol.toStringTag]"),
            Self::IteratorPrototypeToStringTagSetter => Some("set [Symbol.toStringTag]"),
            Self::IteratorFromWrapperNext => Some("next"),
            Self::IteratorFromWrapperReturn => Some("return"),
            Self::ArrayBufferConstructor => Some(ARRAY_BUFFER_NAME),
            Self::SharedArrayBufferConstructor => Some(SHARED_ARRAY_BUFFER_NAME),
            Self::ArrayBufferIsView => Some("isView"),
            Self::ArrayBufferSpeciesGetter => Some("get [Symbol.species]"),
            Self::ArrayBufferPrototypeByteLengthGetter => Some("get byteLength"),
            Self::SharedArrayBufferPrototypeByteLengthGetter => Some("get byteLength"),
            Self::SharedArrayBufferPrototypeMaxByteLengthGetter => Some("get maxByteLength"),
            Self::SharedArrayBufferPrototypeGrowableGetter => Some("get growable"),
            Self::SharedArrayBufferPrototypeGrow => Some("grow"),
            Self::ArrayBufferPrototypeDetachedGetter => Some("get detached"),
            Self::ArrayBufferPrototypeMaxByteLengthGetter => Some("get maxByteLength"),
            Self::ArrayBufferPrototypeResizableGetter => Some("get resizable"),
            Self::ArrayBufferPrototypeResize => Some("resize"),
            Self::ArrayBufferPrototypeSlice => Some("slice"),
            Self::SharedArrayBufferPrototypeSlice => Some("slice"),
            Self::ArrayBufferPrototypeTransfer => Some("transfer"),
            Self::ArrayBufferPrototypeTransferToFixedLength => Some("transferToFixedLength"),
            Self::ArrayBufferPrototypeTransferToImmutable => Some("transferToImmutable"),
            Self::ArrayBufferPrototypeSliceToImmutable => Some("sliceToImmutable"),
            Self::DataViewConstructor => Some(DATA_VIEW_NAME),
            Self::DataViewPrototypeBufferGetter => Some("get buffer"),
            Self::DataViewPrototypeByteLengthGetter => Some("get byteLength"),
            Self::DataViewPrototypeByteOffsetGetter => Some("get byteOffset"),
            Self::JsonParse => Some("parse"),
            Self::JsonStringify => Some("stringify"),
            Self::JsonRawJson => Some("rawJSON"),
            Self::JsonIsRawJson => Some("isRawJSON"),
            Self::AtomicsAdd => Some("add"),
            Self::AtomicsAnd => Some("and"),
            Self::AtomicsCompareExchange => Some("compareExchange"),
            Self::AtomicsExchange => Some("exchange"),
            Self::AtomicsLoad => Some("load"),
            Self::AtomicsNotify => Some("notify"),
            Self::AtomicsOr => Some("or"),
            Self::AtomicsPause => Some("pause"),
            Self::AtomicsStore => Some("store"),
            Self::AtomicsSub => Some("sub"),
            Self::AtomicsWait => Some("wait"),
            Self::AtomicsWaitAsync => Some("waitAsync"),
            Self::AtomicsXor => Some("xor"),
            Self::AtomicsIsLockFree => Some("isLockFree"),
            Self::TypedArrayPrototypeBufferGetter => Some("get buffer"),
            Self::TypedArrayPrototypeByteLengthGetter => Some("get byteLength"),
            Self::TypedArrayPrototypeByteOffsetGetter => Some("get byteOffset"),
            Self::TypedArrayPrototypeLengthGetter => Some("get length"),
            Self::TypedArrayPrototypeToString => Some("toString"),
            Self::TypedArrayPrototypeToLocaleString => Some("toLocaleString"),
            Self::TypedArrayFrom => Some("from"),
            Self::TypedArrayOf => Some("of"),
            Self::DataViewPrototypeGetUint8 => Some("getUint8"),
            Self::DataViewPrototypeSetUint8 => Some("setUint8"),
            Self::DataViewPrototypeGetInt8 => Some("getInt8"),
            Self::DataViewPrototypeSetInt8 => Some("setInt8"),
            Self::DataViewPrototypeGetUint16 => Some("getUint16"),
            Self::DataViewPrototypeSetUint16 => Some("setUint16"),
            Self::DataViewPrototypeGetInt16 => Some("getInt16"),
            Self::DataViewPrototypeSetInt16 => Some("setInt16"),
            Self::DataViewPrototypeGetUint32 => Some("getUint32"),
            Self::DataViewPrototypeSetUint32 => Some("setUint32"),
            Self::DataViewPrototypeGetInt32 => Some("getInt32"),
            Self::DataViewPrototypeSetInt32 => Some("setInt32"),
            Self::DataViewPrototypeGetFloat16 => Some("getFloat16"),
            Self::DataViewPrototypeSetFloat16 => Some("setFloat16"),
            Self::DataViewPrototypeGetFloat32 => Some("getFloat32"),
            Self::DataViewPrototypeSetFloat32 => Some("setFloat32"),
            Self::DataViewPrototypeGetFloat64 => Some("getFloat64"),
            Self::DataViewPrototypeSetFloat64 => Some("setFloat64"),
            Self::DataViewPrototypeGetBigInt64 => Some("getBigInt64"),
            Self::DataViewPrototypeSetBigInt64 => Some("setBigInt64"),
            Self::DataViewPrototypeGetBigUint64 => Some("getBigUint64"),
            Self::DataViewPrototypeSetBigUint64 => Some("setBigUint64"),
            Self::DateConstructor => Some(DATE_NAME),
            Self::DateNow => Some("now"),
            Self::DateUtc => Some("UTC"),
            Self::DatePrototypeGetTime => Some("getTime"),
            Self::DatePrototypeSetTime => Some("setTime"),
            Self::DatePrototypeValueOf => Some("valueOf"),
            Self::DatePrototypeGetFullYear => Some("getFullYear"),
            Self::DatePrototypeGetUtcFullYear => Some("getUTCFullYear"),
            Self::DatePrototypeGetMonth => Some("getMonth"),
            Self::DatePrototypeGetUtcMonth => Some("getUTCMonth"),
            Self::DatePrototypeGetDate => Some("getDate"),
            Self::DatePrototypeGetUtcDate => Some("getUTCDate"),
            Self::DatePrototypeGetDay => Some("getDay"),
            Self::DatePrototypeGetUtcDay => Some("getUTCDay"),
            Self::DatePrototypeGetHours => Some("getHours"),
            Self::DatePrototypeGetUtcHours => Some("getUTCHours"),
            Self::DatePrototypeGetMinutes => Some("getMinutes"),
            Self::DatePrototypeGetUtcMinutes => Some("getUTCMinutes"),
            Self::DatePrototypeGetSeconds => Some("getSeconds"),
            Self::DatePrototypeGetUtcSeconds => Some("getUTCSeconds"),
            Self::DatePrototypeGetMilliseconds => Some("getMilliseconds"),
            Self::DatePrototypeGetUtcMilliseconds => Some("getUTCMilliseconds"),
            Self::DatePrototypeGetTimezoneOffset => Some("getTimezoneOffset"),
            Self::DatePrototypeGetYear => Some("getYear"),
            Self::DatePrototypeSetYear => Some("setYear"),
            Self::DatePrototypeSetFullYear => Some("setFullYear"),
            Self::DatePrototypeSetUtcFullYear => Some("setUTCFullYear"),
            Self::DatePrototypeSetMonth => Some("setMonth"),
            Self::DatePrototypeSetUtcMonth => Some("setUTCMonth"),
            Self::DatePrototypeSetDate => Some("setDate"),
            Self::DatePrototypeSetUtcDate => Some("setUTCDate"),
            Self::DatePrototypeSetHours => Some("setHours"),
            Self::DatePrototypeSetUtcHours => Some("setUTCHours"),
            Self::DatePrototypeSetMinutes => Some("setMinutes"),
            Self::DatePrototypeSetUtcMinutes => Some("setUTCMinutes"),
            Self::DatePrototypeSetSeconds => Some("setSeconds"),
            Self::DatePrototypeSetUtcSeconds => Some("setUTCSeconds"),
            Self::DatePrototypeSetMilliseconds => Some("setMilliseconds"),
            Self::DatePrototypeSetUtcMilliseconds => Some("setUTCMilliseconds"),
            Self::DatePrototypeToUtcString => Some("toUTCString"),
            Self::RegExpConstructor => Some(REGEXP_NAME),
            Self::RegExpSpeciesGetter => Some("get [Symbol.species]"),
            Self::RegExpLegacyStaticGetter => Some("get RegExp legacy static"),
            Self::RegExpLegacyStaticSetter => Some("set RegExp legacy static"),
            Self::RegExpPrototypeSymbolMatch => Some("[Symbol.match]"),
            Self::RegExpPrototypeSymbolMatchAll => Some("[Symbol.matchAll]"),
            Self::RegExpPrototypeSymbolSearch => Some("[Symbol.search]"),
            Self::RegExpEscape => Some("escape"),
            Self::Float64ArrayConstructor => Some(FLOAT64_ARRAY_NAME),
            Self::Float32ArrayConstructor => Some(FLOAT32_ARRAY_NAME),
            Self::Int32ArrayConstructor => Some(INT32_ARRAY_NAME),
            Self::Int16ArrayConstructor => Some(INT16_ARRAY_NAME),
            Self::Int8ArrayConstructor => Some(INT8_ARRAY_NAME),
            Self::Uint32ArrayConstructor => Some(UINT32_ARRAY_NAME),
            Self::Uint16ArrayConstructor => Some(UINT16_ARRAY_NAME),
            Self::Uint8ArrayConstructor => Some(UINT8_ARRAY_NAME),
            Self::Uint8ClampedArrayConstructor => Some(UINT8_CLAMPED_ARRAY_NAME),
            Self::BigInt64ArrayConstructor => Some(BIGINT64_ARRAY_NAME),
            Self::BigUint64ArrayConstructor => Some(BIGUINT64_ARRAY_NAME),
            Self::BigIntConstructor => Some(BIGINT_NAME),
            Self::BigIntAsIntN => Some("asIntN"),
            Self::BigIntAsUintN => Some("asUintN"),
            Self::BigIntPrototypeToString => Some("toString"),
            Self::BigIntPrototypeToLocaleString => Some("toLocaleString"),
            Self::BigIntPrototypeValueOf => Some("valueOf"),
            Self::NumberConstructor => Some(NUMBER_NAME),
            Self::NumberIsInteger => Some("isInteger"),
            Self::NumberIsSafeInteger => Some("isSafeInteger"),
            Self::NumberIsFinite => Some("isFinite"),
            Self::NumberIsNaN => Some("isNaN"),
            Self::NumberPrototypeToExponential => Some("toExponential"),
            Self::NumberPrototypeToFixed => Some("toFixed"),
            Self::NumberPrototypeToPrecision => Some("toPrecision"),
            Self::NumberPrototypeToString => Some("toString"),
            Self::NumberPrototypeToLocaleString => Some("toLocaleString"),
            Self::NumberPrototypeValueOf => Some("valueOf"),
            Self::GlobalIsFinite => Some("isFinite"),
            Self::GlobalIsNaN => Some("isNaN"),
            Self::MathAbs => Some("abs"),
            Self::MathAcos => Some("acos"),
            Self::MathAcosh => Some("acosh"),
            Self::MathAsin => Some("asin"),
            Self::MathAsinh => Some("asinh"),
            Self::MathAtan => Some("atan"),
            Self::MathAtan2 => Some("atan2"),
            Self::MathAtanh => Some("atanh"),
            Self::MathCbrt => Some("cbrt"),
            Self::MathCeil => Some("ceil"),
            Self::MathClz32 => Some("clz32"),
            Self::MathCos => Some("cos"),
            Self::MathCosh => Some("cosh"),
            Self::MathExp => Some("exp"),
            Self::MathExpm1 => Some("expm1"),
            Self::MathF16Round => Some("f16round"),
            Self::MathFloor => Some("floor"),
            Self::MathFround => Some("fround"),
            Self::MathHypot => Some("hypot"),
            Self::MathImul => Some("imul"),
            Self::MathLog => Some("log"),
            Self::MathLog10 => Some("log10"),
            Self::MathLog1p => Some("log1p"),
            Self::MathLog2 => Some("log2"),
            Self::MathPow => Some("pow"),
            Self::MathRandom => Some("random"),
            Self::MathRound => Some("round"),
            Self::MathSign => Some("sign"),
            Self::MathSin => Some("sin"),
            Self::MathSinh => Some("sinh"),
            Self::MathSqrt => Some("sqrt"),
            Self::MathSumPrecise => Some("sumPrecise"),
            Self::MathTan => Some("tan"),
            Self::MathTanh => Some("tanh"),
            Self::MathTrunc => Some("trunc"),
            Self::MathMin => Some("min"),
            Self::MathMax => Some("max"),
            Self::StringConstructor => Some(STRING_NAME),
            Self::StringPrototypeToString => Some("toString"),
            Self::StringPrototypeValueOf => Some("valueOf"),
            Self::StringPrototypeCharAt => Some("charAt"),
            Self::StringPrototypeCharCodeAt => Some("charCodeAt"),
            Self::StringPrototypeCodePointAt => Some("codePointAt"),
            Self::StringPrototypeAt => Some("at"),
            Self::StringPrototypeAnchor => Some("anchor"),
            Self::StringPrototypeBig => Some("big"),
            Self::StringPrototypeBlink => Some("blink"),
            Self::StringPrototypeBold => Some("bold"),
            Self::StringPrototypeFixed => Some("fixed"),
            Self::StringPrototypeFontcolor => Some("fontcolor"),
            Self::StringPrototypeFontsize => Some("fontsize"),
            Self::StringPrototypeItalics => Some("italics"),
            Self::StringPrototypeLink => Some("link"),
            Self::StringPrototypeSmall => Some("small"),
            Self::StringPrototypeStrike => Some("strike"),
            Self::StringPrototypeSub => Some("sub"),
            Self::StringPrototypeSubstr => Some("substr"),
            Self::StringPrototypeSubstring => Some("substring"),
            Self::StringPrototypeSup => Some("sup"),
            Self::StringPrototypeMatch => Some("match"),
            Self::StringPrototypeMatchAll => Some("matchAll"),
            Self::StringPrototypeReplace => Some("replace"),
            Self::StringPrototypeReplaceAll => Some("replaceAll"),
            Self::StringPrototypeSearch => Some("search"),
            Self::StringPrototypeIndexOf => Some("indexOf"),
            Self::StringPrototypeLastIndexOf => Some("lastIndexOf"),
            Self::StringPrototypeSlice => Some("slice"),
            Self::StringPrototypeSplit => Some("split"),
            Self::StringPrototypePadStart => Some("padStart"),
            Self::StringPrototypePadEnd => Some("padEnd"),
            Self::StringPrototypeRepeat => Some("repeat"),
            Self::StringPrototypeEndsWith => Some("endsWith"),
            Self::StringPrototypeIncludes => Some("includes"),
            Self::StringPrototypeStartsWith => Some("startsWith"),
            Self::StringPrototypeToUpperCase => Some("toUpperCase"),
            Self::StringPrototypeTrim => Some("trim"),
            Self::StringPrototypeTrimStart => Some("trimStart"),
            Self::StringPrototypeTrimEnd => Some("trimEnd"),
            Self::StringPrototypeIsWellFormed => Some("isWellFormed"),
            Self::StringPrototypeToWellFormed => Some("toWellFormed"),
            Self::BooleanConstructor => Some(BOOLEAN_NAME),
            Self::SymbolConstructor => Some(SYMBOL_NAME),
            Self::SymbolFor => Some("for"),
            Self::SymbolKeyFor => Some("keyFor"),
            Self::SymbolPrototypeDescriptionGetter => Some("get description"),
            Self::SymbolPrototypeToString => Some("toString"),
            Self::SymbolPrototypeValueOf => Some("valueOf"),
            Self::SymbolPrototypeToPrimitive => Some("[Symbol.toPrimitive]"),
            Self::BooleanPrototypeToString => Some("toString"),
            Self::BooleanPrototypeValueOf => Some("valueOf"),
            Self::ErrorConstructor => Some(ERROR_NAME),
            Self::ErrorIsError => Some("isError"),
            Self::EvalErrorConstructor => Some(EVAL_ERROR_NAME),
            Self::AggregateErrorConstructor => Some(AGGREGATE_ERROR_NAME),
            Self::SuppressedErrorConstructor => Some(SUPPRESSED_ERROR_NAME),
            Self::RangeErrorConstructor => Some(RANGE_ERROR_NAME),
            Self::SyntaxErrorConstructor => Some(SYNTAX_ERROR_NAME),
            Self::TypeErrorConstructor => Some(TYPE_ERROR_NAME),
            Self::URIErrorConstructor => Some(URI_ERROR_NAME),
            Self::ReferenceErrorConstructor => Some(REFERENCE_ERROR_NAME),
            Self::ErrorPrototypeToString => Some("toString"),
            Self::ThrowTypeError => Some(""),
            Self::BoundFunctionInvoker => None,
            Self::Escape => Some(ESCAPE_NAME),
            Self::Unescape => Some(UNESCAPE_NAME),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallableToStringRepresentation {
    ExactSource(String),
    NativeNamed(String),
    NativeAnonymous,
}

impl CallableToStringRepresentation {
    pub fn materialize(&self) -> String {
        match self {
            Self::ExactSource(source) => source.clone(),
            Self::NativeNamed(name) => format!("function {name}() {{ [native code] }}"),
            Self::NativeAnonymous => "function () { [native code] }".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_builtin_ids_round_trip_through_function_ids() {
        for builtin in [
            HostBuiltinId::Print,
            HostBuiltinId::Gc,
            HostBuiltinId::AssertThrows,
            HostBuiltinId::IsConstructor,
            HostBuiltinId::CreateRealm,
            HostBuiltinId::ParseInt,
            HostBuiltinId::ParseFloat,
            HostBuiltinId::DetachArrayBuffer,
        ] {
            let function_id = builtin.function_id();
            assert_eq!(
                HostBuiltinId::from_function_id(&function_id),
                Some(builtin),
                "{function_id}"
            );
            assert!(!builtin.as_str().is_empty());
        }
    }

    #[test]
    fn callable_to_string_representations_materialize_spec_shapes() {
        assert_eq!(
            CallableToStringRepresentation::ExactSource("function f() {}".to_string())
                .materialize(),
            "function f() {}"
        );
        assert_eq!(
            CallableToStringRepresentation::NativeNamed("Array".to_string()).materialize(),
            "function Array() { [native code] }"
        );
        assert_eq!(
            CallableToStringRepresentation::NativeAnonymous.materialize(),
            "function () { [native code] }"
        );
    }
}
