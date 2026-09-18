//! Trusted source positions map private module owners into semantic AOT operations.

use super::record::DefaultExportFormIr;
use super::synchronous_execution::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct SynchronousModuleAnalysis {
    pub(crate) graphs: BTreeMap<usize, SynchronousModuleGraphIr>,
    pub(crate) imports: BTreeMap<usize, ModuleCellIr>,
    pub(crate) reads: BTreeMap<usize, ModuleCellIr>,
    pub(crate) evaluations: BTreeMap<usize, SynchronousModuleEvaluationIr>,
    pub(crate) deferred_evaluations: BTreeMap<usize, DeferredModuleEvaluationIr>,
    pub(crate) publishers: BTreeMap<usize, (u32, ModuleNamespaceModeIr)>,
    pub(crate) boundaries: BTreeSet<usize>,
}

#[derive(Debug)]
pub(super) struct SynchronousUnitDefinition {
    pub(super) module: u32,
    pub(super) imports: Vec<ResolvedBindingIr>,
    pub(super) namespaces: Vec<(ModuleNamespaceModeIr, Vec<ResolvedBindingIr>)>,
    pub(super) has_import_meta: bool,
    pub(super) default_export: DefaultExportFormIr,
    pub(super) evaluation: SynchronousModuleEvaluationIr,
    pub(super) readiness: Vec<ModuleReadinessNodeIr>,
}

#[derive(Debug)]
pub(super) struct SynchronousModuleDefinitions {
    pub(super) graph_span: (boa_ast::Position, boa_ast::Position),
    pub(super) units: Vec<SynchronousUnitDefinition>,
    pub(super) record_count: u32,
    pub(super) dispatcher_namespaces: BTreeMap<String, ModuleCellIr>,
    pub(super) initial_evaluation: Vec<u32>,
}

impl SynchronousModuleDefinitions {
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
        let mut evaluators = Vec::new();
        for pair in graph.as_ref() {
            let Some(Expression::ArrayLiteral(pair)) = pair.as_ref().map(Expression::flatten)
            else {
                panic!("module entry holds owner and evaluator");
            };
            assert_eq!(pair.as_ref().len(), 2);
            let Some(Expression::AsyncArrowFunction(owner)) =
                pair.as_ref()[0].as_ref().map(Expression::flatten)
            else {
                panic!("module owner is lexically transparent");
            };
            let Some(Expression::ArrowFunction(evaluator)) =
                pair.as_ref()[1].as_ref().map(Expression::flatten)
            else {
                panic!("module evaluator is a private closure");
            };
            owners.push(owner);
            evaluators.push(evaluator);
        }
        let functions = owners
            .iter()
            .map(|owner| analysis.function_expr_ids[&async_arrow_function_key(owner)].clone())
            .collect::<Vec<_>>();
        let evaluator_functions = evaluators
            .iter()
            .map(|owner| analysis.function_expr_ids[&arrow_function_key(owner)].clone())
            .collect::<Vec<_>>();
        let mut cells = BTreeMap::new();
        for ((unit, owner), function) in self.units.iter().zip(&owners).zip(&functions) {
            analysis
                .function_plans
                .get_mut(function)
                .expect("module owner was analyzed")
                .protocol = FunctionProtocolIr::ModuleActivation;
            analysis
                .owner_plans
                .get_mut(function)
                .expect("module environment was analyzed")
                .execution_kind = FunctionExecutionKind::Generator;
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
            _ => panic!(
                "synchronous instantiation requires resolved environment or namespace bindings"
            ),
        };
        for (((unit, owner), evaluator), function) in self
            .units
            .iter()
            .zip(&owners)
            .zip(&evaluators)
            .zip(&functions)
        {
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
                analysis.synchronous_modules.imports.insert(
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
                    .synchronous_modules
                    .publishers
                    .insert(pointer, (unit.module, *mode));
                assert_eq!(array.as_ref().len(), 1 + 2 * exports.len());
                if *mode == ModuleNamespaceModeIr::Deferred {
                    analysis.synchronous_modules.deferred_evaluations.insert(
                        std::ptr::from_ref(reader_expression(
                            array.as_ref()[0].as_ref().expect("evaluator"),
                        )) as usize,
                        DeferredModuleEvaluationIr {
                            module: unit.module,
                            readiness: unit.readiness.clone(),
                        },
                    );
                }
                for (index, export) in exports.iter().enumerate() {
                    let expression = reader_expression(
                        array.as_ref()[2 + index * 2]
                            .as_ref()
                            .expect("export reader"),
                    );
                    analysis
                        .synchronous_modules
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
                .synchronous_modules
                .boundaries
                .insert(std::ptr::from_ref(boundary.as_ref()) as usize);
            let [StatementListItem::Statement(statement)] = evaluator.body().statements() else {
                panic!("private evaluator has one return");
            };
            let Statement::Return(statement) = statement.as_ref() else {
                panic!("private evaluator returns");
            };
            analysis.synchronous_modules.evaluations.insert(
                std::ptr::from_ref(statement.target().expect("evaluation placeholder")) as usize,
                unit.evaluation.clone(),
            );
            super::default_export_definition::apply_synchronous_default(
                owner.body(),
                unit.module,
                unit.default_export,
                analysis,
                interner,
            );
            assert_eq!(
                analysis.function_plans[function].protocol,
                FunctionProtocolIr::ModuleActivation
            );
        }
        // Only linker-generated dispatcher source can name these private cells.
        // No Script lexical aliases are created for user code to resolve.
        struct DispatcherReads<'a, 'b> {
            namespaces: &'b BTreeMap<String, ModuleCellIr>,
            interner: &'b Interner,
            analysis: &'b mut Analysis<'a>,
        }
        impl<'a> Visitor<'a> for DispatcherReads<'a, '_> {
            type BreakTy = ();
            fn visit_expression(&mut self, expression: &'a Expression) -> ControlFlow<()> {
                if let Expression::Identifier(identifier) = expression {
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    if let Some(target) = self.namespaces.get(&name) {
                        self.analysis
                            .synchronous_modules
                            .reads
                            .insert(std::ptr::from_ref(expression) as usize, target.clone());
                    }
                }
                expression.visit_with(self)
            }
        }
        let mut dispatcher = DispatcherReads {
            namespaces: &self.dispatcher_namespaces,
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
            analysis.synchronous_modules.evaluations.insert(
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
        analysis.synchronous_modules.graphs.insert(
            std::ptr::from_ref(graph) as usize,
            SynchronousModuleGraphIr {
                record_count: self.record_count,
                activations: self
                    .units
                    .iter()
                    .zip(functions)
                    .zip(evaluator_functions)
                    .map(
                        |((unit, function), evaluator)| SynchronousModuleActivationIr {
                            module: unit.module,
                            function,
                            evaluator,
                        },
                    )
                    .collect(),
            },
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
