//! Eager With and assignable ForIn phases need complete enclosing control.

use super::*;
use boa_ast::statement::iteration::ForInLoop;
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

pub(crate) fn contains_ordinary_generator_phase_owner<N: VisitWith + ?Sized>(source: &N) -> bool {
    struct Find;
    impl<'ast> Visitor<'ast> for Find {
        type BreakTy = ();

        fn visit_statement_list(&mut self, source: &'ast StatementList) -> ControlFlow<()> {
            if crate::async_generator_source::AsyncGeneratorResourceScopeSource::for_protocol(
                source.statements(),
                ResumableRegionProtocolIr::Generator,
            )
            .is_some()
            {
                return ControlFlow::Break(());
            }
            source.visit_with(self)
        }

        fn visit_statement(&mut self, source: &'ast Statement) -> ControlFlow<()> {
            match source {
                Statement::With(_)
                | Statement::ForLoop(_)
                | Statement::WhileLoop(_)
                | Statement::DoWhileLoop(_) => ControlFlow::Break(()),
                Statement::ForInLoop(_) | Statement::ForOfLoop(_) => source.visit_with(self),
                Statement::Var(_)
                | Statement::Empty
                | Statement::Debugger
                | Statement::Expression(_)
                | Statement::Block(_)
                | Statement::If(_)
                | Statement::Switch(_)
                | Statement::Labelled(_)
                | Statement::Break(_)
                | Statement::Continue(_)
                | Statement::Throw(_)
                | Statement::Try(_)
                | Statement::Return(_) => source.visit_with(self),
            }
        }

        // Typed AST roots visit their children directly. Keep the owner check
        // at the typed visitor too, including a labelled iterator's base node.
        fn visit_for_in_loop(&mut self, source: &'ast ForInLoop) -> ControlFlow<()> {
            if crate::async_generator_source::AsyncGeneratorForInSource::for_execution(
                source,
                ResumableRegionProtocolIr::Generator,
            )
            .is_some()
            {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }

        fn visit_for_of_loop(&mut self, source: &'ast ForOfLoop) -> ControlFlow<()> {
            if crate::async_generator_source::AsyncGeneratorForOfSource::for_execution(
                source,
                ResumableRegionProtocolIr::Generator,
            )
            .is_some()
            {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }

        fn visit_if(&mut self, source: &'ast If) -> ControlFlow<()> {
            if crate::lowering_helpers::ordinary_generator_if_requires_complete_owner(source) {
                ControlFlow::Break(())
            } else {
                source.visit_with(self)
            }
        }

        fn visit_function_body(&mut self, _: &'ast FunctionBody) -> ControlFlow<()> {
            ControlFlow::Continue(())
        }
        fn visit_class_declaration(&mut self, _: &'ast ClassDeclaration) -> ControlFlow<()> {
            ControlFlow::Continue(())
        }
        fn visit_class_expression(&mut self, _: &'ast ClassExpression) -> ControlFlow<()> {
            ControlFlow::Continue(())
        }
    }
    source.visit_with(&mut Find).is_break()
}
