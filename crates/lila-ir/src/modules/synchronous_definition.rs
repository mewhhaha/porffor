//! Trusted source positions map private module owners into semantic AOT operations.

use super::record::DefaultExportFormIr;
use super::synchronous_execution::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct ModuleExecutionAnalysis {
    private_dispatchers: BTreeMap<FunctionId, String>,
    original_callable_sources: BTreeMap<(usize, usize), String>,
    pub(crate) graphs: BTreeMap<usize, ModuleExecutionGraphIr>,
    pub(crate) initializers: BTreeMap<usize, ModuleExecutionGraphIr>,
    pub(crate) imports: BTreeMap<usize, ModuleCellIr>,
    pub(crate) reads: BTreeMap<usize, ModuleCellIr>,
    pub(crate) evaluations: BTreeMap<usize, ModuleEvaluationIr>,
    pub(crate) async_dependencies: BTreeMap<usize, ModuleEvaluationIr>,
    pub(crate) deferred_imports: BTreeMap<usize, ModuleEvaluationIr>,
    pub(crate) deferred_evaluations: BTreeMap<usize, DeferredModuleEvaluationIr>,
    pub(crate) publishers: BTreeMap<usize, (u32, ModuleNamespaceModeIr)>,
    pub(crate) boundaries: BTreeSet<usize>,
    pub(crate) intrinsics: BTreeMap<usize, StandardBuiltinId>,
    pub(crate) json_values: BTreeMap<usize, JsonModuleValueIr>,
}

impl ModuleExecutionAnalysis {
    pub(super) fn record_original_callable_sources(
        &mut self,
        sources: BTreeMap<(usize, usize), String>,
    ) {
        self.original_callable_sources = sources;
    }

    pub(crate) fn original_callable_source(&self, span: boa_ast::LinearSpan) -> Option<&str> {
        self.original_callable_sources
            .get(&(span.start().pos(), span.end().pos()))
            .map(String::as_str)
    }

    pub(crate) fn is_private_dispatcher_name(&self, name: &str) -> bool {
        self.private_dispatchers
            .values()
            .any(|private| private == name)
    }

    pub(crate) fn is_private_dispatcher_function(&self, function: &FunctionId) -> bool {
        self.private_dispatchers.contains_key(function)
    }
}

#[derive(Debug)]
pub(super) struct ModuleUnitDefinition {
    pub(super) module: u32,
    pub(super) imports: Vec<ResolvedBindingIr>,
    pub(super) namespaces: Vec<(ModuleNamespaceModeIr, Vec<ResolvedBindingIr>)>,
    pub(super) has_import_meta: bool,
    pub(super) default_export: DefaultExportFormIr,
    pub(super) json: Option<JsonModuleValueIr>,
    pub(super) evaluation: ModuleEvaluationIr,
    pub(super) kind: ModuleActivationKindIr,
    pub(super) requests: Vec<ModuleExecutionRequestIr>,
}

#[derive(Debug)]
pub(super) struct ModuleExecutionDefinitions {
    pub(super) realm_requests: BTreeMap<ModuleRequestKeyIr, RealmModuleResolutionIr>,
    pub(super) graph_span: (boa_ast::Position, boa_ast::Position),
    pub(super) units: Vec<ModuleUnitDefinition>,
    pub(super) record_count: u32,
    pub(super) dispatcher_namespaces: BTreeMap<String, ModuleCellIr>,
    pub(super) dispatcher_evaluations: BTreeMap<String, ModuleEvaluationIr>,
    pub(super) dispatcher_async_dependencies: BTreeMap<String, ModuleEvaluationIr>,
    pub(super) dispatcher_deferred_imports: BTreeMap<String, ModuleEvaluationIr>,
    pub(super) entry: ModuleExecutionEntry,
}

