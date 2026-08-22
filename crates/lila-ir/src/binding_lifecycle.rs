//! Environment Record binding lifecycle (ECMA-262 9.1.1.1).
//!
//! See `docs/rust-rewrite/contracts/environment-record-tdz.md`.
//!
//! A Declarative Environment Record's binding has four attributes, of which two
//! are lifecycle and two are policy. `BindingMode` already carries *mutable*;
//! this module carries *initialized*, which the tree used to spell twice and in
//! neither case on the binding:
//!
//! * a parallel `Vec<BTreeSet<String>>` on `ScriptLowerer` (`tdz_scopes`) that
//!   was zipped positionally against the binding-scope stack, and
//! * a `$tdz.` prefix on the storage name, tested with `starts_with`.
//!
//! Both are gone. There is one state, it lives on `BindingInfo`, and the prefix
//! is demoted to what it always was — a reserved storage *spelling* for a
//! placeholder scope entry, closed by [`TdzPlaceholderName`].

use super::*;

use crate::lowering::{BindingInfo, ScriptLowerer};

/// The lifecycle state of one Environment Record binding (ECMA-262 9.1.1.1).
///
/// Two variants because the spec has two: CreateMutableBinding (9.1.1.1.2) and
/// CreateImmutableBinding (9.1.1.1.3) leave a binding *uninitialized*, and
/// InitializeBinding (9.1.1.1.4) is the sole transition to *initialized*.
/// 9.1.1.1.4 step 2 asserts the binding "must be uninitialized", so the chain
/// has no edge back. Absence is `Option::None` from `ScriptLowerer::lookup_binding`
/// and is deliberately not a variant here.
///
/// No `Default`. A binding whose state was never decided is not a binding whose
/// state is "probably initialized"; it is a declaration site that has not been
/// read against 14.3.1.2 / 16.1.7 step 17 / 10.2.11 steps 21 and 30. Omitting
/// the field is `error[E0063]` at every `BindingInfo` literal, which is the
/// entire point of making it mandatory rather than defaulted.
///
/// No `Unknown`, no `#[non_exhaustive]`: `Initialization` is matched
/// exhaustively at `ScriptLowerer::lexical_storage_name` and — through
/// [`BindingResolution`] — at every Reference-shaped read and write, so a third
/// state must be decided there rather than fall into a `_` arm that quietly
/// answers "not in TDZ".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Initialization {
    /// Created, not yet initialized. GetBindingValue (9.1.1.1.6) step 2 and
    /// SetMutableBinding (9.1.1.1.5) step 3 both throw ReferenceError while a
    /// binding is in this state — the latter *before* the immutability test of
    /// step 6/7, which is why the write sites cannot answer with `mode` alone.
    Uninitialized(UninitializedStorage),
    /// InitializeBinding (9.1.1.1.4) has run, or the binding was created and
    /// stored in one step. The closed list of spec steps that do the latter:
    /// 10.2.11 step 18 (the named function expression's own binding), step 22
    /// (`arguments`), step 24/27 (a formal parameter's initialization), step 28
    /// (`var`), step 30 for the *function* half of `lexDeclarations`; 16.1.7
    /// step 17.b (a hoisted function at script top level); 14.7.4.4 steps 1.e-f
    /// (a per-iteration copy); 14.7.5.5 (a for-in/of iteration binding); 14.15.3
    /// step 4 (a catch parameter); plus Lila's own compiler temporaries and
    /// capture aliases, which have no source-visible window at all.
    Initialized,
}

/// Which storage an uninitialized binding has already claimed.
///
/// Not a spec distinction — 9.1.1.1.2 gives the binding its slot at creation. It
/// is Lila's *name-allocation* question, and it lives on the state so that
/// `ScriptLowerer::lexical_storage_name` can ask it as an exhaustive match on
/// the binding's own record instead of testing its storage name for a `$tdz.`
/// prefix. A third storage disposition is `error[E0004]` there.
///
/// Collapsing the two into a bare `Uninitialized` is the single most dangerous
/// available mistake in this area, and it is silent: the script top level and
/// every function body are *not* direct-lexical scopes, so
/// `direct_lexical_storage_name` falls through to `lexical_storage_name`, whose
/// answer for a source name is stable only while nothing in scope claims it. If
/// a placeholder counted as a claim, a nested block's `let x` would stop
/// shadowing an outer uninitialized `x` and the two would share one slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UninitializedStorage {
    /// The binding's real storage name is already allocated and is the name the
    /// eventual InitializeBinding will write. A later *inner* declaration of the
    /// same source name therefore does shadow it and must allocate a fresh name.
    /// Produced by BlockDeclarationInstantiation-shaped predeclaration, i.e. by
    /// [`LexicalScopeInstantiation`].
    Allocated,
    /// The scope entry is a placeholder: its `storage_name` is a
    /// [`TdzPlaceholderName`], an unspellable name that backs no slot in this
    /// scope, and it exists only so a read of the source name resolves to
    /// *something uninitialized* rather than to an outer binding. It must not
    /// count as a shadowing declaration when the real binding allocates its
    /// name. Produced by `BindingInfo::tdz_placeholder`.
    Placeholder,
}

