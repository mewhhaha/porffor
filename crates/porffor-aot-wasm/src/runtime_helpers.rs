//! The ordered registry of shared runtime-helper functions.
//!
//! Before this module the same 33-element list existed four times: once in the
//! `FunctionSection` (emission of the type index), once in the `CodeSection`
//! (emission of the body), once as 27 hand-written `base + N` accessors on
//! [`FunctionBuilder`](crate::emit::FunctionBuilder), and once as a literal
//! `27` in `debug_dump`. The literal had already drifted — the counted truth is
//! 32 unconditional helpers plus one conditional one — because nothing forced
//! the four copies to agree.
//!
//! Now the enum *is* the order. `RuntimeHelperId as u32` is the offset from the
//! first helper's Wasm function index, [`RuntimeHelperId::ALL`] is asserted at
//! compile time to be in declaration order, and both [`RuntimeHelperId::type_index`]
//! and [`RuntimeHelperId::is_emitted`] are exhaustive matches with no `_` arm,
//! so adding a helper fails to build until its Wasm type and its emission
//! condition are both stated.

use crate::module::{
    ARRAY_ALLOC_TYPE_INDEX, FUNCTION_OBJECT_ALLOC_TYPE_INDEX, HEAP_ALLOC_TYPE_INDEX,
    JS_FUNCTION_TYPE_INDEX, OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX,
    OBJECT_APPEND_DATA_PROPERTY_TYPE_INDEX, PLAIN_OBJECT_ALLOC_TYPE_INDEX,
};

/// The facts a module knows about itself that decide whether a *conditional*
/// runtime helper is emitted at all.
///
/// Passed by value into [`RuntimeHelperId::is_emitted`] so that the emission
/// condition of every helper is stated in exactly one place, next to its type
/// index, rather than being spread over an `if` in the function section and a
/// matching `if let Some(..)` in the code section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeHelperEmission {
    pub(crate) uses_json_stringify: bool,
}

impl RuntimeHelperEmission {
    /// The emission context in which every conditional helper is absent. Used
    /// by [`RuntimeHelperId::is_conditional`] so "conditional" is derived from
    /// `is_emitted` instead of being a second, separately-maintained match.
    pub(crate) const NONE: Self = Self {
        uses_json_stringify: false,
    };
}

/// Every shared runtime helper, in the order its body is written into the code
/// section.
///
/// Declaration order is load-bearing twice over:
///
/// * [`RuntimeHelperId::index`] is `base + self as u32`, so a variant moved in
///   this list moves the Wasm function index every call site uses.
/// * A conditional helper must come last (see [`Self::conditional_helpers_are_last`]),
///   because a helper emitted after a skipped one would silently shift down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub(crate) enum RuntimeHelperId {
    // The six allocation helpers are plain free functions (`emit_*_helper_function`)
    // rather than `FunctionBuilder` bodies, but they occupy the first six slots.
    HeapAlloc = 0,
    ObjectAppendDataProperty = 1,
    ObjectAppendAccessorProperty = 2,
    FunctionObjectAlloc = 3,
    PlainObjectAlloc = 4,
    ArrayAlloc = 5,
    ObjectRead = 6,
    ObjectWrite = 7,
    ObjectDefineData = 8,
    ProxyCall = 9,
    ProxyConstruct = 10,
    StringEquality = 11,
    NumberToString = 12,
    StringToNumber = 13,
    ValueToString = 14,
    ValueToNumber = 15,
    ValueToNumeric = 16,
    ObjectGetPrototypeOf = 17,
    ObjectIsExtensible = 18,
    ObjectReadProxy = 19,
    RegExpMatcher = 20,
    FunctionCall = 21,
    DynamicPropertyRead = 22,
    OrdinarySetDataOnReceiver = 23,
    OrdinarySetDataOnReceiverWithFallback = 24,
    ArrayWrite = 25,
    OrdinarySet = 26,
    OrdinarySetWithoutReceiverFallback = 27,
    DecimalToBinary64 = 28,
    BigIntArithmetic = 29,
    TemporalCalendarIsoDateProbe = 30,
    TemporalCalendarIdentifier = 31,
    /// Only helper whose emission is conditional today. Keep conditional
    /// helpers last; `conditional_helpers_are_last` is a compile-time check,
    /// not a comment.
    JsonStringifyValue = 32,
}

