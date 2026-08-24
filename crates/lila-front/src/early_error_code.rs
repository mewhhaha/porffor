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
//! boa actually emits beside the patterns that are supposed to select them, in
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

/// Byte-wise prefix test usable in a `const` initializer.
///
/// This is deliberately separate from [`contains_sub`]: a fixed Boa message
/// that is followed only by its source position must not also match when user
/// source text embeds that message later inside another diagnostic.
const fn starts_with_sub(haystack: &str, prefix: &str) -> bool {
    let haystack = haystack.as_bytes();
    let prefix = prefix.as_bytes();
    if prefix.len() > haystack.len() {
        return false;
    }
    let mut i = 0;
    while i < prefix.len() {
        if haystack[i] != prefix[i] {
            return false;
        }
        i += 1;
    }
    true
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
            pub const ALL: [EarlyErrorCode; 67] = [$(EarlyErrorCode::$variant,)+];

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
    /// 13.2.5.1 ObjectLiteral early errors: a CoverInitializedName remains
    /// after cover-grammar reinterpretation. Assignment/binding patterns and
    /// arrow parameters are deliberately excluded.
    ObjectLiteralCoverInitializedName => "E_OBJECT_LITERAL_COVER_INITIALIZED_NAME";
    /// 16.1.1 / 16.2.1.2 / 14.2.1 / 14.12.1 / 15.2.1 / 15.7.1. Any lexical
    /// redeclaration: `LexicallyDeclaredNames` with duplicates, or intersecting
    /// `VarDeclaredNames`, or a formal parameter name intersecting the body's
    /// `LexicallyDeclaredNames`.
    DuplicateLexicalDeclaration => "E_DUPLICATE_LEXICAL_DECLARATION";
    /// 14.3.1 / 14.7.5.1 in the frozen 2026 edition: a LexicalDeclaration or
    /// ForDeclaration has a `BoundName` equal to the exact String value
    /// `"let"`. Pinned Boa also applies the living-specification form of this
    /// condition to its resource-declaration grammar.
    LexicalBoundNameLet => "E_LEXICAL_BOUND_NAME_LET";
    /// Duplicate `BoundNames` in a non-simple formal-parameter list, strict
    /// function code, or a grammar production requiring
    /// `UniqueFormalParameters`. Sloppy ordinary functions with simple
    /// parameter lists are deliberately excluded.
    DuplicateFormalParameter => "E_DUPLICATE_FORMAL_PARAMETER";
    /// Callable-production early errors: the callable's own body contains a
    /// Use Strict Directive and its own parameter list is non-simple. Ambient
    /// strictness and parameterless getters are deliberately excluded.
    CallableNonSimpleParametersContainUseStrict => "E_CALLABLE_NON_SIMPLE_PARAMETERS_CONTAIN_USE_STRICT";
    /// FunctionExpression early errors: the expression's FormalParameters or
    /// FunctionBody `Contains SuperProperty` or `Contains SuperCall`.
    /// Nested ordinary callables and classes retain their own boundaries;
    /// ordinary and async arrows remain lexical traversal paths.
    FunctionExpressionContainsSuper => "E_FUNCTION_EXPRESSION_CONTAINS_SUPER";
    /// FunctionDeclaration early errors: the declaration's FormalParameters or
    /// FunctionBody `Contains SuperProperty` or `Contains SuperCall`.
    /// Generator, async-function and async-generator declarations remain
    /// distinct productions with independently owned diagnostics.
    FunctionDeclarationContainsSuper => "E_FUNCTION_DECLARATION_CONTAINS_SUPER";
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
    /// 15.7.1. A class has no ClassHeritage, has a constructor, and
    /// `HasDirectSuper` of that constructor is true. A present heritage,
    /// including `extends null`, is deliberately excluded.
    ClassBaseConstructorHasDirectSuper => "E_CLASS_BASE_CONSTRUCTOR_HAS_DIRECT_SUPER";
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
    /// ClassElement early errors: a public static ordinary, generator, async,
    /// async-generator, getter or setter method has the literal property name
    /// `prototype`. Computed and private names remain excluded.
    ClassStaticMethodPrototypeName => "E_CLASS_STATIC_METHOD_PROTOTYPE_NAME";
    /// ClassBody early errors: `PrivateBoundIdentifiers` contains duplicate
    /// entries, except for the permitted getter/setter pair with matching
    /// static placement. Nested class bodies have independent private-name
    /// domains.
    ClassDuplicatePrivateName => "E_CLASS_DUPLICATE_PRIVATE_NAME";
    /// ClassStaticBlockBody early errors: `ContainsArguments` of the
    /// `ClassStaticBlockStatementList` is true. Nested ordinary function and
    /// method bodies are traversal boundaries; arrow functions are not.
    ClassStaticBlockContainsArguments => "E_CLASS_STATIC_BLOCK_CONTAINS_ARGUMENTS";
    /// 15.7.1. A ClassStaticBlockStatementList `Contains SuperCall`.
    /// Heritage is irrelevant. Ordinary callable bodies are traversal
    /// boundaries, nested classes contribute only computed property names,
    /// and ordinary and async arrows remain lexical traversal paths.
    ClassStaticBlockContainsSuperCall => "E_CLASS_STATIC_BLOCK_CONTAINS_SUPER_CALL";
    /// ClassStaticBlockBody early errors: `ContainsAwait` of the
    /// `ClassStaticBlockStatementList` is true. Nested ordinary and arrow
    /// function bodies are traversal boundaries.
    ClassStaticBlockContainsAwait => "E_CLASS_STATIC_BLOCK_CONTAINS_AWAIT";
    /// ClassElement early errors: a non-static public field or auto-accessor
    /// has the literal property name `constructor`. Computed property names
    /// remain excluded even when their evaluated key is `"constructor"`.
    ClassFieldConstructorName => "E_CLASS_FIELD_CONSTRUCTOR_NAME";
    /// ClassElement early errors: a static public field or auto-accessor has
    /// the literal property name `constructor` or `prototype`. Computed names
    /// remain excluded even when they evaluate to either String value.
    ClassStaticFieldConstructorOrPrototypeName => "E_CLASS_STATIC_FIELD_CONSTRUCTOR_OR_PROTOTYPE_NAME";
    /// FieldDefinition early errors: an initializer of a public/private,
    /// instance/static or auto-accessor field `Contains SuperCall`. Heritage
    /// does not change this restriction; `SuperProperty` remains valid.
    ClassFieldInitializerContainsSuperCall => "E_CLASS_FIELD_INITIALIZER_CONTAINS_SUPER_CALL";
    /// FieldDefinition early errors: `ContainsArguments` of a public/private,
    /// instance/static or auto-accessor initializer is true. Nested ordinary
    /// function and method bodies are boundaries; arrow functions are not.
    ClassFieldContainsArguments => "E_CLASS_FIELD_CONTAINS_ARGUMENTS";
    /// WithStatement early errors: the source text matched by the production
    /// is contained in strict-mode code. Modules and class methods are strict
    /// without a directive; sloppy Script code remains excluded.
    StrictModeWithStatement => "E_STRICT_MODE_WITH_STATEMENT";
    /// `delete UnaryExpression` early errors: the production is contained in
    /// strict-mode code and its recursively uncovered operand is an
    /// IdentifierReference. Sloppy identifier deletion remains excluded.
    StrictModeDeleteIdentifierReference => "E_STRICT_MODE_DELETE_IDENTIFIER_REFERENCE";
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
    /// `delete UnaryExpression` early errors: the production is contained in
    /// strict-mode code and its recursively uncovered operand is a direct or
    /// optional-chain private reference. Sloppy private syntax remains owned by
    /// the separate whole-source private-name validity condition.
    StrictModeDeletePrivateReference => "E_STRICT_MODE_DELETE_PRIVATE_REFERENCE";
    /// 16.1.1. `StatementList Contains NewTarget` is `true` for a ScriptBody.
    /// Ordinary/async/generator functions are traversal boundaries; arrows
    /// inherit `new.target` lexically and are not.
    ScriptTopLevelNewTarget => "E_SCRIPT_TOP_LEVEL_NEW_TARGET";
    /// 16.1.1. `StatementList Contains super` is `true` for a ScriptBody.
    /// Ordinary and async arrows inherit `super` lexically; method and
    /// constructor definitions establish their own `super` context.
    ScriptTopLevelSuper => "E_SCRIPT_TOP_LEVEL_SUPER";
    /// ScriptBody early errors: an immediate top-level lexical declaration is
    /// `using` or `await using`. Nested statement lists and the Module goal are
    /// deliberately excluded.
    ScriptTopLevelUsingDeclaration => "E_SCRIPT_TOP_LEVEL_USING_DECLARATION";
    /// 14.7.4.1 / 14.7.5.1. A classic `for` head's `LexicalDeclaration` or an
    /// iterable loop's `ForDeclaration` has a `BoundName` that also occurs in
    /// the body `Statement`'s `VarDeclaredNames`.
    ForHeadBodyDeclarationConflict => "E_FOR_HEAD_BODY_DECLARATION_CONFLICT";
    /// 14.7.5.1. The `BoundNames` of a `ForDeclaration` contains duplicate
    /// entries. This is reachable through `let`/`const` binding patterns;
    /// `var`, classic-for lexical declarations and resource bindings are
    /// deliberately excluded.
    ForDeclarationDuplicateBoundName => "E_FOR_DECLARATION_DUPLICATE_BOUND_NAME";
    /// 14.7.5.1. A `for-in` head's lexical declaration is `using` or
    /// `await using`. The `for-of` sibling deliberately remains valid.
    ForInUsingDeclaration => "E_FOR_IN_USING_DECLARATION";
    /// 14.12.1. A CaseClause or DefaultClause StatementList directly contains
    /// a `using` or `await using` declaration. Nested blocks are excluded.
    SwitchClauseUsingDeclaration => "E_SWITCH_CLAUSE_USING_DECLARATION";
    /// 15.5.1 / 15.6.1. A GeneratorDeclaration or
    /// AsyncGeneratorDeclaration's FormalParameters Contains YieldExpression.
    /// Generator expressions and methods have distinct parser producers.
    GeneratorDeclarationParametersContainYield => "E_GENERATOR_DECLARATION_PARAMETERS_CONTAIN_YIELD";
    /// An AsyncFunctionDeclaration or AsyncGeneratorDeclaration's
    /// FormalParameters Contains AwaitExpression. Expression forms and methods
    /// have distinct parser producers.
    AsyncDeclarationParametersContainAwait => "E_ASYNC_DECLARATION_PARAMETERS_CONTAIN_AWAIT";
    /// 15.5.1. A GeneratorExpression's FormalParameters Contains
    /// YieldExpression. Declarations, async-generator expressions and methods
    /// have distinct parser producers.
    GeneratorExpressionParametersContainYield => "E_GENERATOR_EXPRESSION_PARAMETERS_CONTAIN_YIELD";
    /// 15.6.1. An AsyncGeneratorExpression's FormalParameters Contains
    /// YieldExpression. Declarations, ordinary generator expressions and
    /// methods have distinct parser producers.
    AsyncGeneratorExpressionParametersContainYield => "E_ASYNC_GENERATOR_EXPRESSION_PARAMETERS_CONTAIN_YIELD";
    /// 15.6.1. An AsyncGeneratorExpression's FormalParameters Contains
    /// AwaitExpression. Declaration forms and async-generator methods have
    /// distinct parser producers.
    AsyncGeneratorExpressionParametersContainAwait => "E_ASYNC_GENERATOR_EXPRESSION_PARAMETERS_CONTAIN_AWAIT";
    /// A GeneratorMethod's UniqueFormalParameters Contains YieldExpression.
    /// Object and class methods share the same parser producer.
    GeneratorMethodParametersContainYield => "E_GENERATOR_METHOD_PARAMETERS_CONTAIN_YIELD";
    /// An AsyncGeneratorMethod's UniqueFormalParameters Contains
    /// YieldExpression. Object and class methods share the same parser
    /// producer.
    AsyncGeneratorMethodParametersContainYield => "E_ASYNC_GENERATOR_METHOD_PARAMETERS_CONTAIN_YIELD";
    /// An AsyncGeneratorMethod's UniqueFormalParameters Contains
    /// AwaitExpression. Object and class methods share the same parser
    /// producer.
    AsyncGeneratorMethodParametersContainAwait => "E_ASYNC_GENERATOR_METHOD_PARAMETERS_CONTAIN_AWAIT";
    /// An ordinary or async arrow's own parameters Contains YieldExpression.
    /// Pinned Boa's producer wordings map to the same closed condition.
    ArrowParametersContainYield => "E_ARROW_PARAMETERS_CONTAIN_YIELD";
    /// An ordinary or async arrow's own parameters Contains AwaitExpression.
    /// Pinned Boa's producer wordings map to the same closed condition.
    ArrowParametersContainAwait => "E_ARROW_PARAMETERS_CONTAIN_AWAIT";
    /// An AsyncFunctionExpression's FormalParameters Contains AwaitExpression.
    /// Declaration and async-generator expression forms have distinct
    /// producers.
    AsyncFunctionExpressionParametersContainAwait => "E_ASYNC_FUNCTION_EXPRESSION_PARAMETERS_CONTAIN_AWAIT";
    /// An AsyncMethod's UniqueFormalParameters Contains AwaitExpression.
    /// Object and class methods share the same parser producer.
    AsyncMethodParametersContainAwait => "E_ASYNC_METHOD_PARAMETERS_CONTAIN_AWAIT";
    /// 13.3.1.1. Source text matches either `?. TemplateLiteral` or
    /// `OptionalChain TemplateLiteral`. Parenthesizing a completed optional
    /// expression before using it as a tag remains excluded.
    OptionalChainTaggedTemplate => "E_OPTIONAL_CHAIN_TAGGED_TEMPLATE";
    /// 13.3.1.1. An ImportMeta production is parsed under a syntactic goal
    /// other than Module. Lexical nesting does not change the source goal;
    /// direct Module source remains excluded.
    ImportMetaOutsideModule => "E_IMPORT_META_OUTSIDE_MODULE";
    /// 16.2.2.1. `WithClauseToAttributes` of one `WithClause` contains two
    /// different entries with the same `[[Key]]`. Dynamic-import option
    /// objects are deliberately excluded.
    ModuleDuplicateImportAttributeKey => "E_MODULE_DUPLICATE_IMPORT_ATTRIBUTE_KEY";
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

/// The closed ways a `boa` static-semantics message may be recognized.
#[derive(Clone, Copy)]
enum ParseFailurePattern {
    /// Every fragment must occur, for a wording with invariant text separated
    /// by parser-owned interpolation.
    ContainsAll(&'static [&'static str]),
    /// The complete fixed wording must begin the rendered message. Boa may
    /// append its source position, but user-controlled text before the wording
    /// cannot forge the condition.
    StartsWith(&'static str),
    /// The complete rendered message must be byte-for-byte equal. Used when a
    /// fixed position is already part of the reviewed wording and decimal
    /// continuations such as `col 10` would make prefix matching too broad.
    Exact(&'static str),
}

/// One `boa` static-semantics message shape, and the code it denotes.
///
/// Private, along with the table it populates: the classification is a function
/// ([`classify_parse_failure`]), not a data structure callers walk themselves.
struct ParseFailureRule {
    /// The match semantics and parser-owned text for this rule.
    pattern: ParseFailurePattern,
    /// The condition this message shape denotes.
    code: EarlyErrorCode,
    /// Every message boa actually produces that this row must classify, copied
    /// verbatim from the cited source.
    ///
    /// A **list**, because one `ContainsAll` pattern legitimately covers
    /// several of boa's wordings for one spec rule. Never empty — P1. Consumed
    /// by P2 and P6, so it is not documentation: a row whose witnesses stop
    /// selecting it does not compile.
    witnesses: &'static [&'static str],
}

/// The row count, in the type. Adding a row without updating this is
/// `error[E0308]`, which is the moment to check the new row against P1/P2/P7.
const PARSE_FAILURE_RULE_COUNT: usize = 67;
const OPTIONAL_CHAIN_TAGGED_TEMPLATE_PREFIX: &str =
    "Invalid tagged template on optional chain at line";
const IMPORT_META_OUTSIDE_MODULE_PREFIX: &str =
    "invalid `import.meta` expression outside a module at line";
const FOR_HEAD_BODY_DECLARATION_CONFLICT_PREFIX: &str =
    "For loop initializer declared in loop body at line";
const FOR_DECLARATION_DUPLICATE_BOUND_NAME_PREFIX: &str =
    "For loop initializer cannot contain duplicate identifiers at line";
const LEXICAL_BOUND_NAME_LET_PREFIX: &str = "'let' is disallowed as a lexically bound name at line";
const FOR_DECLARATION_BOUND_NAME_LET_PREFIX: &str =
    "Cannot use 'let' as a lexically bound name at line";
const SCRIPT_TOP_LEVEL_SUPER_MESSAGE: &str = "invalid super usage at line 1, col 1";
const CLASS_BASE_CONSTRUCTOR_DIRECT_SUPER_PREFIX: &str =
    "base class constructor cannot contain direct super call at line";
const CLASS_STATIC_BLOCK_SUPER_CALL_PREFIX: &str =
    "class static block cannot contain super call at line";
const CLASS_FIELD_INITIALIZER_SUPER_CALL_PREFIX: &str =
    "class field initializer cannot contain super call at line";
const FUNCTION_EXPRESSION_CONTAINS_SUPER_PREFIX: &str =
    "function expression cannot contain super at line";
const FUNCTION_DECLARATION_CONTAINS_SUPER_PREFIX: &str =
    "function declaration cannot contain super at line";

/// The one message-pattern table.
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
        pattern: ParseFailurePattern::ContainsAll(&["Duplicate __proto__ fields"]),
        code: EarlyErrorCode::ObjectDuplicateProto,
        witnesses: &["Duplicate __proto__ fields are not allowed in object literals."],
    },
    // 2. boa_parser/src/parser/mod.rs:541
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["exported name", "declared multiple times"]),
        code: EarlyErrorCode::ModuleDuplicateExport,
        witnesses: &["exported name `x` declared multiple times"],
    },
    // 3. boa_parser/src/parser/mod.rs:556
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["could not find the exported binding"]),
        code: EarlyErrorCode::ModuleUndeclaredExport,
        witnesses: &["could not find the exported binding `x` in the declared names of the module"],
    },
    // 4. W2: boa_parser/src/parser/mod.rs:512,526 (module goal only).
    //    W1: boa_parser/src/parser/mod.rs:366,376; statement/block/mod.rs:109;
    //        statement/switch/mod.rs:88; the shared validator in
    //        statement/declaration/lexical.rs used by ordinary declarations
    //        and classic-for lexical heads; class_decl/mod.rs:718.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["lexical name", "declared multiple times"]),
        code: EarlyErrorCode::DuplicateLexicalDeclaration,
        witnesses: &[
            "lexical name `x` declared multiple times",
            "lexical name declared multiple times",
        ],
    },
    // 5. W3: statement/block/mod.rs:122; class_decl/mod.rs:730.
    //    W4: statement/switch/mod.rs:101.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["lexical name declared in var"]),
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
        pattern: ParseFailurePattern::ContainsAll(&["duplicate lexical declaration"]),
        code: EarlyErrorCode::DuplicateLexicalDeclaration,
        witnesses: &["invalid scope analysis: duplicate lexical declaration"],
    },
    // 7. boa_parser/src/parser/mod.rs:614. 15.2.1: BoundNames of FormalParameters
    //    intersects LexicallyDeclaredNames of FunctionBody.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["formal parameter", "declared in lexically declared names"]),
        code: EarlyErrorCode::DuplicateLexicalDeclaration,
        witnesses: &["formal parameter `x` declared in lexically declared names"],
    },
    // 8. Ten pinned Boa producer sites use this exact, case-sensitive wording
    //    for non-simple parameter lists and strict/context checks. See
    //    `duplicate-formal-parameter-early-errors.md` for the measured inventory.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["Duplicate parameter name not allowed in this context"]),
        code: EarlyErrorCode::DuplicateFormalParameter,
        witnesses: &["Duplicate parameter name not allowed in this context"],
    },
    // 9. boa_parser/src/parser/function/mod.rs:199, shared by every
    //    `UniqueFormalParameters` consumer. The lowercase `duplicate` is part
    //    of the pinned message contract.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["duplicate parameter name not allowed in unique formal parameters"]),
        code: EarlyErrorCode::DuplicateFormalParameter,
        witnesses: &["duplicate parameter name not allowed in unique formal parameters"],
    },
    // 10. boa_parser/src/parser/statement/try_stm/catch.rs:78. This is the sole
    //     pinned producer and exact, case-sensitive wording for duplicate
    //     `BoundNames` in a `CatchParameter`.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["duplicate catch parameter identifier"]),
        code: EarlyErrorCode::DuplicateCatchParameter,
        witnesses: &["duplicate catch parameter identifier"],
    },
    // 11. boa_parser/src/parser/statement/try_stm/catch.rs:99,108. The one
    //     exact, case-sensitive wording covers both the lexical-declaration
    //     branch and the binding-pattern/var-declaration branch.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["catch parameter identifier declared in catch body"]),
        code: EarlyErrorCode::CatchBodyDeclarationConflict,
        witnesses: &["catch parameter identifier declared in catch body"],
    },
    // 12. statement/declaration/hoistable/class_decl/mod.rs:319-324. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["a class may only have one constructor"]),
        code: EarlyErrorCode::DuplicateClassConstructor,
        witnesses: &["a class may only have one constructor"],
    },
    // 13. statement/declaration/hoistable/class_decl/mod.rs:789-795,853-858.
    //     These are the two pinned producers, for generator and async-generator
    //     methods, and they share this complete, case-sensitive wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["class constructor may not be a generator method"]),
        code: EarlyErrorCode::ClassConstructorGeneratorMethod,
        witnesses: &["class constructor may not be a generator method"],
    },
    // 14. statement/declaration/hoistable/class_decl/mod.rs:893-899. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["class constructor may not be an async method"]),
        code: EarlyErrorCode::ClassConstructorAsyncMethod,
        witnesses: &["class constructor may not be an async method"],
    },
    // 15. statement/declaration/hoistable/class_decl/mod.rs:1158-1164. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["class constructor may not be a getter method"]),
        code: EarlyErrorCode::ClassConstructorGetter,
        witnesses: &["class constructor may not be a getter method"],
    },
    // 16. statement/declaration/hoistable/class_decl/mod.rs:1260-1266. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["class constructor may not be a setter method"]),
        code: EarlyErrorCode::ClassConstructorSetter,
        witnesses: &["class constructor may not be a setter method"],
    },
    // 17. statement/declaration/hoistable/class_decl/mod.rs:813-819,
    //     846-852,921-927,989-995,1122-1128,1223-1229,1319-1325. These seven
    //     pinned producers cover private fields and every private method form
    //     with one complete, case-sensitive wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["class constructor may not be a private method"]),
        code: EarlyErrorCode::ClassPrivateConstructorName,
        witnesses: &["class constructor may not be a private method"],
    },
    // 18. statement/declaration/hoistable/class_decl/mod.rs:809-815,874-882,
    //     915-923,1197-1201,1298-1302,1450-1455. These six public static
    //     method/accessor branches share one complete, case-sensitive wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["class may not have static method definitions named 'prototype'"]),
        code: EarlyErrorCode::ClassStaticMethodPrototypeName,
        witnesses: &["class may not have static method definitions named 'prototype'"],
    },
    // 19. statement/declaration/hoistable/class_decl/mod.rs:367-467. Five
    //     pinned branches use this exact, case-sensitive wording for duplicate
    //     private methods, accessors and fields.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["private identifier has already been declared"]),
        code: EarlyErrorCode::ClassDuplicatePrivateName,
        witnesses: &["private identifier has already been declared"],
    },
    // 20. statement/declaration/hoistable/class_decl/mod.rs:740-745. This is
    //     the sole pinned producer and its complete, case-sensitive wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["'arguments' not allowed in class static block"]),
        code: EarlyErrorCode::ClassStaticBlockContainsArguments,
        witnesses: &["'arguments' not allowed in class static block"],
    },
    // 21. statement/declaration/hoistable/class_decl/mod.rs:762-764. The
    //     adjacent `at line` fragment is part of Error::General's rendered
    //     message and excludes the distinct longer generator-parameter error.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["invalid await usage at line"]),
        code: EarlyErrorCode::ClassStaticBlockContainsAwait,
        witnesses: &["invalid await usage at line 1, col 1"],
    },
    // 22. statement/declaration/hoistable/class_decl/mod.rs:1065,1099,1421,
    //     1505. These four pinned branches cover ordinary public fields and
    //     public auto-accessors, with and without initializers.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["class may not have field definitions named 'constructor'"]),
        code: EarlyErrorCode::ClassFieldConstructorName,
        witnesses: &["class may not have field definitions named 'constructor'"],
    },
    // 23. statement/declaration/hoistable/class_decl/mod.rs:1059,1093,1415,
    //     1499. These four corresponding static branches share one complete,
    //     case-sensitive wording for the two forbidden literal names.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "class may not have static field definitions named 'constructor' or 'prototype'",
        ]),
        code: EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName,
        witnesses: &[
            "class may not have static field definitions named 'constructor' or 'prototype'",
        ],
    },
    // 24. statement/declaration/hoistable/class_decl/mod.rs:1525-1561. The
    //     exhaustive class-element match uses this one exact wording for
    //     public/private, instance/static and auto-accessor field initializers.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["'arguments' not allowed in class field definition"]),
        code: EarlyErrorCode::ClassFieldContainsArguments,
        witnesses: &["'arguments' not allowed in class field definition"],
    },
    // 25. statement/with/mod.rs:61-67. The sole pinned producer uses this
    //     complete, case-sensitive wording for WithStatement in strict code.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["with statement not allowed in strict mode"]),
        code: EarlyErrorCode::StrictModeWithStatement,
        witnesses: &["with statement not allowed in strict mode"],
    },
    // 26. boa_parser/src/parser/mod.rs:567
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["module cannot contain", "super"]),
        code: EarlyErrorCode::ModuleTopLevelSuper,
        witnesses: &["module cannot contain `super` on the top-level"],
    },
    // 27. boa_parser/src/parser/mod.rs:575
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["module cannot contain", "new.target"]),
        code: EarlyErrorCode::ModuleTopLevelNewTarget,
        witnesses: &["module cannot contain `new.target` on the top-level"],
    },
    // 28. boa_parser/src/parser/mod.rs:462,593; statement/mod.rs:1020.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["invalid private identifier usage"]),
        code: EarlyErrorCode::InvalidPrivateIdentifier,
        witnesses: &["invalid private identifier usage"],
    },
    // 29-33. `CheckLabelsError::message`, boa_ast/src/operations/mod.rs:1399-1417.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["duplicate label"]),
        code: EarlyErrorCode::DuplicateLabel,
        witnesses: &["duplicate label: lbl"],
    },
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["undefined break target"]),
        code: EarlyErrorCode::UndefinedBreakTarget,
        witnesses: &["undefined break target: lbl"],
    },
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["undefined continue target"]),
        code: EarlyErrorCode::UndefinedContinueTarget,
        witnesses: &["undefined continue target: lbl"],
    },
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["illegal break statement"]),
        code: EarlyErrorCode::IllegalBreak,
        witnesses: &["illegal break statement"],
    },
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["illegal continue statement"]),
        code: EarlyErrorCode::IllegalContinue,
        witnesses: &["illegal continue statement"],
    },
    // 34. boa_parser/src/parser/mod.rs:475-479, function/mod.rs:507-511,
    //     statement/mod.rs:1005-1010 and class_decl/mod.rs:767-771. All four
    //     fixed messages report the same surviving CoverInitializedName AST
    //     condition in a different statement-list context.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["invalid object literal in"]),
        code: EarlyErrorCode::ObjectLiteralCoverInitializedName,
        witnesses: &[
            "invalid object literal in script statement list at line 1, col 1",
            "invalid object literal in function statement list at line 1, col 1",
            "invalid object literal in module item list at line 1, col 1",
            "invalid object literal in class static block statement list at line 1, col 1",
        ],
    },
    // 35. boa_parser/src/parser/mod.rs:447-454. The sole ScriptBody producer
    //     uses one raw message and the fixed Position::new(1, 1), so the full
    //     rendered text is stable and disjoint from the Module producer.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["invalid new.target usage at line 1, col 1"]),
        code: EarlyErrorCode::ScriptTopLevelNewTarget,
        witnesses: &["invalid new.target usage at line 1, col 1"],
    },
    // 36. boa_parser/src/parser/mod.rs:429-437. Lila's ordinary Script entry
    //     selects this fixed branch and position; the sibling direct-eval
    //     wording is a separate T13 boundary.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "`using` declarations are not allowed at the top level of scripts at line 1, col 1",
        ]),
        code: EarlyErrorCode::ScriptTopLevelUsingDeclaration,
        witnesses: &[
            "`using` declarations are not allowed at the top level of scripts at line 1, col 1",
        ],
    },
    // 37. statement/iteration/for_statement.rs::
    //     initializer_to_iterable_loop_initializer. One fixed LexError message
    //     owns both using-declaration variants; `at line` keeps it disjoint
    //     from Boa's other using restrictions without fixing a source
    //     coordinate.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["using declarations are not allowed in for-in loop heads at line"]),
        code: EarlyErrorCode::ForInUsingDeclaration,
        witnesses: &["using declarations are not allowed in for-in loop heads at line 1, col 1"],
    },
    // 38. statement/mod.rs:445-453, selected only by the CaseClause and
    //     DefaultClause StatementLists in statement/switch/mod.rs:168-202.
    //     The body is fixed and `at line` admits the declaration position
    //     without overlapping Boa's other using-declaration restrictions.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["`using` declarations are not allowed in this statement list at line"]),
        code: EarlyErrorCode::SwitchClauseUsingDeclaration,
        witnesses: &[
            "`using` declarations are not allowed in this statement list at line 1, col 1",
        ],
    },
    // 39. statement/declaration/hoistable/mod.rs:241-247. Ordinary and async
    //     generator declarations opt into this shared fixed-message check.
    //     Generator expressions and methods have distinct pinned wordings.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["invalid yield usage in generator function parameters at line"]),
        code: EarlyErrorCode::GeneratorDeclarationParametersContainYield,
        witnesses: &["invalid yield usage in generator function parameters at line 1, col 1"],
    },
    // 40. statement/declaration/hoistable/mod.rs:251-257. Async-function and
    //     async-generator declarations opt into this shared fixed-message
    //     check. Expression forms and methods have distinct producers.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["invalid await usage in generator function parameters at line"]),
        code: EarlyErrorCode::AsyncDeclarationParametersContainAwait,
        witnesses: &["invalid await usage in generator function parameters at line 1, col 1"],
    },
    // 41. expression/primary/generator_expression/mod.rs:144-150. The
    //     ordinary GeneratorExpression parser owns this sole fixed message;
    //     declarations, async generators and methods use distinct wordings.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["generator expression cannot contain yield expression in parameters at line"]),
        code: EarlyErrorCode::GeneratorExpressionParametersContainYield,
        witnesses: &[
            "generator expression cannot contain yield expression in parameters at line 1, col 1",
        ],
    },
    // 42. expression/primary/async_generator_expression/mod.rs:99-106. The
    //     async GeneratorExpression parser owns this sole fixed message; its
    //     adjacent AwaitExpression check and every other form are distinct.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "yield expression not allowed in async generator expression parameters at line",
        ]),
        code: EarlyErrorCode::AsyncGeneratorExpressionParametersContainYield,
        witnesses: &[
            "yield expression not allowed in async generator expression parameters at line 1, col 1",
        ],
    },
    // 43. expression/primary/async_generator_expression/mod.rs:109-114. The
    //     async GeneratorExpression parser owns this sole fixed message;
    //     declaration forms and methods use distinct wordings.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "await expression not allowed in async generator expression parameters at line",
        ]),
        code: EarlyErrorCode::AsyncGeneratorExpressionParametersContainAwait,
        witnesses: &[
            "await expression not allowed in async generator expression parameters at line 1, col 1",
        ],
    },
    // 44. expression/primary/object_initializer/mod.rs:779-786. One
    //     GeneratorMethod parser serves object literals and class elements.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "yield expression not allowed in generator method definition parameters at line",
        ]),
        code: EarlyErrorCode::GeneratorMethodParametersContainYield,
        witnesses: &[
            "yield expression not allowed in generator method definition parameters at line 1, col 1",
        ],
    },
    // 45. expression/primary/object_initializer/mod.rs:868-876. One
    //     AsyncGeneratorMethod parser serves object literals and class
    //     elements; its await sibling has a distinct fixed message.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "yield expression not allowed in async generator method definition parameters at line",
        ]),
        code: EarlyErrorCode::AsyncGeneratorMethodParametersContainYield,
        witnesses: &[
            "yield expression not allowed in async generator method definition parameters at line 1, col 1",
        ],
    },
    // 46. expression/primary/object_initializer/mod.rs:879-885. The same
    //     AsyncGeneratorMethod parser owns this adjacent await condition.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "await expression not allowed in async generator method definition parameters at line",
        ]),
        code: EarlyErrorCode::AsyncGeneratorMethodParametersContainAwait,
        witnesses: &[
            "await expression not allowed in async generator method definition parameters at line 1, col 1",
        ],
    },
    // 47. expression/primary/mod.rs:571-575. Converting a parenthesized cover
    //     expression into ordinary-arrow parameters rejects contained Yield
    //     before assignment/mod.rs can reach its sibling fixed message.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "yield expression is not allowed in formal parameter list of arrow function at line",
        ]),
        code: EarlyErrorCode::ArrowParametersContainYield,
        witnesses: &[
            "yield expression is not allowed in formal parameter list of arrow function at line 1, col 1",
        ],
    },
    // 48. expression/assignment/mod.rs:243-248,
    //     assignment/arrow_function.rs:107-112 and
    //     assignment/async_arrow_function.rs:114-119. This sibling wording
    //     maps to the same typed condition across arrow forms.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["Yield expression not allowed in this context at line"]),
        code: EarlyErrorCode::ArrowParametersContainYield,
        witnesses: &["Yield expression not allowed in this context at line 1, col 1"],
    },
    // 49. expression/assignment/mod.rs:251-256,
    //     assignment/arrow_function.rs:115-120 and
    //     assignment/async_arrow_function.rs:122-127. The uppercase fixed
    //     wording stays disjoint from generator-expression/method messages.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["Await expression not allowed in this context at line"]),
        code: EarlyErrorCode::ArrowParametersContainAwait,
        witnesses: &["Await expression not allowed in this context at line 1, col 1"],
    },
    // 50. expression/primary/async_function_expression/mod.rs. Lila's
    //     vendored producer repairs the missing FormalParameters Contains
    //     AwaitExpression check before body parsing.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "await expression not allowed in async function expression parameters at line",
        ]),
        code: EarlyErrorCode::AsyncFunctionExpressionParametersContainAwait,
        witnesses: &[
            "await expression not allowed in async function expression parameters at line 1, col 1",
        ],
    },
    // 51. expression/primary/object_initializer/mod.rs. Lila's vendored
    //     AsyncMethod producer repairs the corresponding UniqueFormalParameters
    //     check for object and class methods.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["await expression not allowed in async method definition parameters at line"]),
        code: EarlyErrorCode::AsyncMethodParametersContainAwait,
        witnesses: &[
            "await expression not allowed in async method definition parameters at line 1, col 1",
        ],
    },
    // 52. Sixteen spec-applicable, error-reachable parser sites share this
    //     exact raw message. LexError::Syntax appends the source position;
    //     `at line` admits it without broadening to a partial wording.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&[
            "Illegal 'use strict' directive in function with non-simple parameter list at line",
        ]),
        code: EarlyErrorCode::CallableNonSimpleParametersContainUseStrict,
        witnesses: &[
            "Illegal 'use strict' directive in function with non-simple parameter list at line 1, col 1",
        ],
    },
    // 53. expression/unary.rs:92-98. The sole pinned producer recursively
    //     uncovers parentheses and couples the identifier shape to strictness.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["cannot delete variables in strict mode at line"]),
        code: EarlyErrorCode::StrictModeDeleteIdentifierReference,
        witnesses: &["cannot delete variables in strict mode at line 1, col 1"],
    },
    // 54. expression/unary.rs. Lila repairs the adjacent pinned producer to
    //     share the strictness guard and recognize private-ending optional chains.
    ParseFailureRule {
        pattern: ParseFailurePattern::ContainsAll(&["cannot delete private fields at line"]),
        code: EarlyErrorCode::StrictModeDeletePrivateReference,
        witnesses: &["cannot delete private fields at line 1, col 1"],
    },
    // 55. statement/declaration/import.rs:336 and export.rs:284. Static
    //     import and export-from attributes share this complete raw message.
    //     Anchoring both distinguishes the separate keyed lila-ir record error
    //     and prevents a user-chosen local export name from injecting this text
    //     into another Error::general diagnostic.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith("duplicate import attribute key at line"),
        code: EarlyErrorCode::ModuleDuplicateImportAttributeKey,
        witnesses: &["duplicate import attribute key at line 1, col 1"],
    },
    // 56. expression/left_hand_side/optional/mod.rs:130,163. The two
    //     OptionalChain tagged-template productions share this complete raw
    //     message. Anchoring prevents an interpolated Module export name from
    //     injecting the wording into another Error::general diagnostic.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(OPTIONAL_CHAIN_TAGGED_TEMPLATE_PREFIX),
        code: EarlyErrorCode::OptionalChainTaggedTemplate,
        witnesses: &["Invalid tagged template on optional chain at line 1, col 1"],
    },
    // 57. expression/left_hand_side/member.rs:105-109. The sole producer
    //     rejects ImportMeta only when the retained source goal is not Module.
    //     Anchoring prevents user-controlled export text from forging the fixed
    //     message later inside a different diagnostic.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(IMPORT_META_OUTSIDE_MODULE_PREFIX),
        code: EarlyErrorCode::ImportMetaOutsideModule,
        witnesses: &[
            "invalid `import.meta` expression outside a module at line 1, col 1",
        ],
    },
    // 58. statement/iteration/for_statement.rs::ForStatement::parse and
    //     ::parse_iterable_loop_tail. The classic-for LexicalDeclaration and
    //     iterable-loop ForDeclaration producers compute the same BoundNames /
    //     body VarDeclaredNames intersection and emit the same fixed raw
    //     message.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(FOR_HEAD_BODY_DECLARATION_CONFLICT_PREFIX),
        code: EarlyErrorCode::ForHeadBodyDeclarationConflict,
        witnesses: &["For loop initializer declared in loop body at line 1, col 1"],
    },
    // 59. statement/iteration/for_statement.rs::parse_iterable_loop_tail. The
    //     sole producer traverses one iterable-loop ForDeclaration's
    //     BoundNames through an FxHashSet and emits this fixed message when
    //     insertion finds a duplicate.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(
            FOR_DECLARATION_DUPLICATE_BOUND_NAME_PREFIX,
        ),
        code: EarlyErrorCode::ForDeclarationDuplicateBoundName,
        witnesses: &[
            "For loop initializer cannot contain duplicate identifiers at line 1, col 1",
        ],
    },
    // 60. statement/declaration/lexical.rs::
    //     LexicalDeclaration::validate_bound_name_let. The shared validator
    //     owns ordinary declarations and classic-for lexical heads after the
    //     latter's delimiter resolves the ambiguous grammar.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(LEXICAL_BOUND_NAME_LET_PREFIX),
        code: EarlyErrorCode::LexicalBoundNameLet,
        witnesses: &["'let' is disallowed as a lexically bound name at line 1, col 1"],
    },
    // 61. statement/iteration/for_statement.rs::parse_iterable_loop_tail.
    //     The distinct ForDeclaration producer runs before head/body conflict
    //     and duplicate-BoundName validation.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(FOR_DECLARATION_BOUND_NAME_LET_PREFIX),
        code: EarlyErrorCode::LexicalBoundNameLet,
        witnesses: &["Cannot use 'let' as a lexically bound name at line 1, col 1"],
    },
    // 62. boa_parser/src/parser/mod.rs::ScriptBody::parse. The sole Script
    //     producer uses Error::general with a fixed Position::new(1, 1).
    //     Exact matching is required: StartsWith would also absorb the other
    //     raw-message producers when they report columns 10 through 19.
    ParseFailureRule {
        pattern: ParseFailurePattern::Exact(SCRIPT_TOP_LEVEL_SUPER_MESSAGE),
        code: EarlyErrorCode::ScriptTopLevelSuper,
        witnesses: &["invalid super usage at line 1, col 1"],
    },
    // 63. statement/declaration/hoistable/class_decl/mod.rs:203-211. The
    //     absent-heritage / constructor-present / HasDirectSuper conjunction
    //     is the sole owner of this complete fixed message body.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(
            CLASS_BASE_CONSTRUCTOR_DIRECT_SUPER_PREFIX,
        ),
        code: EarlyErrorCode::ClassBaseConstructorHasDirectSuper,
        witnesses: &[
            "base class constructor cannot contain direct super call at line 2, col 1",
        ],
    },
    // 64. statement/declaration/hoistable/class_decl/mod.rs:755-760. The
    //     static-block statement-list Contains SuperCall branch is the sole
    //     owner of this complete fixed message body.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(CLASS_STATIC_BLOCK_SUPER_CALL_PREFIX),
        code: EarlyErrorCode::ClassStaticBlockContainsSuperCall,
        witnesses: &[
            "class static block cannot contain super call at line 2, col 1",
        ],
    },
    // 65. statement/declaration/hoistable/class_decl/mod.rs:436-493. Four
    //     exhaustive class-element arms share the one FieldDefinition
    //     initializer Contains SuperCall condition and fixed message body.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(
            CLASS_FIELD_INITIALIZER_SUPER_CALL_PREFIX,
        ),
        code: EarlyErrorCode::ClassFieldInitializerContainsSuperCall,
        witnesses: &[
            "class field initializer cannot contain super call at line 2, col 1",
        ],
    },
    // 66. expression/primary/function_expression/mod.rs. The sole ordinary
    //     FunctionExpression producer applies Contains Super to the completed
    //     node, covering parameters and body plus SuperProperty and SuperCall.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(FUNCTION_EXPRESSION_CONTAINS_SUPER_PREFIX),
        code: EarlyErrorCode::FunctionExpressionContainsSuper,
        witnesses: &["function expression cannot contain super at line 1, col 11"],
    },
    // 67. statement/declaration/hoistable/function_decl/mod.rs and the shared
    //     parse_callable_declaration predicate. The ordinary FunctionDeclaration
    //     trait implementation supplies this message only for its parameters /
    //     body Contains Super condition.
    ParseFailureRule {
        pattern: ParseFailurePattern::StartsWith(FUNCTION_DECLARATION_CONTAINS_SUPER_PREFIX),
        code: EarlyErrorCode::FunctionDeclarationContainsSuper,
        witnesses: &["function declaration cannot contain super at line 1, col 12"],
    },
];