/// The reserved storage spelling for a for-in/for-of head TDZ binding and a
/// pre-initialization formal parameter (`names::TDZ_BINDING_STORAGE_PREFIX`,
/// `"$tdz."`).
///
/// A newtype with one constructor, for the reason round 2 gave module binding
/// names: the prefix crosses the analysis -> lowering -> aot-wasm boundary as a
/// plain `String` in ten places, and "mint one more `$tdz.` name by hand" and
/// "test for the prefix by hand" are the two ways the domain drifts. There is no
/// `From<String>` and no public field.
///
/// This type says nothing about lifecycle. A binding *named* this way is always
/// `Initialization::Uninitialized(UninitializedStorage::Placeholder)` — enforced
/// by `BindingInfo::tdz_placeholder` being the only constructor that accepts one
/// — but the converse is deliberately false: a predeclared block binding is
/// uninitialized with an ordinary storage name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TdzPlaceholderName(String);

impl TdzPlaceholderName {
    /// The sole constructor. Replaces `lowering_helpers::tdz_binding_storage_name`.
    pub(crate) fn for_source_name(source_name: &str) -> Self {
        Self(format!("{TDZ_BINDING_STORAGE_PREFIX}{source_name}"))
    }

    pub(crate) fn into_string(self) -> String {
        self.0
    }

    /// The sole predicate, and it answers a *name-domain* question — "was this
    /// storage name minted by [`Self::for_source_name`]?" — for the sites that
    /// hold an analysis-minted `&str` and no `BindingInfo` to ask instead:
    /// `lower_function`'s capture-info arm, `is_script_global_var_capture`, and
    /// the `#[cfg(test)]` assertions in `lib.rs`.
    ///
    /// It is NOT the TDZ predicate. No read or write site may call it; they go
    /// through [`BindingResolution`]. Nine `analysis.rs` sites mint these names
    /// for `EnvironmentPlan::binding_storage_names` and the capture-rename map,
    /// which is why the domain cannot simply be folded into [`Initialization`].
    pub(crate) fn names_a_placeholder(storage_name: &str) -> bool {
        storage_name.starts_with(TDZ_BINDING_STORAGE_PREFIX)
    }
}

/// Evidence that a declarator's Initializer has been evaluated.
///
/// `#[must_use]`, not `Clone`, private field. It exists so that
/// [`PendingInitialization::initialize`] — the InitializeBinding of 14.3.1.2
/// step 5 — cannot be written before the Initializer of step 4 has been lowered:
/// there is simply no value of this type in scope until then. `let x = x;`
/// therefore lowers its `x` read against the still-`Uninitialized` binding,
/// which is 9.1.1.1.6 step 2.
///
/// [`Self::evaluated`] is a **general** constructor, not a named loophole, and
/// this doc comment used to claim otherwise. It accepts any `TypedExpr`, so
/// `pending.initialize(LoweredInitializer::evaluated(TypedExpr::undefined()))`
/// followed by lowering the real initializer compiles and reproduces the exact
/// defect the type was introduced against. What the type *does* carry is
/// narrower and is stated in [`PendingInitialization::initialize`]: the
/// transition consumes the token, so it cannot be made twice.
///
/// Its eight call sites, measured rather than asserted (see ledger **L6**):
/// `lower_lexical_declaration`'s four async/generator staging paths
/// (`lowering.rs:16376`, `:16474`, `:16497`, `:16546`, which lower an
/// `await`/`yield` into a temporary and read it back), the ordinary identifier
/// declarator (`:16602`), `lower_using_declaration` (`:16679`),
/// `lower_class_declaration` (`:18657`, whose initializer is the
/// ClassDefinitionEvaluation result of 15.7.16 step 2), and
/// `lower_object_pattern_lexical_binding_from_value` (`:31937`). Every one of
/// them has already produced the value before it calls this — which is a
/// property of those eight bodies, checked by reading them, not by this type.
#[derive(Debug)]
#[must_use = "a lowered initializer that discharges no PendingInitialization is \
              an evaluated expression with no consumer"]
