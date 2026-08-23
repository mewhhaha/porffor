use super::*;

impl<'a> ScriptLowerer<'a> {
    fn merge_optional_value_info(
        &self,
        current: Option<ValueInfo>,
        next: Option<ValueInfo>,
    ) -> Option<ValueInfo> {
        match (current, next) {
            (Some(current), Some(next)) => Some(self.merge_value_infos(current, next)),
            (Some(current), None) => Some(current),
            (None, Some(next)) => Some(next),
            (None, None) => None,
        }
    }

    pub(super) fn infer_block_throw_info(&self, block: &BlockIr) -> Option<ValueInfo> {
        let mut info = None;
        for statement in &block.statements {
            info = self.merge_optional_value_info(info, self.infer_statement_throw_info(statement));
        }
        info
    }

    fn infer_statement_throw_info(&self, statement: &StatementIr) -> Option<ValueInfo> {
        match statement {
            // A module unit body can throw anything; the link stage fills these
            // blocks in, and until then no unit block exists to inspect.
            StatementIr::ModuleUnitOnce { .. } => Some(ValueInfo::new(ValueKind::Dynamic)),
            StatementIr::Empty
            | StatementIr::AnnexBFunctionCopy { .. }
            | StatementIr::Debugger
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. } => None,
            StatementIr::Lexical { init, .. } => self.infer_expr_throw_info(init),
            StatementIr::Var(decls) => {
                let mut info = None;
                for decl in decls {
                    if let Some(init) = &decl.init {
                        info =
                            self.merge_optional_value_info(info, self.infer_expr_throw_info(init));
                    }
                }
                info
            }
            StatementIr::Expression(expr)
            | StatementIr::Return(expr)
            | StatementIr::Throw(expr) => {
                let mut info = self.infer_expr_throw_info(expr);
                if matches!(statement, StatementIr::Throw(_)) {
                    info = self.merge_optional_value_info(info, Some(expr.value_info()));
                }
                info
            }
            StatementIr::GeneratorYield {
                value, resume_mode, ..
            } => {
                let mut info = self.infer_expr_throw_info(value);
                if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
                    match reference.use_view() {
                        SuspendedPropertyReferenceUse::Ordinary {
                            base_and_receiver,
                            key,
                            strictness: _,
                        } => {
                            info = self.merge_optional_value_info(
                                info,
                                self.infer_expr_throw_info(base_and_receiver),
                            );
                            if let PropertyKeyIr::StringExpr(expr)
                            | PropertyKeyIr::ArrayIndex(expr) = key
                            {
                                info = self.merge_optional_value_info(
                                    info,
                                    self.infer_expr_throw_info(expr),
                                );
                            }
                        }
                    }
                }
                info
            }
            StatementIr::AsyncAwait { value, .. } => self.infer_expr_throw_info(value),
            StatementIr::Block(block) => self.infer_block_throw_info(block),
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => {
                let mut info = None;
                for statement in statements {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_statement_throw_info(statement),
                    );
                }
                info
            }
            StatementIr::SyncDisposableScope {
                resources, body, ..
            } => {
                let mut info = Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                });
                for resource in resources.iter() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_expr_throw_info(&resource.initializer),
                    );
                }
                self.merge_optional_value_info(info, self.infer_block_throw_info(body))
            }
            StatementIr::AsyncDisposableScope {
                resources, body, ..
            } => {
                let mut info = Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                });
                for resource in resources.iter() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_expr_throw_info(resource.initializer()),
                    );
                }
                self.merge_optional_value_info(info, self.infer_block_throw_info(body))
            }
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut info = self.infer_expr_throw_info(condition);
                info = self
                    .merge_optional_value_info(info, self.infer_statement_throw_info(then_branch));
                info = self.merge_optional_value_info(
                    info,
                    else_branch
                        .as_deref()
                        .and_then(|branch| self.infer_statement_throw_info(branch)),
                );
                info
            }
            StatementIr::While { condition, body } => self.merge_optional_value_info(
                self.infer_expr_throw_info(condition),
                self.infer_statement_throw_info(body),
            ),
            StatementIr::DoWhile { body, condition } => self.merge_optional_value_info(
                self.infer_statement_throw_info(body),
                self.infer_expr_throw_info(condition),
            ),
            StatementIr::For {
                init,
                test,
                update,
                body,
                ..
            } => {
                let mut info = init.as_ref().and_then(|init| match init {
                    ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => {
                        self.infer_expr_throw_info(init)
                    }
                    ForInitIr::LexicalBlock(bindings) => {
                        let mut info = None;
                        for binding in bindings {
                            info = self.merge_optional_value_info(
                                info,
                                self.infer_expr_throw_info(&binding.init),
                            );
                        }
                        info
                    }
                    ForInitIr::Var(decls) => {
                        let mut info = None;
                        for decl in decls {
                            if let Some(init) = &decl.init {
                                info = self.merge_optional_value_info(
                                    info,
                                    self.infer_expr_throw_info(init),
                                );
                            }
                        }
                        info
                    }
                    ForInitIr::Statements(statements) => {
                        statements.iter().fold(None, |info, statement| {
                            self.merge_optional_value_info(
                                info,
                                self.infer_statement_throw_info(statement),
                            )
                        })
                    }
                    ForInitIr::SyncDisposable(resources) => {
                        resources.iter().fold(None, |info, resource| {
                            self.merge_optional_value_info(
                                info,
                                self.infer_expr_throw_info(&resource.initializer),
                            )
                        })
                    }
                    ForInitIr::AsyncDisposable(init) => {
                        init.resources().iter().fold(None, |info, resource| {
                            self.merge_optional_value_info(
                                info,
                                self.infer_expr_throw_info(resource.initializer()),
                            )
                        })
                    }
                });
                info = self.merge_optional_value_info(
                    info,
                    test.as_ref()
                        .and_then(|expr| self.infer_expr_throw_info(expr)),
                );
                info = self.merge_optional_value_info(
                    info,
                    update
                        .as_ref()
                        .and_then(|expr| self.infer_expr_throw_info(expr)),
                );
                info = self.merge_optional_value_info(info, self.infer_statement_throw_info(body));
                info
            }
            StatementIr::GeneratorLoop {
                init,
                test,
                update,
                before_suspension,
                suspension_statement,
                after_suspension,
                ..
            } => {
                let mut info = init.as_ref().and_then(|init| match init {
                    ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => {
                        self.infer_expr_throw_info(init)
                    }
                    ForInitIr::LexicalBlock(bindings) => {
                        bindings.iter().fold(None, |info, binding| {
                            self.merge_optional_value_info(
                                info,
                                self.infer_expr_throw_info(&binding.init),
                            )
                        })
                    }
                    ForInitIr::Var(decls) => decls.iter().fold(None, |info, decl| {
                        self.merge_optional_value_info(
                            info,
                            decl.init
                                .as_ref()
                                .and_then(|init| self.infer_expr_throw_info(init)),
                        )
                    }),
                    ForInitIr::Statements(statements) => {
                        statements.iter().fold(None, |info, statement| {
                            self.merge_optional_value_info(
                                info,
                                self.infer_statement_throw_info(statement),
                            )
                        })
                    }
                    ForInitIr::SyncDisposable(resources) => {
                        resources.iter().fold(None, |info, resource| {
                            self.merge_optional_value_info(
                                info,
                                self.infer_expr_throw_info(&resource.initializer),
                            )
                        })
                    }
                    ForInitIr::AsyncDisposable(init) => {
                        init.resources().iter().fold(None, |info, resource| {
                            self.merge_optional_value_info(
                                info,
                                self.infer_expr_throw_info(resource.initializer()),
                            )
                        })
                    }
                });
                info = self.merge_optional_value_info(
                    info,
                    test.as_ref()
                        .and_then(|test| self.infer_expr_throw_info(test)),
                );
                info = self.merge_optional_value_info(
                    info,
                    update
                        .as_ref()
                        .and_then(|update| self.infer_expr_throw_info(update)),
                );
                for statement in before_suspension
                    .iter()
                    .chain(std::iter::once(suspension_statement.as_ref()))
                    .chain(after_suspension)
                {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_statement_throw_info(statement),
                    );
                }
                info
            }
            StatementIr::GeneratorIf {
                condition,
                then_before_yield,
                then_yield_statement,
                then_after_yield,
                else_before_yield,
                else_yield_statement,
                else_after_yield,
                ..
            } => {
                let mut info = self.infer_expr_throw_info(condition);
                for statement in then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_statement_throw_info(statement),
                    );
                }
                info
            }
            StatementIr::ForOfArray { iterable, body, .. }
            | StatementIr::ForOfString { iterable, body, .. }
            | StatementIr::ForOfIterator { iterable, body, .. } => self.merge_optional_value_info(
                self.infer_expr_throw_info(iterable),
                self.infer_statement_throw_info(body),
            ),
            StatementIr::ForInArray { target, body, .. }
            | StatementIr::ForInString { target, body, .. }
            | StatementIr::ForInObject { target, body, .. } => self.merge_optional_value_info(
                self.infer_expr_throw_info(target),
                self.infer_statement_throw_info(body),
            ),
            StatementIr::Switch {
                discriminant,
                lexical_declarations,
                cases,
                ..
            } => {
                let mut info = self.infer_expr_throw_info(discriminant);
                for declaration in lexical_declarations {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_statement_throw_info(declaration),
                    );
                }
                for case in cases {
                    if let Some(condition) = &case.condition {
                        info = self
                            .merge_optional_value_info(info, self.infer_expr_throw_info(condition));
                    }
                    info = self
                        .merge_optional_value_info(info, self.infer_block_throw_info(&case.body));
                }
                info
            }
            StatementIr::Labelled { statement, .. } => self.infer_statement_throw_info(statement),
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => self.merge_optional_value_info(
                self.infer_block_throw_info(try_block),
                self.infer_block_throw_info(catch_block),
            ),
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => self.merge_optional_value_info(
                self.infer_block_throw_info(try_block),
                self.infer_block_throw_info(finally_block),
            ),
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                let mut info = self.infer_block_throw_info(try_block);
                info =
                    self.merge_optional_value_info(info, self.infer_block_throw_info(catch_block));
                info = self
                    .merge_optional_value_info(info, self.infer_block_throw_info(finally_block));
                info
            }
        }
    }

    /// What an expression can throw, if anything.
    ///
    /// A Reference write throws for two independent reasons: something in its
    /// operands throws, and — PutValue 3.d, `delete` 5.e — the write itself
    /// reports failure while the Reference's `[[Strict]]` is `Strict`. The
    /// second reason is a property of the Reference rather than of any
    /// operand, so it is merged in here, where the carried `[[Strict]]` is
    /// read as a total function of the node.
    ///
    /// This is also the product call site that keeps `carried_put_value_failure`
    /// honest: the exhaustive match inside it only earns its `E0004` if
    /// something outside the tests calls it.
    ///
    /// The error *type* matters as much as the fact of throwing. PutValue 2.a
    /// raises a **ReferenceError** and 3.d a **TypeError**, and this info feeds
    /// `infer_catch_binding_info`, so attributing only a TypeError to
    /// `"use strict"; try { undeclaredXyz = 1 } catch (e) { … }` would narrow
    /// `e` to a TypeError-shaped object — complete with
    /// `prototype: standard_error_prototype_shape(TypeErrorConstructor)` — for
    /// a value that is a ReferenceError.
    fn infer_expr_throw_info(&self, expr: &TypedExpr) -> Option<ValueInfo> {
        let strict_put_value_throw = match carried_put_value_failure(&expr.expr) {
            Some((Strictness::Strict, failure)) => {
                let type_error =
                    Self::standard_error_instance_info(StandardBuiltinId::TypeErrorConstructor);
                Some(match failure {
                    PutValueFailure::TypeErrorOnly => type_error,
                    PutValueFailure::TypeErrorOrReferenceError => self.merge_value_infos(
                        type_error,
                        Self::standard_error_instance_info(
                            StandardBuiltinId::ReferenceErrorConstructor,
                        ),
                    ),
                })
            }
            Some((Strictness::Sloppy, _)) | None => None,
        };
        self.merge_optional_value_info(
            strict_put_value_throw,
            self.infer_expr_operand_throw_info(expr),
        )
    }

    /// The part of [`Self::infer_expr_throw_info`] that comes from the node's
    /// own operands. Recursive calls go back through the wrapper, so a nested
    /// strict Reference write contributes its TypeError too.
    fn infer_expr_operand_throw_info(&self, expr: &TypedExpr) -> Option<ValueInfo> {
        match &expr.expr {
            // `import()` rejects rather than throws, and reading `import.meta`
            // or a namespace object cannot throw.
            ExprIr::DynamicImport { .. }
            | ExprIr::ImportMeta { .. }
            | ExprIr::ModuleNamespace { .. } => None,
            ExprIr::Undefined
            | ExprIr::ArrayHole
            | ExprIr::Null
            | ExprIr::Boolean(_)
            | ExprIr::Number(_)
            | ExprIr::BigInt(_)
            | ExprIr::Symbol { .. }
            | ExprIr::String(_)
            | ExprIr::TemplateObject(_)
            | ExprIr::RegExpLiteral { .. }
            | ExprIr::FunctionValue(_)
            | ExprIr::This
            | ExprIr::Arguments
            | ExprIr::Identifier(_)
            | ExprIr::GlobalPropertyRead { .. }
            | ExprIr::UpdateIdentifier { .. }
            | ExprIr::GlobalPropertyUpdate { .. }
            | ExprIr::NewTarget
            | ExprIr::TypeOfUnresolvedIdentifier { .. } => None,
            ExprIr::SuperPropertyRead { receiver, .. } => self.infer_expr_throw_info(receiver),
            ExprIr::SuperPropertyMutation(mutation) => {
                let mut info = self.infer_expr_throw_info(mutation.receiver());
                if let PropertyKeyIr::StringExpr(key) | PropertyKeyIr::ArrayIndex(key) =
                    mutation.referenced_name()
                {
                    info = self.merge_optional_value_info(info, self.infer_expr_throw_info(key));
                }
                match mutation.operation() {
                    SuperPropertyMutationOperationIr::NumericUpdate { .. } => info,
                    SuperPropertyMutationOperationIr::EagerCompound { result, .. } => {
                        self.merge_optional_value_info(info, self.infer_expr_throw_info(result))
                    }
                }
            }
            ExprIr::GlobalIdentifierRead { .. } => Some(Self::standard_error_instance_info(
                StandardBuiltinId::ReferenceErrorConstructor,
            )),
            // No match here on purpose. `NativeErrorKind::constructor` is total
            // over the nine error intrinsics (20.5.1, 20.5.5, 20.5.7 and
            // Explicit Resource Management), so a tenth kind cannot be omitted
            // at this call site at all — the exhaustiveness obligation lives in
            // the one row list that generates it. The six-arm match this
            // replaced fell through to `ErrorConstructor` for `AggregateError`
            // and `SuppressedError`, which would have typed both as base
            // `Error` and made every downstream `instanceof` and shape
            // inference keyed on the result wrong.
            ExprIr::RuntimeThrow { name, .. } => {
                Some(Self::standard_error_instance_info(name.constructor()))
            }
            ExprIr::ObjectLiteral(properties) => {
                let mut info = None;
                for property in properties {
                    match property {
                        ObjectPropertyIr::PrototypeSetter { value }
                        | ObjectPropertyIr::Spread { source: value }
                        | ObjectPropertyIr::Data { value, .. }
                        | ObjectPropertyIr::NonEnumerableData { value, .. } => {
                            info = self
                                .merge_optional_value_info(info, self.infer_expr_throw_info(value));
                        }
                        ObjectPropertyIr::ComputedData { key, value } => {
                            info = self
                                .merge_optional_value_info(info, self.infer_expr_throw_info(key));
                            info = self
                                .merge_optional_value_info(info, self.infer_expr_throw_info(value));
                        }
                        ObjectPropertyIr::ComputedMethod { key, .. }
                        | ObjectPropertyIr::ComputedGetter { key, .. }
                        | ObjectPropertyIr::ComputedSetter { key, .. } => {
                            info = self
                                .merge_optional_value_info(info, self.infer_expr_throw_info(key));
                        }
                        ObjectPropertyIr::Method { .. }
                        | ObjectPropertyIr::Getter { .. }
                        | ObjectPropertyIr::Setter { .. } => {}
                    }
                }
                info
            }
            ExprIr::ArrayLiteral(elements) => {
                let mut info = None;
                for element in elements {
                    info =
                        self.merge_optional_value_info(info, self.infer_expr_throw_info(element));
                }
                info
            }
            ExprIr::ArrayAccumulation(accumulation) => {
                let mut info = None;
                for element in accumulation.elements() {
                    let value = match element {
                        ArrayAccumulationElementIr::Elision => continue,
                        ArrayAccumulationElementIr::Value(value) => value,
                        ArrayAccumulationElementIr::Spread(spread) => {
                            // @@iterator lookup and iterator calls may throw an
                            // arbitrary language value supplied by user code.
                            info = self.merge_optional_value_info(
                                info,
                                Some(ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                }),
                            );
                            &spread.value
                        }
                    };
                    info = self.merge_optional_value_info(info, self.infer_expr_throw_info(value));
                }
                info
            }
            ExprIr::AssignIdentifier { value, .. }
            | ExprIr::GlobalPropertyWrite { value, .. }
            | ExprIr::CompoundAssignIdentifier { value, .. }
            | ExprIr::GlobalPropertyCompoundAssign { value, .. }
            | ExprIr::SpreadArgument(SpreadArgumentIr { value, .. })
            | ExprIr::Void { expr: value }
            | ExprIr::DeleteValue { expr: value }
            | ExprIr::TypeOf { expr: value }
            | ExprIr::LogicalNot { expr: value }
            | ExprIr::UnaryNumber { expr: value, .. }
            | ExprIr::UnaryBitwiseNumeric { expr: value, .. }
            | ExprIr::StringFromCharCode { code: value } => self.infer_expr_throw_info(value),
            ExprIr::SuperPropertyWrite {
                receiver, value, ..
            } => self.merge_optional_value_info(
                self.infer_expr_throw_info(receiver),
                self.infer_expr_throw_info(value),
            ),
            ExprIr::StringCharCodeAt { target, index } => self.merge_optional_value_info(
                self.infer_expr_throw_info(target),
                self.infer_expr_throw_info(index),
            ),
            ExprIr::SpecOperation {
                operation,
                operands,
            } => {
                let mut info = None;
                for operand in operands {
                    info =
                        self.merge_optional_value_info(info, self.infer_expr_throw_info(operand));
                }
                if *operation == SpecOperationIr::HasProperty {
                    if let Some(target) = operands.first() {
                        let object_like = KindSet::from_kind(ValueKind::Object)
                            .union(KindSet::from_kind(ValueKind::Array))
                            .union(KindSet::from_kind(ValueKind::Arguments))
                            .union(KindSet::from_kind(ValueKind::Function));
                        if !target.possible_kinds.is_subset_of(object_like) {
                            info = self.merge_optional_value_info(
                                info,
                                Some(Self::standard_error_instance_info(
                                    StandardBuiltinId::TypeErrorConstructor,
                                )),
                            );
                        }
                    }
                }
                info
            }
            ExprIr::DeleteIdentifier { .. } | ExprIr::DeleteGlobalProperty { .. } => None,
            ExprIr::PropertyRead { target, key } | ExprIr::DeleteProperty { target, key, .. } => {
                self.merge_optional_value_info(
                    self.infer_expr_throw_info(target),
                    self.infer_property_key_throw_info(key),
                )
            }
            ExprIr::OptionalPropertyChain { target, chain } => {
                let mut info = self.infer_expr_throw_info(target);
                for operation in chain {
                    match operation {
                        OptionalChainOperationIr::Property { key, .. } => {
                            info = self.merge_optional_value_info(
                                info,
                                self.infer_property_key_throw_info(key),
                            );
                        }
                        OptionalChainOperationIr::PrivateProperty { .. } => {
                            info = self.merge_optional_value_info(
                                info,
                                Some(ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                }),
                            );
                        }
                        OptionalChainOperationIr::Call { args, .. } => {
                            info = self.merge_optional_value_info(
                                info,
                                Some(ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                }),
                            );
                            for arg in args {
                                info = self.merge_optional_value_info(
                                    info,
                                    self.infer_expr_throw_info(arg),
                                );
                            }
                        }
                    }
                }
                info
            }
            ExprIr::PropertyWrite {
                target, key, value, ..
            } => {
                let mut info = self.infer_expr_throw_info(target);
                info =
                    self.merge_optional_value_info(info, self.infer_property_key_throw_info(key));
                info = self.merge_optional_value_info(info, self.infer_expr_throw_info(value));
                info
            }
            ExprIr::OrdinaryPropertyAssignment(assignment) => {
                let mut info = self.infer_expr_throw_info(assignment.base_and_receiver());
                info = self.merge_optional_value_info(
                    info,
                    self.infer_property_key_throw_info(assignment.referenced_name()),
                );
                info = self.merge_optional_value_info(info, Some(unknown_runtime_value_info()));
                self.merge_optional_value_info(info, self.infer_expr_throw_info(assignment.rhs()))
            }
            ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {
                let mut info = self.infer_expr_throw_info(assignment.base_and_receiver());
                info = self.merge_optional_value_info(
                    info,
                    self.infer_property_key_throw_info(assignment.referenced_name()),
                );
                // ToPropertyKey, [[Get]], and the branch-local [[Set]] can
                // invoke source accessors or Proxy traps which throw any
                // language value, not merely a shaped strict-Set TypeError.
                info = self.merge_optional_value_info(info, Some(unknown_runtime_value_info()));
                self.merge_optional_value_info(info, self.infer_expr_throw_info(assignment.rhs()))
            }
            ExprIr::OrdinaryPropertyNumericUpdate(update) => {
                let mut info = self.infer_expr_throw_info(update.base_and_receiver());
                info = self.merge_optional_value_info(
                    info,
                    self.infer_property_key_throw_info(update.referenced_name()),
                );
                self.merge_optional_value_info(info, Some(unknown_runtime_value_info()))
            }
            ExprIr::OrdinaryPropertyEagerCompoundAssignment(assignment) => {
                let mut info = self.infer_expr_throw_info(assignment.base_and_receiver());
                info = self.merge_optional_value_info(
                    info,
                    self.infer_property_key_throw_info(assignment.referenced_name()),
                );
                // Ordinary [[Get]], conversion, and [[Set]] hooks can throw an
                // arbitrary ECMAScript value. This unknown contribution is
                // shared by every retained ordinary mutation carrier; strict
                // failed-Set TypeErrors are only one possible throw source.
                info = self.merge_optional_value_info(info, Some(unknown_runtime_value_info()));
                self.merge_optional_value_info(
                    info,
                    self.infer_expr_throw_info(assignment.result()),
                )
            }
            ExprIr::BinaryNumber { lhs, rhs, .. }
            | ExprIr::CoerciveAdd { lhs, rhs }
            | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
            | ExprIr::BitwiseNumeric { lhs, rhs, .. }
            | ExprIr::StringConcat { lhs, rhs }
            | ExprIr::CompareNumber { lhs, rhs, .. }
            | ExprIr::CompareValue { lhs, rhs, .. }
            | ExprIr::StrictEquality { lhs, rhs, .. }
            | ExprIr::LooseEquality { lhs, rhs, .. }
            | ExprIr::AssertSameValue {
                actual: lhs,
                expected: rhs,
                ..
            }
            | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
            | ExprIr::Comma { lhs, rhs }
            | ExprIr::InstanceOf { lhs, rhs } => self.merge_optional_value_info(
                self.infer_expr_throw_info(lhs),
                self.infer_expr_throw_info(rhs),
            ),
            ExprIr::MaterializeBinding { value, body, .. } => self.merge_optional_value_info(
                self.infer_expr_throw_info(value),
                self.infer_expr_throw_info(body),
            ),
            ExprIr::ArrayDestructure { value, pattern, .. } => {
                let mut info = self.infer_expr_throw_info(value);
                pattern.visit_expressions(&mut |expr| {
                    info = self
                        .merge_optional_value_info(info.clone(), self.infer_expr_throw_info(expr));
                });
                info
            }
            ExprIr::ObjectDestructure { value, pattern } => {
                let mut info = self.infer_expr_throw_info(value);
                pattern.visit_expressions(&mut |expr| {
                    info = self
                        .merge_optional_value_info(info.clone(), self.infer_expr_throw_info(expr));
                });
                info
            }
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                let mut info = self.infer_expr_throw_info(condition);
                info = self.merge_optional_value_info(info, self.infer_expr_throw_info(then_expr));
                self.merge_optional_value_info(info, self.infer_expr_throw_info(else_expr))
            }
            ExprIr::CallNamed { args, .. } | ExprIr::SuperConstruct { args } => {
                let mut info = None;
                for arg in args {
                    info = self.merge_optional_value_info(info, self.infer_expr_throw_info(arg));
                }
                info
            }
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
                ..
            } => {
                let mut info = self.infer_expr_throw_info(callee);
                if matches!(
                    self.resolve_single_function_target(callee)
                        .and_then(|function_id| StandardBuiltinId::from_function_id(&function_id)),
                    Some(
                        StandardBuiltinId::ArrayBufferPrototypeResize
                            | StandardBuiltinId::ArrayBufferPrototypeTransfer
                            | StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength
                            | StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable
                    )
                ) {
                    info = self.merge_optional_value_info(
                        info,
                        Some(Self::standard_error_instance_info(
                            StandardBuiltinId::TypeErrorConstructor,
                        )),
                    );
                }
                if let Some(this_arg) = this_arg {
                    info =
                        self.merge_optional_value_info(info, self.infer_expr_throw_info(this_arg));
                }
                for arg in args {
                    info = self.merge_optional_value_info(info, self.infer_expr_throw_info(arg));
                }
                info
            }
            ExprIr::JsonParseStaticReviver { reviver, .. } => self.infer_expr_throw_info(reviver),
            ExprIr::Construct { callee, args, .. } => {
                let mut info = self.infer_expr_throw_info(callee);
                for arg in args {
                    info = self.merge_optional_value_info(info, self.infer_expr_throw_info(arg));
                }
                info
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                let mut info = self.infer_expr_throw_info(receiver);
                info =
                    self.merge_optional_value_info(info, self.infer_property_key_throw_info(key));
                for arg in args {
                    info = self.merge_optional_value_info(info, self.infer_expr_throw_info(arg));
                }
                info
            }
            ExprIr::ClassDefinition(class) => {
                let mut info = class
                    .heritage
                    .as_deref()
                    .and_then(|heritage| self.infer_expr_throw_info(heritage));
                for definition in &class.element_plan.definitions {
                    let key = match definition {
                        ClassElementDefinitionIr::PublicMethod(method) => Some(&method.key),
                        ClassElementDefinitionIr::AutoAccessor(accessor) => {
                            accessor.computed_key.as_ref()
                        }
                        ClassElementDefinitionIr::PrivateMethod(_)
                        | ClassElementDefinitionIr::ComputedFieldKey { .. } => None,
                    };
                    let Some(key) = key else { continue };
                    info = self
                        .merge_optional_value_info(info, self.infer_property_key_throw_info(key));
                }
                info
            }
            ExprIr::PrivateRead { target, .. } => self.infer_expr_throw_info(target),
            ExprIr::PrivateWrite { target, value, .. } => self.merge_optional_value_info(
                self.infer_expr_throw_info(target),
                self.infer_expr_throw_info(value),
            ),
            ExprIr::PrivateIn { rhs, .. } => {
                let mut info = self.infer_expr_throw_info(rhs);
                if !matches!(rhs.expr, ExprIr::RuntimeThrow { .. })
                    && !rhs
                        .possible_kinds
                        .is_subset_of(Self::object_like_kind_set())
                {
                    info = self.merge_optional_value_info(
                        info,
                        Some(Self::standard_error_instance_info(
                            StandardBuiltinId::TypeErrorConstructor,
                        )),
                    );
                }
                info
            }
            ExprIr::In { lhs, rhs } => {
                let mut info = self.merge_optional_value_info(
                    self.infer_expr_throw_info(lhs),
                    self.infer_expr_throw_info(rhs),
                );
                let rhs_object_like = KindSet::from_kind(ValueKind::Object)
                    .union(KindSet::from_kind(ValueKind::Array))
                    .union(KindSet::from_kind(ValueKind::Arguments))
                    .union(KindSet::from_kind(ValueKind::Function));
                if !rhs.possible_kinds.is_subset_of(rhs_object_like) {
                    info = self.merge_optional_value_info(
                        info,
                        Some(Self::standard_error_instance_info(
                            StandardBuiltinId::TypeErrorConstructor,
                        )),
                    );
                }
                info
            }
        }
    }

    fn infer_property_key_throw_info(&self, key: &PropertyKeyIr) -> Option<ValueInfo> {
        match key {
            PropertyKeyIr::StaticString(_)
            | PropertyKeyIr::ArrayIndex(_)
            | PropertyKeyIr::ArrayLength => None,
            PropertyKeyIr::StringExpr(expr) => self.infer_expr_throw_info(expr),
        }
    }
}
