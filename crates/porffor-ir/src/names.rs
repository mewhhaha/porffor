pub(crate) const SCRIPT_OWNER_ID: &str = "$script";
pub(crate) const MAX_STATIC_ARRAY_SHAPE_INDEX: usize = 1_000_000;
pub(crate) const MAX_ARRAY_INDEX: f64 = 4_294_967_294.0;
pub const JS_STRING_SURROGATE_SENTINEL: char = '\u{F0000}';
pub const LEXICAL_THIS_NAME: &str = "$this";
pub const LEXICAL_ARGUMENTS_NAME: &str = "$arguments";
pub const LEXICAL_NEW_TARGET_NAME: &str = "$new.target";
/// Compiler-private lexical `[[HomeObject]]` binding for arrows using `super`.
pub const LEXICAL_HOME_OBJECT_NAME: &str = "$homeObject";
/// Compiler-private environment bindings for lexical `super()` in arrows.
pub const DERIVED_ACTIVATION_THIS_NAME: &str = "$derived.this";
pub const DERIVED_ACTIVATION_THIS_STATUS_NAME: &str = "$derived.thisStatus";
pub const DERIVED_ACTIVATION_NEW_TARGET_NAME: &str = "$derived.newTarget";
pub const DERIVED_ACTIVATION_FUNCTION_NAME: &str = "$derived.activeFunction";
pub(crate) const TDZ_BINDING_STORAGE_PREFIX: &str = "$tdz.";
pub const GLOBAL_THIS_NAME: &str = "globalThis";
pub const MATH_NAME: &str = "Math";
pub const PRINT_NAME: &str = "print";
pub const GC_NAME: &str = "gc";
pub const ASSERT_THROWS_NAME: &str = "__porfAssertThrows";
pub const IS_CONSTRUCTOR_NAME: &str = "__porfIsConstructor";
pub const CREATE_REALM_NAME: &str = "__porfCreateRealm";
pub const PARSE_INT_NAME: &str = "parseInt";
pub const PARSE_FLOAT_NAME: &str = "parseFloat";
pub const ESCAPE_NAME: &str = "escape";
pub const UNESCAPE_NAME: &str = "unescape";
pub const HOST_PRINT_FUNCTION_ID: &str = "$host.print";
pub const HOST_GC_FUNCTION_ID: &str = "$host.gc";
pub const HOST_ASSERT_THROWS_FUNCTION_ID: &str = "$host.assertThrows";
pub const HOST_IS_CONSTRUCTOR_FUNCTION_ID: &str = "$host.isConstructor";
pub const HOST_CREATE_REALM_FUNCTION_ID: &str = "$host.createRealm";
pub const HOST_PARSE_INT_FUNCTION_ID: &str = "$host.parseInt";
pub const HOST_PARSE_FLOAT_FUNCTION_ID: &str = "$host.parseFloat";
pub const DETACH_ARRAY_BUFFER_NAME: &str = "__porfDetachArrayBuffer";
pub const HOST_DETACH_ARRAY_BUFFER_FUNCTION_ID: &str = "$host.detachArrayBuffer";
pub const FUNCTION_NAME: &str = "Function";
pub const OBJECT_NAME: &str = "Object";
pub const ARRAY_NAME: &str = "Array";
pub const ARRAY_BUFFER_NAME: &str = "ArrayBuffer";
pub const ARRAY_BUFFER_IMMUTABLE_SLOT: &str = "$ArrayBuffer.immutable";
pub const ARRAY_BUFFER_MAX_BYTE_LENGTH_SLOT: &str = "$ArrayBuffer.maxByteLength";
pub const ARRAY_BUFFER_RESIZABLE_SLOT: &str = "$ArrayBuffer.resizable";
pub const ARRAY_BUFFER_SHARED_SLOT: &str = "$ArrayBuffer.shared";
pub const SHARED_ARRAY_BUFFER_NAME: &str = "SharedArrayBuffer";
pub const DATA_VIEW_NAME: &str = "DataView";
pub const DATE_NAME: &str = "Date";
pub const REGEXP_NAME: &str = "RegExp";
pub const JSON_NAME: &str = "JSON";
pub const ATOMICS_NAME: &str = "Atomics";
pub const FLOAT64_ARRAY_NAME: &str = "Float64Array";
pub const FLOAT32_ARRAY_NAME: &str = "Float32Array";
pub const INT32_ARRAY_NAME: &str = "Int32Array";
pub const INT16_ARRAY_NAME: &str = "Int16Array";
pub const INT8_ARRAY_NAME: &str = "Int8Array";
pub const UINT32_ARRAY_NAME: &str = "Uint32Array";
pub const UINT16_ARRAY_NAME: &str = "Uint16Array";
pub const UINT8_ARRAY_NAME: &str = "Uint8Array";
pub const UINT8_CLAMPED_ARRAY_NAME: &str = "Uint8ClampedArray";
pub const BIGINT64_ARRAY_NAME: &str = "BigInt64Array";
pub const BIGUINT64_ARRAY_NAME: &str = "BigUint64Array";
pub const BIGINT_NAME: &str = "BigInt";
pub const PROXY_NAME: &str = "Proxy";
pub const REFLECT_NAME: &str = "Reflect";
pub const NUMBER_NAME: &str = "Number";
pub const STRING_NAME: &str = "String";
pub const BOOLEAN_NAME: &str = "Boolean";
pub const SYMBOL_NAME: &str = "Symbol";
pub const ERROR_NAME: &str = "Error";
pub const EVAL_ERROR_NAME: &str = "EvalError";
pub const AGGREGATE_ERROR_NAME: &str = "AggregateError";
pub const SUPPRESSED_ERROR_NAME: &str = "SuppressedError";
pub const RANGE_ERROR_NAME: &str = "RangeError";
pub const SYNTAX_ERROR_NAME: &str = "SyntaxError";
pub const TYPE_ERROR_NAME: &str = "TypeError";
pub const URI_ERROR_NAME: &str = "URIError";
pub const REFERENCE_ERROR_NAME: &str = "ReferenceError";
pub const BUILTIN_FUNCTION_FUNCTION_ID: &str = "$builtin.Function";
pub const BUILTIN_FUNCTION_PROTOTYPE_CALL_FUNCTION_ID: &str = "$builtin.Function.prototype.call";
pub const BUILTIN_FUNCTION_PROTOTYPE_APPLY_FUNCTION_ID: &str = "$builtin.Function.prototype.apply";
pub const BUILTIN_FUNCTION_PROTOTYPE_BIND_FUNCTION_ID: &str = "$builtin.Function.prototype.bind";
pub const BUILTIN_FUNCTION_PROTOTYPE_TO_STRING_FUNCTION_ID: &str =
    "$builtin.Function.prototype.toString";