pub(crate) struct LoweredInitializer(TypedExpr);

impl LoweredInitializer {
    /// See the type's doc comment for the measured list of call sites and for
    /// what this constructor does and does not prove.
    pub(crate) fn evaluated(value: TypedExpr) -> Self {
        Self(value)
    }

    pub(crate) fn into_expr(self) -> TypedExpr {
        self.0
    }
}

/// A binding that CreateMutableBinding / CreateImmutableBinding has created and
/// InitializeBinding has not yet initialized: the *obligation* half of
/// 14.3.1.2's two-phase shape.
///
/// Neither `Clone` nor `Copy`, private fields, no `Default`, and the only exit
/// consumes `self` — so a binding cannot be initialized twice, which is
/// 9.1.1.1.4 step 2's assertion made structural (`error[E0382]`).
///
/// It carries `storage_name` because the *predeclaration* allocated it. That is
/// not a convenience. At a scope that is not a direct-lexical scope — every
/// function body and the script top level — `lexical_storage_name` returns a
/// different name once the predeclared entry is in scope, so a declarator that
/// recomputed its own name would write a slot nothing reads. Taking the name
/// from the token is what makes the two halves of one binding provably the same
/// slot. (`ScriptLowerer::direct_lexical_storage_name` enforces the same rule
/// for the destructuring paths that take no token; see ledger entry L2.)
#[derive(Debug)]
#[must_use = "a created binding that is never initialized stays in TDZ for the \
              rest of its scope"]
pub(crate) struct PendingInitialization {
    source_name: String,
    mode: BindingMode,
    storage_name: String,
}

impl PendingInitialization {
    /// InitializeBinding (9.1.1.1.4) for `LexicalBinding : BindingIdentifier
    /// Initializer` and `LexicalBinding : BindingIdentifier` (14.3.1.2 steps 3
    /// and 5).
    ///
    /// Returns an [`InitializedBinding`], **not** the storage name as a bare
    /// `String`. That is the M2b-name fix: while the name was returned loose, a
    /// declarator could take the token, declare the returned record — which
    /// flips the scope entry to `Initialized` and so makes
    /// `direct_lexical_storage_name`'s reuse branch stop matching — and then
    /// recompute `direct_lexical_storage_name` for the emitted
    /// `StatementIr::Lexical`, writing `$lexN` while every earlier read of the
    /// name resolved to the created slot. `lower_class_declaration` held both
    /// spellings in scope simultaneously, so the mistake was one variable
    /// rename away. There is now no `String` in the caller's hands to
    /// substitute: 9.1.1.1.2's slot and 9.1.1.1.4's slot are the same value.
    pub(crate) fn initialize(self, init: LoweredInitializer) -> InitializedBinding {
        let value = init.into_expr();
        let info = BindingInfo {
            mode: self.mode,
            storage_name: self.storage_name.clone(),
            kind: value.kind,
            possible_kinds: value.possible_kinds,
            heap_shape: value.heap_shape.clone(),
            function_targets: value.function_targets.clone(),
            initialization: Initialization::Initialized,
        };
        InitializedBinding {
            source_name: self.source_name,
            mode: self.mode,
            storage_name: self.storage_name,
            info,
            value,
        }
    }
}

/// The completed InitializeBinding (9.1.1.1.4): the scope-record write and the
/// emitted `StatementIr::Lexical`, held together so the storage name in the IR
/// node is provably the one CreateMutableBinding (9.1.1.1.2) allocated.
///
/// All fields are private and the only exit is [`Self::declare`], which does
/// both halves. A declarator therefore has no `String` it could swap for a
/// recomputed one, which is what [`PendingInitialization::initialize`]'s bare
/// `(String, BindingInfo, TypedExpr)` return used to allow.
///
/// [`Self::without_creation`] is the counterpart for the paths where
/// BlockDeclarationInstantiation did **not** create the name — a for-head
/// binding, a compiler-generated declarator, or a form the sweep could not
/// resolve. It is a named constructor rather than an implicit fallback so that
/// "there was no token" is a decision at the call site. Classic `for` heads
/// are explicit untokened paths; statement-list fallbacks sit in the `None`
/// arm of a `match` on the token.
#[derive(Debug)]
#[must_use = "an initialized binding that is never declared leaves the scope \
              entry uninitialized and emits no lexical statement"]
