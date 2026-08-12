//! The ordered registry of shared runtime-helper functions.
//!
//! Before this module the same list existed four times: once in the
//! `FunctionSection` (emission of the type index), once in the `CodeSection`
//! (emission of the body), once as 27 hand-written `base + N` accessors on
//! [`FunctionBuilder`](crate::emit::FunctionBuilder), and once as a literal
//! `27` in `debug_dump`. The literal had already drifted — the counted truth is
//! now 34 unconditional helpers plus one conditional one — because nothing
//! forced the four copies to agree.
//!
//! Now the enum *is* the order. `RuntimeHelperId as u32` is the offset from the
//! first helper's Wasm function index, [`RuntimeHelperId::ALL`] is asserted at
//! compile time to be in declaration order, and both [`RuntimeHelperId::type_index`]
//! and [`RuntimeHelperId::is_emitted`] are exhaustive matches with no `_` arm,
//! so adding a helper fails to build until its Wasm type and its emission
//! condition are both stated.

use lila_ir::ToPrimitiveHint;

use crate::module::{
    ARRAY_ALLOC_TYPE_INDEX, FUNCTION_OBJECT_ALLOC_TYPE_INDEX, HEAP_ALLOC_TYPE_INDEX,
    JS_FUNCTION_TYPE_INDEX, OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX,
    OBJECT_APPEND_DATA_PROPERTY_TYPE_INDEX, PLAIN_OBJECT_ALLOC_TYPE_INDEX,
};

/// One fact a module can carry about itself that some *conditional* runtime
/// helper's emission depends on.
///
/// Closed set, and the only way to put a fact into a [`RuntimeHelperEmission`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum RuntimeHelperFact {
    UsesJsonStringify = 0,
}

impl RuntimeHelperFact {
    const fn bit(self) -> u32 {
        1u32 << (self as u32)
    }
}

/// The facts a module knows about itself that decide whether a *conditional*
/// runtime helper is emitted at all.
///
/// Passed by value into [`RuntimeHelperId::is_emitted`] so that the emission
/// condition of every helper is stated in exactly one place, next to its type
/// index, rather than being spread over an `if` in the function section and a
/// matching `if let Some(..)` in the code section.
///
/// A *set* of [`RuntimeHelperFact`]s rather than a struct of `bool` fields,
/// because [`Self::NONE`] has to mean "no fact holds" by construction. As a
/// struct literal it was a hand-written list that whoever added the second
/// conditional helper would also have had to remember to extend with `false`;
/// writing `true` there instead compiles, makes [`RuntimeHelperId::is_conditional`]
/// answer `false` for a genuinely conditional helper, and silently defeats
/// [`RuntimeHelperId::conditional_helpers_are_last`] — handing out shifted
/// function indices with both `const` assertions still green. The empty set
/// cannot be written wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeHelperEmission(u32);

impl RuntimeHelperEmission {
    /// The emission context in which every conditional helper is absent. Used
    /// by [`RuntimeHelperId::is_conditional`] so "conditional" is derived from
    /// `is_emitted` instead of being a second, separately-maintained match.
    pub(crate) const NONE: Self = Self(0);

    /// Adds `fact` to the set when `holds`. The only constructor beyond
    /// [`Self::NONE`], so every emission context is built up from the empty one.
    pub(crate) const fn with(self, fact: RuntimeHelperFact, holds: bool) -> Self {
        if holds {
            Self(self.0 | fact.bit())
        } else {
            self
        }
    }