pub const BUILTIN_EVAL_FUNCTION_ID: &str = "$builtin.eval";
pub const BUILTIN_OBJECT_FUNCTION_ID: &str = "$builtin.Object";
pub const BUILTIN_OBJECT_CREATE_FUNCTION_ID: &str = "$builtin.Object.create";
pub const BUILTIN_OBJECT_GET_PROTOTYPE_OF_FUNCTION_ID: &str = "$builtin.Object.getPrototypeOf";
pub const BUILTIN_OBJECT_SET_PROTOTYPE_OF_FUNCTION_ID: &str = "$builtin.Object.setPrototypeOf";
pub const BUILTIN_OBJECT_DEFINE_PROPERTY_FUNCTION_ID: &str = "$builtin.Object.defineProperty";
pub const BUILTIN_OBJECT_DEFINE_PROPERTIES_FUNCTION_ID: &str = "$builtin.Object.defineProperties";
pub const BUILTIN_OBJECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID: &str =
    "$builtin.Object.getOwnPropertyDescriptor";
pub const BUILTIN_OBJECT_GET_OWN_PROPERTY_NAMES_FUNCTION_ID: &str =
    "$builtin.Object.getOwnPropertyNames";
pub const BUILTIN_OBJECT_GET_OWN_PROPERTY_SYMBOLS_FUNCTION_ID: &str =
    "$builtin.Object.getOwnPropertySymbols";
pub const BUILTIN_OBJECT_KEYS_FUNCTION_ID: &str = "$builtin.Object.keys";
pub const BUILTIN_OBJECT_VALUES_FUNCTION_ID: &str = "$builtin.Object.values";
pub const BUILTIN_OBJECT_HAS_OWN_FUNCTION_ID: &str = "$builtin.Object.hasOwn";
pub const BUILTIN_OBJECT_IS_FUNCTION_ID: &str = "$builtin.Object.is";
pub const BUILTIN_OBJECT_IS_SEALED_FUNCTION_ID: &str = "$builtin.Object.isSealed";
pub const BUILTIN_OBJECT_IS_FROZEN_FUNCTION_ID: &str = "$builtin.Object.isFrozen";
pub const BUILTIN_OBJECT_FREEZE_FUNCTION_ID: &str = "$builtin.Object.freeze";
pub const BUILTIN_OBJECT_IS_EXTENSIBLE_FUNCTION_ID: &str = "$builtin.Object.isExtensible";
pub const BUILTIN_OBJECT_PREVENT_EXTENSIONS_FUNCTION_ID: &str = "$builtin.Object.preventExtensions";
pub const BUILTIN_OBJECT_PROTOTYPE_HAS_OWN_PROPERTY_FUNCTION_ID: &str =
    "$builtin.Object.prototype.hasOwnProperty";
pub const BUILTIN_OBJECT_PROTOTYPE_LOOKUP_GETTER_FUNCTION_ID: &str =
    "$builtin.Object.prototype.__lookupGetter__";
pub const BUILTIN_OBJECT_PROTOTYPE_LOOKUP_SETTER_FUNCTION_ID: &str =
    "$builtin.Object.prototype.__lookupSetter__";
pub const BUILTIN_OBJECT_PROTOTYPE_PROPERTY_IS_ENUMERABLE_FUNCTION_ID: &str =
    "$builtin.Object.prototype.propertyIsEnumerable";
pub const BUILTIN_OBJECT_PROTOTYPE_IS_PROTOTYPE_OF_FUNCTION_ID: &str =
    "$builtin.Object.prototype.isPrototypeOf";
pub const BUILTIN_OBJECT_PROTOTYPE_TO_STRING_FUNCTION_ID: &str =
    "$builtin.Object.prototype.toString";
pub const BUILTIN_OBJECT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID: &str =
    "$builtin.Object.prototype.toLocaleString";
pub const BUILTIN_OBJECT_PROTOTYPE_VALUE_OF_FUNCTION_ID: &str = "$builtin.Object.prototype.valueOf";
pub const BUILTIN_PROXY_FUNCTION_ID: &str = "$builtin.Proxy";
pub const BUILTIN_PROXY_REVOCABLE_FUNCTION_ID: &str = "$builtin.Proxy.revocable";
pub const BUILTIN_PROXY_REVOKE_FUNCTION_ID: &str = "$builtin.[[ProxyRevoke]]";
pub const BUILTIN_REFLECT_CONSTRUCT_FUNCTION_ID: &str = "$builtin.Reflect.construct";
pub const BUILTIN_REFLECT_APPLY_FUNCTION_ID: &str = "$builtin.Reflect.apply";
pub const BUILTIN_REFLECT_GET_FUNCTION_ID: &str = "$builtin.Reflect.get";
pub const BUILTIN_REFLECT_GET_PROTOTYPE_OF_FUNCTION_ID: &str = "$builtin.Reflect.getPrototypeOf";
pub const BUILTIN_REFLECT_GET_OWN_PROPERTY_DESCRIPTOR_FUNCTION_ID: &str =
    "$builtin.Reflect.getOwnPropertyDescriptor";
pub const BUILTIN_REFLECT_SET_FUNCTION_ID: &str = "$builtin.Reflect.set";
pub const BUILTIN_REFLECT_HAS_FUNCTION_ID: &str = "$builtin.Reflect.has";
pub const BUILTIN_REFLECT_DEFINE_PROPERTY_FUNCTION_ID: &str = "$builtin.Reflect.defineProperty";
pub const BUILTIN_REFLECT_DELETE_PROPERTY_FUNCTION_ID: &str = "$builtin.Reflect.deleteProperty";
pub const BUILTIN_REFLECT_IS_EXTENSIBLE_FUNCTION_ID: &str = "$builtin.Reflect.isExtensible";
pub const BUILTIN_REFLECT_PREVENT_EXTENSIONS_FUNCTION_ID: &str =
    "$builtin.Reflect.preventExtensions";
pub const BUILTIN_REFLECT_SET_PROTOTYPE_OF_FUNCTION_ID: &str = "$builtin.Reflect.setPrototypeOf";
pub const BUILTIN_REFLECT_OWN_KEYS_FUNCTION_ID: &str = "$builtin.Reflect.ownKeys";
pub const BUILTIN_ARRAY_FUNCTION_ID: &str = "$builtin.Array";
pub const BUILTIN_ARRAY_FROM_FUNCTION_ID: &str = "$builtin.Array.from";
pub const BUILTIN_ARRAY_OF_FUNCTION_ID: &str = "$builtin.Array.of";
pub const BUILTIN_ARRAY_IS_ARRAY_FUNCTION_ID: &str = "$builtin.Array.isArray";
pub const BUILTIN_ARRAY_SPECIES_GETTER_FUNCTION_ID: &str = "$builtin.Array[Symbol.species].get";
pub const BUILTIN_ARRAY_PROTOTYPE_CONCAT_FUNCTION_ID: &str = "$builtin.Array.prototype.concat";
pub const BUILTIN_ARRAY_PROTOTYPE_JOIN_FUNCTION_ID: &str = "$builtin.Array.prototype.join";
pub const BUILTIN_ARRAY_PROTOTYPE_SLICE_FUNCTION_ID: &str = "$builtin.Array.prototype.slice";
pub const BUILTIN_ARRAY_PROTOTYPE_SPLICE_FUNCTION_ID: &str = "$builtin.Array.prototype.splice";
pub const BUILTIN_ARRAY_PROTOTYPE_SORT_FUNCTION_ID: &str = "$builtin.Array.prototype.sort";
pub const BUILTIN_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID: &str =
    "$builtin.Array.prototype.toLocaleString";