pub(crate) struct InitializedBinding {
    source_name: String,
    mode: BindingMode,
    storage_name: String,
    info: BindingInfo,
    value: TypedExpr,
}

impl InitializedBinding {
    /// The untokened path: this lowering entry has no
    /// `PendingInitialization` for `source_name`, so the caller allocated the
    /// storage name itself.
    pub(crate) fn without_creation(
        source_name: String,
        mode: BindingMode,
        storage_name: String,
        value: TypedExpr,
    ) -> Self {
        let info = BindingInfo::initialized(mode, storage_name.clone(), value.value_info());
        Self {
            source_name,
            mode,
            storage_name,
            info,
            value,
        }
    }

    /// Performs the scope-record write and returns the lexical statement, in
    /// one step. The `name` of the emitted node is `self.storage_name` and
    /// there is no other way to spell it.
    pub(crate) fn declare(self, lowerer: &mut ScriptLowerer<'_>) -> StatementIr {
        lowerer.declare_initialized_binding(self.source_name, self.info);
        StatementIr::Lexical {
            mode: self.mode,
            name: self.storage_name,
            init: self.value,
        }
    }

    /// Transfers runtime initialization ownership to a synchronous resource
    /// entry while completing the lowerer's compile-time lifecycle transition.
    ///
    /// Unlike [`Self::declare`], this emits no `Lexical`: the dedicated scope
    /// backend must acquire and register `@@dispose` before it initializes the
    /// binding, so a generic lexical statement would encode the wrong order.
    pub(crate) fn into_sync_disposable_resource(
        self,
        lowerer: &mut ScriptLowerer<'_>,
    ) -> SyncDisposableResourceIr {
        lowerer.declare_initialized_binding(self.source_name, self.info);
        SyncDisposableResourceIr {
            binding_name: self.storage_name,
            initializer: self.value,
        }
    }

    /// Transfers runtime initialization ownership to an async-dispose resource
    /// entry while completing the same compile-time binding transition.
    ///
    /// The dedicated scope backend must acquire and register the selected
    /// async-dispose protocol before initializing this immutable binding.
    pub(crate) fn into_async_disposable_resource(
        self,
        lowerer: &mut ScriptLowerer<'_>,
    ) -> AsyncDisposableResourceIr {
        lowerer.declare_initialized_binding(self.source_name, self.info);
        AsyncDisposableResourceIr::new(self.storage_name, self.value)
    }
}

/// One statement-list scope's BlockDeclarationInstantiation (14.3.1.2), from the
/// sweep that creates every lexically scoped binding to the last LexicalBinding
/// evaluation that initializes one.
///
/// This type exists so that **a statement-list lowering entry cannot be written
/// without predeclaring**. The three entries — `lower_statement_items`,
/// `lower_statement_items_without_function_initialization` and
/// `lower_root_statement_items_with_function_bindings` — take one, and the only
/// constructors are [`Self::instantiate`], [`Self::instantiate_in_current_scope`]
/// and [`Self::instantiate_switch`], each of which performs the sweep. A fourth
/// statement-list entry added later is `error[E0061]` until its author builds
/// one, rather than compiling into a scope whose `let`s are never created and
/// whose earlier reads therefore see an outer binding or a global.
///
/// The two block-shaped constructors also **own the Environment Record** the
/// sweep populates: they push it and [`Self::finish`] pops it, so the sweep
/// cannot run into the enclosing scope. See [`InstantiatedFrame`].
///
/// `#[must_use]`, not `Clone`. Tokens are taken out by source name; whatever is
/// left when the statement list ends is the set of names whose declarators
/// lowering never reached (unsupported binding forms, and the destructuring
/// patterns of ledger entry L2). Those names stay `Uninitialized` for the rest
/// of the scope, which is the correct answer for an unreached declarator, not a
/// leak.
#[must_use = "a statement list whose lexical declarations were swept but never \
              threaded to its declarators re-opens M5"]
pub(crate) struct LexicalScopeInstantiation {
    pending: BTreeMap<String, PendingInitialization>,
    frame: InstantiatedFrame,
}