/// Slice view of [`PARSE_FAILURE_RULE_TABLE`], so the walkers below index a
/// `&'static [_]` rather than copying the array on every call.
const PARSE_FAILURE_RULES: &[ParseFailureRule] = &PARSE_FAILURE_RULE_TABLE;

const fn rule_matches(rule: &ParseFailureRule, message: &str) -> bool {
    match rule.pattern {
        ParseFailurePattern::ContainsAll(fragments) => {
            let mut i = 0;
            while i < fragments.len() {
                if !contains_sub(message, fragments[i]) {
                    return false;
                }
                i += 1;
            }
            true
        }
        ParseFailurePattern::StartsWith(prefix) => starts_with_sub(message, prefix),
        ParseFailurePattern::Exact(expected) => str_eq(message, expected),
    }
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
    /// The gate. `None` for a code no row of the message-pattern table carries — i.e.
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
/// `ContainsAll` row's whole fragment set verbatim, and the string oracle would
/// classify an ordinary syntax error as an early error. On the entry path that is
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
/// This is the only such function in the workspace. `lila-front::parse` calls
/// it at the sole Boa parse boundary; downstream crates consume the retained
/// typed code.
#[must_use]
pub const fn classify_parse_failure(message: &str) -> Option<ParseClassified> {
    // ContainsAll patterns inspect messages Boa can build by interpolation.
    // Refuse to read a known shape that can carry user source text into that
    // oracle; `Malformed`/`Unsupported` is the honest answer there. StartsWith
    // and Exact patterns additionally prevent source text later in a General
    // diagnostic from forging a complete fixed message.
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
    /// True iff some row of the one message-pattern table can produce this code — i.e.
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

/// True only when at least one row owns `code` and every such row is anchored.
const fn code_is_owned_only_by_starts_with(code: EarlyErrorCode) -> bool {
    let mut found = false;
    let mut i = 0;
    while i < PARSE_FAILURE_RULES.len() {
        let rule = &PARSE_FAILURE_RULES[i];
        if code_eq(rule.code, code) {
            found = true;
            if !matches!(rule.pattern, ParseFailurePattern::StartsWith(_)) {
                return false;
            }
        }
        i += 1;
    }
    found
}

/// True only when exactly one row owns `code` and that row uses the complete
/// reviewed prefix. This is stronger than checking the pattern variant: a
/// shortened prefix can remain anchored while absorbing unrelated diagnostics.
const fn code_is_owned_once_by_exact_starts_with(
    code: EarlyErrorCode,
    expected_prefix: &str,
) -> bool {
    let mut owners = 0;
    let mut i = 0;
    while i < PARSE_FAILURE_RULES.len() {
        let rule = &PARSE_FAILURE_RULES[i];
        if code_eq(rule.code, code) {
            match rule.pattern {
                ParseFailurePattern::StartsWith(prefix) => {
                    if !str_eq(prefix, expected_prefix) {
                        return false;
                    }
                    owners += 1;
                }
                ParseFailurePattern::ContainsAll(_) | ParseFailurePattern::Exact(_) => {
                    return false;
                }
            }
        }
        i += 1;
    }
    owners == 1
}

/// True only when exactly one row owns `code` and that row requires byte-for-
/// byte equality with the independently reviewed complete message.
const fn code_is_owned_once_by_exact_message(code: EarlyErrorCode, expected_message: &str) -> bool {
    let mut owners = 0;
    let mut i = 0;
    while i < PARSE_FAILURE_RULES.len() {
        let rule = &PARSE_FAILURE_RULES[i];
        if code_eq(rule.code, code) {
            match rule.pattern {
                ParseFailurePattern::Exact(message) => {
                    if !str_eq(message, expected_message) {
                        return false;
                    }
                    owners += 1;
                }
                ParseFailurePattern::ContainsAll(_) | ParseFailurePattern::StartsWith(_) => {
                    return false;
                }
            }
        }
        i += 1;
    }
    owners == 1
}

/// True only when exactly two independently spelled anchored rows own `code`,
/// one for each complete reviewed prefix.
const fn code_is_owned_twice_by_exact_starts_with(
    code: EarlyErrorCode,
    first_prefix: &str,
    second_prefix: &str,
) -> bool {
    if str_eq(first_prefix, second_prefix) {
        return false;
    }

    let mut owners = 0;
    let mut first_owners = 0;
    let mut second_owners = 0;
    let mut i = 0;
    while i < PARSE_FAILURE_RULES.len() {
        let rule = &PARSE_FAILURE_RULES[i];
        if code_eq(rule.code, code) {
            owners += 1;
            match rule.pattern {
                ParseFailurePattern::StartsWith(prefix) => {
                    if str_eq(prefix, first_prefix) {
                        first_owners += 1;
                    } else if str_eq(prefix, second_prefix) {
                        second_owners += 1;
                    } else {
                        return false;
                    }
                }
                ParseFailurePattern::ContainsAll(_) | ParseFailurePattern::Exact(_) => {
                    return false;
                }
            }
        }
        i += 1;
    }
    owners == 2 && first_owners == 1 && second_owners == 1
}

// These conditions are intentionally parse-owned. Deleting any table row while
// leaving its enum variant must fail during `cargo check`, not merely change a
// retained dependency rejection from EarlyError back to Unsupported at run time.
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::CallableNonSimpleParametersContainUseStrict);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::StrictModeDeleteIdentifierReference);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::StrictModeDeletePrivateReference);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::OptionalChainTaggedTemplate);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ImportMetaOutsideModule);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ForHeadBodyDeclarationConflict);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ForDeclarationDuplicateBoundName);
const _: ParseClassified = ParseClassified::from_parse_table(EarlyErrorCode::LexicalBoundNameLet);
const _: ParseClassified = ParseClassified::from_parse_table(EarlyErrorCode::ScriptTopLevelSuper);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ClassBaseConstructorHasDirectSuper);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ClassStaticBlockContainsSuperCall);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ClassFieldInitializerContainsSuperCall);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::FunctionExpressionContainsSuper);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::FunctionDeclarationContainsSuper);
const _: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ModuleDuplicateImportAttributeKey);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::OptionalChainTaggedTemplate,
        "Invalid tagged template on optional chain at line",
    ),
    "the optional-chain tagged-template code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::ImportMetaOutsideModule,
        "invalid `import.meta` expression outside a module at line",
    ),
    "the ImportMeta outside-Module code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::ForHeadBodyDeclarationConflict,
        "For loop initializer declared in loop body at line",
    ),
    "the for-head/body declaration-conflict code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::ForDeclarationDuplicateBoundName,
        "For loop initializer cannot contain duplicate identifiers at line",
    ),
    "the ForDeclaration duplicate-BoundName code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_twice_by_exact_starts_with(
        EarlyErrorCode::LexicalBoundNameLet,
        "'let' is disallowed as a lexically bound name at line",
        "Cannot use 'let' as a lexically bound name at line",
    ),
    "the lexical BoundName let code must have exactly its two independently spelled anchored owners"
);
const _: () = assert!(
    code_is_owned_once_by_exact_message(
        EarlyErrorCode::ScriptTopLevelSuper,
        "invalid super usage at line 1, col 1",
    ),
    "the ScriptBody super code must have one owner using its complete reviewed message"
);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::ClassBaseConstructorHasDirectSuper,
        "base class constructor cannot contain direct super call at line",
    ),
    "the base-constructor direct-super code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::ClassStaticBlockContainsSuperCall,
        "class static block cannot contain super call at line",
    ),
    "the class-static-block SuperCall code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::ClassFieldInitializerContainsSuperCall,
        "class field initializer cannot contain super call at line",
    ),
    "the class-field initializer SuperCall code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::FunctionExpressionContainsSuper,
        "function expression cannot contain super at line",
    ),
    "the FunctionExpression super code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_once_by_exact_starts_with(
        EarlyErrorCode::FunctionDeclarationContainsSuper,
        "function declaration cannot contain super at line",
    ),
    "the FunctionDeclaration super code must have one owner using its complete reviewed prefix"
);
const _: () = assert!(
    code_is_owned_only_by_starts_with(EarlyErrorCode::ModuleDuplicateImportAttributeKey),
    "the fixed duplicate import-attribute message must use anchored prefix matching"
);

