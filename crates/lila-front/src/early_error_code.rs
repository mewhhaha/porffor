//! The closed domain of pre-evaluation rejection codes, and the one table that
//! classifies `boa`'s static-semantics messages into it.
//!
//! ECMA-262 **clause 17** fixes the shape of this area. An early error is
//! detectable and reportable *prior to the evaluation of any construct*; an
//! implementation must treat as an early error every occurrence of a condition
//! listed in a *Static Semantics: Early Errors* subclause, and **must not**
//! treat other kinds of error as early errors. That last sentence is what makes
//! the set closed, and a closed set is an `enum` matched without a catch-all.
//!
//! Two further consequences of clause 17 are carried by the *types* rather than
//! by convention:
//!
//! * `ParseScript` (16.1.4) and `ParseModule` (16.2.1.6.1) each return "a List
//!   of **SyntaxError** objects", and `InitializeEnvironment` throws a
//!   **SyntaxError**. The error type and the reporting phase are therefore
//!   properties of the operation that produced the rejection, never free
//!   parameters of a call site. Nothing here stores them; they are derived —
//!   see [`crate::ParseCode`] and `lila_ir::IrDiagnosticKind`.
//! * The Script early errors (16.1.1) and the Module early errors (16.2.1.2)
//!   apply the *same* abstract operations to their respective item lists. A
//!   source that is an early error as a Script is an early error as a Module,
//!   under the same rule, with the same name. So there is **one** classification
//!   table, not one per parse path.
//!
//! That last point is why this module lives in `lila-front` rather than in
//! `lila-ir`. Every entry and dependency is parsed here exactly once, and the
//! retained structured result is consumed by `lila-ir`. Before this module
//! existed there were two classification tables, and they had measurably drifted: a duplicate
//! `__proto__` in a dependency module and a `lexical name declared in var
//! names` failure in a dependency module were both reported as *unsupported*
//! rather than as the `SyntaxError` the spec requires.
//!
//! # What this buys, and what it does not
//!
//! `boa_parser` reports every static-semantics failure as a generic
//! `Error::general` / `Error::lex` with no machine-readable kind, so the only
//! oracle available is the message text. The types here buy **single-sourcing**
//! (one spelling authority, one table, one classifier) and **exhaustiveness** (a
//! new code fails to build at every consumer). They do **not** buy oracle
//! robustness: if boa rewords a message, one row goes dead and no compile error
//! fires. The `witnesses` column is the mitigation — it keeps the byte strings
//! boa actually emits beside the fragments that are supposed to select them, in
//! one place, so a `vendor/` bump has exactly one file to re-read. See ledger
//! entry L1 of
//! `docs/rust-rewrite/contracts/early-error-taxonomy.md`.
//!
//! Deliberately absent, and deliberately never to be added (the same rule as
//! `lila_ir::NativeErrorKind`): `Display`, `AsRef<str>`,
//! `Deref<Target = str>`, `FromStr`, `Default`, and
//! `From<EarlyErrorCode> for String`. A stringification must name
//! [`EarlyErrorCode::wire_name`] at the call site, so that `format!("{code}")`
//! cannot quietly reintroduce the `&str` domain this type replaces.

/// Byte-wise substring test usable in a `const` initializer.
///
/// Private on purpose: a `pub const fn contains_sub` would be workspace surface
/// with no product call site, which is the shape AGENTS.md names as having
/// shipped here before. `lila_ir::native_error` keeps its own private
/// `str_eq` for the same reason.
///
/// Semantics match `str::contains(&str)`: an empty needle matches everywhere,
/// and because both sides are valid UTF-8 a byte-level match can only ever land
/// on a character boundary.
const fn contains_sub(haystack: &str, needle: &str) -> bool {
    let haystack = haystack.as_bytes();
    let needle = needle.as_bytes();
    if needle.len() > haystack.len() {
        return false;
    }
    let mut start = 0;
    while start + needle.len() <= haystack.len() {
        let mut i = 0;
        let mut matched = true;
        while i < needle.len() {
            if haystack[start + i] != needle[i] {
                matched = false;
                break;
            }
            i += 1;
        }
        if matched {
            return true;
        }
        start += 1;
    }
    false
}

/// Byte-wise `&str` equality usable in a `const` initializer. Private for the
/// same reason as [`contains_sub`].
const fn str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

