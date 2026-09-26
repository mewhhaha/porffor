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
    pub(super) bytes_len: Option<usize>,
}

/// What the merged program runs after the graph statement.
#[derive(Debug)]
pub(super) enum ModuleExecutionEntry {
    /// One private evaluation statement starts the Module entry's DFS.
    Module(ModuleEvaluationIr),
    /// The Script entry's own statements follow; they are user code, and no
    /// trusted operation is recognized among them.
    Script,
}

#[cfg(test)]
impl ModuleExecutionEntry {
    pub(super) const fn evaluated_module(&self) -> Option<ModuleUnitId> {
        match self {
            Self::Module(evaluation) => Some(evaluation.module()),
            Self::Script => None,
        }
    }
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
    pub(super) entry: ModuleExecutionEntry,
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
            if let Some(bytes_len) = unit.bytes_len {
                mark_bytes_intrinsics(owner.body(), bytes_len, analysis);
            }
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
        match &self.entry {
            ModuleExecutionEntry::Module(evaluation) => {
                let [StatementListItem::Statement(statement)] = &statements[graph_index + 1..]
                else {
                    panic!("one evaluation statement follows a Module entry's graph");
                };
                let Statement::Expression(expression) = statement.as_ref() else {
                    panic!("evaluation is an expression");
                };
                assert!(
                    self.units.iter().any(|unit| unit.evaluation == *evaluation),
                    "the Module entry owns an activation"
                );
                analysis
                    .module_execution
                    .evaluations
                    .insert(std::ptr::from_ref(expression) as usize, evaluation.clone());
            }
            ModuleExecutionEntry::Script => {}
        }
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

/// Only an owner backed by host Bytes provenance can acquire these exact
/// intrinsic expressions. Its synthetic body has two constructors and two
/// `Reflect.apply` calls; every other expression is a literal, local read, or
/// canonical integer-indexed store. The parser owns the pointers, so source
/// identifiers and filenames do not participate in the privilege decision.
pub(super) fn mark_bytes_intrinsics<'a>(
    body: &'a FunctionBody,
    bytes_len: usize,
    analysis: &mut Analysis<'a>,
) {
    struct Sites<'a, 'b> {
        analysis: &'b mut Analysis<'a>,
        constructors: usize,
        calls: usize,
        stores: usize,
    }
    impl<'a> Visitor<'a> for Sites<'a, '_> {
        type BreakTy = ();

        fn visit_expression(&mut self, expression: &'a Expression) -> ControlFlow<()> {
            match expression {
                Expression::New(new) => {
                    let callee = new.constructor().flatten();
                    self.analysis.module_execution.intrinsics.insert(
                        std::ptr::from_ref(callee) as usize,
                        StandardBuiltinId::Uint8ArrayConstructor,
                    );
                    self.constructors += 1;
                }
                Expression::Call(call) => {
                    if matches!(call.function().flatten(), Expression::ArrowFunction(_)) {
                        return expression.visit_with(self);
                    }
                    let getter = match self.calls {
                        0 => StandardBuiltinId::TypedArrayPrototypeBufferGetter,
                        1 => StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable,
                        _ => panic!("bytes module has exactly two intrinsic calls"),
                    };
                    let argument = call
                        .args()
                        .first()
                        .expect("intrinsic getter argument")
                        .flatten();
                    self.analysis.module_execution.intrinsics.insert(
                        std::ptr::from_ref(call.function().flatten()) as usize,
                        StandardBuiltinId::ReflectApply,
                    );
                    self.analysis
                        .module_execution
                        .intrinsics
                        .insert(std::ptr::from_ref(argument) as usize, getter);
                    self.calls += 1;
                }
                Expression::Assign(_) => self.stores += 1,
                _ => {}
            }
            expression.visit_with(self)
        }
    }
    let mut sites = Sites {
        analysis,
        constructors: 0,
        calls: 0,
        stores: 0,
    };
    let _ = body.visit_with(&mut sites);
    assert_eq!(
        sites.constructors, 2,
        "bytes module has two view constructors"
    );
    assert_eq!(
        sites.calls, 2,
        "bytes module has getter and immutable-transfer calls"
    );
    assert_eq!(
        sites.stores, bytes_len,
        "bytes module writes every loaded byte"
    );
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
