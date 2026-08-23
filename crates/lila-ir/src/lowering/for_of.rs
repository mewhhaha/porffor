use super::async_disposable::LoweredForOfHeadKind;
use super::*;

/// Whether a `for`-`of` head can take the resumable array-index walk
/// (`ScriptLowerer::lower_async_for_of_array_with_body_await`), and if not, the
/// one premise that failed.
///
/// # Why this is a closed type and not the `bool` it replaces
///
/// It used to be `has_unsupported_binding_form: bool`, computed as
///
/// ```ignore
/// !for_of_environment_is_storage_only(lexical_environment.as_ref())
///     || pattern_initializer.is_some()
///     || assignment_pattern_initializer.is_some()
///     || access_initializer.is_some()
/// ```
///
/// and OR-ed at the use site with an array-typing test and an
/// `Option<u32>` entry state, so **four** independent premises produced one
/// message: `async for-of with a body await requires an array iterable and a
/// plain binding`.
///
/// That message is wrong for the family it actually rejects. Measured on the
/// batch-7 baseline sweep, `built-ins/Array/fromAsync` is 95 cases, 93 passed,
/// and both failures carry that string under one `detail_hash`
/// (`10438609855492019567`):
///
/// ```ignore
/// asyncTest(async function () {
///   for (const v of [true, "", Symbol(), 1, 1n, {}]) {
///     await assert.throwsAsync(TypeError,
///       () => Array.fromAsync({ [Symbol.asyncIterator]: v }),
///       `@@asyncIterator = ${typeof v}`);
///   }
/// });
/// ```
///
/// The iterable IS an array literal and the binding IS a plain `const v`. Both
/// stated premises hold. At that baseline the real disqualifier was the third,
/// unnamed one: the arrow captures `v`, so ECMA-262 14.7.5.7 per-iteration
/// bindings materialise an *iteration environment*, which the specialization
/// did not yet reproduce. The same head then reported directly at the CLI:
///
/// ```ignore
/// async function f() {
///   const out = [];
///   for (const v of [1, 2, 3]) { out.push(() => v); await 0; }
/// }
/// // historical result: unsupported in lila wasm-aot first slice: async
/// // for-of with a body await requires an array iterable and a plain binding
/// ```
///
/// That rejection has since been lifted: [`Self::CapturedPerIterationBinding`]
/// owns the analyzed environment and [`Self::into_plan`] returns the closed
/// fresh-per-iteration runtime plan. The historical wrong reason remains useful
/// provenance: it sent triage at an iterable-typing problem that was not there.
/// Each variant below names exactly one premise, so a future variant cannot be
/// added without stating what it means.
///
/// # What each variant costs to lift
///
/// `PlainStorageOnly` covers both "no head environment at all" and "storage
/// slots only, no runtime environment object". Both are reproducible here: the
/// lowering declares those slots as `StatementIr::Lexical` bindings, which every
/// loop compiler marks uninitialized before running the loop init — the same
/// observable TDZ (14.7.5.5 ForIn/OfHeadEvaluation, 8.6.2).
///
/// `CapturedPerIterationBinding` is liftable only by carrying the analyzed
/// environment through `StatementIr::GeneratorLoop`. Its backend must allocate
/// a fresh record for every entered iteration, persist the active pointer across
/// suspension, and restore the parent before the update or next test. Hoisting
/// the binding into one activation slot instead would give every closure the
/// *same* cell and silently break per-iteration semantics. See
/// `docs/rust-rewrite/contracts/resumable-loop-per-iteration-environment.md`.
/// # Why the iterable's type is a variant here and not a test at the call site
///
/// Two of the messages below assert that the *iterable type* is fine, which was
/// once a premise this type could not see: it was tested separately at the sole
/// call site, and the doc here carried a prose "caller obligation" saying that
/// site must test typing **first**. That is exactly the substitution AGENTS.md's
/// "Code Invariants Before Test Invariants" warns against — a second call site
/// would have compiled cleanly while emitting a message asserting a premise it
/// never checked, in a type whose entire thesis is that a wrong reason routes
/// triage away from the defect.
///
/// So the premise moved into the type. [`Self::NonArrayIterable`] is tested
/// first inside [`Self::classify`], which makes the load-bearing order
/// unrepresentable-wrong rather than documented: `for (const c of "ab") { f = ()
/// => c; await 0; }` is captured **and** non-array at once, and answering
/// "captured" for it would certify a String iterable as fine.
#[derive(Debug, Clone, PartialEq, Eq)]
enum AsyncForOfArrayWalkForm {
    /// One plain storage name over an array iterable, and no runtime
    /// environment record to reproduce.
    PlainStorageOnly,
    /// The iterable can be something other than an array, so the walk would
    /// have to be the `@@iterator` protocol instead of an index walk. Tested
    /// first, because it is the cheapest and most universal premise.
    NonArrayIterable,
    /// The head destructures, or assigns into a property/member target, so the
    /// per-iteration value is not one storage name.
    DestructuringOrPropertyTarget,
    /// A closure in the body captures the loop binding, so 14.7.5.7 requires a
    /// fresh environment record per iteration.
    CapturedPerIterationBinding(LexicalEnvironmentIr),
    /// A closure captures a name the head puts in TDZ while the iterable is
    /// evaluated, so the TDZ scope needs a real environment record too. This is
    /// a different problem from the iteration environment and is left rejected
    /// until it is measured on its own.
    CapturedTdzBinding,
}