pub const BUILTIN_ARRAY_PROTOTYPE_FLAT_FUNCTION_ID: &str = "$builtin.Array.prototype.flat";
pub const BUILTIN_ARRAY_PROTOTYPE_FLAT_MAP_FUNCTION_ID: &str = "$builtin.Array.prototype.flatMap";
pub const BUILTIN_ARRAY_PROTOTYPE_AT_FUNCTION_ID: &str = "$builtin.Array.prototype.at";
pub const BUILTIN_ARRAY_PROTOTYPE_TO_REVERSED_FUNCTION_ID: &str =
    "$builtin.Array.prototype.toReversed";
pub const BUILTIN_ARRAY_PROTOTYPE_TO_SPLICED_FUNCTION_ID: &str =
    "$builtin.Array.prototype.toSpliced";
pub const BUILTIN_ARRAY_PROTOTYPE_TO_SORTED_FUNCTION_ID: &str = "$builtin.Array.prototype.toSorted";
pub const BUILTIN_ARRAY_PROTOTYPE_WITH_FUNCTION_ID: &str = "$builtin.Array.prototype.with";
pub const BUILTIN_ARRAY_PROTOTYPE_REVERSE_FUNCTION_ID: &str = "$builtin.Array.prototype.reverse";
pub const BUILTIN_ARRAY_PROTOTYPE_COPY_WITHIN_FUNCTION_ID: &str =
    "$builtin.Array.prototype.copyWithin";
pub const BUILTIN_ARRAY_PROTOTYPE_INCLUDES_FUNCTION_ID: &str = "$builtin.Array.prototype.includes";
pub const BUILTIN_ARRAY_PROTOTYPE_INDEX_OF_FUNCTION_ID: &str = "$builtin.Array.prototype.indexOf";
pub const BUILTIN_ARRAY_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID: &str =
    "$builtin.Array.prototype.lastIndexOf";
pub const BUILTIN_ARRAY_PROTOTYPE_FIND_FUNCTION_ID: &str = "$builtin.Array.prototype.find";
pub const BUILTIN_ARRAY_PROTOTYPE_FIND_INDEX_FUNCTION_ID: &str =
    "$builtin.Array.prototype.findIndex";
pub const BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_FUNCTION_ID: &str = "$builtin.Array.prototype.findLast";
pub const BUILTIN_ARRAY_PROTOTYPE_FIND_LAST_INDEX_FUNCTION_ID: &str =
    "$builtin.Array.prototype.findLastIndex";
pub const BUILTIN_ARRAY_PROTOTYPE_EVERY_FUNCTION_ID: &str = "$builtin.Array.prototype.every";
pub const BUILTIN_ARRAY_PROTOTYPE_SOME_FUNCTION_ID: &str = "$builtin.Array.prototype.some";
pub const BUILTIN_ARRAY_PROTOTYPE_FOR_EACH_FUNCTION_ID: &str = "$builtin.Array.prototype.forEach";
pub const BUILTIN_ARRAY_PROTOTYPE_FILTER_FUNCTION_ID: &str = "$builtin.Array.prototype.filter";
pub const BUILTIN_ARRAY_PROTOTYPE_MAP_FUNCTION_ID: &str = "$builtin.Array.prototype.map";
pub const BUILTIN_ARRAY_PROTOTYPE_REDUCE_FUNCTION_ID: &str = "$builtin.Array.prototype.reduce";
pub const BUILTIN_ARRAY_PROTOTYPE_REDUCE_RIGHT_FUNCTION_ID: &str =
    "$builtin.Array.prototype.reduceRight";