/// Which Environment Record the sweep created its bindings in.
///
/// 14.3.1.2 step 1 is "let `env` be the running execution context's
/// LexicalEnvironment" — for a Block that is the **new** declarative
/// Environment Record the Block's evaluation just pushed, not the enclosing
/// one. The token used to witness only that *a* sweep ran: `instantiate` calls
/// `create_lexical_binding` -> `declare_binding` -> `scopes.last_mut()`, and
/// every block-shaped call site performed its push in a different function, so
/// `let scope = LexicalScopeInstantiation::instantiate(self, items);
/// self.push_scope(); self.lower_statement_items(items, scope);` compiled,
/// satisfied M5's `E0061`, and declared every `let` of the inner list into the
/// *parent* scope — where they outlive the block and shadow its siblings.
///
/// The push is therefore part of the constructor and the pop is
/// [`LexicalScopeInstantiation::finish`], which consumes the token. A caller
/// that pushes on its own now gets a frame it has no token to pop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstantiatedFrame {
    /// The constructor pushed the declarative Environment Record the bindings
    /// live in, and `finish` pops it.
    Pushed,
    /// The bindings were created in the frame the caller already owns and whose
    /// lifetime it already manages: the script/module top level and a function
    /// body, whose Environment Record is 16.1.7's global lexical record and
    /// 10.2.11's `lexEnv` respectively, both established before the statement
    /// list is reached. `finish` pops nothing.
    Current,
}

impl LexicalScopeInstantiation {
    /// 14.3.1.2 BlockDeclarationInstantiation, in full: push the block's
    /// declarative Environment Record, then create every lexically scoped name
    /// of `items` in it as
    /// `Initialization::Uninitialized(UninitializedStorage::Allocated)` — the
    /// whole list, before any statement of the list is lowered — and keep one
    /// token per name.
    ///
    /// The push is here rather than at the call site because the scope the
    /// sweep populates must be the scope the statement list is lowered in; see
    /// [`InstantiatedFrame`]. Pair every call with
    /// [`Self::finish`], which the statement-list entries perform.
    pub(crate) fn instantiate(
        lowerer: &mut ScriptLowerer<'_>,
        items: &[StatementListItem],
    ) -> Self {
        lowerer.push_instantiation_scope();
        let mut scope = Self {
            pending: BTreeMap::new(),
            frame: InstantiatedFrame::Pushed,
        };
        for item in items {
            scope.instantiate_item(lowerer, item);
        }
        scope
    }

    /// 16.1.7 GlobalDeclarationInstantiation step 17 and 10.2.11
    /// FunctionDeclarationInstantiation step 30, whose Environment Record
    /// (`env` / `lexEnv`) is established by the caller before the body's
    /// statement list is reached and outlives it.
    ///
    /// Three call sites, all of them a root statement list, none of them a
    /// Block: the script prepass and the script final body
    /// (`lowering.rs:8488`, `:8591`) and the two function-body lowering paths
    /// (`:15033`, `:20188`). Anything block-shaped uses [`Self::instantiate`],
    /// which owns its frame.
    pub(crate) fn instantiate_in_current_scope(
        lowerer: &mut ScriptLowerer<'_>,
        items: &[StatementListItem],
    ) -> Self {
        let mut scope = Self {
            pending: BTreeMap::new(),
            frame: InstantiatedFrame::Current,
        };
        for item in items {
            scope.instantiate_item(lowerer, item);
        }
        scope
    }

    /// 14.12.4 CaseBlockEvaluation instantiates the whole CaseBlock's
    /// LexicallyScopedDeclarations **once**, not once per case, so the sweep is
    /// the union over every case's statement list and the resulting token map is
    /// shared by all of them. Like [`Self::instantiate`] it owns the frame.
    pub(crate) fn instantiate_switch(lowerer: &mut ScriptLowerer<'_>, switch: &AstSwitch) -> Self {
        lowerer.push_instantiation_scope();
        let mut scope = Self {
            pending: BTreeMap::new(),
            frame: InstantiatedFrame::Pushed,
        };
        for case in switch.cases() {
            for item in case.body().statements() {
                scope.instantiate_item(lowerer, item);
            }
        }
        scope
    }