impl AsyncForOfArrayWalkForm {
    /// The only constructor.
    ///
    /// `iterable_is_array` is the `KindSet` subset test on the iterable, passed
    /// in rather than assumed — see the type's doc for why it is a parameter and
    /// not a caller obligation. `binds_pattern_or_property_target` is the
    /// disjunction of the head's three non-identifier initializer forms
    /// (`pattern_initializer`, `assignment_pattern_initializer`,
    /// `access_initializer`).
    ///
    /// The arm order is the classification order: typing, then binding shape,
    /// then captured environments. Do not reorder — the TDZ-capture error
    /// states that the array and plain-binding premises above it hold.
    fn classify(
        iterable_is_array: bool,
        environment: Option<&ForInOfEnvironmentIr>,
        binds_pattern_or_property_target: bool,
    ) -> Self {
        if !iterable_is_array {
            return Self::NonArrayIterable;
        }
        if binds_pattern_or_property_target {
            return Self::DestructuringOrPropertyTarget;
        }
        let Some(environment) = environment else {
            return Self::PlainStorageOnly;
        };
        if let Some(iteration_environment) = &environment.iteration_environment {
            return Self::CapturedPerIterationBinding(iteration_environment.clone());
        }
        if environment.tdz_environment.is_some() {
            return Self::CapturedTdzBinding;
        }
        Self::PlainStorageOnly
    }

    /// Converts every supported source shape into the required closed IR plan,
    /// or returns the one failed premise. A captured iteration binding is a
    /// supported plan only because the variant owns its analyzed environment;
    /// there is no path that can lose the layout and silently select storage.
    fn into_plan(self) -> Result<ResumableLoopIterationEnvironmentIr, &'static str> {
        match self {
            Self::PlainStorageOnly => Ok(ResumableLoopIterationEnvironmentIr::StorageOnly),
            Self::CapturedPerIterationBinding(environment) => Ok(
                ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment),
            ),
            Self::NonArrayIterable => Err(
                "async for-of with a body await requires an array iterable, and this one \
                 can be something else; every other iterable keeps the @@iterator protocol, \
                 whose own suspension points this index walk does not have",
            ),
            Self::DestructuringOrPropertyTarget => Err(
                "async for-of with a body await requires a plain single-name binding, \
                 and this head destructures or assigns into a property target",
            ),
            Self::CapturedTdzBinding => Err(
                "async for-of with a body await cannot materialize the head's TDZ \
                 environment record, and a closure captures a name the head puts in TDZ; \
                 the iterable is an array and the head binds one plain name",
            ),
        }
    }
}

/// What lowering a `for`-`of` head produced: the statement, the kind its body
/// evaluates to, and the witness saying how that statement discharged the four
/// 7.4 obligations.
///
/// Attaching the witness to the three `ForOf*` variants alone was not enough.
/// There is already a **fourth** for-of specialization that is not spelled as a
/// `ForOf*` variant: `for (x of arr) { … await … }` inside a plain async
/// function is desugared to `StatementIr::GeneratorLoop` with an explicit
/// `index < PropertyKeyIr::ArrayLength` test and `PropertyKeyIr::ArrayIndex`
/// element reads — an index walk resting on all the array premises, which no
/// `protocol` field on a `ForOf*` variant could have demanded.
///
/// So the obligation is attached to the *lowering of the head* instead. Every
/// path out of `ScriptLowerer::lower_for_of_head` returns one of these, the
/// only constructor takes a witness, and there is no `Default`. A new
/// desugaring target therefore cannot be added without naming its premises,
/// and the `ForOf*` `protocol` field becomes a consumer of that value rather
/// than the only place it is demanded.
///
/// Before the statement crosses back to dispatch,
/// [`ForOfLoweringIr::into_statement_and_kind`] consumes the carrier, reads the
/// witness and checks the two locally decidable conditions. Spread, `yield*`
/// and array destructuring reach the protocol by other routes and are named as
/// `EmissionSite`s instead — see ledger L6.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ForOfLoweringIr {
    statement: StatementIr,
    result_kind: ValueKind,
    protocol: IteratorProtocolWitness,
}

impl ForOfLoweringIr {
    fn new(
        statement: StatementIr,
        result_kind: ValueKind,
        protocol: IteratorProtocolWitness,
    ) -> Self {
        Self {
            statement,
            result_kind,
            protocol,
        }
    }

    /// The head did not lower to an iteration: an unsupported form was reported
    /// and the statement is `StatementIr::Empty`.
    fn no_iteration() -> Self {
        Self::new(
            StatementIr::Empty,
            ValueKind::Undefined,
            IteratorProtocolWitness::NO_ITERATION,
        )
    }