pub const BUILTIN_ARRAY_PROTOTYPE_POP_FUNCTION_ID: &str = "$builtin.Array.prototype.pop";
pub const BUILTIN_ARRAY_PROTOTYPE_PUSH_FUNCTION_ID: &str = "$builtin.Array.prototype.push";
pub const BUILTIN_ARRAY_PROTOTYPE_SHIFT_FUNCTION_ID: &str = "$builtin.Array.prototype.shift";
pub const BUILTIN_ARRAY_PROTOTYPE_UNSHIFT_FUNCTION_ID: &str = "$builtin.Array.prototype.unshift";
pub const BUILTIN_ARRAY_PROTOTYPE_FILL_FUNCTION_ID: &str = "$builtin.Array.prototype.fill";
pub const BUILTIN_ARRAY_PROTOTYPE_KEYS_FUNCTION_ID: &str = "$builtin.Array.prototype.keys";
pub const BUILTIN_ARRAY_PROTOTYPE_ENTRIES_FUNCTION_ID: &str = "$builtin.Array.prototype.entries";
pub const BUILTIN_ARRAY_PROTOTYPE_VALUES_FUNCTION_ID: &str = "$builtin.Array.prototype.values";
pub const BUILTIN_ARRAY_ITERATOR_NEXT_FUNCTION_ID: &str = "$builtin.ArrayIterator.next";
pub const BUILTIN_ARRAY_ITERATOR_IDENTITY_FUNCTION_ID: &str = "$builtin.ArrayIterator.identity";
pub const BUILTIN_STRING_ITERATOR_NEXT_FUNCTION_ID: &str = "$builtin.StringIterator.next";
pub const BUILTIN_ITERATOR_FUNCTION_ID: &str = "$builtin.Iterator";
pub const BUILTIN_ITERATOR_FROM_FUNCTION_ID: &str = "$builtin.Iterator.from";
pub const BUILTIN_ITERATOR_ZIP_FUNCTION_ID: &str = "$builtin.Iterator.zip";
pub const BUILTIN_ITERATOR_ZIP_NEXT_FUNCTION_ID: &str = "$builtin.Iterator.zip.next";
pub const BUILTIN_ITERATOR_ZIP_RETURN_FUNCTION_ID: &str = "$builtin.Iterator.zip.return";
pub const BUILTIN_ITERATOR_HELPER_NEXT_FUNCTION_ID: &str = "$builtin.Iterator.helper.next";
pub const BUILTIN_ITERATOR_HELPER_RETURN_FUNCTION_ID: &str = "$builtin.Iterator.helper.return";
pub const BUILTIN_ITERATOR_PROTOTYPE_TO_ARRAY_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype.toArray";
pub const BUILTIN_ITERATOR_PROTOTYPE_FOR_EACH_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype.forEach";
pub const BUILTIN_ITERATOR_PROTOTYPE_EVERY_FUNCTION_ID: &str = "$builtin.Iterator.prototype.every";
pub const BUILTIN_ITERATOR_PROTOTYPE_SOME_FUNCTION_ID: &str = "$builtin.Iterator.prototype.some";
pub const BUILTIN_ITERATOR_PROTOTYPE_FIND_FUNCTION_ID: &str = "$builtin.Iterator.prototype.find";
pub const BUILTIN_ITERATOR_PROTOTYPE_REDUCE_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype.reduce";
pub const BUILTIN_ITERATOR_PROTOTYPE_MAP_FUNCTION_ID: &str = "$builtin.Iterator.prototype.map";
pub const BUILTIN_ITERATOR_MAP_NEXT_FUNCTION_ID: &str = "$builtin.Iterator.map.next";
pub const BUILTIN_ITERATOR_MAP_RETURN_FUNCTION_ID: &str = "$builtin.Iterator.map.return";
pub const BUILTIN_ITERATOR_PROTOTYPE_FILTER_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype.filter";
pub const BUILTIN_ITERATOR_FILTER_NEXT_FUNCTION_ID: &str = "$builtin.Iterator.filter.next";
pub const BUILTIN_ITERATOR_FILTER_RETURN_FUNCTION_ID: &str = "$builtin.Iterator.filter.return";
pub const BUILTIN_ITERATOR_PROTOTYPE_FLAT_MAP_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype.flatMap";
pub const BUILTIN_ITERATOR_FLAT_MAP_NEXT_FUNCTION_ID: &str = "$builtin.Iterator.flatMap.next";
pub const BUILTIN_ITERATOR_FLAT_MAP_RETURN_FUNCTION_ID: &str = "$builtin.Iterator.flatMap.return";
pub const BUILTIN_ITERATOR_PROTOTYPE_TAKE_FUNCTION_ID: &str = "$builtin.Iterator.prototype.take";
pub const BUILTIN_ITERATOR_TAKE_NEXT_FUNCTION_ID: &str = "$builtin.Iterator.take.next";
pub const BUILTIN_ITERATOR_TAKE_RETURN_FUNCTION_ID: &str = "$builtin.Iterator.take.return";
pub const BUILTIN_ITERATOR_PROTOTYPE_DROP_FUNCTION_ID: &str = "$builtin.Iterator.prototype.drop";
pub const BUILTIN_ITERATOR_DROP_NEXT_FUNCTION_ID: &str = "$builtin.Iterator.drop.next";
pub const BUILTIN_ITERATOR_DROP_RETURN_FUNCTION_ID: &str = "$builtin.Iterator.drop.return";
pub const BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_GETTER_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype.constructor.get";
pub const BUILTIN_ITERATOR_PROTOTYPE_CONSTRUCTOR_SETTER_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype.constructor.set";
pub const BUILTIN_ITERATOR_PROTOTYPE_SYMBOL_DISPOSE_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype[Symbol.dispose]";
pub const BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_GETTER_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype[Symbol.toStringTag].get";
pub const BUILTIN_ITERATOR_PROTOTYPE_TO_STRING_TAG_SETTER_FUNCTION_ID: &str =
    "$builtin.Iterator.prototype[Symbol.toStringTag].set";
pub const BUILTIN_ITERATOR_FROM_WRAPPER_RETURN_FUNCTION_ID: &str =
    "$builtin.Iterator.from.wrapper.return";
pub const BUILTIN_ITERATOR_FROM_WRAPPER_NEXT_FUNCTION_ID: &str =
    "$builtin.Iterator.from.wrapper.next";
pub const BUILTIN_ARRAY_BUFFER_FUNCTION_ID: &str = "$builtin.ArrayBuffer";
pub const BUILTIN_SHARED_ARRAY_BUFFER_FUNCTION_ID: &str = "$builtin.SharedArrayBuffer";
pub const BUILTIN_ARRAY_BUFFER_IS_VIEW_FUNCTION_ID: &str = "$builtin.ArrayBuffer.isView";
pub const BUILTIN_ARRAY_BUFFER_SPECIES_GETTER_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer[Symbol.species].get";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.byteLength.get";
pub const BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID: &str =
    "$builtin.SharedArrayBuffer.prototype.byteLength.get";
pub const BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID: &str =
    "$builtin.SharedArrayBuffer.prototype.maxByteLength.get";
pub const BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROWABLE_GETTER_FUNCTION_ID: &str =
    "$builtin.SharedArrayBuffer.prototype.growable.get";
pub const BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_GROW_FUNCTION_ID: &str =
    "$builtin.SharedArrayBuffer.prototype.grow";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_DETACHED_GETTER_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.detached.get";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_MAX_BYTE_LENGTH_GETTER_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.maxByteLength.get";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZABLE_GETTER_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.resizable.get";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_RESIZE_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.resize";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.slice";
pub const BUILTIN_SHARED_ARRAY_BUFFER_PROTOTYPE_SLICE_FUNCTION_ID: &str =
    "$builtin.SharedArrayBuffer.prototype.slice";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.transfer";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_FIXED_LENGTH_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.transferToFixedLength";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_TRANSFER_TO_IMMUTABLE_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.transferToImmutable";
pub const BUILTIN_ARRAY_BUFFER_PROTOTYPE_SLICE_TO_IMMUTABLE_FUNCTION_ID: &str =
    "$builtin.ArrayBuffer.prototype.sliceToImmutable";
pub const BUILTIN_DATA_VIEW_FUNCTION_ID: &str = "$builtin.DataView";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.buffer.get";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.byteLength.get";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.byteOffset.get";
pub const BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_LENGTH_GETTER_FUNCTION_ID: &str =
    "$builtin.TypedArray.prototype.byteLength.get";
pub const BUILTIN_TYPED_ARRAY_PROTOTYPE_BYTE_OFFSET_GETTER_FUNCTION_ID: &str =
    "$builtin.TypedArray.prototype.byteOffset.get";
pub const BUILTIN_TYPED_ARRAY_PROTOTYPE_LENGTH_GETTER_FUNCTION_ID: &str =
    "$builtin.TypedArray.prototype.length.get";
pub const BUILTIN_TYPED_ARRAY_PROTOTYPE_BUFFER_GETTER_FUNCTION_ID: &str =
    "$builtin.TypedArray.prototype.buffer.get";
