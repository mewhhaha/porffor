fn lower_script(source: &str) -> ProgramIr {
    let source = parse(source, ParseOptions::script()).expect("script should parse");
    lower(&source)
}
fn lower_module(source: &str) -> ProgramIr {
    let source = parse(source, ParseOptions::module()).expect("module should parse");
    lower(&source)
}

fn lower_test262_script(source: &str) -> ProgramIr {
    let source = parse(source, ParseOptions::script()).expect("script should parse");
    lower_with_host_surface_policy(&source, HostSurfacePolicy::Test262)
}

fn assert_prepared_script(program: &ProgramIr, kind: PreparedScriptKind) {
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let prepared = &program.script.as_ref().expect("Script IR").prepared_scripts;
    assert!(
        prepared.iter().any(|script| script.kind == kind
            && script.admission == PreparedScriptAdmission::ResolvedIntrinsic
            && matches!(script.outcome, PreparedScriptOutcome::Executable(_))),
        "{prepared:?}"
    );
}

fn assert_zero_suspension_generator(function: &FunctionIr) {
    assert_eq!(
        function.protocol.execution_kind(),
        FunctionExecutionKind::Generator
    );
    assert!(!function.protocol.is_constructable());
    assert_eq!(
        function.generator_plan,
        Some(GeneratorPlanIr::without_suspensions())
    );
}

fn indirect_call_body(expression: &TypedExpr) -> Option<&TypedExpr> {
    match &expression.expr {
        ExprIr::CallIndirect { .. } => Some(expression),
        ExprIr::MaterializeBinding { body, .. }
            if matches!(body.expr, ExprIr::CallIndirect { .. }) =>
        {
            Some(body)
        }
        _ => None,
    }
}

fn function_return(function: &FunctionIr) -> Option<&TypedExpr> {
    function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(value) => Some(value),
            _ => None,
        })
}

fn assert_caller_flow_invalidation_reaches_final_addition(source: &str) {
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("expected the final addition: {source}");
    };
    assert!(
        matches!(result.expr, ExprIr::CoerciveAdd { .. }),
        "{source}: {result:?}"
    );
}

fn assert_caller_flow_preservation_reaches_final_addition(source: &str) {
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("expected the final addition: {source}");
    };
    assert!(
        matches!(
            result.expr,
            ExprIr::BinaryNumber {
                op: ArithmeticBinaryOp::Add,
                ..
            } | ExprIr::CoerciveBinaryNumber {
                op: ArithmeticBinaryOp::Add,
                ..
            }
        ),
        "{source}: {result:?}"
    );
}

