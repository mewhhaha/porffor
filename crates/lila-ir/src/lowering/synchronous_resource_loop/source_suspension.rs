use super::*;

/// Eager source inspection shares the actual activation and class-definition
/// boundaries. Resumable owners use their complete typed source tapes instead.
pub(super) struct SourceSuspension;

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
