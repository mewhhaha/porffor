//! AST-only admission for the actual complete mixed statement-region owner.
use super::*;

pub(super) fn admits<T: VisitWith>(source: &T) -> bool {
    source.visit_with(&mut MixedRegionGate).is_continue()
}

struct MixedRegionGate;

impl<'ast> Visitor<'ast> for MixedRegionGate {
    type BreakTy = ();

    fn visit_statement(&mut self, source: &'ast Statement) -> ControlFlow<()> {
        match source {
            Statement::Block(_)
            | Statement::ForLoop(_)
            | Statement::WhileLoop(_)
            | Statement::DoWhileLoop(_)
            | Statement::If(_)
            | Statement::Try(_)
            | Statement::With(_)
            | Statement::Switch(_)
            | Statement::ForInLoop(_)
            | Statement::Empty
            | Statement::Var(_)
            | Statement::Expression(_)
            | Statement::Return(_)
            | Statement::Throw(_)
            | Statement::Break(_)
            | Statement::Continue(_)
            | Statement::ForOfLoop(_)
            | Statement::Labelled(_)
            | Statement::Debugger => source.visit_with(self),
        }
    }
    fn visit_for_in_loop(
        &mut self,
        source: &'ast boa_ast::statement::iteration::ForInLoop,
    ) -> ControlFlow<()> {
        if matches!(
            source.initializer(),
            IterableLoopInitializer::Using(_) | IterableLoopInitializer::AwaitUsing(_)
        ) || (crate::lowering_helpers::mixed_for_in_head_mode_and_kind(source).is_none()
            && has_suspension(source))
        {
            return ControlFlow::Break(());
        }
        // This is one AST shape walk. It never dry-allocates nested factories.
        // Phase-free unsupported heads retain the original eager path.
        source.visit_with(self)
    }
    fn visit_for_of_loop(&mut self, source: &'ast ForOfLoop) -> ControlFlow<()> {
        if for_of::head_mode(source).is_none()
            && (source.r#await()
                || has_suspension(source)
                || matches!(
                    source.initializer(),
                    IterableLoopInitializer::Using(_) | IterableLoopInitializer::AwaitUsing(_)
                ))
        {
            return ControlFlow::Break(());
        }
        source.visit_with(self)
    }
    fn visit_switch(&mut self, source: &'ast AstSwitch) -> ControlFlow<()> {
        source.visit_with(self)
    }
    fn visit_lexical_declaration(&mut self, source: &'ast LexicalDeclaration) -> ControlFlow<()> {
        match source {
            LexicalDeclaration::Using(list) | LexicalDeclaration::AwaitUsing(list) => {
                if list.as_ref().iter().any(|variable| {
                    !matches!(variable.binding(), Binding::Identifier(_))
                        || variable.init().is_none()
                }) {
                    return ControlFlow::Break(());
                }
                source.visit_with(self)
            }
            LexicalDeclaration::Let(_) | LexicalDeclaration::Const(_) => source.visit_with(self),
        }
    }
    fn visit_statement_list_item(&mut self, source: &'ast StatementListItem) -> ControlFlow<()> {
        source.visit_with(self)
    }
    fn visit_function_body(&mut self, _: &'ast FunctionBody) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
    fn visit_formal_parameter_list(&mut self, _: &'ast FormalParameterList) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
    fn visit_class_declaration(&mut self, source: &'ast ClassDeclaration) -> ControlFlow<()> {
        for expression in class_evaluation_expressions(source.super_ref(), source.elements()) {
            expression.visit_with(self)?;
        }
        ControlFlow::Continue(())
    }
    fn visit_class_expression(&mut self, source: &'ast ClassExpression) -> ControlFlow<()> {
        for expression in class_evaluation_expressions(source.super_ref(), source.elements()) {
            expression.visit_with(self)?;
        }
        ControlFlow::Continue(())
    }
    fn visit_object_method_definition(
        &mut self,
        source: &'ast ObjectMethodDefinition,
    ) -> ControlFlow<()> {
        source.name().visit_with(self)
    }
}
