use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DerivedConstructorValidation {
    pub(crate) super_calls: usize,
    pub(crate) saw_super: bool,
    pub(crate) this_before_super: bool,
    pub(crate) return_before_super: bool,
}

fn expr_contains_this_before_super(expr: &TypedExpr, state: &mut DerivedConstructorValidation) {
    if state.saw_super {
        return;
    }
    match &expr.expr {
        ExprIr::This => state.this_before_super = true,
        ExprIr::Identifier(name) if name == LEXICAL_THIS_NAME => state.this_before_super = true,
        ExprIr::SuperConstruct { .. } => {
            state.super_calls += 1;
            state.saw_super = true;
        }
        ExprIr::UnaryNumber { expr: operand, .. }
        | ExprIr::SpreadArgument(operand)
        | ExprIr::StringFromCharCode { code: operand }
        | ExprIr::TypeOf { expr: operand }
        | ExprIr::Void { expr: operand }
        | ExprIr::DeleteValue { expr: operand }
        | ExprIr::LogicalNot { expr: operand }
        | ExprIr::PropertyRead {
            target: operand, ..
        } => {
            expr_contains_this_before_super(operand, state);
        }
        ExprIr::StringCharCodeAt { target, index } => {
            expr_contains_this_before_super(target, state);
            expr_contains_this_before_super(index, state);
        }
        ExprIr::SpecOperation { operands, .. } => {
            for operand in operands {
                expr_contains_this_before_super(operand, state);
            }
        }
        ExprIr::DeleteIdentifier { .. } | ExprIr::DeleteGlobalProperty { .. } => {}
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::BitwiseNumber { lhs, rhs, .. }
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
        | ExprIr::Comma { lhs, rhs } => {
            expr_contains_this_before_super(lhs, state);
            expr_contains_this_before_super(rhs, state);
        }
        ExprIr::LogicalShortCircuit { lhs, rhs, .. } => {
            expr_contains_this_before_super(lhs, state);
            expr_contains_this_before_super(rhs, state);
        }
        ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_contains_this_before_super(condition, state);
            expr_contains_this_before_super(then_expr, state);
            expr_contains_this_before_super(else_expr, state);
        }
        ExprIr::CallIndirect {
            callee,
            this_arg,
            args,
        } => {
            expr_contains_this_before_super(callee, state);
            if let Some(this_arg) = this_arg {
                expr_contains_this_before_super(this_arg, state);
            }
            for arg in args {
                expr_contains_this_before_super(arg, state);
            }
        }
        ExprIr::JsonParseStaticReviver { reviver, .. } => {
            expr_contains_this_before_super(reviver, state);
        }
        ExprIr::Construct { callee, args } => {
            expr_contains_this_before_super(callee, state);
            for arg in args {
                expr_contains_this_before_super(arg, state);
            }
        }
        ExprIr::CallMethod { receiver, args, .. } => {
            expr_contains_this_before_super(receiver, state);
            for arg in args {
                expr_contains_this_before_super(arg, state);
            }
        }
        ExprIr::PropertyWrite { target, value, .. }
        | ExprIr::PrivateWrite { target, value, .. } => {
            expr_contains_this_before_super(target, state);
            expr_contains_this_before_super(value, state);
        }
        ExprIr::PropertyUpdate { target, key, .. } => {
            expr_contains_this_before_super(target, state);
            match key {
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    expr_contains_this_before_super(expr, state);
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
        }
        ExprIr::DeleteProperty { target, key, .. } => {
            expr_contains_this_before_super(target, state);
            match key {
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    expr_contains_this_before_super(expr, state);
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
        }
        ExprIr::In { lhs, rhs } => {
            expr_contains_this_before_super(lhs, state);
            expr_contains_this_before_super(rhs, state);
        }
        ExprIr::ArrayLiteral(elements) => {
            for element in elements {
                expr_contains_this_before_super(element, state);
            }
        }
        ExprIr::ObjectLiteral(properties) => {
            for property in properties {
                match property {
                    ObjectPropertyIr::Data { value, .. }
                    | ObjectPropertyIr::NonEnumerableData { value, .. }
                    | ObjectPropertyIr::Method {
                        function: value, ..
                    }
                    | ObjectPropertyIr::Getter {
                        function: value, ..
                    }
                    | ObjectPropertyIr::Setter {
                        function: value, ..
                    } => {
                        expr_contains_this_before_super(value, state);
                    }
                    ObjectPropertyIr::ComputedData { key, value } => {
                        expr_contains_this_before_super(key, state);
                        expr_contains_this_before_super(value, state);
                    }
                    ObjectPropertyIr::ComputedMethod { key, function }
                    | ObjectPropertyIr::ComputedGetter { key, function }
                    | ObjectPropertyIr::ComputedSetter { key, function } => {
                        expr_contains_this_before_super(key, state);
                        expr_contains_this_before_super(function, state);
                    }
                }
            }
        }
        ExprIr::ClassDefinition(class) => {
            if let Some(heritage) = &class.heritage {
                expr_contains_this_before_super(heritage, state);
            }
            for method in &class.public_methods {
                match &method.key {
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_contains_this_before_super(expr, state);
                    }
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
                }
            }
        }
        ExprIr::FunctionValue(_)
        | ExprIr::String(_)
        | ExprIr::Number(_)
        | ExprIr::BigInt(_)
        | ExprIr::Symbol { .. }
        | ExprIr::Boolean(_)
        | ExprIr::Null
        | ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Arguments
        | ExprIr::Identifier(_)
        | ExprIr::NewTarget
        | ExprIr::GlobalPropertyRead { .. }
        | ExprIr::AssignIdentifier { .. }
        | ExprIr::GlobalPropertyWrite { .. }
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::GlobalPropertyUpdate { .. }
        | ExprIr::CompoundAssignIdentifier { .. }
        | ExprIr::GlobalPropertyCompoundAssign { .. }
        | ExprIr::TypeOfUnresolvedIdentifier { .. }
        | ExprIr::SuperPropertyRead { .. }
        | ExprIr::SuperPropertyWrite { .. }
        | ExprIr::PrivateRead { .. }
        | ExprIr::PrivateIn { .. }
        | ExprIr::InstanceOf { .. }
        | ExprIr::CallNamed { .. }
        | ExprIr::RuntimeThrow { .. } => {}
    }
}

