use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Lower one ordinary property numeric update into the same non-cloneable
    /// Reference plan used by eager compound assignment. Runtime type is kept
    /// dynamic: the property Get can produce either Number or BigInt and owns
    /// the sole ToNumeric operation.
    pub(super) fn lower_ordinary_property_numeric_update(
        &mut self,
        op: UpdateOp,
        access: &boa_ast::expression::access::SimplePropertyAccess,
    ) -> TypedExpr {
        let (op, return_mode) = match op {
            UpdateOp::IncrementPost => (NumericUpdateOp::Increment, UpdateReturnMode::Postfix),
            UpdateOp::IncrementPre => (NumericUpdateOp::Increment, UpdateReturnMode::Prefix),
            UpdateOp::DecrementPost => (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix),
            UpdateOp::DecrementPre => (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix),
        };
        let (plan, referenced_name, _) = self.lower_ordinary_property_reference_plan(access);
        let result = plan.numeric_update(op, return_mode);
        self.update_written_shape(access.target(), &referenced_name, &result.value_info());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};

    fn lower(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        crate::lower(&source)
    }

    fn returned_update<'a>(
        script: &'a ScriptIr,
        function_name: &str,
    ) -> &'a OrdinaryPropertyNumericUpdateIr {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("missing function {function_name}"));
        let StatementIr::Return(value) = function
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("function should return its update")
        else {
            unreachable!("selected statement is a return")
        };
        let ExprIr::OrdinaryPropertyNumericUpdate(update) = &value.expr else {
            panic!(
                "expected fused ordinary property update, got {:?}",
                value.expr
            );
        };
        update
    }

    #[test]
    fn ordinary_property_numeric_update_owns_one_reference() {
        let program = lower(
            r#"
            function postIncrement(base, key) { "use strict"; return base[key]++; }
            function preIncrement(base, key) { "use strict"; return ++base[key]; }
            function postDecrement(base, key) { "use strict"; return base[key]--; }
            function preDecrement(base, key) { "use strict"; return --base[key]; }
            "#,
        );
        let script = program.script.as_ref().expect("script IR should exist");

        for (name, expected_op, expected_mode) in [
            (
                "postIncrement",
                NumericUpdateOp::Increment,
                UpdateReturnMode::Postfix,
            ),
            (
                "preIncrement",
                NumericUpdateOp::Increment,
                UpdateReturnMode::Prefix,
            ),
            (
                "postDecrement",
                NumericUpdateOp::Decrement,
                UpdateReturnMode::Postfix,
            ),
            (
                "preDecrement",
                NumericUpdateOp::Decrement,
                UpdateReturnMode::Prefix,
            ),
        ] {
            let update = returned_update(script, name);
            assert!(matches!(
                &update.base_and_receiver().expr,
                ExprIr::Identifier(_)
            ));
            assert!(matches!(
                update.referenced_name(),
                PropertyKeyIr::StringExpr(key)
                    if matches!(&key.expr, ExprIr::Identifier(_))
            ));
            assert_eq!(update.strictness(), Strictness::Strict);
            assert_eq!(update.op(), expected_op);
            assert_eq!(update.return_mode(), expected_mode);
            assert_eq!(update.value_kind(), ValueKind::Dynamic);
        }
    }
}