    /// Ends the statement list: pops the Environment Record the constructor
    /// pushed, and nothing otherwise.
    ///
    /// Consumes the token, so the frame is popped exactly once and only by the
    /// value that pushed it. Whatever obligations are left in `pending` are the
    /// names whose declarators lowering never reached; those bindings stay
    /// uninitialized for the rest of the scope, which is the correct answer for
    /// an unreached declarator.
    pub(crate) fn finish(self, lowerer: &mut ScriptLowerer<'_>) {
        match self.frame {
            InstantiatedFrame::Pushed => lowerer.pop_instantiation_scope(),
            InstantiatedFrame::Current => {}
        }
    }

    /// LexicallyScopedDeclarations (8.2.6) for one StatementListItem.
    ///
    /// The match over `boa_ast::Declaration` is exhaustive over all six variants
    /// on purpose. The four function forms are named with the reason they are
    /// not in TDZ rather than swept into a `_` arm: 16.1.7 step 17.b and 10.2.11
    /// step 30 create *and* initialize a hoisted function declaration at
    /// instantiation time, so it never has an uninitialized window. A seventh
    /// boa variant that bound a name lexically would be `error[E0004]` here
    /// instead of a silently unpredeclared binding.
    fn instantiate_item(&mut self, lowerer: &mut ScriptLowerer<'_>, item: &StatementListItem) {
        let StatementListItem::Declaration(declaration) = item else {
            return;
        };
        match declaration.as_ref() {
            Declaration::Lexical(lexical) => {
                let (mode, variables) = match lexical {
                    LexicalDeclaration::Let(variables) => (BindingMode::Let, variables),
                    // A `using` binding is immutable like `const` (ECMA-262
                    // 14.3.4) and is in TDZ until its declaration runs.
                    LexicalDeclaration::Const(variables)
                    | LexicalDeclaration::Using(variables)
                    | LexicalDeclaration::AwaitUsing(variables) => (BindingMode::Const, variables),
                };
                for variable in variables.as_ref() {
                    let Some(bound_names) =
                        supported_bound_names(lowerer.interner(), variable.binding())
                    else {
                        continue;
                    };
                    for bound in bound_names {
                        let span = bound.span;
                        self.create(lowerer, bound.source_name, mode, span);
                    }
                }
            }
            Declaration::ClassDeclaration(class) => {
                // 15.7.16 step 2: the class name is created uninitialized by
                // BlockDeclarationInstantiation and initialized by
                // InitializeBoundName only after ClassDefinitionEvaluation, so
                // the class body evaluates while the name is in TDZ.
                let name = lowerer
                    .interner()
                    .resolve_expect(class.name().sym())
                    .to_string();
                let span = class.name().span();
                self.create(lowerer, name, BindingMode::Let, span);
            }
            // 16.1.7 step 17.b / 10.2.11 step 30: created and initialized in one
            // step at instantiation. Not a TDZ moment, and deliberately spelled
            // out so a new lexical `Declaration` variant cannot join them.
            Declaration::FunctionDeclaration(_)
            | Declaration::GeneratorDeclaration(_)
            | Declaration::AsyncFunctionDeclaration(_)
            | Declaration::AsyncGeneratorDeclaration(_) => {}
        }
    }

    /// CreateMutableBinding (9.1.1.1.2) / CreateImmutableBinding (9.1.1.1.3),
    /// and the sole producer of a [`PendingInitialization`].
    fn create(
        &mut self,
        lowerer: &mut ScriptLowerer<'_>,
        source_name: String,
        mode: BindingMode,
        span: boa_ast::Span,
    ) {
        let storage_name = lowerer.create_lexical_binding(&source_name, mode, span);
        self.pending.insert(
            source_name.clone(),
            PendingInitialization {
                source_name,
                mode,
                storage_name,
            },
        );
    }

    /// Claims the obligation for one source name. `None` means this statement
    /// list did not create the name — a for-head binding, a compiler-generated
    /// declarator, or a name the sweep could not resolve — and the caller falls
    /// back to its own storage-name allocation.
    pub(crate) fn take(&mut self, source_name: &str) -> Option<PendingInitialization> {
        self.pending.remove(source_name)
    }
}

