//! Trusted source positions map private module owners into semantic AOT operations.

use super::record::DefaultExportFormIr;
use super::synchronous_execution::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct ModuleExecutionAnalysis {
    pub(crate) graphs: BTreeMap<usize, ModuleExecutionGraphIr>,
    pub(crate) imports: BTreeMap<usize, ModuleCellIr>,
    pub(crate) reads: BTreeMap<usize, ModuleCellIr>,
    pub(crate) evaluations: BTreeMap<usize, ModuleEvaluationIr>,
    pub(crate) async_dependencies: BTreeMap<usize, ModuleEvaluationIr>,
    pub(crate) deferred_imports: BTreeMap<usize, ModuleEvaluationIr>,
    pub(crate) deferred_evaluations: BTreeMap<usize, DeferredModuleEvaluationIr>,
    pub(crate) publishers: BTreeMap<usize, (u32, ModuleNamespaceModeIr)>,
    pub(crate) boundaries: BTreeSet<usize>,
    pub(crate) intrinsics: BTreeMap<usize, StandardBuiltinId>,
}

#[derive(Debug)]
pub(super) struct ModuleUnitDefinition {
    pub(super) module: u32,
    pub(super) imports: Vec<ResolvedBindingIr>,
    pub(super) namespaces: Vec<(ModuleNamespaceModeIr, Vec<ResolvedBindingIr>)>,
    pub(super) has_import_meta: bool,
    pub(super) default_export: DefaultExportFormIr,
    pub(super) evaluation: ModuleEvaluationIr,
    pub(super) kind: ModuleActivationKindIr,
    pub(super) requests: Vec<ModuleExecutionRequestIr>,
}

#[derive(Debug)]
pub(super) struct ModuleExecutionDefinitions {
    pub(super) graph_span: (boa_ast::Position, boa_ast::Position),
    pub(super) units: Vec<ModuleUnitDefinition>,
    pub(super) record_count: u32,
    pub(super) dispatcher_namespaces: BTreeMap<String, ModuleCellIr>,
    pub(super) dispatcher_evaluations: BTreeMap<String, ModuleEvaluationIr>,
    pub(super) dispatcher_async_dependencies: BTreeMap<String, ModuleEvaluationIr>,
    pub(super) dispatcher_deferred_imports: BTreeMap<String, ModuleEvaluationIr>,
    pub(super) initial_evaluation: Vec<u32>,
}

impl ModuleExecutionDefinitions {
    pub(super) fn apply<'a>(
        &self,
        script: &'a Script,
        analysis: &mut Analysis<'a>,
        interner: &Interner,
    ) {
        let statements = script.statements().statements();
        let (graph_index, graph) = statements
            .iter()
            .enumerate()
            .find_map(|(index, statement)| {
                let StatementListItem::Statement(statement) = statement else {
                    return None;
                };
                let Statement::Expression(Expression::ArrayLiteral(array)) = statement.as_ref()
                else {
                    return None;
                };
                let span = array.span();
                ((span.start(), span.end()) == self.graph_span).then_some((index, array))
            })
            .expect("trusted module graph span survives Script parsing");
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
            _ => panic!("module instantiation requires resolved environment or namespace bindings"),
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
            let StatementListItem::Statement(boundary) =
                statements.next().expect("instantiation boundary")
            else {
                panic!("boundary is a statement");
            };
            analysis
                .module_execution
                .boundaries
                .insert(std::ptr::from_ref(boundary.as_ref()) as usize);
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
        for statement in &statements[..graph_index] {
            let _ = statement.visit_with(&mut dispatcher);
        }
        let mut following = statements[graph_index + 1..].iter();
        for module in &self.initial_evaluation {
            let StatementListItem::Statement(statement) =
                following.next().expect("initial evaluation exists")
            else {
                panic!("evaluation is an expression");
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
        assert!(following.next().is_none());
        analysis.module_execution.graphs.insert(
            std::ptr::from_ref(graph) as usize,
            ModuleExecutionGraphIr::new(
                self.record_count,
                self.units
                    .iter()
                    .zip(functions)
                    .map(|(unit, function)| {
                        ModuleActivationIr::new(
                            unit.module,
                            function,
                            unit.kind,
                            unit.requests.clone(),
                        )
                    })
                    .collect(),
            ),
        );
    }
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