    /// The statement and the kind its body evaluates to. The witness is dropped
    /// here — its work is done by the time the head has lowered — but it is
    /// *read* on the way out rather than silently discarded.
    ///
    /// The `protocol()` accessor this replaces had **zero callers anywhere in
    /// the workspace** and was `pub`, so no `dead_code` warning fired: the
    /// "survival by `pub`" shape ledger row I7 exists to delete, as recorded in
    /// the iterator-protocol contract. The two conditions below are its
    /// replacement, and each names a real mistake:
    ///
    /// * A head that lowered to *nothing* must carry the bail-out witness.
    ///   Returning `StatementIr::Empty` with, say,
    ///   `SYNC_ITERATOR_PROTOCOL` would credit `compile_for_of_iterator` with
    ///   emitting four obligations for a statement that never runs, which is
    ///   exactly the attribution K1 and J10 exist to keep honest.
    /// * A head that lowered to a real for-of specialization must *not* carry
    ///   it: `NO_ITERATION` says every obligation is vacuous because nothing
    ///   runs, and one of the three `ForOf*` statements is not nothing.
    fn into_statement_and_kind(self) -> (StatementIr, ValueKind) {
        debug_assert!(
            !matches!(self.statement, StatementIr::Empty)
                || self.protocol == IteratorProtocolWitness::NO_ITERATION,
            "a for-of head that lowered to no statement must carry the NO_ITERATION witness",
        );
        debug_assert!(
            !matches!(
                self.statement,
                StatementIr::ForOfArray { .. }
                    | StatementIr::ForOfString { .. }
                    | StatementIr::ForOfIterator { .. }
            ) || self.protocol != IteratorProtocolWitness::NO_ITERATION,
            "a for-of head that lowered to a real specialization must not claim that no \
             iteration was lowered",
        );
        (self.statement, self.result_kind)
    }
}