/// The result of ResolveBinding (9.1.2.1) at a site that is about to perform
/// GetBindingValue (9.1.1.1.6) or SetMutableBinding (9.1.1.1.5).
///
/// Three variants, matched exhaustively at every Reference-shaped site. What
/// this discharges, exactly: with `lookup_binding`'s bare `Option<BindingInfo>`
/// a site that forgets step 2 / step 3 compiles and answers `undefined`; with
/// this, **the `Uninitialized` arm must exist**, and the only value that arm
/// can be handed is a [`TdzViolation`]. Immediate reads/writes consume it via
/// `into_throw`; a deferred destructuring write must hand it to
/// [`IdentifierWriteReferenceIr::uninitialized_binding`].
///
/// It does **not** force an arbitrary future arm to use the witness:
/// `#[must_use]` on a struct fires for a discarded expression statement, never
/// for a value bound or wildcarded in a `match` pattern. The current deferred
/// destructuring arm does consume it into its typed Reference, closing the one
/// known exception; future arms remain ledger entry **L8**.
#[must_use = "the 9.1.1.1.6 step 2 / 9.1.1.1.5 step 3 test is the reason this \
              exists; discarding it is the mistake it prevents"]
pub(crate) enum BindingResolution {
    /// 9.1.1.1.6 step 2 / 9.1.1.1.5 step 3.
    Uninitialized(TdzViolation),
    Initialized(BindingInfo),
    /// No Environment Record in the chain has the binding; the caller falls
    /// through to its own global / builtin / unresolvable handling.
    Unresolvable,
}

impl BindingResolution {
    /// The sole producer, and a *total* function of the resolved binding: there
    /// is no way to obtain a `BindingResolution` that skipped the state test.
    pub(crate) fn of(binding: Option<BindingInfo>) -> Self {
        match binding {
            None => Self::Unresolvable,
            Some(binding) => match binding.initialization {
                Initialization::Uninitialized(_) => Self::Uninitialized(TdzViolation(())),
                Initialization::Initialized => Self::Initialized(binding),
            },
        }
    }
}

/// An uninitialized binding reached by a Reference. Not `Clone`, private field.
/// Immediate operations consume it through [`Self::into_throw`]; deferred
/// destructuring PutValue consumes it into an [`IdentifierWriteReferenceIr`].
///
/// Rust cannot make a value undroppable, so it *can* still be discarded by
/// binding it to `_`. See [`BindingResolution`] and ledger entry **L8** for the
/// honest statement of what this carries.
#[derive(Debug)]
#[must_use = "an uninitialized binding that is read or written must throw"]
pub(crate) struct TdzViolation(());

impl TdzViolation {
    /// The ReferenceError of 9.1.1.1.6 step 2 and 9.1.1.1.5 step 3. Its kind
    /// and message come from the same closed identifier-write error consumed by
    /// deferred destructuring, rather than being re-spelled at each site.
    ///
    /// 13.5.3 step 3 special-cases only an *unresolvable* Reference, so `typeof`
    /// does not exempt: a Reference whose base has the binding goes through
    /// GetValue and throws.
    pub(crate) fn into_throw(self) -> TypedExpr {
        let error = IdentifierWriteErrorIr::UninitializedBinding;
        TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::RuntimeThrow {
                name: error.kind(),
                message: error.message(),
            },
        )
    }
}

impl BindingInfo {
    /// A placeholder scope entry for a name that exists and is uninitialized but
    /// whose real slot belongs to another scope: a for-in/for-of head binding
    /// (14.7.5.5) and a formal parameter before its initialization (10.2.11
    /// step 21).
    ///
    /// The only constructor that accepts a [`TdzPlaceholderName`], which is what
    /// ties the reserved spelling and the `Placeholder` state together.
    pub(crate) fn tdz_placeholder(mode: BindingMode, name: TdzPlaceholderName) -> Self {
        Self {
            mode,
            storage_name: name.into_string(),
            kind: ValueKind::Dynamic,
            possible_kinds: KindSet::all_runtime_tags(),
            heap_shape: None,
            function_targets: BTreeSet::new(),
            initialization: Initialization::Uninitialized(UninitializedStorage::Placeholder),
        }
    }

    /// A binding that InitializeBinding has just run for, or that was created
    /// and stored in one step. See `Initialization::Initialized` for the closed
    /// list of spec steps in the second class.
    pub(crate) fn initialized(mode: BindingMode, storage_name: String, info: ValueInfo) -> Self {
        Self {
            mode,
            storage_name,
            kind: info.kind,
            possible_kinds: info.possible_kinds,
            heap_shape: info.heap_shape,
            function_targets: info.function_targets,
            initialization: Initialization::Initialized,
        }
    }
}