    pub(crate) const fn holds(self, fact: RuntimeHelperFact) -> bool {
        self.0 & fact.bit() != 0
    }
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
    /// The whole `expr[index]` *read* composite: Arguments / Array (with the
    /// prototype walk) / TypedArray element load / ordinary object key read,
    /// including the number→string key materialization for the object arm.
    ///
    /// **Retraction, batch 3.** This doc comment used to assert that this
    /// composite "is what pushed the `testIntl.js` `canonicalizeLanguageTag`
    /// body to 3,615,449 bytes". The *causal* half of that claim is refuted.
    /// The *predicted* replacement cause is [`Self::ValueToPrimitiveString`] /
    /// [`Self::ValueToPropertyKey`] — predicted, not established: see the note
    /// on those variants for the ratio argument, its falsifier and its known
    /// blind spot. What survives here without qualification is only that this
    /// composite is large per inline site; with this helper in place
    /// `helper::indexed_element_read` is 2,761 bytes, and
    /// `canonicalizeLanguageTag` was still *exactly* 3,615,449 bytes
    /// afterwards, which is what falsified the causal claim.
    ///
    /// Reproduce (no cargo needed, ~20 s per build):
    ///
    /// ```sh
    /// LILA_WASM_DUMP=/tmp/x.wasm ./target/debug/lila build wasm probe.js
    /// # then read the code-section body lengths back against the custom
    /// # `name` section this crate emits.
    /// ```
    IndexedElementRead = 32,
    /// The whole `expr[index] = value` *write* composite: TypedArray element
    /// store versus ordinary `[[Set]]`. 174,558 bytes per inline site.
    IndexedElementWrite = 33,
    /// `ToPrimitive(value)` with no hint — the `@@toPrimitive` / `valueOf` /
    /// `toString` hook chain plus the Array / Arguments / Function arms.
    ///
    /// Measured, on this tree, 2026-08-09, with `lila build wasm` +
    /// `LILA_WASM_DUMP` and the code-section/`name`-section reader described
    /// on [`Self::IndexedElementRead`]:
    ///
    /// | probe function body | bytes |
    /// |---|---|
    /// | `var A = k.split('-'); return A.length;` | 14,754 |
    /// | ... plus `var x = ''; x = A[0];` (static index) | 14,911 |
    /// | ... plus `var x = ''; x = A[i];` (dynamic key) | 87,101 |
    /// | ... plus a second `y = A[j];` | 159,811 |
    ///
    /// So one dynamic-key site costs `(159,811 - 14,754) / 2 = 72,528` bytes,
    /// and a `+` on two operands of unknown kind costs `140,144 / 2 = 70,072`
    /// bytes per operand — the same composite, reached through ToPropertyKey in
    /// the first case and directly in the second.
    ///
    /// The table above is measured. The attribution of
    /// `js::canonicalizeLanguageTag#f46` (3,615,449 bytes, 154 locals) to this
    /// composite is **a prediction, not a measurement**, and is recorded here
    /// with its falsifier so it cannot harden into folklore the way the
    /// [`Self::IndexedElementRead`] claim retracted above did.
    ///
    /// Predicted cause: five independent call-count ratios against the one-site
    /// probe agree on ~49 copies of this composite in that one body —
    /// `object_define_data 7130/144.5`, `string_equality 6352/130`,
    /// `plain_object_alloc 3565/72.5`, `function_call 1177/24`,
    /// `array_alloc 1225/26`. 49 x 72,347 is 98.0% of the body.
    ///
    /// **Falsifier:** the post-split size of `js::canonicalizeLanguageTag#f46`,
    /// read exactly as the table above was read. Predicted under 250,000 bytes.
    /// **Above 500,000 the attribution is wrong** and the residual must be
    /// re-attributed before anyone claims the split worked. That measurement has
    /// not been taken — no compiler ran in the session that wrote this.
    ///
    /// **Known blind spot of the ratio method:** a call-count ratio cannot
    /// separate a composite from the composites emitted *adjacent to it at the
    /// same sites*, because they scale together. That is precisely how batch 2
    /// misattributed these same 49 sites to [`Self::IndexedElementRead`]. The
    /// ratios are therefore evidence that ~49 *sites* dominate the body, and
    /// only weak evidence about which composite at those sites owns the bytes.
    ValueToPrimitiveDefault = 34,
    /// `ToPrimitive(value, number)`. Separate body rather than a runtime hint
    /// parameter: the hook order differs (`valueOf` before `toString`), so a
    /// shared body would have to branch on a value the call site already knows
    /// at compile time.
    ValueToPrimitiveNumber = 35,
    /// `ToPrimitive(value, string)` — `toString` before `valueOf`.
    ValueToPrimitiveString = 36,
    /// `ToPropertyKey(value)`: ToPrimitive with the string hint, then the
    /// symbol-marker/`ToString` split. Reached by every computed member access
    /// whose key is not statically known to be a String or a Symbol.
    ValueToPropertyKey = 37,
    /// Only helper whose emission is conditional today. Keep conditional
    /// helpers last; `conditional_helpers_are_last` is a compile-time check,
    /// not a comment.
    JsonStringifyValue = 38,
}