macro_rules! early_error_codes {
    (
        $(
            $(#[$variant_meta:meta])*
            $variant:ident => $spelling:literal;
        )+
    ) => {
        /// One pre-evaluation rejection condition. See the module docs.
        ///
        /// This is a closed domain: `match` over it without a catch-all, so that
        /// a new condition is `error[E0004]` at every consumer rather
        /// than a silent fall-through to some default classification.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum EarlyErrorCode {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl EarlyErrorCode {
            /// Every code, in declaration order.
            ///
            /// The length is written into the type: adding a row without
            /// updating it is `error[E0308]`, and the tie between this order and
            /// the `#[repr(u8)]` discriminants is checked by assertion P3.
            pub const ALL: [EarlyErrorCode; 30] = [$(EarlyErrorCode::$variant,)+];

            /// The single spelling authority for these codes in this workspace.
            ///
            /// This function's arms are the only `"E_..."` literals in the
            /// workspace, apart from [`NO_EARLY_ERROR_CODE`] — the one
            /// placeholder that names the *absence* of a code, now spelled in
            /// this module beside them and proved distinct from every code by
            /// assertion P5'.
            #[must_use]
            pub const fn wire_name(self) -> &'static str {
                match self {
                    $(EarlyErrorCode::$variant => $spelling,)+
                }
            }

            /// The only parse. Total on the domain, `None` off it.
            ///
            /// **Private on purpose.** Its sole consumer is assertion P3, which
            /// uses the round trip to prove [`EarlyErrorCode::wire_name`] is
            /// injective. `lila_ir::NativeErrorKind::from_str` is `pub`
            /// because it has a product call site; this one does not, and a
            /// `pub` item with no call site is the "survival by `pub`" shape
            /// AGENTS.md names. Make it `pub` in the same patch that gives it a
            /// consumer, not before.
            const fn from_wire_name(name: &str) -> Option<Self> {
                $(
                    if str_eq(name, $spelling) {
                        return Some(EarlyErrorCode::$variant);
                    }
                )+
                None
            }
        }
    };
}