impl RuntimeHelperId {
    /// Every helper, in emission order. Asserted below to be exactly the
    /// declaration order, so `ALL[i] as u32 == i`.
    pub(crate) const ALL: [Self; 33] = [
        Self::HeapAlloc,
        Self::ObjectAppendDataProperty,
        Self::ObjectAppendAccessorProperty,
        Self::FunctionObjectAlloc,
        Self::PlainObjectAlloc,
        Self::ArrayAlloc,
        Self::ObjectRead,
        Self::ObjectWrite,
        Self::ObjectDefineData,
        Self::ProxyCall,
        Self::ProxyConstruct,
        Self::StringEquality,
        Self::NumberToString,
        Self::StringToNumber,
        Self::ValueToString,
        Self::ValueToNumber,
        Self::ValueToNumeric,
        Self::ObjectGetPrototypeOf,
        Self::ObjectIsExtensible,
        Self::ObjectReadProxy,
        Self::RegExpMatcher,
        Self::FunctionCall,
        Self::DynamicPropertyRead,
        Self::OrdinarySetDataOnReceiver,
        Self::OrdinarySetDataOnReceiverWithFallback,
        Self::ArrayWrite,
        Self::OrdinarySet,
        Self::OrdinarySetWithoutReceiverFallback,
        Self::DecimalToBinary64,
        Self::BigIntArithmetic,
        Self::TemporalCalendarIsoDateProbe,
        Self::TemporalCalendarIdentifier,
        Self::JsonStringifyValue,
    ];

    /// The Wasm function index of this helper, given the index of the first
    /// helper (`heap_alloc_function_index`).
    ///
    /// This is the *only* way to turn a helper into an index. The 27 accessors
    /// on `FunctionBuilder` used to spell `base + 6` .. `base + 32` by hand,
    /// which is exactly the arithmetic that silently rebinds every call site
    /// when a helper is inserted.
    pub(crate) const fn index(self, base: u32) -> u32 {
        base + self as u32
    }

    /// The Wasm type index this helper's signature is declared with, matching
    /// the entry the function section writes for it.
    pub(crate) const fn type_index(self) -> u32 {
        match self {
            Self::HeapAlloc => HEAP_ALLOC_TYPE_INDEX,
            Self::ObjectAppendDataProperty => OBJECT_APPEND_DATA_PROPERTY_TYPE_INDEX,
            Self::ObjectAppendAccessorProperty => OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX,
            Self::FunctionObjectAlloc => FUNCTION_OBJECT_ALLOC_TYPE_INDEX,
            Self::PlainObjectAlloc => PLAIN_OBJECT_ALLOC_TYPE_INDEX,
            Self::ArrayAlloc => ARRAY_ALLOC_TYPE_INDEX,
            // The object-define-data helper takes seven i64 params and returns
            // nothing, which is the accessor-append signature.
            Self::ObjectDefineData => OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX,
            Self::ObjectRead
            | Self::ObjectWrite
            | Self::ProxyCall
            | Self::ProxyConstruct
            | Self::StringEquality
            | Self::NumberToString
            | Self::StringToNumber
            | Self::ValueToString
            | Self::ValueToNumber
            | Self::ValueToNumeric
            | Self::ObjectGetPrototypeOf
            | Self::ObjectIsExtensible
            | Self::ObjectReadProxy
            | Self::RegExpMatcher
            | Self::FunctionCall
            | Self::DynamicPropertyRead
            | Self::OrdinarySetDataOnReceiver
            | Self::OrdinarySetDataOnReceiverWithFallback
            | Self::ArrayWrite
            | Self::OrdinarySet
            | Self::OrdinarySetWithoutReceiverFallback
            | Self::DecimalToBinary64
            | Self::BigIntArithmetic
            | Self::TemporalCalendarIsoDateProbe
            | Self::TemporalCalendarIdentifier
            | Self::JsonStringifyValue => JS_FUNCTION_TYPE_INDEX,
        }
    }