fn collect_annex_b_copies(block: &BlockIr) -> Vec<(String, String, AnnexBFunctionCopyTargetIr)> {
    fn collect(
        statement: &StatementIr,
        copies: &mut Vec<(String, String, AnnexBFunctionCopyTargetIr)>,
    ) {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => collect(item.statement(), copies),
            StatementIr::AsyncGeneratorSwitch(plan) => {
                for statement in plan.lexical_declarations() { collect(statement, copies); }
                for region in plan.regions() {
                    for statement in &region.block().statements { collect(statement, copies); }
                }
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                for region in [plan.head().region(), plan.initialization(), plan.body()] {
                    for statement in &region.block().statements { collect(statement, copies); }
                }
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                for statement in &plan.body().block().statements { collect(statement, copies); }
            }
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                for statement in &plan.body().block().statements {
                    collect(statement, copies);
                }
            }
            StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
                for statement in &plan.body().block().statements {
                    collect(statement, copies);
                }
            }
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                for statement in &plan.body().statements {
                    collect(statement, copies);
                }
            }
            StatementIr::OrdinaryGeneratorWith(plan) => {
                for region in [plan.head().region(), plan.body()] {
                    for statement in &region.block().statements {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::AsyncFunctionWith(plan) => {
                for block in [plan.head(), plan.body()] {
                    for statement in &block.statements {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                for statement in plan.lexical_declarations() {
                    collect(statement, copies);
                }
                for region in plan.regions() {
                    for statement in &region.block().statements {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::ResumableClassDefinition(plan) => {
                for statement in plan.prefixes().flat_map(|prefix| prefix.statements()) {
                    collect(statement, copies);
                }
            }
            StatementIr::AnnexBFunctionCopy {
                source_name,
                block_storage_name,
                target,
                ..
            } => copies.push((
                source_name.clone(),
                block_storage_name.clone(),
                target.clone(),
            )),
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => {
                for statement in statements {
                    collect(statement, copies);
                }
            }
            StatementIr::SyncDisposableScope { body, .. }
            | StatementIr::AsyncDisposableScope { body, .. } => {
                for statement in &body.statements {
                    collect(statement, copies);
                }
            }
            StatementIr::Block(block) => {
                for statement in &block.statements {
                    collect(statement, copies);
                }
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() {
                    for statement in &region.block().statements {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    for statement in &region.block().statements {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                for region in [plan.then_branch(), plan.else_branch()] {
                    for statement in &region.block().statements {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                for region in [
                    plan.condition().region(),
                    plan.then_branch(),
                    plan.else_branch(),
                ] {
                    for statement in &region.block().statements {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            }
            | StatementIr::AsyncFunctionIf {
                condition: _,
                then_branch,
                else_branch,
                plan: _,
            } => {
                collect(then_branch, copies);
                if let Some(else_branch) = else_branch {
                    collect(else_branch, copies);
                }
            }
            StatementIr::AsyncFunctionWhile(plan) => {
                for statement in plan.condition_prefix() {
                    collect(statement, copies);
                }
                collect(plan.body(), copies);
            }
            StatementIr::While { body, .. }
            | StatementIr::DoWhile { body, .. }
            | StatementIr::For { body, .. }
            | StatementIr::ForOfIterator { body, .. }
            | StatementIr::ForInArray { body, .. }
            | StatementIr::ForInString { body, .. }
            | StatementIr::ForInObject { body, .. }
            | StatementIr::Labelled {
                statement: body, ..
            } => collect(body, copies),
            StatementIr::AsyncFunctionForOfIterator { plan, .. } => {
                for statement in plan.body().statements() {
                    collect(statement, copies);
                }
            }
            StatementIr::GeneratorForOfIterator { plan, .. } => {
                for statement in plan.body().statements() {
                    collect(statement, copies);
                }
            }
            StatementIr::AsyncFunctionSwitch(plan) => {
                for declaration in plan.lexical_declarations() {
                    collect(declaration, copies);
                }
                for case in plan.cases() {
                    for statement in case
                        .condition_prefix()
                        .iter()
                        .chain(&case.body().statements)
                    {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::Switch {
                lexical_declarations,
                cases,
                ..
            } => {
                for declaration in lexical_declarations {
                    collect(declaration, copies);
                }
                for case in cases {
                    for statement in &case.body.statements {
                        collect(statement, copies);
                    }
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                for statement in &try_block.statements {
                    collect(statement, copies);
                }
                for statement in &catch_block.statements {
                    collect(statement, copies);
                }
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                for statement in &try_block.statements {
                    collect(statement, copies);
                }
                for statement in &finally_block.statements {
                    collect(statement, copies);
                }
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                for block in [try_block, catch_block, finally_block] {
                    for statement in &block.statements {
                        collect(statement, copies);
                    }
                }
            }
            _ => {}
        }
    }

    let mut copies = Vec::new();
    for statement in &block.statements {
        collect(statement, &mut copies);
    }
    copies
}

fn collect_binding_storage_names(block: &BlockIr) -> BTreeSet<String> {
    fn collect(statement: &StatementIr, names: &mut BTreeSet<String>) {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => collect(item.statement(), names),
            StatementIr::AsyncGeneratorWith(plan) => {
                names.insert(plan.head_binding().name.clone());
                names.insert(plan.object_binding().name.clone());
                for region in [plan.head().region(), plan.body()] {
                    for statement in &region.block().statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                for binding in [
                    plan.head_binding(),
                    plan.enumerator_binding(),
                    plan.key_binding(),
                    plan.value_binding(),
                ] {
                    names.insert(binding.name.clone());
                }
                for region in [plan.head().region(), plan.initialization_region(), plan.body()] {
                    for statement in &region.block().statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::ArrayDestructuringOperation(operation) => {
                names.insert(operation.storage().binding().name.clone());
                if let Some(result) = operation.result_binding() {
                    names.insert(result.name.clone());
                }
            }
            StatementIr::AsyncGeneratorSwitch(plan) => {
                names.insert(plan.discriminant_binding().name.clone());
                names.insert(plan.value_binding().name.clone());
                if let Some(resource) = plan.resource() { names.insert(resource.capability_binding().name.clone()); }
                for statement in plan.lexical_declarations() { collect(statement, names); }
                for region in plan.regions() {
                    for statement in &region.block().statements { collect(statement, names); }
                }
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                if let Some(resource) = plan.resource() { names.insert(resource.capability_binding().name.clone()); }
                for binding in [plan.head_binding(), plan.incoming_binding(), plan.value_binding()] {
                    names.insert(binding.name.clone());
                }
                for region in [plan.head().region(), plan.initialization(), plan.body()] {
                    for statement in &region.block().statements { collect(statement, names); }
                }
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                names.insert(plan.capability_binding().name.clone());
                for statement in &plan.body().block().statements { collect(statement, names); }
            }
            StatementIr::AsyncGeneratorResourceRegistration(operation) => {
                names.insert(operation.binding_name().to_string());
            }
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                names.insert(plan.storage().binding().name.clone());
                for statement in &plan.body().block().statements {
                    collect(statement, names);
                }
            }
            StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
                names.insert(plan.storage().binding().name.clone());
                for statement in &plan.body().block().statements {
                    collect(statement, names);
                }
            }
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                names.insert(plan.storage().binding().name.clone());
                for statement in &plan.body().statements {
                    collect(statement, names);
                }
            }
            StatementIr::OrdinaryGeneratorWith(plan) => {
                names.insert(plan.head_binding().name.clone());
                names.insert(plan.object_binding().name.clone());
                for region in [plan.head().region(), plan.body()] {
                    for statement in &region.block().statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::AsyncFunctionWith(plan) => {
                names.insert(plan.head_binding().name.clone());
                names.insert(plan.object_binding().name.clone());
                for block in [plan.head(), plan.body()] {
                    for statement in &block.statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                names.insert(plan.discriminant_binding().name.clone());
                names.insert(plan.value_binding().name.clone());
                for statement in plan.lexical_declarations() {
                    collect(statement, names);
                }
                for region in plan.regions() {
                    for statement in &region.block().statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::ResumableClassDefinition(plan) => {
                names.insert(plan.constructor_binding().to_string());
                names.insert(plan.completion_binding().to_string());
                names.extend(plan.name_environment_binding().map(str::to_string));
                if let Some(binding) = &plan.class().name_binding {
                    names.insert(binding.storage_name.clone());
                }
                for statement in plan.prefixes().flat_map(|prefix| prefix.statements()) {
                    collect(statement, names);
                }
            }
            StatementIr::ModuleUnitOnce { block, .. } => {
                for statement in &block.statements {
                    collect(statement, names);
                }
            }
            StatementIr::ModuleImportBinding(import) => {
                names.insert(import.name.clone());
            }
            StatementIr::Lexical { name, .. } => {
                names.insert(name.clone());
            }
            StatementIr::Var(declarators) => {
                names.extend(declarators.iter().map(|declarator| declarator.name.clone()));
            }
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => {
                for statement in statements {
                    collect(statement, names);
                }
            }
            StatementIr::Block(block) => {
                for statement in &block.statements {
                    collect(statement, names);
                }
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            }
            | StatementIr::AsyncFunctionIf {
                condition: _,
                then_branch,
                else_branch,
                plan: _,
            } => {
                collect(then_branch, names);
                if let Some(else_branch) = else_branch {
                    collect(else_branch, names);
                }
            }
            StatementIr::AsyncFunctionWhile(plan) => {
                for statement in plan.condition_prefix() {
                    collect(statement, names);
                }
                collect(plan.body(), names);
            }
            StatementIr::While { body, .. }
            | StatementIr::DoWhile { body, .. }
            | StatementIr::Labelled {
                statement: body, ..
            } => collect(body, names),
            StatementIr::For { init, body, .. } => {
                if let Some(init) = init {
                    match init {
                        ForInitIr::Lexical { name, .. } => {
                            names.insert(name.clone());
                        }
                        ForInitIr::LexicalBlock(bindings) => {
                            names.extend(bindings.iter().map(|binding| binding.name.clone()));
                        }
                        ForInitIr::Var(declarators) => {
                            names.extend(
                                declarators.iter().map(|declarator| declarator.name.clone()),
                            );
                        }
                        ForInitIr::Expression(_) => {}
                        ForInitIr::Statements(statements) => {
                            for statement in statements {
                                collect(statement, names);
                            }
                        }
                        ForInitIr::SyncDisposable(resources) => {
                            names.extend(
                                resources
                                    .iter()
                                    .map(|resource| resource.binding_name.clone()),
                            );
                        }
                        ForInitIr::AsyncDisposable(init) => {
                            names.extend(
                                init.resources()
                                    .iter()
                                    .map(|resource| resource.binding_name().to_string()),
                            );
                        }
                    }
                }
                collect(body, names);
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                names.insert(plan.value_binding_name().to_string());
                for region in plan.regions() {
                    for statement in &region.block().statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                names.insert(plan.value_binding_name().to_string());
                if let Some(resource) = plan.resource() { names.insert(resource.capability_binding().name.clone()); }
                for region in plan.regions() {
                    for statement in &region.block().statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                for region in [plan.then_branch(), plan.else_branch()] {
                    for statement in &region.block().statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                for region in [
                    plan.condition().region(),
                    plan.then_branch(),
                    plan.else_branch(),
                ] {
                    for statement in &region.block().statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::GeneratorLoop {
                init,
                before_suspension,
                suspension_statement,
                after_suspension,
                ..
            } => {
                if let Some(init) = init {
                    match init {
                        ForInitIr::Lexical { name, .. } => {
                            names.insert(name.clone());
                        }
                        ForInitIr::LexicalBlock(bindings) => {
                            names.extend(bindings.iter().map(|binding| binding.name.clone()));
                        }
                        ForInitIr::Var(declarators) => {
                            names.extend(
                                declarators.iter().map(|declarator| declarator.name.clone()),
                            );
                        }
                        ForInitIr::Expression(_) => {}
                        ForInitIr::Statements(statements) => {
                            for statement in statements {
                                collect(statement, names);
                            }
                        }
                        ForInitIr::SyncDisposable(resources) => {
                            names.extend(
                                resources
                                    .iter()
                                    .map(|resource| resource.binding_name.clone()),
                            );
                        }
                        ForInitIr::AsyncDisposable(init) => {
                            names.extend(
                                init.resources()
                                    .iter()
                                    .map(|resource| resource.binding_name().to_string()),
                            );
                        }
                    }
                }
                for statement in before_suspension
                    .iter()
                    .chain(std::iter::once(suspension_statement.as_ref()))
                    .chain(after_suspension)
                {
                    collect(statement, names);
                }
            }
            StatementIr::GeneratorIf {
                then_before_yield,
                then_yield_statement,
                then_after_yield,
                else_before_yield,
                else_yield_statement,
                else_after_yield,
                ..
            } => {
                for statement in then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                {
                    collect(statement, names);
                }
            }
            StatementIr::SyncDisposableScope {
                resources, body, ..
            } => {
                names.extend(
                    resources
                        .iter()
                        .map(|resource| resource.binding_name.clone()),
                );
                for statement in &body.statements {
                    collect(statement, names);
                }
            }
            StatementIr::AsyncDisposableScope {
                resources, body, ..
            } => {
                names.extend(
                    resources
                        .iter()
                        .map(|resource| resource.binding_name().to_string()),
                );
                for statement in &body.statements {
                    collect(statement, names);
                }
            }
            StatementIr::ForOfIterator { head, body, .. } => {
                let name = match head {
                    ForOfIteratorHeadIr::Assignment { binding, .. } => &binding.name,
                    ForOfIteratorHeadIr::SyncDisposable(head) => head.binding_name(),
                    ForOfIteratorHeadIr::AsyncDisposable(head) => head.binding_name(),
                };
                names.insert(name.to_string());
                collect(body, names);
            }
            StatementIr::AsyncFunctionForOfIterator { plan, .. } => {
                names.insert(plan.value_name().to_string());
                for statement in plan.body().statements() {
                    collect(statement, names);
                }
            }
            StatementIr::GeneratorForOfIterator { plan, .. } => {
                names.insert(plan.value_name().to_string());
                for statement in plan.body().statements() {
                    collect(statement, names);
                }
            }
            StatementIr::ForInArray { name, body, .. }
            | StatementIr::ForInString { name, body, .. }
            | StatementIr::ForInObject { name, body, .. } => {
                names.insert(name.clone());
                collect(body, names);
            }
            StatementIr::AsyncFunctionSwitch(plan) => {
                for declaration in plan.lexical_declarations() {
                    collect(declaration, names);
                }
                for case in plan.cases() {
                    for statement in case
                        .condition_prefix()
                        .iter()
                        .chain(&case.body().statements)
                    {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::Switch {
                lexical_declarations,
                cases,
                ..
            } => {
                for declaration in lexical_declarations {
                    collect(declaration, names);
                }
                for case in cases {
                    for statement in &case.body.statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::TryCatch {
                catch_name,
                try_block,
                catch_block,
                ..
            } => {
                names.insert(catch_name.clone());
                for block in [try_block, catch_block] {
                    for statement in &block.statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                for block in [try_block, finally_block] {
                    for statement in &block.statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::TryCatchFinally {
                catch_name,
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                names.insert(catch_name.clone());
                for block in [try_block, catch_block, finally_block] {
                    for statement in &block.statements {
                        collect(statement, names);
                    }
                }
            }
            StatementIr::DeclarationEvaluation(TypedExpr {
                expr:
                    ExprIr::ArrayDestructure {
                        pattern,
                        evaluation,
                        ..
                    },
                ..
            }) => match *evaluation {
                ArrayDestructuringEvaluationIr::BindingInitialization => {
                    pattern.visit_bindings(&mut |_, name| {
                        names.insert(name.to_string());
                    });
                }
                ArrayDestructuringEvaluationIr::AssignmentEvaluation => {}
            },
            StatementIr::DeclarationEvaluation(TypedExpr {
                expr: ExprIr::ObjectDestructure { pattern, .. },
                ..
            }) => {
                pattern.visit_bindings(&mut |_, name| {
                    names.insert(name.to_string());
                });
            }
            StatementIr::AsyncModuleInstantiation
            | StatementIr::Empty
            | StatementIr::AnnexBFunctionCopy { .. }
            | StatementIr::DeclarationEvaluation(_)
            | StatementIr::Expression(_)
            | StatementIr::GeneratorYield { .. }
            | StatementIr::AsyncAwait { .. }
            | StatementIr::Debugger
            | StatementIr::Throw(_)
            | StatementIr::Return(_)
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. } => {}
        }
    }

    let mut names = BTreeSet::new();
    for statement in &block.statements {
        collect(statement, &mut names);
    }
    names
}

fn block_environment_owns_binding(block: &BlockIr, name: &str, slot: u32) -> bool {
    fn lexical_environment_owns_binding(
        environment: Option<&LexicalEnvironmentIr>,
        name: &str,
        slot: u32,
    ) -> bool {
        environment.as_ref().is_some_and(|environment| {
            environment
                .bindings
                .iter()
                .any(|binding| binding.name == name && binding.slot == slot)
        })
    }

    if lexical_environment_owns_binding(block.lexical_environment.as_ref(), name, slot) {
        return true;
    }

    fn statement_owns_binding(statement: &StatementIr, name: &str, slot: u32) -> bool {
        match statement {
            StatementIr::AsyncGeneratorSwitch(plan) => {
                lexical_environment_owns_binding(plan.lexical_environment(), name, slot)
                    || plan.lexical_declarations().iter().any(|statement| statement_owns_binding(statement, name, slot))
                    || plan.regions().any(|region| block_environment_owns_binding(region.block(), name, slot))
            }
            StatementIr::EmptyStatementCompletion(item) => {
                statement_owns_binding(item.statement(), name, slot)
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                plan.lexical_environment().is_some_and(|environment| {
                    lexical_environment_owns_binding(environment.tdz_environment.as_ref(), name, slot)
                    || lexical_environment_owns_binding(environment.iteration_environment.as_ref(), name, slot)
                }) || [plan.head().region(), plan.initialization(), plan.body()].iter()
                    .any(|region| block_environment_owns_binding(region.block(), name, slot))
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                block_environment_owns_binding(plan.body().block(), name, slot)
            }
            StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => {
                block_environment_owns_binding(plan.body().block(), name, slot)
            }
            StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
                block_environment_owns_binding(plan.body().block(), name, slot)
            }
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                block_environment_owns_binding(plan.body(), name, slot)
            }
            StatementIr::OrdinaryGeneratorWith(plan) => {
                lexical_environment_owns_binding(Some(plan.lexical_environment()), name, slot)
                    || block_environment_owns_binding(plan.head().region().block(), name, slot)
                    || block_environment_owns_binding(plan.body().block(), name, slot)
            }
            StatementIr::AsyncFunctionWith(plan) => {
                lexical_environment_owns_binding(Some(plan.lexical_environment()), name, slot)
                    || block_environment_owns_binding(plan.head(), name, slot)
                    || block_environment_owns_binding(plan.body(), name, slot)
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                lexical_environment_owns_binding(plan.lexical_environment(), name, slot)
                    || plan
                        .lexical_declarations()
                        .iter()
                        .any(|statement| statement_owns_binding(statement, name, slot))
                    || plan
                        .regions()
                        .any(|region| block_environment_owns_binding(region.block(), name, slot))
            }
            StatementIr::ResumableClassDefinition(plan) => {
                plan.class().name_binding.as_ref().is_some_and(|binding| {
                    lexical_environment_owns_binding(Some(&binding.environment), name, slot)
                }) || plan
                    .prefixes()
                    .flat_map(|prefix| prefix.statements())
                    .any(|statement| statement_owns_binding(statement, name, slot))
            }
            StatementIr::Block(block) => block_environment_owns_binding(block, name, slot),
            StatementIr::LexicalBlock(statements) => statements
                .iter()
                .any(|statement| statement_owns_binding(statement, name, slot)),
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                plan.lexical_environment().is_some_and(|environment| {
                    environment
                        .bindings
                        .iter()
                        .any(|binding| binding.name == name && binding.slot == slot)
                }) || plan
                    .regions()
                    .any(|region| block_environment_owns_binding(region.block(), name, slot))
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                plan.lexical_environment().is_some_and(|environment| {
                    environment
                        .bindings
                        .iter()
                        .any(|binding| binding.name == name && binding.slot == slot)
                }) || plan
                    .regions()
                    .any(|region| block_environment_owns_binding(region.block(), name, slot))
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                block_environment_owns_binding(plan.then_branch().block(), name, slot)
                    || block_environment_owns_binding(plan.else_branch().block(), name, slot)
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                block_environment_owns_binding(plan.condition().region().block(), name, slot)
                    || block_environment_owns_binding(plan.then_branch().block(), name, slot)
                    || block_environment_owns_binding(plan.else_branch().block(), name, slot)
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            }
            | StatementIr::AsyncFunctionIf {
                condition: _,
                then_branch,
                else_branch,
                plan: _,
            } => {
                statement_owns_binding(then_branch, name, slot)
                    || else_branch
                        .as_deref()
                        .is_some_and(|branch| statement_owns_binding(branch, name, slot))
            }
            StatementIr::AsyncFunctionWhile(plan) => {
                plan.condition_prefix()
                    .iter()
                    .any(|statement| statement_owns_binding(statement, name, slot))
                    || statement_owns_binding(plan.body(), name, slot)
            }
            StatementIr::While { body, .. }
            | StatementIr::DoWhile { body, .. }
            | StatementIr::Labelled {
                statement: body, ..
            } => statement_owns_binding(body, name, slot),
            StatementIr::SyncDisposableScope { body, .. }
            | StatementIr::AsyncDisposableScope { body, .. } => {
                block_environment_owns_binding(body, name, slot)
            }
            StatementIr::For {
                body,
                lexical_environment,
                ..
            } => {
                lexical_environment.as_ref().is_some_and(|environment| {
                    environment
                        .bindings
                        .iter()
                        .any(|binding| binding.name == name && binding.slot == slot)
                }) || statement_owns_binding(body, name, slot)
            }
            StatementIr::ForOfIterator {
                body,
                lexical_environment,
                ..
            }
            | StatementIr::ForInArray {
                body,
                lexical_environment,
                ..
            }
            | StatementIr::ForInString {
                body,
                lexical_environment,
                ..
            }
            | StatementIr::ForInObject {
                body,
                lexical_environment,
                ..
            } => {
                lexical_environment.as_ref().is_some_and(|environment| {
                    lexical_environment_owns_binding(
                        environment.tdz_environment.as_ref(),
                        name,
                        slot,
                    ) || lexical_environment_owns_binding(
                        environment.iteration_environment.as_ref(),
                        name,
                        slot,
                    )
                }) || statement_owns_binding(body, name, slot)
            }
            StatementIr::AsyncFunctionForOfIterator { plan, .. } => {
                let environment_owns_binding = plan.head_environment().is_some_and(|environment| {
                    lexical_environment_owns_binding(
                        environment.tdz_environment.as_ref(),
                        name,
                        slot,
                    ) || lexical_environment_owns_binding(
                        environment.iteration_environment.as_ref(),
                        name,
                        slot,
                    )
                });
                environment_owns_binding
                    || plan
                        .body()
                        .statements()
                        .iter()
                        .any(|statement| statement_owns_binding(statement, name, slot))
            }
            StatementIr::GeneratorForOfIterator { plan, .. } => {
                let environment_owns_binding = plan.head_environment().is_some_and(|environment| {
                    lexical_environment_owns_binding(
                        environment.tdz_environment.as_ref(),
                        name,
                        slot,
                    ) || lexical_environment_owns_binding(
                        environment.iteration_environment.as_ref(),
                        name,
                        slot,
                    )
                });
                environment_owns_binding
                    || plan
                        .body()
                        .statements()
                        .iter()
                        .any(|statement| statement_owns_binding(statement, name, slot))
            }
            StatementIr::AsyncFunctionSwitch(plan) => {
                lexical_environment_owns_binding(plan.lexical_environment(), name, slot)
                    || plan.cases().iter().any(|case| {
                        case.condition_prefix()
                            .iter()
                            .any(|statement| statement_owns_binding(statement, name, slot))
                            || block_environment_owns_binding(case.body(), name, slot)
                    })
            }
            StatementIr::Switch {
                lexical_environment,
                cases,
                ..
            } => {
                lexical_environment_owns_binding(lexical_environment.as_ref(), name, slot)
                    || cases
                        .iter()
                        .any(|case| block_environment_owns_binding(&case.body, name, slot))
            }
            StatementIr::TryCatch {
                try_block,
                catch_parameter_environment,
                catch_block,
                ..
            } => {
                block_environment_owns_binding(try_block, name, slot)
                    || lexical_environment_owns_binding(
                        catch_parameter_environment.as_ref(),
                        name,
                        slot,
                    )
                    || block_environment_owns_binding(catch_block, name, slot)
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                block_environment_owns_binding(try_block, name, slot)
                    || block_environment_owns_binding(finally_block, name, slot)
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_parameter_environment,
                catch_block,
                finally_block,
                ..
            } => {
                block_environment_owns_binding(try_block, name, slot)
                    || lexical_environment_owns_binding(
                        catch_parameter_environment.as_ref(),
                        name,
                        slot,
                    )
                    || block_environment_owns_binding(catch_block, name, slot)
                    || block_environment_owns_binding(finally_block, name, slot)
            }
            _ => false,
        }
    }

    block
        .statements
        .iter()
        .any(|statement| statement_owns_binding(statement, name, slot))
}

fn assert_function_capture_storage_contract(
    source: &str,
    owner_name: &str,
    capture_function_name: Option<&str>,
    expected_storage_prefix: &str,
) {
    let program = lower_script(source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == owner_name)
        .expect("capture owner should be lowered");
    let capture = script
        .functions
        .iter()
        .filter(|function| capture_function_name.map_or(true, |name| function.name == name))
        .flat_map(|function| &function.captured_bindings)
        .find(|binding| binding.name.starts_with(expected_storage_prefix))
        .expect("capturing function should use the expected physical binding");
    assert!(
        owner
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == capture.name && binding.slot == capture.slot)
            || block_environment_owns_binding(&owner.body, &capture.name, capture.slot)
    );
    assert!(collect_binding_storage_names(&owner.body).contains(&capture.name));
}

fn assert_canonical_derived_activation(function: &FunctionIr) {
    let activation = function
        .lexical_derived_activation
        .as_ref()
        .expect("derived constructor should own activation metadata");
    assert_eq!(activation.owner_function_id, function.id);
    assert_eq!(activation.this_binding, DERIVED_ACTIVATION_THIS_NAME);
    assert_eq!(
        activation.this_status_binding,
        DERIVED_ACTIVATION_THIS_STATUS_NAME
    );
    assert_eq!(
        activation.new_target_binding,
        DERIVED_ACTIVATION_NEW_TARGET_NAME
    );
    assert_eq!(
        activation.active_function_binding,
        DERIVED_ACTIVATION_FUNCTION_NAME
    );
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .map(|binding| (binding.name.as_str(), binding.slot))
            .collect::<Vec<_>>(),
        vec![
            (DERIVED_ACTIVATION_FUNCTION_NAME, 0),
            (DERIVED_ACTIVATION_NEW_TARGET_NAME, 1),
            (DERIVED_ACTIVATION_THIS_NAME, 2),
            (DERIVED_ACTIVATION_THIS_STATUS_NAME, 3),
        ]
    );
}

fn with_script_analysis(source: &str, assert_analysis: impl FnOnce(&Analysis<'_>)) {
    let mut interner = Interner::default();
    let scope = Scope::new_global();
    let parsed_script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&scope, &mut interner)
        .expect("script should parse");
    let analysis = AnalysisBuilder::default().finish(&parsed_script, &interner, source);
    assert_analysis(&analysis);
}

fn function_owner_plan_by_name<'a>(analysis: &'a Analysis<'_>, name: &str) -> &'a OwnerPlan {
    let function = analysis
        .function_plans
        .values()
        .find(|function| function.name == name)
        .expect("function should be planned");
    &analysis.owner_plans[&function.id]
}

fn environment_with_binding_suffix<'a>(
    analysis: &'a Analysis<'_>,
    suffix: &str,
) -> &'a EnvironmentPlan {
    analysis
        .environment_plans
        .values()
        .find(|environment| {
            environment
                .binding_storage_names
                .iter()
                .any(|binding| binding.ends_with(suffix))
        })
        .expect("environment should own the physical binding")
}

fn binding_with_suffix<'a>(environment: &'a EnvironmentPlan, suffix: &str) -> &'a str {
    environment
        .binding_storage_names
        .iter()
        .find(|binding| binding.ends_with(suffix))
        .map(String::as_str)
        .expect("environment should contain the physical binding")
}

fn assert_physical_binding_owner(
    analysis: &Analysis<'_>,
    binding_storage_name: &str,
    environment: &EnvironmentPlan,
) {
    let owners = analysis
        .physical_binding_environments
        .get(binding_storage_name)
        .expect("physical binding should have an environment owner");
    assert_eq!(owners.len(), 1);
    assert!(owners.contains(&environment.id));
}