early_error_codes! {
    // ---- rejected during parse (clause 17; ParseScript 16.1.4 / ParseModule 16.2.1.6.1)
    /// B.3.1 `sec-__proto__-property-names-in-object-initializers`, amending the
    /// 13.2.5.1 ObjectLiteral early errors: `PropertyNameList of
    /// PropertyDefinitionList` contains two or more `"__proto__"` entries.
    ObjectDuplicateProto => "E_OBJECT_DUPLICATE_PROTO";
    /// 16.1.1 / 16.2.1.2 / 14.2.1 / 14.12.1 / 15.2.1 / 15.7.1. Any lexical
    /// redeclaration: `LexicallyDeclaredNames` with duplicates, or intersecting
    /// `VarDeclaredNames`, or a formal parameter name intersecting the body's
    /// `LexicallyDeclaredNames`.
    DuplicateLexicalDeclaration => "E_DUPLICATE_LEXICAL_DECLARATION";
    /// Duplicate `BoundNames` in a non-simple formal-parameter list, strict
    /// function code, or a grammar production requiring
    /// `UniqueFormalParameters`. Sloppy ordinary functions with simple
    /// parameter lists are deliberately excluded.
    DuplicateFormalParameter => "E_DUPLICATE_FORMAL_PARAMETER";
    /// TryStatement early errors: `BoundNames` of a `CatchParameter` contains
    /// duplicate elements. Unlike ordinary-function parameters, this condition
    /// has no sloppy simple-list exception.
    DuplicateCatchParameter => "E_DUPLICATE_CATCH_PARAMETER";
    /// TryStatement early errors: a catch parameter's `BoundNames` intersects
    /// the catch block's `LexicallyDeclaredNames`, or a binding-pattern catch
    /// parameter's `BoundNames` intersects the block's `VarDeclaredNames`. A
    /// simple `BindingIdentifier` retains its specified `var` exception.
    CatchBodyDeclarationConflict => "E_CATCH_BODY_DECLARATION_CONFLICT";
    /// ClassBody early errors: `PrototypePropertyNameList` contains more than
    /// one occurrence of `"constructor"`. Static and computed methods named
    /// `constructor` are not constructor definitions and remain excluded.
    DuplicateClassConstructor => "E_DUPLICATE_CLASS_CONSTRUCTOR";
    /// ClassElement early errors: a non-static generator or async-generator
    /// method has the literal property name `"constructor"`. Static and
    /// computed generator methods named `constructor` remain excluded.
    ClassConstructorGeneratorMethod => "E_CLASS_CONSTRUCTOR_GENERATOR_METHOD";
    /// ClassElement early errors: a non-static async method has the literal
    /// property name `"constructor"`. Static and computed async methods remain
    /// excluded.
    ClassConstructorAsyncMethod => "E_CLASS_CONSTRUCTOR_ASYNC_METHOD";
    /// ClassElement early errors: a non-static getter has the literal property
    /// name `"constructor"`. Static and computed getters remain excluded.
    ClassConstructorGetter => "E_CLASS_CONSTRUCTOR_GETTER";
    /// ClassElement early errors: a non-static setter has the literal property
    /// name `"constructor"`. Static and computed setters remain excluded.
    ClassConstructorSetter => "E_CLASS_CONSTRUCTOR_SETTER";
    /// ClassElementName early errors: the PrivateIdentifier is
    /// `#constructor`. Public computed names whose StringValue is
    /// `"#constructor"` remain excluded.
    ClassPrivateConstructorName => "E_CLASS_PRIVATE_CONSTRUCTOR_NAME";
    /// ClassBody early errors: `PrivateBoundIdentifiers` contains duplicate
    /// entries, except for the permitted getter/setter pair with matching
    /// static placement. Nested class bodies have independent private-name
    /// domains.
    ClassDuplicatePrivateName => "E_CLASS_DUPLICATE_PRIVATE_NAME";
    /// ClassStaticBlockBody early errors: `ContainsArguments` of the
    /// `ClassStaticBlockStatementList` is true. Nested ordinary function and
    /// method bodies are traversal boundaries; arrow functions are not.
    ClassStaticBlockContainsArguments => "E_CLASS_STATIC_BLOCK_CONTAINS_ARGUMENTS";
    /// FieldDefinition early errors: `ContainsArguments` of a public/private,
    /// instance/static or auto-accessor initializer is true. Nested ordinary
    /// function and method bodies are boundaries; arrow functions are not.
    ClassFieldContainsArguments => "E_CLASS_FIELD_CONTAINS_ARGUMENTS";
    /// 14.13.1. `ContainsDuplicateLabels` with argument « » is `true`.
    DuplicateLabel => "E_DUPLICATE_LABEL";
    /// 14.13.1, applied by 16.1.1 / 16.2.1.2. `ContainsUndefinedBreakTarget`
    /// with argument « » is `true`.
    UndefinedBreakTarget => "E_UNDEFINED_BREAK_TARGET";
    /// 14.13.1, applied by 16.1.1 / 16.2.1.2. `ContainsUndefinedContinueTarget`
    /// with arguments « », « » is `true`.
    UndefinedContinueTarget => "E_UNDEFINED_CONTINUE_TARGET";
    /// 14.9.1. A `BreakStatement` not nested within an `IterationStatement` or a
    /// `SwitchStatement`.
    IllegalBreak => "E_ILLEGAL_BREAK";
    /// 14.8.1. A `ContinueStatement` not nested within an `IterationStatement`.
    IllegalContinue => "E_ILLEGAL_CONTINUE";
    /// 15.7.1 `AllPrivateIdentifiersValid`, applied by 16.1.1 / 16.2.1.2.
    InvalidPrivateIdentifier => "E_INVALID_PRIVATE_IDENTIFIER";
    /// 16.2.3.1 / 16.2.1.2. `ExportedNames of ModuleItemList` contains
    /// duplicates. An **early** error, which is why `rejection_kind` maps it to
    /// `EarlyError` even though a link-stage producer also raises it.
    ModuleDuplicateExport => "E_MODULE_DUPLICATE_EXPORT";
    /// 16.2.3.1 / 16.2.1.2. An element of `ExportedBindings` occurs in neither
    /// `VarDeclaredNames` nor `LexicallyDeclaredNames`.
    ModuleUndeclaredExport => "E_MODULE_UNDECLARED_EXPORT";
    /// 16.2.1.2. `ModuleItemList Contains super` is `true`.
    ModuleTopLevelSuper => "E_MODULE_TOP_LEVEL_SUPER";
    /// 16.2.1.2. `ModuleItemList Contains NewTarget` is `true`.
    ModuleTopLevelNewTarget => "E_MODULE_TOP_LEVEL_NEW_TARGET";
    // ---- rejected during linking (16.2.1.5 InnerModuleLinking, 16.2.1.6.4
    // ResolveExport, InitializeEnvironment). Not clause-17 territory, but every
    // property that matters here is shared: decided before any construct
    // evaluates, reported as a `SyntaxError`, and in an AOT compiler produced at
    // compile time. test262 spells the phase `resolution` and the type
    // `SyntaxError`.
    /// A requested specifier the host could not resolve.
    ModuleUnresolved => "E_MODULE_UNRESOLVED";
    /// `ResolveExport` returned **null**.
    ModuleMissingExport => "E_MODULE_MISSING_EXPORT";
    /// `ResolveExport` returned **ambiguous**.
    ModuleAmbiguousExport => "E_MODULE_AMBIGUOUS_EXPORT";
    /// Host invariant: one key loaded twice with different source text.
    ModuleInconsistentLoad => "E_MODULE_INCONSISTENT_LOAD";
    /// An implementation limit, **not** a spec condition. Claiming `SyntaxError`
    /// for it is a recorded defect (ledger L4), deliberately not fixed by the
    /// lane that introduced this enum: the fix changes which diagnostics reach
    /// the backend and belongs to whoever owns `modules/graph.rs` and `emit.rs`.
    ModuleUnsupportedPhase => "E_MODULE_UNSUPPORTED_PHASE";
    /// An implementation limit, **not** a spec condition. See ledger L4.
    ModuleTooManyUnits => "E_MODULE_TOO_MANY_UNITS";
}

const fn code_eq(a: EarlyErrorCode, b: EarlyErrorCode) -> bool {
    a as u8 == b as u8
}

/// One `boa` static-semantics message shape, and the code it denotes.
///
/// Private, along with the table it populates: the classification is a function
/// ([`classify_parse_failure`]), not a data structure callers walk themselves.
struct ParseFailureRule {
    /// Every fragment that must appear in boa's message for this rule to fire.
    ///
    /// Chosen to be the invariant part of boa's `format!`, never an interpolated
    /// identifier. Never empty — assertion P1; an empty fragment list would
    /// match *every* message, because `[].iter().all(_)` is `true`.
    fragments: &'static [&'static str],
    /// The condition this message shape denotes.
    code: EarlyErrorCode,
    /// Every message boa actually produces that this row must classify, copied
    /// verbatim from the cited source.
    ///
    /// A **list**, because one fragment set legitimately covers several of boa's
    /// wordings for one spec rule. Never empty — P1. Consumed by P2 and P6, so
    /// it is not documentation: a row whose witnesses stop selecting it does not
    /// compile.
    witnesses: &'static [&'static str],
}

