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
        ExprIr::EnvironmentIdentifier(identifier) => {
            for operand in identifier.operation.operands() {
                expr_contains_this_before_super(operand, state);
            }
        }
        ExprIr::ModuleEntryEvaluation(entry) => {
            expr_contains_this_before_super(entry.evaluation(), state)
        }
        ExprIr::ModuleExecutionGraph(_)
        | ExprIr::ModuleBindingRead(_)
        | ExprIr::JsonModuleValue(_)
        | ExprIr::ModuleEvaluate(_)
        | ExprIr::DeferredModuleEvaluate(_)
        | ExprIr::ModuleHasAsyncDependencies(_)
        | ExprIr::ModuleDeferredImportEvaluate(_) => {}
        ExprIr::ModuleNamespacePublish { namespace, .. } => {
            expr_contains_this_before_super(namespace, state)
        }
        ExprIr::ImportMeta { .. } => {}
        ExprIr::ModuleNamespace { exports, .. } => expr_contains_this_before_super(exports, state),
        ExprIr::DynamicImport {
            specifier, options, ..
        } => {
            expr_contains_this_before_super(specifier, state);
            if let Some(options) = options {
                expr_contains_this_before_super(options, state);
            }
        }
        ExprIr::CaptureOptionalCallReference(capture) => {
            for operand in capture.operands() {
                expr_contains_this_before_super(operand, state);
            }
        }
        ExprIr::CaptureArgumentList(capture) => {
            for argument in capture.arguments() {
                expr_contains_this_before_super(argument, state);
            }
        }
        ExprIr::CapturedArgumentList(list) => {
            expr_contains_this_before_super(list.binding(), state)
        }
        ExprIr::This => state.this_before_super = true,
        ExprIr::Identifier(name) if name == LEXICAL_THIS_NAME => state.this_before_super = true,
        ExprIr::SuperConstruct { .. } | ExprIr::PreparedSuperConstruct(_) => {
            state.super_calls += 1;
            state.saw_super = true;
        }
        ExprIr::UnaryPlus { expr: operand }
        | ExprIr::UnaryMinusNumeric { expr: operand }
        | ExprIr::UnaryBitwiseNumeric { expr: operand, .. }
        | ExprIr::SpreadArgument(SpreadArgumentIr { value: operand, .. })
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
        ExprIr::OptionalPropertyChain { target, chain } => {
            expr_contains_this_before_super(target, state);
            for operation in chain {
                match operation {
                    OptionalChainOperationIr::Property { key, .. } => match key {
                        PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                            expr_contains_this_before_super(expr, state);
                        }
                        PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
                    },
                    OptionalChainOperationIr::PrivateProperty { .. } => {}
                    OptionalChainOperationIr::Call { args, receiver, .. } => {
                        if *receiver == OptionalChainCallReceiverIr::CurrentThis {
                            state.this_before_super = true;
                        }
                        for arg in args {
                            expr_contains_this_before_super(arg, state);
                        }
                    }
                }
            }
        }
        ExprIr::DeleteOptionalPropertyChain(deletion) => {
            let target = deletion.target();
            let chain = deletion.prefix();
            expr_contains_this_before_super(target, state);
            for operation in chain {
                match operation {
                    OptionalChainOperationIr::Property { key, .. } => match key {
                        PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                            expr_contains_this_before_super(expr, state);
                        }
                        PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
                    },
                    OptionalChainOperationIr::PrivateProperty { .. } => {}
                    OptionalChainOperationIr::Call { args, receiver, .. } => {
                        if *receiver == OptionalChainCallReceiverIr::CurrentThis {
                            state.this_before_super = true;
                        }
                        for arg in args {
                            expr_contains_this_before_super(arg, state);
                        }
                    }
                }
            }

            match deletion.key() {
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    expr_contains_this_before_super(expr, state);
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
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
        | ExprIr::Comma { lhs, rhs } => {
            expr_contains_this_before_super(lhs, state);
            expr_contains_this_before_super(rhs, state);
        }
        ExprIr::MaterializeBinding { value, body, .. } => {
            expr_contains_this_before_super(value, state);
            expr_contains_this_before_super(body, state);
        }
        ExprIr::ArrayDestructure { value, pattern, .. } => {
            expr_contains_this_before_super(value, state);
            pattern.visit_expressions(&mut |expr| expr_contains_this_before_super(expr, state));
        }
        ExprIr::ObjectDestructure { value, pattern } => {
            expr_contains_this_before_super(value, state);
            pattern.visit_expressions(&mut |expr| expr_contains_this_before_super(expr, state));
        }
        ExprIr::ObjectDestructuringOperation(operation) => {
            operation.visit_expressions(&mut |expr| expr_contains_this_before_super(expr, state));
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
            ..
        } => {
            expr_contains_this_before_super(callee, state);
            if let Some(this_arg) = this_arg {
                expr_contains_this_before_super(this_arg, state);
            }
            for arg in args {
                expr_contains_this_before_super(arg, state);
            }
        }
        ExprIr::Construct { callee, args, .. } => {
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
        ExprIr::OrdinaryPropertyAssignment(assignment) => {
            expr_contains_this_before_super(assignment.base_and_receiver(), state);
            match assignment.referenced_name() {
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    expr_contains_this_before_super(expr, state);
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
            expr_contains_this_before_super(assignment.rhs(), state);
        }
        ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {
            expr_contains_this_before_super(assignment.base_and_receiver(), state);
            match assignment.referenced_name() {
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    expr_contains_this_before_super(expr, state);
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
            expr_contains_this_before_super(assignment.rhs(), state);
        }
        ExprIr::OrdinaryPropertyGetCapture(capture) => {
            expr_contains_this_before_super(capture.base_and_receiver(), state);
            match capture.referenced_name() {
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    expr_contains_this_before_super(expr, state)
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
        }
        ExprIr::CapturedOrdinaryPropertyWrite(write) => {
            expr_contains_this_before_super(write.rhs(), state)
        }
        ExprIr::OrdinaryPropertyNumericUpdate(update) => {
            expr_contains_this_before_super(update.base_and_receiver(), state);
            match update.referenced_name() {
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    expr_contains_this_before_super(expr, state);
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
        }
        ExprIr::OrdinaryPropertyEagerCompoundAssignment(assignment) => {
            expr_contains_this_before_super(assignment.base_and_receiver(), state);
            match assignment.referenced_name() {
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    expr_contains_this_before_super(expr, state);
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
            expr_contains_this_before_super(assignment.result(), state);
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
        ExprIr::ArrayAccumulation(accumulation) => {
            for element in accumulation.elements() {
                match element {
                    ArrayAccumulationElementIr::Elision => {}
                    ArrayAccumulationElementIr::Value(value) => {
                        expr_contains_this_before_super(value, state)
                    }
                    ArrayAccumulationElementIr::Spread(spread) => {
                        expr_contains_this_before_super(&spread.value, state)
                    }
                }
            }
        }
        ExprIr::ObjectLiteral(properties) => {
            for property in properties {
                object_property_contains_this_before_super(property, state);
            }
        }
        ExprIr::ObjectPropertyDefinition(definition) => {
            expr_contains_this_before_super(definition.target(), state);
            object_property_contains_this_before_super(definition.property(), state);
        }
        ExprIr::ClassDefinition(class) => {
            if let Some(heritage) = &class.heritage {
                expr_contains_this_before_super(heritage, state);
            }
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
                match key {
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_contains_this_before_super(expr, state);
                    }
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
                }
            }
        }
        ExprIr::FunctionValue(_)
        | ExprIr::WellKnownSymbol(_)
        | ExprIr::String(_)
        | ExprIr::TemplateObject(_)
        | ExprIr::RegExpLiteral { .. }
        | ExprIr::Number(_)
        | ExprIr::BigInt(_)
        | ExprIr::Symbol { .. }
        | ExprIr::Boolean(_)
        | ExprIr::Null
        | ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Arguments
        | ExprIr::ExecutionGlobalObject
        | ExprIr::Identifier(_)
        | ExprIr::NewTarget
        | ExprIr::SuperNewTarget
        | ExprIr::SuperConstructor
        | ExprIr::GlobalPropertyRead { .. }
        | ExprIr::GlobalIdentifierRead { .. }
        | ExprIr::AssignIdentifier { .. }
        | ExprIr::GlobalPropertyWrite { .. }
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::CompoundAssignIdentifier { .. }
        | ExprIr::TypeOfUnresolvedIdentifier { .. }
        | ExprIr::PrivateRead { .. }
        | ExprIr::PrivateIn { .. }
        | ExprIr::InstanceOf { .. }
        | ExprIr::CallNamed { .. }
        | ExprIr::RuntimeThrow { .. } => {}
        ExprIr::SuperPropertyRead { receiver, .. } => {
            expr_contains_this_before_super(receiver, state);
        }
        ExprIr::SuperPropertyWrite {
            receiver, value, ..
        } => {
            expr_contains_this_before_super(receiver, state);
            expr_contains_this_before_super(value, state);
        }
        ExprIr::SuperPropertyMutation(mutation) => {
            expr_contains_this_before_super(mutation.receiver(), state);
            match mutation.referenced_name() {
                PropertyKeyIr::StringExpr(key) | PropertyKeyIr::ArrayIndex(key) => {
                    expr_contains_this_before_super(key, state);
                }
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
            }
            match mutation.operation() {
                SuperPropertyMutationOperationIr::NumericUpdate { .. }
                | SuperPropertyMutationOperationIr::Capture(_) => {}
                SuperPropertyMutationOperationIr::EagerCompound { result, .. }
                | SuperPropertyMutationOperationIr::PutCaptured { value: result, .. } => {
                    expr_contains_this_before_super(result, state);
                }
            }
        }
    }
}

fn object_property_contains_this_before_super(
    property: &ObjectPropertyIr,
    state: &mut DerivedConstructorValidation,
) {
    match property {
        ObjectPropertyIr::PrototypeSetter { value }
        | ObjectPropertyIr::Spread { source: value }
        | ObjectPropertyIr::Data { value, .. }
        | ObjectPropertyIr::NonEnumerableData { value, .. } => {
            expr_contains_this_before_super(value, state);
        }
        ObjectPropertyIr::ComputedData { key, value, .. } => {
            expr_contains_this_before_super(key, state);
            expr_contains_this_before_super(value, state);
        }
        ObjectPropertyIr::ComputedMethod { key, .. }
        | ObjectPropertyIr::ComputedGetter { key, .. }
        | ObjectPropertyIr::ComputedSetter { key, .. } => {
            expr_contains_this_before_super(key, state);
        }
        ObjectPropertyIr::Method { .. }
        | ObjectPropertyIr::Getter { .. }
        | ObjectPropertyIr::Setter { .. } => {}
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
        StatementIr::ResumableClassDefinition(plan) => {
            for statement in plan.prefixes().flat_map(|prefix| prefix.statements()) {
                statement_contains_this_before_super(statement, state);
            }
            expr_contains_this_before_super(plan.expression(), state);
        }
        StatementIr::ModuleUnitOnce { block, .. } => {
            for statement in &block.statements {
                statement_contains_this_before_super(statement, state);
            }
        }
        StatementIr::ModuleImportBinding(_) => {}
        StatementIr::AsyncModuleInstantiation
        | StatementIr::Empty
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
        StatementIr::Lexical { init, .. }
        | StatementIr::DeclarationEvaluation(init)
        | StatementIr::Expression(init)
        | StatementIr::Return(init)
        | StatementIr::Throw(init) => {
            if matches!(statement, StatementIr::Return(_)) && !state.saw_super {
                state.return_before_super = true;
            }
            expr_contains_this_before_super(init, state);
        }
        StatementIr::GeneratorYield {
            value, resume_mode, ..
        } => {
            if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
                match reference.use_view() {
                    SuspendedPropertyReferenceUse::Ordinary {
                        base_and_receiver,
                        key,
                        strictness: _,
                    } => {
                        expr_contains_this_before_super(base_and_receiver, state);
                        if let PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) =
                            key
                        {
                            expr_contains_this_before_super(expr, state);
                        }
                    }
                }
            }
            expr_contains_this_before_super(value, state);
        }
        StatementIr::AsyncAwait { value, .. } => {
            expr_contains_this_before_super(value, state);
        }
        StatementIr::EmptyStatementCompletion(item) => {
            statement_contains_this_before_super(item.statement(), state);
        }
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => {
            for statement in statements {
                statement_contains_this_before_super(statement, state);
                if state.saw_super {
                    break;
                }
            }
        }
        StatementIr::SyncDisposableScope {
            resources, body, ..
        } => {
            for resource in resources.iter() {
                expr_contains_this_before_super(&resource.initializer, state);
                if state.saw_super {
                    return;
                }
            }
            for statement in &body.statements {
                statement_contains_this_before_super(statement, state);
                if state.saw_super {
                    break;
                }
            }
        }
        StatementIr::AsyncDisposableScope {
            resources, body, ..
        } => {
            for resource in resources.iter() {
                expr_contains_this_before_super(resource.initializer(), state);
                if state.saw_super {
                    return;
                }
            }
            for statement in &body.statements {
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
        }
        | StatementIr::AsyncFunctionIf {
            condition,
            then_branch,
            else_branch,
            plan: _,
        } => {
            expr_contains_this_before_super(condition, state);
            statement_contains_this_before_super(then_branch, state);
            if let Some(else_branch) = else_branch {
                statement_contains_this_before_super(else_branch, state);
            }
        }
        StatementIr::AsyncFunctionWhile(plan) => {
            for statement in plan.condition_prefix() {
                statement_contains_this_before_super(statement, state);
            }
            expr_contains_this_before_super(plan.condition(), state);
            statement_contains_this_before_super(plan.body(), state);
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
            ..
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
                    ForInitIr::Statements(statements) => {
                        for statement in statements {
                            statement_contains_this_before_super(statement, state);
                        }
                    }
                    ForInitIr::SyncDisposable(resources) => {
                        for resource in resources.iter() {
                            expr_contains_this_before_super(&resource.initializer, state);
                        }
                    }
                    ForInitIr::AsyncDisposable(init) => {
                        for resource in init.resources().iter() {
                            expr_contains_this_before_super(resource.initializer(), state);
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
        StatementIr::OrdinaryGeneratorLoop(plan) => {
            for region in plan.regions() {
                for statement in &region.block().statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
            for expression in plan.expressions() {
                expr_contains_this_before_super(expression, state);
            }
        }
        StatementIr::AsyncGeneratorLoop(plan) => {
            for region in plan.regions() {
                for statement in &region.block().statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
            for expression in plan.expressions() {
                expr_contains_this_before_super(expression, state);
            }
        }
        StatementIr::OrdinaryGeneratorIf(plan) => {
            expr_contains_this_before_super(plan.condition(), state);
            for region in [plan.then_branch(), plan.else_branch()] {
                for statement in &region.block().statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
        }
        StatementIr::AsyncGeneratorIf(plan) => {
            expr_contains_this_before_super(plan.condition().value(), state);
            for region in [
                plan.condition().region(),
                plan.then_branch(),
                plan.else_branch(),
            ] {
                for statement in &region.block().statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
        }
        StatementIr::OrdinaryGeneratorSwitch(plan) => {
            for statement in &plan.discriminant().region().block().statements {
                statement_contains_this_before_super(statement, state);
            }
            expr_contains_this_before_super(plan.discriminant().value(), state);
            for declaration in plan.lexical_declarations() {
                statement_contains_this_before_super(declaration, state);
            }
            for case in plan.cases() {
                if let Some(selector) = case.selector() {
                    for statement in &selector.region().block().statements {
                        statement_contains_this_before_super(statement, state);
                    }
                    expr_contains_this_before_super(selector.value(), state);
                }
                for statement in &case.body().block().statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
        }
        StatementIr::AsyncGeneratorSwitch(plan) => {
            for statement in &plan.discriminant().region().block().statements {
                statement_contains_this_before_super(statement, state);
            }
            expr_contains_this_before_super(plan.discriminant().value(), state);
            for declaration in plan.lexical_declarations() {
                statement_contains_this_before_super(declaration, state);
            }
            for case in plan.cases() {
                if let Some(selector) = case.selector() {
                    for statement in &selector.region().block().statements {
                        statement_contains_this_before_super(statement, state);
                    }
                    expr_contains_this_before_super(selector.value(), state);
                }
                for statement in &case.body().block().statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
        }
        StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
            expr_contains_this_before_super(plan.raw_source(), state);
            for statement in &plan.body().block().statements {
                statement_contains_this_before_super(statement, state);
            }
        }
        StatementIr::AsyncGeneratorResourceScope(plan) => {
            for statement in &plan.body().block().statements {
                statement_contains_this_before_super(statement, state);
            }
        }
        StatementIr::AsyncGeneratorResourceRegistration(operation) => {
            expr_contains_this_before_super(operation.initializer(), state)
        }
        StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
            expr_contains_this_before_super(plan.raw_source(), state);
            for statement in &plan.body().block().statements {
                statement_contains_this_before_super(statement, state);
            }
        }
        StatementIr::AsyncFunctionArrayDestructuring(plan) => {
            expr_contains_this_before_super(plan.raw_source(), state);
            for statement in &plan.body().statements {
                statement_contains_this_before_super(statement, state);
            }
        }
        StatementIr::ArrayDestructuringOperation(_) => {}
        StatementIr::AsyncGeneratorForIn(plan) => {
            for block in [
                plan.head().region().block(),
                plan.initialization(),
                plan.body().block(),
            ] {
                for statement in &block.statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
            expr_contains_this_before_super(plan.head().value(), state);
        }
        StatementIr::AsyncGeneratorForOf(plan) => {
            for block in [
                plan.head().region().block(),
                plan.initialization().block(),
                plan.body().block(),
            ] {
                for statement in &block.statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
            expr_contains_this_before_super(plan.head().value(), state);
        }
        StatementIr::OrdinaryGeneratorWith(plan) => {
            for region in [plan.head().region(), plan.body()] {
                for statement in &region.block().statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
            expr_contains_this_before_super(plan.head().value(), state);
        }
        StatementIr::AsyncGeneratorWith(plan) => {
            for region in plan.regions() {
                for statement in &region.block().statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
            expr_contains_this_before_super(plan.head().value(), state);
        }
        StatementIr::AsyncFunctionWith(plan) => {
            for block in [plan.head(), plan.body()] {
                for statement in &block.statements {
                    statement_contains_this_before_super(statement, state);
                }
            }
            expr_contains_this_before_super(plan.head_value(), state);
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
                    ForInitIr::Statements(statements) => {
                        for statement in statements {
                            statement_contains_this_before_super(statement, state);
                        }
                    }
                    ForInitIr::SyncDisposable(resources) => {
                        for resource in resources.iter() {
                            expr_contains_this_before_super(&resource.initializer, state);
                        }
                    }
                    ForInitIr::AsyncDisposable(init) => {
                        for resource in init.resources().iter() {
                            expr_contains_this_before_super(resource.initializer(), state);
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
            for statement in before_suspension {
                statement_contains_this_before_super(statement, state);
            }
            statement_contains_this_before_super(suspension_statement, state);
            for statement in after_suspension {
                statement_contains_this_before_super(statement, state);
            }
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
            expr_contains_this_before_super(condition, state);
            for statement in then_before_yield
                .iter()
                .chain(then_yield_statement.as_deref())
                .chain(then_after_yield)
                .chain(else_before_yield)
                .chain(else_yield_statement.as_deref())
                .chain(else_after_yield)
            {
                statement_contains_this_before_super(statement, state);
            }
        }
        StatementIr::AsyncFunctionForOfIterator { iterable, plan } => {
            expr_contains_this_before_super(iterable, state);
            for statement in plan.body().statements() {
                if state.saw_super {
                    break;
                }
                statement_contains_this_before_super(statement, state);
            }
        }
        StatementIr::GeneratorForOfIterator { iterable, plan } => {
            expr_contains_this_before_super(iterable, state);
            for statement in plan.body().statements() {
                if state.saw_super {
                    break;
                }
                statement_contains_this_before_super(statement, state);
            }
        }
        StatementIr::ForOfIterator { iterable, body, .. }
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
        StatementIr::AsyncFunctionSwitch(plan) => {
            expr_contains_this_before_super(plan.discriminant(), state);
            for declaration in plan.lexical_declarations() {
                statement_contains_this_before_super(declaration, state);
            }
            for case in plan.cases() {
                for statement in case.condition_prefix() {
                    statement_contains_this_before_super(statement, state);
                    if state.saw_super {
                        break;
                    }
                }
                if let Some(condition) = case.condition() {
                    expr_contains_this_before_super(condition, state);
                }
                for statement in &case.body().statements {
                    statement_contains_this_before_super(statement, state);
                    if state.saw_super {
                        break;
                    }
                }
            }
        }
        StatementIr::Switch {
            discriminant,
            lexical_declarations,
            cases,
            ..
        } => {
            expr_contains_this_before_super(discriminant, state);
            for declaration in lexical_declarations {
                statement_contains_this_before_super(declaration, state);
            }
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
            ..
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