impl<'a> ScriptLowerer<'a> {
    /// Rewrites `for (x of arr) { … await … }` inside a plain async function
    /// into the index-driven `StatementIr::GeneratorLoop` the resumable async
    /// dispatcher can re-enter.
    ///
    /// `StatementIr::ForOfArray` emits a straight-line wasm loop: on resume the
    /// async driver re-enters the function from the top, the loop would restart
    /// at element zero, and the suspension would never fire again. Hoisting the
    /// array and the cursor into loop-init bindings puts both in the activation
    /// record, so each invocation runs exactly one iteration — the same shape
    /// `lower_for_loop` produces (ECMA-262 14.7.5 ForIn/OfBodyEvaluation over an
    /// array, 27.7.5.3 AsyncBlockStart for the resume).
    ///
    /// Only the array-shaped iterable is rewritten. Anything else keeps the
    /// iterator protocol, which has its own suspension points, so it is reported
    /// as unsupported rather than silently miscompiled.
    ///
    /// This is the **fourth** for-of specialization, and the one a `protocol`
    /// field on `StatementIr::ForOfArray` could never have caught: the statement
    /// it produces is a `GeneratorLoop`, not a `ForOf*`. It rests on exactly the
    /// premises of [`IteratorProtocolWitness::ARRAY_INDEX_WALK`] — an
    /// `index < ArrayLength` test and `ArrayIndex` element reads, with no
    /// `@@iterator` `Get` anywhere — which is why it returns a
    /// [`ForOfLoweringIr`] carrying
    /// [`IteratorProtocolWitness::ARRAY_INDEX_WALK_RESUMABLE`].
    #[allow(clippy::too_many_arguments)]
    fn lower_async_for_of_array_with_body_await(
        &mut self,
        mode: BindingMode,
        storage_name: &str,
        iterable: TypedExpr,
        element_info: &ValueInfo,
        body: StatementIr,
        body_kind: ValueKind,
        head_form: AsyncForOfArrayWalkForm,
        entry_state: Option<u32>,
        head_environment: Option<ForInOfEnvironmentIr>,
    ) -> ForOfLoweringIr {
        // Four premises and one entry-state safety net used to be one `&&`/`||`
        // chain behind one string —
        // "requires an array iterable and a plain binding" — which was reported
        // for `built-ins/Array/fromAsync/asyncitems-*-not-callable.js`, whose
        // head is `for (const v of [array literal])`, i.e. a case where both
        // named premises hold. A rejection reason that names the wrong premise
        // is worse than none: it routes triage away from the defect.
        //
        // Classification order (typing, then binding shape, then captured
        // environment) lives inside `classify` rather than here, so a second
        // call site cannot get it wrong. See the type's doc.
        let iteration_environment = match head_form.into_plan() {
            Ok(plan) => plan,
            Err(message) => {
                self.unsupported(message);
                return ForOfLoweringIr::no_iteration();
            }
        };
        // A SAFETY NET, not a fifth diagnostic family — do not count it among
        // the messages this split delivers, and do not expect it in a sweep
        // family map. Reaching it requires `plain_async_entry_state()` to answer
        // `Some` when `plain_async_await_body` is computed and `None` when it is
        // re-read just before the body is lowered, and only one transition can
        // do that: neither `current_async_resume_state` nor
        // `current_generator_resume_state` is ever assigned `None` after the
        // lowerer is constructed (every assignment across lowering.rs and its
        // child modules is `Some(..)`, and the `lowerer.*` ones belong to nested
        // lowerers), so the sole route is `current_generator_resume_state`
        // going `None -> Some` while the *head* is lowered. The window is
        // head-only: the iterable is already
        // known to contain no `await`, and the re-read happens before
        // `lower_loop_body`. That transition has not been shown to be reachable
        // and has not been shown to be impossible either, which is exactly why
        // the arm stays: falling through with no entry state would emit a
        // straight-line loop holding a suspension, i.e. a miscompile, and this
        // returns a diagnostic instead.
        let Some(entry_state) = entry_state else {
            self.unsupported(
                "async for-of with a body await requires a plain async function body with a \
                 resumable entry state, and this body has none",
            );
            return ForOfLoweringIr::no_iteration();
        };
        let Some((before_suspension, suspension_statement, after_suspension)) =
            Self::split_resumable_loop_body(body)
        else {
            self.unsupported("async for-of body did not lower to one direct await");
            return ForOfLoweringIr::no_iteration();
        };
        let StatementIr::AsyncAwait { resume_state, .. } = &suspension_statement else {
            self.unsupported("async for-of body did not lower to one direct await");
            return ForOfLoweringIr::no_iteration();
        };
        let resume_state = *resume_state;
        let exit_state = resume_state + 1;

        // The array and the cursor are read on every resume, so they have to
        // live in the activation record rather than in wasm locals, which the
        // return to the job queue discards. Source-level bindings get that from
        // the owner analysis; these two are synthesized here, so they have to
        // ask for it explicitly.
        let array_info = iterable.value_info();
        let number_info = ValueInfo::new(ValueKind::Number);
        let array_name =
            self.alloc_suspension_owned_binding("async.forof.array.", array_info.clone());
        let index_name =
            self.alloc_suspension_owned_binding("async.forof.index.", number_info.clone());
        let array_ref =
            || TypedExpr::from_info(array_info.clone(), ExprIr::Identifier(array_name.clone()));
        let index_ref =
            || TypedExpr::from_info(number_info.clone(), ExprIr::Identifier(index_name.clone()));

        let init = ForInitIr::LexicalBlock(vec![
            ForLexicalInitIr {
                mode: BindingMode::Let,
                name: array_name.clone(),
                init: iterable,
            },
            ForLexicalInitIr {
                mode: BindingMode::Let,
                name: index_name.clone(),
                init: TypedExpr::from_info(number_info.clone(), ExprIr::Number(0.0f64.to_bits())),
            },
        ]);
        let test = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Boolean),
            ExprIr::CompareValue {
                op: RelationalBinaryOp::LessThan,
                lhs: Box::new(index_ref()),
                rhs: Box::new(TypedExpr::from_info(
                    number_info.clone(),
                    ExprIr::PropertyRead {
                        target: Box::new(array_ref()),
                        key: PropertyKeyIr::ArrayLength,
                    },
                )),
            },
        );
        let update = TypedExpr::from_info(
            number_info.clone(),
            ExprIr::UpdateIdentifier {
                name: index_name.clone(),
                op: NumericUpdateOp::Increment,
                return_mode: UpdateReturnMode::Postfix,
                value_kind: ValueKind::Number,
            },
        );
        let element_value = TypedExpr::from_info(
            element_info.clone(),
            ExprIr::PropertyRead {
                target: Box::new(array_ref()),
                key: PropertyKeyIr::ArrayIndex(Box::new(index_ref())),
            },
        );
        // `var` is hoisted to the enclosing function scope and has no TDZ, so it
        // takes the assignment form; `let`/`const` are re-created per iteration.
        let element_binding = if mode == BindingMode::Var {
            StatementIr::Var(vec![VarDeclaratorIr {
                name: storage_name.to_string(),
                init: Some(element_value),
            }])
        } else {
            StatementIr::Lexical {
                mode,
                name: storage_name.to_string(),
                init: element_value,
            }
        };
        // The head environment only puts the loop name's TDZ slot in scope while
        // the iterable is evaluated. Declaring those slots here reproduces it:
        // every resumable loop compiler marks `before_suspension` bindings
        // uninitialized before it runs the loop init, so a self-referential
        // iterable still sees the ReferenceError (ECMA-262 14.7.5.5).
        let tdz_names = if mode == BindingMode::Var {
            Vec::new()
        } else {
            head_environment
                .as_ref()
                .map(|environment| environment.tdz_binding_names.clone())
                .unwrap_or_default()
        };
        let mut before = Vec::with_capacity(before_suspension.len() + tdz_names.len() + 1);
        for name in tdz_names {
            if name == storage_name {
                continue;
            }
            before.push(StatementIr::Lexical {
                mode,
                name,
                init: TypedExpr::undefined(),
            });
        }
        before.push(element_binding);
        before.extend(before_suspension);

        self.current_async_resume_state = Some(exit_state);
        ForOfLoweringIr::new(
            StatementIr::GeneratorLoop {
                init: Some(init),
                test: Some(test),
                update: Some(update),
                iteration_environment,
                before_suspension: before,
                suspension_statement: Box::new(suspension_statement),
                after_suspension,
                entry_state,
                resume_state,
                exit_state,
            },
            body_kind,
            IteratorProtocolWitness::ARRAY_INDEX_WALK_RESUMABLE,
        )
    }

    /// Lowers a `for`-`of` head.
    ///
    /// Thin wrapper: the witness [`lower_for_of_head`] produced has done its
    /// work by the time control returns here (every path out of that function
    /// had to name one), and no emitter may read it.
    ///
    /// [`lower_for_of_head`]: Self::lower_for_of_head
    pub(super) fn lower_for_of_loop(&mut self, for_of: &ForOfLoop) -> (StatementIr, ValueKind) {
        self.lower_for_of_head(for_of).into_statement_and_kind()
    }

    /// Every path out of this function returns a [`ForOfLoweringIr`], whose only
    /// constructor takes an [`IteratorProtocolWitness`]. That is what makes
    /// "add a fourth for-of desugaring and silently assume the protocol away" a
    /// compile error rather than a silent wrong answer: attaching `protocol` to
    /// the three `ForOf*` variants alone missed the desugaring to
    /// `StatementIr::GeneratorLoop` that already existed.
    fn lower_for_of_head(&mut self, for_of: &ForOfLoop) -> ForOfLoweringIr {
        let uses_unified_resumable_plan = for_of.r#await() && self.current_resumable_plan.is_some();
        if for_of.r#await()
            && !uses_unified_resumable_plan
            && self.current_async_resume_state.is_none()
        {
            self.unsupported("for-await-of outside async function");
            return ForOfLoweringIr::no_iteration();
        }
        if for_of.r#await() && contains(for_of.body(), ContainsSymbol::AwaitExpression) {
            self.unsupported("explicit await in for-await-of body");
            return ForOfLoweringIr::no_iteration();
        }
        // A plain `for (x of …)` whose body awaits needs the same
        // one-iteration-per-invocation shape a resumable `for` loop gets, so it
        // is rewritten below into an index-driven `StatementIr::GeneratorLoop`.
        let plain_async_await_body = self.plain_async_entry_state().is_some()
            && !for_of.r#await()
            && contains(for_of.body(), ContainsSymbol::AwaitExpression);
        if plain_async_await_body
            && (generator_loop_has_unsupported_control(for_of.body(), false)
                || contains(for_of.iterable(), ContainsSymbol::AwaitExpression))
        {
            self.unsupported(
                "async for-of with await requires an eager iterable and a body without break or continue",
            );
            return ForOfLoweringIr::no_iteration();
        }
        if let IterableLoopInitializer::WebCompatCall(call) = for_of.initializer() {
            // The head evaluates the iterable and then throws a ReferenceError,
            // so no 7.4 operation ever runs: the loop is gone, not specialized.
            return ForOfLoweringIr::new(
                StatementIr::Expression(
                    self.lower_web_compat_loop_assignment_target(call, for_of.iterable()),
                ),
                ValueKind::Undefined,
                IteratorProtocolWitness::NO_ITERATION,
            );
        }
        let mut pattern_initializer: Option<(BindingMode, Pattern)> = None;
        let mut assignment_pattern_initializer: Option<Pattern> = None;
        let mut access_initializer: Option<PropertyAccess> = None;
        let (head_kind, mode, name) = match for_of.initializer() {
            IterableLoopInitializer::Identifier(identifier) => (
                LoweredForOfHeadKind::Assignment,
                BindingMode::Var,
                self.interner.resolve_expect(identifier.sym()).to_string(),
            ),
            IterableLoopInitializer::Var(variable) => match variable.binding() {
                Binding::Identifier(identifier) => (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Var,
                    self.interner.resolve_expect(identifier.sym()).to_string(),
                ),
                Binding::Pattern(pattern) => {
                    pattern_initializer = Some((BindingMode::Var, pattern.clone()));
                    (
                        LoweredForOfHeadKind::Assignment,
                        BindingMode::Let,
                        self.alloc_temp_binding_name("forof"),
                    )
                }
            },
            IterableLoopInitializer::Let(Binding::Identifier(identifier)) => (
                LoweredForOfHeadKind::Assignment,
                BindingMode::Let,
                self.interner.resolve_expect(identifier.sym()).to_string(),
            ),
            IterableLoopInitializer::Const(Binding::Identifier(identifier)) => (
                LoweredForOfHeadKind::Assignment,
                BindingMode::Const,
                self.interner.resolve_expect(identifier.sym()).to_string(),
            ),
            IterableLoopInitializer::Let(Binding::Pattern(pattern)) => {
                pattern_initializer = Some((BindingMode::Let, pattern.clone()));
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof"),
                )
            }
            IterableLoopInitializer::Const(Binding::Pattern(pattern)) => {
                pattern_initializer = Some((BindingMode::Const, pattern.clone()));
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof"),
                )
            }
            IterableLoopInitializer::Using(Binding::Identifier(identifier)) => {
                if for_of.r#await() {
                    self.unsupported("using declaration in for-await-of");
                    return ForOfLoweringIr::no_iteration();
                }
                if self.root_this_binding == RootThisBinding::Undefined {
                    self.unsupported("using declaration in a module");
                    return ForOfLoweringIr::no_iteration();
                }
                if self.current_generator_resume_state.is_some()
                    || self.current_async_resume_state.is_some()
                    || self.current_resumable_plan.is_some()
                {
                    self.unsupported("using declaration in a generator or async function");
                    return ForOfLoweringIr::no_iteration();
                }
                (
                    LoweredForOfHeadKind::SyncDisposable,
                    BindingMode::Const,
                    self.interner.resolve_expect(identifier.sym()).to_string(),
                )
            }
            IterableLoopInitializer::Using(Binding::Pattern(_)) => {
                self.unsupported("using declaration binding pattern in for-of");
                return ForOfLoweringIr::no_iteration();
            }
            IterableLoopInitializer::AwaitUsing(binding) => {
                let Some(name) = self.admit_async_disposable_for_of_head(for_of, binding) else {
                    return ForOfLoweringIr::no_iteration();
                };
                (
                    LoweredForOfHeadKind::AsyncDisposable,
                    BindingMode::Const,
                    name,
                )
            }
            IterableLoopInitializer::Pattern(pattern) => {
                assignment_pattern_initializer = Some(pattern.clone());
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof"),
                )
            }
            // `for (obj.key of …)` and `for (this.#field of …)` both assign to a
            // reference that the spec re-evaluates on every iteration, so the
            // element lands in a temporary and the body prefix performs the store.
            IterableLoopInitializer::Access(
                access @ (PropertyAccess::Simple(_) | PropertyAccess::Private(_)),
            ) => {
                access_initializer = Some(access.clone());
                (
                    LoweredForOfHeadKind::Assignment,
                    BindingMode::Let,
                    self.alloc_temp_binding_name("forof.access"),
                )
            }
            _ => {
                self.unsupported("for-of initializer");
                return ForOfLoweringIr::no_iteration();
            }
        };
        let static_generator_elements =
            self.static_generator_call_elements_owned(for_of.iterable());
        let lexical_environment =
            self.lower_for_in_of_environment(for_of as *const ForOfLoop as usize);
        let iterable = match (
            pattern_initializer.as_ref(),
            assignment_pattern_initializer.as_ref(),
        ) {
            (None, None) if access_initializer.is_none() => {
                self.lower_for_head_expression_with_tdz(mode, &name, for_of.iterable())
            }
            (None, None) => self.lower_expression(for_of.iterable()),
            (Some((BindingMode::Var, _)), _) | (None, Some(_)) => {
                self.lower_expression(for_of.iterable())
            }
            (Some((pattern_mode, pattern)), None) => {
                self.push_scope();
                let binding = Binding::Pattern(pattern.clone());
                let Some(bound_names) = supported_bound_names(self.interner, &binding) else {
                    self.unsupported("for-of initializer");
                    self.pop_scope();
                    return ForOfLoweringIr::no_iteration();
                };
                for bound in bound_names {
                    self.declare_binding(
                        bound.source_name.clone(),
                        BindingInfo::tdz_placeholder(
                            *pattern_mode,
                            TdzPlaceholderName::for_source_name(&bound.source_name),
                        ),
                    );
                }
                let iterable = self.lower_expression(for_of.iterable());
                self.pop_scope();
                iterable
            }
            (Some(_), Some(_)) => unreachable!("loop head cannot be binding and assignment"),
        };
        let async_generator_next_suspension = uses_unified_resumable_plan
            .then(|| self.take_resumable_suspension(ResumableSuspensionKindIr::ForAwaitNext))
            .flatten();
        // The loop's own first suspension is the `await` on `next()`, which
        // the spec reaches once the iterable has been evaluated and before the
        // body runs. Claiming the entry state here puts it in that same order:
        // after any `await` staged out of the loop head, which has already
        // consumed states, and before the body allocates its own.
        let async_entry_state = (for_of.r#await() && !uses_unified_resumable_plan)
            .then_some(self.current_async_resume_state)
            .flatten();
        if for_of.r#await() {
            // 7.4.3 GetIterator tries `@@asyncIterator` first, then falls back
            // to `@@iterator` wrapped as an async iterator. The order is the
            // spec obligation.
            for key in [WellKnownSymbol::AsyncIterator, WellKnownSymbol::Iterator] {
                let function_targets = self
                    .optional_chain_well_known_symbol_property_info(&iterable.value_info(), key)
                    .map(|method| method.function_targets)
                    .unwrap_or_default();
                for function_id in function_targets {
                    let fallback = self
                        .function_signatures
                        .get(&function_id)
                        .map(|signature| signature.this_info.clone())
                        .unwrap_or_else(|| ValueInfo::new(ValueKind::Dynamic));
                    let this_info = self.explicit_this_info_for_function_target(
                        &function_id,
                        &iterable,
                        fallback,
                    );
                    self.merge_function_this_info(&function_id, this_info);
                }
            }
        }
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.push_scope();
        let element_info = if let Some(elements) = &static_generator_elements {
            elements
                .iter()
                .map(TypedExpr::value_info)
                .reduce(|a, b| self.merge_value_infos(a, b))
                .unwrap_or_else(ValueInfo::undefined)
        } else {
            match iterable.heap_shape.as_deref() {
                Some(HeapShape::Array(shape)) => shape
                    .elements
                    .iter()
                    .cloned()
                    .reduce(|a, b| self.merge_value_infos(a, b))
                    .unwrap_or_else(ValueInfo::undefined),
                _ if iterable
                    .possible_kinds
                    .is_subset_of(KindSet::from_kind(ValueKind::String)) =>
                {
                    ValueInfo::new(ValueKind::String)
                }
                _ => ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
            }
        };
        let storage_name = if mode == BindingMode::Var
            || pattern_initializer.is_some()
            || assignment_pattern_initializer.is_some()
            || access_initializer.is_some()
        {
            name.clone()
        } else {
            for_of_loop_binding_storage_name(for_of, &name)
        };
        let Ok(pending_async_disposable_head) =
            self.begin_async_disposable_for_of_if_needed(head_kind, &storage_name)
        else {
            self.pop_scope();
            return ForOfLoweringIr::no_iteration();
        };
        self.declare_binding(
            name.clone(),
            BindingInfo {
                mode,
                storage_name: storage_name.clone(),
                kind: element_info.kind,
                possible_kinds: element_info.possible_kinds,
                heap_shape: element_info.heap_shape.clone(),
                function_targets: element_info.function_targets.clone(),
                initialization: Initialization::Initialized,
            },
        );
        let mut pattern_prefix = if let Some(access) = access_initializer.as_ref() {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let access = access.clone();
            vec![StatementIr::Expression(
                self.lower_property_assign_value(&access, value),
            )]
        } else if let Some(pattern) = assignment_pattern_initializer.as_ref() {
            let value = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            let Some(assign) = self.lower_pattern_assign_value(pattern, value) else {
                self.pop_scope();
                return ForOfLoweringIr::no_iteration();
            };
            vec![StatementIr::Expression(assign)]
        } else if let Some((pattern_mode, pattern)) = pattern_initializer.as_ref() {
            let init = TypedExpr::from_info(
                element_info.clone(),
                ExprIr::Identifier(storage_name.clone()),
            );
            if *pattern_mode == BindingMode::Var {
                let Some(prefix) = self.lower_pattern_var_binding_from_value(pattern, init) else {
                    self.pop_scope();
                    return ForOfLoweringIr::no_iteration();
                };
                prefix
            } else {
                let binding = Binding::Pattern(pattern.clone());
                let storage_names = supported_bound_names(self.interner, &binding)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|bound| {
                        let storage_name =
                            for_of_loop_binding_storage_name(for_of, &bound.source_name);
                        (bound.source_name, storage_name)
                    })
                    .collect();
                let Some(prefix) = self
                    .lower_pattern_lexical_binding_from_value_with_storage_names(
                        *pattern_mode,
                        pattern,
                        init,
                        Some(&storage_names),
                    )
                else {
                    self.pop_scope();
                    return ForOfLoweringIr::no_iteration();
                };
                prefix
            }
        } else {
            Vec::new()
        };
        let plain_async_entry_state = self.plain_async_entry_state();
        let (mut body, body_kind) = self.lower_loop_body(for_of.body());
        let async_disposable_head = pending_async_disposable_head
            .map(|pending| self.finish_async_disposable_for_of_head(pending));
        let async_generator_close_suspension = uses_unified_resumable_plan
            .then(|| self.take_resumable_suspension(ResumableSuspensionKindIr::ForAwaitClose))
            .flatten();
        if !pattern_prefix.is_empty() {
            pattern_prefix.push(body);
            body = StatementIr::Block(BlockIr {
                result_kind: body_kind,
                statements: pattern_prefix,
                lexical_environment: None,
            });
        }
        self.pop_scope();
        let after_vars = self.var_bindings.clone();
        let after_globals = self.global_properties.clone();
        self.var_bindings = self.merge_var_bindings(&before_vars, &after_vars);
        self.global_properties = self.merge_global_properties(&before_globals, &after_globals);
        if plain_async_await_body {
            let iterable_is_array = iterable
                .possible_kinds
                .is_subset_of(KindSet::from_kind(ValueKind::Array));
            return self.lower_async_for_of_array_with_body_await(
                mode,
                &storage_name,
                iterable,
                &element_info,
                body,
                body_kind,
                // One closed reason instead of the four-premise boolean that
                // used to be OR-ed together here. See
                // `AsyncForOfArrayWalkForm`'s doc comment for the measurement
                // that forced the split, and for why the iterable's type is an
                // argument rather than a separate test inside the callee.
                AsyncForOfArrayWalkForm::classify(
                    iterable_is_array,
                    lexical_environment.as_ref(),
                    pattern_initializer.is_some()
                        || assignment_pattern_initializer.is_some()
                        || access_initializer.is_some(),
                ),
                plain_async_entry_state,
                lexical_environment.clone(),
            );
        }
        // Each arm produces its statement together with the witness that says how
        // it discharged the four 7.4 obligations, so the two cannot drift apart
        // and a fourth arm cannot be added without stating its premises.
        let (statement, protocol) = if iterable
            .possible_kinds
            .is_subset_of(KindSet::from_kind(ValueKind::Array))
            && !for_of.r#await()
            && head_kind == LoweredForOfHeadKind::Assignment
        {
            let protocol = IteratorProtocolWitness::ARRAY_INDEX_WALK;
            (
                StatementIr::ForOfArray {
                    head: ForOfAssignmentIr {
                        mode,
                        name: storage_name,
                    },
                    iterable,
                    body: Box::new(body),
                    lexical_environment,
                    protocol,
                },
                protocol,
            )
        } else if iterable
            .possible_kinds
            .is_subset_of(KindSet::from_kind(ValueKind::String))
            && !for_of.r#await()
            && head_kind == LoweredForOfHeadKind::Assignment
        {
            let protocol = IteratorProtocolWitness::STRING_CODE_POINT_WALK;
            (
                StatementIr::ForOfString {
                    head: ForOfAssignmentIr {
                        mode,
                        name: storage_name,
                    },
                    iterable,
                    body: Box::new(body),
                    lexical_environment,
                    protocol,
                },
                protocol,
            )
        } else {
            // Everything else goes through the generic iterator protocol. That
            // includes primitives: `for (x of 37)` has to reach `GetIterator`,
            // which does `ToObject` and then looks `@@iterator` up on the wrapper
            // prototype, so a missing or non-callable method throws a TypeError at
            // runtime instead of being refused at compile time.
            match head_kind {
                LoweredForOfHeadKind::SyncDisposable => {
                    let protocol = IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL;
                    (
                        StatementIr::ForOfIterator {
                            head: ForOfIteratorHeadIr::SyncDisposable(
                                SyncDisposableForOfHeadIr::new(storage_name),
                            ),
                            iterable,
                            body: Box::new(body),
                            lexical_environment,
                        },
                        protocol,
                    )
                }
                LoweredForOfHeadKind::AsyncDisposable => Self::async_disposable_for_of_statement(
                    async_disposable_head
                        .expect("an admitted async-disposable head must be finalized"),
                    iterable,
                    body,
                    lexical_environment,
                ),
                LoweredForOfHeadKind::Assignment => {
                    let async_states = if uses_unified_resumable_plan {
                        async_generator_next_suspension
                            .zip(async_generator_close_suspension)
                            .map(|(next, close)| {
                                (
                                    next.suspend_state,
                                    next.resume_state,
                                    close.suspend_state,
                                    close.resume_state,
                                )
                            })
                    } else {
                        async_entry_state.map(|entry_state| {
                            let value_resume_state = entry_state + 1;
                            let close_resume_state = entry_state + 2;
                            let exit_state = entry_state + 3;
                            self.current_async_resume_state = Some(exit_state);
                            (
                                entry_state,
                                value_resume_state,
                                close_resume_state,
                                exit_state,
                            )
                        })
                    };
                    let async_plan = async_states.map(
                        |(entry_state, value_resume_state, close_resume_state, exit_state)| {
                            // Allocation order is load-bearing: `alloc_temp_binding_name`
                            // numbers bindings as it hands them out, so these five calls
                            // must stay in this sequence for the emitted names to be the
                            // ones they were before the Iterator Record retrofit.
                            let iterator = self.alloc_iterator_slot();
                            let next_method = self.alloc_next_method_slot();
                            let async_iterator_binding = self.alloc_suspension_owned_binding(
                                "async.forof.async_iterator.",
                                ValueInfo::new(ValueKind::Boolean),
                            );
                            let done = self.alloc_done_slot();
                            let close_on_rejection_binding = self.alloc_suspension_owned_binding(
                                "async.forof.close_on_rejection.",
                                ValueInfo::new(ValueKind::Boolean),
                            );
                            AsyncForOfIteratorPlanIr {
                                entry_state,
                                value_resume_state,
                                close_resume_state,
                                exit_state,
                                record: IteratorRecordIr::new(iterator, next_method, done),
                                async_iterator_binding,
                                close_on_rejection_binding,
                            }
                        },
                    );
                    let protocol = if async_plan.is_some() {
                        IteratorProtocolWitness::ASYNC_ITERATOR_PROTOCOL
                    } else {
                        IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL
                    };
                    (
                        StatementIr::ForOfIterator {
                            head: ForOfIteratorHeadIr::Assignment {
                                binding: ForOfAssignmentIr {
                                    mode,
                                    name: storage_name,
                                },
                                async_plan,
                                protocol,
                            },
                            iterable,
                            body: Box::new(body),
                            lexical_environment,
                        },
                        protocol,
                    )
                }
            }
        };
        ForOfLoweringIr::new(statement, body_kind, protocol)
    }
}