/// The row count, in the type. Adding a row without updating this is
/// `error[E0308]`, which is the moment to check the new row against P1/P2/P7.
const PARSE_FAILURE_RULE_COUNT: usize = 28;

/// The one fragment table.
///
/// Rows are keyed by boa's *message shape*; codes are keyed by the *spec rule*.
/// That is why four rows (4, 5, 6, 7) carry [`EarlyErrorCode::DuplicateLexicalDeclaration`]:
/// boa emits five distinct wordings for one rule and the spec does not require
/// one code per wording.
///
/// Order is irrelevant. `classify_parse_failure` takes the first match, but P2
/// proves at most one row can match any witness, so inserting a row cannot
/// silently change an existing classification.
const PARSE_FAILURE_RULE_TABLE: [ParseFailureRule; PARSE_FAILURE_RULE_COUNT] = [
    // 1. boa_parser/src/parser/expression/primary/object_initializer/mod.rs:133
    ParseFailureRule {
        fragments: &["Duplicate __proto__ fields"],
        code: EarlyErrorCode::ObjectDuplicateProto,
        witnesses: &["Duplicate __proto__ fields are not allowed in object literals."],
    },
    // 2. boa_parser/src/parser/mod.rs:541
    ParseFailureRule {
        fragments: &["exported name", "declared multiple times"],
        code: EarlyErrorCode::ModuleDuplicateExport,
        witnesses: &["exported name `x` declared multiple times"],
    },
    // 3. boa_parser/src/parser/mod.rs:556
    ParseFailureRule {
        fragments: &["could not find the exported binding"],
        code: EarlyErrorCode::ModuleUndeclaredExport,
        witnesses: &["could not find the exported binding `x` in the declared names of the module"],
    },
    // 4. W2: boa_parser/src/parser/mod.rs:512,526 (module goal only).
    //    W1: boa_parser/src/parser/mod.rs:366,376; statement/block/mod.rs:109;
    //        statement/switch/mod.rs:88; statement/declaration/lexical.rs:239;
    //        statement/declaration/hoistable/class_decl/mod.rs:712.
    ParseFailureRule {
        fragments: &["lexical name", "declared multiple times"],
        code: EarlyErrorCode::DuplicateLexicalDeclaration,
        witnesses: &[
            "lexical name `x` declared multiple times",
            "lexical name declared multiple times",
        ],
    },
    // 5. W3: statement/block/mod.rs:122; class_decl/mod.rs:724.
    //    W4: statement/switch/mod.rs:101.
    ParseFailureRule {
        fragments: &["lexical name declared in var"],
        code: EarlyErrorCode::DuplicateLexicalDeclaration,
        witnesses: &[
            "lexical name declared in var names",
            "lexical name declared in var declared names",
        ],
    },
    // 6. W5: boa_parser/src/parser/mod.rs:186-191 and :230-235 wrap
    //    boa_ast/src/scope_analyzer.rs:1783,1793.
    //
    //    **Measured: unreachable through this crate's entry points.** The only
    //    payload-carrying `ControlFlow::Break` in `scope_analyzer.rs` is at
    //    :1220, in `visit_script_mut`, forwarding
    //    `global_declaration_instantiation`'s `Err` at :1783 and :1793; both
    //    require `env.has_binding(name)` / `env.has_lex_binding(name)` on the
    //    scope that was passed in, and both `lila_front::parse` (lib.rs:239)
    //    passes a fresh `Scope::new_global()` whose `bindings` are
    //    `RefCell::default()` (boa_ast/src/scope.rs:115-128).
    //    `visit_module_mut` (:1202-1210) has no `Break` at all, so the module
    //    goal cannot reach it under any scope either.
    //
    //    Retained rather than deleted (DR-6 forbids deleting a row on a
    //    negative reachability result): boa has a *third* producer of this
    //    wording at scope_analyzer.rs:2364, in
    //    `eval_declaration_instantiation_scope`, reachable only via
    //    `analyze_scope_eval`, which this compiler never calls. It matches this
    //    row and only this row, so an eval path added later classifies
    //    correctly with no edit here. Ledger L5's confirmed instance.
    ParseFailureRule {
        fragments: &["duplicate lexical declaration"],
        code: EarlyErrorCode::DuplicateLexicalDeclaration,
        witnesses: &["invalid scope analysis: duplicate lexical declaration"],
    },
    // 7. boa_parser/src/parser/mod.rs:614. 15.2.1: BoundNames of FormalParameters
    //    intersects LexicallyDeclaredNames of FunctionBody.
    ParseFailureRule {
        fragments: &["formal parameter", "declared in lexically declared names"],
        code: EarlyErrorCode::DuplicateLexicalDeclaration,
        witnesses: &["formal parameter `x` declared in lexically declared names"],
    },
    // 8. Ten pinned Boa producer sites use this exact, case-sensitive wording
    //    for non-simple parameter lists and strict/context checks. See
    //    `duplicate-formal-parameter-early-errors.md` for the measured inventory.
    ParseFailureRule {
        fragments: &["Duplicate parameter name not allowed in this context"],
        code: EarlyErrorCode::DuplicateFormalParameter,
        witnesses: &["Duplicate parameter name not allowed in this context"],
    },
    // 9. boa_parser/src/parser/function/mod.rs:199, shared by every
    //    `UniqueFormalParameters` consumer. The lowercase `duplicate` is part
    //    of the pinned message contract.
    ParseFailureRule {
        fragments: &["duplicate parameter name not allowed in unique formal parameters"],
        code: EarlyErrorCode::DuplicateFormalParameter,
        witnesses: &["duplicate parameter name not allowed in unique formal parameters"],
    },
    // 10. boa_parser/src/parser/statement/try_stm/catch.rs:78. This is the sole
    //     pinned producer and exact, case-sensitive wording for duplicate
    //     `BoundNames` in a `CatchParameter`.
    ParseFailureRule {
        fragments: &["duplicate catch parameter identifier"],
        code: EarlyErrorCode::DuplicateCatchParameter,
        witnesses: &["duplicate catch parameter identifier"],
    },
    // 11. boa_parser/src/parser/statement/try_stm/catch.rs:99,108. The one
    //     exact, case-sensitive wording covers both the lexical-declaration
    //     branch and the binding-pattern/var-declaration branch.
    ParseFailureRule {
        fragments: &["catch parameter identifier declared in catch body"],
        code: EarlyErrorCode::CatchBodyDeclarationConflict,
        witnesses: &["catch parameter identifier declared in catch body"],
    },
    // 12. statement/declaration/hoistable/class_decl/mod.rs:319-324. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        fragments: &["a class may only have one constructor"],
        code: EarlyErrorCode::DuplicateClassConstructor,
        witnesses: &["a class may only have one constructor"],
    },
    // 13. statement/declaration/hoistable/class_decl/mod.rs:786-792,850-855.
    //     These are the two pinned producers, for generator and async-generator
    //     methods, and they share this complete, case-sensitive wording.
    ParseFailureRule {
        fragments: &["class constructor may not be a generator method"],
        code: EarlyErrorCode::ClassConstructorGeneratorMethod,
        witnesses: &["class constructor may not be a generator method"],
    },
    // 14. statement/declaration/hoistable/class_decl/mod.rs:890-896. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        fragments: &["class constructor may not be an async method"],
        code: EarlyErrorCode::ClassConstructorAsyncMethod,
        witnesses: &["class constructor may not be an async method"],
    },
    // 15. statement/declaration/hoistable/class_decl/mod.rs:1155-1161. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        fragments: &["class constructor may not be a getter method"],
        code: EarlyErrorCode::ClassConstructorGetter,
        witnesses: &["class constructor may not be a getter method"],
    },
    // 16. statement/declaration/hoistable/class_decl/mod.rs:1257-1263. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        fragments: &["class constructor may not be a setter method"],
        code: EarlyErrorCode::ClassConstructorSetter,
        witnesses: &["class constructor may not be a setter method"],
    },
    // 17. statement/declaration/hoistable/class_decl/mod.rs:810-816,
    //     843-849,918-924,986-992,1119-1125,1220-1226,1316-1322. These seven
    //     pinned producers cover private fields and every private method form
    //     with one complete, case-sensitive wording.
    ParseFailureRule {
        fragments: &["class constructor may not be a private method"],
        code: EarlyErrorCode::ClassPrivateConstructorName,
        witnesses: &["class constructor may not be a private method"],
    },
    // 18. statement/declaration/hoistable/class_decl/mod.rs:367-467. Five
    //     pinned branches use this exact, case-sensitive wording for duplicate
    //     private methods, accessors and fields.
    ParseFailureRule {
        fragments: &["private identifier has already been declared"],
        code: EarlyErrorCode::ClassDuplicatePrivateName,
        witnesses: &["private identifier has already been declared"],
    },
    // 19. statement/declaration/hoistable/class_decl/mod.rs:740-745. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        fragments: &["'arguments' not allowed in class static block"],
        code: EarlyErrorCode::ClassStaticBlockContainsArguments,
        witnesses: &["'arguments' not allowed in class static block"],
    },
    // 20. statement/declaration/hoistable/class_decl/mod.rs:1522-1558. The
    //     exhaustive class-element match uses this one exact wording for
    //     public/private, instance/static and auto-accessor field initializers.
    ParseFailureRule {
        fragments: &["'arguments' not allowed in class field definition"],
        code: EarlyErrorCode::ClassFieldContainsArguments,
        witnesses: &["'arguments' not allowed in class field definition"],
    },
    // 21. boa_parser/src/parser/mod.rs:567
    ParseFailureRule {
        fragments: &["module cannot contain", "super"],
        code: EarlyErrorCode::ModuleTopLevelSuper,
        witnesses: &["module cannot contain `super` on the top-level"],
    },
    // 22. boa_parser/src/parser/mod.rs:575
    ParseFailureRule {
        fragments: &["module cannot contain", "new.target"],
        code: EarlyErrorCode::ModuleTopLevelNewTarget,
        witnesses: &["module cannot contain `new.target` on the top-level"],
    },
    // 23. boa_parser/src/parser/mod.rs:462,593; statement/mod.rs:1020.
    ParseFailureRule {
        fragments: &["invalid private identifier usage"],
        code: EarlyErrorCode::InvalidPrivateIdentifier,
        witnesses: &["invalid private identifier usage"],
    },
    // 24-28. `CheckLabelsError::message`, boa_ast/src/operations/mod.rs:1399-1417.
    ParseFailureRule {
        fragments: &["duplicate label"],
        code: EarlyErrorCode::DuplicateLabel,
        witnesses: &["duplicate label: lbl"],
    },
    ParseFailureRule {
        fragments: &["undefined break target"],
        code: EarlyErrorCode::UndefinedBreakTarget,
        witnesses: &["undefined break target: lbl"],
    },
    ParseFailureRule {
        fragments: &["undefined continue target"],
        code: EarlyErrorCode::UndefinedContinueTarget,
        witnesses: &["undefined continue target: lbl"],
    },
    ParseFailureRule {
        fragments: &["illegal break statement"],
        code: EarlyErrorCode::IllegalBreak,
        witnesses: &["illegal break statement"],
    },
    ParseFailureRule {
        fragments: &["illegal continue statement"],
        code: EarlyErrorCode::IllegalContinue,
        witnesses: &["illegal continue statement"],
    },
];

