use super::*;

/// The two addresses by which the backend reaches one error prototype.
///
/// Keeping the global and per-realm locations in one row prevents the two
/// independently-maintained maps from assigning an error kind to different
/// prototypes. The exhaustive match is the authority: adding a
/// `NativeErrorKind` without assigning both locations is a compile error.
#[derive(Clone, Copy)]
struct ErrorPrototypeLocation {
    global_index: u32,
    realm_offset: u64,
}

const fn error_prototype_location(kind: NativeErrorKind) -> ErrorPrototypeLocation {
    let (global_index, realm_offset) = match kind {
        NativeErrorKind::Error => (
            ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_ERROR_PROTOTYPE_OFFSET,
        ),
        NativeErrorKind::EvalError => (
            EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_EVAL_ERROR_PROTOTYPE_OFFSET,
        ),
        NativeErrorKind::RangeError => (
            RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET,
        ),
        NativeErrorKind::ReferenceError => (
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_REFERENCE_ERROR_PROTOTYPE_OFFSET,
        ),
        NativeErrorKind::SyntaxError => (
            SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_SYNTAX_ERROR_PROTOTYPE_OFFSET,
        ),
        NativeErrorKind::TypeError => (
            TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
        ),
        NativeErrorKind::URIError => (
            URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_URI_ERROR_PROTOTYPE_OFFSET,
        ),
        NativeErrorKind::AggregateError => (
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
        ),
        NativeErrorKind::SuppressedError => (
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET,
        ),
    };
    ErrorPrototypeLocation {
        global_index,
        realm_offset,
    }
}

pub(crate) const fn error_prototype_global_index(kind: NativeErrorKind) -> u32 {
    error_prototype_location(kind).global_index
}

pub(crate) const fn error_realm_prototype_offset(kind: NativeErrorKind) -> u64 {
    error_prototype_location(kind).realm_offset
}

pub(crate) fn error_realm_prototype_entries() -> [(NativeErrorKind, u32, u64); 9] {
    NativeErrorKind::ALL.map(|kind| {
        let location = error_prototype_location(kind);
        (kind, location.global_index, location.realm_offset)
    })
}