pub const BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_STRING_FUNCTION_ID: &str =
    "$builtin.TypedArray.prototype.toString";
pub const BUILTIN_TYPED_ARRAY_PROTOTYPE_JOIN_FUNCTION_ID: &str =
    "$builtin.TypedArray.prototype.join";
pub const BUILTIN_TYPED_ARRAY_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID: &str =
    "$builtin.TypedArray.prototype.toLocaleString";
pub const BUILTIN_TYPED_ARRAY_FROM_FUNCTION_ID: &str = "$builtin.TypedArray.from";
pub const BUILTIN_TYPED_ARRAY_OF_FUNCTION_ID: &str = "$builtin.TypedArray.of";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT8_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getUint8";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT8_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setUint8";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT8_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getInt8";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT8_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setInt8";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT16_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getUint16";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT16_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setUint16";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT16_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getInt16";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT16_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setInt16";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_UINT32_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getUint32";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_UINT32_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setUint32";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_INT32_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getInt32";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_INT32_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setInt32";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT16_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getFloat16";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT16_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setFloat16";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT32_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getFloat32";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT32_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setFloat32";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_FLOAT64_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getFloat64";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_FLOAT64_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setFloat64";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGINT64_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getBigInt64";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGINT64_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setBigInt64";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_GET_BIGUINT64_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.getBigUint64";
pub const BUILTIN_DATA_VIEW_PROTOTYPE_SET_BIGUINT64_FUNCTION_ID: &str =
    "$builtin.DataView.prototype.setBigUint64";
pub const BUILTIN_DATE_FUNCTION_ID: &str = "$builtin.Date";
pub const BUILTIN_DATE_NOW_FUNCTION_ID: &str = "$builtin.Date.now";
pub const BUILTIN_DATE_UTC_FUNCTION_ID: &str = "$builtin.Date.UTC";
pub const BUILTIN_DATE_PROTOTYPE_GET_TIME_FUNCTION_ID: &str = "$builtin.Date.prototype.getTime";
pub const BUILTIN_DATE_PROTOTYPE_SET_TIME_FUNCTION_ID: &str = "$builtin.Date.prototype.setTime";
pub const BUILTIN_DATE_PROTOTYPE_VALUE_OF_FUNCTION_ID: &str = "$builtin.Date.prototype.valueOf";
pub const BUILTIN_DATE_PROTOTYPE_GET_FULL_YEAR_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getFullYear";
pub const BUILTIN_DATE_PROTOTYPE_GET_UTC_FULL_YEAR_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getUTCFullYear";
pub const BUILTIN_DATE_PROTOTYPE_GET_MONTH_FUNCTION_ID: &str = "$builtin.Date.prototype.getMonth";
pub const BUILTIN_DATE_PROTOTYPE_GET_UTC_MONTH_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getUTCMonth";
pub const BUILTIN_DATE_PROTOTYPE_GET_DATE_FUNCTION_ID: &str = "$builtin.Date.prototype.getDate";
pub const BUILTIN_DATE_PROTOTYPE_GET_UTC_DATE_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getUTCDate";
pub const BUILTIN_DATE_PROTOTYPE_GET_DAY_FUNCTION_ID: &str = "$builtin.Date.prototype.getDay";
pub const BUILTIN_DATE_PROTOTYPE_GET_UTC_DAY_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getUTCDay";
pub const BUILTIN_DATE_PROTOTYPE_GET_HOURS_FUNCTION_ID: &str = "$builtin.Date.prototype.getHours";
pub const BUILTIN_DATE_PROTOTYPE_GET_UTC_HOURS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getUTCHours";
pub const BUILTIN_DATE_PROTOTYPE_GET_MINUTES_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getMinutes";
pub const BUILTIN_DATE_PROTOTYPE_GET_UTC_MINUTES_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getUTCMinutes";
pub const BUILTIN_DATE_PROTOTYPE_GET_SECONDS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getSeconds";
pub const BUILTIN_DATE_PROTOTYPE_GET_UTC_SECONDS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getUTCSeconds";
pub const BUILTIN_DATE_PROTOTYPE_GET_MILLISECONDS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getMilliseconds";
pub const BUILTIN_DATE_PROTOTYPE_GET_UTC_MILLISECONDS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getUTCMilliseconds";
pub const BUILTIN_DATE_PROTOTYPE_GET_TIMEZONE_OFFSET_FUNCTION_ID: &str =
    "$builtin.Date.prototype.getTimezoneOffset";
pub const BUILTIN_DATE_PROTOTYPE_GET_YEAR_FUNCTION_ID: &str = "$builtin.Date.prototype.getYear";
pub const BUILTIN_DATE_PROTOTYPE_SET_YEAR_FUNCTION_ID: &str = "$builtin.Date.prototype.setYear";
pub const BUILTIN_DATE_PROTOTYPE_SET_FULL_YEAR_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setFullYear";
pub const BUILTIN_DATE_PROTOTYPE_SET_UTC_FULL_YEAR_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setUTCFullYear";
pub const BUILTIN_DATE_PROTOTYPE_SET_MONTH_FUNCTION_ID: &str = "$builtin.Date.prototype.setMonth";
pub const BUILTIN_DATE_PROTOTYPE_SET_UTC_MONTH_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setUTCMonth";
pub const BUILTIN_DATE_PROTOTYPE_SET_DATE_FUNCTION_ID: &str = "$builtin.Date.prototype.setDate";
pub const BUILTIN_DATE_PROTOTYPE_SET_UTC_DATE_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setUTCDate";
pub const BUILTIN_DATE_PROTOTYPE_SET_HOURS_FUNCTION_ID: &str = "$builtin.Date.prototype.setHours";
pub const BUILTIN_DATE_PROTOTYPE_SET_UTC_HOURS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setUTCHours";
pub const BUILTIN_DATE_PROTOTYPE_SET_MINUTES_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setMinutes";
pub const BUILTIN_DATE_PROTOTYPE_SET_UTC_MINUTES_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setUTCMinutes";
pub const BUILTIN_DATE_PROTOTYPE_SET_SECONDS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setSeconds";
pub const BUILTIN_DATE_PROTOTYPE_SET_UTC_SECONDS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setUTCSeconds";
pub const BUILTIN_DATE_PROTOTYPE_SET_MILLISECONDS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setMilliseconds";
pub const BUILTIN_DATE_PROTOTYPE_SET_UTC_MILLISECONDS_FUNCTION_ID: &str =
    "$builtin.Date.prototype.setUTCMilliseconds";
pub const BUILTIN_DATE_PROTOTYPE_TO_UTC_STRING_FUNCTION_ID: &str =
    "$builtin.Date.prototype.toUTCString";