/// Slice view of [`PARSE_FAILURE_RULE_TABLE`], so the walkers below index a
/// `&'static [_]` rather than copying the array on every call.
const PARSE_FAILURE_RULES: &[ParseFailureRule] = &PARSE_FAILURE_RULE_TABLE;

const fn rule_matches(rule: &ParseFailureRule, message: &str) -> bool {
    let mut i = 0;
    while i < rule.fragments.len() {
        if !contains_sub(message, rule.fragments[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// The failure-detail token that names the **absence** of a code.
///
/// Spelled once, here, beside the codes it must never collide with; assertion
/// P5' proves no `wire_name()` equals it. `lila-test262` names this constant
/// rather than re-spelling the literal, so a new variant spelled
/// `E_IR_DIAGNOSTIC` fails to build instead of silently merging with the
/// "no code" bucket in every failure report.
pub const NO_EARLY_ERROR_CODE: &str = "E_IR_DIAGNOSTIC";

/// A code that the **parse** table can actually produce.
///
/// A witness type with no public constructor beyond the two gated ones below.
/// It exists because the parse-stage producers and the link-stage producers
/// share one `EarlyErrorCode` domain but *not* one reporting phase: a
/// `ParseCode::Early(EarlyErrorCode::ModuleMissingExport)` would report a
/// `resolution`-kind condition at `ParseDiagnosticPhase::Early`, which is one
/// code under two phases from two paths — the exact mistake class the merged
/// domain was built to end. Assertion P7 constrains what the *table* can yield;
/// this type constrains what a *call site* can say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParseClassified(EarlyErrorCode);

impl ParseClassified {
    /// The gate. `None` for a code no row of the fragment table carries — i.e.
    /// for a link-only condition, which a parse-stage producer must not claim.
    #[must_use]
    pub const fn from_early(code: EarlyErrorCode) -> Option<Self> {
        if code.is_parse_classified() {
            Some(Self(code))
        } else {
            None
        }
    }

    /// [`Self::from_early`] for a code named as a literal, with `None` turned
    /// into a **compile error**.
    ///
    /// Only meaningful in a `const` initializer; that is the point. The two
    /// parse-stage producers in `lila_ir::modules::early` bind their codes to
    /// `const` items built with this, so naming a link-only code there fails to
    /// build rather than reporting under the wrong phase.
    #[must_use]
    pub const fn from_parse_table(code: EarlyErrorCode) -> Self {
        match Self::from_early(code) {
            Some(classified) => classified,
            None => panic!(
                "this EarlyErrorCode is not producible by PARSE_FAILURE_RULES, so a parse-stage \
                 producer must not name it"
            ),
        }
    }

    /// The underlying condition. One-way on purpose: widening back to
    /// `EarlyErrorCode` is free, narrowing is gated.
    #[must_use]
    pub const fn code(self) -> EarlyErrorCode {
        self.0
    }
}

/// Message shapes in which `boa` interpolates **source text** the user wrote.
///
/// `boa_parser::Error::Unexpected` renders as
/// `unexpected token '{found}', {message} at line L, col C`, and
/// `Error::Expected` as `expected token '{t}', got '{found}' in {ctx} …`;
/// `TokenKind::StringLiteral` renders as its raw contents
/// (`boa_parser/src/lexer/token.rs:313`). So
/// `var x = "illegal break statement" "y";` produces a message that contains a
/// row's whole fragment set verbatim, and the string oracle would classify an
/// ordinary syntax error as an early error. On the entry path that is
/// taxonomy-only (both report `parse`/`SyntaxError`); on the dependency path it
/// converts an `IrDiagnostic::unsupported` into a spec rejection, which is a
/// compiler gap wearing a spec claim.
///
/// Detected by substring rather than prefix because Boa includes location and
/// context around the token. Assertion P10 proves this guard eats no witness
/// of any row. Ledger L1.
const INTERPOLATING_MESSAGE_SHAPES: &[&str] =
    &["unexpected token '", "expected token '", "expected one of "];

const fn message_interpolates_source_text(message: &str) -> bool {
    let mut i = 0;
    while i < INTERPOLATING_MESSAGE_SHAPES.len() {
        if contains_sub(message, INTERPOLATING_MESSAGE_SHAPES[i]) {
            return true;
        }
        i += 1;
    }
    false
}

/// Classifies a `boa` parse failure into the one closed domain.
///
/// `None` means "a syntax error whose wording we do not model", and it must stay
/// that way: claiming a spec rejection for a source we merely failed to read
/// would dress a compiler gap up as a spec claim. Callers turn `None` into
/// `Unsupported` / `Malformed`, never into a code.
///
/// This is the only such function in the workspace. `lila-ir` calls this one.
#[must_use]
pub const fn classify_parse_failure(message: &str) -> Option<ParseClassified> {
    // The oracle is a substring test over a message boa built by interpolation.
    // Refuse to read one of the two shapes that can carry user source text into
    // it; `Malformed`/`Unsupported` is the honest answer there.
    if message_interpolates_source_text(message) {
        return None;
    }
    let mut i = 0;
    while i < PARSE_FAILURE_RULES.len() {
        let rule = &PARSE_FAILURE_RULES[i];
        if rule_matches(rule, message) {
            return Some(ParseClassified(rule.code));
        }
        i += 1;
    }
    None
}

impl EarlyErrorCode {
    /// True iff some row of the one fragment table can produce this code — i.e.
    /// iff a boa **parse** failure can be classified as this condition.
    ///
    /// `pub` because its consumer is `lila_ir::early_error_code`'s assertion
    /// P7, which must hold across the crate boundary while the table itself
    /// stays private to this module. P7 is the structural replacement for the
    /// doc comment that used to ask, in words, that the two tables agree.
    #[must_use]
    pub const fn is_parse_classified(self) -> bool {
        let mut i = 0;
        while i < PARSE_FAILURE_RULES.len() {
            if code_eq(PARSE_FAILURE_RULES[i].code, self) {
                return true;
            }
            i += 1;
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Const assertions P1-P6. Each names the mistake it makes fail to build.
// ---------------------------------------------------------------------------

/// P1: no row has an empty `fragments` or `witnesses` list, **and no fragment
/// or witness is the empty string**.
///
/// An empty `fragments` list matches every message, so one row would swallow
/// every parse failure into a single code. An empty `witnesses` list would
/// exempt the row from P2, P6 and P10 entirely. And a single empty *fragment*
/// has the same effect as an empty list: `contains_sub` returns `true` for an
/// empty needle (its length guard only rejects a needle **longer** than the
/// haystack), so `fragments: &[""]` matches every message. That last case was
/// caught only as a side effect of P2 — i.e. only because other rows happened
/// to have witnesses that the empty row would then also match.
const fn every_row_is_populated() -> bool {
    let mut i = 0;
    while i < PARSE_FAILURE_RULES.len() {
        let rule = &PARSE_FAILURE_RULES[i];
        if rule.fragments.is_empty() || rule.witnesses.is_empty() {
            return false;
        }
        let mut f = 0;
        while f < rule.fragments.len() {
            if rule.fragments[f].is_empty() {
                return false;
            }
            f += 1;
        }
        let mut w = 0;
        while w < rule.witnesses.len() {
            if rule.witnesses[w].is_empty() {
                return false;
            }
            w += 1;
        }
        i += 1;
    }
    true
}

/// P2: every witness selects exactly one row, and it is the row that owns it.
///
/// This is the disjointness the old two-table code asserted in a comment
/// ("the patterns are disjoint"). As a checked fact it does more than the
/// comment did: it makes the table **order-independent**, so a row inserted
/// above an existing one cannot silently shadow it, and a row whose fragments
/// are a superset of another's cannot be silently unreachable.
const fn witnesses_select_their_own_row() -> bool {
    let mut row = 0;
    while row < PARSE_FAILURE_RULES.len() {
        let mut w = 0;
        while w < PARSE_FAILURE_RULES[row].witnesses.len() {
            let witness = PARSE_FAILURE_RULES[row].witnesses[w];
            let mut other = 0;
            while other < PARSE_FAILURE_RULES.len() {
                let matched = rule_matches(&PARSE_FAILURE_RULES[other], witness);
                if matched != (other == row) {
                    return false;
                }
                other += 1;
            }
            w += 1;
        }
        row += 1;
    }
    true
}

const fn found_is(found: Option<EarlyErrorCode>, expected: EarlyErrorCode) -> bool {
    match found {
        Some(code) => code_eq(code, expected),
        None => false,
    }
}

const fn classified_is(found: Option<ParseClassified>, expected: EarlyErrorCode) -> bool {
    match found {
        Some(classified) => code_eq(classified.code(), expected),
        None => false,
    }
}

/// P3: `ALL` is in discriminant order and complete, and `wire_name` round-trips
/// through `from_wire_name` — so print and parse cannot diverge, and the round
/// trip proves `wire_name` is injective.
const fn all_is_ordered_and_round_trips() -> bool {
    let all = EarlyErrorCode::ALL;
    let mut i = 0;
    while i < all.len() {
        if all[i] as u8 != i as u8 {
            return false;
        }
        if !found_is(EarlyErrorCode::from_wire_name(all[i].wire_name()), all[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// P4: no two codes share a `wire_name()`.
///
/// A duplicated spelling would make one of the two unreachable through
/// `from_wire_name` and would collapse two taxonomy buckets in every failure
/// report built from the wire name.
const fn wire_names_are_distinct() -> bool {
    let all = EarlyErrorCode::ALL;
    let mut i = 0;
    while i < all.len() {
        let mut j = i + 1;
        while j < all.len() {
            if str_eq(all[i].wire_name(), all[j].wire_name()) {
                return false;
            }
            j += 1;
        }
        i += 1;
    }
    true
}

/// P5: every `wire_name()` is `E_` followed by `A`-`Z` and `_` only.
///
/// `"e_FOO"`, `"E_Foo"` or a stray space is a typo that a `&str` code domain
/// would have carried all the way into the failure taxonomy, where it reads as
/// a distinct failure family.
const fn wire_names_are_well_formed() -> bool {
    let all = EarlyErrorCode::ALL;
    let mut i = 0;
    while i < all.len() {
        let bytes = all[i].wire_name().as_bytes();
        if bytes.len() < 3 || bytes[0] != b'E' || bytes[1] != b'_' {
            return false;
        }
        let mut b = 2;
        while b < bytes.len() {
            let ch = bytes[b];
            if !(ch == b'_' || ch.is_ascii_uppercase()) {
                return false;
            }
            b += 1;
        }
        i += 1;
    }
    true
}

/// P5': no code's `wire_name()` collides with [`NO_EARLY_ERROR_CODE`].
///
/// That token names the *absence* of a code in every failure-detail string a
/// coded diagnostic does not produce. A new variant spelled the same way would
/// pass P4 and P5 — it is distinct from the other codes and
/// well-formed — and would then be indistinguishable from "no code" in every
/// report built from the wire name, reintroducing exactly the confusion the
/// `Option<EarlyErrorCode>` representation was chosen to end.
const fn no_wire_name_is_the_absence_placeholder() -> bool {
    let all = EarlyErrorCode::ALL;
    let mut i = 0;
    while i < all.len() {
        if str_eq(all[i].wire_name(), NO_EARLY_ERROR_CODE) {
            return false;
        }
        i += 1;
    }
    true
}

/// P10: every witness still classifies, **through the whole classifier**.
///
/// P2 checks `rule_matches` row by row; this checks `classify_parse_failure`,
/// which now also refuses [`INTERPOLATING_MESSAGE_SHAPES`]. Without it, adding
/// a guard shape that happens to occur inside a real boa wording would silently
/// take a whole spec condition out of the taxonomy, and P2 would still pass.
const fn every_witness_classifies_to_its_own_code() -> bool {
    let mut row = 0;
    while row < PARSE_FAILURE_RULES.len() {
        let rule = &PARSE_FAILURE_RULES[row];
        let mut w = 0;
        while w < rule.witnesses.len() {
            if !classified_is(classify_parse_failure(rule.witnesses[w]), rule.code) {
                return false;
            }
            w += 1;
        }
        row += 1;
    }
    true
}

const _: () = assert!(
    every_row_is_populated(),
    "P1: a PARSE_FAILURE_RULES row has an empty `fragments` (it would match every message) or an empty `witnesses`"
);
const _: () = assert!(
    witnesses_select_their_own_row(),
    "P2: a PARSE_FAILURE_RULES witness is matched by no row, or by a row other than the one that lists it — the table is shadowing"
);
const _: () = assert!(
    all_is_ordered_and_round_trips(),
    "P3: EarlyErrorCode::ALL must be in declaration order, and wire_name must round-trip through from_wire_name"
);
const _: () = assert!(
    wire_names_are_distinct(),
    "P4: two EarlyErrorCode variants share a wire_name() spelling"
);
const _: () = assert!(
    wire_names_are_well_formed(),
    "P5: an EarlyErrorCode wire_name() is not `E_` followed by uppercase ASCII and underscores"
);
const _: () = assert!(
    no_wire_name_is_the_absence_placeholder(),
    "P5': an EarlyErrorCode wire_name() collides with NO_EARLY_ERROR_CODE, the token that names the absence of a code"
);
const _: () = assert!(
    every_witness_classifies_to_its_own_code(),
    "P10: a PARSE_FAILURE_RULES witness no longer classifies to its own code through classify_parse_failure — most likely an INTERPOLATING_MESSAGE_SHAPES guard now eats it"
);