// ---------------------------------------------------------------------------
// Const assertions P1-P6. Each names the mistake it makes fail to build.
// ---------------------------------------------------------------------------

/// P1: no row has an empty match pattern or `witnesses` list, **and no pattern
/// component or witness is the empty string**.
///
/// An empty `ContainsAll` list matches every message, so one row would swallow
/// every parse failure into a single code. An empty `StartsWith` prefix does the
/// same; an empty `Exact` message cannot own a real diagnostic. An empty
/// `witnesses` list would exempt the row from P2, P6 and P10 entirely. A single
/// empty fragment also matches everything because `contains_sub` follows
/// `str::contains` semantics.
const fn every_row_is_populated() -> bool {
    let mut i = 0;
    while i < PARSE_FAILURE_RULES.len() {
        let rule = &PARSE_FAILURE_RULES[i];
        if rule.witnesses.is_empty() {
            return false;
        }
        match rule.pattern {
            ParseFailurePattern::ContainsAll(fragments) => {
                if fragments.is_empty() {
                    return false;
                }
                let mut f = 0;
                while f < fragments.len() {
                    if fragments[f].is_empty() {
                        return false;
                    }
                    f += 1;
                }
            }
            ParseFailurePattern::StartsWith(prefix) => {
                if prefix.is_empty() {
                    return false;
                }
            }
            ParseFailurePattern::Exact(message) => {
                if message.is_empty() {
                    return false;
                }
            }
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
/// above an existing one cannot silently shadow it, and a row whose pattern is
/// a superset of another's cannot be silently unreachable.
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

/// P11: user-controlled text inside another `Error::general` cannot forge the
/// fixed import-attribute condition, including where it overlaps the existing
/// duplicate-export fragments.
const fn import_attribute_prefix_is_injection_safe() -> bool {
    if !matches!(
        classify_parse_failure(
            "local referenced binding `duplicate import attribute key at line` cannot be a string literal at line 1, col 1",
        ),
        None
    ) {
        return false;
    }
    classified_is(
        classify_parse_failure(
            "exported name `duplicate import attribute key at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    )
}

/// P12: the fixed optional-chain wording remains unforgeable when a Module
/// export name carries it inside a different `Error::general` diagnostic.
const fn optional_chain_tagged_template_prefix_is_injection_safe() -> bool {
    if !matches!(
        classify_parse_failure(
            "local referenced binding `Invalid tagged template on optional chain at line` cannot be a string literal at line 1, col 1",
        ),
        None
    ) {
        return false;
    }
    classified_is(
        classify_parse_failure(
            "exported name `Invalid tagged template on optional chain at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    )
}

/// P13: the fixed ImportMeta goal-error wording remains unforgeable when a
/// Module export name carries it inside another `Error::general` diagnostic.
const fn import_meta_outside_module_prefix_is_injection_safe() -> bool {
    if !matches!(
        classify_parse_failure(
            "local referenced binding `invalid `import.meta` expression outside a module at line` cannot be a string literal at line 1, col 1",
        ),
        None
    ) {
        return false;
    }
    classified_is(
        classify_parse_failure(
            "exported name `invalid `import.meta` expression outside a module at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    )
}

/// P14: neither fixed lexical-BoundName wording can be forged by a
/// user-controlled Module export name inside another diagnostic.
const fn lexical_bound_name_let_prefixes_are_injection_safe() -> bool {
    classified_is(
        classify_parse_failure(
            "exported name `'let' is disallowed as a lexically bound name at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    ) && classified_is(
        classify_parse_failure(
            "exported name `Cannot use 'let' as a lexically bound name at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    )
}

/// P15: the complete fixed ScriptBody message cannot be injected into another
/// diagnostic, and decimal continuations of its final column do not compare
/// equal to it.
const fn script_top_level_super_message_is_exact_and_injection_safe() -> bool {
    if !matches!(
        classify_parse_failure("invalid super usage at line 1, col 2"),
        None
    ) || !matches!(
        classify_parse_failure("invalid super usage at line 1, col 10"),
        None
    ) {
        return false;
    }
    classified_is(
        classify_parse_failure(
            "exported name `invalid super usage at line 1, col 1` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    )
}

/// P16: the three class-owned SuperCall prefixes remain distinct from each
/// other and from the adjacent generic Script/callable/method wordings. A
/// user-controlled Module export name carrying any complete prefix must
/// remain owned by the duplicate-export condition.
const fn class_super_call_prefixes_are_distinct_and_injection_safe() -> bool {
    if !classified_is(
        classify_parse_failure(
            "base class constructor cannot contain direct super call at line 2, col 1",
        ),
        EarlyErrorCode::ClassBaseConstructorHasDirectSuper,
    ) || !classified_is(
        classify_parse_failure("class static block cannot contain super call at line 2, col 1"),
        EarlyErrorCode::ClassStaticBlockContainsSuperCall,
    ) || !classified_is(
        classify_parse_failure(
            "class field initializer cannot contain super call at line 2, col 1",
        ),
        EarlyErrorCode::ClassFieldInitializerContainsSuperCall,
    ) || !matches!(
        classify_parse_failure("invalid super call usage at line 1, col 1"),
        None
    ) {
        return false;
    }

    classified_is(
        classify_parse_failure(
            "exported name `base class constructor cannot contain direct super call at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    ) && classified_is(
        classify_parse_failure(
            "exported name `class static block cannot contain super call at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    ) && classified_is(
        classify_parse_failure(
            "exported name `class field initializer cannot contain super call at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    )
}

/// P17: the ordinary FunctionExpression prefix remains distinct from the
/// adjacent generic declaration/generator/async-expression and method
/// producers. User-controlled Module export text carrying the complete prefix
/// must remain owned by the duplicate-export condition.
const fn function_expression_super_prefix_is_distinct_and_injection_safe() -> bool {
    if !classified_is(
        classify_parse_failure("function expression cannot contain super at line 1, col 11"),
        EarlyErrorCode::FunctionExpressionContainsSuper,
    ) || !matches!(
        classify_parse_failure("invalid super usage at line 1, col 11"),
        None
    ) || !matches!(
        classify_parse_failure("invalid super call usage at line 1, col 11"),
        None
    ) {
        return false;
    }

    classified_is(
        classify_parse_failure(
            "exported name `function expression cannot contain super at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    )
}

/// P18: the ordinary FunctionDeclaration prefix remains distinct from the
/// FunctionExpression prefix and the generic generator/async declaration
/// producers. User-controlled Module export text carrying the complete prefix
/// must remain owned by the duplicate-export condition.
const fn function_declaration_super_prefix_is_distinct_and_injection_safe() -> bool {
    if !classified_is(
        classify_parse_failure("function declaration cannot contain super at line 1, col 12"),
        EarlyErrorCode::FunctionDeclarationContainsSuper,
    ) || !classified_is(
        classify_parse_failure("function expression cannot contain super at line 1, col 11"),
        EarlyErrorCode::FunctionExpressionContainsSuper,
    ) || !matches!(
        classify_parse_failure("invalid super usage at line 1, col 12"),
        None
    ) {
        return false;
    }

    classified_is(
        classify_parse_failure(
            "exported name `function declaration cannot contain super at line` declared multiple times",
        ),
        EarlyErrorCode::ModuleDuplicateExport,
    )
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
    "P1: a PARSE_FAILURE_RULES row has an empty match pattern (it would match every message) or an empty `witnesses`"
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
const _: () = assert!(
    import_attribute_prefix_is_injection_safe(),
    "P11: user-controlled export text can forge or shadow the anchored duplicate import-attribute classification"
);
const _: () = assert!(
    optional_chain_tagged_template_prefix_is_injection_safe(),
    "P12: user-controlled export text can forge or shadow the anchored optional-chain tagged-template classification"
);
const _: () = assert!(
    import_meta_outside_module_prefix_is_injection_safe(),
    "P13: user-controlled export text can forge or shadow the anchored ImportMeta outside-Module classification"
);
const _: () = assert!(
    lexical_bound_name_let_prefixes_are_injection_safe(),
    "P14: user-controlled export text can forge or shadow an anchored lexical BoundName let classification"
);
const _: () = assert!(
    script_top_level_super_message_is_exact_and_injection_safe(),
    "P15: the fixed ScriptBody super message can be forged or prefix-matches another source position"
);
const _: () = assert!(
    class_super_call_prefixes_are_distinct_and_injection_safe(),
    "P16: a class-owned SuperCall prefix is ambiguous, absorbs an adjacent message, or can be forged through Module export text"
);
const _: () = assert!(
    function_expression_super_prefix_is_distinct_and_injection_safe(),
    "P17: the FunctionExpression super prefix absorbs an adjacent producer or can be forged through Module export text"
);
const _: () = assert!(
    function_declaration_super_prefix_is_distinct_and_injection_safe(),
    "P18: the FunctionDeclaration super prefix absorbs an adjacent producer or can be forged through Module export text"
);