pub const BUILTIN_REGEXP_FUNCTION_ID: &str = "$builtin.RegExp";
pub const BUILTIN_REGEXP_SPECIES_GETTER_FUNCTION_ID: &str = "$builtin.RegExp[Symbol.species].get";
pub const BUILTIN_REGEXP_PROTOTYPE_FLAGS_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.flags.get";
pub const BUILTIN_REGEXP_PROTOTYPE_SOURCE_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.source.get";
pub const BUILTIN_REGEXP_PROTOTYPE_HAS_INDICES_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.hasIndices.get";
pub const BUILTIN_REGEXP_PROTOTYPE_GLOBAL_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.global.get";
pub const BUILTIN_REGEXP_PROTOTYPE_IGNORE_CASE_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.ignoreCase.get";
pub const BUILTIN_REGEXP_PROTOTYPE_MULTILINE_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.multiline.get";
pub const BUILTIN_REGEXP_PROTOTYPE_DOT_ALL_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.dotAll.get";
pub const BUILTIN_REGEXP_PROTOTYPE_UNICODE_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.unicode.get";
pub const BUILTIN_REGEXP_PROTOTYPE_UNICODE_SETS_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.unicodeSets.get";
pub const BUILTIN_REGEXP_PROTOTYPE_STICKY_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.sticky.get";
pub const BUILTIN_REGEXP_LEGACY_STATIC_GETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.legacyStatic.get";
pub const BUILTIN_REGEXP_LEGACY_STATIC_SETTER_FUNCTION_ID: &str =
    "$builtin.RegExp.legacyStatic.set";
pub const BUILTIN_REGEXP_PROTOTYPE_EXEC_FUNCTION_ID: &str = "$builtin.RegExp.prototype.exec";
pub const BUILTIN_REGEXP_PROTOTYPE_TEST_FUNCTION_ID: &str = "$builtin.RegExp.prototype.test";
pub const BUILTIN_REGEXP_PROTOTYPE_COMPILE_FUNCTION_ID: &str = "$builtin.RegExp.prototype.compile";
pub const BUILTIN_REGEXP_PROTOTYPE_TO_STRING_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype.toString";
pub const BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype[Symbol.match]";
pub const BUILTIN_REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype[Symbol.matchAll]";
pub const BUILTIN_REGEXP_PROTOTYPE_SYMBOL_REPLACE_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype[Symbol.replace]";
pub const BUILTIN_REGEXP_PROTOTYPE_SYMBOL_SEARCH_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype[Symbol.search]";
pub const BUILTIN_REGEXP_PROTOTYPE_SYMBOL_SPLIT_FUNCTION_ID: &str =
    "$builtin.RegExp.prototype[Symbol.split]";
pub const BUILTIN_REGEXP_ESCAPE_FUNCTION_ID: &str = "$builtin.RegExp.escape";
pub const BUILTIN_JSON_PARSE_FUNCTION_ID: &str = "$builtin.JSON.parse";
pub const BUILTIN_JSON_STRINGIFY_FUNCTION_ID: &str = "$builtin.JSON.stringify";
pub const BUILTIN_JSON_RAW_JSON_FUNCTION_ID: &str = "$builtin.JSON.rawJSON";
pub const BUILTIN_JSON_IS_RAW_JSON_FUNCTION_ID: &str = "$builtin.JSON.isRawJSON";
pub const BUILTIN_ATOMICS_ADD_FUNCTION_ID: &str = "$builtin.Atomics.add";
pub const BUILTIN_ATOMICS_AND_FUNCTION_ID: &str = "$builtin.Atomics.and";
pub const BUILTIN_ATOMICS_COMPARE_EXCHANGE_FUNCTION_ID: &str = "$builtin.Atomics.compareExchange";
pub const BUILTIN_ATOMICS_EXCHANGE_FUNCTION_ID: &str = "$builtin.Atomics.exchange";
pub const BUILTIN_ATOMICS_LOAD_FUNCTION_ID: &str = "$builtin.Atomics.load";
pub const BUILTIN_ATOMICS_NOTIFY_FUNCTION_ID: &str = "$builtin.Atomics.notify";
pub const BUILTIN_ATOMICS_OR_FUNCTION_ID: &str = "$builtin.Atomics.or";
pub const BUILTIN_ATOMICS_PAUSE_FUNCTION_ID: &str = "$builtin.Atomics.pause";
pub const BUILTIN_ATOMICS_STORE_FUNCTION_ID: &str = "$builtin.Atomics.store";
pub const BUILTIN_ATOMICS_SUB_FUNCTION_ID: &str = "$builtin.Atomics.sub";
pub const BUILTIN_ATOMICS_WAIT_FUNCTION_ID: &str = "$builtin.Atomics.wait";
pub const BUILTIN_ATOMICS_WAIT_ASYNC_FUNCTION_ID: &str = "$builtin.Atomics.waitAsync";
pub const BUILTIN_ATOMICS_XOR_FUNCTION_ID: &str = "$builtin.Atomics.xor";
pub const BUILTIN_ATOMICS_IS_LOCK_FREE_FUNCTION_ID: &str = "$builtin.Atomics.isLockFree";
pub const BUILTIN_FLOAT64_ARRAY_FUNCTION_ID: &str = "$builtin.Float64Array";
pub const BUILTIN_FLOAT32_ARRAY_FUNCTION_ID: &str = "$builtin.Float32Array";
pub const BUILTIN_INT32_ARRAY_FUNCTION_ID: &str = "$builtin.Int32Array";
pub const BUILTIN_INT16_ARRAY_FUNCTION_ID: &str = "$builtin.Int16Array";
pub const BUILTIN_INT8_ARRAY_FUNCTION_ID: &str = "$builtin.Int8Array";
pub const BUILTIN_UINT32_ARRAY_FUNCTION_ID: &str = "$builtin.Uint32Array";
pub const BUILTIN_UINT16_ARRAY_FUNCTION_ID: &str = "$builtin.Uint16Array";
pub const BUILTIN_UINT8_ARRAY_FUNCTION_ID: &str = "$builtin.Uint8Array";
pub const BUILTIN_UINT8_CLAMPED_ARRAY_FUNCTION_ID: &str = "$builtin.Uint8ClampedArray";
pub const BUILTIN_BIGINT64_ARRAY_FUNCTION_ID: &str = "$builtin.BigInt64Array";
pub const BUILTIN_BIGUINT64_ARRAY_FUNCTION_ID: &str = "$builtin.BigUint64Array";
pub const BUILTIN_BIGINT_FUNCTION_ID: &str = "$builtin.BigInt";
pub const BUILTIN_BIGINT_AS_INT_N_FUNCTION_ID: &str = "$builtin.BigInt.asIntN";
pub const BUILTIN_BIGINT_AS_UINT_N_FUNCTION_ID: &str = "$builtin.BigInt.asUintN";
pub const BUILTIN_BIGINT_PROTOTYPE_TO_STRING_FUNCTION_ID: &str =
    "$builtin.BigInt.prototype.toString";