impl RuntimeHelperId {
    /// Every helper, in emission order. Asserted below to be exactly the
    /// declaration order, so `ALL[i] as u32 == i`.
    pub(crate) const ALL: [Self; 39] = [
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
        Self::IndexedElementRead,
        Self::IndexedElementWrite,
        Self::ValueToPrimitiveDefault,
        Self::ValueToPrimitiveNumber,
        Self::ValueToPrimitiveString,
        Self::ValueToPropertyKey,
        Self::JsonStringifyValue,
    ];

    /// The helper that implements `ToPrimitive` for `hint`.
    ///
    /// Exhaustive over the closed [`ToPrimitiveHint`] domain with no `_` arm: a
    /// fourth hint cannot compile until it names the body that serves it. This
    /// exists so the hint stays a *compile-time* choice of callee. Passing the
    /// hint to one shared helper as a bare `i64` would make three bodies whose
    /// hook order genuinely differs indistinguishable at the call site and move
    /// a wrong-hint bug from `cargo check` to a Test262 run.
    pub(crate) const fn helper_for(hint: ToPrimitiveHint) -> Self {
        match hint {
            ToPrimitiveHint::Default => Self::ValueToPrimitiveDefault,
            ToPrimitiveHint::Number => Self::ValueToPrimitiveNumber,
            ToPrimitiveHint::String => Self::ValueToPrimitiveString,
        }
    }

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
            // Read: 0=target payload, 1=target tag, 2=index. Write:
            // 0=target payload, 1=target tag, 2=index, 3=key payload,
            // 4=value payload, 5=value tag, 6=caller strictness. Both fit the
            // seven-i64/four-i64 shape, so neither needs a new Wasm type.
            | Self::IndexedElementRead
            | Self::IndexedElementWrite
            // ToPrimitive/ToPropertyKey: 0=value payload, 1=value tag, 2..6
            // unused. Results are the standard
            // `(payload, tag, completion, completion_aux)` tuple, so neither
            // needs a new Wasm type either.
            | Self::ValueToPrimitiveDefault
            | Self::ValueToPrimitiveNumber
            | Self::ValueToPrimitiveString
            | Self::ValueToPropertyKey
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
            | Self::TemporalCalendarIdentifier
            | Self::IndexedElementRead
            | Self::IndexedElementWrite
            | Self::ValueToPrimitiveDefault
            | Self::ValueToPrimitiveNumber
            | Self::ValueToPrimitiveString
            | Self::ValueToPropertyKey => true,
            Self::JsonStringifyValue => emission.holds(RuntimeHelperFact::UsesJsonStringify),
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
            Self::IndexedElementRead => "indexed_element_read",
            Self::IndexedElementWrite => "indexed_element_write",
            Self::ValueToPrimitiveDefault => "value_to_primitive_default",
            Self::ValueToPrimitiveNumber => "value_to_primitive_number",
            Self::ValueToPrimitiveString => "value_to_primitive_string",
            Self::ValueToPropertyKey => "value_to_property_key",
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
        // 38 unconditional helpers plus JSON.stringify's value helper. The
        // `debug_dump` line used to hard-code 27 and had drifted by five.
        let without_json = RuntimeHelperId::ALL
            .iter()
            .filter(|helper| helper.is_emitted(RuntimeHelperEmission::NONE))
            .count();
        let with_json = RuntimeHelperId::ALL
            .iter()
            .filter(|helper| {
                helper.is_emitted(
                    RuntimeHelperEmission::NONE.with(RuntimeHelperFact::UsesJsonStringify, true),
                )
            })
            .count();
        assert_eq!(without_json, 38);
        assert_eq!(with_json, 39);
    }

    /// Every hint names a distinct body, and every body is a real helper in
    /// `ALL`. Without the second half a hint could name a variant that the
    /// function/code sections never write, which would hand every ToPrimitive
    /// call site an index one past the end of the helper block.
    #[test]
    fn every_to_primitive_hint_names_its_own_helper() {
        let helpers = [
            ToPrimitiveHint::Default,
            ToPrimitiveHint::Number,
            ToPrimitiveHint::String,
        ]
        .map(RuntimeHelperId::helper_for);
        assert_eq!(
            helpers,
            [
                RuntimeHelperId::ValueToPrimitiveDefault,
                RuntimeHelperId::ValueToPrimitiveNumber,
                RuntimeHelperId::ValueToPrimitiveString,
            ]
        );
        for helper in helpers {
            assert!(
                RuntimeHelperId::ALL.contains(&helper),
                "{helper:?} is not in ALL, so its body is never emitted"
            );
            assert!(
                helper.is_emitted(RuntimeHelperEmission::NONE),
                "{helper:?} must be unconditional: the ToPrimitive seam has no \
                 emission fact to gate on"
            );
        }
    }

    /// Only `FunctionBuilder::begin_helper_body` may build a helper body.
    ///
    /// A `FunctionBuilder` body is a `Function` with one `i64` local per
    /// `local_count()`. Exactly three places in the crate build one:
    /// `FunctionBuilder::compile` (user functions), `compile_builtin`
    /// (standard/host builtin bodies), and `begin_helper_body` — and only the
    /// last is a helper. `begin_helper_body` is what derives, from the
    /// [`RuntimeHelperId`] it is handed, the clearing of that helper's own
    /// inline seam.
    ///
    /// Nothing in the type system can forbid a fourth site: `Function` is
    /// `code_sink::Function` and anyone in the crate may construct one. But the
    /// failure mode
    /// of a new `compile_x_helper` that copies the constructor instead of
    /// calling `begin_helper_body` is invisible to every cheap check — the
    /// helper's body reaches its own seam and emits `call $itself`, which
    /// type-checks in Rust, encodes to valid Wasm, links, and validates,
    /// diverging only when a case that reaches it is executed under the full
    /// suite. So the ban is enforced here instead, at rung 1 rather than at
    /// rung 5.
    ///
    /// The needle is assembled at run time rather than written out, so that
    /// this file does not match itself.
    #[test]
    fn only_three_places_build_a_function_builder_body() {
        let needle = format!(
            "Function::new_with_locals_types(std::iter::repeat_n({},{}))",
            "ValType::I64", "self.local_count()"
        );
        let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut sites: Vec<String> = Vec::new();
        let mut pending = vec![source_root.clone()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).expect("crate `src` directory is readable") {
                let path = entry.expect("readable directory entry").path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).expect("source file is readable");
                let dense: String = text.chars().filter(|c| !c.is_whitespace()).collect();
                let relative = path
                    .strip_prefix(&source_root)
                    .expect("scanned path is under `src`")
                    .display()
                    .to_string();
                sites.extend(std::iter::repeat_n(
                    relative,
                    dense.matches(&needle).count(),
                ));
            }
        }
        sites.sort();
        assert_eq!(
            sites,
            vec!["emit.rs".to_string(); 3],
            "a `FunctionBuilder` body was built outside `compile`, `compile_builtin` and \
             `begin_helper_body`. If it is a helper body, call `begin_helper_body` so the \
             helper's own inline seam is cleared from its id; if it is not, this assertion \
             needs updating together with an explanation of what the fourth body shape is."
        );
    }

    /// `NONE` is the input `is_conditional` is defined against, so "no fact
    /// holds in it" is what makes a conditional helper detectable at all.
    #[test]
    fn the_empty_emission_context_holds_no_fact() {
        for fact in [RuntimeHelperFact::UsesJsonStringify] {
            assert!(!RuntimeHelperEmission::NONE.holds(fact), "{fact:?}");
            assert!(RuntimeHelperEmission::NONE.with(fact, true).holds(fact));
            assert_eq!(
                RuntimeHelperEmission::NONE.with(fact, false),
                RuntimeHelperEmission::NONE
            );
        }
    }
}