impl ModuleExecutionDefinitions {
    pub(super) fn apply<'a>(
        &self,
        script: &'a Script,
        analysis: &mut Analysis<'a>,
        interner: &Interner,
    ) {
        let root_statements = script.statements().statements();
        let (initializer_index, initializer_expression, initializer, graph_index, graph) =
            root_statements
                .iter()
                .enumerate()
                .find_map(|(index, statement)| {
                    let StatementListItem::Statement(statement) = statement else {
                        return None;
                    };
                    let Statement::Expression(expression) = statement.as_ref() else {
                        return None;
                    };
                    let Expression::ArrowFunction(initializer) = expression.flatten() else {
                        return None;
                    };
                    initializer.body().statements().iter().enumerate().find_map(
                        |(graph_index, statement)| {
                            let StatementListItem::Statement(statement) = statement else {
                                return None;
                            };
                            let Statement::Expression(Expression::ArrayLiteral(array)) =
                                statement.as_ref()
                            else {
                                return None;
                            };
                            let span = array.span();
                            ((span.start(), span.end()) == self.graph_span).then_some((
                                index,
                                expression.flatten(),
                                initializer,
                                graph_index,
                                array,
                            ))
                        },
                    )
                })
                .expect("trusted module graph span survives Script parsing");
        let initializer_function =
            analysis.function_expr_ids[&arrow_function_key(initializer)].clone();
        let statements = initializer.body().statements();
        // Only declarations within the compiler-owned prelude become private
        // root storage. User functions in the preserved Script tail retain
        // ordinary global declaration semantics, regardless of their spelling.
        for statement in root_statements[..initializer_index]
            .iter()
            .chain(&statements[..graph_index])
        {
            let StatementListItem::Declaration(declaration) = statement else {
                continue;
            };
            let Declaration::AsyncFunctionDeclaration(function) = declaration.as_ref() else {
                panic!("the compiled import prelude contains only async dispatcher declarations");
            };
            let function_id = analysis.function_declaration_ids
                [&async_function_declaration_key(function)]
                .clone();
            analysis.module_execution.private_dispatchers.insert(
                function_id,
                interner.resolve_expect(function.name().sym()).to_string(),
            );
        }
        // Private storage can share a physical activation with user source,
        // but source eval must never resolve the compiler's helper names. Keep
        // slots and captures intact while removing only their named exposure.
        let private_dispatchers = &analysis.module_execution;
        for environment in analysis.environment_plans.values_mut() {
            if let Some(EvalEnvironmentRoleIr::Declarative { bindings, .. }) =
                &mut environment.eval_environment
            {
                bindings.retain(|binding| {
                    !private_dispatchers.is_private_dispatcher_name(&binding.source_name)
                });
            }
        }
        assert_eq!(graph.as_ref().len(), self.units.len());
        let mut owners = Vec::new();
        for expression in graph.as_ref() {
            let Some(Expression::AsyncArrowFunction(owner)) =
                expression.as_ref().map(Expression::flatten)
            else {
                panic!("module owner is lexically transparent");
            };
            owners.push(owner);
        }
        let functions = owners
            .iter()
            .map(|owner| analysis.function_expr_ids[&async_arrow_function_key(owner)].clone())
            .collect::<Vec<_>>();
        let mut cells = BTreeMap::new();
        for ((unit, owner), function) in self.units.iter().zip(&owners).zip(&functions) {
            analysis
                .function_plans
                .get_mut(function)
                .expect("module owner was analyzed")
                .protocol = match unit.kind {
                ModuleActivationKindIr::Synchronous => FunctionProtocolIr::ModuleActivation,
                ModuleActivationKindIr::Async => FunctionProtocolIr::AsyncModuleActivation,
            };
            analysis
                .owner_plans
                .get_mut(function)
                .expect("module environment was analyzed")
                .execution_kind = match unit.kind {
                ModuleActivationKindIr::Synchronous => FunctionExecutionKind::Generator,
                ModuleActivationKindIr::Async => FunctionExecutionKind::Async,
            };
            if unit.kind == ModuleActivationKindIr::Synchronous {
                remove_wrapper_for_of_continuations(
                    owner.body(),
                    &mut analysis.complete_for_of_owners,
                );
            }
            let plan = &analysis.owner_plans[function];
            for (name, slot) in &plan.owned_env_slots {
                cells.insert((unit.module, name.clone()), *slot);
            }
            for statement in owner.body().statements() {
                if let StatementListItem::Declaration(declaration) = statement {
                    match declaration.as_ref() {
                        Declaration::Lexical(declaration) => {
                            for variable in declaration.variable_list().as_ref() {
                                if let Binding::Identifier(identifier) = variable.binding() {
                                    let name =
                                        interner.resolve_expect(identifier.sym()).to_string();
                                    let storage = scoped_lexical_binding_storage_name(
                                        &name,
                                        identifier.span(),
                                    );
                                    if let Some(slot) = plan.owned_env_slots.get(&storage) {
                                        cells.insert((unit.module, name), *slot);
                                    }
                                }
                            }
                        }
                        Declaration::ClassDeclaration(class) => {
                            let identifier = class.name();
                            let name = interner.resolve_expect(identifier.sym()).to_string();
                            let storage =
                                scoped_lexical_binding_storage_name(&name, identifier.span());
                            if let Some(slot) = plan.owned_env_slots.get(&storage) {
                                cells.insert((unit.module, name), *slot);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        let resolve = |target: &ResolvedBindingIr| match target {
            ResolvedBindingIr::Resolved {
                module,
                binding: ModuleBindingNameIr::Name(name),
            } => {
                let name = name.merged_in(*module);
                ModuleCellIr::Binding {
                    module: *module,
                    slot: *cells
                        .get(&(*module, name.as_str().to_string()))
                        .unwrap_or_else(|| {
                            panic!("module {module} binding {name:?} must own a canonical slot")
                        }),
                }
            }
            ResolvedBindingIr::Resolved {
                module,
                binding: ModuleBindingNameIr::Namespace(mode),
            } => ModuleCellIr::Namespace {
                module: *module,
                mode: *mode,
            },
            // Admission rejects graphs with source-phase imports, and linking
            // rejects unresolved or ambiguous imports, before instantiation.
            ResolvedBindingIr::Resolved {
                binding: ModuleBindingNameIr::ModuleSource,
                ..
            } => panic!("module instantiation admits no source-phase import bindings"),
            ResolvedBindingIr::Ambiguous | ResolvedBindingIr::NotFound => {
                panic!("module instantiation requires resolved environment or namespace bindings")
            }
        };
        for ((unit, owner), function) in self.units.iter().zip(&owners).zip(&functions) {
            let mut statements = owner.body().statements().iter();
            statements.next().expect("strict directive exists");
            for target in &unit.imports {
                let StatementListItem::Declaration(declaration) =
                    statements.next().expect("import exists")
                else {
                    panic!("import is lexical");
                };
                let Declaration::Lexical(declaration) = declaration.as_ref() else {
                    panic!("import is lexical");
                };
                let [variable] = declaration.variable_list().as_ref() else {
                    panic!("one import per declaration");
                };
                analysis.module_execution.imports.insert(
                    std::ptr::from_ref(variable.init().expect("import placeholder")) as usize,
                    resolve(target),
                );
            }
            if unit.has_import_meta {
                statements.next().expect("import.meta binding exists");
            }
            for (mode, exports) in &unit.namespaces {
                let StatementListItem::Statement(statement) =
                    statements.next().expect("namespace exists")
                else {
                    panic!("namespace is an expression");
                };
                let Statement::Expression(Expression::ArrayLiteral(array)) = statement.as_ref()
                else {
                    panic!("namespace is a private table");
                };
                let pointer = std::ptr::from_ref(array) as usize;
                analysis.namespace_initializers.insert(pointer, *mode);
                analysis
                    .module_execution
                    .publishers
                    .insert(pointer, (unit.module, *mode));
                assert_eq!(array.as_ref().len(), 1 + 2 * exports.len());
                if *mode == ModuleNamespaceModeIr::Deferred {
                    analysis.module_execution.deferred_evaluations.insert(
                        std::ptr::from_ref(reader_expression(
                            array.as_ref()[0].as_ref().expect("evaluator"),
                        )) as usize,
                        DeferredModuleEvaluationIr::new(unit.module),
                    );
                }
                for (index, export) in exports.iter().enumerate() {
                    let expression = reader_expression(
                        array.as_ref()[2 + index * 2]
                            .as_ref()
                            .expect("export reader"),
                    );
                    analysis
                        .module_execution
                        .reads
                        .insert(std::ptr::from_ref(expression) as usize, resolve(export));
                }
            }
            if unit.json.is_some() {
                let StatementListItem::Declaration(declaration) =
                    statements.next().expect("JSON default cell")
                else {
                    panic!("JSON default cell is lexical")
                };
                let Declaration::Lexical(declaration) = declaration.as_ref() else {
                    panic!("JSON default cell is lexical")
                };
                assert_eq!(declaration.variable_list().as_ref().len(), 1);
            }
            let StatementListItem::Statement(boundary) =
                statements.next().expect("instantiation boundary")
            else {
                panic!("boundary is a statement");
            };
            analysis
                .module_execution
                .boundaries
                .insert(std::ptr::from_ref(boundary.as_ref()) as usize);
            if let Some(json) = &unit.json {
                let StatementListItem::Statement(statement) =
                    statements.next().expect("JSON native evaluation")
                else {
                    panic!("JSON evaluation is a statement")
                };
                let Statement::Expression(Expression::Assign(assignment)) = statement.as_ref()
                else {
                    panic!("JSON evaluation publishes its default cell")
                };
                analysis
                    .module_execution
                    .json_values
                    .insert(std::ptr::from_ref(assignment.rhs()) as usize, json.clone());
            }
            super::default_export_definition::apply_synchronous_default(
                owner.body(),
                unit.module,
                unit.default_export,
                analysis,
                interner,
            );
            assert_eq!(
                analysis.function_plans[function].protocol,
                match unit.kind {
                    ModuleActivationKindIr::Synchronous => FunctionProtocolIr::ModuleActivation,
                    ModuleActivationKindIr::Async => FunctionProtocolIr::AsyncModuleActivation,
                }
            );
        }
        // Only linker-generated dispatcher source can name these private cells.
        // No Script lexical aliases are created for user code to resolve.
        struct DispatcherReads<'a, 'b> {
            namespaces: &'b BTreeMap<String, ModuleCellIr>,
            evaluations: &'b BTreeMap<String, ModuleEvaluationIr>,
            async_dependencies: &'b BTreeMap<String, ModuleEvaluationIr>,
            deferred_imports: &'b BTreeMap<String, ModuleEvaluationIr>,
            interner: &'b Interner,
            analysis: &'b mut Analysis<'a>,
        }
        impl<'a> Visitor<'a> for DispatcherReads<'a, '_> {
            type BreakTy = ();
            fn visit_expression(&mut self, expression: &'a Expression) -> ControlFlow<()> {
                if let Expression::Identifier(identifier) = expression {
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    let pointer = std::ptr::from_ref(expression) as usize;
                    if let Some(builtin) = super::dynamic::module_dispatcher_intrinsic(&name) {
                        self.analysis
                            .module_execution
                            .intrinsics
                            .insert(pointer, builtin);
                    } else if let Some(evaluation) = self.evaluations.get(&name) {
                        self.analysis
                            .module_execution
                            .evaluations
                            .insert(pointer, evaluation.clone());
                    } else if let Some(evaluation) = self.async_dependencies.get(&name) {
                        self.analysis
                            .module_execution
                            .async_dependencies
                            .insert(pointer, evaluation.clone());
                    } else if let Some(evaluation) = self.deferred_imports.get(&name) {
                        self.analysis
                            .module_execution
                            .deferred_imports
                            .insert(pointer, evaluation.clone());
                    } else if let Some(target) = self.namespaces.get(&name) {
                        self.analysis
                            .module_execution
                            .reads
                            .insert(std::ptr::from_ref(expression) as usize, target.clone());
                    }
                }
                expression.visit_with(self)
            }
        }
        let mut dispatcher = DispatcherReads {
            namespaces: &self.dispatcher_namespaces,
            evaluations: &self.dispatcher_evaluations,
            async_dependencies: &self.dispatcher_async_dependencies,
            deferred_imports: &self.dispatcher_deferred_imports,
            interner,
            analysis,
        };
        for statement in root_statements[..initializer_index]
            .iter()
            .chain(&statements[..graph_index])
        {
            let _ = statement.visit_with(&mut dispatcher);
        }
        match &self.entry {
            ModuleExecutionEntry::Module(module) => {
                let [StatementListItem::Statement(statement)] =
                    &root_statements[initializer_index + 1..]
                else {
                    panic!("a Module graph ends with exactly its entry evaluation");
                };
                let Statement::Expression(expression) = statement.as_ref() else {
                    panic!("evaluation is an expression");
                };
                analysis.module_execution.evaluations.insert(
                    std::ptr::from_ref(expression) as usize,
                    self.units
                        .iter()
                        .find(|unit| unit.module == *module)
                        .expect("initial module exists")
                        .evaluation
                        .clone(),
                );
            }
            // The preserved Script tail is ordinary user source: no private
            // entry evaluation consumes or replaces its completion value.
            ModuleExecutionEntry::Script(_) => {}
        }
        let realm_import_dispatcher = analysis
            .module_execution
            .private_dispatchers
            .iter()
            .find(|(_, name)| name.as_str() == super::realm_request::REALM_IMPORT_DISPATCHER)
            .map(|(id, _)| id.clone())
            .expect("trusted Realm import dispatcher is declared");
        let execution_graph = ModuleExecutionGraphIr::new(
            self.record_count,
            self.units
                .iter()
                .zip(functions)
                .map(|(unit, function)| {
                    ModuleActivationIr::new(unit.module, function, unit.kind, unit.requests.clone())
                })
                .collect(),
            match &self.entry {
                ModuleExecutionEntry::Module(module) => ModuleExecutionEntry::Module(*module),
                ModuleExecutionEntry::Script(script) => ModuleExecutionEntry::Script(*script),
            },
        )
        .with_realm_requests(self.realm_requests.clone(), realm_import_dispatcher);
        analysis.module_execution.initializers.insert(
            std::ptr::from_ref(initializer_expression) as usize,
            execution_graph
                .clone()
                .initialize_with(initializer_function),
        );
        analysis
            .module_execution
            .graphs
            .insert(std::ptr::from_ref(graph) as usize, execution_graph);
    }
}

fn remove_wrapper_for_of_continuations(
    body: &FunctionBody,
    owners: &mut BTreeMap<usize, crate::analysis::CompleteResumableForOfOwner>,
) {
    // The trusted async-arrow wrapper was analyzed before its canonical module
    // protocol was installed. Synchronous evaluation has no continuation after
    // its private instantiation boundary; its original iteration records stay.
    struct ModuleBody<'a>(&'a mut BTreeMap<usize, crate::analysis::CompleteResumableForOfOwner>);
    impl<'ast> Visitor<'ast> for ModuleBody<'_> {
        type BreakTy = ();

        fn visit_for_of_loop(&mut self, source: &'ast ForOfLoop) -> ControlFlow<()> {
            self.0.remove(&(std::ptr::from_ref(source) as usize));
            source.visit_with(self)
        }

        fn visit_function_body(&mut self, _: &'ast FunctionBody) -> ControlFlow<()> {
            ControlFlow::Continue(())
        }
    }
    let _ = body.visit_with(&mut ModuleBody(owners));
}

fn reader_expression(expression: &Expression) -> &Expression {
    let Expression::ArrowFunction(reader) = expression.flatten() else {
        panic!("private reader is an arrow");
    };
    let [StatementListItem::Statement(statement)] = reader.body().statements() else {
        panic!("private reader has one return");
    };
    let Statement::Return(statement) = statement.as_ref() else {
        panic!("private reader returns its operand");
    };
    statement.target().expect("private reader operand")
}