pub const BUILTIN_BIGINT_PROTOTYPE_TO_LOCALE_STRING_FUNCTION_ID: &str =
    "$builtin.BigInt.prototype.toLocaleString";
pub const BUILTIN_BIGINT_PROTOTYPE_VALUE_OF_FUNCTION_ID: &str = "$builtin.BigInt.prototype.valueOf";
pub const BUILTIN_SYMBOL_FUNCTION_ID: &str = "$builtin.Symbol";
pub const BUILTIN_SYMBOL_FOR_FUNCTION_ID: &str = "$builtin.Symbol.for";
pub const BUILTIN_SYMBOL_KEY_FOR_FUNCTION_ID: &str = "$builtin.Symbol.keyFor";
pub const BUILTIN_NUMBER_FUNCTION_ID: &str = "$builtin.Number";
pub const BUILTIN_NUMBER_IS_INTEGER_FUNCTION_ID: &str = "$builtin.Number.isInteger";
pub const BUILTIN_STRING_FUNCTION_ID: &str = "$builtin.String";
pub const BUILTIN_STRING_FROM_CHAR_CODE_FUNCTION_ID: &str = "$builtin.String.fromCharCode";
pub const BUILTIN_STRING_FROM_CODE_POINT_FUNCTION_ID: &str = "$builtin.String.fromCodePoint";
pub const BUILTIN_STRING_RAW_FUNCTION_ID: &str = "$builtin.String.raw";
pub const BUILTIN_STRING_PROTOTYPE_TO_STRING_FUNCTION_ID: &str =
    "$builtin.String.prototype.toString";
pub const BUILTIN_STRING_PROTOTYPE_VALUE_OF_FUNCTION_ID: &str = "$builtin.String.prototype.valueOf";
pub const BUILTIN_STRING_PROTOTYPE_CHAR_AT_FUNCTION_ID: &str = "$builtin.String.prototype.charAt";
pub const BUILTIN_STRING_PROTOTYPE_CONCAT_FUNCTION_ID: &str = "$builtin.String.prototype.concat";
pub const BUILTIN_STRING_PROTOTYPE_CHAR_CODE_AT_FUNCTION_ID: &str =
    "$builtin.String.prototype.charCodeAt";
pub const BUILTIN_STRING_PROTOTYPE_CODE_POINT_AT_FUNCTION_ID: &str =
    "$builtin.String.prototype.codePointAt";
pub const BUILTIN_STRING_PROTOTYPE_AT_FUNCTION_ID: &str = "$builtin.String.prototype.at";
pub const BUILTIN_STRING_PROTOTYPE_ANCHOR_FUNCTION_ID: &str = "$builtin.String.prototype.anchor";
pub const BUILTIN_STRING_PROTOTYPE_BIG_FUNCTION_ID: &str = "$builtin.String.prototype.big";
pub const BUILTIN_STRING_PROTOTYPE_BLINK_FUNCTION_ID: &str = "$builtin.String.prototype.blink";
pub const BUILTIN_STRING_PROTOTYPE_BOLD_FUNCTION_ID: &str = "$builtin.String.prototype.bold";
pub const BUILTIN_STRING_PROTOTYPE_FIXED_FUNCTION_ID: &str = "$builtin.String.prototype.fixed";
pub const BUILTIN_STRING_PROTOTYPE_FONTCOLOR_FUNCTION_ID: &str =
    "$builtin.String.prototype.fontcolor";
pub const BUILTIN_STRING_PROTOTYPE_FONTSIZE_FUNCTION_ID: &str =
    "$builtin.String.prototype.fontsize";
pub const BUILTIN_STRING_PROTOTYPE_ITALICS_FUNCTION_ID: &str = "$builtin.String.prototype.italics";
pub const BUILTIN_STRING_PROTOTYPE_LINK_FUNCTION_ID: &str = "$builtin.String.prototype.link";
pub const BUILTIN_STRING_PROTOTYPE_SMALL_FUNCTION_ID: &str = "$builtin.String.prototype.small";
pub const BUILTIN_STRING_PROTOTYPE_STRIKE_FUNCTION_ID: &str = "$builtin.String.prototype.strike";
pub const BUILTIN_STRING_PROTOTYPE_SUB_FUNCTION_ID: &str = "$builtin.String.prototype.sub";
pub const BUILTIN_STRING_PROTOTYPE_SUBSTR_FUNCTION_ID: &str = "$builtin.String.prototype.substr";
pub const BUILTIN_STRING_PROTOTYPE_SUBSTRING_FUNCTION_ID: &str =
    "$builtin.String.prototype.substring";
pub const BUILTIN_STRING_PROTOTYPE_SUP_FUNCTION_ID: &str = "$builtin.String.prototype.sup";
pub const BUILTIN_STRING_PROTOTYPE_MATCH_FUNCTION_ID: &str = "$builtin.String.prototype.match";
pub const BUILTIN_STRING_PROTOTYPE_MATCH_ALL_FUNCTION_ID: &str =
    "$builtin.String.prototype.matchAll";
pub const BUILTIN_STRING_PROTOTYPE_REPLACE_FUNCTION_ID: &str = "$builtin.String.prototype.replace";
pub const BUILTIN_STRING_PROTOTYPE_REPLACE_ALL_FUNCTION_ID: &str =
    "$builtin.String.prototype.replaceAll";
pub const BUILTIN_STRING_PROTOTYPE_SEARCH_FUNCTION_ID: &str = "$builtin.String.prototype.search";
pub const BUILTIN_STRING_PROTOTYPE_INDEX_OF_FUNCTION_ID: &str = "$builtin.String.prototype.indexOf";
pub const BUILTIN_STRING_PROTOTYPE_LAST_INDEX_OF_FUNCTION_ID: &str =
    "$builtin.String.prototype.lastIndexOf";
pub const BUILTIN_STRING_PROTOTYPE_SLICE_FUNCTION_ID: &str = "$builtin.String.prototype.slice";
pub const BUILTIN_STRING_PROTOTYPE_SPLIT_FUNCTION_ID: &str = "$builtin.String.prototype.split";
pub const BUILTIN_STRING_PROTOTYPE_PAD_START_FUNCTION_ID: &str =
    "$builtin.String.prototype.padStart";
pub const BUILTIN_STRING_PROTOTYPE_PAD_END_FUNCTION_ID: &str = "$builtin.String.prototype.padEnd";
pub const BUILTIN_STRING_PROTOTYPE_REPEAT_FUNCTION_ID: &str = "$builtin.String.prototype.repeat";
pub const BUILTIN_STRING_PROTOTYPE_ENDS_WITH_FUNCTION_ID: &str =
    "$builtin.String.prototype.endsWith";