fn statement_contains_this_before_super(
    statement: &StatementIr,
    state: &mut DerivedConstructorValidation,
) {
    if state.saw_super {
        return;
    }
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Return(init)
        | StatementIr::Throw(init) => {
            if matches!(statement, StatementIr::Return(_)) && !state.saw_super {
                state.return_before_super = true;
            }
            expr_contains_this_before_super(init, state);
        }
        StatementIr::LexicalBlock(statements) => {
            for statement in statements {
                statement_contains_this_before_super(statement, state);
                if state.saw_super {
                    break;
                }
            }
        }
        StatementIr::Var(decls) => {
            for decl in decls {
                if let Some(init) = &decl.init {
                    expr_contains_this_before_super(init, state);
                }
            }
        }
        StatementIr::Block(block) => {
            for statement in &block.statements {
                statement_contains_this_before_super(statement, state);
                if state.saw_super {
                    break;
                }
            }
        }
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_contains_this_before_super(condition, state);
            statement_contains_this_before_super(then_branch, state);
            if let Some(else_branch) = else_branch {
                statement_contains_this_before_super(else_branch, state);
            }
        }
        StatementIr::While { condition, body } => {
            expr_contains_this_before_super(condition, state);
            statement_contains_this_before_super(body, state);
        }
        StatementIr::DoWhile { body, condition } => {
            statement_contains_this_before_super(body, state);
            expr_contains_this_before_super(condition, state);
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => {
            if let Some(init) = init {
                match init {
                    ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => {
                        expr_contains_this_before_super(init, state);
                    }
                    ForInitIr::LexicalBlock(bindings) => {
                        for binding in bindings {
                            expr_contains_this_before_super(&binding.init, state);
                        }
                    }
                    ForInitIr::Var(decls) => {
                        for decl in decls {
                            if let Some(init) = &decl.init {
                                expr_contains_this_before_super(init, state);
                            }
                        }
                    }
                }
            }
            if let Some(test) = test {
                expr_contains_this_before_super(test, state);
            }
            if let Some(update) = update {
                expr_contains_this_before_super(update, state);
            }
            statement_contains_this_before_super(body, state);
        }
        StatementIr::ForOfArray { iterable, body, .. }
        | StatementIr::ForOfString { iterable, body, .. }
        | StatementIr::ForOfIterator { iterable, body, .. }
        | StatementIr::ForInArray {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInString {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInObject {
            target: iterable,
            body,
            ..
        } => {
            expr_contains_this_before_super(iterable, state);
            statement_contains_this_before_super(body, state);
        }
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            expr_contains_this_before_super(discriminant, state);
            for case in cases {
                if let Some(condition) = &case.condition {
                    expr_contains_this_before_super(condition, state);
                }
                for statement in &case.body.statements {
                    statement_contains_this_before_super(statement, state);
                    if state.saw_super {
                        break;
                    }
                }
            }
        }
        StatementIr::Labelled { statement, .. } => {
            statement_contains_this_before_super(statement, state);
        }
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => {
            for statement in &try_block.statements {
                statement_contains_this_before_super(statement, state);
                if state.saw_super {
                    break;
                }
            }
            if !state.saw_super {
                for statement in &catch_block.statements {
                    statement_contains_this_before_super(statement, state);
                    if state.saw_super {
                        break;
                    }
                }
            }
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => {
            for statement in &try_block.statements {
                statement_contains_this_before_super(statement, state);
                if state.saw_super {
                    break;
                }
            }
            if !state.saw_super {
                for statement in &finally_block.statements {
                    statement_contains_this_before_super(statement, state);
                    if state.saw_super {
                        break;
                    }
                }
            }
        }
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            for statement in &try_block.statements {
                statement_contains_this_before_super(statement, state);
                if state.saw_super {
                    break;
                }
            }
            if !state.saw_super {
                for statement in &catch_block.statements {
                    statement_contains_this_before_super(statement, state);
                    if state.saw_super {
                        break;
                    }
                }
            }
            if !state.saw_super {
                for statement in &finally_block.statements {
                    statement_contains_this_before_super(statement, state);
                    if state.saw_super {
                        break;
                    }
                }
            }
        }
    }
}

pub(crate) fn validate_derived_constructor_body(block: &BlockIr) -> DerivedConstructorValidation {
    let mut state = DerivedConstructorValidation::default();
    for statement in &block.statements {
        statement_contains_this_before_super(statement, &mut state);
    }
    state
}
