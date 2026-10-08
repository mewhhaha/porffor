mod expression;

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
        self.infer_statement_sequence_throw_info(&block.statements)
    }

    fn infer_statement_sequence_throw_info(&self, statements: &[StatementIr]) -> Option<ValueInfo> {
        statements.iter().fold(None, |info, statement| {
            self.merge_optional_value_info(info, self.infer_statement_throw_info(statement))
        })
    }

    fn infer_statement_throw_info(&self, statement: &StatementIr) -> Option<ValueInfo> {
        match statement {
            StatementIr::ResumableClassDefinition(_) => Some(unknown_runtime_value_info()),
            // A module unit body can throw anything; the link stage fills these
            // blocks in, and until then no unit block exists to inspect.
            StatementIr::ModuleUnitOnce { .. } => Some(ValueInfo::new(ValueKind::Dynamic)),
            StatementIr::ModuleImportBinding(_) => None,
            StatementIr::AsyncModuleInstantiation
            | StatementIr::Empty
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
            StatementIr::DeclarationEvaluation(expr)
            | StatementIr::Expression(expr)
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
            StatementIr::EmptyStatementCompletion(item) => {
                self.infer_statement_throw_info(item.statement())
            }
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => {
                self.infer_statement_sequence_throw_info(statements)
            }
            StatementIr::SyncDisposableScope {
                resources, body, ..
            } => {
                let mut info = Some(ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: FunctionTargetKnowledge::unknown(),
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
                    function_targets: FunctionTargetKnowledge::unknown(),
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
            }
            | StatementIr::AsyncFunctionIf {
                condition,
                then_branch,
                else_branch,
                plan: _,
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
            StatementIr::AsyncFunctionWhile(plan) => {
                let prefix = self.infer_statement_sequence_throw_info(plan.condition_prefix());
                let test = self.infer_expr_throw_info(plan.condition());
                let body = self.infer_statement_throw_info(plan.body());
                self.merge_optional_value_info(self.merge_optional_value_info(prefix, test), body)
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
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                let mut info = None;
                for region in plan.regions() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_block_throw_info(region.block()),
                    );
                }
                for expression in plan.expressions() {
                    info = self
                        .merge_optional_value_info(info, self.infer_expr_throw_info(expression));
                }
                info
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                let mut info = None;
                for region in plan.regions() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_block_throw_info(region.block()),
                    );
                }
                for expression in plan.expressions() {
                    info = self
                        .merge_optional_value_info(info, self.infer_expr_throw_info(expression));
                }
                info
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                let mut info = self.infer_expr_throw_info(plan.condition());
                for region in [plan.then_branch(), plan.else_branch()] {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_block_throw_info(region.block()),
                    );
                }
                info
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                let mut info = self.infer_expr_throw_info(plan.condition().value());
                for region in [
                    plan.condition().region(),
                    plan.then_branch(),
                    plan.else_branch(),
                ] {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_block_throw_info(region.block()),
                    );
                }
                info
            }
            StatementIr::ArrayDestructuringOperation(_) => Some(unknown_runtime_value_info()),
            StatementIr::AsyncGeneratorForIn(plan) => {
                // OwnKeys, descriptor and prototype queries can run user code.
                let mut info = Some(unknown_runtime_value_info());
                for block in [
                    plan.head().region().block(),
                    plan.initialization(),
                    plan.body().block(),
                ] {
                    info = self.merge_optional_value_info(info, self.infer_block_throw_info(block));
                }
                info
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                // Iterator acquisition, stepping and closing can run user code.
                let mut info = Some(unknown_runtime_value_info());
                for block in [
                    plan.head().region().block(),
                    plan.initialization().block(),
                    plan.body().block(),
                ] {
                    info = self.merge_optional_value_info(info, self.infer_block_throw_info(block));
                }
                info
            }
            StatementIr::OrdinaryGeneratorWith(plan) => {
                let head = self.infer_block_throw_info(plan.head().region().block());
                let body = self.infer_block_throw_info(plan.body().block());
                self.merge_optional_value_info(head, body)
            }
            StatementIr::AsyncGeneratorWith(plan) => {
                let head = self.infer_block_throw_info(plan.head().region().block());
                let body = self.infer_block_throw_info(plan.body().block());
                let children = self.merge_optional_value_info(head, body);
                self.merge_optional_value_info(Some(unknown_runtime_value_info()), children)
            }
            StatementIr::AsyncFunctionWith(plan) => {
                let head = self.infer_block_throw_info(plan.head());
                let body = self.infer_block_throw_info(plan.body());
                let children = self.merge_optional_value_info(head, body);
                self.merge_optional_value_info(Some(unknown_runtime_value_info()), children)
            }
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                let body = self.infer_block_throw_info(plan.body().block());
                self.merge_optional_value_info(Some(unknown_runtime_value_info()), body)
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                let body = self.infer_block_throw_info(plan.body().block());
                self.merge_optional_value_info(Some(unknown_runtime_value_info()), body)
            }
            StatementIr::AsyncGeneratorResourceRegistration(_) => {
                Some(unknown_runtime_value_info())
            }
            StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
                let body = self.infer_block_throw_info(plan.body().block());
                self.merge_optional_value_info(Some(unknown_runtime_value_info()), body)
            }
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                let body = self.infer_block_throw_info(plan.body());
                self.merge_optional_value_info(Some(unknown_runtime_value_info()), body)
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                let mut info = self.infer_block_throw_info(plan.discriminant().region().block());
                info = self.merge_optional_value_info(
                    info,
                    self.infer_expr_throw_info(plan.discriminant().value()),
                );
                for declaration in plan.lexical_declarations() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_statement_throw_info(declaration),
                    );
                }
                for case in plan.cases() {
                    if let Some(selector) = case.selector() {
                        info = self.merge_optional_value_info(
                            info,
                            self.infer_block_throw_info(selector.region().block()),
                        );
                        info = self.merge_optional_value_info(
                            info,
                            self.infer_expr_throw_info(selector.value()),
                        );
                    }
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_block_throw_info(case.body().block()),
                    );
                }
                info
            }
            StatementIr::AsyncGeneratorSwitch(plan) => {
                let mut info = None;
                for region in plan.regions() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_block_throw_info(region.block()),
                    );
                }
                for expression in plan.expressions() {
                    info = self
                        .merge_optional_value_info(info, self.infer_expr_throw_info(expression));
                }
                for declaration in plan.lexical_declarations() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_statement_throw_info(declaration),
                    );
                }
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
            StatementIr::AsyncFunctionForOfIterator { iterable, plan } => self
                .merge_optional_value_info(
                    self.infer_expr_throw_info(iterable),
                    self.infer_statement_sequence_throw_info(plan.body().statements()),
                ),
            StatementIr::GeneratorForOfIterator { iterable, plan } => self
                .merge_optional_value_info(
                    self.infer_expr_throw_info(iterable),
                    self.infer_statement_sequence_throw_info(plan.body().statements()),
                ),
            StatementIr::ForOfIterator { iterable, body, .. } => self.merge_optional_value_info(
                self.infer_expr_throw_info(iterable),
                self.infer_statement_throw_info(body),
            ),
            StatementIr::ForInArray { target, body, .. }
            | StatementIr::ForInString { target, body, .. }
            | StatementIr::ForInObject { target, body, .. } => self.merge_optional_value_info(
                self.infer_expr_throw_info(target),
                self.infer_statement_throw_info(body),
            ),
            StatementIr::AsyncFunctionSwitch(plan) => {
                let mut info = self.infer_expr_throw_info(plan.discriminant());
                for declaration in plan.lexical_declarations() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_statement_throw_info(declaration),
                    );
                }
                for case in plan.cases() {
                    info = self.merge_optional_value_info(
                        info,
                        self.infer_statement_sequence_throw_info(case.condition_prefix()),
                    );
                    if let Some(condition) = case.condition() {
                        info = self
                            .merge_optional_value_info(info, self.infer_expr_throw_info(condition));
                    }
                    info = self
                        .merge_optional_value_info(info, self.infer_block_throw_info(case.body()));
                }
                info
            }
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

    /// Merge operand throws with carried strict Reference failures: PutValue 2.a
    /// raises ReferenceError; 3.d and `delete` 5.e raise TypeError on write failure.
    /// Read `[[Strict]]` per node and preserve both error shapes for catch narrowing:
    /// unresolvable writes must not acquire a TypeError prototype in `infer_catch_binding_info`.
    /// This product call consumes `carried_put_value_failure`'s exhaustive match/E0004.
    fn infer_expr_throw_info(&self, expr: &TypedExpr) -> Option<ValueInfo> {
        let strict_put_value_throw = match carried_put_value_failure(&expr.expr) {
            Some((Strictness::Strict, failure)) => {
                let type_error =
                    self.standard_error_instance_info(StandardBuiltinId::TypeErrorConstructor);
                Some(match failure {
                    PutValueFailure::TypeErrorOnly => type_error,
                    PutValueFailure::TypeErrorOrReferenceError => self.merge_value_infos(
                        type_error,
                        self.standard_error_instance_info(
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
}