    /// Whether this helper's body is written into this module at all.
    ///
    /// Exhaustive with no `_` arm: a new helper cannot compile until it says
    /// whether it is unconditional or what makes it conditional.
    pub(crate) const fn is_emitted(self, emission: RuntimeHelperEmission) -> bool {
        match self {
            Self::HeapAlloc
            | Self::ObjectAppendDataProperty
            | Self::ObjectAppendAccessorProperty
            | Self::FunctionObjectAlloc
            | Self::PlainObjectAlloc
            | Self::ArrayAlloc
            | Self::ObjectRead
            | Self::ObjectWrite
            | Self::ObjectDefineData
            | Self::ProxyCall
            | Self::ProxyConstruct
            | Self::StringEquality
            | Self::NumberToString
            | Self::StringToNumber
            | Self::ValueToString
            | Self::ValueToNumber
            | Self::ValueToNumeric
            | Self::ObjectGetPrototypeOf
            | Self::ObjectIsExtensible
            | Self::ObjectReadProxy
            | Self::RegExpMatcher
            | Self::FunctionCall
            | Self::DynamicPropertyRead
            | Self::OrdinarySetDataOnReceiver
            | Self::OrdinarySetDataOnReceiverWithFallback
            | Self::ArrayWrite
            | Self::OrdinarySet
            | Self::OrdinarySetWithoutReceiverFallback
            | Self::DecimalToBinary64
            | Self::BigIntArithmetic
            | Self::TemporalCalendarIsoDateProbe
            | Self::TemporalCalendarIdentifier => true,
            Self::JsonStringifyValue => emission.uses_json_stringify,
        }
    }

    /// Derived from [`Self::is_emitted`] rather than stated separately, so the
    /// two can never disagree: a helper is conditional exactly when some
    /// emission context omits it.
    pub(crate) const fn is_conditional(self) -> bool {
        !self.is_emitted(RuntimeHelperEmission::NONE)
    }

