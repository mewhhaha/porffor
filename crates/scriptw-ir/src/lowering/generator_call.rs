use super::*;

impl ScriptLowerer<'_> {
    fn retain_generator_operand(
        &mut self,
        statements: &mut Vec<StatementIr>,
        value: TypedExpr,
        hint: &str,
    ) -> TypedExpr {
        let mut retained_info = value.value_info();
        // Later key or RHS evaluation can mutate the retained receiver. Keep
        // its whole Value and callable identity, but discard a stale shape.
        retained_info.heap_shape = None;
        let name = self.alloc_suspension_owned_binding(hint, retained_info);
        statements.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: name.clone(),
            init: value,
        });
        self.lower_identifier_name(name, false)
    }

    pub(super) fn lower_staged_generator_property(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
    ) -> Option<(Vec<StatementIr>, TypedExpr, TypedExpr)> {
        let (mut statements, target) = self.lower_staged_generator_expression(access.target())?;
        let target = self.retain_generator_operand(&mut statements, target, "generator.receiver.");
        let key = match access.field() {
            PropertyAccessField::Const(field) => {
                PropertyKeyIr::StaticString(self.interner.resolve_expect(field.sym()).to_string())
            }
            PropertyAccessField::Expr(expression) => {
                let (prefix, key) = self.lower_staged_generator_expression(expression)?;
                statements.extend(prefix);
                PropertyKeyIr::StringExpr(Box::new(key))
            }
        };
        // Keep the ordinary property Reference operation: computed key
        // evaluation, the nullish check, ToPropertyKey and Get retain their
        // specified order, and a later Call retains the original receiver.
        self.invalidate_unknown_user_code_effects();
        let value = TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::PropertyRead {
                target: Box::new(target.clone()),
                key,
            },
        );
        Some((statements, target, value))
    }

    pub(super) fn lower_staged_generator_property_assignment(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
        rhs: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let protocol = super::resumable_operand::ResumableOperandProtocol::current(self)?;
        let (mut statements, target) = protocol.lower(self, access.target())?;
        let target = self.retain_generator_operand(&mut statements, target, "generator.receiver.");
        let key = match access.field() {
            PropertyAccessField::Const(field) => {
                PropertyKeyIr::StaticString(self.interner.resolve_expect(field.sym()).to_string())
            }
            PropertyAccessField::Expr(expression) => {
                let (prefix, key) = protocol.lower(self, expression)?;
                statements.extend(prefix);
                let key = self.retain_generator_operand(&mut statements, key, "generator.key.");
                PropertyKeyIr::StringExpr(Box::new(key))
            }
        };
        let (prefix, value) = protocol.lower(self, rhs)?;
        statements.extend(prefix);

        // A plain assignment retains the raw Reference. Its existing consumer
        // evaluates the RHS before ToObject, ToPropertyKey and Set; using the
        // property-read staging path here would also introduce an unwanted Get.
        let (_, mut possible_setters) = self.possible_unknown_accessor_functions();
        possible_setters.extend_known(self.dynamically_installed_setters.iter().cloned());
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();
        let reference =
            OrdinaryPropertyReferencePlan::new(Box::new(target), key, self.reference_strictness());
        Some((
            statements,
            reference.plain_assignment(value, possible_setters),
        ))
    }
}
