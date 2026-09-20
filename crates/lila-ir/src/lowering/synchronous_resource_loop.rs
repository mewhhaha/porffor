use super::*;

/// A resource loop whose eagerly evaluated source cannot suspend. Its private
/// constructor is the only entry to lowering without allocating continuation
/// states for nested ordinary try clauses. The function protocol is retained.
pub(super) struct SynchronousResourceLoop<'ast> {
    source: ResourceLoopSource<'ast>,
}

enum ResourceLoopSource<'ast> {
    Classic(&'ast ForLoop),
    Iterator(&'ast ForOfLoop),
}

impl<'ast> SynchronousResourceLoop<'ast> {
    pub(super) fn classic(source: &'ast ForLoop) -> Option<Self> {
        source
            .visit_with(&mut SourceSuspension)
            .is_continue()
            .then_some(Self {
                source: ResourceLoopSource::Classic(source),
            })
    }

    pub(super) fn iterator(source: &'ast ForOfLoop) -> Option<Self> {
        SourceSuspension
            .visit_for_of_loop(source)
            .is_continue()
            .then_some(Self {
                source: ResourceLoopSource::Iterator(source),
            })
    }

    pub(super) fn lower(self, lowerer: &mut ScriptLowerer<'_>) -> (StatementIr, ValueKind) {
        let Some(owner) = lowerer.admit_sync_disposable_scope_owner() else {
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        match owner {
            SyncDisposableScopeOwnerPlan::Immediate
            | SyncDisposableScopeOwnerPlan::AsyncFunction => {}
            SyncDisposableScopeOwnerPlan::PlainGenerator
            | SyncDisposableScopeOwnerPlan::AsyncGenerator => {
                lowerer.unsupported("synchronous resource loop in a generator");
                return (StatementIr::Empty, ValueKind::Undefined);
            }
        }
        // Only this proven region omits continuation allocation. Keeping the
        // analyzed owner intact preserves activation-backed nested using scopes,
        // return settlement, captured environments and the execution Realm.
        let continuation = lowerer.current_async_resume_state.take();
        let result = match self.source {
            ResourceLoopSource::Classic(source) => lowerer.lower_for_loop_region(source),
            ResourceLoopSource::Iterator(source) => lowerer.lower_for_of_loop_region(source),
        };
        assert!(
            lowerer.current_async_resume_state.is_none(),
            "a synchronous resource region must not allocate a continuation"
        );
        lowerer.current_async_resume_state = continuation;
        result
    }
}

struct SourceSuspension;

impl<'ast> Visitor<'ast> for SourceSuspension {
    type BreakTy = ();

    fn visit_await(&mut self, _: &'ast boa_ast::expression::Await) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    fn visit_yield(&mut self, _: &'ast boa_ast::expression::Yield) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    fn visit_lexical_declaration(
        &mut self,
        declaration: &'ast LexicalDeclaration,
    ) -> ControlFlow<()> {
        match declaration {
            LexicalDeclaration::AwaitUsing(_) => ControlFlow::Break(()),
            LexicalDeclaration::Let(_)
            | LexicalDeclaration::Const(_)
            | LexicalDeclaration::Using(_) => declaration.visit_with(self),
        }
    }

    fn visit_for_of_loop(&mut self, source: &'ast ForOfLoop) -> ControlFlow<()> {
        if source.r#await()
            || matches!(source.initializer(), IterableLoopInitializer::AwaitUsing(_))
        {
            return ControlFlow::Break(());
        }
        source.visit_with(self)
    }

    fn visit_function_body(&mut self, _: &'ast FunctionBody) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_formal_parameter_list(&mut self, _: &'ast FormalParameterList) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_class_element(&mut self, element: &'ast ClassElement) -> ControlFlow<()> {
        match element {
            ClassElement::FieldDefinition(field) | ClassElement::AccessorFieldDefinition(field) => {
                for decorator in field.decorators() {
                    self.visit_expression(decorator)?;
                }
                // Instance initializers execute later, but their computed names
                // and decorators are evaluated while the class is defined.
                self.visit_property_name(field.name())
            }
            ClassElement::PrivateFieldDefinition(field) => {
                for decorator in field.decorators() {
                    self.visit_expression(decorator)?;
                }
                ControlFlow::Continue(())
            }
            ClassElement::StaticBlock(block) => {
                self.visit_statement_list(block.statements().statement_list())
            }
            ClassElement::MethodDefinition(_)
            | ClassElement::StaticFieldDefinition(_)
            | ClassElement::StaticAccessorFieldDefinition(_)
            | ClassElement::PrivateStaticFieldDefinition(_) => element.visit_with(self),
        }
    }
}