pub const BUILTIN_STRING_PROTOTYPE_INCLUDES_FUNCTION_ID: &str =
    "$builtin.String.prototype.includes";
pub const BUILTIN_STRING_PROTOTYPE_STARTS_WITH_FUNCTION_ID: &str =
    "$builtin.String.prototype.startsWith";
pub const BUILTIN_STRING_PROTOTYPE_NORMALIZE_FUNCTION_ID: &str =
    "$builtin.String.prototype.normalize";
pub const BUILTIN_STRING_PROTOTYPE_LOCALE_COMPARE_FUNCTION_ID: &str =
    "$builtin.String.prototype.localeCompare";
pub const BUILTIN_STRING_PROTOTYPE_ITERATOR_FUNCTION_ID: &str =
    "$builtin.String.prototype[Symbol.iterator]";
pub const BUILTIN_STRING_PROTOTYPE_TO_UPPER_CASE_FUNCTION_ID: &str =
    "$builtin.String.prototype.toUpperCase";
pub const BUILTIN_STRING_PROTOTYPE_TO_LOWER_CASE_FUNCTION_ID: &str =
    "$builtin.String.prototype.toLowerCase";
pub const BUILTIN_STRING_PROTOTYPE_TO_LOCALE_LOWER_CASE_FUNCTION_ID: &str =
    "$builtin.String.prototype.toLocaleLowerCase";
pub const BUILTIN_STRING_PROTOTYPE_TO_LOCALE_UPPER_CASE_FUNCTION_ID: &str =
    "$builtin.String.prototype.toLocaleUpperCase";
pub const BUILTIN_STRING_PROTOTYPE_TRIM_FUNCTION_ID: &str = "$builtin.String.prototype.trim";
pub const BUILTIN_STRING_PROTOTYPE_TRIM_START_FUNCTION_ID: &str =
    "$builtin.String.prototype.trimStart";
pub const BUILTIN_STRING_PROTOTYPE_TRIM_END_FUNCTION_ID: &str = "$builtin.String.prototype.trimEnd";
pub const BUILTIN_STRING_PROTOTYPE_IS_WELL_FORMED_FUNCTION_ID: &str =
    "$builtin.String.prototype.isWellFormed";
pub const BUILTIN_STRING_PROTOTYPE_TO_WELL_FORMED_FUNCTION_ID: &str =
    "$builtin.String.prototype.toWellFormed";
pub const BUILTIN_BOOLEAN_FUNCTION_ID: &str = "$builtin.Boolean";
pub const BUILTIN_ERROR_FUNCTION_ID: &str = "$builtin.Error";
pub const BUILTIN_ERROR_IS_ERROR_FUNCTION_ID: &str = "$builtin.Error.isError";
pub const BUILTIN_EVAL_ERROR_FUNCTION_ID: &str = "$builtin.EvalError";
pub const BUILTIN_AGGREGATE_ERROR_FUNCTION_ID: &str = "$builtin.AggregateError";
pub const BUILTIN_SUPPRESSED_ERROR_FUNCTION_ID: &str = "$builtin.SuppressedError";
pub const BUILTIN_RANGE_ERROR_FUNCTION_ID: &str = "$builtin.RangeError";
pub const BUILTIN_SYNTAX_ERROR_FUNCTION_ID: &str = "$builtin.SyntaxError";
pub const BUILTIN_TYPE_ERROR_FUNCTION_ID: &str = "$builtin.TypeError";
pub const BUILTIN_URI_ERROR_FUNCTION_ID: &str = "$builtin.URIError";
pub const BUILTIN_REFERENCE_ERROR_FUNCTION_ID: &str = "$builtin.ReferenceError";
pub const BUILTIN_ERROR_PROTOTYPE_TO_STRING_FUNCTION_ID: &str = "$builtin.Error.prototype.toString";
pub const BUILTIN_THROW_TYPE_ERROR_FUNCTION_ID: &str = "$builtin.%ThrowTypeError%";
pub const BUILTIN_BOUND_FUNCTION_INVOKER_FUNCTION_ID: &str = "$builtin.[[BoundFunctionInvoke]]";
pub const BUILTIN_ESCAPE_FUNCTION_ID: &str = "$builtin.escape";
pub const BUILTIN_UNESCAPE_FUNCTION_ID: &str = "$builtin.unescape";
pub const ARRAY_BUFFER_DATA_PTR_SLOT: &str = "$ArrayBufferDataPtr";
pub const ARRAY_BUFFER_BYTE_LENGTH_SLOT: &str = "$ArrayBufferByteLength";
pub const DATA_VIEW_DATA_PTR_SLOT: &str = "$DataViewDataPtr";
pub const DATA_VIEW_BYTE_OFFSET_SLOT: &str = "$DataViewByteOffset";
pub const DATA_VIEW_BYTE_LENGTH_SLOT: &str = "$DataViewByteLength";
pub const DATA_VIEW_LENGTH_TRACKING_SLOT: &str = "$DataViewLengthTracking";
pub const TYPED_ARRAY_VIEWED_ARRAY_BUFFER_SLOT: &str = "$TypedArrayViewedArrayBuffer";
pub const TYPED_ARRAY_BYTE_OFFSET_SLOT: &str = "$TypedArrayByteOffset";
pub const TYPED_ARRAY_BYTE_LENGTH_SLOT: &str = "$TypedArrayByteLength";
pub const TYPED_ARRAY_BYTES_PER_ELEMENT_SLOT: &str = "$TypedArrayBytesPerElement";
pub const TYPED_ARRAY_ELEMENT_KIND_SLOT: &str = "$TypedArrayElementKind";
pub const TYPED_ARRAY_LENGTH_TRACKING_SLOT: &str = "$TypedArrayLengthTracking";
pub const PORFFOR_GENERATOR_THROW_SLOT: &str = "$PorfforGeneratorThrow";
pub const PORFFOR_ITERATOR_FROM_WRAPPER_SLOT: &str = "$PorfforIteratorFromWrapper";
pub const PORFFOR_YIELD_STAR_GENERATOR_SLOT: &str = "$PorfforYieldStarGenerator";
pub const PORFFOR_YIELD_STAR_RETURN_NON_OBJECT_SLOT: &str = "$PorfforYieldStarReturnNonObject";
pub const PORFFOR_YIELD_STAR_THROW_NON_OBJECT_SLOT: &str = "$PorfforYieldStarThrowNonObject";
pub const PORFFOR_STATIC_GENERATOR_VALUES_METHOD: &str = "$PorfforStaticGenerator.values";
pub const PORFFOR_STATIC_GENERATOR_ITERATOR_SLOT: &str = "$PorfforStaticGeneratorIterator";
pub const DATE_VALUE_SLOT: &str = "$DateValue";
