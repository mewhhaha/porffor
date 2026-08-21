// This registry is the sole source for StandardBuiltinId and its metadata.
//
// Row order is declaration order and therefore the derived `Ord` contract.
// `FunctionOrdinal` preserves the independent Wasm function-index order;
// `GlobalOrdinal` preserves the independent global installation order.
// Choose the ordinals and installer deliberately when adding a builtin. The
// generated const checks reject duplicate or missing ordinals and duplicate
// function IDs; the mandatory installer field makes bootstrap participation a
// compile-time choice rather than an append-only backend catch-all.
use super::*;

/// The family-specific realm installer, if any, run after a builtin's common
/// function/prototype initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardBuiltinInstaller {
    None,
    Function,
    Promise,
    Map,
    WeakMap,
    WeakSet,
    WeakRef,
    FinalizationRegistry,
    AsyncDisposableStack,
    DisposableStack,
    Set,
    Object,
    Proxy,
    RegExp,
    Iterator,
    Array,
    String,
    ArrayBuffer,
    DataView,
    TemporalInstant,
    TemporalZonedDateTime,
    TemporalPlainDate,
    TemporalDuration,
    TemporalPlainTime,
    TemporalPlainDateTime,
    TemporalPlainYearMonth,
    TemporalPlainMonthDay,
    IntlLocale,
    IntlDateTimeFormat,
    Date,
    Error,
    BigInt,
    Symbol,
    Number,
    Boolean,
}