    /// Stable symbol fragment used for the Wasm `name` section and for the
    /// emitted-size report.
    pub(crate) const fn debug_name(self) -> &'static str {
        match self {
            Self::HeapAlloc => "heap_alloc",
            Self::ObjectAppendDataProperty => "object_append_data_property",
            Self::ObjectAppendAccessorProperty => "object_append_accessor_property",
            Self::FunctionObjectAlloc => "function_object_alloc",
            Self::PlainObjectAlloc => "plain_object_alloc",
            Self::ArrayAlloc => "array_alloc",
            Self::ObjectRead => "object_read",
            Self::ObjectWrite => "object_write",
            Self::ObjectDefineData => "object_define_data",
            Self::ProxyCall => "proxy_call",
            Self::ProxyConstruct => "proxy_construct",
            Self::StringEquality => "string_equality",
            Self::NumberToString => "number_to_string",
            Self::StringToNumber => "string_to_number",
            Self::ValueToString => "value_to_string",
            Self::ValueToNumber => "value_to_number",
            Self::ValueToNumeric => "value_to_numeric",
            Self::ObjectGetPrototypeOf => "object_get_prototype_of",
            Self::ObjectIsExtensible => "object_is_extensible",
            Self::ObjectReadProxy => "object_read_proxy",
            Self::RegExpMatcher => "regexp_matcher",
            Self::FunctionCall => "function_call",
            Self::DynamicPropertyRead => "dynamic_property_read",
            Self::OrdinarySetDataOnReceiver => "ordinary_set_data_on_receiver",
            Self::OrdinarySetDataOnReceiverWithFallback => {
                "ordinary_set_data_on_receiver_with_fallback"
            }
            Self::ArrayWrite => "array_write",
            Self::OrdinarySet => "ordinary_set",
            Self::OrdinarySetWithoutReceiverFallback => "ordinary_set_without_receiver_fallback",
            Self::DecimalToBinary64 => "decimal_to_binary64",
            Self::BigIntArithmetic => "bigint_arithmetic",
            Self::TemporalCalendarIsoDateProbe => "temporal_calendar_iso_date_probe",
            Self::TemporalCalendarIdentifier => "temporal_calendar_identifier",
            Self::JsonStringifyValue => "json_stringify_value",
        }
    }

    /// `ALL` must be the declaration order, because `index()` derives the Wasm
    /// function index from `self as u32` while the function and code sections
    /// are generated by walking `ALL`.
    const fn all_is_declaration_ordered() -> bool {
        let mut position = 0;
        while position < Self::ALL.len() {
            if Self::ALL[position] as u32 != position as u32 {
                return false;
            }
            position += 1;
        }
        true
    }

    /// `index()` is `base + self as u32`, which is only the real Wasm index
    /// when no emitted helper is preceded by a skipped one. Keeping every
    /// conditional helper at the end of the list is what makes that true, and
    /// this is the check that used to be the comment "their fixed offsets never
    /// shift".
    const fn conditional_helpers_are_last() -> bool {
        let mut position = 0;
        let mut seen_conditional = false;
        while position < Self::ALL.len() {
            let helper = Self::ALL[position];
            if helper.is_conditional() {
                seen_conditional = true;
            } else if seen_conditional {
                return false;
            }
            position += 1;
        }
        true
    }
}

const _: () = assert!(
    RuntimeHelperId::all_is_declaration_ordered(),
    "RuntimeHelperId::ALL must list every helper exactly once, in declaration order"
);

const _: () = assert!(
    RuntimeHelperId::conditional_helpers_are_last(),
    "an unconditionally emitted runtime helper must not follow a conditional one: \
     RuntimeHelperId::index() would hand out an index that shifts when the \
     conditional helper is skipped"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_are_dense_from_the_base() {
        for (position, helper) in RuntimeHelperId::ALL.iter().enumerate() {
            assert_eq!(helper.index(100), 100 + position as u32);
        }
    }

    #[test]
    fn debug_names_are_unique() {
        let mut names = std::collections::BTreeSet::new();
        for helper in RuntimeHelperId::ALL {
            assert!(
                names.insert(helper.debug_name()),
                "duplicate runtime helper name {}",
                helper.debug_name()
            );
        }
        assert_eq!(names.len(), RuntimeHelperId::ALL.len());
    }

    #[test]
    fn json_stringify_is_the_only_conditional_helper() {
        let conditional = RuntimeHelperId::ALL
            .iter()
            .copied()
            .filter(|helper| helper.is_conditional())
            .collect::<Vec<_>>();
        assert_eq!(conditional, vec![RuntimeHelperId::JsonStringifyValue]);
    }

    #[test]
    fn emitted_count_matches_the_counted_truth() {
        // 32 unconditional helpers plus JSON.stringify's value helper. The
        // `debug_dump` line used to hard-code 27 and had drifted by five.
        let without_json = RuntimeHelperId::ALL
            .iter()
            .filter(|helper| helper.is_emitted(RuntimeHelperEmission::NONE))
            .count();
        let with_json = RuntimeHelperId::ALL
            .iter()
            .filter(|helper| {
                helper.is_emitted(RuntimeHelperEmission {
                    uses_json_stringify: true,
                })
            })
            .count();
        assert_eq!(without_json, 32);
        assert_eq!(with_json, 33);
    }
}