standard_builtin_catalog! {
    FunctionConstructor {
        function: FunctionOrdinal(0) => BUILTIN_FUNCTION_FUNCTION_ID,
        global: GlobalOrdinal(0),
        global_name: FUNCTION_NAME,
        debug: FUNCTION_NAME,
        flags: [CONSTRUCTABLE],
        installer: Function,
        native: FUNCTION_NAME,
    }
    FunctionPrototypeCall {
        function: FunctionOrdinal(1) => BUILTIN_FUNCTION_PROTOTYPE_CALL_FUNCTION_ID,
        debug: "Function.prototype.call",
        flags: [],
        installer: None,
        native: "call",
    }
    FunctionPrototypeApply {
        function: FunctionOrdinal(2) => BUILTIN_FUNCTION_PROTOTYPE_APPLY_FUNCTION_ID,
        debug: "Function.prototype.apply",
        flags: [],
        installer: None,
        native: "apply",
    }
    FunctionPrototypeBind {
        function: FunctionOrdinal(3) => BUILTIN_FUNCTION_PROTOTYPE_BIND_FUNCTION_ID,
        debug: "Function.prototype.bind",
        flags: [],
        installer: None,
        native: "bind",
    }
    FunctionPrototypeToString {
        function: FunctionOrdinal(4) => BUILTIN_FUNCTION_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Function.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    EvalFunction {
        function: FunctionOrdinal(5) => BUILTIN_EVAL_FUNCTION_ID,
        global: GlobalOrdinal(1),
        global_name: "eval",
        debug: "eval",
        flags: [],
        installer: None,
        native: "eval",
    }
    ObjectConstructor {
        function: FunctionOrdinal(6) => BUILTIN_OBJECT_FUNCTION_ID,
        global: GlobalOrdinal(2),
        global_name: OBJECT_NAME,
        debug: OBJECT_NAME,
        flags: [CONSTRUCTABLE],
        installer: Object,
        native: OBJECT_NAME,
    }
    ObjectGroupBy {
        function: FunctionOrdinal(7) => BUILTIN_OBJECT_GROUP_BY_FUNCTION_ID,
        debug: "Object.groupBy",
        flags: [],
        installer: None,
        native: "groupBy",
    }
    ObjectFromEntries {
        function: FunctionOrdinal(8) => BUILTIN_OBJECT_FROM_ENTRIES_FUNCTION_ID,
        debug: "Object.fromEntries",
        flags: [],
        installer: None,
        native: "fromEntries",
    }
    ObjectAssign {
        function: FunctionOrdinal(9) => BUILTIN_OBJECT_ASSIGN_FUNCTION_ID,
        debug: "Object.assign",
        flags: [STATIC_METHOD],
        installer: None,
        native: "assign",
    }
    ObjectCreate {
        function: FunctionOrdinal(10) => BUILTIN_OBJECT_CREATE_FUNCTION_ID,
        debug: "Object.create",
        flags: [STATIC_METHOD],
        installer: None,
        native: "create",
    }
    ObjectGetPrototypeOf {
        function: FunctionOrdinal(11) => BUILTIN_OBJECT_GET_PROTOTYPE_OF_FUNCTION_ID,
        debug: "Object.getPrototypeOf",
        flags: [STATIC_METHOD],
        installer: None,
        native: "getPrototypeOf",
    }
    ObjectSetPrototypeOf {
        function: FunctionOrdinal(12) => BUILTIN_OBJECT_SET_PROTOTYPE_OF_FUNCTION_ID,
        debug: "Object.setPrototypeOf",
        flags: [STATIC_METHOD],
        installer: None,
        native: "setPrototypeOf",
    }
    ObjectDefineProperty {
        function: FunctionOrdinal(13) => BUILTIN_OBJECT_DEFINE_PROPERTY_FUNCTION_ID,
        debug: "Object.defineProperty",
        flags: [STATIC_METHOD],
        installer: None,
        native: "defineProperty",
    }
    ObjectDefineProperties {
        function: FunctionOrdinal(14) => BUILTIN_OBJECT_DEFINE_PROPERTIES_FUNCTION_ID,
        debug: "Object.defineProperties",
        flags: [STATIC_METHOD],
        installer: None,
        native: "defineProperties",
    }
    ObjectGetOwnPropertyDescriptor {
        function: FunctionOrdinal(15) => BUILTIN_OBJECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID,
        debug: "Object.getOwnPropertyDescriptor",
        flags: [STATIC_METHOD],
        installer: None,
        native: "getOwnPropertyDescriptor",
    }
    ObjectGetOwnPropertyDescriptors {
        function: FunctionOrdinal(16) => BUILTIN_OBJECT_GET_OWN_PROPERTY_DESCRIPTORS_FUNCTION_ID,
        debug: "Object.getOwnPropertyDescriptors",
        flags: [STATIC_METHOD],
        installer: None,
        native: "getOwnPropertyDescriptors",
    }
    ObjectGetOwnPropertyNames {
        function: FunctionOrdinal(17) => BUILTIN_OBJECT_GET_OWN_PROPERTY_NAMES_FUNCTION_ID,
        debug: "Object.getOwnPropertyNames",
        flags: [STATIC_METHOD],
        installer: None,
        native: "getOwnPropertyNames",
    }
    ObjectGetOwnPropertySymbols {
        function: FunctionOrdinal(18) => BUILTIN_OBJECT_GET_OWN_PROPERTY_SYMBOLS_FUNCTION_ID,
        debug: "Object.getOwnPropertySymbols",
        flags: [STATIC_METHOD],
        installer: None,
        native: "getOwnPropertySymbols",
    }
    ObjectKeys {
        function: FunctionOrdinal(19) => BUILTIN_OBJECT_KEYS_FUNCTION_ID,
        debug: "Object.keys",
        flags: [STATIC_METHOD],
        installer: None,
        native: "keys",
    }
    ObjectValues {
        function: FunctionOrdinal(20) => BUILTIN_OBJECT_VALUES_FUNCTION_ID,
        debug: "Object.values",
        flags: [STATIC_METHOD],
        installer: None,
        native: "values",
    }
    ObjectEntries {
        function: FunctionOrdinal(21) => BUILTIN_OBJECT_ENTRIES_FUNCTION_ID,
        debug: "Object.entries",
        flags: [STATIC_METHOD],
        installer: None,
        native: "entries",
    }
    ObjectHasOwn {
        function: FunctionOrdinal(22) => BUILTIN_OBJECT_HAS_OWN_FUNCTION_ID,
        debug: "Object.hasOwn",
        flags: [STATIC_METHOD],
        installer: None,
        native: "hasOwn",
    }
    ObjectIs {
        function: FunctionOrdinal(23) => BUILTIN_OBJECT_IS_FUNCTION_ID,
        debug: "Object.is",
        flags: [STATIC_METHOD],
        installer: None,
        native: "is",
    }
    ObjectIsSealed {
        function: FunctionOrdinal(24) => BUILTIN_OBJECT_IS_SEALED_FUNCTION_ID,
        debug: "Object.isSealed",
        flags: [],
        installer: None,
        native: "isSealed",
    }
    ObjectIsFrozen {
        function: FunctionOrdinal(25) => BUILTIN_OBJECT_IS_FROZEN_FUNCTION_ID,
        debug: "Object.isFrozen",
        flags: [],
        installer: None,
        native: "isFrozen",
    }
    ObjectSeal {
        function: FunctionOrdinal(26) => BUILTIN_OBJECT_SEAL_FUNCTION_ID,
        debug: "Object.seal",
        flags: [],
        installer: None,
        native: "seal",
    }
    ObjectFreeze {
        function: FunctionOrdinal(27) => BUILTIN_OBJECT_FREEZE_FUNCTION_ID,
        debug: "Object.freeze",
        flags: [],
        installer: None,
        native: "freeze",
    }
    ObjectIsExtensible {
        function: FunctionOrdinal(28) => BUILTIN_OBJECT_IS_EXTENSIBLE_FUNCTION_ID,
        debug: "Object.isExtensible",
        flags: [],
        installer: None,
        native: "isExtensible",
    }
    ObjectPreventExtensions {
        function: FunctionOrdinal(29) => BUILTIN_OBJECT_PREVENT_EXTENSIONS_FUNCTION_ID,
        debug: "Object.preventExtensions",
        flags: [],
        installer: None,
        native: "preventExtensions",
    }
    ObjectPrototypeHasOwnProperty {
        function: FunctionOrdinal(30) => BUILTIN_OBJECT_PROTOTYPE_HAS_OWN_PROPERTY_FUNCTION_ID,
        debug: "Object.prototype.hasOwnProperty",
        flags: [],
        installer: None,
        native: "hasOwnProperty",
    }
    ObjectPrototypeLookupGetter {
        function: FunctionOrdinal(31) => BUILTIN_OBJECT_PROTOTYPE_LOOKUP_GETTER_FUNCTION_ID,
        debug: "Object.prototype.__lookupGetter__",
        flags: [],
        installer: None,
        native: "__lookupGetter__",
    }
    ObjectPrototypeLookupSetter {
        function: FunctionOrdinal(32) => BUILTIN_OBJECT_PROTOTYPE_LOOKUP_SETTER_FUNCTION_ID,
        debug: "Object.prototype.__lookupSetter__",
        flags: [],
        installer: None,
        native: "__lookupSetter__",
    }
    ObjectPrototypeProtoGetter {
        function: FunctionOrdinal(33) => BUILTIN_OBJECT_PROTOTYPE_PROTO_GETTER_FUNCTION_ID,
        debug: "get Object.prototype.__proto__",
        flags: [],
        installer: None,
        native: "get __proto__",
    }
    ObjectPrototypeProtoSetter {
        function: FunctionOrdinal(34) => BUILTIN_OBJECT_PROTOTYPE_PROTO_SETTER_FUNCTION_ID,
        debug: "set Object.prototype.__proto__",
        flags: [],
        installer: None,
        native: "set __proto__",
    }
    ObjectPrototypePropertyIsEnumerable {
        function: FunctionOrdinal(35) => BUILTIN_OBJECT_PROTOTYPE_PROPERTY_IS_ENUMERABLE_FUNCTION_ID,
        debug: "Object.prototype.propertyIsEnumerable",
        flags: [],
        installer: None,
        native: "propertyIsEnumerable",
    }
    ObjectPrototypeIsPrototypeOf {
        function: FunctionOrdinal(36) => BUILTIN_OBJECT_PROTOTYPE_IS_PROTOTYPE_OF_FUNCTION_ID,
        debug: "Object.prototype.isPrototypeOf",
        flags: [],
        installer: None,
        native: "isPrototypeOf",
    }
    ObjectPrototypeToString {
        function: FunctionOrdinal(37) => BUILTIN_OBJECT_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Object.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    ObjectPrototypeToLocaleString {
        function: FunctionOrdinal(38) => BUILTIN_OBJECT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Object.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    ObjectPrototypeValueOf {
        function: FunctionOrdinal(39) => BUILTIN_OBJECT_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Object.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    ProxyConstructor {
        function: FunctionOrdinal(40) => BUILTIN_PROXY_FUNCTION_ID,
        global: GlobalOrdinal(3),
        global_name: PROXY_NAME,
        debug: PROXY_NAME,
        flags: [CONSTRUCTABLE],
        installer: Proxy,
        native: PROXY_NAME,
    }
    ProxyRevocable {
        function: FunctionOrdinal(41) => BUILTIN_PROXY_REVOCABLE_FUNCTION_ID,
        debug: "Proxy.revocable",
        flags: [STATIC_METHOD],
        installer: None,
        native: "revocable",
    }
    ProxyRevoke {
        function: FunctionOrdinal(42) => BUILTIN_PROXY_REVOKE_FUNCTION_ID,
        debug: "[[ProxyRevoke]]",
        flags: [],
        installer: None,
        native: "revoke",
    }
    ReflectConstruct {
        function: FunctionOrdinal(43) => BUILTIN_REFLECT_CONSTRUCT_FUNCTION_ID,
        debug: "Reflect.construct",
        flags: [STATIC_METHOD],
        installer: None,
        native: "construct",
    }
    ReflectApply {
        function: FunctionOrdinal(44) => BUILTIN_REFLECT_APPLY_FUNCTION_ID,
        debug: "Reflect.apply",
        flags: [STATIC_METHOD],
        installer: None,
        native: "apply",
    }
    ReflectGet {
        function: FunctionOrdinal(45) => BUILTIN_REFLECT_GET_FUNCTION_ID,
        debug: "Reflect.get",
        flags: [STATIC_METHOD],
        installer: None,
        native: "get",
    }
    ReflectGetPrototypeOf {
        function: FunctionOrdinal(46) => BUILTIN_REFLECT_GET_PROTOTYPE_OF_FUNCTION_ID,
        debug: "Reflect.getPrototypeOf",
        flags: [STATIC_METHOD],
        installer: None,
        native: "getPrototypeOf",
    }
    ReflectGetOwnPropertyDescriptor {
        function: FunctionOrdinal(47) => BUILTIN_REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID,
        debug: "Reflect.getOwnPropertyDescriptor",
        flags: [],
        installer: None,
        native: "getOwnPropertyDescriptor",
    }
    ReflectSet {
        function: FunctionOrdinal(48) => BUILTIN_REFLECT_SET_FUNCTION_ID,
        debug: "Reflect.set",
        flags: [STATIC_METHOD],
        installer: None,
        native: "set",
    }
    ReflectHas {
        function: FunctionOrdinal(49) => BUILTIN_REFLECT_HAS_FUNCTION_ID,
        debug: "Reflect.has",
        flags: [STATIC_METHOD],
        installer: None,
        native: "has",
    }
    ReflectDefineProperty {
        function: FunctionOrdinal(50) => BUILTIN_REFLECT_DEFINE_PROPERTY_FUNCTION_ID,
        debug: "Reflect.defineProperty",
        flags: [STATIC_METHOD],
        installer: None,
        native: "defineProperty",
    }
    ReflectDeleteProperty {
        function: FunctionOrdinal(51) => BUILTIN_REFLECT_DELETE_PROPERTY_FUNCTION_ID,
        debug: "Reflect.deleteProperty",
        flags: [STATIC_METHOD],
        installer: None,
        native: "deleteProperty",
    }
    ReflectIsExtensible {
        function: FunctionOrdinal(52) => BUILTIN_REFLECT_IS_EXTENSIBLE_FUNCTION_ID,
        debug: "Reflect.isExtensible",
        flags: [STATIC_METHOD],
        installer: None,
        native: "isExtensible",
    }
    ReflectPreventExtensions {
        function: FunctionOrdinal(53) => BUILTIN_REFLECT_PREVENT_EXTENSIONS_FUNCTION_ID,
        debug: "Reflect.preventExtensions",
        flags: [STATIC_METHOD],
        installer: None,
        native: "preventExtensions",
    }
    ReflectSetPrototypeOf {
        function: FunctionOrdinal(54) => BUILTIN_REFLECT_SET_PROTOTYPE_OF_FUNCTION_ID,
        debug: "Reflect.setPrototypeOf",
        flags: [STATIC_METHOD],
        installer: None,
        native: "setPrototypeOf",
    }
    ReflectOwnKeys {
        function: FunctionOrdinal(55) => BUILTIN_REFLECT_OWN_KEYS_FUNCTION_ID,
        debug: "Reflect.ownKeys",
        flags: [STATIC_METHOD],
        installer: None,
        native: "ownKeys",
    }
    ArrayConstructor {
        function: FunctionOrdinal(56) => BUILTIN_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(5),
        global_name: ARRAY_NAME,
        debug: ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: Array,
        native: ARRAY_NAME,
    }
    ArrayFrom {
        function: FunctionOrdinal(57) => BUILTIN_ARRAY_FROM_FUNCTION_ID,
        debug: "Array.from",
        flags: [STATIC_METHOD],
        installer: None,
        native: "from",
    }
    ArrayFromAsync {
        function: FunctionOrdinal(58) => BUILTIN_ARRAY_FROM_ASYNC_FUNCTION_ID,
        debug: "Array.fromAsync",
        flags: [STATIC_METHOD],
        installer: None,
        native: "fromAsync",
    }
    ArrayFromAsyncFulfilled {
        function: FunctionOrdinal(59) => BUILTIN_ARRAY_FROM_ASYNC_FULFILLED_FUNCTION_ID,
        debug: "Array.fromAsync Fulfilled Function",
        flags: [],
        installer: None,
        native: "",
    }
    ArrayFromAsyncRejected {
        function: FunctionOrdinal(60) => BUILTIN_ARRAY_FROM_ASYNC_REJECTED_FUNCTION_ID,
        debug: "Array.fromAsync Rejected Function",
        flags: [],
        installer: None,
        native: "",
    }
    ArrayOf {
        function: FunctionOrdinal(61) => BUILTIN_ARRAY_OF_FUNCTION_ID,
        debug: "Array.of",
        flags: [STATIC_METHOD],
        installer: None,
        native: "of",
    }
    ArrayIsArray {
        function: FunctionOrdinal(62) => BUILTIN_ARRAY_IS_ARRAY_FUNCTION_ID,
        debug: "Array.isArray",
        flags: [STATIC_METHOD],
        installer: None,
        native: "isArray",
    }
    ArraySpeciesGetter {
        function: FunctionOrdinal(63) => BUILTIN_ARRAY_SPECIES_GETTER_FUNCTION_ID,
        debug: "get Array [Symbol.species]",
        flags: [],
        installer: None,
        native: "get [Symbol.species]",
    }
    ArrayPrototypeConcat {
        function: FunctionOrdinal(64) => BUILTIN_ARRAY_PROTOTYPE_CONCAT_FUNCTION_ID,
        debug: "Array.prototype.concat",
        flags: [],
        installer: None,
        native: "concat",
    }
    ArrayPrototypeJoin {
        function: FunctionOrdinal(65) => BUILTIN_ARRAY_PROTOTYPE_JOIN_FUNCTION_ID,
        debug: "Array.prototype.join",
        flags: [],
        installer: None,
        native: "join",
    }
    ArrayPrototypeSlice {
        function: FunctionOrdinal(66) => BUILTIN_ARRAY_PROTOTYPE_SLICE_FUNCTION_ID,
        debug: "Array.prototype.slice",
        flags: [],
        installer: None,
        native: "slice",
    }
    ArrayPrototypeSplice {
        function: FunctionOrdinal(67) => BUILTIN_ARRAY_PROTOTYPE_SPLICE_FUNCTION_ID,
        debug: "Array.prototype.splice",
        flags: [],
        installer: None,
        native: "splice",
    }
    ArrayPrototypeSort {
        function: FunctionOrdinal(68) => BUILTIN_ARRAY_PROTOTYPE_SORT_FUNCTION_ID,
        debug: "Array.prototype.sort",
        flags: [],
        installer: None,
        native: "sort",
    }
    ArrayPrototypeToLocaleString {
        function: FunctionOrdinal(69) => BUILTIN_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Array.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    ArrayPrototypeFlat {
        function: FunctionOrdinal(70) => BUILTIN_ARRAY_PROTOTYPE_FLAT_FUNCTION_ID,
        debug: "Array.prototype.flat",
        flags: [],
        installer: None,
        native: "flat",
    }
    ArrayPrototypeFlatMap {
        function: FunctionOrdinal(71) => BUILTIN_ARRAY_PROTOTYPE_FLAT_MAP_FUNCTION_ID,
        debug: "Array.prototype.flatMap",
        flags: [],
        installer: None,
        native: "flatMap",
    }
    ArrayPrototypeAt {
        function: FunctionOrdinal(72) => BUILTIN_ARRAY_PROTOTYPE_AT_FUNCTION_ID,
        debug: "Array.prototype.at",
        flags: [],
        installer: None,
        native: "at",
    }
    ArrayPrototypeToReversed {
        function: FunctionOrdinal(73) => BUILTIN_ARRAY_PROTOTYPE_TO_REVERSED_FUNCTION_ID,
        debug: "Array.prototype.toReversed",
        flags: [],
        installer: None,
        native: "toReversed",
    }
    ArrayPrototypeToSpliced {
        function: FunctionOrdinal(74) => BUILTIN_ARRAY_PROTOTYPE_TO_SPLICED_FUNCTION_ID,
        debug: "Array.prototype.toSpliced",
        flags: [],
        installer: None,
        native: "toSpliced",
    }
    ArrayPrototypeToSorted {
        function: FunctionOrdinal(75) => BUILTIN_ARRAY_PROTOTYPE_TO_SORTED_FUNCTION_ID,
        debug: "Array.prototype.toSorted",
        flags: [],
        installer: None,
        native: "toSorted",
    }
    ArrayPrototypeWith {
        function: FunctionOrdinal(76) => BUILTIN_ARRAY_PROTOTYPE_WITH_FUNCTION_ID,
        debug: "Array.prototype.with",
        flags: [],
        installer: None,
        native: "with",
    }
    ArrayPrototypeReverse {
        function: FunctionOrdinal(77) => BUILTIN_ARRAY_PROTOTYPE_REVERSE_FUNCTION_ID,
        debug: "Array.prototype.reverse",
        flags: [],
        installer: None,
        native: "reverse",
    }
    ArrayPrototypeCopyWithin {
        function: FunctionOrdinal(78) => BUILTIN_ARRAY_PROTOTYPE_COPY_WITHIN_FUNCTION_ID,
        debug: "Array.prototype.copyWithin",
        flags: [],
        installer: None,
        native: "copyWithin",
    }
    ArrayPrototypeIncludes {
        function: FunctionOrdinal(79) => BUILTIN_ARRAY_PROTOTYPE_INCLUDES_FUNCTION_ID,
        debug: "Array.prototype.includes",
        flags: [],
        installer: None,
        native: "includes",
    }
    ArrayPrototypeIndexOf {
        function: FunctionOrdinal(80) => BUILTIN_ARRAY_PROTOTYPE_INDEX_OF_FUNCTION_ID,
        debug: "Array.prototype.indexOf",
        flags: [],
        installer: None,
        native: "indexOf",
    }
    ArrayPrototypeLastIndexOf {
        function: FunctionOrdinal(81) => BUILTIN_ARRAY_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID,
        debug: "Array.prototype.lastIndexOf",
        flags: [],
        installer: None,
        native: "lastIndexOf",
    }
    ArrayPrototypeFind {
        function: FunctionOrdinal(82) => BUILTIN_ARRAY_PROTOTYPE_FIND_FUNCTION_ID,
        debug: "Array.prototype.find",
        flags: [],
        installer: None,
        native: "find",
    }
    ArrayPrototypeFindIndex {
        function: FunctionOrdinal(83) => BUILTIN_ARRAY_PROTOTYPE_FIND_INDEX_FUNCTION_ID,
        debug: "Array.prototype.findIndex",
        flags: [],
        installer: None,
        native: "findIndex",
    }
    ArrayPrototypeFindLast {
        function: FunctionOrdinal(84) => BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_FUNCTION_ID,
        debug: "Array.prototype.findLast",
        flags: [],
        installer: None,
        native: "findLast",
    }
    ArrayPrototypeFindLastIndex {
        function: FunctionOrdinal(85) => BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_INDEX_FUNCTION_ID,
        debug: "Array.prototype.findLastIndex",
        flags: [],
        installer: None,
        native: "findLastIndex",
    }
    ArrayPrototypeEvery {
        function: FunctionOrdinal(86) => BUILTIN_ARRAY_PROTOTYPE_EVERY_FUNCTION_ID,
        debug: "Array.prototype.every",
        flags: [],
        installer: None,
        native: "every",
    }
    ArrayPrototypeSome {
        function: FunctionOrdinal(87) => BUILTIN_ARRAY_PROTOTYPE_SOME_FUNCTION_ID,
        debug: "Array.prototype.some",
        flags: [],
        installer: None,
        native: "some",
    }
    ArrayPrototypeForEach {
        function: FunctionOrdinal(88) => BUILTIN_ARRAY_PROTOTYPE_FOR_EACH_FUNCTION_ID,
        debug: "Array.prototype.forEach",
        flags: [],
        installer: None,
        native: "forEach",
    }
    ArrayPrototypeFilter {
        function: FunctionOrdinal(89) => BUILTIN_ARRAY_PROTOTYPE_FILTER_FUNCTION_ID,
        debug: "Array.prototype.filter",
        flags: [],
        installer: None,
        native: "filter",
    }
    ArrayPrototypeMap {
        function: FunctionOrdinal(90) => BUILTIN_ARRAY_PROTOTYPE_MAP_FUNCTION_ID,
        debug: "Array.prototype.map",
        flags: [],
        installer: None,
        native: "map",
    }
    ArrayPrototypeReduce {
        function: FunctionOrdinal(91) => BUILTIN_ARRAY_PROTOTYPE_REDUCE_FUNCTION_ID,
        debug: "Array.prototype.reduce",
        flags: [],
        installer: None,
        native: "reduce",
    }
    ArrayPrototypeReduceRight {
        function: FunctionOrdinal(92) => BUILTIN_ARRAY_PROTOTYPE_REDUCE_RIGHT_FUNCTION_ID,
        debug: "Array.prototype.reduceRight",
        flags: [],
        installer: None,
        native: "reduceRight",
    }
    ArrayPrototypePop {
        function: FunctionOrdinal(93) => BUILTIN_ARRAY_PROTOTYPE_POP_FUNCTION_ID,
        debug: "Array.prototype.pop",
        flags: [],
        installer: None,
        native: "pop",
    }
    ArrayPrototypePush {
        function: FunctionOrdinal(94) => BUILTIN_ARRAY_PROTOTYPE_PUSH_FUNCTION_ID,
        debug: "Array.prototype.push",
        flags: [],
        installer: None,
        native: "push",
    }
    ArrayPrototypeShift {
        function: FunctionOrdinal(95) => BUILTIN_ARRAY_PROTOTYPE_SHIFT_FUNCTION_ID,
        debug: "Array.prototype.shift",
        flags: [],
        installer: None,
        native: "shift",
    }
    ArrayPrototypeUnshift {
        function: FunctionOrdinal(96) => BUILTIN_ARRAY_PROTOTYPE_UNSHIFT_FUNCTION_ID,
        debug: "Array.prototype.unshift",
        flags: [],
        installer: None,
        native: "unshift",
    }
    ArrayPrototypeFill {
        function: FunctionOrdinal(97) => BUILTIN_ARRAY_PROTOTYPE_FILL_FUNCTION_ID,
        debug: "Array.prototype.fill",
        flags: [],
        installer: None,
        native: "fill",
    }
    ArrayPrototypeKeys {
        function: FunctionOrdinal(98) => BUILTIN_ARRAY_PROTOTYPE_KEYS_FUNCTION_ID,
        debug: "Array.prototype.keys",
        flags: [],
        installer: None,
        native: "keys",
    }
    ArrayPrototypeEntries {
        function: FunctionOrdinal(99) => BUILTIN_ARRAY_PROTOTYPE_ENTRIES_FUNCTION_ID,
        debug: "Array.prototype.entries",
        flags: [],
        installer: None,
        native: "entries",
    }
    ArrayPrototypeValues {
        function: FunctionOrdinal(100) => BUILTIN_ARRAY_PROTOTYPE_VALUES_FUNCTION_ID,
        debug: "Array.prototype.values",
        flags: [],
        installer: None,
        native: "values",
    }
    ArrayIteratorNext {
        function: FunctionOrdinal(101) => BUILTIN_ARRAY_ITERATOR_NEXT_FUNCTION_ID,
        debug: "Array Iterator.prototype.next",
        flags: [],
        installer: None,
        native: "next",
    }
    ArrayIteratorIdentity {
        function: FunctionOrdinal(102) => BUILTIN_ARRAY_ITERATOR_IDENTITY_FUNCTION_ID,
        debug: "Array Iterator.prototype [Symbol.iterator]",
        flags: [],
        installer: None,
        native: "[Symbol.iterator]",
    }
    StringIteratorNext {
        function: FunctionOrdinal(103) => BUILTIN_STRING_ITERATOR_NEXT_FUNCTION_ID,
        debug: "String Iterator.prototype.next",
        flags: [],
        installer: None,
        native: "next",
    }
    GeneratorPrototypeNext {
        function: FunctionOrdinal(104) => BUILTIN_GENERATOR_PROTOTYPE_NEXT_FUNCTION_ID,
        debug: "Generator.prototype.next",
        flags: [],
        installer: None,
        native: "next",
    }
    GeneratorPrototypeReturn {
        function: FunctionOrdinal(105) => BUILTIN_GENERATOR_PROTOTYPE_RETURN_FUNCTION_ID,
        debug: "Generator.prototype.return",
        flags: [],
        installer: None,
        native: "return",
    }
    GeneratorPrototypeThrow {
        function: FunctionOrdinal(106) => BUILTIN_GENERATOR_PROTOTYPE_THROW_FUNCTION_ID,
        debug: "Generator.prototype.throw",
        flags: [],
        installer: None,
        native: "throw",
    }
    AsyncGeneratorPrototypeNext {
        function: FunctionOrdinal(107) => BUILTIN_ASYNC_GENERATOR_PROTOTYPE_NEXT_FUNCTION_ID,
        debug: "AsyncGenerator.prototype.next",
        flags: [],
        installer: None,
        native: "next",
    }
    AsyncGeneratorPrototypeReturn {
        function: FunctionOrdinal(108) => BUILTIN_ASYNC_GENERATOR_PROTOTYPE_RETURN_FUNCTION_ID,
        debug: "AsyncGenerator.prototype.return",
        flags: [],
        installer: None,
        native: "return",
    }
    AsyncGeneratorPrototypeThrow {
        function: FunctionOrdinal(109) => BUILTIN_ASYNC_GENERATOR_PROTOTYPE_THROW_FUNCTION_ID,
        debug: "AsyncGenerator.prototype.throw",
        flags: [],
        installer: None,
        native: "throw",
    }
    AsyncIteratorPrototypeAsyncDispose {
        function: FunctionOrdinal(110) => BUILTIN_ASYNC_ITERATOR_PROTOTYPE_ASYNC_DISPOSE_FUNCTION_ID,
        debug: "AsyncIterator.prototype[Symbol.asyncDispose]",
        flags: [],
        installer: None,
        native: "[Symbol.asyncDispose]",
    }
    AsyncIteratorPrototypeAsyncDisposeFulfilled {
        function: FunctionOrdinal(111) => BUILTIN_ASYNC_ITERATOR_PROTOTYPE_ASYNC_DISPOSE_FULFILLED_FUNCTION_ID,
        debug: "AsyncIterator asyncDispose Fulfilled Function",
        flags: [],
        installer: None,
        native: "",
    }
    AsyncIteratorPrototypeAsyncDisposeRejected {
        function: FunctionOrdinal(112) => BUILTIN_ASYNC_ITERATOR_PROTOTYPE_ASYNC_DISPOSE_REJECTED_FUNCTION_ID,
        debug: "AsyncIterator asyncDispose Rejected Function",
        flags: [],
        installer: None,
        native: "",
    }
    IteratorConstructor {
        function: FunctionOrdinal(113) => BUILTIN_ITERATOR_FUNCTION_ID,
        global: GlobalOrdinal(4),
        global_name: "Iterator",
        debug: "Iterator",
        flags: [CONSTRUCTABLE],
        installer: Iterator,
        native: "Iterator",
    }
    IteratorFrom {
        function: FunctionOrdinal(114) => BUILTIN_ITERATOR_FROM_FUNCTION_ID,
        debug: "Iterator.from",
        flags: [STATIC_METHOD],
        installer: None,
        native: "from",
    }
    IteratorConcat {
        function: FunctionOrdinal(115) => BUILTIN_ITERATOR_CONCAT_FUNCTION_ID,
        debug: "Iterator.concat",
        flags: [STATIC_METHOD],
        installer: None,
        native: "concat",
    }
    IteratorConcatNext {
        function: FunctionOrdinal(116) => BUILTIN_ITERATOR_CONCAT_NEXT_FUNCTION_ID,
        debug: "Iterator concat helper next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorConcatReturn {
        function: FunctionOrdinal(117) => BUILTIN_ITERATOR_CONCAT_RETURN_FUNCTION_ID,
        debug: "Iterator concat helper return",
        flags: [],
        installer: None,
        native: "return",
    }
    IteratorZip {
        function: FunctionOrdinal(118) => BUILTIN_ITERATOR_ZIP_FUNCTION_ID,
        debug: "Iterator.zip",
        flags: [STATIC_METHOD],
        installer: None,
        native: "zip",
    }
    IteratorZipKeyed {
        function: FunctionOrdinal(119) => BUILTIN_ITERATOR_ZIP_KEYED_FUNCTION_ID,
        debug: "Iterator.zipKeyed",
        flags: [STATIC_METHOD],
        installer: None,
        native: "zipKeyed",
    }
    IteratorZipNext {
        function: FunctionOrdinal(120) => BUILTIN_ITERATOR_ZIP_NEXT_FUNCTION_ID,
        debug: "Iterator zip helper next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorZipReturn {
        function: FunctionOrdinal(121) => BUILTIN_ITERATOR_ZIP_RETURN_FUNCTION_ID,
        debug: "Iterator zip helper return",
        flags: [],
        installer: None,
        native: "return",
    }
    IteratorHelperNext {
        function: FunctionOrdinal(122) => BUILTIN_ITERATOR_HELPER_NEXT_FUNCTION_ID,
        debug: "%IteratorHelperPrototype%.next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorHelperReturn {
        function: FunctionOrdinal(123) => BUILTIN_ITERATOR_HELPER_RETURN_FUNCTION_ID,
        debug: "%IteratorHelperPrototype%.return",
        flags: [],
        installer: None,
        native: "return",
    }
    IteratorPrototypeToArray {
        function: FunctionOrdinal(124) => BUILTIN_ITERATOR_PROTOTYPE_TO_ARRAY_FUNCTION_ID,
        debug: "Iterator.prototype.toArray",
        flags: [],
        installer: None,
        native: "toArray",
    }
    IteratorPrototypeForEach {
        function: FunctionOrdinal(125) => BUILTIN_ITERATOR_PROTOTYPE_FOR_EACH_FUNCTION_ID,
        debug: "Iterator.prototype.forEach",
        flags: [],
        installer: None,
        native: "forEach",
    }
    IteratorPrototypeEvery {
        function: FunctionOrdinal(126) => BUILTIN_ITERATOR_PROTOTYPE_EVERY_FUNCTION_ID,
        debug: "Iterator.prototype.every",
        flags: [],
        installer: None,
        native: "every",
    }
    IteratorPrototypeSome {
        function: FunctionOrdinal(127) => BUILTIN_ITERATOR_PROTOTYPE_SOME_FUNCTION_ID,
        debug: "Iterator.prototype.some",
        flags: [],
        installer: None,
        native: "some",
    }
    IteratorPrototypeFind {
        function: FunctionOrdinal(128) => BUILTIN_ITERATOR_PROTOTYPE_FIND_FUNCTION_ID,
        debug: "Iterator.prototype.find",
        flags: [],
        installer: None,
        native: "find",
    }
    IteratorPrototypeReduce {
        function: FunctionOrdinal(129) => BUILTIN_ITERATOR_PROTOTYPE_REDUCE_FUNCTION_ID,
        debug: "Iterator.prototype.reduce",
        flags: [],
        installer: None,
        native: "reduce",
    }
    IteratorPrototypeMap {
        function: FunctionOrdinal(130) => BUILTIN_ITERATOR_PROTOTYPE_MAP_FUNCTION_ID,
        debug: "Iterator.prototype.map",
        flags: [],
        installer: None,
        native: "map",
    }
    IteratorMapNext {
        function: FunctionOrdinal(131) => BUILTIN_ITERATOR_MAP_NEXT_FUNCTION_ID,
        debug: "Iterator map helper next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorMapReturn {
        function: FunctionOrdinal(132) => BUILTIN_ITERATOR_MAP_RETURN_FUNCTION_ID,
        debug: "Iterator map helper return",
        flags: [],
        installer: None,
        native: "return",
    }
    IteratorPrototypeFilter {
        function: FunctionOrdinal(133) => BUILTIN_ITERATOR_PROTOTYPE_FILTER_FUNCTION_ID,
        debug: "Iterator.prototype.filter",
        flags: [],
        installer: None,
        native: "filter",
    }
    IteratorFilterNext {
        function: FunctionOrdinal(134) => BUILTIN_ITERATOR_FILTER_NEXT_FUNCTION_ID,
        debug: "Iterator filter helper next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorFilterReturn {
        function: FunctionOrdinal(135) => BUILTIN_ITERATOR_FILTER_RETURN_FUNCTION_ID,
        debug: "Iterator filter helper return",
        flags: [],
        installer: None,
        native: "return",
    }
    IteratorPrototypeFlatMap {
        function: FunctionOrdinal(136) => BUILTIN_ITERATOR_PROTOTYPE_FLAT_MAP_FUNCTION_ID,
        debug: "Iterator.prototype.flatMap",
        flags: [],
        installer: None,
        native: "flatMap",
    }
    IteratorFlatMapNext {
        function: FunctionOrdinal(137) => BUILTIN_ITERATOR_FLAT_MAP_NEXT_FUNCTION_ID,
        debug: "Iterator flatMap helper next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorFlatMapReturn {
        function: FunctionOrdinal(138) => BUILTIN_ITERATOR_FLAT_MAP_RETURN_FUNCTION_ID,
        debug: "Iterator flatMap helper return",
        flags: [],
        installer: None,
        native: "return",
    }
    IteratorPrototypeTake {
        function: FunctionOrdinal(139) => BUILTIN_ITERATOR_PROTOTYPE_TAKE_FUNCTION_ID,
        debug: "Iterator.prototype.take",
        flags: [],
        installer: None,
        native: "take",
    }
    IteratorTakeNext {
        function: FunctionOrdinal(140) => BUILTIN_ITERATOR_TAKE_NEXT_FUNCTION_ID,
        debug: "Iterator take helper next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorTakeReturn {
        function: FunctionOrdinal(141) => BUILTIN_ITERATOR_TAKE_RETURN_FUNCTION_ID,
        debug: "Iterator take helper return",
        flags: [],
        installer: None,
        native: "return",
    }
    IteratorPrototypeDrop {
        function: FunctionOrdinal(142) => BUILTIN_ITERATOR_PROTOTYPE_DROP_FUNCTION_ID,
        debug: "Iterator.prototype.drop",
        flags: [],
        installer: None,
        native: "drop",
    }
    IteratorDropNext {
        function: FunctionOrdinal(143) => BUILTIN_ITERATOR_DROP_NEXT_FUNCTION_ID,
        debug: "Iterator drop helper next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorDropReturn {
        function: FunctionOrdinal(144) => BUILTIN_ITERATOR_DROP_RETURN_FUNCTION_ID,
        debug: "Iterator drop helper return",
        flags: [],
        installer: None,
        native: "return",
    }
    IteratorPrototypeConstructorGetter {
        function: FunctionOrdinal(145) => BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_GETTER_FUNCTION_ID,
        debug: "get Iterator.prototype.constructor",
        flags: [],
        installer: None,
        native: "get constructor",
    }
    IteratorPrototypeConstructorSetter {
        function: FunctionOrdinal(146) => BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_SETTER_FUNCTION_ID,
        debug: "set Iterator.prototype.constructor",
        flags: [],
        installer: None,
        native: "set constructor",
    }
    IteratorPrototypeSymbolDispose {
        function: FunctionOrdinal(147) => BUILTIN_ITERATOR_PROTOTYPE_SYMBOL_DISPOSE_FUNCTION_ID,
        debug: "Iterator.prototype[Symbol.dispose]",
        flags: [],
        installer: None,
        native: "[Symbol.dispose]",
    }
    IteratorPrototypeToStringTagGetter {
        function: FunctionOrdinal(148) => BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_GETTER_FUNCTION_ID,
        debug: "get Iterator.prototype[Symbol.toStringTag]",
        flags: [],
        installer: None,
        native: "get [Symbol.toStringTag]",
    }
    IteratorPrototypeToStringTagSetter {
        function: FunctionOrdinal(149) => BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_SETTER_FUNCTION_ID,
        debug: "set Iterator.prototype[Symbol.toStringTag]",
        flags: [],
        installer: None,
        native: "set [Symbol.toStringTag]",
    }
    IteratorFromWrapperNext {
        function: FunctionOrdinal(150) => BUILTIN_ITERATOR_FROM_WRAPPER_NEXT_FUNCTION_ID,
        debug: "%WrapForValidIteratorPrototype%.next",
        flags: [],
        installer: None,
        native: "next",
    }
    IteratorFromWrapperReturn {
        function: FunctionOrdinal(151) => BUILTIN_ITERATOR_FROM_WRAPPER_RETURN_FUNCTION_ID,
        debug: "%WrapForValidIteratorPrototype%.return",
        flags: [],
        installer: None,
        native: "return",
    }
    ArrayBufferConstructor {
        function: FunctionOrdinal(152) => BUILTIN_ARRAY_BUFFER_FUNCTION_ID,
        global: GlobalOrdinal(6),
        global_name: ARRAY_BUFFER_NAME,
        debug: ARRAY_BUFFER_NAME,
        flags: [CONSTRUCTABLE],
        installer: ArrayBuffer,
        native: ARRAY_BUFFER_NAME,
    }
    SharedArrayBufferConstructor {
        function: FunctionOrdinal(153) => BUILTIN_SHARED_ARRAY_BUFFER_FUNCTION_ID,
        global: GlobalOrdinal(7),
        global_name: SHARED_ARRAY_BUFFER_NAME,
        debug: SHARED_ARRAY_BUFFER_NAME,
        flags: [CONSTRUCTABLE],
        installer: ArrayBuffer,
        native: SHARED_ARRAY_BUFFER_NAME,
    }
    ArrayBufferIsView {
        function: FunctionOrdinal(154) => BUILTIN_ARRAY_BUFFER_IS_VIEW_FUNCTION_ID,
        debug: "ArrayBuffer.isView",
        flags: [STATIC_METHOD],
        installer: None,
        native: "isView",
    }
    ArrayBufferSpeciesGetter {
        function: FunctionOrdinal(155) => BUILTIN_ARRAY_BUFFER_SPECIES_GETTER_FUNCTION_ID,
        debug: "get ArrayBuffer [Symbol.species]",
        flags: [],
        installer: None,
        native: "get [Symbol.species]",
    }
    ArrayBufferPrototypeByteLengthGetter {
        function: FunctionOrdinal(156) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID,
        debug: "get ArrayBuffer.prototype.byteLength",
        flags: [],
        installer: None,
        native: "get byteLength",
    }
    SharedArrayBufferPrototypeByteLengthGetter {
        function: FunctionOrdinal(157) => BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID,
        debug: "get SharedArrayBuffer.prototype.byteLength",
        flags: [],
        installer: None,
        native: "get byteLength",
    }
    SharedArrayBufferPrototypeMaxByteLengthGetter {
        function: FunctionOrdinal(158) => BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID,
        debug: "get SharedArrayBuffer.prototype.maxByteLength",
        flags: [],
        installer: None,
        native: "get maxByteLength",
    }
    SharedArrayBufferPrototypeGrowableGetter {
        function: FunctionOrdinal(159) => BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROWABLE_GETTER_FUNCTION_ID,
        debug: "get SharedArrayBuffer.prototype.growable",
        flags: [],
        installer: None,
        native: "get growable",
    }
    SharedArrayBufferPrototypeGrow {
        function: FunctionOrdinal(160) => BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROW_FUNCTION_ID,
        debug: "SharedArrayBuffer.prototype.grow",
        flags: [],
        installer: None,
        native: "grow",
    }
    ArrayBufferPrototypeDetachedGetter {
        function: FunctionOrdinal(161) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_DETACHED_GETTER_FUNCTION_ID,
        debug: "get ArrayBuffer.prototype.detached",
        flags: [],
        installer: None,
        native: "get detached",
    }
    ArrayBufferPrototypeMaxByteLengthGetter {
        function: FunctionOrdinal(162) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID,
        debug: "get ArrayBuffer.prototype.maxByteLength",
        flags: [],
        installer: None,
        native: "get maxByteLength",
    }
    ArrayBufferPrototypeResizableGetter {
        function: FunctionOrdinal(163) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZABLE_GETTER_FUNCTION_ID,
        debug: "get ArrayBuffer.prototype.resizable",
        flags: [],
        installer: None,
        native: "get resizable",
    }
    ArrayBufferPrototypeResize {
        function: FunctionOrdinal(164) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZE_FUNCTION_ID,
        debug: "ArrayBuffer.prototype.resize",
        flags: [],
        installer: None,
        native: "resize",
    }
    ArrayBufferPrototypeSlice {
        function: FunctionOrdinal(165) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID,
        debug: "ArrayBuffer.prototype.slice",
        flags: [],
        installer: None,
        native: "slice",
    }
    SharedArrayBufferPrototypeSlice {
        function: FunctionOrdinal(166) => BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID,
        debug: "SharedArrayBuffer.prototype.slice",
        flags: [],
        installer: None,
        native: "slice",
    }
    ArrayBufferPrototypeTransfer {
        function: FunctionOrdinal(167) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_FUNCTION_ID,
        debug: "ArrayBuffer.prototype.transfer",
        flags: [],
        installer: None,
        native: "transfer",
    }
    ArrayBufferPrototypeTransferToFixedLength {
        function: FunctionOrdinal(168) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_FIXED_LENGTH_FUNCTION_ID,
        debug: "ArrayBuffer.prototype.transferToFixedLength",
        flags: [],
        installer: None,
        native: "transferToFixedLength",
    }
    ArrayBufferPrototypeTransferToImmutable {
        function: FunctionOrdinal(169) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_IMMUTABLE_FUNCTION_ID,
        debug: "ArrayBuffer.prototype.transferToImmutable",
        flags: [],
        installer: None,
        native: "transferToImmutable",
    }
    ArrayBufferPrototypeSliceToImmutable {
        function: FunctionOrdinal(170) => BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_TO_IMMUTABLE_FUNCTION_ID,
        debug: "ArrayBuffer.prototype.sliceToImmutable",
        flags: [],
        installer: None,
        native: "sliceToImmutable",
    }
    DataViewConstructor {
        function: FunctionOrdinal(171) => BUILTIN_DATA_VIEW_FUNCTION_ID,
        global: GlobalOrdinal(8),
        global_name: DATA_VIEW_NAME,
        debug: DATA_VIEW_NAME,
        flags: [CONSTRUCTABLE],
        installer: DataView,
        native: DATA_VIEW_NAME,
    }
    DataViewPrototypeBufferGetter {
        function: FunctionOrdinal(172) => BUILTIN_DATA_VIEW_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID,
        debug: "get DataView.prototype.buffer",
        flags: [],
        installer: None,
        native: "get buffer",
    }
    DataViewPrototypeByteLengthGetter {
        function: FunctionOrdinal(173) => BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID,
        debug: "get DataView.prototype.byteLength",
        flags: [],
        installer: None,
        native: "get byteLength",
    }
    DataViewPrototypeByteOffsetGetter {
        function: FunctionOrdinal(174) => BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID,
        debug: "get DataView.prototype.byteOffset",
        flags: [],
        installer: None,
        native: "get byteOffset",
    }
    TypedArraySpeciesGetter {
        function: FunctionOrdinal(175) => BUILTIN_TYPED_ARRAY_SPECIES_GETTER_FUNCTION_ID,
        debug: "get TypedArray [Symbol.species]",
        flags: [],
        installer: None,
        native: "get [Symbol.species]",
    }
    TypedArrayPrototypeBufferGetter {
        function: FunctionOrdinal(176) => BUILTIN_TYPED_ARRAY_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID,
        debug: "get TypedArray.prototype.buffer",
        flags: [],
        installer: None,
        native: "get buffer",
    }
    TypedArrayPrototypeByteLengthGetter {
        function: FunctionOrdinal(177) => BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID,
        debug: "get TypedArray.prototype.byteLength",
        flags: [],
        installer: None,
        native: "get byteLength",
    }
    TypedArrayPrototypeByteOffsetGetter {
        function: FunctionOrdinal(178) => BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID,
        debug: "get TypedArray.prototype.byteOffset",
        flags: [],
        installer: None,
        native: "get byteOffset",
    }
    TypedArrayPrototypeLengthGetter {
        function: FunctionOrdinal(179) => BUILTIN_TYPED_ARRAY_PROTOTYPE_LENGTH_GETTER_FUNCTION_ID,
        debug: "get TypedArray.prototype.length",
        flags: [],
        installer: None,
        native: "get length",
    }
    TypedArrayPrototypeToStringTagGetter {
        function: FunctionOrdinal(180) => BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_STRING_TAG_GETTER_FUNCTION_ID,
        debug: "get TypedArray.prototype[Symbol.toStringTag]",
        flags: [],
        installer: None,
        native: "get [Symbol.toStringTag]",
    }
    TypedArrayPrototypeToString {
        function: FunctionOrdinal(181) => BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "TypedArray.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    TypedArrayPrototypeAt {
        function: FunctionOrdinal(182) => BUILTIN_TYPED_ARRAY_PROTOTYPE_AT_FUNCTION_ID,
        debug: "TypedArray.prototype.at",
        flags: [],
        installer: None,
        native: "at",
    }
    TypedArrayPrototypeCopyWithin {
        function: FunctionOrdinal(183) => BUILTIN_TYPED_ARRAY_PROTOTYPE_COPY_WITHIN_FUNCTION_ID,
        debug: "TypedArray.prototype.copyWithin",
        flags: [],
        installer: None,
        native: "copyWithin",
    }
    TypedArrayPrototypeIncludes {
        function: FunctionOrdinal(184) => BUILTIN_TYPED_ARRAY_PROTOTYPE_INCLUDES_FUNCTION_ID,
        debug: "TypedArray.prototype.includes",
        flags: [],
        installer: None,
        native: "includes",
    }
    TypedArrayPrototypeIndexOf {
        function: FunctionOrdinal(185) => BUILTIN_TYPED_ARRAY_PROTOTYPE_INDEX_OF_FUNCTION_ID,
        debug: "TypedArray.prototype.indexOf",
        flags: [],
        installer: None,
        native: "indexOf",
    }
    TypedArrayPrototypeLastIndexOf {
        function: FunctionOrdinal(186) => BUILTIN_TYPED_ARRAY_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID,
        debug: "TypedArray.prototype.lastIndexOf",
        flags: [],
        installer: None,
        native: "lastIndexOf",
    }
    TypedArrayPrototypeFind {
        function: FunctionOrdinal(187) => BUILTIN_TYPED_ARRAY_PROTOTYPE_FIND_FUNCTION_ID,
        debug: "TypedArray.prototype.find",
        flags: [],
        installer: None,
        native: "find",
    }
    TypedArrayPrototypeFindIndex {
        function: FunctionOrdinal(188) => BUILTIN_TYPED_ARRAY_PROTOTYPE_FIND_INDEX_FUNCTION_ID,
        debug: "TypedArray.prototype.findIndex",
        flags: [],
        installer: None,
        native: "findIndex",
    }
    TypedArrayPrototypeFindLast {
        function: FunctionOrdinal(189) => BUILTIN_TYPED_ARRAY_PROTOTYPE_FIND_LAST_FUNCTION_ID,
        debug: "TypedArray.prototype.findLast",
        flags: [],
        installer: None,
        native: "findLast",
    }
    TypedArrayPrototypeFindLastIndex {
        function: FunctionOrdinal(190) => BUILTIN_TYPED_ARRAY_PROTOTYPE_FIND_LAST_INDEX_FUNCTION_ID,
        debug: "TypedArray.prototype.findLastIndex",
        flags: [],
        installer: None,
        native: "findLastIndex",
    }
    TypedArrayPrototypeEvery {
        function: FunctionOrdinal(191) => BUILTIN_TYPED_ARRAY_PROTOTYPE_EVERY_FUNCTION_ID,
        debug: "TypedArray.prototype.every",
        flags: [],
        installer: None,
        native: "every",
    }
    TypedArrayPrototypeSome {
        function: FunctionOrdinal(192) => BUILTIN_TYPED_ARRAY_PROTOTYPE_SOME_FUNCTION_ID,
        debug: "TypedArray.prototype.some",
        flags: [],
        installer: None,
        native: "some",
    }
    TypedArrayPrototypeMap {
        function: FunctionOrdinal(193) => BUILTIN_TYPED_ARRAY_PROTOTYPE_MAP_FUNCTION_ID,
        debug: "TypedArray.prototype.map",
        flags: [],
        installer: None,
        native: "map",
    }
    TypedArrayPrototypeFilter {
        function: FunctionOrdinal(194) => BUILTIN_TYPED_ARRAY_PROTOTYPE_FILTER_FUNCTION_ID,
        debug: "TypedArray.prototype.filter",
        flags: [],
        installer: None,
        native: "filter",
    }
    TypedArrayPrototypeForEach {
        function: FunctionOrdinal(195) => BUILTIN_TYPED_ARRAY_PROTOTYPE_FOR_EACH_FUNCTION_ID,
        debug: "TypedArray.prototype.forEach",
        flags: [],
        installer: None,
        native: "forEach",
    }
    TypedArrayPrototypeReduce {
        function: FunctionOrdinal(196) => BUILTIN_TYPED_ARRAY_PROTOTYPE_REDUCE_FUNCTION_ID,
        debug: "TypedArray.prototype.reduce",
        flags: [],
        installer: None,
        native: "reduce",
    }
    TypedArrayPrototypeReduceRight {
        function: FunctionOrdinal(197) => BUILTIN_TYPED_ARRAY_PROTOTYPE_REDUCE_RIGHT_FUNCTION_ID,
        debug: "TypedArray.prototype.reduceRight",
        flags: [],
        installer: None,
        native: "reduceRight",
    }
    TypedArrayPrototypeValues {
        function: FunctionOrdinal(198) => BUILTIN_TYPED_ARRAY_PROTOTYPE_VALUES_FUNCTION_ID,
        debug: "TypedArray.prototype.values",
        flags: [],
        installer: None,
        native: "values",
    }
    TypedArrayPrototypeKeys {
        function: FunctionOrdinal(199) => BUILTIN_TYPED_ARRAY_PROTOTYPE_KEYS_FUNCTION_ID,
        debug: "TypedArray.prototype.keys",
        flags: [],
        installer: None,
        native: "keys",
    }
    TypedArrayPrototypeEntries {
        function: FunctionOrdinal(200) => BUILTIN_TYPED_ARRAY_PROTOTYPE_ENTRIES_FUNCTION_ID,
        debug: "TypedArray.prototype.entries",
        flags: [],
        installer: None,
        native: "entries",
    }
    TypedArrayPrototypeJoin {
        function: FunctionOrdinal(201) => BUILTIN_TYPED_ARRAY_PROTOTYPE_JOIN_FUNCTION_ID,
        debug: "TypedArray.prototype.join",
        flags: [],
        installer: None,
        native: "join",
    }
    TypedArrayPrototypeToLocaleString {
        function: FunctionOrdinal(202) => BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "TypedArray.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    TypedArrayPrototypeSubarray {
        function: FunctionOrdinal(203) => BUILTIN_TYPED_ARRAY_PROTOTYPE_SUBARRAY_FUNCTION_ID,
        debug: "TypedArray.prototype.subarray",
        flags: [],
        installer: None,
        native: "subarray",
    }
    TypedArrayPrototypeSlice {
        function: FunctionOrdinal(204) => BUILTIN_TYPED_ARRAY_PROTOTYPE_SLICE_FUNCTION_ID,
        debug: "TypedArray.prototype.slice",
        flags: [],
        installer: None,
        native: "slice",
    }
    TypedArrayPrototypeSet {
        function: FunctionOrdinal(205) => BUILTIN_TYPED_ARRAY_PROTOTYPE_SET_FUNCTION_ID,
        debug: "TypedArray.prototype.set",
        flags: [],
        installer: None,
        native: "set",
    }
    TypedArrayPrototypeReverse {
        function: FunctionOrdinal(206) => BUILTIN_TYPED_ARRAY_PROTOTYPE_REVERSE_FUNCTION_ID,
        debug: "TypedArray.prototype.reverse",
        flags: [],
        installer: None,
        native: "reverse",
    }
    TypedArrayPrototypeSort {
        function: FunctionOrdinal(207) => BUILTIN_TYPED_ARRAY_PROTOTYPE_SORT_FUNCTION_ID,
        debug: "TypedArray.prototype.sort",
        flags: [],
        installer: None,
        native: "sort",
    }
    TypedArrayPrototypeToReversed {
        function: FunctionOrdinal(208) => BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_REVERSED_FUNCTION_ID,
        debug: "TypedArray.prototype.toReversed",
        flags: [],
        installer: None,
        native: "toReversed",
    }
    TypedArrayPrototypeToSorted {
        function: FunctionOrdinal(209) => BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_SORTED_FUNCTION_ID,
        debug: "TypedArray.prototype.toSorted",
        flags: [],
        installer: None,
        native: "toSorted",
    }
    TypedArrayPrototypeWith {
        function: FunctionOrdinal(210) => BUILTIN_TYPED_ARRAY_PROTOTYPE_WITH_FUNCTION_ID,
        debug: "TypedArray.prototype.with",
        flags: [],
        installer: None,
        native: "with",
    }
    TypedArrayFrom {
        function: FunctionOrdinal(211) => BUILTIN_TYPED_ARRAY_FROM_FUNCTION_ID,
        debug: "TypedArray.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TypedArrayOf {
        function: FunctionOrdinal(212) => BUILTIN_TYPED_ARRAY_OF_FUNCTION_ID,
        debug: "TypedArray.of",
        flags: [],
        installer: None,
        native: "of",
    }
    DataViewPrototypeGetUint8 {
        function: FunctionOrdinal(213) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT8_FUNCTION_ID,
        debug: "DataView.prototype.getUint8",
        flags: [],
        installer: None,
        native: "getUint8",
    }
    DataViewPrototypeSetUint8 {
        function: FunctionOrdinal(214) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT8_FUNCTION_ID,
        debug: "DataView.prototype.setUint8",
        flags: [],
        installer: None,
        native: "setUint8",
    }
    DataViewPrototypeGetInt8 {
        function: FunctionOrdinal(215) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT8_FUNCTION_ID,
        debug: "DataView.prototype.getInt8",
        flags: [],
        installer: None,
        native: "getInt8",
    }
    DataViewPrototypeSetInt8 {
        function: FunctionOrdinal(216) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT8_FUNCTION_ID,
        debug: "DataView.prototype.setInt8",
        flags: [],
        installer: None,
        native: "setInt8",
    }
    DataViewPrototypeGetUint16 {
        function: FunctionOrdinal(217) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT16_FUNCTION_ID,
        debug: "DataView.prototype.getUint16",
        flags: [],
        installer: None,
        native: "getUint16",
    }
    DataViewPrototypeSetUint16 {
        function: FunctionOrdinal(218) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT16_FUNCTION_ID,
        debug: "DataView.prototype.setUint16",
        flags: [],
        installer: None,
        native: "setUint16",
    }
    DataViewPrototypeGetInt16 {
        function: FunctionOrdinal(219) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT16_FUNCTION_ID,
        debug: "DataView.prototype.getInt16",
        flags: [],
        installer: None,
        native: "getInt16",
    }
    DataViewPrototypeSetInt16 {
        function: FunctionOrdinal(220) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT16_FUNCTION_ID,
        debug: "DataView.prototype.setInt16",
        flags: [],
        installer: None,
        native: "setInt16",
    }
    DataViewPrototypeGetUint32 {
        function: FunctionOrdinal(221) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT32_FUNCTION_ID,
        debug: "DataView.prototype.getUint32",
        flags: [],
        installer: None,
        native: "getUint32",
    }
    DataViewPrototypeSetUint32 {
        function: FunctionOrdinal(222) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT32_FUNCTION_ID,
        debug: "DataView.prototype.setUint32",
        flags: [],
        installer: None,
        native: "setUint32",
    }
    DataViewPrototypeGetInt32 {
        function: FunctionOrdinal(223) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT32_FUNCTION_ID,
        debug: "DataView.prototype.getInt32",
        flags: [],
        installer: None,
        native: "getInt32",
    }
    DataViewPrototypeSetInt32 {
        function: FunctionOrdinal(224) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT32_FUNCTION_ID,
        debug: "DataView.prototype.setInt32",
        flags: [],
        installer: None,
        native: "setInt32",
    }
    DataViewPrototypeGetFloat16 {
        function: FunctionOrdinal(225) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT16_FUNCTION_ID,
        debug: "DataView.prototype.getFloat16",
        flags: [],
        installer: None,
        native: "getFloat16",
    }
    DataViewPrototypeSetFloat16 {
        function: FunctionOrdinal(226) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT16_FUNCTION_ID,
        debug: "DataView.prototype.setFloat16",
        flags: [],
        installer: None,
        native: "setFloat16",
    }
    DataViewPrototypeGetFloat32 {
        function: FunctionOrdinal(227) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT32_FUNCTION_ID,
        debug: "DataView.prototype.getFloat32",
        flags: [],
        installer: None,
        native: "getFloat32",
    }
    DataViewPrototypeSetFloat32 {
        function: FunctionOrdinal(228) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT32_FUNCTION_ID,
        debug: "DataView.prototype.setFloat32",
        flags: [],
        installer: None,
        native: "setFloat32",
    }
    DataViewPrototypeGetFloat64 {
        function: FunctionOrdinal(229) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT64_FUNCTION_ID,
        debug: "DataView.prototype.getFloat64",
        flags: [],
        installer: None,
        native: "getFloat64",
    }
    DataViewPrototypeSetFloat64 {
        function: FunctionOrdinal(230) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT64_FUNCTION_ID,
        debug: "DataView.prototype.setFloat64",
        flags: [],
        installer: None,
        native: "setFloat64",
    }
    DataViewPrototypeGetBigInt64 {
        function: FunctionOrdinal(231) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGINT64_FUNCTION_ID,
        debug: "DataView.prototype.getBigInt64",
        flags: [],
        installer: None,
        native: "getBigInt64",
    }
    DataViewPrototypeSetBigInt64 {
        function: FunctionOrdinal(232) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGINT64_FUNCTION_ID,
        debug: "DataView.prototype.setBigInt64",
        flags: [],
        installer: None,
        native: "setBigInt64",
    }
    DataViewPrototypeGetBigUint64 {
        function: FunctionOrdinal(233) => BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGUINT64_FUNCTION_ID,
        debug: "DataView.prototype.getBigUint64",
        flags: [],
        installer: None,
        native: "getBigUint64",
    }
    DataViewPrototypeSetBigUint64 {
        function: FunctionOrdinal(234) => BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGUINT64_FUNCTION_ID,
        debug: "DataView.prototype.setBigUint64",
        flags: [],
        installer: None,
        native: "setBigUint64",
    }
    DateConstructor {
        function: FunctionOrdinal(235) => BUILTIN_DATE_FUNCTION_ID,
        global: GlobalOrdinal(9),
        global_name: DATE_NAME,
        debug: DATE_NAME,
        flags: [WALL_CLOCK, CONSTRUCTABLE],
        installer: Date,
        native: DATE_NAME,
    }
    DateNow {
        function: FunctionOrdinal(236) => BUILTIN_DATE_NOW_FUNCTION_ID,
        debug: "Date.now",
        flags: [WALL_CLOCK, STATIC_METHOD],
        installer: None,
        native: "now",
    }
    DateParse {
        function: FunctionOrdinal(237) => BUILTIN_DATE_PARSE_FUNCTION_ID,
        debug: "Date.parse",
        flags: [STATIC_METHOD],
        installer: None,
        native: "parse",
    }
    DateUtc {
        function: FunctionOrdinal(238) => BUILTIN_DATE_UTC_FUNCTION_ID,
        debug: "Date.UTC",
        flags: [STATIC_METHOD],
        installer: None,
        native: "UTC",
    }
    DatePrototypeGetTime {
        function: FunctionOrdinal(239) => BUILTIN_DATE_PROTOTYPE_GET_TIME_FUNCTION_ID,
        debug: "Date.prototype.getTime",
        flags: [],
        installer: None,
        native: "getTime",
    }
    DatePrototypeSetTime {
        function: FunctionOrdinal(240) => BUILTIN_DATE_PROTOTYPE_SET_TIME_FUNCTION_ID,
        debug: "Date.prototype.setTime",
        flags: [],
        installer: None,
        native: "setTime",
    }
    DatePrototypeValueOf {
        function: FunctionOrdinal(241) => BUILTIN_DATE_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Date.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    DatePrototypeGetFullYear {
        function: FunctionOrdinal(242) => BUILTIN_DATE_PROTOTYPE_GET_FULL_YEAR_FUNCTION_ID,
        debug: "Date.prototype.getFullYear",
        flags: [],
        installer: None,
        native: "getFullYear",
    }
    DatePrototypeGetUtcFullYear {
        function: FunctionOrdinal(243) => BUILTIN_DATE_PROTOTYPE_GET_UTC_FULL_YEAR_FUNCTION_ID,
        debug: "Date.prototype.getUTCFullYear",
        flags: [],
        installer: None,
        native: "getUTCFullYear",
    }
    DatePrototypeGetMonth {
        function: FunctionOrdinal(244) => BUILTIN_DATE_PROTOTYPE_GET_MONTH_FUNCTION_ID,
        debug: "Date.prototype.getMonth",
        flags: [],
        installer: None,
        native: "getMonth",
    }
    DatePrototypeGetUtcMonth {
        function: FunctionOrdinal(245) => BUILTIN_DATE_PROTOTYPE_GET_UTC_MONTH_FUNCTION_ID,
        debug: "Date.prototype.getUTCMonth",
        flags: [],
        installer: None,
        native: "getUTCMonth",
    }
    DatePrototypeGetDate {
        function: FunctionOrdinal(246) => BUILTIN_DATE_PROTOTYPE_GET_DATE_FUNCTION_ID,
        debug: "Date.prototype.getDate",
        flags: [],
        installer: None,
        native: "getDate",
    }
    DatePrototypeGetUtcDate {
        function: FunctionOrdinal(247) => BUILTIN_DATE_PROTOTYPE_GET_UTC_DATE_FUNCTION_ID,
        debug: "Date.prototype.getUTCDate",
        flags: [],
        installer: None,
        native: "getUTCDate",
    }
    DatePrototypeGetDay {
        function: FunctionOrdinal(248) => BUILTIN_DATE_PROTOTYPE_GET_DAY_FUNCTION_ID,
        debug: "Date.prototype.getDay",
        flags: [],
        installer: None,
        native: "getDay",
    }
    DatePrototypeGetUtcDay {
        function: FunctionOrdinal(249) => BUILTIN_DATE_PROTOTYPE_GET_UTC_DAY_FUNCTION_ID,
        debug: "Date.prototype.getUTCDay",
        flags: [],
        installer: None,
        native: "getUTCDay",
    }
    DatePrototypeGetHours {
        function: FunctionOrdinal(250) => BUILTIN_DATE_PROTOTYPE_GET_HOURS_FUNCTION_ID,
        debug: "Date.prototype.getHours",
        flags: [],
        installer: None,
        native: "getHours",
    }
    DatePrototypeGetUtcHours {
        function: FunctionOrdinal(251) => BUILTIN_DATE_PROTOTYPE_GET_UTC_HOURS_FUNCTION_ID,
        debug: "Date.prototype.getUTCHours",
        flags: [],
        installer: None,
        native: "getUTCHours",
    }
    DatePrototypeGetMinutes {
        function: FunctionOrdinal(252) => BUILTIN_DATE_PROTOTYPE_GET_MINUTES_FUNCTION_ID,
        debug: "Date.prototype.getMinutes",
        flags: [],
        installer: None,
        native: "getMinutes",
    }
    DatePrototypeGetUtcMinutes {
        function: FunctionOrdinal(253) => BUILTIN_DATE_PROTOTYPE_GET_UTC_MINUTES_FUNCTION_ID,
        debug: "Date.prototype.getUTCMinutes",
        flags: [],
        installer: None,
        native: "getUTCMinutes",
    }
    DatePrototypeGetSeconds {
        function: FunctionOrdinal(254) => BUILTIN_DATE_PROTOTYPE_GET_SECONDS_FUNCTION_ID,
        debug: "Date.prototype.getSeconds",
        flags: [],
        installer: None,
        native: "getSeconds",
    }
    DatePrototypeGetUtcSeconds {
        function: FunctionOrdinal(255) => BUILTIN_DATE_PROTOTYPE_GET_UTC_SECONDS_FUNCTION_ID,
        debug: "Date.prototype.getUTCSeconds",
        flags: [],
        installer: None,
        native: "getUTCSeconds",
    }
    DatePrototypeGetMilliseconds {
        function: FunctionOrdinal(256) => BUILTIN_DATE_PROTOTYPE_GET_MILLISECONDS_FUNCTION_ID,
        debug: "Date.prototype.getMilliseconds",
        flags: [],
        installer: None,
        native: "getMilliseconds",
    }
    DatePrototypeGetUtcMilliseconds {
        function: FunctionOrdinal(257) => BUILTIN_DATE_PROTOTYPE_GET_UTC_MILLISECONDS_FUNCTION_ID,
        debug: "Date.prototype.getUTCMilliseconds",
        flags: [],
        installer: None,
        native: "getUTCMilliseconds",
    }
    DatePrototypeGetTimezoneOffset {
        function: FunctionOrdinal(258) => BUILTIN_DATE_PROTOTYPE_GET_TIMEZONE_OFFSET_FUNCTION_ID,
        debug: "Date.prototype.getTimezoneOffset",
        flags: [],
        installer: None,
        native: "getTimezoneOffset",
    }
    DatePrototypeGetYear {
        function: FunctionOrdinal(259) => BUILTIN_DATE_PROTOTYPE_GET_YEAR_FUNCTION_ID,
        debug: "Date.prototype.getYear",
        flags: [],
        installer: None,
        native: "getYear",
    }
    DatePrototypeSetYear {
        function: FunctionOrdinal(260) => BUILTIN_DATE_PROTOTYPE_SET_YEAR_FUNCTION_ID,
        debug: "Date.prototype.setYear",
        flags: [],
        installer: None,
        native: "setYear",
    }
    DatePrototypeSetFullYear {
        function: FunctionOrdinal(261) => BUILTIN_DATE_PROTOTYPE_SET_FULL_YEAR_FUNCTION_ID,
        debug: "Date.prototype.setFullYear",
        flags: [],
        installer: None,
        native: "setFullYear",
    }
    DatePrototypeSetUtcFullYear {
        function: FunctionOrdinal(262) => BUILTIN_DATE_PROTOTYPE_SET_UTC_FULL_YEAR_FUNCTION_ID,
        debug: "Date.prototype.setUTCFullYear",
        flags: [],
        installer: None,
        native: "setUTCFullYear",
    }
    DatePrototypeSetMonth {
        function: FunctionOrdinal(263) => BUILTIN_DATE_PROTOTYPE_SET_MONTH_FUNCTION_ID,
        debug: "Date.prototype.setMonth",
        flags: [],
        installer: None,
        native: "setMonth",
    }
    DatePrototypeSetUtcMonth {
        function: FunctionOrdinal(264) => BUILTIN_DATE_PROTOTYPE_SET_UTC_MONTH_FUNCTION_ID,
        debug: "Date.prototype.setUTCMonth",
        flags: [],
        installer: None,
        native: "setUTCMonth",
    }
    DatePrototypeSetDate {
        function: FunctionOrdinal(265) => BUILTIN_DATE_PROTOTYPE_SET_DATE_FUNCTION_ID,
        debug: "Date.prototype.setDate",
        flags: [],
        installer: None,
        native: "setDate",
    }
    DatePrototypeSetUtcDate {
        function: FunctionOrdinal(266) => BUILTIN_DATE_PROTOTYPE_SET_UTC_DATE_FUNCTION_ID,
        debug: "Date.prototype.setUTCDate",
        flags: [],
        installer: None,
        native: "setUTCDate",
    }
    DatePrototypeSetHours {
        function: FunctionOrdinal(267) => BUILTIN_DATE_PROTOTYPE_SET_HOURS_FUNCTION_ID,
        debug: "Date.prototype.setHours",
        flags: [],
        installer: None,
        native: "setHours",
    }
    DatePrototypeSetUtcHours {
        function: FunctionOrdinal(268) => BUILTIN_DATE_PROTOTYPE_SET_UTC_HOURS_FUNCTION_ID,
        debug: "Date.prototype.setUTCHours",
        flags: [],
        installer: None,
        native: "setUTCHours",
    }
    DatePrototypeSetMinutes {
        function: FunctionOrdinal(269) => BUILTIN_DATE_PROTOTYPE_SET_MINUTES_FUNCTION_ID,
        debug: "Date.prototype.setMinutes",
        flags: [],
        installer: None,
        native: "setMinutes",
    }
    DatePrototypeSetUtcMinutes {
        function: FunctionOrdinal(270) => BUILTIN_DATE_PROTOTYPE_SET_UTC_MINUTES_FUNCTION_ID,
        debug: "Date.prototype.setUTCMinutes",
        flags: [],
        installer: None,
        native: "setUTCMinutes",
    }
    DatePrototypeSetSeconds {
        function: FunctionOrdinal(271) => BUILTIN_DATE_PROTOTYPE_SET_SECONDS_FUNCTION_ID,
        debug: "Date.prototype.setSeconds",
        flags: [],
        installer: None,
        native: "setSeconds",
    }
    DatePrototypeSetUtcSeconds {
        function: FunctionOrdinal(272) => BUILTIN_DATE_PROTOTYPE_SET_UTC_SECONDS_FUNCTION_ID,
        debug: "Date.prototype.setUTCSeconds",
        flags: [],
        installer: None,
        native: "setUTCSeconds",
    }
    DatePrototypeSetMilliseconds {
        function: FunctionOrdinal(273) => BUILTIN_DATE_PROTOTYPE_SET_MILLISECONDS_FUNCTION_ID,
        debug: "Date.prototype.setMilliseconds",
        flags: [],
        installer: None,
        native: "setMilliseconds",
    }
    DatePrototypeSetUtcMilliseconds {
        function: FunctionOrdinal(274) => BUILTIN_DATE_PROTOTYPE_SET_UTC_MILLISECONDS_FUNCTION_ID,
        debug: "Date.prototype.setUTCMilliseconds",
        flags: [],
        installer: None,
        native: "setUTCMilliseconds",
    }
    DatePrototypeToIsoString {
        function: FunctionOrdinal(275) => BUILTIN_DATE_PROTOTYPE_TO_ISO_STRING_FUNCTION_ID,
        debug: "Date.prototype.toISOString",
        flags: [],
        installer: None,
        native: "toISOString",
    }
    DatePrototypeToJson {
        function: FunctionOrdinal(276) => BUILTIN_DATE_PROTOTYPE_TO_JSON_FUNCTION_ID,
        debug: "Date.prototype.toJSON",
        flags: [],
        installer: None,
        native: "toJSON",
    }
    DatePrototypeToPrimitive {
        function: FunctionOrdinal(277) => BUILTIN_DATE_PROTOTYPE_TO_PRIMITIVE_FUNCTION_ID,
        debug: "Date.prototype[Symbol.toPrimitive]",
        flags: [],
        installer: None,
        native: "[Symbol.toPrimitive]",
    }
    DatePrototypeToDateString {
        function: FunctionOrdinal(278) => BUILTIN_DATE_PROTOTYPE_TO_DATE_STRING_FUNCTION_ID,
        debug: "Date.prototype.toDateString",
        flags: [],
        installer: None,
        native: "toDateString",
    }
    DatePrototypeToLocaleDateString {
        function: FunctionOrdinal(279) => BUILTIN_DATE_PROTOTYPE_TO_LOCALE_DATE_STRING_FUNCTION_ID,
        debug: "Date.prototype.toLocaleDateString",
        flags: [],
        installer: None,
        native: "toLocaleDateString",
    }
    DatePrototypeToLocaleString {
        function: FunctionOrdinal(280) => BUILTIN_DATE_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Date.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    DatePrototypeToLocaleTimeString {
        function: FunctionOrdinal(281) => BUILTIN_DATE_PROTOTYPE_TO_LOCALE_TIME_STRING_FUNCTION_ID,
        debug: "Date.prototype.toLocaleTimeString",
        flags: [],
        installer: None,
        native: "toLocaleTimeString",
    }
    DatePrototypeToTemporalInstant {
        function: FunctionOrdinal(282) => BUILTIN_DATE_PROTOTYPE_TO_TEMPORAL_INSTANT_FUNCTION_ID,
        debug: "Date.prototype.toTemporalInstant",
        flags: [],
        installer: None,
        native: "toTemporalInstant",
    }
    DatePrototypeToTimeString {
        function: FunctionOrdinal(283) => BUILTIN_DATE_PROTOTYPE_TO_TIME_STRING_FUNCTION_ID,
        debug: "Date.prototype.toTimeString",
        flags: [],
        installer: None,
        native: "toTimeString",
    }
    DatePrototypeToString {
        function: FunctionOrdinal(284) => BUILTIN_DATE_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Date.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    DatePrototypeToUtcString {
        function: FunctionOrdinal(285) => BUILTIN_DATE_PROTOTYPE_TO_UTC_STRING_FUNCTION_ID,
        debug: "Date.prototype.toUTCString",
        flags: [],
        installer: None,
        native: "toUTCString",
    }
    TemporalPlainDateConstructor {
        function: FunctionOrdinal(286) => BUILTIN_TEMPORAL_PLAIN_DATE_FUNCTION_ID,
        debug: "Temporal.PlainDate",
        flags: [CONSTRUCTABLE],
        installer: TemporalPlainDate,
        native: "PlainDate",
    }
    TemporalPlainDateFrom {
        function: FunctionOrdinal(287) => BUILTIN_TEMPORAL_PLAIN_DATE_FROM_FUNCTION_ID,
        debug: "Temporal.PlainDate.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TemporalPlainDateCompare {
        function: FunctionOrdinal(288) => BUILTIN_TEMPORAL_PLAIN_DATE_COMPARE_FUNCTION_ID,
        debug: "Temporal.PlainDate.compare",
        flags: [],
        installer: None,
        native: "compare",
    }
    TemporalPlainDatePrototypeCalendarIdGetter {
        function: FunctionOrdinal(289) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_CALENDAR_ID_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.calendarId",
        flags: [],
        installer: None,
        native: "get calendarId",
    }
    TemporalPlainDatePrototypeEraGetter {
        function: FunctionOrdinal(290) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_ERA_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.era",
        flags: [],
        installer: None,
        native: "get era",
    }
    TemporalPlainDatePrototypeEraYearGetter {
        function: FunctionOrdinal(291) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_ERA_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.eraYear",
        flags: [],
        installer: None,
        native: "get eraYear",
    }
    TemporalPlainDatePrototypeYearGetter {
        function: FunctionOrdinal(292) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.year",
        flags: [],
        installer: None,
        native: "get year",
    }
    TemporalPlainDatePrototypeMonthGetter {
        function: FunctionOrdinal(293) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_MONTH_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.month",
        flags: [],
        installer: None,
        native: "get month",
    }
    TemporalPlainDatePrototypeMonthCodeGetter {
        function: FunctionOrdinal(294) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_MONTH_CODE_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.monthCode",
        flags: [],
        installer: None,
        native: "get monthCode",
    }
    TemporalPlainDatePrototypeDayGetter {
        function: FunctionOrdinal(295) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_DAY_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.day",
        flags: [],
        installer: None,
        native: "get day",
    }
    TemporalPlainDatePrototypeDayOfWeekGetter {
        function: FunctionOrdinal(296) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_DAY_OF_WEEK_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.dayOfWeek",
        flags: [],
        installer: None,
        native: "get dayOfWeek",
    }
    TemporalPlainDatePrototypeDayOfYearGetter {
        function: FunctionOrdinal(297) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_DAY_OF_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.dayOfYear",
        flags: [],
        installer: None,
        native: "get dayOfYear",
    }
    TemporalPlainDatePrototypeWeekOfYearGetter {
        function: FunctionOrdinal(298) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_WEEK_OF_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.weekOfYear",
        flags: [],
        installer: None,
        native: "get weekOfYear",
    }
    TemporalPlainDatePrototypeYearOfWeekGetter {
        function: FunctionOrdinal(299) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_YEAR_OF_WEEK_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.yearOfWeek",
        flags: [],
        installer: None,
        native: "get yearOfWeek",
    }
    TemporalPlainDatePrototypeDaysInWeekGetter {
        function: FunctionOrdinal(300) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_DAYS_IN_WEEK_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.daysInWeek",
        flags: [],
        installer: None,
        native: "get daysInWeek",
    }
    TemporalPlainDatePrototypeDaysInMonthGetter {
        function: FunctionOrdinal(301) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_DAYS_IN_MONTH_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.daysInMonth",
        flags: [],
        installer: None,
        native: "get daysInMonth",
    }
    TemporalPlainDatePrototypeDaysInYearGetter {
        function: FunctionOrdinal(302) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_DAYS_IN_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.daysInYear",
        flags: [],
        installer: None,
        native: "get daysInYear",
    }
    TemporalPlainDatePrototypeMonthsInYearGetter {
        function: FunctionOrdinal(303) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_MONTHS_IN_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.monthsInYear",
        flags: [],
        installer: None,
        native: "get monthsInYear",
    }
    TemporalPlainDatePrototypeInLeapYearGetter {
        function: FunctionOrdinal(304) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_IN_LEAP_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDate.prototype.inLeapYear",
        flags: [],
        installer: None,
        native: "get inLeapYear",
    }
    TemporalPlainDatePrototypeWith {
        function: FunctionOrdinal(305) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_WITH_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.with",
        flags: [],
        installer: None,
        native: "with",
    }
    TemporalPlainDatePrototypeWithCalendar {
        function: FunctionOrdinal(306) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_WITH_CALENDAR_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.withCalendar",
        flags: [],
        installer: None,
        native: "withCalendar",
    }
    TemporalPlainDatePrototypeEquals {
        function: FunctionOrdinal(307) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_EQUALS_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.equals",
        flags: [],
        installer: None,
        native: "equals",
    }
    TemporalPlainDatePrototypeToString {
        function: FunctionOrdinal(308) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    TemporalPlainDatePrototypeToJson {
        function: FunctionOrdinal(309) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_TO_JSON_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.toJSON",
        flags: [],
        installer: None,
        native: "toJSON",
    }
    TemporalPlainDatePrototypeToLocaleString {
        function: FunctionOrdinal(310) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    TemporalPlainDatePrototypeValueOf {
        function: FunctionOrdinal(311) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    TemporalPlainDatePrototypeAdd {
        function: FunctionOrdinal(312) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_ADD_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.add",
        flags: [],
        installer: None,
        native: "add",
    }
    TemporalPlainDatePrototypeSubtract {
        function: FunctionOrdinal(313) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_SUBTRACT_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.subtract",
        flags: [],
        installer: None,
        native: "subtract",
    }
    TemporalPlainDatePrototypeUntil {
        function: FunctionOrdinal(314) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_UNTIL_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.until",
        flags: [],
        installer: None,
        native: "until",
    }
    TemporalPlainDatePrototypeSince {
        function: FunctionOrdinal(315) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_SINCE_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.since",
        flags: [],
        installer: None,
        native: "since",
    }
    TemporalPlainDatePrototypeToPlainDateTime {
        function: FunctionOrdinal(316) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_TO_PLAIN_DATE_TIME_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.toPlainDateTime",
        flags: [],
        installer: None,
        native: "toPlainDateTime",
    }
    TemporalPlainDatePrototypeToPlainYearMonth {
        function: FunctionOrdinal(317) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_TO_PLAIN_YEAR_MONTH_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.toPlainYearMonth",
        flags: [],
        installer: None,
        native: "toPlainYearMonth",
    }
    TemporalPlainDatePrototypeToPlainMonthDay {
        function: FunctionOrdinal(318) => BUILTIN_TEMPORAL_PLAIN_DATE_PROTOTYPE_TO_PLAIN_MONTH_DAY_FUNCTION_ID,
        debug: "Temporal.PlainDate.prototype.toPlainMonthDay",
        flags: [],
        installer: None,
        native: "toPlainMonthDay",
    }
    TemporalPlainTimeConstructor {
        function: FunctionOrdinal(355) => BUILTIN_TEMPORAL_PLAIN_TIME_FUNCTION_ID,
        debug: "Temporal.PlainTime",
        flags: [CONSTRUCTABLE],
        installer: TemporalPlainTime,
        native: "PlainTime",
    }
    TemporalPlainTimeFrom {
        function: FunctionOrdinal(356) => BUILTIN_TEMPORAL_PLAIN_TIME_FROM_FUNCTION_ID,
        debug: "Temporal.PlainTime.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TemporalPlainTimeCompare {
        function: FunctionOrdinal(357) => BUILTIN_TEMPORAL_PLAIN_TIME_COMPARE_FUNCTION_ID,
        debug: "Temporal.PlainTime.compare",
        flags: [],
        installer: None,
        native: "compare",
    }
    TemporalPlainTimePrototypeHourGetter {
        function: FunctionOrdinal(358) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_HOUR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainTime.prototype.hour",
        flags: [],
        installer: None,
        native: "get hour",
    }
    TemporalPlainTimePrototypeMinuteGetter {
        function: FunctionOrdinal(359) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_MINUTE_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainTime.prototype.minute",
        flags: [],
        installer: None,
        native: "get minute",
    }
    TemporalPlainTimePrototypeSecondGetter {
        function: FunctionOrdinal(360) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_SECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainTime.prototype.second",
        flags: [],
        installer: None,
        native: "get second",
    }
    TemporalPlainTimePrototypeMillisecondGetter {
        function: FunctionOrdinal(361) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_MILLISECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainTime.prototype.millisecond",
        flags: [],
        installer: None,
        native: "get millisecond",
    }
    TemporalPlainTimePrototypeMicrosecondGetter {
        function: FunctionOrdinal(362) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_MICROSECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainTime.prototype.microsecond",
        flags: [],
        installer: None,
        native: "get microsecond",
    }
    TemporalPlainTimePrototypeNanosecondGetter {
        function: FunctionOrdinal(363) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_NANOSECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainTime.prototype.nanosecond",
        flags: [],
        installer: None,
        native: "get nanosecond",
    }
    TemporalPlainTimePrototypeWith {
        function: FunctionOrdinal(364) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_WITH_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.with",
        flags: [],
        installer: None,
        native: "with",
    }
    TemporalPlainTimePrototypeAdd {
        function: FunctionOrdinal(365) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_ADD_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.add",
        flags: [],
        installer: None,
        native: "add",
    }
    TemporalPlainTimePrototypeSubtract {
        function: FunctionOrdinal(366) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_SUBTRACT_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.subtract",
        flags: [],
        installer: None,
        native: "subtract",
    }
    TemporalPlainTimePrototypeUntil {
        function: FunctionOrdinal(367) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_UNTIL_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.until",
        flags: [],
        installer: None,
        native: "until",
    }
    TemporalPlainTimePrototypeSince {
        function: FunctionOrdinal(368) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_SINCE_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.since",
        flags: [],
        installer: None,
        native: "since",
    }
    TemporalPlainTimePrototypeRound {
        function: FunctionOrdinal(369) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_ROUND_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.round",
        flags: [],
        installer: None,
        native: "round",
    }
    TemporalPlainTimePrototypeEquals {
        function: FunctionOrdinal(370) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_EQUALS_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.equals",
        flags: [],
        installer: None,
        native: "equals",
    }
    TemporalPlainTimePrototypeToString {
        function: FunctionOrdinal(371) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    TemporalPlainTimePrototypeToJson {
        function: FunctionOrdinal(372) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_TO_JSON_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.toJSON",
        flags: [],
        installer: None,
        native: "toJSON",
    }
    TemporalPlainTimePrototypeToLocaleString {
        function: FunctionOrdinal(373) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    TemporalPlainTimePrototypeValueOf {
        function: FunctionOrdinal(374) => BUILTIN_TEMPORAL_PLAIN_TIME_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Temporal.PlainTime.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    TemporalPlainYearMonthConstructor {
        function: FunctionOrdinal(319) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth",
        flags: [CONSTRUCTABLE],
        installer: TemporalPlainYearMonth,
        native: "PlainYearMonth",
    }
    TemporalPlainYearMonthFrom {
        function: FunctionOrdinal(320) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_FROM_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TemporalPlainYearMonthCompare {
        function: FunctionOrdinal(321) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_COMPARE_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.compare",
        flags: [],
        installer: None,
        native: "compare",
    }
    TemporalPlainYearMonthPrototypeCalendarIdGetter {
        function: FunctionOrdinal(322) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_CALENDAR_ID_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.calendarId",
        flags: [],
        installer: None,
        native: "calendarId",
    }
    TemporalPlainYearMonthPrototypeEraGetter {
        function: FunctionOrdinal(323) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_ERA_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.era",
        flags: [],
        installer: None,
        native: "era",
    }
    TemporalPlainYearMonthPrototypeEraYearGetter {
        function: FunctionOrdinal(324) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_ERA_YEAR_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.eraYear",
        flags: [],
        installer: None,
        native: "eraYear",
    }
    TemporalPlainYearMonthPrototypeYearGetter {
        function: FunctionOrdinal(325) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_YEAR_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.year",
        flags: [],
        installer: None,
        native: "year",
    }
    TemporalPlainYearMonthPrototypeMonthGetter {
        function: FunctionOrdinal(326) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_MONTH_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.month",
        flags: [],
        installer: None,
        native: "month",
    }
    TemporalPlainYearMonthPrototypeMonthCodeGetter {
        function: FunctionOrdinal(327) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_MONTH_CODE_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.monthCode",
        flags: [],
        installer: None,
        native: "monthCode",
    }
    TemporalPlainYearMonthPrototypeDaysInYearGetter {
        function: FunctionOrdinal(328) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_DAYS_IN_YEAR_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.daysInYear",
        flags: [],
        installer: None,
        native: "daysInYear",
    }
    TemporalPlainYearMonthPrototypeDaysInMonthGetter {
        function: FunctionOrdinal(329) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_DAYS_IN_MONTH_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.daysInMonth",
        flags: [],
        installer: None,
        native: "daysInMonth",
    }
    TemporalPlainYearMonthPrototypeMonthsInYearGetter {
        function: FunctionOrdinal(330) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_MONTHS_IN_YEAR_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.monthsInYear",
        flags: [],
        installer: None,
        native: "monthsInYear",
    }
    TemporalPlainYearMonthPrototypeInLeapYearGetter {
        function: FunctionOrdinal(331) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_IN_LEAP_YEAR_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.inLeapYear",
        flags: [],
        installer: None,
        native: "inLeapYear",
    }
    TemporalPlainYearMonthPrototypeWith {
        function: FunctionOrdinal(332) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_WITH_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.with",
        flags: [],
        installer: None,
        native: "with",
    }
    TemporalPlainYearMonthPrototypeAdd {
        function: FunctionOrdinal(333) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_ADD_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.add",
        flags: [],
        installer: None,
        native: "add",
    }
    TemporalPlainYearMonthPrototypeSubtract {
        function: FunctionOrdinal(334) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_SUBTRACT_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.subtract",
        flags: [],
        installer: None,
        native: "subtract",
    }
    TemporalPlainYearMonthPrototypeUntil {
        function: FunctionOrdinal(335) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_UNTIL_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.until",
        flags: [],
        installer: None,
        native: "until",
    }
    TemporalPlainYearMonthPrototypeSince {
        function: FunctionOrdinal(336) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_SINCE_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.since",
        flags: [],
        installer: None,
        native: "since",
    }
    TemporalPlainYearMonthPrototypeEquals {
        function: FunctionOrdinal(337) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_EQUALS_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.equals",
        flags: [],
        installer: None,
        native: "equals",
    }
    TemporalPlainYearMonthPrototypeToString {
        function: FunctionOrdinal(338) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    TemporalPlainYearMonthPrototypeToJson {
        function: FunctionOrdinal(339) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_TO_JSON_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.toJSON",
        flags: [],
        installer: None,
        native: "toJSON",
    }
    TemporalPlainYearMonthPrototypeToLocaleString {
        function: FunctionOrdinal(340) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    TemporalPlainYearMonthPrototypeValueOf {
        function: FunctionOrdinal(341) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    TemporalPlainYearMonthPrototypeToPlainDate {
        function: FunctionOrdinal(342) => BUILTIN_TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_TO_PLAIN_DATE_FUNCTION_ID,
        debug: "Temporal.PlainYearMonth.prototype.toPlainDate",
        flags: [],
        installer: None,
        native: "toPlainDate",
    }
    TemporalPlainMonthDayConstructor {
        function: FunctionOrdinal(343) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay",
        flags: [CONSTRUCTABLE],
        installer: TemporalPlainMonthDay,
        native: "PlainMonthDay",
    }
    TemporalPlainMonthDayFrom {
        function: FunctionOrdinal(344) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_FROM_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TemporalPlainMonthDayPrototypeCalendarIdGetter {
        function: FunctionOrdinal(345) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_CALENDAR_ID_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.calendarId",
        flags: [],
        installer: None,
        native: "calendarId",
    }
    TemporalPlainMonthDayPrototypeMonthCodeGetter {
        function: FunctionOrdinal(346) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_MONTH_CODE_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.monthCode",
        flags: [],
        installer: None,
        native: "monthCode",
    }
    TemporalPlainMonthDayPrototypeDayGetter {
        function: FunctionOrdinal(347) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_DAY_GETTER_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.day",
        flags: [],
        installer: None,
        native: "day",
    }
    TemporalPlainMonthDayPrototypeWith {
        function: FunctionOrdinal(348) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_WITH_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.with",
        flags: [],
        installer: None,
        native: "with",
    }
    TemporalPlainMonthDayPrototypeEquals {
        function: FunctionOrdinal(349) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_EQUALS_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.equals",
        flags: [],
        installer: None,
        native: "equals",
    }
    TemporalPlainMonthDayPrototypeToString {
        function: FunctionOrdinal(350) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    TemporalPlainMonthDayPrototypeToJson {
        function: FunctionOrdinal(351) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_TO_JSON_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.toJSON",
        flags: [],
        installer: None,
        native: "toJSON",
    }
    TemporalPlainMonthDayPrototypeToLocaleString {
        function: FunctionOrdinal(352) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    TemporalPlainMonthDayPrototypeValueOf {
        function: FunctionOrdinal(353) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    TemporalPlainMonthDayPrototypeToPlainDate {
        function: FunctionOrdinal(354) => BUILTIN_TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_TO_PLAIN_DATE_FUNCTION_ID,
        debug: "Temporal.PlainMonthDay.prototype.toPlainDate",
        flags: [],
        installer: None,
        native: "toPlainDate",
    }
    TemporalPlainDateTimeConstructor {
        function: FunctionOrdinal(375) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_FUNCTION_ID,
        debug: "Temporal.PlainDateTime",
        flags: [CONSTRUCTABLE],
        installer: TemporalPlainDateTime,
        native: "PlainDateTime",
    }
    TemporalPlainDateTimeFrom {
        function: FunctionOrdinal(376) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_FROM_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TemporalPlainDateTimeCompare {
        function: FunctionOrdinal(377) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_COMPARE_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.compare",
        flags: [],
        installer: None,
        native: "compare",
    }
    TemporalPlainDateTimePrototypeCalendarIdGetter {
        function: FunctionOrdinal(378) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_CALENDAR_ID_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.calendarId",
        flags: [],
        installer: None,
        native: "get calendarId",
    }
    TemporalPlainDateTimePrototypeEraGetter {
        function: FunctionOrdinal(379) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_ERA_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.era",
        flags: [],
        installer: None,
        native: "get era",
    }
    TemporalPlainDateTimePrototypeEraYearGetter {
        function: FunctionOrdinal(380) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_ERA_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.eraYear",
        flags: [],
        installer: None,
        native: "get eraYear",
    }
    TemporalPlainDateTimePrototypeYearGetter {
        function: FunctionOrdinal(381) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.year",
        flags: [],
        installer: None,
        native: "get year",
    }
    TemporalPlainDateTimePrototypeMonthGetter {
        function: FunctionOrdinal(382) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_MONTH_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.month",
        flags: [],
        installer: None,
        native: "get month",
    }
    TemporalPlainDateTimePrototypeMonthCodeGetter {
        function: FunctionOrdinal(383) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_MONTH_CODE_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.monthCode",
        flags: [],
        installer: None,
        native: "get monthCode",
    }
    TemporalPlainDateTimePrototypeDayGetter {
        function: FunctionOrdinal(384) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_DAY_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.day",
        flags: [],
        installer: None,
        native: "get day",
    }
    TemporalPlainDateTimePrototypeHourGetter {
        function: FunctionOrdinal(385) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_HOUR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.hour",
        flags: [],
        installer: None,
        native: "get hour",
    }
    TemporalPlainDateTimePrototypeMinuteGetter {
        function: FunctionOrdinal(386) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_MINUTE_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.minute",
        flags: [],
        installer: None,
        native: "get minute",
    }
    TemporalPlainDateTimePrototypeSecondGetter {
        function: FunctionOrdinal(387) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_SECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.second",
        flags: [],
        installer: None,
        native: "get second",
    }
    TemporalPlainDateTimePrototypeMillisecondGetter {
        function: FunctionOrdinal(388) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_MILLISECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.millisecond",
        flags: [],
        installer: None,
        native: "get millisecond",
    }
    TemporalPlainDateTimePrototypeMicrosecondGetter {
        function: FunctionOrdinal(389) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_MICROSECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.microsecond",
        flags: [],
        installer: None,
        native: "get microsecond",
    }
    TemporalPlainDateTimePrototypeNanosecondGetter {
        function: FunctionOrdinal(390) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_NANOSECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.nanosecond",
        flags: [],
        installer: None,
        native: "get nanosecond",
    }
    TemporalPlainDateTimePrototypeDayOfWeekGetter {
        function: FunctionOrdinal(391) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_DAY_OF_WEEK_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.dayOfWeek",
        flags: [],
        installer: None,
        native: "get dayOfWeek",
    }
    TemporalPlainDateTimePrototypeDayOfYearGetter {
        function: FunctionOrdinal(392) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_DAY_OF_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.dayOfYear",
        flags: [],
        installer: None,
        native: "get dayOfYear",
    }
    TemporalPlainDateTimePrototypeWeekOfYearGetter {
        function: FunctionOrdinal(393) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_WEEK_OF_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.weekOfYear",
        flags: [],
        installer: None,
        native: "get weekOfYear",
    }
    TemporalPlainDateTimePrototypeYearOfWeekGetter {
        function: FunctionOrdinal(394) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_YEAR_OF_WEEK_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.yearOfWeek",
        flags: [],
        installer: None,
        native: "get yearOfWeek",
    }
    TemporalPlainDateTimePrototypeDaysInWeekGetter {
        function: FunctionOrdinal(395) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_DAYS_IN_WEEK_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.daysInWeek",
        flags: [],
        installer: None,
        native: "get daysInWeek",
    }
    TemporalPlainDateTimePrototypeDaysInMonthGetter {
        function: FunctionOrdinal(396) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_DAYS_IN_MONTH_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.daysInMonth",
        flags: [],
        installer: None,
        native: "get daysInMonth",
    }
    TemporalPlainDateTimePrototypeDaysInYearGetter {
        function: FunctionOrdinal(397) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_DAYS_IN_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.daysInYear",
        flags: [],
        installer: None,
        native: "get daysInYear",
    }
    TemporalPlainDateTimePrototypeMonthsInYearGetter {
        function: FunctionOrdinal(398) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_MONTHS_IN_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.monthsInYear",
        flags: [],
        installer: None,
        native: "get monthsInYear",
    }
    TemporalPlainDateTimePrototypeInLeapYearGetter {
        function: FunctionOrdinal(399) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_IN_LEAP_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.PlainDateTime.prototype.inLeapYear",
        flags: [],
        installer: None,
        native: "get inLeapYear",
    }
    TemporalPlainDateTimePrototypeWith {
        function: FunctionOrdinal(400) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_WITH_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.with",
        flags: [],
        installer: None,
        native: "with",
    }
    TemporalPlainDateTimePrototypeWithPlainTime {
        function: FunctionOrdinal(401) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_WITH_PLAIN_TIME_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.withPlainTime",
        flags: [],
        installer: None,
        native: "withPlainTime",
    }
    TemporalPlainDateTimePrototypeWithCalendar {
        function: FunctionOrdinal(402) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_WITH_CALENDAR_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.withCalendar",
        flags: [],
        installer: None,
        native: "withCalendar",
    }
    TemporalPlainDateTimePrototypeAdd {
        function: FunctionOrdinal(403) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_ADD_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.add",
        flags: [],
        installer: None,
        native: "add",
    }
    TemporalPlainDateTimePrototypeSubtract {
        function: FunctionOrdinal(404) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_SUBTRACT_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.subtract",
        flags: [],
        installer: None,
        native: "subtract",
    }
    TemporalPlainDateTimePrototypeUntil {
        function: FunctionOrdinal(405) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_UNTIL_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.until",
        flags: [],
        installer: None,
        native: "until",
    }
    TemporalPlainDateTimePrototypeSince {
        function: FunctionOrdinal(406) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_SINCE_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.since",
        flags: [],
        installer: None,
        native: "since",
    }
    TemporalPlainDateTimePrototypeRound {
        function: FunctionOrdinal(407) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_ROUND_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.round",
        flags: [],
        installer: None,
        native: "round",
    }
    TemporalPlainDateTimePrototypeEquals {
        function: FunctionOrdinal(408) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_EQUALS_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.equals",
        flags: [],
        installer: None,
        native: "equals",
    }
    TemporalPlainDateTimePrototypeToString {
        function: FunctionOrdinal(409) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    TemporalPlainDateTimePrototypeToJson {
        function: FunctionOrdinal(410) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_TO_JSON_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.toJSON",
        flags: [],
        installer: None,
        native: "toJSON",
    }
    TemporalPlainDateTimePrototypeToLocaleString {
        function: FunctionOrdinal(411) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    TemporalPlainDateTimePrototypeValueOf {
        function: FunctionOrdinal(412) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    TemporalPlainDateTimePrototypeToPlainDate {
        function: FunctionOrdinal(413) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_TO_PLAIN_DATE_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.toPlainDate",
        flags: [],
        installer: None,
        native: "toPlainDate",
    }
    TemporalPlainDateTimePrototypeToPlainTime {
        function: FunctionOrdinal(414) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_TO_PLAIN_TIME_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.toPlainTime",
        flags: [],
        installer: None,
        native: "toPlainTime",
    }
    TemporalPlainDateTimePrototypeToZonedDateTime {
        function: FunctionOrdinal(415) => BUILTIN_TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_TO_ZONED_DATE_TIME_FUNCTION_ID,
        debug: "Temporal.PlainDateTime.prototype.toZonedDateTime",
        flags: [],
        installer: None,
        native: "toZonedDateTime",
    }
    TemporalDurationConstructor {
        function: FunctionOrdinal(416) => BUILTIN_TEMPORAL_DURATION_FUNCTION_ID,
        debug: "Temporal.Duration",
        flags: [CONSTRUCTABLE],
        installer: TemporalDuration,
        native: "Duration",
    }
    TemporalDurationFrom {
        function: FunctionOrdinal(417) => BUILTIN_TEMPORAL_DURATION_FROM_FUNCTION_ID,
        debug: "Temporal.Duration.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TemporalDurationCompare {
        function: FunctionOrdinal(418) => BUILTIN_TEMPORAL_DURATION_COMPARE_FUNCTION_ID,
        debug: "Temporal.Duration.compare",
        flags: [],
        installer: None,
        native: "compare",
    }
    TemporalDurationPrototypeYearsGetter {
        function: FunctionOrdinal(419) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_YEARS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.years",
        flags: [],
        installer: None,
        native: "get years",
    }
    TemporalDurationPrototypeMonthsGetter {
        function: FunctionOrdinal(420) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_MONTHS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.months",
        flags: [],
        installer: None,
        native: "get months",
    }
    TemporalDurationPrototypeWeeksGetter {
        function: FunctionOrdinal(421) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_WEEKS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.weeks",
        flags: [],
        installer: None,
        native: "get weeks",
    }
    TemporalDurationPrototypeDaysGetter {
        function: FunctionOrdinal(422) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_DAYS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.days",
        flags: [],
        installer: None,
        native: "get days",
    }
    TemporalDurationPrototypeHoursGetter {
        function: FunctionOrdinal(423) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_HOURS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.hours",
        flags: [],
        installer: None,
        native: "get hours",
    }
    TemporalDurationPrototypeMinutesGetter {
        function: FunctionOrdinal(424) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_MINUTES_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.minutes",
        flags: [],
        installer: None,
        native: "get minutes",
    }
    TemporalDurationPrototypeSecondsGetter {
        function: FunctionOrdinal(425) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_SECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.seconds",
        flags: [],
        installer: None,
        native: "get seconds",
    }
    TemporalDurationPrototypeMillisecondsGetter {
        function: FunctionOrdinal(426) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_MILLISECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.milliseconds",
        flags: [],
        installer: None,
        native: "get milliseconds",
    }
    TemporalDurationPrototypeMicrosecondsGetter {
        function: FunctionOrdinal(427) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_MICROSECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.microseconds",
        flags: [],
        installer: None,
        native: "get microseconds",
    }
    TemporalDurationPrototypeNanosecondsGetter {
        function: FunctionOrdinal(428) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_NANOSECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.nanoseconds",
        flags: [],
        installer: None,
        native: "get nanoseconds",
    }
    TemporalDurationPrototypeSignGetter {
        function: FunctionOrdinal(429) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_SIGN_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.sign",
        flags: [],
        installer: None,
        native: "get sign",
    }
    TemporalDurationPrototypeBlankGetter {
        function: FunctionOrdinal(430) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_BLANK_GETTER_FUNCTION_ID,
        debug: "get Temporal.Duration.prototype.blank",
        flags: [],
        installer: None,
        native: "get blank",
    }
    TemporalDurationPrototypeWith {
        function: FunctionOrdinal(431) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_WITH_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.with",
        flags: [],
        installer: None,
        native: "with",
    }
    TemporalDurationPrototypeNegated {
        function: FunctionOrdinal(432) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_NEGATED_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.negated",
        flags: [],
        installer: None,
        native: "negated",
    }
    TemporalDurationPrototypeAbs {
        function: FunctionOrdinal(433) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_ABS_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.abs",
        flags: [],
        installer: None,
        native: "abs",
    }
    TemporalDurationPrototypeAdd {
        function: FunctionOrdinal(434) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_ADD_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.add",
        flags: [],
        installer: None,
        native: "add",
    }
    TemporalDurationPrototypeSubtract {
        function: FunctionOrdinal(435) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_SUBTRACT_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.subtract",
        flags: [],
        installer: None,
        native: "subtract",
    }
    TemporalDurationPrototypeRound {
        function: FunctionOrdinal(436) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_ROUND_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.round",
        flags: [],
        installer: None,
        native: "round",
    }
    TemporalDurationPrototypeTotal {
        function: FunctionOrdinal(437) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_TOTAL_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.total",
        flags: [],
        installer: None,
        native: "total",
    }
    TemporalDurationPrototypeToString {
        function: FunctionOrdinal(438) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    TemporalDurationPrototypeToJson {
        function: FunctionOrdinal(439) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_TO_JSON_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.toJSON",
        flags: [],
        installer: None,
        native: "toJSON",
    }
    TemporalDurationPrototypeToLocaleString {
        function: FunctionOrdinal(440) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    TemporalDurationPrototypeValueOf {
        function: FunctionOrdinal(441) => BUILTIN_TEMPORAL_DURATION_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Temporal.Duration.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    TemporalNowInstant {
        function: FunctionOrdinal(442) => BUILTIN_TEMPORAL_NOW_INSTANT_FUNCTION_ID,
        debug: "Temporal.Now.instant",
        flags: [WALL_CLOCK],
        installer: None,
        native: "instant",
    }
    TemporalNowTimeZoneId {
        function: FunctionOrdinal(443) => BUILTIN_TEMPORAL_NOW_TIME_ZONE_ID_FUNCTION_ID,
        debug: "Temporal.Now.timeZoneId",
        flags: [],
        installer: None,
        native: "timeZoneId",
    }
    TemporalNowZonedDateTimeIso {
        function: FunctionOrdinal(444) => BUILTIN_TEMPORAL_NOW_ZONED_DATE_TIME_ISO_FUNCTION_ID,
        debug: "Temporal.Now.zonedDateTimeISO",
        flags: [WALL_CLOCK],
        installer: None,
        native: "zonedDateTimeISO",
    }
    TemporalInstantConstructor {
        function: FunctionOrdinal(445) => BUILTIN_TEMPORAL_INSTANT_FUNCTION_ID,
        debug: "Temporal.Instant",
        flags: [CONSTRUCTABLE],
        installer: TemporalInstant,
        native: TEMPORAL_INSTANT_NAME,
    }
    TemporalInstantFrom {
        function: FunctionOrdinal(446) => BUILTIN_TEMPORAL_INSTANT_FROM_FUNCTION_ID,
        debug: "Temporal.Instant.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TemporalInstantCompare {
        function: FunctionOrdinal(447) => BUILTIN_TEMPORAL_INSTANT_COMPARE_FUNCTION_ID,
        debug: "Temporal.Instant.compare",
        flags: [],
        installer: None,
        native: "compare",
    }
    TemporalInstantFromEpochMilliseconds {
        function: FunctionOrdinal(448) => BUILTIN_TEMPORAL_INSTANT_FROM_EPOCH_MILLISECONDS_FUNCTION_ID,
        debug: "Temporal.Instant.fromEpochMilliseconds",
        flags: [],
        installer: None,
        native: "fromEpochMilliseconds",
    }
    TemporalInstantFromEpochNanoseconds {
        function: FunctionOrdinal(449) => BUILTIN_TEMPORAL_INSTANT_FROM_EPOCH_NANOSECONDS_FUNCTION_ID,
        debug: "Temporal.Instant.fromEpochNanoseconds",
        flags: [],
        installer: None,
        native: "fromEpochNanoseconds",
    }
    TemporalInstantPrototypeEpochMillisecondsGetter {
        function: FunctionOrdinal(450) => BUILTIN_TEMPORAL_INSTANT_PROTOTYPE_EPOCH_MILLISECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Instant.prototype.epochMilliseconds",
        flags: [],
        installer: None,
        native: "get epochMilliseconds",
    }
    TemporalInstantPrototypeEpochNanosecondsGetter {
        function: FunctionOrdinal(451) => BUILTIN_TEMPORAL_INSTANT_PROTOTYPE_EPOCH_NANOSECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.Instant.prototype.epochNanoseconds",
        flags: [],
        installer: None,
        native: "get epochNanoseconds",
    }
    TemporalInstantPrototypeEquals {
        function: FunctionOrdinal(452) => BUILTIN_TEMPORAL_INSTANT_PROTOTYPE_EQUALS_FUNCTION_ID,
        debug: "Temporal.Instant.prototype.equals",
        flags: [],
        installer: None,
        native: "equals",
    }
    TemporalInstantPrototypeToString {
        function: FunctionOrdinal(453) => BUILTIN_TEMPORAL_INSTANT_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Temporal.Instant.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    TemporalInstantPrototypeToJson {
        function: FunctionOrdinal(454) => BUILTIN_TEMPORAL_INSTANT_PROTOTYPE_TO_JSON_FUNCTION_ID,
        debug: "Temporal.Instant.prototype.toJSON",
        flags: [],
        installer: None,
        native: "toJSON",
    }
    TemporalInstantPrototypeValueOf {
        function: FunctionOrdinal(455) => BUILTIN_TEMPORAL_INSTANT_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "Temporal.Instant.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    TemporalZonedDateTimeConstructor {
        function: FunctionOrdinal(456) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime",
        flags: [CONSTRUCTABLE],
        installer: TemporalZonedDateTime,
        native: "ZonedDateTime",
    }
    TemporalZonedDateTimeFrom {
        function: FunctionOrdinal(457) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_FROM_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.from",
        flags: [],
        installer: None,
        native: "from",
    }
    TemporalZonedDateTimePrototypeEpochMillisecondsGetter {
        function: FunctionOrdinal(458) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_EPOCH_MILLISECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.epochMilliseconds",
        flags: [],
        installer: None,
        native: "get epochMilliseconds",
    }
    TemporalZonedDateTimePrototypeEpochNanosecondsGetter {
        function: FunctionOrdinal(459) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_EPOCH_NANOSECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.epochNanoseconds",
        flags: [],
        installer: None,
        native: "get epochNanoseconds",
    }
    TemporalZonedDateTimePrototypeOffsetGetter {
        function: FunctionOrdinal(460) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_OFFSET_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.offset",
        flags: [],
        installer: None,
        native: "get offset",
    }
    TemporalZonedDateTimePrototypeOffsetNanosecondsGetter {
        function: FunctionOrdinal(461) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_OFFSET_NANOSECONDS_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.offsetNanoseconds",
        flags: [],
        installer: None,
        native: "get offsetNanoseconds",
    }
    TemporalZonedDateTimePrototypeTimeZoneIdGetter {
        function: FunctionOrdinal(462) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_TIME_ZONE_ID_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.timeZoneId",
        flags: [],
        installer: None,
        native: "get timeZoneId",
    }
    TemporalZonedDateTimePrototypeCalendarIdGetter {
        function: FunctionOrdinal(463) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_CALENDAR_ID_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.calendarId",
        flags: [],
        installer: None,
        native: "get calendarId",
    }
    TemporalZonedDateTimePrototypeEraGetter {
        function: FunctionOrdinal(464) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_ERA_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.era",
        flags: [],
        installer: None,
        native: "get era",
    }
    TemporalZonedDateTimePrototypeEraYearGetter {
        function: FunctionOrdinal(465) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_ERA_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.eraYear",
        flags: [],
        installer: None,
        native: "get eraYear",
    }
    TemporalZonedDateTimePrototypeYearGetter {
        function: FunctionOrdinal(466) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_YEAR_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.year",
        flags: [],
        installer: None,
        native: "get year",
    }
    TemporalZonedDateTimePrototypeMonthGetter {
        function: FunctionOrdinal(467) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_MONTH_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.month",
        flags: [],
        installer: None,
        native: "get month",
    }
    TemporalZonedDateTimePrototypeMonthCodeGetter {
        function: FunctionOrdinal(468) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_MONTH_CODE_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.monthCode",
        flags: [],
        installer: None,
        native: "get monthCode",
    }
    TemporalZonedDateTimePrototypeDayGetter {
        function: FunctionOrdinal(469) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_DAY_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.day",
        flags: [],
        installer: None,
        native: "get day",
    }
    TemporalZonedDateTimePrototypeHourGetter {
        function: FunctionOrdinal(470) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_HOUR_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.hour",
        flags: [],
        installer: None,
        native: "get hour",
    }
    TemporalZonedDateTimePrototypeMinuteGetter {
        function: FunctionOrdinal(471) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_MINUTE_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.minute",
        flags: [],
        installer: None,
        native: "get minute",
    }
    TemporalZonedDateTimePrototypeSecondGetter {
        function: FunctionOrdinal(472) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_SECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.second",
        flags: [],
        installer: None,
        native: "get second",
    }
    TemporalZonedDateTimePrototypeMillisecondGetter {
        function: FunctionOrdinal(473) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_MILLISECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.millisecond",
        flags: [],
        installer: None,
        native: "get millisecond",
    }
    TemporalZonedDateTimePrototypeMicrosecondGetter {
        function: FunctionOrdinal(474) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_MICROSECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.microsecond",
        flags: [],
        installer: None,
        native: "get microsecond",
    }
    TemporalZonedDateTimePrototypeNanosecondGetter {
        function: FunctionOrdinal(475) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_NANOSECOND_GETTER_FUNCTION_ID,
        debug: "get Temporal.ZonedDateTime.prototype.nanosecond",
        flags: [],
        installer: None,
        native: "get nanosecond",
    }
    TemporalZonedDateTimePrototypeEquals {
        function: FunctionOrdinal(476) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_EQUALS_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.equals",
        flags: [],
        installer: None,
        native: "equals",
    }
    TemporalZonedDateTimePrototypeToInstant {
        function: FunctionOrdinal(477) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_TO_INSTANT_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.toInstant",
        flags: [],
        installer: None,
        native: "toInstant",
    }
    TemporalZonedDateTimePrototypeToPlainDateTime {
        function: FunctionOrdinal(478) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_TO_PLAIN_DATE_TIME_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.toPlainDateTime",
        flags: [],
        installer: None,
        native: "toPlainDateTime",
    }
    TemporalZonedDateTimePrototypeWithTimeZone {
        function: FunctionOrdinal(479) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_WITH_TIME_ZONE_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.withTimeZone",
        flags: [],
        installer: None,
        native: "withTimeZone",
    }
    TemporalZonedDateTimePrototypeWithCalendar {
        function: FunctionOrdinal(480) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_WITH_CALENDAR_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.withCalendar",
        flags: [],
        installer: None,
        native: "withCalendar",
    }
    TemporalZonedDateTimePrototypeAdd {
        function: FunctionOrdinal(481) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_ADD_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.add",
        flags: [],
        installer: None,
        native: "add",
    }
    TemporalZonedDateTimePrototypeSubtract {
        function: FunctionOrdinal(482) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_SUBTRACT_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.subtract",
        flags: [],
        installer: None,
        native: "subtract",
    }
    TemporalZonedDateTimePrototypeUntil {
        function: FunctionOrdinal(483) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_UNTIL_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.until",
        flags: [],
        installer: None,
        native: "until",
    }
    TemporalZonedDateTimePrototypeSince {
        function: FunctionOrdinal(484) => BUILTIN_TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_SINCE_FUNCTION_ID,
        debug: "Temporal.ZonedDateTime.prototype.since",
        flags: [],
        installer: None,
        native: "since",
    }
    IntlGetCanonicalLocales {
        function: FunctionOrdinal(485) => BUILTIN_INTL_GET_CANONICAL_LOCALES_FUNCTION_ID,
        debug: "Intl.getCanonicalLocales",
        flags: [],
        installer: None,
        native: "getCanonicalLocales",
    }
    IntlLocaleConstructor {
        function: FunctionOrdinal(486) => BUILTIN_INTL_LOCALE_FUNCTION_ID,
        debug: "Intl.Locale",
        flags: [CONSTRUCTABLE],
        installer: IntlLocale,
        native: INTL_LOCALE_NAME,
    }
    IntlLocalePrototypeLanguageGetter {
        function: FunctionOrdinal(487) => BUILTIN_INTL_LOCALE_PROTOTYPE_LANGUAGE_GETTER_FUNCTION_ID,
        debug: "get Intl.Locale.prototype.language",
        flags: [],
        installer: None,
        native: "get language",
    }
    IntlLocalePrototypeScriptGetter {
        function: FunctionOrdinal(488) => BUILTIN_INTL_LOCALE_PROTOTYPE_SCRIPT_GETTER_FUNCTION_ID,
        debug: "get Intl.Locale.prototype.script",
        flags: [],
        installer: None,
        native: "get script",
    }
    IntlLocalePrototypeRegionGetter {
        function: FunctionOrdinal(489) => BUILTIN_INTL_LOCALE_PROTOTYPE_REGION_GETTER_FUNCTION_ID,
        debug: "get Intl.Locale.prototype.region",
        flags: [],
        installer: None,
        native: "get region",
    }
    IntlLocalePrototypeBaseNameGetter {
        function: FunctionOrdinal(490) => BUILTIN_INTL_LOCALE_PROTOTYPE_BASE_NAME_GETTER_FUNCTION_ID,
        debug: "get Intl.Locale.prototype.baseName",
        flags: [],
        installer: None,
        native: "get baseName",
    }
    IntlLocalePrototypeToString {
        function: FunctionOrdinal(491) => BUILTIN_INTL_LOCALE_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Intl.Locale.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    IntlDateTimeFormatConstructor {
        function: FunctionOrdinal(492) => BUILTIN_INTL_DATE_TIME_FORMAT_FUNCTION_ID,
        debug: "Intl.DateTimeFormat",
        flags: [CONSTRUCTABLE],
        installer: IntlDateTimeFormat,
        native: INTL_DATE_TIME_FORMAT_NAME,
    }
    IntlDateTimeFormatSupportedLocalesOf {
        function: FunctionOrdinal(493) => BUILTIN_INTL_DATE_TIME_FORMAT_SUPPORTED_LOCALES_OF_FUNCTION_ID,
        debug: "Intl.DateTimeFormat.supportedLocalesOf",
        flags: [],
        installer: None,
        native: "supportedLocalesOf",
    }
    IntlDateTimeFormatPrototypeResolvedOptions {
        function: FunctionOrdinal(494) => BUILTIN_INTL_DATE_TIME_FORMAT_PROTOTYPE_RESOLVED_OPTIONS_FUNCTION_ID,
        debug: "Intl.DateTimeFormat.prototype.resolvedOptions",
        flags: [],
        installer: None,
        native: "resolvedOptions",
    }
    IntlDateTimeFormatPrototypeFormatGetter {
        function: FunctionOrdinal(495) => BUILTIN_INTL_DATE_TIME_FORMAT_PROTOTYPE_FORMAT_GETTER_FUNCTION_ID,
        debug: "get Intl.DateTimeFormat.prototype.format",
        flags: [],
        installer: None,
        native: "get format",
    }
    IntlDateTimeFormatPrototypeFormatToParts {
        function: FunctionOrdinal(496) => BUILTIN_INTL_DATE_TIME_FORMAT_PROTOTYPE_FORMAT_TO_PARTS_FUNCTION_ID,
        debug: "Intl.DateTimeFormat.prototype.formatToParts",
        flags: [WALL_CLOCK],
        installer: None,
        native: "formatToParts",
    }
    IntlDateTimeFormatPrototypeFormatRange {
        function: FunctionOrdinal(497) => BUILTIN_INTL_DATE_TIME_FORMAT_PROTOTYPE_FORMAT_RANGE_FUNCTION_ID,
        debug: "Intl.DateTimeFormat.prototype.formatRange",
        flags: [],
        installer: None,
        native: "formatRange",
    }
    IntlDateTimeFormatPrototypeFormatRangeToParts {
        function: FunctionOrdinal(498) => BUILTIN_INTL_DATE_TIME_FORMAT_PROTOTYPE_FORMAT_RANGE_TO_PARTS_FUNCTION_ID,
        debug: "Intl.DateTimeFormat.prototype.formatRangeToParts",
        flags: [],
        installer: None,
        native: "formatRangeToParts",
    }
    IntlDateTimeFormatBoundFormat {
        function: FunctionOrdinal(499) => BUILTIN_INTL_DATE_TIME_FORMAT_BOUND_FORMAT_FUNCTION_ID,
        debug: "Intl.DateTimeFormat Format Function",
        flags: [WALL_CLOCK],
        installer: None,
        native: "",
    }
    RegExpConstructor {
        function: FunctionOrdinal(500) => BUILTIN_REGEXP_FUNCTION_ID,
        global: GlobalOrdinal(10),
        global_name: REGEXP_NAME,
        debug: REGEXP_NAME,
        flags: [CONSTRUCTABLE],
        installer: RegExp,
        native: REGEXP_NAME,
    }
    RegExpSpeciesGetter {
        function: FunctionOrdinal(501) => BUILTIN_REGEXP_SPECIES_GETTER_FUNCTION_ID,
        debug: "get RegExp [Symbol.species]",
        flags: [],
        installer: None,
        native: "get [Symbol.species]",
    }
    RegExpPrototypeFlagsGetter {
        function: FunctionOrdinal(502) => BUILTIN_REGEXP_PROTOTYPE_FLAGS_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.flags",
        flags: [],
        installer: None,
        native: "get flags",
    }
    RegExpPrototypeSourceGetter {
        function: FunctionOrdinal(503) => BUILTIN_REGEXP_PROTOTYPE_SOURCE_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.source",
        flags: [],
        installer: None,
        native: "get source",
    }
    RegExpPrototypeHasIndicesGetter {
        function: FunctionOrdinal(504) => BUILTIN_REGEXP_PROTOTYPE_HAS_INDICES_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.hasIndices",
        flags: [],
        installer: None,
        native: "get hasIndices",
    }
    RegExpPrototypeGlobalGetter {
        function: FunctionOrdinal(505) => BUILTIN_REGEXP_PROTOTYPE_GLOBAL_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.global",
        flags: [],
        installer: None,
        native: "get global",
    }
    RegExpPrototypeIgnoreCaseGetter {
        function: FunctionOrdinal(506) => BUILTIN_REGEXP_PROTOTYPE_IGNORE_CASE_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.ignoreCase",
        flags: [],
        installer: None,
        native: "get ignoreCase",
    }
    RegExpPrototypeMultilineGetter {
        function: FunctionOrdinal(507) => BUILTIN_REGEXP_PROTOTYPE_MULTILINE_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.multiline",
        flags: [],
        installer: None,
        native: "get multiline",
    }
    RegExpPrototypeDotAllGetter {
        function: FunctionOrdinal(508) => BUILTIN_REGEXP_PROTOTYPE_DOT_ALL_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.dotAll",
        flags: [],
        installer: None,
        native: "get dotAll",
    }
    RegExpPrototypeUnicodeGetter {
        function: FunctionOrdinal(509) => BUILTIN_REGEXP_PROTOTYPE_UNICODE_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.unicode",
        flags: [],
        installer: None,
        native: "get unicode",
    }
    RegExpPrototypeUnicodeSetsGetter {
        function: FunctionOrdinal(510) => BUILTIN_REGEXP_PROTOTYPE_UNICODE_SETS_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.unicodeSets",
        flags: [],
        installer: None,
        native: "get unicodeSets",
    }
    RegExpPrototypeStickyGetter {
        function: FunctionOrdinal(511) => BUILTIN_REGEXP_PROTOTYPE_STICKY_GETTER_FUNCTION_ID,
        debug: "get RegExp.prototype.sticky",
        flags: [],
        installer: None,
        native: "get sticky",
    }
    RegExpLegacyStaticGetter {
        function: FunctionOrdinal(512) => BUILTIN_REGEXP_LEGACY_STATIC_GETTER_FUNCTION_ID,
        debug: "get RegExp legacy static",
        flags: [],
        installer: None,
        native: "get RegExp legacy static",
    }
    RegExpLegacyStaticSetter {
        function: FunctionOrdinal(513) => BUILTIN_REGEXP_LEGACY_STATIC_SETTER_FUNCTION_ID,
        debug: "set RegExp legacy static",
        flags: [],
        installer: None,
        native: "set RegExp legacy static",
    }
    RegExpPrototypeCompile {
        function: FunctionOrdinal(514) => BUILTIN_REGEXP_PROTOTYPE_COMPILE_FUNCTION_ID,
        debug: "RegExp.prototype.compile",
        flags: [],
        installer: None,
        native: "compile",
    }
    RegExpPrototypeExec {
        function: FunctionOrdinal(515) => BUILTIN_REGEXP_PROTOTYPE_EXEC_FUNCTION_ID,
        debug: "RegExp.prototype.exec",
        flags: [],
        installer: None,
        native: "exec",
    }
    RegExpPrototypeTest {
        function: FunctionOrdinal(516) => BUILTIN_REGEXP_PROTOTYPE_TEST_FUNCTION_ID,
        debug: "RegExp.prototype.test",
        flags: [],
        installer: None,
        native: "test",
    }
    RegExpPrototypeToString {
        function: FunctionOrdinal(517) => BUILTIN_REGEXP_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "RegExp.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    RegExpPrototypeSymbolMatch {
        function: FunctionOrdinal(518) => BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_FUNCTION_ID,
        debug: "RegExp.prototype[Symbol.match]",
        flags: [],
        installer: None,
        native: "[Symbol.match]",
    }
    RegExpPrototypeSymbolMatchAll {
        function: FunctionOrdinal(519) => BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_FUNCTION_ID,
        debug: "RegExp.prototype[Symbol.matchAll]",
        flags: [],
        installer: None,
        native: "[Symbol.matchAll]",
    }
    RegExpPrototypeSymbolReplace {
        function: FunctionOrdinal(520) => BUILTIN_REGEXP_PROTOTYPE_SYMBOL_REPLACE_FUNCTION_ID,
        debug: "RegExp.prototype[Symbol.replace]",
        flags: [],
        installer: None,
        native: "[Symbol.replace]",
    }
    RegExpPrototypeSymbolSearch {
        function: FunctionOrdinal(521) => BUILTIN_REGEXP_PROTOTYPE_SYMBOL_SEARCH_FUNCTION_ID,
        debug: "RegExp.prototype[Symbol.search]",
        flags: [],
        installer: None,
        native: "[Symbol.search]",
    }
    RegExpPrototypeSymbolSplit {
        function: FunctionOrdinal(522) => BUILTIN_REGEXP_PROTOTYPE_SYMBOL_SPLIT_FUNCTION_ID,
        debug: "RegExp.prototype[Symbol.split]",
        flags: [],
        installer: None,
        native: "[Symbol.split]",
    }
    RegExpEscape {
        function: FunctionOrdinal(523) => BUILTIN_REGEXP_ESCAPE_FUNCTION_ID,
        debug: "RegExp.escape",
        flags: [],
        installer: None,
        native: "escape",
    }
    JsonParse {
        function: FunctionOrdinal(524) => BUILTIN_JSON_PARSE_FUNCTION_ID,
        debug: "JSON.parse",
        flags: [],
        installer: None,
        native: "parse",
    }
    JsonStringify {
        function: FunctionOrdinal(525) => BUILTIN_JSON_STRINGIFY_FUNCTION_ID,
        debug: "JSON.stringify",
        flags: [],
        installer: None,
        native: "stringify",
    }
    JsonRawJson {
        function: FunctionOrdinal(526) => BUILTIN_JSON_RAW_JSON_FUNCTION_ID,
        debug: "JSON.rawJSON",
        flags: [],
        installer: None,
        native: "rawJSON",
    }
    JsonIsRawJson {
        function: FunctionOrdinal(527) => BUILTIN_JSON_IS_RAW_JSON_FUNCTION_ID,
        debug: "JSON.isRawJSON",
        flags: [],
        installer: None,
        native: "isRawJSON",
    }
    AtomicsAdd {
        function: FunctionOrdinal(528) => BUILTIN_ATOMICS_ADD_FUNCTION_ID,
        debug: "Atomics.add",
        flags: [],
        installer: None,
        native: "add",
    }
    AtomicsAnd {
        function: FunctionOrdinal(529) => BUILTIN_ATOMICS_AND_FUNCTION_ID,
        debug: "Atomics.and",
        flags: [],
        installer: None,
        native: "and",
    }
    AtomicsCompareExchange {
        function: FunctionOrdinal(530) => BUILTIN_ATOMICS_COMPARE_EXCHANGE_FUNCTION_ID,
        debug: "Atomics.compareExchange",
        flags: [],
        installer: None,
        native: "compareExchange",
    }
    AtomicsExchange {
        function: FunctionOrdinal(531) => BUILTIN_ATOMICS_EXCHANGE_FUNCTION_ID,
        debug: "Atomics.exchange",
        flags: [],
        installer: None,
        native: "exchange",
    }
    AtomicsLoad {
        function: FunctionOrdinal(532) => BUILTIN_ATOMICS_LOAD_FUNCTION_ID,
        debug: "Atomics.load",
        flags: [],
        installer: None,
        native: "load",
    }
    AtomicsNotify {
        function: FunctionOrdinal(533) => BUILTIN_ATOMICS_NOTIFY_FUNCTION_ID,
        debug: "Atomics.notify",
        flags: [],
        installer: None,
        native: "notify",
    }
    AtomicsOr {
        function: FunctionOrdinal(534) => BUILTIN_ATOMICS_OR_FUNCTION_ID,
        debug: "Atomics.or",
        flags: [],
        installer: None,
        native: "or",
    }
    AtomicsPause {
        function: FunctionOrdinal(535) => BUILTIN_ATOMICS_PAUSE_FUNCTION_ID,
        debug: "Atomics.pause",
        flags: [],
        installer: None,
        native: "pause",
    }
    AtomicsStore {
        function: FunctionOrdinal(536) => BUILTIN_ATOMICS_STORE_FUNCTION_ID,
        debug: "Atomics.store",
        flags: [],
        installer: None,
        native: "store",
    }
    AtomicsSub {
        function: FunctionOrdinal(537) => BUILTIN_ATOMICS_SUB_FUNCTION_ID,
        debug: "Atomics.sub",
        flags: [],
        installer: None,
        native: "sub",
    }
    AtomicsWait {
        function: FunctionOrdinal(538) => BUILTIN_ATOMICS_WAIT_FUNCTION_ID,
        debug: "Atomics.wait",
        flags: [],
        installer: None,
        native: "wait",
    }
    AtomicsWaitAsync {
        function: FunctionOrdinal(539) => BUILTIN_ATOMICS_WAIT_ASYNC_FUNCTION_ID,
        debug: "Atomics.waitAsync",
        flags: [],
        installer: None,
        native: "waitAsync",
    }
    AtomicsXor {
        function: FunctionOrdinal(540) => BUILTIN_ATOMICS_XOR_FUNCTION_ID,
        debug: "Atomics.xor",
        flags: [],
        installer: None,
        native: "xor",
    }
    AtomicsIsLockFree {
        function: FunctionOrdinal(541) => BUILTIN_ATOMICS_IS_LOCK_FREE_FUNCTION_ID,
        debug: "Atomics.isLockFree",
        flags: [],
        installer: None,
        native: "isLockFree",
    }
    Float64ArrayConstructor {
        function: FunctionOrdinal(542) => BUILTIN_FLOAT64_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(11),
        global_name: FLOAT64_ARRAY_NAME,
        debug: FLOAT64_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: FLOAT64_ARRAY_NAME,
    }
    Float32ArrayConstructor {
        function: FunctionOrdinal(543) => BUILTIN_FLOAT32_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(12),
        global_name: FLOAT32_ARRAY_NAME,
        debug: FLOAT32_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: FLOAT32_ARRAY_NAME,
    }
    Int32ArrayConstructor {
        function: FunctionOrdinal(544) => BUILTIN_INT32_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(13),
        global_name: INT32_ARRAY_NAME,
        debug: INT32_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: INT32_ARRAY_NAME,
    }
    Int16ArrayConstructor {
        function: FunctionOrdinal(545) => BUILTIN_INT16_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(14),
        global_name: INT16_ARRAY_NAME,
        debug: INT16_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: INT16_ARRAY_NAME,
    }
    Int8ArrayConstructor {
        function: FunctionOrdinal(546) => BUILTIN_INT8_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(15),
        global_name: INT8_ARRAY_NAME,
        debug: INT8_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: INT8_ARRAY_NAME,
    }
    Uint32ArrayConstructor {
        function: FunctionOrdinal(547) => BUILTIN_UINT32_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(16),
        global_name: UINT32_ARRAY_NAME,
        debug: UINT32_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: UINT32_ARRAY_NAME,
    }
    Uint16ArrayConstructor {
        function: FunctionOrdinal(548) => BUILTIN_UINT16_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(17),
        global_name: UINT16_ARRAY_NAME,
        debug: UINT16_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: UINT16_ARRAY_NAME,
    }
    Uint8ArrayConstructor {
        function: FunctionOrdinal(549) => BUILTIN_UINT8_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(18),
        global_name: UINT8_ARRAY_NAME,
        debug: UINT8_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: UINT8_ARRAY_NAME,
    }
    Uint8ClampedArrayConstructor {
        function: FunctionOrdinal(550) => BUILTIN_UINT8_CLAMPED_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(19),
        global_name: UINT8_CLAMPED_ARRAY_NAME,
        debug: UINT8_CLAMPED_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: UINT8_CLAMPED_ARRAY_NAME,
    }
    BigInt64ArrayConstructor {
        function: FunctionOrdinal(551) => BUILTIN_BIGINT64_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(20),
        global_name: BIGINT64_ARRAY_NAME,
        debug: BIGINT64_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: BIGINT64_ARRAY_NAME,
    }
    BigUint64ArrayConstructor {
        function: FunctionOrdinal(552) => BUILTIN_BIGUINT64_ARRAY_FUNCTION_ID,
        global: GlobalOrdinal(21),
        global_name: BIGUINT64_ARRAY_NAME,
        debug: BIGUINT64_ARRAY_NAME,
        flags: [CONSTRUCTABLE],
        installer: None,
        native: BIGUINT64_ARRAY_NAME,
    }
    BigIntConstructor {
        function: FunctionOrdinal(553) => BUILTIN_BIGINT_FUNCTION_ID,
        global: GlobalOrdinal(22),
        global_name: BIGINT_NAME,
        debug: BIGINT_NAME,
        flags: [CONSTRUCTABLE],
        installer: BigInt,
        native: BIGINT_NAME,
    }
    BigIntAsIntN {
        function: FunctionOrdinal(554) => BUILTIN_BIGINT_AS_INT_N_FUNCTION_ID,
        global_name: BIGINT_NAME,
        debug: "BigInt.asIntN",
        flags: [STATIC_METHOD],
        installer: None,
        native: "asIntN",
    }
    BigIntAsUintN {
        function: FunctionOrdinal(555) => BUILTIN_BIGINT_AS_UINT_N_FUNCTION_ID,
        global_name: BIGINT_NAME,
        debug: "BigInt.asUintN",
        flags: [STATIC_METHOD],
        installer: None,
        native: "asUintN",
    }
    BigIntPrototypeToString {
        function: FunctionOrdinal(556) => BUILTIN_BIGINT_PROTOTYPE_TO_STRING_FUNCTION_ID,
        global_name: BIGINT_NAME,
        debug: "BigInt.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    BigIntPrototypeToLocaleString {
        function: FunctionOrdinal(557) => BUILTIN_BIGINT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID,
        global_name: BIGINT_NAME,
        debug: "BigInt.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    BigIntPrototypeValueOf {
        function: FunctionOrdinal(558) => BUILTIN_BIGINT_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        global_name: BIGINT_NAME,
        debug: "BigInt.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    NumberConstructor {
        function: FunctionOrdinal(559) => BUILTIN_NUMBER_FUNCTION_ID,
        global: GlobalOrdinal(23),
        global_name: NUMBER_NAME,
        debug: NUMBER_NAME,
        flags: [CONSTRUCTABLE, BOXED_PRIMITIVE],
        installer: Number,
        native: NUMBER_NAME,
    }
    NumberIsInteger {
        function: FunctionOrdinal(560) => BUILTIN_NUMBER_IS_INTEGER_FUNCTION_ID,
        debug: "Number.isInteger",
        flags: [STATIC_METHOD],
        installer: None,
        native: "isInteger",
    }
    NumberIsSafeInteger {
        function: FunctionOrdinal(561) => "$builtin.Number.isSafeInteger",
        debug: "Number.isSafeInteger",
        flags: [STATIC_METHOD],
        installer: None,
        native: "isSafeInteger",
    }
    NumberIsFinite {
        function: FunctionOrdinal(562) => "$builtin.Number.isFinite",
        debug: "Number.isFinite",
        flags: [STATIC_METHOD],
        installer: None,
        native: "isFinite",
    }
    NumberIsNaN {
        function: FunctionOrdinal(563) => "$builtin.Number.isNaN",
        debug: "Number.isNaN",
        flags: [STATIC_METHOD],
        installer: None,
        native: "isNaN",
    }
    NumberPrototypeToExponential {
        function: FunctionOrdinal(564) => "$builtin.Number.prototype.toExponential",
        debug: "Number.prototype.toExponential",
        flags: [],
        installer: None,
        native: "toExponential",
    }
    NumberPrototypeToFixed {
        function: FunctionOrdinal(565) => "$builtin.Number.prototype.toFixed",
        debug: "Number.prototype.toFixed",
        flags: [],
        installer: None,
        native: "toFixed",
    }
    NumberPrototypeToPrecision {
        function: FunctionOrdinal(566) => "$builtin.Number.prototype.toPrecision",
        debug: "Number.prototype.toPrecision",
        flags: [],
        installer: None,
        native: "toPrecision",
    }
    NumberPrototypeToString {
        function: FunctionOrdinal(567) => "$builtin.Number.prototype.toString",
        debug: "Number.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    NumberPrototypeToLocaleString {
        function: FunctionOrdinal(568) => "$builtin.Number.prototype.toLocaleString",
        debug: "Number.prototype.toLocaleString",
        flags: [],
        installer: None,
        native: "toLocaleString",
    }
    NumberPrototypeValueOf {
        function: FunctionOrdinal(569) => "$builtin.Number.prototype.valueOf",
        debug: "Number.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    GlobalIsFinite {
        function: FunctionOrdinal(570) => "$builtin.isFinite",
        global: GlobalOrdinal(24),
        global_name: "isFinite",
        debug: "isFinite",
        flags: [],
        installer: None,
        native: "isFinite",
    }
    GlobalIsNaN {
        function: FunctionOrdinal(571) => "$builtin.isNaN",
        global: GlobalOrdinal(25),
        global_name: "isNaN",
        debug: "isNaN",
        flags: [],
        installer: None,
        native: "isNaN",
    }
    MathAbs {
        function: FunctionOrdinal(572) => "$builtin.Math.abs",
        debug: "Math.abs",
        flags: [STATIC_METHOD],
        installer: None,
        native: "abs",
    }
    MathAcos {
        function: FunctionOrdinal(573) => "$builtin.Math.acos",
        debug: "Math.acos",
        flags: [STATIC_METHOD],
        installer: None,
        native: "acos",
    }
    MathAcosh {
        function: FunctionOrdinal(574) => "$builtin.Math.acosh",
        debug: "Math.acosh",
        flags: [STATIC_METHOD],
        installer: None,
        native: "acosh",
    }
    MathAsin {
        function: FunctionOrdinal(575) => "$builtin.Math.asin",
        debug: "Math.asin",
        flags: [STATIC_METHOD],
        installer: None,
        native: "asin",
    }
    MathAsinh {
        function: FunctionOrdinal(576) => "$builtin.Math.asinh",
        debug: "Math.asinh",
        flags: [STATIC_METHOD],
        installer: None,
        native: "asinh",
    }
    MathAtan {
        function: FunctionOrdinal(577) => "$builtin.Math.atan",
        debug: "Math.atan",
        flags: [STATIC_METHOD],
        installer: None,
        native: "atan",
    }
    MathAtan2 {
        function: FunctionOrdinal(578) => "$builtin.Math.atan2",
        debug: "Math.atan2",
        flags: [STATIC_METHOD],
        installer: None,
        native: "atan2",
    }
    MathAtanh {
        function: FunctionOrdinal(579) => "$builtin.Math.atanh",
        debug: "Math.atanh",
        flags: [STATIC_METHOD],
        installer: None,
        native: "atanh",
    }
    MathCbrt {
        function: FunctionOrdinal(580) => "$builtin.Math.cbrt",
        debug: "Math.cbrt",
        flags: [STATIC_METHOD],
        installer: None,
        native: "cbrt",
    }
    MathCeil {
        function: FunctionOrdinal(581) => "$builtin.Math.ceil",
        debug: "Math.ceil",
        flags: [STATIC_METHOD],
        installer: None,
        native: "ceil",
    }
    MathClz32 {
        function: FunctionOrdinal(582) => "$builtin.Math.clz32",
        debug: "Math.clz32",
        flags: [STATIC_METHOD],
        installer: None,
        native: "clz32",
    }
    MathCos {
        function: FunctionOrdinal(583) => "$builtin.Math.cos",
        debug: "Math.cos",
        flags: [STATIC_METHOD],
        installer: None,
        native: "cos",
    }
    MathCosh {
        function: FunctionOrdinal(584) => "$builtin.Math.cosh",
        debug: "Math.cosh",
        flags: [STATIC_METHOD],
        installer: None,
        native: "cosh",
    }
    MathExp {
        function: FunctionOrdinal(585) => "$builtin.Math.exp",
        debug: "Math.exp",
        flags: [STATIC_METHOD],
        installer: None,
        native: "exp",
    }
    MathExpm1 {
        function: FunctionOrdinal(586) => "$builtin.Math.expm1",
        debug: "Math.expm1",
        flags: [STATIC_METHOD],
        installer: None,
        native: "expm1",
    }
    MathF16Round {
        function: FunctionOrdinal(587) => "$builtin.Math.f16round",
        debug: "Math.f16round",
        flags: [STATIC_METHOD],
        installer: None,
        native: "f16round",
    }
    MathFloor {
        function: FunctionOrdinal(588) => "$builtin.Math.floor",
        debug: "Math.floor",
        flags: [STATIC_METHOD],
        installer: None,
        native: "floor",
    }
    MathFround {
        function: FunctionOrdinal(589) => "$builtin.Math.fround",
        debug: "Math.fround",
        flags: [STATIC_METHOD],
        installer: None,
        native: "fround",
    }
    MathHypot {
        function: FunctionOrdinal(590) => "$builtin.Math.hypot",
        debug: "Math.hypot",
        flags: [STATIC_METHOD],
        installer: None,
        native: "hypot",
    }
    MathImul {
        function: FunctionOrdinal(591) => "$builtin.Math.imul",
        debug: "Math.imul",
        flags: [STATIC_METHOD],
        installer: None,
        native: "imul",
    }
    MathLog {
        function: FunctionOrdinal(592) => "$builtin.Math.log",
        debug: "Math.log",
        flags: [STATIC_METHOD],
        installer: None,
        native: "log",
    }
    MathLog10 {
        function: FunctionOrdinal(593) => "$builtin.Math.log10",
        debug: "Math.log10",
        flags: [STATIC_METHOD],
        installer: None,
        native: "log10",
    }
    MathLog1p {
        function: FunctionOrdinal(594) => "$builtin.Math.log1p",
        debug: "Math.log1p",
        flags: [STATIC_METHOD],
        installer: None,
        native: "log1p",
    }
    MathLog2 {
        function: FunctionOrdinal(595) => "$builtin.Math.log2",
        debug: "Math.log2",
        flags: [STATIC_METHOD],
        installer: None,
        native: "log2",
    }
    MathPow {
        function: FunctionOrdinal(596) => "$builtin.Math.pow",
        debug: "Math.pow",
        flags: [STATIC_METHOD],
        installer: None,
        native: "pow",
    }
    MathRandom {
        function: FunctionOrdinal(597) => "$builtin.Math.random",
        debug: "Math.random",
        flags: [STATIC_METHOD, RANDOM],
        installer: None,
        native: "random",
    }
    MathRound {
        function: FunctionOrdinal(598) => "$builtin.Math.round",
        debug: "Math.round",
        flags: [STATIC_METHOD],
        installer: None,
        native: "round",
    }
    MathSign {
        function: FunctionOrdinal(599) => "$builtin.Math.sign",
        debug: "Math.sign",
        flags: [STATIC_METHOD],
        installer: None,
        native: "sign",
    }
    MathSin {
        function: FunctionOrdinal(600) => "$builtin.Math.sin",
        debug: "Math.sin",
        flags: [STATIC_METHOD],
        installer: None,
        native: "sin",
    }
    MathSinh {
        function: FunctionOrdinal(601) => "$builtin.Math.sinh",
        debug: "Math.sinh",
        flags: [STATIC_METHOD],
        installer: None,
        native: "sinh",
    }
    MathSqrt {
        function: FunctionOrdinal(602) => "$builtin.Math.sqrt",
        debug: "Math.sqrt",
        flags: [STATIC_METHOD],
        installer: None,
        native: "sqrt",
    }
    MathSumPrecise {
        function: FunctionOrdinal(603) => "$builtin.Math.sumPrecise",
        debug: "Math.sumPrecise",
        flags: [STATIC_METHOD],
        installer: None,
        native: "sumPrecise",
    }
    MathTan {
        function: FunctionOrdinal(604) => "$builtin.Math.tan",
        debug: "Math.tan",
        flags: [STATIC_METHOD],
        installer: None,
        native: "tan",
    }
    MathTanh {
        function: FunctionOrdinal(605) => "$builtin.Math.tanh",
        debug: "Math.tanh",
        flags: [STATIC_METHOD],
        installer: None,
        native: "tanh",
    }
    MathTrunc {
        function: FunctionOrdinal(606) => "$builtin.Math.trunc",
        debug: "Math.trunc",
        flags: [STATIC_METHOD],
        installer: None,
        native: "trunc",
    }
    MathMin {
        function: FunctionOrdinal(607) => "$builtin.Math.min",
        debug: "Math.min",
        flags: [STATIC_METHOD],
        installer: None,
        native: "min",
    }
    MathMax {
        function: FunctionOrdinal(608) => "$builtin.Math.max",
        debug: "Math.max",
        flags: [STATIC_METHOD],
        installer: None,
        native: "max",
    }
    StringConstructor {
        function: FunctionOrdinal(609) => BUILTIN_STRING_FUNCTION_ID,
        global: GlobalOrdinal(26),
        global_name: STRING_NAME,
        debug: STRING_NAME,
        flags: [CONSTRUCTABLE, BOXED_PRIMITIVE],
        installer: String,
        native: STRING_NAME,
    }
    StringFromCharCode {
        function: FunctionOrdinal(610) => BUILTIN_STRING_FROM_CHAR_CODE_FUNCTION_ID,
        debug: "String.fromCharCode",
        flags: [],
        installer: None,
        native: "fromCharCode",
    }
    StringFromCodePoint {
        function: FunctionOrdinal(611) => BUILTIN_STRING_FROM_CODE_POINT_FUNCTION_ID,
        debug: "String.fromCodePoint",
        flags: [],
        installer: None,
        native: "fromCodePoint",
    }
    StringRaw {
        function: FunctionOrdinal(612) => BUILTIN_STRING_RAW_FUNCTION_ID,
        debug: "String.raw",
        flags: [],
        installer: None,
        native: "raw",
    }
    StringPrototypeToString {
        function: FunctionOrdinal(613) => BUILTIN_STRING_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "String.prototype.toString",
        flags: [],
        installer: None,
        string: "toString",
        native: "toString",
    }
    StringPrototypeValueOf {
        function: FunctionOrdinal(614) => BUILTIN_STRING_PROTOTYPE_VALUE_OF_FUNCTION_ID,
        debug: "String.prototype.valueOf",
        flags: [],
        installer: None,
        string: "valueOf",
        native: "valueOf",
    }
    StringPrototypeCharAt {
        function: FunctionOrdinal(615) => BUILTIN_STRING_PROTOTYPE_CHAR_AT_FUNCTION_ID,
        debug: "String.prototype.charAt",
        flags: [],
        installer: None,
        string: "charAt",
        native: "charAt",
    }
    StringPrototypeConcat {
        function: FunctionOrdinal(616) => BUILTIN_STRING_PROTOTYPE_CONCAT_FUNCTION_ID,
        debug: "String.prototype.concat",
        flags: [],
        installer: None,
        string: "concat",
        native: "concat",
    }
    StringPrototypeCharCodeAt {
        function: FunctionOrdinal(617) => BUILTIN_STRING_PROTOTYPE_CHAR_CODE_AT_FUNCTION_ID,
        debug: "String.prototype.charCodeAt",
        flags: [],
        installer: None,
        string: "charCodeAt",
        native: "charCodeAt",
    }
    StringPrototypeCodePointAt {
        function: FunctionOrdinal(618) => BUILTIN_STRING_PROTOTYPE_CODE_POINT_AT_FUNCTION_ID,
        debug: "String.prototype.codePointAt",
        flags: [],
        installer: None,
        string: "codePointAt",
        native: "codePointAt",
    }
    StringPrototypeAt {
        function: FunctionOrdinal(619) => BUILTIN_STRING_PROTOTYPE_AT_FUNCTION_ID,
        debug: "String.prototype.at",
        flags: [],
        installer: None,
        string: "at",
        native: "at",
    }
    StringPrototypeAnchor {
        function: FunctionOrdinal(620) => BUILTIN_STRING_PROTOTYPE_ANCHOR_FUNCTION_ID,
        debug: "String.prototype.anchor",
        flags: [],
        installer: None,
        html: "anchor",
        string: "anchor",
        native: "anchor",
    }
    StringPrototypeBig {
        function: FunctionOrdinal(621) => BUILTIN_STRING_PROTOTYPE_BIG_FUNCTION_ID,
        debug: "String.prototype.big",
        flags: [],
        installer: None,
        html: "big",
        string: "big",
        native: "big",
    }
    StringPrototypeBlink {
        function: FunctionOrdinal(622) => BUILTIN_STRING_PROTOTYPE_BLINK_FUNCTION_ID,
        debug: "String.prototype.blink",
        flags: [],
        installer: None,
        html: "blink",
        string: "blink",
        native: "blink",
    }
    StringPrototypeBold {
        function: FunctionOrdinal(623) => BUILTIN_STRING_PROTOTYPE_BOLD_FUNCTION_ID,
        debug: "String.prototype.bold",
        flags: [],
        installer: None,
        html: "bold",
        string: "bold",
        native: "bold",
    }
    StringPrototypeFixed {
        function: FunctionOrdinal(624) => BUILTIN_STRING_PROTOTYPE_FIXED_FUNCTION_ID,
        debug: "String.prototype.fixed",
        flags: [],
        installer: None,
        html: "fixed",
        string: "fixed",
        native: "fixed",
    }
    StringPrototypeFontcolor {
        function: FunctionOrdinal(625) => BUILTIN_STRING_PROTOTYPE_FONTCOLOR_FUNCTION_ID,
        debug: "String.prototype.fontcolor",
        flags: [],
        installer: None,
        html: "fontcolor",
        string: "fontcolor",
        native: "fontcolor",
    }
    StringPrototypeFontsize {
        function: FunctionOrdinal(626) => BUILTIN_STRING_PROTOTYPE_FONTSIZE_FUNCTION_ID,
        debug: "String.prototype.fontsize",
        flags: [],
        installer: None,
        html: "fontsize",
        string: "fontsize",
        native: "fontsize",
    }
    StringPrototypeItalics {
        function: FunctionOrdinal(627) => BUILTIN_STRING_PROTOTYPE_ITALICS_FUNCTION_ID,
        debug: "String.prototype.italics",
        flags: [],
        installer: None,
        html: "italics",
        string: "italics",
        native: "italics",
    }
    StringPrototypeLink {
        function: FunctionOrdinal(628) => BUILTIN_STRING_PROTOTYPE_LINK_FUNCTION_ID,
        debug: "String.prototype.link",
        flags: [],
        installer: None,
        html: "link",
        string: "link",
        native: "link",
    }
    StringPrototypeSmall {
        function: FunctionOrdinal(629) => BUILTIN_STRING_PROTOTYPE_SMALL_FUNCTION_ID,
        debug: "String.prototype.small",
        flags: [],
        installer: None,
        html: "small",
        string: "small",
        native: "small",
    }
    StringPrototypeStrike {
        function: FunctionOrdinal(630) => BUILTIN_STRING_PROTOTYPE_STRIKE_FUNCTION_ID,
        debug: "String.prototype.strike",
        flags: [],
        installer: None,
        html: "strike",
        string: "strike",
        native: "strike",
    }
    StringPrototypeSub {
        function: FunctionOrdinal(631) => BUILTIN_STRING_PROTOTYPE_SUB_FUNCTION_ID,
        debug: "String.prototype.sub",
        flags: [],
        installer: None,
        html: "sub",
        string: "sub",
        native: "sub",
    }
    StringPrototypeSubstr {
        function: FunctionOrdinal(632) => BUILTIN_STRING_PROTOTYPE_SUBSTR_FUNCTION_ID,
        debug: "String.prototype.substr",
        flags: [],
        installer: None,
        string: "substr",
        native: "substr",
    }
    StringPrototypeSubstring {
        function: FunctionOrdinal(633) => BUILTIN_STRING_PROTOTYPE_SUBSTRING_FUNCTION_ID,
        debug: "String.prototype.substring",
        flags: [],
        installer: None,
        string: "substring",
        native: "substring",
    }
    StringPrototypeSup {
        function: FunctionOrdinal(634) => BUILTIN_STRING_PROTOTYPE_SUP_FUNCTION_ID,
        debug: "String.prototype.sup",
        flags: [],
        installer: None,
        html: "sup",
        string: "sup",
        native: "sup",
    }
    StringPrototypeMatch {
        function: FunctionOrdinal(635) => BUILTIN_STRING_PROTOTYPE_MATCH_FUNCTION_ID,
        debug: "String.prototype.match",
        flags: [],
        installer: None,
        string: "match",
        native: "match",
    }
    StringPrototypeMatchAll {
        function: FunctionOrdinal(636) => BUILTIN_STRING_PROTOTYPE_MATCH_ALL_FUNCTION_ID,
        debug: "String.prototype.matchAll",
        flags: [],
        installer: None,
        string: "matchAll",
        native: "matchAll",
    }
    StringPrototypeReplace {
        function: FunctionOrdinal(637) => BUILTIN_STRING_PROTOTYPE_REPLACE_FUNCTION_ID,
        debug: "String.prototype.replace",
        flags: [],
        installer: None,
        string: "replace",
        native: "replace",
    }
    StringPrototypeReplaceAll {
        function: FunctionOrdinal(638) => BUILTIN_STRING_PROTOTYPE_REPLACE_ALL_FUNCTION_ID,
        debug: "String.prototype.replaceAll",
        flags: [],
        installer: None,
        string: "replaceAll",
        native: "replaceAll",
    }
    StringPrototypeSearch {
        function: FunctionOrdinal(639) => BUILTIN_STRING_PROTOTYPE_SEARCH_FUNCTION_ID,
        debug: "String.prototype.search",
        flags: [],
        installer: None,
        string: "search",
        native: "search",
    }
    StringPrototypeIndexOf {
        function: FunctionOrdinal(640) => BUILTIN_STRING_PROTOTYPE_INDEX_OF_FUNCTION_ID,
        debug: "String.prototype.indexOf",
        flags: [],
        installer: None,
        string: "indexOf",
        native: "indexOf",
    }
    StringPrototypeLastIndexOf {
        function: FunctionOrdinal(641) => BUILTIN_STRING_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID,
        debug: "String.prototype.lastIndexOf",
        flags: [],
        installer: None,
        string: "lastIndexOf",
        native: "lastIndexOf",
    }
    StringPrototypeSlice {
        function: FunctionOrdinal(642) => BUILTIN_STRING_PROTOTYPE_SLICE_FUNCTION_ID,
        debug: "String.prototype.slice",
        flags: [],
        installer: None,
        string: "slice",
        native: "slice",
    }
    StringPrototypeSplit {
        function: FunctionOrdinal(643) => BUILTIN_STRING_PROTOTYPE_SPLIT_FUNCTION_ID,
        debug: "String.prototype.split",
        flags: [],
        installer: None,
        string: "split",
        native: "split",
    }
    StringPrototypePadStart {
        function: FunctionOrdinal(644) => BUILTIN_STRING_PROTOTYPE_PAD_START_FUNCTION_ID,
        debug: "String.prototype.padStart",
        flags: [],
        installer: None,
        string: "padStart",
        native: "padStart",
    }
    StringPrototypePadEnd {
        function: FunctionOrdinal(645) => BUILTIN_STRING_PROTOTYPE_PAD_END_FUNCTION_ID,
        debug: "String.prototype.padEnd",
        flags: [],
        installer: None,
        string: "padEnd",
        native: "padEnd",
    }
    StringPrototypeRepeat {
        function: FunctionOrdinal(646) => BUILTIN_STRING_PROTOTYPE_REPEAT_FUNCTION_ID,
        debug: "String.prototype.repeat",
        flags: [],
        installer: None,
        string: "repeat",
        native: "repeat",
    }
    StringPrototypeEndsWith {
        function: FunctionOrdinal(647) => BUILTIN_STRING_PROTOTYPE_ENDS_WITH_FUNCTION_ID,
        debug: "String.prototype.endsWith",
        flags: [],
        installer: None,
        string: "endsWith",
        native: "endsWith",
    }
    StringPrototypeIncludes {
        function: FunctionOrdinal(648) => BUILTIN_STRING_PROTOTYPE_INCLUDES_FUNCTION_ID,
        debug: "String.prototype.includes",
        flags: [],
        installer: None,
        string: "includes",
        native: "includes",
    }
    StringPrototypeStartsWith {
        function: FunctionOrdinal(649) => BUILTIN_STRING_PROTOTYPE_STARTS_WITH_FUNCTION_ID,
        debug: "String.prototype.startsWith",
        flags: [],
        installer: None,
        string: "startsWith",
        native: "startsWith",
    }
    StringPrototypeNormalize {
        function: FunctionOrdinal(650) => BUILTIN_STRING_PROTOTYPE_NORMALIZE_FUNCTION_ID,
        debug: "String.prototype.normalize",
        flags: [],
        installer: None,
        string: "normalize",
        native: "normalize",
    }
    StringPrototypeLocaleCompare {
        function: FunctionOrdinal(651) => BUILTIN_STRING_PROTOTYPE_LOCALE_COMPARE_FUNCTION_ID,
        debug: "String.prototype.localeCompare",
        flags: [],
        installer: None,
        string: "localeCompare",
        native: "localeCompare",
    }
    StringPrototypeIterator {
        function: FunctionOrdinal(652) => BUILTIN_STRING_PROTOTYPE_ITERATOR_FUNCTION_ID,
        debug: "String.prototype [Symbol.iterator]",
        flags: [],
        installer: None,
        string: "[Symbol.iterator]",
        native: "[Symbol.iterator]",
    }
    StringPrototypeToLocaleLowerCase {
        function: FunctionOrdinal(653) => BUILTIN_STRING_PROTOTYPE_TO_LOCALE_LOWER_CASE_FUNCTION_ID,
        debug: "String.prototype.toLocaleLowerCase",
        flags: [],
        installer: None,
        string: "toLocaleLowerCase",
        native: "toLocaleLowerCase",
    }
    StringPrototypeToLocaleUpperCase {
        function: FunctionOrdinal(654) => BUILTIN_STRING_PROTOTYPE_TO_LOCALE_UPPER_CASE_FUNCTION_ID,
        debug: "String.prototype.toLocaleUpperCase",
        flags: [],
        installer: None,
        string: "toLocaleUpperCase",
        native: "toLocaleUpperCase",
    }
    StringPrototypeToLowerCase {
        function: FunctionOrdinal(655) => BUILTIN_STRING_PROTOTYPE_TO_LOWER_CASE_FUNCTION_ID,
        debug: "String.prototype.toLowerCase",
        flags: [],
        installer: None,
        string: "toLowerCase",
        native: "toLowerCase",
    }
    StringPrototypeToUpperCase {
        function: FunctionOrdinal(656) => BUILTIN_STRING_PROTOTYPE_TO_UPPER_CASE_FUNCTION_ID,
        debug: "String.prototype.toUpperCase",
        flags: [],
        installer: None,
        string: "toUpperCase",
        native: "toUpperCase",
    }
    StringPrototypeTrim {
        function: FunctionOrdinal(657) => BUILTIN_STRING_PROTOTYPE_TRIM_FUNCTION_ID,
        debug: "String.prototype.trim",
        flags: [],
        installer: None,
        string: "trim",
        native: "trim",
    }
    StringPrototypeTrimStart {
        function: FunctionOrdinal(658) => BUILTIN_STRING_PROTOTYPE_TRIM_START_FUNCTION_ID,
        debug: "String.prototype.trimStart",
        flags: [],
        installer: None,
        string: "trimStart",
        native: "trimStart",
    }
    StringPrototypeTrimEnd {
        function: FunctionOrdinal(659) => BUILTIN_STRING_PROTOTYPE_TRIM_END_FUNCTION_ID,
        debug: "String.prototype.trimEnd",
        flags: [],
        installer: None,
        string: "trimEnd",
        native: "trimEnd",
    }
    StringPrototypeIsWellFormed {
        function: FunctionOrdinal(660) => BUILTIN_STRING_PROTOTYPE_IS_WELL_FORMED_FUNCTION_ID,
        debug: "String.prototype.isWellFormed",
        flags: [],
        installer: None,
        string: "isWellFormed",
        native: "isWellFormed",
    }
    StringPrototypeToWellFormed {
        function: FunctionOrdinal(661) => BUILTIN_STRING_PROTOTYPE_TO_WELL_FORMED_FUNCTION_ID,
        debug: "String.prototype.toWellFormed",
        flags: [],
        installer: None,
        string: "toWellFormed",
        native: "toWellFormed",
    }
    BooleanConstructor {
        function: FunctionOrdinal(662) => BUILTIN_BOOLEAN_FUNCTION_ID,
        global: GlobalOrdinal(27),
        global_name: BOOLEAN_NAME,
        debug: BOOLEAN_NAME,
        flags: [CONSTRUCTABLE, BOXED_PRIMITIVE],
        installer: Boolean,
        native: BOOLEAN_NAME,
    }
    BooleanPrototypeToString {
        function: FunctionOrdinal(663) => "$builtin.Boolean.prototype.toString",
        debug: "Boolean.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    BooleanPrototypeValueOf {
        function: FunctionOrdinal(664) => "$builtin.Boolean.prototype.valueOf",
        debug: "Boolean.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    PromiseConstructor {
        function: FunctionOrdinal(665) => BUILTIN_PROMISE_FUNCTION_ID,
        global: GlobalOrdinal(28),
        global_name: PROMISE_NAME,
        debug: PROMISE_NAME,
        flags: [CONSTRUCTABLE],
        installer: Promise,
        native: PROMISE_NAME,
    }
    PromisePrototypeThen {
        function: FunctionOrdinal(666) => BUILTIN_PROMISE_PROTOTYPE_THEN_FUNCTION_ID,
        debug: "Promise.prototype.then",
        flags: [],
        installer: None,
        native: "then",
    }
    PromisePrototypeCatch {
        function: FunctionOrdinal(667) => BUILTIN_PROMISE_PROTOTYPE_CATCH_FUNCTION_ID,
        debug: "Promise.prototype.catch",
        flags: [],
        installer: None,
        native: "catch",
    }
    PromisePrototypeFinally {
        function: FunctionOrdinal(668) => BUILTIN_PROMISE_PROTOTYPE_FINALLY_FUNCTION_ID,
        debug: "Promise.prototype.finally",
        flags: [],
        installer: None,
        native: "finally",
    }
    PromiseThenFinally {
        function: FunctionOrdinal(669) => BUILTIN_PROMISE_THEN_FINALLY_FUNCTION_ID,
        debug: "Promise Then Finally Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseCatchFinally {
        function: FunctionOrdinal(670) => BUILTIN_PROMISE_CATCH_FINALLY_FUNCTION_ID,
        debug: "Promise Catch Finally Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseValueThunk {
        function: FunctionOrdinal(671) => BUILTIN_PROMISE_VALUE_THUNK_FUNCTION_ID,
        debug: "Promise Value Thunk Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseThrower {
        function: FunctionOrdinal(672) => BUILTIN_PROMISE_THROWER_FUNCTION_ID,
        debug: "Promise Thrower Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseSpeciesGetter {
        function: FunctionOrdinal(673) => BUILTIN_PROMISE_SPECIES_GETTER_FUNCTION_ID,
        debug: "get Promise [Symbol.species]",
        flags: [],
        installer: None,
        native: "get [Symbol.species]",
    }
    PromiseResolve {
        function: FunctionOrdinal(674) => BUILTIN_PROMISE_STATIC_RESOLVE_FUNCTION_ID,
        debug: "Promise.resolve",
        flags: [],
        installer: None,
        native: "resolve",
    }
    PromiseWithResolvers {
        function: FunctionOrdinal(675) => BUILTIN_PROMISE_STATIC_WITH_RESOLVERS_FUNCTION_ID,
        debug: "Promise.withResolvers",
        flags: [],
        installer: None,
        native: "withResolvers",
    }
    PromiseTry {
        function: FunctionOrdinal(676) => BUILTIN_PROMISE_STATIC_TRY_FUNCTION_ID,
        debug: "Promise.try",
        flags: [],
        installer: None,
        native: "try",
    }
    PromiseReject {
        function: FunctionOrdinal(677) => BUILTIN_PROMISE_STATIC_REJECT_FUNCTION_ID,
        debug: "Promise.reject",
        flags: [],
        installer: None,
        native: "reject",
    }
    PromiseAll {
        function: FunctionOrdinal(678) => BUILTIN_PROMISE_STATIC_ALL_FUNCTION_ID,
        debug: "Promise.all",
        flags: [],
        installer: None,
        native: "all",
    }
    PromiseAllSettled {
        function: FunctionOrdinal(679) => BUILTIN_PROMISE_STATIC_ALL_SETTLED_FUNCTION_ID,
        debug: "Promise.allSettled",
        flags: [],
        installer: None,
        native: "allSettled",
    }
    PromiseAllKeyed {
        function: FunctionOrdinal(680) => BUILTIN_PROMISE_STATIC_ALL_KEYED_FUNCTION_ID,
        debug: "Promise.allKeyed",
        flags: [],
        installer: None,
        native: "allKeyed",
    }
    PromiseAllSettledKeyed {
        function: FunctionOrdinal(681) => BUILTIN_PROMISE_STATIC_ALL_SETTLED_KEYED_FUNCTION_ID,
        debug: "Promise.allSettledKeyed",
        flags: [],
        installer: None,
        native: "allSettledKeyed",
    }
    PromiseAny {
        function: FunctionOrdinal(682) => BUILTIN_PROMISE_STATIC_ANY_FUNCTION_ID,
        debug: "Promise.any",
        flags: [],
        installer: None,
        native: "any",
    }
    PromiseRace {
        function: FunctionOrdinal(683) => BUILTIN_PROMISE_STATIC_RACE_FUNCTION_ID,
        debug: "Promise.race",
        flags: [],
        installer: None,
        native: "race",
    }
    PromiseAllResolveElement {
        function: FunctionOrdinal(684) => BUILTIN_PROMISE_ALL_RESOLVE_ELEMENT_FUNCTION_ID,
        debug: "Promise.all Resolve Element Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseAllSettledResolveElement {
        function: FunctionOrdinal(685) => BUILTIN_PROMISE_ALL_SETTLED_RESOLVE_ELEMENT_FUNCTION_ID,
        debug: "Promise.allSettled Resolve Element Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseAllSettledRejectElement {
        function: FunctionOrdinal(686) => BUILTIN_PROMISE_ALL_SETTLED_REJECT_ELEMENT_FUNCTION_ID,
        debug: "Promise.allSettled Reject Element Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseAnyRejectElement {
        function: FunctionOrdinal(687) => BUILTIN_PROMISE_ANY_REJECT_ELEMENT_FUNCTION_ID,
        debug: "Promise.any Reject Element Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseAllKeyedResolveElement {
        function: FunctionOrdinal(688) => BUILTIN_PROMISE_ALL_KEYED_RESOLVE_ELEMENT_FUNCTION_ID,
        debug: "Promise.allKeyed Resolve Element Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseAllSettledKeyedResolveElement {
        function: FunctionOrdinal(689) => BUILTIN_PROMISE_ALL_SETTLED_KEYED_RESOLVE_ELEMENT_FUNCTION_ID,
        debug: "Promise.allSettledKeyed Resolve Element Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseAllSettledKeyedRejectElement {
        function: FunctionOrdinal(690) => BUILTIN_PROMISE_ALL_SETTLED_KEYED_REJECT_ELEMENT_FUNCTION_ID,
        debug: "Promise.allSettledKeyed Reject Element Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseCapabilityExecutor {
        function: FunctionOrdinal(691) => BUILTIN_PROMISE_CAPABILITY_EXECUTOR_FUNCTION_ID,
        debug: "Promise Capability Executor",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseResolveFunction {
        function: FunctionOrdinal(692) => BUILTIN_PROMISE_RESOLVE_FUNCTION_ID,
        debug: "Promise Resolve Function",
        flags: [],
        installer: None,
        native: "",
    }
    PromiseRejectFunction {
        function: FunctionOrdinal(693) => BUILTIN_PROMISE_REJECT_FUNCTION_ID,
        debug: "Promise Reject Function",
        flags: [],
        installer: None,
        native: "",
    }
    MapConstructor {
        function: FunctionOrdinal(694) => BUILTIN_MAP_FUNCTION_ID,
        global: GlobalOrdinal(29),
        global_name: MAP_NAME,
        debug: MAP_NAME,
        flags: [CONSTRUCTABLE],
        installer: Map,
        native: MAP_NAME,
    }
    MapSpeciesGetter {
        function: FunctionOrdinal(695) => BUILTIN_MAP_SPECIES_GETTER_FUNCTION_ID,
        debug: "get Map [Symbol.species]",
        flags: [],
        installer: None,
        native: "get [Symbol.species]",
    }
    MapGroupBy {
        function: FunctionOrdinal(696) => BUILTIN_MAP_GROUP_BY_FUNCTION_ID,
        debug: "Map.groupBy",
        flags: [],
        installer: None,
        native: "groupBy",
    }
    MapPrototypeClear {
        function: FunctionOrdinal(697) => BUILTIN_MAP_PROTOTYPE_CLEAR_FUNCTION_ID,
        debug: "Map.prototype.clear",
        flags: [],
        installer: None,
        native: "clear",
    }
    MapPrototypeDelete {
        function: FunctionOrdinal(698) => BUILTIN_MAP_PROTOTYPE_DELETE_FUNCTION_ID,
        debug: "Map.prototype.delete",
        flags: [],
        installer: None,
        native: "delete",
    }
    MapPrototypeForEach {
        function: FunctionOrdinal(699) => BUILTIN_MAP_PROTOTYPE_FOR_EACH_FUNCTION_ID,
        debug: "Map.prototype.forEach",
        flags: [],
        installer: None,
        native: "forEach",
    }
    MapPrototypeKeys {
        function: FunctionOrdinal(700) => BUILTIN_MAP_PROTOTYPE_KEYS_FUNCTION_ID,
        debug: "Map.prototype.keys",
        flags: [],
        installer: None,
        native: "keys",
    }
    MapPrototypeValues {
        function: FunctionOrdinal(701) => BUILTIN_MAP_PROTOTYPE_VALUES_FUNCTION_ID,
        debug: "Map.prototype.values",
        flags: [],
        installer: None,
        native: "values",
    }
    MapPrototypeEntries {
        function: FunctionOrdinal(702) => BUILTIN_MAP_PROTOTYPE_ENTRIES_FUNCTION_ID,
        debug: "Map.prototype.entries",
        flags: [],
        installer: None,
        native: "entries",
    }
    MapIteratorNext {
        function: FunctionOrdinal(703) => BUILTIN_MAP_ITERATOR_NEXT_FUNCTION_ID,
        debug: "Map Iterator.prototype.next",
        flags: [],
        installer: None,
        native: "next",
    }
    MapPrototypeGet {
        function: FunctionOrdinal(704) => BUILTIN_MAP_PROTOTYPE_GET_FUNCTION_ID,
        debug: "Map.prototype.get",
        flags: [],
        installer: None,
        native: "get",
    }
    MapPrototypeGetOrInsert {
        function: FunctionOrdinal(705) => BUILTIN_MAP_PROTOTYPE_GET_OR_INSERT_FUNCTION_ID,
        debug: "Map.prototype.getOrInsert",
        flags: [],
        installer: None,
        native: "getOrInsert",
    }
    MapPrototypeGetOrInsertComputed {
        function: FunctionOrdinal(706) => BUILTIN_MAP_PROTOTYPE_GET_OR_INSERT_COMPUTED_FUNCTION_ID,
        debug: "Map.prototype.getOrInsertComputed",
        flags: [],
        installer: None,
        native: "getOrInsertComputed",
    }
    MapPrototypeHas {
        function: FunctionOrdinal(707) => BUILTIN_MAP_PROTOTYPE_HAS_FUNCTION_ID,
        debug: "Map.prototype.has",
        flags: [],
        installer: None,
        native: "has",
    }
    MapPrototypeSet {
        function: FunctionOrdinal(708) => BUILTIN_MAP_PROTOTYPE_SET_FUNCTION_ID,
        debug: "Map.prototype.set",
        flags: [],
        installer: None,
        native: "set",
    }
    MapPrototypeSizeGetter {
        function: FunctionOrdinal(709) => BUILTIN_MAP_PROTOTYPE_SIZE_GETTER_FUNCTION_ID,
        debug: "get Map.prototype.size",
        flags: [],
        installer: None,
        native: "get size",
    }
    WeakMapConstructor {
        function: FunctionOrdinal(710) => BUILTIN_WEAK_MAP_FUNCTION_ID,
        global: GlobalOrdinal(30),
        global_name: WEAK_MAP_NAME,
        debug: WEAK_MAP_NAME,
        flags: [CONSTRUCTABLE],
        installer: WeakMap,
        native: WEAK_MAP_NAME,
    }
    WeakMapPrototypeDelete {
        function: FunctionOrdinal(711) => BUILTIN_WEAK_MAP_PROTOTYPE_DELETE_FUNCTION_ID,
        debug: "WeakMap.prototype.delete",
        flags: [],
        installer: None,
        native: "delete",
    }
    WeakMapPrototypeGet {
        function: FunctionOrdinal(712) => BUILTIN_WEAK_MAP_PROTOTYPE_GET_FUNCTION_ID,
        debug: "WeakMap.prototype.get",
        flags: [],
        installer: None,
        native: "get",
    }
    WeakMapPrototypeGetOrInsert {
        function: FunctionOrdinal(713) => BUILTIN_WEAK_MAP_PROTOTYPE_GET_OR_INSERT_FUNCTION_ID,
        debug: "WeakMap.prototype.getOrInsert",
        flags: [],
        installer: None,
        native: "getOrInsert",
    }
    WeakMapPrototypeGetOrInsertComputed {
        function: FunctionOrdinal(714) => BUILTIN_WEAK_MAP_PROTOTYPE_GET_OR_INSERT_COMPUTED_FUNCTION_ID,
        debug: "WeakMap.prototype.getOrInsertComputed",
        flags: [],
        installer: None,
        native: "getOrInsertComputed",
    }
    WeakMapPrototypeHas {
        function: FunctionOrdinal(715) => BUILTIN_WEAK_MAP_PROTOTYPE_HAS_FUNCTION_ID,
        debug: "WeakMap.prototype.has",
        flags: [],
        installer: None,
        native: "has",
    }
    WeakMapPrototypeSet {
        function: FunctionOrdinal(716) => BUILTIN_WEAK_MAP_PROTOTYPE_SET_FUNCTION_ID,
        debug: "WeakMap.prototype.set",
        flags: [],
        installer: None,
        native: "set",
    }
    WeakSetConstructor {
        function: FunctionOrdinal(717) => BUILTIN_WEAK_SET_FUNCTION_ID,
        global: GlobalOrdinal(31),
        global_name: WEAK_SET_NAME,
        debug: WEAK_SET_NAME,
        flags: [CONSTRUCTABLE],
        installer: WeakSet,
        native: WEAK_SET_NAME,
    }
    WeakSetPrototypeAdd {
        function: FunctionOrdinal(718) => BUILTIN_WEAK_SET_PROTOTYPE_ADD_FUNCTION_ID,
        debug: "WeakSet.prototype.add",
        flags: [],
        installer: None,
        native: "add",
    }
    WeakSetPrototypeDelete {
        function: FunctionOrdinal(719) => BUILTIN_WEAK_SET_PROTOTYPE_DELETE_FUNCTION_ID,
        debug: "WeakSet.prototype.delete",
        flags: [],
        installer: None,
        native: "delete",
    }
    WeakSetPrototypeHas {
        function: FunctionOrdinal(720) => BUILTIN_WEAK_SET_PROTOTYPE_HAS_FUNCTION_ID,
        debug: "WeakSet.prototype.has",
        flags: [],
        installer: None,
        native: "has",
    }
    WeakRefConstructor {
        function: FunctionOrdinal(721) => BUILTIN_WEAK_REF_FUNCTION_ID,
        global: GlobalOrdinal(32),
        global_name: WEAK_REF_NAME,
        debug: WEAK_REF_NAME,
        flags: [CONSTRUCTABLE],
        installer: WeakRef,
        native: WEAK_REF_NAME,
    }
    WeakRefPrototypeDeref {
        function: FunctionOrdinal(722) => BUILTIN_WEAK_REF_PROTOTYPE_DEREF_FUNCTION_ID,
        debug: "WeakRef.prototype.deref",
        flags: [],
        installer: None,
        native: "deref",
    }
    FinalizationRegistryConstructor {
        function: FunctionOrdinal(723) => BUILTIN_FINALIZATION_REGISTRY_FUNCTION_ID,
        global: GlobalOrdinal(33),
        global_name: FINALIZATION_REGISTRY_NAME,
        debug: FINALIZATION_REGISTRY_NAME,
        flags: [CONSTRUCTABLE],
        installer: FinalizationRegistry,
        native: FINALIZATION_REGISTRY_NAME,
    }
    FinalizationRegistryPrototypeRegister {
        function: FunctionOrdinal(724) => BUILTIN_FINALIZATION_REGISTRY_PROTOTYPE_REGISTER_FUNCTION_ID,
        debug: "FinalizationRegistry.prototype.register",
        flags: [],
        installer: None,
        native: "register",
    }
    FinalizationRegistryPrototypeUnregister {
        function: FunctionOrdinal(725) => BUILTIN_FINALIZATION_REGISTRY_PROTOTYPE_UNREGISTER_FUNCTION_ID,
        debug: "FinalizationRegistry.prototype.unregister",
        flags: [],
        installer: None,
        native: "unregister",
    }
    AsyncDisposableStackConstructor {
        function: FunctionOrdinal(726) => BUILTIN_ASYNC_DISPOSABLE_STACK_FUNCTION_ID,
        global: GlobalOrdinal(34),
        global_name: ASYNC_DISPOSABLE_STACK_NAME,
        debug: ASYNC_DISPOSABLE_STACK_NAME,
        flags: [CONSTRUCTABLE],
        installer: AsyncDisposableStack,
        native: ASYNC_DISPOSABLE_STACK_NAME,
    }
    AsyncDisposableStackPrototypeUse {
        function: FunctionOrdinal(727) => BUILTIN_ASYNC_DISPOSABLE_STACK_PROTOTYPE_USE_FUNCTION_ID,
        debug: "AsyncDisposableStack.prototype.use",
        flags: [],
        installer: None,
        native: "use",
    }
    AsyncDisposableStackPrototypeAdopt {
        function: FunctionOrdinal(728) => BUILTIN_ASYNC_DISPOSABLE_STACK_PROTOTYPE_ADOPT_FUNCTION_ID,
        debug: "AsyncDisposableStack.prototype.adopt",
        flags: [],
        installer: None,
        native: "adopt",
    }
    AsyncDisposableStackPrototypeDefer {
        function: FunctionOrdinal(729) => BUILTIN_ASYNC_DISPOSABLE_STACK_PROTOTYPE_DEFER_FUNCTION_ID,
        debug: "AsyncDisposableStack.prototype.defer",
        flags: [],
        installer: None,
        native: "defer",
    }
    AsyncDisposableStackPrototypeMove {
        function: FunctionOrdinal(730) => BUILTIN_ASYNC_DISPOSABLE_STACK_PROTOTYPE_MOVE_FUNCTION_ID,
        debug: "AsyncDisposableStack.prototype.move",
        flags: [],
        installer: None,
        native: "move",
    }
    AsyncDisposableStackPrototypeDisposeAsync {
        function: FunctionOrdinal(731) => BUILTIN_ASYNC_DISPOSABLE_STACK_PROTOTYPE_DISPOSE_ASYNC_FUNCTION_ID,
        debug: "AsyncDisposableStack.prototype.disposeAsync",
        flags: [],
        installer: None,
        native: "disposeAsync",
    }
    AsyncDisposableStackPrototypeDisposedGetter {
        function: FunctionOrdinal(732) => BUILTIN_ASYNC_DISPOSABLE_STACK_PROTOTYPE_DISPOSED_GETTER_FUNCTION_ID,
        debug: "get AsyncDisposableStack.prototype.disposed",
        flags: [],
        installer: None,
        native: "get disposed",
    }
    AsyncDisposableStackDisposeAsyncFulfilled {
        function: FunctionOrdinal(733) => BUILTIN_ASYNC_DISPOSABLE_STACK_DISPOSE_ASYNC_FULFILLED_FUNCTION_ID,
        debug: "AsyncDisposableStack disposeAsync Fulfilled Function",
        flags: [],
        installer: None,
        native: "",
    }
    AsyncDisposableStackDisposeAsyncRejected {
        function: FunctionOrdinal(734) => BUILTIN_ASYNC_DISPOSABLE_STACK_DISPOSE_ASYNC_REJECTED_FUNCTION_ID,
        debug: "AsyncDisposableStack disposeAsync Rejected Function",
        flags: [],
        installer: None,
        native: "",
    }
    SetConstructor {
        function: FunctionOrdinal(735) => BUILTIN_SET_FUNCTION_ID,
        global: GlobalOrdinal(35),
        global_name: SET_NAME,
        debug: SET_NAME,
        flags: [CONSTRUCTABLE],
        installer: Set,
        native: SET_NAME,
    }
    SetSpeciesGetter {
        function: FunctionOrdinal(736) => BUILTIN_SET_SPECIES_GETTER_FUNCTION_ID,
        debug: "get Set [Symbol.species]",
        flags: [],
        installer: None,
        native: "get [Symbol.species]",
    }
    SetPrototypeAdd {
        function: FunctionOrdinal(737) => BUILTIN_SET_PROTOTYPE_ADD_FUNCTION_ID,
        debug: "Set.prototype.add",
        flags: [],
        installer: None,
        native: "add",
    }
    SetPrototypeClear {
        function: FunctionOrdinal(738) => BUILTIN_SET_PROTOTYPE_CLEAR_FUNCTION_ID,
        debug: "Set.prototype.clear",
        flags: [],
        installer: None,
        native: "clear",
    }
    SetPrototypeDelete {
        function: FunctionOrdinal(739) => BUILTIN_SET_PROTOTYPE_DELETE_FUNCTION_ID,
        debug: "Set.prototype.delete",
        flags: [],
        installer: None,
        native: "delete",
    }
    SetPrototypeDifference {
        function: FunctionOrdinal(740) => BUILTIN_SET_PROTOTYPE_DIFFERENCE_FUNCTION_ID,
        debug: "Set.prototype.difference",
        flags: [],
        installer: None,
        native: "difference",
    }
    SetPrototypeForEach {
        function: FunctionOrdinal(741) => BUILTIN_SET_PROTOTYPE_FOR_EACH_FUNCTION_ID,
        debug: "Set.prototype.forEach",
        flags: [],
        installer: None,
        native: "forEach",
    }
    SetPrototypeIntersection {
        function: FunctionOrdinal(742) => BUILTIN_SET_PROTOTYPE_INTERSECTION_FUNCTION_ID,
        debug: "Set.prototype.intersection",
        flags: [],
        installer: None,
        native: "intersection",
    }
    SetPrototypeIsDisjointFrom {
        function: FunctionOrdinal(743) => BUILTIN_SET_PROTOTYPE_IS_DISJOINT_FROM_FUNCTION_ID,
        debug: "Set.prototype.isDisjointFrom",
        flags: [],
        installer: None,
        native: "isDisjointFrom",
    }
    SetPrototypeIsSubsetOf {
        function: FunctionOrdinal(744) => BUILTIN_SET_PROTOTYPE_IS_SUBSET_OF_FUNCTION_ID,
        debug: "Set.prototype.isSubsetOf",
        flags: [],
        installer: None,
        native: "isSubsetOf",
    }
    SetPrototypeIsSupersetOf {
        function: FunctionOrdinal(745) => BUILTIN_SET_PROTOTYPE_IS_SUPERSET_OF_FUNCTION_ID,
        debug: "Set.prototype.isSupersetOf",
        flags: [],
        installer: None,
        native: "isSupersetOf",
    }
    SetPrototypeSymmetricDifference {
        function: FunctionOrdinal(746) => BUILTIN_SET_PROTOTYPE_SYMMETRIC_DIFFERENCE_FUNCTION_ID,
        debug: "Set.prototype.symmetricDifference",
        flags: [],
        installer: None,
        native: "symmetricDifference",
    }
    SetPrototypeUnion {
        function: FunctionOrdinal(747) => BUILTIN_SET_PROTOTYPE_UNION_FUNCTION_ID,
        debug: "Set.prototype.union",
        flags: [],
        installer: None,
        native: "union",
    }
    SetPrototypeValues {
        function: FunctionOrdinal(748) => BUILTIN_SET_PROTOTYPE_VALUES_FUNCTION_ID,
        debug: "Set.prototype.values",
        flags: [],
        installer: None,
        native: "values",
    }
    SetPrototypeEntries {
        function: FunctionOrdinal(749) => BUILTIN_SET_PROTOTYPE_ENTRIES_FUNCTION_ID,
        debug: "Set.prototype.entries",
        flags: [],
        installer: None,
        native: "entries",
    }
    SetIteratorNext {
        function: FunctionOrdinal(750) => BUILTIN_SET_ITERATOR_NEXT_FUNCTION_ID,
        debug: "Set Iterator.prototype.next",
        flags: [],
        installer: None,
        native: "next",
    }
    SetPrototypeHas {
        function: FunctionOrdinal(751) => BUILTIN_SET_PROTOTYPE_HAS_FUNCTION_ID,
        debug: "Set.prototype.has",
        flags: [],
        installer: None,
        native: "has",
    }
    SetPrototypeSizeGetter {
        function: FunctionOrdinal(752) => BUILTIN_SET_PROTOTYPE_SIZE_GETTER_FUNCTION_ID,
        debug: "get Set.prototype.size",
        flags: [],
        installer: None,
        native: "get size",
    }
    SymbolConstructor {
        function: FunctionOrdinal(753) => BUILTIN_SYMBOL_FUNCTION_ID,
        global: GlobalOrdinal(36),
        global_name: SYMBOL_NAME,
        debug: SYMBOL_NAME,
        flags: [CONSTRUCTABLE],
        installer: Symbol,
        native: SYMBOL_NAME,
    }
    SymbolFor {
        function: FunctionOrdinal(754) => BUILTIN_SYMBOL_FOR_FUNCTION_ID,
        debug: "Symbol.for",
        flags: [STATIC_METHOD],
        installer: None,
        native: "for",
    }
    SymbolKeyFor {
        function: FunctionOrdinal(755) => BUILTIN_SYMBOL_KEY_FOR_FUNCTION_ID,
        debug: "Symbol.keyFor",
        flags: [STATIC_METHOD],
        installer: None,
        native: "keyFor",
    }
    SymbolPrototypeDescriptionGetter {
        function: FunctionOrdinal(756) => "$builtin.Symbol.prototype.description",
        debug: "get Symbol.prototype.description",
        flags: [],
        installer: None,
        native: "get description",
    }
    SymbolPrototypeToString {
        function: FunctionOrdinal(757) => "$builtin.Symbol.prototype.toString",
        debug: "Symbol.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    SymbolPrototypeValueOf {
        function: FunctionOrdinal(758) => "$builtin.Symbol.prototype.valueOf",
        debug: "Symbol.prototype.valueOf",
        flags: [],
        installer: None,
        native: "valueOf",
    }
    SymbolPrototypeToPrimitive {
        function: FunctionOrdinal(759) => "$builtin.Symbol.prototype.toPrimitive",
        debug: "Symbol.prototype[Symbol.toPrimitive]",
        flags: [],
        installer: None,
        native: "[Symbol.toPrimitive]",
    }
    ErrorConstructor {
        function: FunctionOrdinal(760) => BUILTIN_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(37),
        global_name: ERROR_NAME,
        debug: ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: Error,
        native: ERROR_NAME,
    }
    ErrorIsError {
        function: FunctionOrdinal(761) => BUILTIN_ERROR_IS_ERROR_FUNCTION_ID,
        debug: "Error.isError",
        flags: [STATIC_METHOD],
        installer: None,
        native: "isError",
    }
    EvalErrorConstructor {
        function: FunctionOrdinal(762) => BUILTIN_EVAL_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(38),
        global_name: EVAL_ERROR_NAME,
        debug: EVAL_ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: None,
        native: EVAL_ERROR_NAME,
    }
    AggregateErrorConstructor {
        function: FunctionOrdinal(763) => BUILTIN_AGGREGATE_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(39),
        global_name: AGGREGATE_ERROR_NAME,
        debug: AGGREGATE_ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: None,
        native: AGGREGATE_ERROR_NAME,
    }
    SuppressedErrorConstructor {
        function: FunctionOrdinal(764) => BUILTIN_SUPPRESSED_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(40),
        global_name: SUPPRESSED_ERROR_NAME,
        debug: SUPPRESSED_ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: None,
        native: SUPPRESSED_ERROR_NAME,
    }
    RangeErrorConstructor {
        function: FunctionOrdinal(765) => BUILTIN_RANGE_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(41),
        global_name: RANGE_ERROR_NAME,
        debug: RANGE_ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: None,
        native: RANGE_ERROR_NAME,
    }
    SyntaxErrorConstructor {
        function: FunctionOrdinal(766) => BUILTIN_SYNTAX_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(42),
        global_name: SYNTAX_ERROR_NAME,
        debug: SYNTAX_ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: None,
        native: SYNTAX_ERROR_NAME,
    }
    TypeErrorConstructor {
        function: FunctionOrdinal(767) => BUILTIN_TYPE_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(43),
        global_name: TYPE_ERROR_NAME,
        debug: TYPE_ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: None,
        native: TYPE_ERROR_NAME,
    }
    URIErrorConstructor {
        function: FunctionOrdinal(768) => BUILTIN_URI_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(44),
        global_name: URI_ERROR_NAME,
        debug: URI_ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: None,
        native: URI_ERROR_NAME,
    }
    ReferenceErrorConstructor {
        function: FunctionOrdinal(769) => BUILTIN_REFERENCE_ERROR_FUNCTION_ID,
        global: GlobalOrdinal(45),
        global_name: REFERENCE_ERROR_NAME,
        debug: REFERENCE_ERROR_NAME,
        flags: [CONSTRUCTABLE, ERROR_CONSTRUCTOR],
        installer: None,
        native: REFERENCE_ERROR_NAME,
    }
    ErrorPrototypeToString {
        function: FunctionOrdinal(770) => BUILTIN_ERROR_PROTOTYPE_TO_STRING_FUNCTION_ID,
        debug: "Error.prototype.toString",
        flags: [],
        installer: None,
        native: "toString",
    }
    ThrowTypeError {
        function: FunctionOrdinal(771) => BUILTIN_THROW_TYPE_ERROR_FUNCTION_ID,
        debug: "%ThrowTypeError%",
        flags: [],
        installer: None,
        native: "",
    }
    BoundFunctionInvoker {
        function: FunctionOrdinal(772) => BUILTIN_BOUND_FUNCTION_INVOKER_FUNCTION_ID,
        debug: "[[BoundFunctionInvoke]]",
        flags: [CONSTRUCTABLE],
        installer: None,
    }
    Escape {
        function: FunctionOrdinal(773) => BUILTIN_ESCAPE_FUNCTION_ID,
        global: GlobalOrdinal(46),
        global_name: ESCAPE_NAME,
        debug: ESCAPE_NAME,
        flags: [],
        installer: None,
        native: ESCAPE_NAME,
    }
    Unescape {
        function: FunctionOrdinal(774) => BUILTIN_UNESCAPE_FUNCTION_ID,
        global: GlobalOrdinal(47),
        global_name: UNESCAPE_NAME,
        debug: UNESCAPE_NAME,
        flags: [],
        installer: None,
        native: UNESCAPE_NAME,
    }
    EncodeUri {
        function: FunctionOrdinal(775) => BUILTIN_ENCODE_URI_FUNCTION_ID,
        global: GlobalOrdinal(48),
        global_name: ENCODE_URI_NAME,
        debug: ENCODE_URI_NAME,
        flags: [],
        installer: None,
        native: ENCODE_URI_NAME,
    }
    EncodeUriComponent {
        function: FunctionOrdinal(776) => BUILTIN_ENCODE_URI_COMPONENT_FUNCTION_ID,
        global: GlobalOrdinal(49),
        global_name: ENCODE_URI_COMPONENT_NAME,
        debug: ENCODE_URI_COMPONENT_NAME,
        flags: [],
        installer: None,
        native: ENCODE_URI_COMPONENT_NAME,
    }
    DecodeUri {
        function: FunctionOrdinal(777) => BUILTIN_DECODE_URI_FUNCTION_ID,
        global: GlobalOrdinal(50),
        global_name: DECODE_URI_NAME,
        debug: DECODE_URI_NAME,
        flags: [],
        installer: None,
        native: DECODE_URI_NAME,
    }
    DecodeUriComponent {
        function: FunctionOrdinal(778) => BUILTIN_DECODE_URI_COMPONENT_FUNCTION_ID,
        global: GlobalOrdinal(51),
        global_name: DECODE_URI_COMPONENT_NAME,
        debug: DECODE_URI_COMPONENT_NAME,
        flags: [],
        installer: None,
        native: DECODE_URI_COMPONENT_NAME,
    }
    DisposableStackConstructor {
        function: FunctionOrdinal(779) => BUILTIN_DISPOSABLE_STACK_FUNCTION_ID,
        global: GlobalOrdinal(52),
        global_name: DISPOSABLE_STACK_NAME,
        debug: DISPOSABLE_STACK_NAME,
        flags: [CONSTRUCTABLE],
        installer: DisposableStack,
        native: DISPOSABLE_STACK_NAME,
    }
    DisposableStackPrototypeUse {
        function: FunctionOrdinal(780) => BUILTIN_DISPOSABLE_STACK_PROTOTYPE_USE_FUNCTION_ID,
        debug: "DisposableStack.prototype.use",
        flags: [],
        installer: None,
        native: "use",
    }
    DisposableStackPrototypeAdopt {
        function: FunctionOrdinal(781) => BUILTIN_DISPOSABLE_STACK_PROTOTYPE_ADOPT_FUNCTION_ID,
        debug: "DisposableStack.prototype.adopt",
        flags: [],
        installer: None,
        native: "adopt",
    }
    DisposableStackPrototypeDefer {
        function: FunctionOrdinal(782) => BUILTIN_DISPOSABLE_STACK_PROTOTYPE_DEFER_FUNCTION_ID,
        debug: "DisposableStack.prototype.defer",
        flags: [],
        installer: None,
        native: "defer",
    }
    DisposableStackPrototypeMove {
        function: FunctionOrdinal(783) => BUILTIN_DISPOSABLE_STACK_PROTOTYPE_MOVE_FUNCTION_ID,
        debug: "DisposableStack.prototype.move",
        flags: [],
        installer: None,
        native: "move",
    }
    DisposableStackPrototypeDispose {
        function: FunctionOrdinal(784) => BUILTIN_DISPOSABLE_STACK_PROTOTYPE_DISPOSE_FUNCTION_ID,
        debug: "DisposableStack.prototype.dispose",
        flags: [],
        installer: None,
        native: "dispose",
    }
    DisposableStackPrototypeDisposedGetter {
        function: FunctionOrdinal(785) => BUILTIN_DISPOSABLE_STACK_PROTOTYPE_DISPOSED_GETTER_FUNCTION_ID,
        debug: "get DisposableStack.prototype.disposed",
        flags: [],
        installer: None,
        native: "get disposed",
    }
}

impl StandardBuiltinId {
    /// Whether this builtin reads the realm's host randomness provider. This
    /// catalog bit is the sole authority for importing `lila_host.random_f64`.
    pub const fn requires_random(self) -> bool {
        self.flags().contains(BuiltinFlags::RANDOM)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disposable_stack_surface_has_one_closed_builtin_identity_per_algorithm() {
        let constructor = StandardBuiltinId::DisposableStackConstructor;
        for (builtin, native_name) in [
            (constructor, "DisposableStack"),
            (StandardBuiltinId::DisposableStackPrototypeUse, "use"),
            (StandardBuiltinId::DisposableStackPrototypeAdopt, "adopt"),
            (StandardBuiltinId::DisposableStackPrototypeDefer, "defer"),
            (StandardBuiltinId::DisposableStackPrototypeMove, "move"),
            (
                StandardBuiltinId::DisposableStackPrototypeDispose,
                "dispose",
            ),
            (
                StandardBuiltinId::DisposableStackPrototypeDisposedGetter,
                "get disposed",
            ),
        ] {
            assert_eq!(
                StandardBuiltinId::from_function_id(&builtin.function_id()),
                Some(builtin)
            );
            assert_eq!(builtin.native_function_name(), Some(native_name));
            assert!(StandardBuiltinId::all_functions().contains(&builtin));
            assert_eq!(builtin.constructable(), builtin == constructor);
        }
        assert!(StandardBuiltinId::all_globals().contains(&constructor));
    }

    #[test]
    fn math_random_alone_declares_the_host_random_import() {
        assert!(StandardBuiltinId::MathRandom.requires_random());
        assert_eq!(
            StandardBuiltinId::all_functions()
                .iter()
                .copied()
                .filter(|builtin| builtin.requires_random())
                .collect::<Vec<_>>(),
            vec![StandardBuiltinId::MathRandom]
        );
    }

    #[test]
    fn intrinsic_installer_roots_are_explicit_and_stable() {
        use StandardBuiltinId as Builtin;
        use StandardBuiltinInstaller as Installer;

        let roots = StandardBuiltinId::all_functions()
            .iter()
            .copied()
            .map(|builtin| (builtin, builtin.intrinsic_installer()))
            .filter(|(_, installer)| *installer != Installer::None)
            .collect::<Vec<_>>();

        assert_eq!(
            roots.as_slice(),
            &[
                (Builtin::FunctionConstructor, Installer::Function),
                (Builtin::ObjectConstructor, Installer::Object),
                (Builtin::ProxyConstructor, Installer::Proxy),
                (Builtin::ArrayConstructor, Installer::Array),
                (Builtin::IteratorConstructor, Installer::Iterator),
                (Builtin::ArrayBufferConstructor, Installer::ArrayBuffer),
                (
                    Builtin::SharedArrayBufferConstructor,
                    Installer::ArrayBuffer,
                ),
                (Builtin::DataViewConstructor, Installer::DataView),
                (Builtin::DateConstructor, Installer::Date),
                (
                    Builtin::TemporalPlainDateConstructor,
                    Installer::TemporalPlainDate,
                ),
                (
                    Builtin::TemporalPlainYearMonthConstructor,
                    Installer::TemporalPlainYearMonth,
                ),
                (
                    Builtin::TemporalPlainMonthDayConstructor,
                    Installer::TemporalPlainMonthDay,
                ),
                (
                    Builtin::TemporalPlainTimeConstructor,
                    Installer::TemporalPlainTime,
                ),
                (
                    Builtin::TemporalPlainDateTimeConstructor,
                    Installer::TemporalPlainDateTime,
                ),
                (
                    Builtin::TemporalDurationConstructor,
                    Installer::TemporalDuration,
                ),
                (
                    Builtin::TemporalInstantConstructor,
                    Installer::TemporalInstant,
                ),
                (
                    Builtin::TemporalZonedDateTimeConstructor,
                    Installer::TemporalZonedDateTime,
                ),
                (Builtin::IntlLocaleConstructor, Installer::IntlLocale),
                (
                    Builtin::IntlDateTimeFormatConstructor,
                    Installer::IntlDateTimeFormat,
                ),
                (Builtin::RegExpConstructor, Installer::RegExp),
                (Builtin::BigIntConstructor, Installer::BigInt),
                (Builtin::NumberConstructor, Installer::Number),
                (Builtin::StringConstructor, Installer::String),
                (Builtin::BooleanConstructor, Installer::Boolean),
                (Builtin::PromiseConstructor, Installer::Promise),
                (Builtin::MapConstructor, Installer::Map),
                (Builtin::WeakMapConstructor, Installer::WeakMap),
                (Builtin::WeakSetConstructor, Installer::WeakSet),
                (Builtin::WeakRefConstructor, Installer::WeakRef),
                (
                    Builtin::FinalizationRegistryConstructor,
                    Installer::FinalizationRegistry,
                ),
                (
                    Builtin::AsyncDisposableStackConstructor,
                    Installer::AsyncDisposableStack,
                ),
                (Builtin::SetConstructor, Installer::Set),
                (Builtin::SymbolConstructor, Installer::Symbol),
                (Builtin::ErrorConstructor, Installer::Error),
                (
                    Builtin::DisposableStackConstructor,
                    Installer::DisposableStack,
                ),
            ]
        );
    }
}
