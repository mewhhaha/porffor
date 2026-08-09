use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::ControlFlow;
use std::panic::{self, AssertUnwindSafe};

use boa_ast::operations::{
    annex_b_function_declarations, contains, lexically_declared_names, ContainsSymbol,
};
use boa_ast::property::{MethodDefinitionKind, PropertyName};
use boa_ast::visitor::{VisitWith, Visitor};
use boa_ast::{
    declaration::{Binding, ExportDeclaration, LexicalDeclaration, VarDeclaration, Variable},
    expression::access::{
        PrivatePropertyAccess, PropertyAccess, PropertyAccessField, SuperPropertyAccess,
    },
    expression::literal::{
        ArrayLiteral, LiteralKind, ObjectLiteral, ObjectMethodDefinition, PropertyDefinition,
        TemplateElement, TemplateLiteral,
    },
    expression::operator::{
        assign::{AssignOp, AssignTarget},
        binary::{ArithmeticOp, BinaryInPrivate, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
        unary::UnaryOp,
        update::{UpdateOp, UpdateTarget},
    },
    expression::New,
    expression::{
        Call, Expression, Optional, OptionalOperationKind, RegExpLiteral, SuperCall, TaggedTemplate,
    },
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunctionDeclaration, AsyncFunctionExpression,
        AsyncGeneratorDeclaration, AsyncGeneratorExpression, ClassDeclaration, ClassElement,
        ClassElementName, ClassExpression, ClassMethodDefinition, FormalParameter,
        FormalParameterList, FunctionBody, FunctionDeclaration, FunctionExpression, PrivateName,
        StaticBlockBody,
    },
    function::{GeneratorDeclaration, GeneratorExpression},
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    scope::Scope,
    statement::{
        iteration::{
            Break as AstBreak, Continue as AstContinue, DoWhileLoop, ForLoop, ForLoopInitializer,
            ForOfLoop, IterableLoopInitializer, WhileLoop,
        },
        Block, If, Labelled as AstLabelled, LabelledItem, Return as AstReturn, Statement,
        Switch as AstSwitch, Throw as AstThrow, Try as AstTry,
    },
    Declaration, ModuleItem, Script, Spanned, StatementListItem,
};
use boa_interner::Interner;
use boa_parser::{Parser, Source};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, ToPrimitive, Zero};
use porffor_front::{ParseGoal, SourceUnit};
use regress::Regex;

mod analysis;
mod binding_names;
mod builtins;
mod diagnostics;
// The `EarlyErrorCode` -> rejection-stage map. UNRELATED to `early_errors`
// below, despite the adjacency: this is the diagnostic taxonomy, that is
// derived-constructor validation over `ExprIr` arms.
mod early_error_code;
mod early_errors;
mod ir;
mod iterator_obligations;
mod lowering;
mod lowering_helpers;
mod modules;
mod names;
mod native_error;
mod operations;
mod regexp;
mod well_known;
pub(crate) use analysis::*;
pub use builtins::{CallableToStringRepresentation, HostBuiltinId, StandardBuiltinId};
pub use diagnostics::{IrDiagnostic, IrDiagnosticKind, IrDiagnosticPhase, LoweringStage};
pub(crate) use early_errors::validate_derived_constructor_body;
pub use ir::*;
pub(crate) use ir::{read_heap_shape_property, summarize_block};
/// The iterator-protocol obligations of 7.4 and the witness a for-of
/// specialization carries. See
/// `docs/rust-rewrite/contracts/iterator-protocol.md`.
pub use iterator_obligations::{
    EmissionSite, GetIteratorDischarge, IntactnessPremise, IteratorCloseDischarge,
    IteratorObligation, IteratorProtocolWitness, IteratorStepDischarge, IteratorValueDischarge,
    ObligationDischarge, PremiseKind,
};
pub use lowering::{lower, lower_module_graph, lower_script_graph};
pub(crate) use lowering_helpers::*;
pub use modules::{
    evaluation_components, parse_module_record, scan_module_requests, source_writes_dynamic_import,
    DynamicComponentIr, DynamicImportSiteIr, ImportAttributeIr, ImportEntryIr, ImportNameIr,
    ImportPhaseIr, IndirectExportEntryIr, LinkedProgram, LocalExportEntryIr, ModuleBindingKindIr,
    ModuleBindingNameIr, ModuleEnvBindingIr, ModuleEvaluationModeIr, ModuleGraphIr,
    ModuleGraphSources, ModuleLinkErrorIr, ModuleNamespaceExportIr, ModuleNamespaceIr,
    ModuleRequestIr, ModuleSourceIr, ModuleUnitId, ModuleUnitIr, ResolvedBindingIr,
    SourceTextModuleRecordIr, StarExportEntryIr, ANONYMOUS_MODULE_KEY, MODULE_SOURCE_TO_STRING_TAG,
};
pub use operations::{
    completion_abi_slots, find_completion_abi_slot, find_spec_operation, spec_operation_catalog,
    ArithmeticBinaryOp, BindingMode, BitwiseBinaryOp, CompletionAbiSlot, CompletionAbruptKind,
    CompletionKindIr, CompletionRecordIr, DoneSlot, EcmaLanguageType, EmitterEvidence,
    EqualityBinaryOp, IteratorRecordIr, IteratorSlot, LogicalBinaryOp, NextMethodSlot,
    NormalResult, NumericUpdateOp, OperationLoweringStatus, OwnerTaskId, RelationalBinaryOp,
    RowSource, SpecOperationCatalogEntry, SpecOperationFamily, SpecOperationIr,
    StatementEmissionRow, ToPrimitiveHint, TrackedGapReason, TrackedGapRow, UnaryNumericOp,
    UpdateReturnMode, COMPLETION_ABI_SLOTS, SPEC_OPERATION_CATALOG, SPEC_OPERATION_ROW_COUNT,
    STATEMENT_EMISSION_ROWS, TRACKED_GAP_ROWS,
};
pub use regexp::{
    RegExpCompileError, RegExpCompileErrorKind, RegExpFlags, RegExpInstruction, RegExpNamedGroup,
    RegExpProgram, REGEXP_INSTRUCTION_WIDTH, REGEXP_OPCODE_ACCEPT, REGEXP_OPCODE_ASSERT_END,
    REGEXP_OPCODE_ASSERT_START, REGEXP_OPCODE_CAPTURE_END, REGEXP_OPCODE_CAPTURE_START,
    REGEXP_OPCODE_CLEAR_CAPTURE_RANGE, REGEXP_OPCODE_DOT, REGEXP_OPCODE_JUMP,
    REGEXP_OPCODE_LITERAL_ASCII, REGEXP_OPCODE_LITERAL_CODE_POINT, REGEXP_OPCODE_LOOKBEHIND_END,
    REGEXP_OPCODE_LOOKBEHIND_FAILURE, REGEXP_OPCODE_LOOKBEHIND_START,
    REGEXP_OPCODE_NAMED_BACKREFERENCE, REGEXP_OPCODE_NEGATIVE_ASCII_CLASS,
    REGEXP_OPCODE_NEGATIVE_ASCII_LOOKAHEAD, REGEXP_OPCODE_NOT_WHITESPACE,
    REGEXP_OPCODE_NUMBERED_BACKREFERENCE, REGEXP_OPCODE_POSITIVE_ASCII_CLASS,
    REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD, REGEXP_OPCODE_SPLIT, REGEXP_OPCODE_UNICODE_PROPERTY,
    REGEXP_OPCODE_WHITESPACE, REGEXP_RANGE_ENTRY_WIDTH,
};

pub use names::*;
pub(crate) use names::{
    MAX_ARRAY_INDEX, MAX_STATIC_ARRAY_SHAPE_INDEX, SCRIPT_OWNER_ID, TDZ_BINDING_STORAGE_PREFIX,
};

/// The three module binding-name domains. See
/// `docs/rust-rewrite/contracts/module-binding-names.md`.
pub use binding_names::*;
pub(crate) use binding_names::{
    DEFAULT_BINDING_ASSIGN, DEFAULT_BINDING_LET, DEFAULT_BINDING_VAR, DEFAULT_KEYWORD,
    EXPORT_KEYWORD, IMPORT_META_HEAD, IMPORT_META_TAIL,
};

/// The two closed spec name domains. See
/// `docs/rust-rewrite/contracts/closed-name-domains.md`.
pub use native_error::NativeErrorKind;

/// The closed domain of pre-evaluation rejection codes, re-exported from
/// `porffor-front` so consumers of `IrDiagnostic::code` have one path to it. See
/// `docs/rust-rewrite/contracts/early-error-taxonomy.md`.
pub use early_error_code::{EarlyErrorCode, ParseClassified};
pub use well_known::{
    is_symbol_description, shape_namespace_key, SymbolDescription, SymbolMemberName,
    WellKnownSymbol,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lowering::ScriptLowerer;
    use porffor_front::{parse, ParseOptions};

    fn lower_script(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        lower(&source)
    }

    fn lower_module(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::module()).expect("module should parse");
        lower(&source)
    }

    fn assert_zero_suspension_generator(function: &FunctionIr) {
        assert_eq!(function.execution_kind, FunctionExecutionKind::Generator);
        assert!(!function.constructable);
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

    fn collect_annex_b_copies(block: &BlockIr) -> Vec<(String, String, String)> {
        fn collect(statement: &StatementIr, copies: &mut Vec<(String, String, String)>) {
            match statement {
                StatementIr::AnnexBFunctionCopy {
                    source_name,
                    block_storage_name,
                    variable_storage_name,
                } => copies.push((
                    source_name.clone(),
                    block_storage_name.clone(),
                    variable_storage_name.clone(),
                )),
                StatementIr::LexicalBlock(statements)
                | StatementIr::ParameterInitialization { statements, .. } => {
                    for statement in statements {
                        collect(statement, copies);
                    }
                }
                StatementIr::Block(block) => {
                    for statement in &block.statements {
                        collect(statement, copies);
                    }
                }
                StatementIr::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    collect(then_branch, copies);
                    if let Some(else_branch) = else_branch {
                        collect(else_branch, copies);
                    }
                }
                StatementIr::While { body, .. }
                | StatementIr::DoWhile { body, .. }
                | StatementIr::For { body, .. }
                | StatementIr::ForOfArray { body, .. }
                | StatementIr::ForOfString { body, .. }
                | StatementIr::ForOfIterator { body, .. }
                | StatementIr::ForInArray { body, .. }
                | StatementIr::ForInString { body, .. }
                | StatementIr::ForInObject { body, .. }
                | StatementIr::Labelled {
                    statement: body, ..
                } => collect(body, copies),
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
                StatementIr::ModuleUnitOnce { block, .. } => {
                    for statement in &block.statements {
                        collect(statement, names);
                    }
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
                } => {
                    collect(then_branch, names);
                    if let Some(else_branch) = else_branch {
                        collect(else_branch, names);
                    }
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
                        }
                    }
                    collect(body, names);
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
                StatementIr::ForOfArray { name, body, .. }
                | StatementIr::ForOfString { name, body, .. }
                | StatementIr::ForOfIterator { name, body, .. }
                | StatementIr::ForInArray { name, body, .. }
                | StatementIr::ForInString { name, body, .. }
                | StatementIr::ForInObject { name, body, .. } => {
                    names.insert(name.clone());
                    collect(body, names);
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
                StatementIr::Expression(TypedExpr {
                    expr:
                        ExprIr::ArrayDestructure {
                            pattern,
                            assignment: false,
                            ..
                        },
                    ..
                }) => {
                    pattern.visit_bindings(&mut |_, name| {
                        names.insert(name.to_string());
                    });
                }
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ObjectDestructure { pattern, .. },
                    ..
                }) => {
                    pattern.visit_bindings(&mut |_, name| {
                        names.insert(name.to_string());
                    });
                }
                StatementIr::Empty
                | StatementIr::AnnexBFunctionCopy { .. }
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
                StatementIr::Block(block) => block_environment_owns_binding(block, name, slot),
                StatementIr::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    statement_owns_binding(then_branch, name, slot)
                        || else_branch
                            .as_deref()
                            .is_some_and(|branch| statement_owns_binding(branch, name, slot))
                }
                StatementIr::While { body, .. }
                | StatementIr::DoWhile { body, .. }
                | StatementIr::Labelled {
                    statement: body, ..
                } => statement_owns_binding(body, name, slot),
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
                StatementIr::ForOfArray {
                    body,
                    lexical_environment,
                    ..
                }
                | StatementIr::ForOfString {
                    body,
                    lexical_environment,
                    ..
                }
                | StatementIr::ForOfIterator {
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

    #[test]
    fn analysis_tracks_nested_block_environment_ownership() {
        with_script_analysis(
            "\"use strict\"; { let outer = 1; { const inner = 2; function read() { return outer + inner; } } }",
            |analysis| {
                let script_activation =
                    analysis.owner_plans[SCRIPT_OWNER_ID].activation_environment_id;
                let outer = environment_with_binding_suffix(analysis, ".outer");
                let inner = environment_with_binding_suffix(analysis, ".inner");

                assert_eq!(outer.kind, EnvironmentKind::Block);
                assert_eq!(
                    outer
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(script_activation)
                );
                assert_eq!(inner.kind, EnvironmentKind::Block);
                assert_eq!(
                    inner
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(outer.id)
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(outer, ".outer"),
                    outer,
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(inner, ".inner"),
                    inner,
                );

                let read = function_owner_plan_by_name(analysis, "read");
                assert_eq!(read.definition_environment_cursor.environment_id, inner.id);
            },
        );
    }

    #[test]
    fn analysis_shares_one_environment_for_switch_cases() {
        with_script_analysis(
            "switch (0) { case 0: let first = 1; break; default: const second = 2; }",
            |analysis| {
                let first = environment_with_binding_suffix(analysis, ".first");
                let second = environment_with_binding_suffix(analysis, ".second");
                let script_activation =
                    analysis.owner_plans[SCRIPT_OWNER_ID].activation_environment_id;

                assert_eq!(first.kind, EnvironmentKind::SwitchCaseBlock);
                assert_eq!(first.id, second.id);
                assert_eq!(analysis.switch_environment_ids.len(), 1);
                assert!(analysis
                    .switch_environment_ids
                    .values()
                    .any(|environment_id| environment_id == &first.id));
                assert_eq!(
                    first
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(script_activation)
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(first, ".first"),
                    first,
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(second, ".second"),
                    second,
                );
            },
        );
    }

    #[test]
    fn analysis_separates_try_catch_parameter_catch_body_and_finally_blocks() {
        with_script_analysis(
            "\"use strict\"; try { let tried = 1; } catch (error) { let handled = error; function reader() { return error + handled; } } finally { const cleaned = 1; }",
            |analysis| {
                let script_activation =
                    analysis.owner_plans[SCRIPT_OWNER_ID].activation_environment_id;
                let tried = environment_with_binding_suffix(analysis, ".tried");
                let error = environment_with_binding_suffix(analysis, ".error");
                let handled = environment_with_binding_suffix(analysis, ".handled");
                let cleaned = environment_with_binding_suffix(analysis, ".cleaned");

                assert_eq!(tried.kind, EnvironmentKind::Block);
                assert_eq!(error.kind, EnvironmentKind::CatchParameter);
                assert_eq!(handled.kind, EnvironmentKind::Block);
                assert_eq!(cleaned.kind, EnvironmentKind::Block);
                assert_eq!(analysis.catch_parameter_environment_ids.len(), 1);
                assert!(
                    analysis
                        .catch_parameter_environment_ids
                        .values()
                        .any(|environment_id| environment_id == &error.id)
                );
                for environment in [tried, handled, cleaned] {
                    assert!(
                        analysis
                            .block_environment_ids
                            .values()
                            .any(|environment_id| environment_id == &environment.id)
                    );
                }
                for environment in [tried, error, cleaned] {
                    assert_eq!(
                        environment
                            .parent_cursor
                            .as_ref()
                            .map(|cursor| cursor.environment_id),
                        Some(script_activation)
                    );
                }
                assert_eq!(
                    handled
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(error.id)
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(tried, ".tried"),
                    tried,
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(error, ".error"),
                    error,
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(handled, ".handled"),
                    handled,
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(cleaned, ".cleaned"),
                    cleaned,
                );

                let reader = function_owner_plan_by_name(analysis, "reader");
                assert_eq!(
                    reader.definition_environment_cursor.environment_id,
                    handled.id
                );
            },
        );
    }

    #[test]
    fn analysis_tracks_classic_for_lexical_head_environment() {
        with_script_analysis(
            "\"use strict\"; for (let index = 0; index < 1; index++) { const value = index; function read() { return index + value; } }",
            |analysis| {
                let script_activation =
                    analysis.owner_plans[SCRIPT_OWNER_ID].activation_environment_id;
                let index = environment_with_binding_suffix(analysis, ".index");
                let value = environment_with_binding_suffix(analysis, ".value");

                assert_eq!(index.kind, EnvironmentKind::ForLexicalHead);
                assert_eq!(value.kind, EnvironmentKind::Block);
                assert!(
                    analysis
                        .for_lexical_environment_ids
                        .values()
                        .any(|environment_id| *environment_id == index.id)
                );
                assert_eq!(
                    index
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(script_activation)
                );
                assert_eq!(
                    value
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(index.id)
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(index, ".index"),
                    index,
                );
                assert_physical_binding_owner(
                    analysis,
                    binding_with_suffix(value, ".value"),
                    value,
                );

                let read = function_owner_plan_by_name(analysis, "read");
                assert_eq!(read.definition_environment_cursor.environment_id, value.id);
            },
        );
    }

    #[test]
    fn analysis_tracks_for_in_and_for_of_tdz_and_iteration_environments() {
        with_script_analysis(
            "\"use strict\"; for (let property in property) { const inRead = () => property; } for (const value of value) { const ofRead = () => value; }",
            |analysis| {
                let script_activation =
                    analysis.owner_plans[SCRIPT_OWNER_ID].activation_environment_id;
                let property_tdz = analysis
                    .environment_plans
                    .values()
                    .find(|environment| {
                        environment.kind == EnvironmentKind::ForInOfTdzHead
                            && environment.binding_storage_names.contains("$tdz.property")
                    })
                    .expect("for-in TDZ environment should be planned");
                let property_iteration = analysis
                    .environment_plans
                    .values()
                    .find(|environment| {
                        environment.kind == EnvironmentKind::ForInOfIteration
                            && environment.binding_storage_names.iter().any(|binding| {
                                binding.starts_with("$forin.lex.") && binding.ends_with(".property")
                            })
                    })
                    .expect("for-in iteration environment should be planned");
                let value_tdz = analysis
                    .environment_plans
                    .values()
                    .find(|environment| {
                        environment.kind == EnvironmentKind::ForInOfTdzHead
                            && environment.binding_storage_names.contains("$tdz.value")
                    })
                    .expect("for-of TDZ environment should be planned");
                let value_iteration = analysis
                    .environment_plans
                    .values()
                    .find(|environment| {
                        environment.kind == EnvironmentKind::ForInOfIteration
                            && environment.binding_storage_names.iter().any(|binding| {
                                binding.starts_with("$forof.lex.") && binding.ends_with(".value")
                            })
                    })
                    .expect("for-of iteration environment should be planned");

                for environment in [property_tdz, value_tdz] {
                    assert!(
                        analysis
                            .for_in_of_tdz_environment_ids
                            .values()
                            .any(|environment_id| *environment_id == environment.id)
                    );
                }
                for environment in [property_iteration, value_iteration] {
                    assert!(
                        analysis
                            .for_in_of_iteration_environment_ids
                            .values()
                            .any(|environment_id| *environment_id == environment.id)
                    );
                }

                assert_eq!(
                    property_iteration
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(script_activation)
                );
                assert_eq!(
                    property_tdz
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(script_activation)
                );
                assert_eq!(
                    value_iteration
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(script_activation)
                );
                assert_eq!(
                    value_tdz
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| cursor.environment_id),
                    Some(script_activation)
                );
                assert_physical_binding_owner(analysis, "$tdz.property", property_tdz);
                assert_physical_binding_owner(analysis, "$tdz.value", value_tdz);
                let property_binding = property_iteration
                    .binding_storage_names
                    .iter()
                    .find(|binding| {
                        binding.starts_with("$forin.lex.") && binding.ends_with(".property")
                    })
                    .expect("for-in iteration should own its physical binding");
                let value_binding = value_iteration
                    .binding_storage_names
                    .iter()
                    .find(|binding| {
                        binding.starts_with("$forof.lex.") && binding.ends_with(".value")
                    })
                    .expect("for-of iteration should own its physical binding");
                assert_physical_binding_owner(analysis, property_binding, property_iteration);
                assert_physical_binding_owner(analysis, value_binding, value_iteration);
            },
        );
    }

    #[test]
    fn analysis_stamps_root_and_block_function_definition_cursors() {
        with_script_analysis(
            "\"use strict\"; function root(parameter) { var variable = parameter; } { function nested() {} }",
            |analysis| {
                let script_activation =
                    analysis.owner_plans[SCRIPT_OWNER_ID].activation_environment_id;
                let nested_environment = analysis
                    .environment_plans
                    .values()
                    .find(|environment| {
                        environment.kind == EnvironmentKind::Block
                            && environment
                                .binding_storage_names
                                .iter()
                                .any(|binding| binding.ends_with(".nested"))
                    })
                    .expect("nested function block should be planned");
                let root = function_owner_plan_by_name(analysis, "root");
                let root_activation = &analysis.environment_plans[&root.activation_environment_id];

                assert_eq!(
                    root.definition_environment_cursor.environment_id,
                    script_activation
                );
                assert_eq!(root_activation.kind, EnvironmentKind::Activation);
                assert_eq!(
                    root_activation
                        .parent_cursor
                        .as_ref()
                        .map(|cursor| (cursor.owner_id.as_str(), cursor.environment_id)),
                    Some((SCRIPT_OWNER_ID, script_activation))
                );
                assert!(root_activation.binding_storage_names.contains("parameter"));
                assert!(root_activation.binding_storage_names.contains("variable"));
                assert_physical_binding_owner(analysis, "parameter", root_activation);
                assert_physical_binding_owner(analysis, "variable", root_activation);
                assert_eq!(
                    function_owner_plan_by_name(analysis, "nested")
                        .definition_environment_cursor
                        .environment_id,
                    nested_environment.id
                );
            },
        );
    }

    #[test]
    fn lowers_simple_script_ir() {
        let program = lower_script("let x = 40; const y = 2; x + y;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.body.statements.len(), 3);
        assert_eq!(script.result_kind(), ValueKind::Number);
    }

    #[test]
    fn lowers_no_import_module_export_declaration() {
        let program = lower_module("export const value = 1; value;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
    }

    /// A single-source `lower` has no host loader, so it resolves no specifier.
    /// The honest report is that the *request* did not resolve, not that imports
    /// are unsupported — `modules::link` links them once a loader supplies the
    /// dependency. The assertion previously looked for the string
    /// "module imports", which no diagnostic in the crate has ever produced.
    #[test]
    fn rejects_an_import_whose_specifier_no_loader_resolved() {
        let program = lower_module("import value from './dep.js'; value;");
        assert!(!program.is_wasm_supported());
        assert!(
            program.diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("unresolved module request")
                && diagnostic.message.contains("./dep.js")),
            "got {:?}",
            program.diagnostics
        );
    }

    #[test]
    fn allows_non_prototype_proto_property_forms() {
        let program = lower_script(r#"({ __proto__() { return 1; }, ["__proto__"]: 2 });"#);
        assert!(
            program.diagnostics.iter().all(|diagnostic| {
                diagnostic.code() != Some(EarlyErrorCode::ObjectDuplicateProto)
            }),
            "diagnostics: {:?}",
            program.diagnostics
        );
    }

    #[test]
    fn lowers_assignment_and_if_ir() {
        let program = lower_script("let x = 0; if (!x) { x = 5; } x;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
        assert!(matches!(script.body.statements[1], StatementIr::If { .. }));
        assert!(program.ir_summary().contains("ifs=1"));
        assert!(program.ir_summary().contains("assigns=1"));
    }

    #[test]
    fn lowers_object_keys_join_before_control_as_runtime_property_call() {
        let program =
            lower_script(r#"var o = { a: 1 }; Object.keys(o).join(""); if (false) {} 1;"#);
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.body.statements.iter().any(|statement| {
            let StatementIr::Expression(expression) = statement else {
                return false;
            };
            matches!(
                indirect_call_body(expression).map(|call| &call.expr),
                Some(ExprIr::CallIndirect { callee, .. }) if matches!(
                    &callee.expr,
                    ExprIr::SpecOperation {
                        operation: SpecOperationIr::GetV,
                        operands,
                    } if operands.len() == 2
                        && matches!(&operands[1].expr, ExprIr::String(name) if name == "join")
                )
            )
        }));
    }

    #[test]
    fn lowers_mutable_array_prototype_method_call_through_runtime_getv() {
        let program = lower_script(
            "var alias = Array.prototype; alias.join = function () { return 'alias'; }; Array.prototype.join();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.body.statements.iter().any(|statement| {
            let StatementIr::Expression(expression) = statement else {
                return false;
            };
            matches!(
                indirect_call_body(expression).map(|call| &call.expr),
                Some(ExprIr::CallIndirect { callee, .. }) if matches!(
                    &callee.expr,
                    ExprIr::SpecOperation {
                        operation: SpecOperationIr::GetV,
                        operands,
                    } if operands.len() == 2
                        && matches!(&operands[1].expr, ExprIr::String(name) if name == "join")
                )
            )
        }));
    }

    #[test]
    fn lowers_copied_mutable_array_prototype_method_as_indirect_call() {
        let program = lower_script("var obj = {}; obj.join = Array.prototype.join; obj.join();");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.body.statements.iter().any(|statement| {
            let StatementIr::Expression(expression) = statement else {
                return false;
            };
            indirect_call_body(expression)
                .is_some_and(|call| call.possible_kinds == KindSet::all_runtime_tags())
        }));
    }

    #[test]
    fn preserves_call_spreads_in_source_argument_order() {
        let program = lower_script(
            "function collect() {} let values = [2, 3]; collect(42, ...[1], ...values,);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(call) = script.body.statements.last().expect("call expression")
        else {
            panic!("expected call expression");
        };
        let args = match &call.expr {
            ExprIr::CallNamed { args, .. } | ExprIr::CallIndirect { args, .. } => args,
            other => panic!("expected function call, got {other:?}"),
        };

        assert_eq!(args.len(), 3);
        assert!(matches!(args[0].expr, ExprIr::Number(_)));
        assert!(matches!(
            args[1].expr,
            ExprIr::SpreadArgument(ref value)
                if matches!(value.expr, ExprIr::ArrayLiteral(_))
        ));
        assert!(matches!(
            args[2].expr,
            ExprIr::SpreadArgument(ref value)
                if matches!(value.expr, ExprIr::Identifier(ref name) if name == "values")
        ));
    }

    #[test]
    fn lowers_computed_array_subclass_method_through_runtime_getv() {
        let program = lower_script(
            "class Derived extends Array { push() { return 'derived'; } } var key = 'push'; new Derived()[key]();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let Some(StatementIr::Expression(final_expr)) = script.body.statements.last() else {
            panic!("expected final expression statement");
        };
        let call = indirect_call_body(final_expr).expect("expected materialized indirect call");
        let ExprIr::CallIndirect { callee, .. } = &call.expr else {
            panic!("expected indirect call, got {:?}", final_expr.expr);
        };
        let ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } = &callee.expr
        else {
            panic!(
                "expected GetV callee, got {:?} with targets {:?}",
                callee.expr, callee.function_targets
            );
        };
        assert_eq!(operands.len(), 2);
        assert!(
            matches!(
                &operands[1].expr,
                ExprIr::String(name) if name == "push"
            ) || matches!(
                &operands[1].expr,
                ExprIr::GlobalPropertyRead { name } if name == "key"
            ),
            "unexpected key operand: {:?}",
            operands[1]
        );
    }

    #[test]
    fn lowers_loop_ir() {
        let program = lower_script("let i = 0; while (i < 3) { i = i + 1; continue; } i;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(
            script.body.statements[1],
            StatementIr::While { .. }
        ));
        assert!(program.ir_summary().contains("whiles=1"));
        assert!(program.ir_summary().contains("continues=1"));
    }

    #[test]
    fn lowers_array_spread_with_concat_result_element_shape() {
        let program =
            lower_script("let source = [17, NaN, 'tail']; let copy = [...source]; copy[1];");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical {
            name,
            init: copy_init,
            ..
        } = &script.body.statements[1]
        else {
            panic!("expected spread copy declaration");
        };
        assert_eq!(name, "copy");
        let Some(shape) = copy_init.heap_shape.as_deref() else {
            panic!("spread concat result must retain its array shape");
        };
        let HeapShape::Array(shape) = shape else {
            panic!("spread concat result must retain an array shape");
        };
        assert_eq!(shape.elements.len(), 3);
        assert_eq!(shape.elements[0].kind, ValueKind::Number);
        assert_eq!(shape.elements[1].kind, ValueKind::Number);
        assert_eq!(shape.elements[2].kind, ValueKind::String);

        let StatementIr::Expression(read) = &script.body.statements[2] else {
            panic!("expected spread copy element read");
        };
        assert_eq!(read.kind, ValueKind::Dynamic);
    }

    #[test]
    fn lowers_array_spread_with_unshaped_concat_input_as_dynamic_elements() {
        let program =
            lower_script("let source = [].concat({ length: 1 }); let copy = [...source]; copy[0];");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical {
            name,
            init: copy_init,
            ..
        } = &script.body.statements[1]
        else {
            panic!("expected spread copy declaration");
        };
        assert_eq!(name, "copy");
        assert!(
            copy_init.heap_shape.is_none(),
            "an unshaped concat input must not become an empty array shape"
        );

        let StatementIr::Expression(read) = &script.body.statements[2] else {
            panic!("expected spread copy element read");
        };
        assert_eq!(read.kind, ValueKind::Dynamic);
    }

    #[test]
    fn concat_discards_element_shape_after_array_length_write() {
        let program = lower_script(
            "let source = [17, NaN, 'tail']; source.length = 1; let copy = [].concat(source); copy[1];",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical {
            name,
            init: copy_init,
            ..
        } = &script.body.statements[2]
        else {
            panic!("expected concat copy declaration");
        };
        assert_eq!(name, "copy");
        assert!(
            copy_init.heap_shape.is_none(),
            "a length-mutated source must not retain stale concat elements"
        );

        let StatementIr::Expression(read) = &script.body.statements[3] else {
            panic!("expected concat copy element read");
        };
        assert_eq!(read.kind, ValueKind::Dynamic);
    }

    #[test]
    fn concat_discards_element_shape_for_custom_spreadability() {
        let program = lower_script(
            "let source = [17]; source[Symbol.isConcatSpreadable] = false; let copy = [].concat(source); copy[0];",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical {
            name,
            init: copy_init,
            ..
        } = &script.body.statements[2]
        else {
            panic!("expected concat copy declaration");
        };
        assert_eq!(name, "copy");
        assert!(
            copy_init.heap_shape.is_none(),
            "@@isConcatSpreadable must make concat element layout dynamic"
        );

        let StatementIr::Expression(read) = &script.body.statements[3] else {
            panic!("expected concat copy element read");
        };
        assert_eq!(read.kind, ValueKind::Dynamic);
    }

    #[test]
    fn concat_discards_element_shape_for_holey_arrays() {
        let program = lower_script("let source = [, 17]; let copy = [].concat(source); copy[0];");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical {
            init: copy_init, ..
        } = &script.body.statements[1]
        else {
            panic!("expected concat copy declaration");
        };
        assert!(
            copy_init.heap_shape.is_none(),
            "a hole must not be confused with an explicit undefined element"
        );

        let StatementIr::Expression(read) = &script.body.statements[2] else {
            panic!("expected concat copy element read");
        };
        assert_eq!(read.kind, ValueKind::Dynamic);
    }

    #[test]
    fn flat_does_not_reuse_unflattened_receiver_shape() {
        let program =
            lower_script("let nested = [[17]]; let flattened = nested.flat(); flattened[0];");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical {
            name,
            init: flattened_init,
            ..
        } = &script.body.statements[1]
        else {
            panic!("expected flat result declaration");
        };
        assert_eq!(name, "flattened");
        assert!(
            flattened_init.heap_shape.is_none(),
            "flat must not report the receiver's nested element shape"
        );

        let StatementIr::Expression(read) = &script.body.statements[2] else {
            panic!("expected flat result element read");
        };
        assert_eq!(read.kind, ValueKind::Dynamic);
    }

    #[test]
    fn flat_map_result_elements_remain_dynamic() {
        let program = lower_script(
            "let source = [1, 2]; let mapped = source.flatMap(function (value) { return [value, value]; }); mapped[0];",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical {
            name,
            init: mapped_init,
            ..
        } = &script.body.statements[1]
        else {
            panic!("expected flatMap result declaration");
        };
        assert_eq!(name, "mapped");
        assert!(
            mapped_init.heap_shape.is_none(),
            "flatMap output cardinality and element layout are runtime-dependent"
        );

        let StatementIr::Expression(read) = &script.body.statements[2] else {
            panic!("expected flatMap result element read");
        };
        assert_eq!(read.kind, ValueKind::Dynamic);
    }

    #[test]
    fn species_capable_array_results_preserve_runtime_object_tags() {
        for (source, result_statement) in [
            ("let result = [1].flat();", 0),
            ("let result = [1].slice();", 0),
            (
                "let result = [1].flatMap(function (value) { return value; });",
                0,
            ),
            ("let source = []; let result = source.concat(1);", 1),
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Lexical { init, .. } = &script.body.statements[result_statement]
            else {
                panic!("expected result declaration for {source}");
            };
            assert_eq!(init.kind, ValueKind::Dynamic, "{source}");
            assert_eq!(init.possible_kinds, KindSet::all_runtime_tags(), "{source}");
            assert!(init.heap_shape.is_none(), "{source}");
        }
    }

    #[test]
    fn concat_with_proven_array_layout_remains_runtime_dynamic() {
        let program = lower_script("let result = [1].concat([2]);");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
            panic!("expected concat result declaration");
        };
        assert_eq!(init.kind, ValueKind::Dynamic);
        assert_eq!(init.possible_kinds, KindSet::all_runtime_tags());
        assert!(init.heap_shape.is_none());
    }

    #[test]
    fn lowers_for_multi_binding_lexical_init_ir() {
        let program = lower_script("for (let i = 0, j = 1; i < 1; i = i + 1) { j; }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::For {
            init: Some(ForInitIr::LexicalBlock(bindings)),
            ..
        } = &script.body.statements[0]
        else {
            panic!("expected multi-binding lexical for initializer");
        };
        assert_eq!(bindings.len(), 2);
        assert!(program.ir_summary().contains("lets=2"));
    }

    #[test]
    fn lowers_update_and_compound_ir() {
        let program = lower_script("let i = 2; let x = i++; x += ++i; x;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
        let summary = program.ir_summary();
        assert!(summary.contains("postfix_updates=1"));
        assert!(summary.contains("prefix_updates=1"));
        assert!(summary.contains("compound_assigns=1"));
    }

    #[test]
    fn lowers_numeric_update_for_dynamically_typed_binding() {
        let program = lower_script(
            "function visitRange(start, end) { for (let codePoint = start; codePoint <= end; codePoint++) {} }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(program.ir_summary().contains("postfix_updates=1"));
    }

    #[test]
    fn lowers_property_update_after_generic_get_v() {
        let program = lower_script(
            "var count = -1; function increment() { this.count++; } Array.from([0], increment, this);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(program.ir_summary().contains("postfix_updates=1"));
    }

    #[test]
    fn lowers_string_compound_add_with_dynamic_rhs_ir() {
        let program = lower_script(
            r#"let result = String.fromCodePoint.apply(null, [65]); result += String.fromCodePoint.apply(null, [66]); result;"#,
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::String);
        assert!(program.ir_summary().contains("compound_assigns=1"));
    }

    #[test]
    fn lowers_coercive_compound_add_in_reduce_callback() {
        let program = lower_script(
            "function callback(accumulator, value) { accumulator += value; return accumulator; } [11, 9].reduceRight(callback, 0);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(program.ir_summary().contains("heap_coercions=1"));
    }

    #[test]
    fn lowers_dynamic_primitive_ir() {
        let program = lower_script(
            "let x = 0; x || \"fallback\"; null ?? 3; typeof missingName; \"a\" + \"b\";",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("string_concats=1"));
        assert!(summary.contains("typeof_uses=1"));
        assert!(summary.contains("nullish_ops=1"));
    }

    #[test]
    fn operations_lowers_boolean_call_to_to_boolean_spec_operation() {
        let program = lower_script("Boolean(globalThis.flag);");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=2"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected Boolean call to lower to a spec operation");
        };
        assert_eq!(*operation, SpecOperationIr::ToBoolean);
        assert_eq!(operands.len(), 1);
        assert!(matches!(
            operands[0].expr,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                ..
            }
        ));
    }

    #[test]
    fn operations_lowers_host_is_constructor_call_to_spec_operation() {
        let program = lower_script("let value = function C() {}; __porfIsConstructor(value);");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected __porfIsConstructor call to lower to a spec operation");
        };
        assert_eq!(*operation, SpecOperationIr::IsConstructor);
        assert_eq!(operands.len(), 1);
    }

    #[test]
    fn operations_lowers_number_call_to_to_number_spec_operation() {
        let program = lower_script("let value = \"42\"; Number(value);");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected spec operation");
        };
        assert_eq!(*operation, SpecOperationIr::ToNumber);
        assert_eq!(operands.len(), 1);
    }

    #[test]
    fn operations_keeps_bigint_number_call_off_to_number_spec_operation() {
        let program = lower_script("let value = 1n; Number(value);");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        assert!(!matches!(
            expr.expr,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToNumber,
                ..
            }
        ));
    }

    #[test]
    fn operations_keeps_number_call_with_reassigned_parameter_off_to_number_spec_operation() {
        let program = lower_script(
            "function convert(value) { value = Number(value); return value; } convert(1); convert(1n);",
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script.functions.first().expect("convert function");
        let StatementIr::Expression(expr) = &function.body.statements[0] else {
            panic!("expected assignment expression statement");
        };
        let ExprIr::AssignIdentifier { value, .. } = &expr.expr else {
            panic!("expected identifier assignment");
        };
        assert!(!matches!(
            value.expr,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToNumber,
                ..
            }
        ));
    }

    #[test]
    fn array_sort_call_keeps_comparator_parameters_dynamic() {
        let program = lower_script(
            "function compare(a, b) { a = Number(a); b = Number(b); return a - b; } const values = new BigInt64Array([2n, 1n]); Array.prototype.sort.call(values, compare);",
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let compare = script
            .functions
            .iter()
            .find(|function| function.name == "compare")
            .expect("compare function");

        assert!(compare
            .params
            .iter()
            .all(|param| param.kind == ValueKind::Dynamic));
    }

    #[test]
    fn arithmetic_with_a_bigint_operand_preserves_its_only_normal_result_kind() {
        let program = lower_script(
            "function decrement(value) { return value - 1n; } decrement(globalThis.value);",
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let decrement = script
            .functions
            .iter()
            .find(|function| function.name == "decrement")
            .expect("decrement function");
        let StatementIr::Return(expr) = &decrement.body.statements[0] else {
            panic!("expected return statement");
        };

        assert_eq!(expr.kind, ValueKind::BigInt);
        assert_eq!(expr.possible_kinds, KindSet::from_kind(ValueKind::BigInt));
        assert!(matches!(expr.expr, ExprIr::CoerciveBinaryNumber { .. }));
    }

    #[test]
    fn bigint_literal_ir_preserves_arbitrary_precision_decimal() {
        let program = lower_script("184467440737095516161234567890n;");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::BigInt(value) = &expr.expr else {
            panic!("expected BigInt literal");
        };

        assert_eq!(value.decimal, "184467440737095516161234567890");
        assert!(value.requires_arbitrary_precision_storage);
        assert_eq!(
            value.wrapping_payload(),
            184467440737095516161234567890_u128 as u64
        );
    }

    #[test]
    fn bigint_literal_ir_keeps_signed_minimum_in_immediate_storage() {
        let program = lower_script("-0x8000000000000000n;");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::BigInt(value) = &expr.expr else {
            panic!("expected BigInt literal");
        };

        assert_eq!(value.decimal, i64::MIN.to_string());
        assert!(!value.requires_arbitrary_precision_storage);
        assert_eq!(value.wrapping_payload(), i64::MIN as u64);
    }

    #[test]
    fn bigint_constant_fold_uses_arbitrary_precision_decimal() {
        let program = lower_script("184467440737095516161234567890n + 10n;");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::BigInt(value) = &expr.expr else {
            panic!("expected folded BigInt literal");
        };

        assert_eq!(value.decimal, "184467440737095516161234567900");
        assert!(value.requires_arbitrary_precision_storage);
    }

    #[test]
    fn operations_lowers_string_call_to_to_string_spec_operation() {
        let program = lower_script("let value = 42; String(value);");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected spec operation");
        };
        assert_eq!(*operation, SpecOperationIr::ToString);
        assert_eq!(operands.len(), 1);
    }

    #[test]
    fn operations_lowers_strict_equality_to_spec_operation() {
        let program = lower_script("let value = 1; value === 1;");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected strict equality to lower to a spec operation");
        };
        assert_eq!(*operation, SpecOperationIr::StrictEqualityComparison);
        assert_eq!(operands.len(), 2);
    }

    #[test]
    fn operations_lowers_strict_not_equal_to_logical_not_spec_operation() {
        let program = lower_script("let value = 1; value !== 2;");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::LogicalNot { expr } = &expr.expr else {
            panic!("expected strict not equal to lower to logical not");
        };
        assert!(matches!(
            expr.expr,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::StrictEqualityComparison,
                ..
            }
        ));
    }

    #[test]
    fn operations_lowers_loose_equality_to_spec_operation() {
        let program = lower_script("let value = 1; value == \"1\";");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected loose equality to lower to a spec operation");
        };
        assert_eq!(*operation, SpecOperationIr::IsLooselyEqual);
        assert_eq!(operands.len(), 2);
    }

    #[test]
    fn operations_lowers_loose_not_equal_to_logical_not_spec_operation() {
        let program = lower_script("let value = 1; value != \"2\";");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::LogicalNot { expr } = &expr.expr else {
            panic!("expected loose not equal to lower to logical not");
        };
        assert!(matches!(
            expr.expr,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::IsLooselyEqual,
                ..
            }
        ));
    }

    #[test]
    fn operations_lowers_object_is_to_same_value_spec_operation() {
        let program = lower_script("Object.is(NaN, NaN);");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected Object.is to lower to a spec operation");
        };
        assert_eq!(*operation, SpecOperationIr::SameValue);
        assert_eq!(operands.len(), 2);
    }

    #[test]
    fn operations_lowers_generic_object_property_read_to_get_v_spec_operation() {
        let program = lower_script("let object = {}; object.missing;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("spec_operations=1"));
        assert!(summary.contains("property_reads=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected generic property read to lower to GetV");
        };
        assert_eq!(*operation, SpecOperationIr::GetV);
        assert_eq!(operands.len(), 2);
    }

    #[test]
    fn operations_lowers_in_operator_to_has_property_spec_operation() {
        let program = lower_script("let object = {}; \"missing\" in object;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected in operator to lower to HasProperty");
        };
        assert_eq!(*operation, SpecOperationIr::HasProperty);
        assert_eq!(operands.len(), 2);
    }

    #[test]
    fn preserves_function_or_for_htmldda_truthiness() {
        let program = lower_script("function f() {} f || 2;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        assert!(matches!(
            expr.expr,
            ExprIr::LogicalShortCircuit {
                op: LogicalBinaryOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn lowers_identifier_logical_assignment_ir() {
        let program = lower_script("let value = 0; value ||= 2; value &&= 3; value ??= 4;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        for statement in &script.body.statements[1..] {
            let StatementIr::Expression(expr) = statement else {
                panic!("expected logical assignment expression statement");
            };
            let ExprIr::AssignIdentifier { value, .. } = &expr.expr else {
                panic!("expected identifier assignment");
            };
            assert!(matches!(value.expr, ExprIr::LogicalShortCircuit { .. }));
        }
    }

    #[test]
    fn lowers_simple_destructuring_assignment_patterns() {
        let program = lower_script(
            "var x; var base = {}; ([x = 1, base.y = 3] = [2, 4]); ({x = 5} = {x: 6});",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::ArrayDestructure { pattern, .. },
            ..
        }) = &script.body.statements[2]
        else {
            panic!("expected semantic array destructuring assignment");
        };
        assert!(matches!(
            pattern.elements[1],
            ArrayDestructuringElementIr::Target {
                target: DestructuringTargetIr::AssignmentProperty { .. },
                ..
            }
        ));
    }

    #[test]
    fn lowers_simple_destructuring_lexical_bindings() {
        let program = lower_script("const [x = 1] = [2]; const { y = 3 } = { y: 4 }; x + y;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(
            script.body.statements[0],
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ArrayDestructure {
                    assignment: false,
                    ..
                },
                ..
            })
        ));
        assert!(matches!(
            script.body.statements[1],
            StatementIr::LexicalBlock(_)
        ));
    }

    #[test]
    fn lowers_object_destructuring_rhs_once_before_ordered_property_reads() {
        let program = lower_script(
            "let { first: renamed = 10, second, missing, } = source(); renamed + second + missing;",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::LexicalBlock(statements) = &script.body.statements[0] else {
            panic!("expected destructuring lexical block");
        };
        assert_eq!(statements.len(), 4);

        let StatementIr::Lexical {
            mode: BindingMode::Let,
            name: temporary_name,
            init,
        } = &statements[0]
        else {
            panic!("expected RHS materialization");
        };
        assert!(temporary_name.starts_with("$destructure.internal."));
        assert!(matches!(
            init.expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));

        for (statement, expected_name, expected_key) in [
            (&statements[1], "renamed", "first"),
            (&statements[2], "second", "second"),
            (&statements[3], "missing", "missing"),
        ] {
            let StatementIr::Lexical { name, init, .. } = statement else {
                panic!("expected lexical property binding");
            };
            assert_eq!(name, expected_name);
            let value = if expected_name == "renamed" {
                let ExprIr::Conditional { else_expr, .. } = &init.expr else {
                    panic!("expected default initializer");
                };
                else_expr
            } else {
                init
            };
            let ExprIr::PropertyRead { target, key } = &value.expr else {
                panic!("expected property read");
            };
            assert!(matches!(target.expr, ExprIr::Identifier(ref name) if name == temporary_name));
            assert_eq!(key, &PropertyKeyIr::StaticString(expected_key.to_string()));
        }
    }

    #[test]
    fn materializes_literal_array_before_pattern_defaults_and_bindings() {
        let program =
            lower_script("let [, selected = fallback()] = [leading(), , trailing()]; selected;");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(TypedExpr {
            expr:
                ExprIr::ArrayDestructure {
                    value,
                    pattern,
                    assignment: false,
                },
            ..
        }) = &script.body.statements[0]
        else {
            panic!("expected semantic array destructuring expression");
        };
        let ExprIr::ArrayLiteral(elements) = &value.expr else {
            panic!("expected the complete literal array RHS");
        };
        assert_eq!(elements.len(), 3);
        assert!(matches!(
            elements[0].expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));
        assert!(matches!(elements[1].expr, ExprIr::ArrayHole));
        assert!(matches!(
            elements[2].expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));
        assert!(matches!(
            pattern.elements[0],
            ArrayDestructuringElementIr::Elision
        ));
        let ArrayDestructuringElementIr::Target {
            target: DestructuringTargetIr::Binding { name, .. },
            default: Some(default),
        } = &pattern.elements[1]
        else {
            panic!("expected selected binding with a default initializer");
        };
        assert_eq!(name, "selected");
        assert!(matches!(
            default.expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));
    }

    #[test]
    fn materializes_literal_array_assignment_before_target_writes() {
        let program = lower_script(
            "var selected; var result = ([, selected] = [leading(), chosen(), trailing()]); result;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Var(declarators) = &script.body.statements[1] else {
            panic!("expected result declaration");
        };
        let init = declarators[0]
            .init
            .as_ref()
            .expect("result should have an initializer");
        let ExprIr::ArrayDestructure {
            value,
            pattern,
            assignment: true,
        } = &init.expr
        else {
            panic!("expected semantic array assignment");
        };
        let ExprIr::ArrayLiteral(elements) = &value.expr else {
            panic!("expected the complete literal array RHS");
        };
        assert_eq!(elements.len(), 3);
        assert!(matches!(
            pattern.elements[0],
            ArrayDestructuringElementIr::Elision
        ));
        assert!(matches!(
            pattern.elements[1],
            ArrayDestructuringElementIr::Target {
                target: DestructuringTargetIr::AssignmentIdentifier { .. },
                ..
            }
        ));
    }

    #[test]
    fn lowers_for_of_array_assignment_pattern_as_iteration_prefix() {
        let program = lower_script("var x; for ([x] of [[1]]) {}");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::ForOfArray { body, .. } = &script.body.statements[1] else {
            panic!("expected array for-of statement");
        };
        let StatementIr::Block(block) = body.as_ref() else {
            panic!("expected assignment prefix block");
        };
        assert!(matches!(
            block.statements[0],
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ArrayDestructure {
                    assignment: true,
                    ..
                },
                ..
            })
        ));
    }

    #[test]
    fn lowers_private_loop_heads_as_per_iteration_writes() {
        let program = lower_script(
            "class C {
                #value;
                assign() {
                    for (this.#value of [1, 2]) {}
                    for (this.#value in { first: 1, second: 2 }) {}
                }
            }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "C.assign")
            .expect("assign method should be lowered");

        let mut private_loop_count = 0;
        for statement in &function.body.statements {
            let body = match statement {
                StatementIr::ForOfArray { body, .. } | StatementIr::ForInObject { body, .. } => {
                    body
                }
                _ => continue,
            };
            private_loop_count += 1;
            let StatementIr::Block(block) = body.as_ref() else {
                panic!("private loop head should add an iteration prefix");
            };
            assert!(matches!(
                block.statements.first(),
                Some(StatementIr::Expression(TypedExpr {
                    expr: ExprIr::PrivateWrite { .. },
                    ..
                }))
            ));
        }
        assert_eq!(private_loop_count, 2);
    }

    #[test]
    fn preserves_private_array_assignment_targets_in_destructuring_ir() {
        let program = lower_script(
            "class C {
                #value;
                assign() { [this.#value, ...this.#value] = [1, 2, 3]; }
            }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "C.assign")
            .expect("assign method should be lowered");
        let pattern = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ArrayDestructure { pattern, .. },
                    ..
                }) => Some(pattern),
                _ => None,
            })
            .expect("array assignment should be lowered");

        assert!(matches!(
            pattern.elements.as_slice(),
            [
                ArrayDestructuringElementIr::Target {
                    target: DestructuringTargetIr::AssignmentPrivate { .. },
                    ..
                },
                ArrayDestructuringElementIr::Rest {
                    target: DestructuringTargetIr::AssignmentPrivate { .. }
                }
            ]
        ));
    }

    #[test]
    fn lowers_object_assignment_properties_and_private_rest_target() {
        let program = lower_script(
            "class C {
                #value;
                assign(source, key, target) {
                    ({ [key]: target.value, fallback = 1, ...this.#value } = source);
                }
            }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "C.assign")
            .expect("assign method should be lowered");
        let pattern = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ObjectDestructure { pattern, .. },
                    ..
                }) => Some(pattern),
                _ => None,
            })
            .expect("object assignment should be lowered");

        assert_eq!(pattern.properties.len(), 2);
        assert!(matches!(
            pattern.properties[0],
            ObjectDestructuringPropertyIr {
                key: DestructuringPropertyKeyIr::Computed(_),
                target: DestructuringTargetIr::AssignmentProperty { .. },
                default: None,
            }
        ));
        assert!(pattern.properties[1].default.is_some());
        assert!(matches!(
            pattern.rest,
            Some(DestructuringTargetIr::AssignmentPrivate { .. })
        ));
    }

    #[test]
    fn plans_functions_in_array_pattern_defaults() {
        let program = lower_script(
            "let [first = function() { return 1; }] = []; var second; [second = function() { return 2; }] = [];",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        assert_eq!(
            script
                .functions
                .iter()
                .filter(|function| function.is_expression)
                .count(),
            2
        );
    }

    #[test]
    fn array_assignment_targets_are_captured_in_nested_functions() {
        let program = lower_script(
            "function owner(iterable) { let outer = 0; function assign() { [outer] = iterable; return outer; } return assign; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let assign = script
            .functions
            .iter()
            .find(|function| function.name == "assign")
            .expect("nested assign function should be lowered");
        assert!(assign
            .captured_bindings
            .iter()
            .any(|binding| binding.name == "outer"));
    }

    #[test]
    fn captured_const_array_assignment_target_remains_immutable() {
        let program =
            lower_script("const outer = 0; function assign(iterable) { [outer] = iterable; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let assign = script
            .functions
            .iter()
            .find(|function| function.name == "assign")
            .expect("assign function should be lowered");
        assert!(
            matches!(
                &assign.body.statements[0],
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ArrayDestructure { pattern, .. },
                    ..
                }) if matches!(
                    &pattern.elements[0],
                    ArrayDestructuringElementIr::Target {
                        target: DestructuringTargetIr::AssignmentIdentifier {
                            immutable: true,
                            ..
                        },
                        ..
                    }
                )
            ),
            "{:?}",
            assign.body
        );
    }

    #[test]
    fn depth_two_function_const_capture_preserves_array_target_immutability() {
        let program = lower_script(
            "function outer() { const value = 0; function middle() { function inner() { [value] = [1]; } return inner; } return middle; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let inner = script
            .functions
            .iter()
            .find(|function| function.name == "inner")
            .expect("inner function should be lowered");
        assert!(inner.captured_bindings.iter().any(|binding| {
            binding.source_name == "value" && binding.mode == BindingMode::Const
        }));
        assert!(
            matches!(
                &inner.body.statements[0],
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ArrayDestructure { pattern, .. },
                    ..
                }) if matches!(
                    &pattern.elements[0],
                    ArrayDestructuringElementIr::Target {
                        target: DestructuringTargetIr::AssignmentIdentifier {
                            immutable: true,
                            ..
                        },
                        ..
                    }
                )
            ),
            "{:?}",
            inner.body
        );
    }

    #[test]
    fn depth_two_block_const_capture_uses_selected_environment_mode() {
        let source = "function outer() { { const value = 0; function middle() { function inner() { [value] = [1]; } return inner; } return middle; } }";
        with_script_analysis(source, |analysis| {
            let inner = analysis
                .function_plans
                .values()
                .find(|function| function.name == "inner")
                .expect("inner function should be planned");
            let (storage_name, capture) = inner
                .captures
                .iter()
                .find(|(_, capture)| capture.source_name == "value")
                .expect("inner should capture the block const binding");
            let environment = &analysis.environment_plans[&capture.environment_id];
            assert_eq!(environment.kind, EnvironmentKind::Block);
            assert_ne!(storage_name, "value");
            assert!(
                capture.environment_id
                    != analysis.owner_plans[&capture.owner_id].activation_environment_id
            );
            assert_eq!(capture.mode, BindingMode::Const);
            assert_eq!(
                environment.binding_modes.get(storage_name),
                Some(&BindingMode::Const)
            );
        });
    }

    #[test]
    fn generated_class_method_capture_preserves_planned_const_mode() {
        let program = lower_script(
            "function outer() { const value = 0; class Holder { assign() { [value] = [1]; } } return Holder; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let method = script
            .functions
            .iter()
            .find(|function| function.name.ends_with(".assign"))
            .unwrap_or_else(|| {
                panic!(
                    "class method should be lowered: {:?}",
                    script
                        .functions
                        .iter()
                        .map(|function| function.name.as_str())
                        .collect::<Vec<_>>()
                )
            });
        assert!(method.captured_bindings.iter().any(|binding| {
            binding.source_name == "value" && binding.mode == BindingMode::Const
        }));
        assert!(
            matches!(
                &method.body.statements[0],
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ArrayDestructure { pattern, .. },
                    ..
                }) if matches!(
                    &pattern.elements[0],
                    ArrayDestructuringElementIr::Target {
                        target: DestructuringTargetIr::AssignmentIdentifier {
                            immutable: true,
                            ..
                        },
                        ..
                    }
                )
            ),
            "{:?}",
            method.body
        );
    }

    #[test]
    fn capture_planning_preserves_annex_b_and_mutable_binding_modes() {
        with_script_analysis("if (true) { function copied() {} }", |analysis| {
            let activation = &analysis.environment_plans
                [&analysis.owner_plans[SCRIPT_OWNER_ID].activation_environment_id];
            assert_eq!(
                activation.binding_modes.get("copied"),
                Some(&BindingMode::Var)
            );
            assert!(analysis.environment_plans.values().any(|environment| {
                environment.binding_modes.iter().any(|(name, mode)| {
                    name.starts_with("$annexb.block.") && *mode == BindingMode::Let
                })
            }));
        });

        let program =
            lower_script("function outer() { let value = 0; function read() { return value; } }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let read = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("reader function should be lowered");
        assert!(read
            .captured_bindings
            .iter()
            .any(|binding| { binding.source_name == "value" && binding.mode == BindingMode::Let }));
    }

    #[test]
    fn captured_var_assignments_share_the_owners_environment_slot() {
        let program = lower_script(
            "function outer(TA) { var ta; function read() { return ta.buffer; } ta = new TA(1); return read(); } outer(Float64Array);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let outer = script
            .functions
            .iter()
            .find(|function| function.name == "outer")
            .expect("outer function should be lowered");
        let read = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("read function should be lowered");
        let owned = outer
            .owned_env_bindings
            .iter()
            .find(|binding| binding.name == "ta")
            .expect("outer function should own the captured var");
        let captured = read
            .captured_bindings
            .iter()
            .find(|binding| binding.name == "ta")
            .expect("read function should capture the var");
        assert_eq!(captured.slot, owned.slot);
        let StatementIr::Return(TypedExpr {
            expr: ExprIr::SpecOperation { operands, .. },
            ..
        }) = &read.body.statements[0]
        else {
            panic!("captured property read should lower through GetV");
        };
        assert_eq!(
            operands[0].possible_kinds,
            KindSet::all_runtime_tags(),
            "a mutable capture must not retain its pre-assignment undefined type"
        );
    }

    #[test]
    fn materializes_defaulted_object_var_property_reads_once() {
        let program = lower_script(
            "var getterHits = 0; var receiver = { get value() { getterHits += 1; return undefined; } }; function fallback() { return 1; } var { value = fallback() } = receiver; value;",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let statements = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::LexicalBlock(statements)
                    if matches!(statements.first(), Some(StatementIr::Lexical { .. }))
                        && matches!(statements.get(1), Some(StatementIr::Lexical { .. }))
                        && matches!(statements.get(2), Some(StatementIr::Var(_))) =>
                {
                    Some(statements)
                }
                _ => None,
            })
            .expect("expected destructuring var bindings");

        let StatementIr::Lexical {
            mode: BindingMode::Let,
            name: temporary_name,
            init,
        } = &statements[0]
        else {
            panic!("expected RHS materialization");
        };
        assert!(temporary_name.starts_with("$destructure.internal."));
        assert!(matches!(
            init.expr,
            ExprIr::Identifier(ref name) | ExprIr::GlobalPropertyRead { ref name }
                if name == "receiver"
        ));

        let StatementIr::Lexical {
            mode: BindingMode::Let,
            name: property_value_name,
            init: property_value,
        } = &statements[1]
        else {
            panic!("expected materialized property read");
        };
        assert!(property_value_name.starts_with("$destructure.value.internal."));
        let ExprIr::PropertyRead { target, key } = &property_value.expr else {
            panic!("expected property read");
        };
        assert!(matches!(target.expr, ExprIr::Identifier(ref name) if name == temporary_name));
        assert_eq!(key, &PropertyKeyIr::StaticString("value".to_string()));

        let StatementIr::Var(declarators) = &statements[2] else {
            panic!("expected var property binding");
        };
        let [declarator] = declarators.as_slice() else {
            panic!("expected one var property binding");
        };
        assert_eq!(declarator.name, "value");
        let init = declarator.init.as_ref().expect("expected var initializer");
        let ExprIr::Conditional {
            condition,
            else_expr,
            ..
        } = &init.expr
        else {
            panic!("expected default initializer");
        };
        let ExprIr::StrictEquality { lhs, .. } = &condition.expr else {
            panic!("expected undefined check");
        };
        assert!(matches!(lhs.expr, ExprIr::Identifier(ref name) if name == property_value_name));
        assert!(
            matches!(else_expr.expr, ExprIr::Identifier(ref name) if name == property_value_name)
        );
    }

    #[test]
    fn hoists_object_var_bindings_in_for_of_loops() {
        let program = lower_script(
            "for (var { iterator, error } of [{ iterator: 1, error: 2 }]) {} iterator + error;",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        for name in ["iterator", "error"] {
            assert!(script.global_bindings.iter().any(|binding| {
                binding.name == name && matches!(binding.kind, ScriptGlobalBindingKind::Var)
            }));
        }

        let StatementIr::ForOfArray { body, .. } = &script.body.statements[0] else {
            panic!("expected array for-of loop");
        };
        let StatementIr::Block(block) = body.as_ref() else {
            panic!("expected destructuring loop body block");
        };
        let StatementIr::Var(declarators) = &block.statements[0] else {
            panic!("expected hoisted var property bindings");
        };
        assert_eq!(
            declarators
                .iter()
                .map(|declarator| declarator.name.as_str())
                .collect::<Vec<_>>(),
            ["iterator", "error"]
        );
    }

    #[test]
    fn materializes_defaulted_object_var_property_reads_in_for_of_loops() {
        let program =
            lower_script("for (var { value = fallback() } of [{ value: undefined }]) {} value;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::ForOfArray { body, .. } = &script.body.statements[0] else {
            panic!("expected array for-of loop");
        };
        let StatementIr::Block(block) = body.as_ref() else {
            panic!("expected destructuring loop body block");
        };
        let StatementIr::Lexical {
            name: property_value_name,
            init: property_value,
            ..
        } = &block.statements[0]
        else {
            panic!("expected materialized property read");
        };
        assert!(property_value_name.starts_with("$destructure.value.internal."));
        assert!(matches!(property_value.expr, ExprIr::PropertyRead { .. }));

        let StatementIr::Var(declarators) = &block.statements[1] else {
            panic!("expected var property binding");
        };
        let [declarator] = declarators.as_slice() else {
            panic!("expected one var property binding");
        };
        let init = declarator.init.as_ref().expect("expected var initializer");
        let ExprIr::Conditional {
            condition,
            else_expr,
            ..
        } = &init.expr
        else {
            panic!("expected default initializer");
        };
        let ExprIr::StrictEquality { lhs, .. } = &condition.expr else {
            panic!("expected undefined check");
        };
        assert!(matches!(lhs.expr, ExprIr::Identifier(ref name) if name == property_value_name));
        assert!(
            matches!(else_expr.expr, ExprIr::Identifier(ref name) if name == property_value_name)
        );
    }

    #[test]
    fn hoists_object_var_bindings_from_for_of_loops_in_functions() {
        let program = lower_script(
            "function values() { for (var { value } of [{ value: 1 }]) {} return value; } values();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(!script
            .global_bindings
            .iter()
            .any(|binding| binding.name == "value"));
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "values")
            .expect("function should be lowered");
        assert!(matches!(
            function.body.statements.last(),
            Some(StatementIr::Return(TypedExpr {
                expr: ExprIr::Identifier(name),
                ..
            })) if name == "value"
        ));
    }

    #[test]
    fn object_destructuring_predeclares_tdz_and_coerces_empty_patterns() {
        let program = lower_script("let { value } = value; let {} = null; let {} = undefined;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::LexicalBlock(value_binding) = &script.body.statements[0] else {
            panic!("expected value destructuring block");
        };
        let StatementIr::Lexical { init, .. } = &value_binding[0] else {
            panic!("expected RHS materialization");
        };
        assert!(matches!(init.expr, ExprIr::RuntimeThrow { .. }));

        for statement in &script.body.statements[1..] {
            let StatementIr::LexicalBlock(bindings) = statement else {
                panic!("expected empty destructuring block");
            };
            assert_eq!(bindings.len(), 2);
            let StatementIr::Expression(coercion) = &bindings[1] else {
                panic!("expected RequireObjectCoercible representation");
            };
            assert!(matches!(
                coercion.expr,
                ExprIr::SpecOperation {
                    operation: SpecOperationIr::ToObject,
                    ..
                }
            ));
        }
    }

    #[test]
    fn object_var_destructuring_coerces_empty_patterns() {
        let program = lower_script("var {} = null;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::LexicalBlock(bindings) = &script.body.statements[0] else {
            panic!("expected empty destructuring block");
        };
        assert_eq!(bindings.len(), 2);
        let StatementIr::Expression(coercion) = &bindings[1] else {
            panic!("expected RequireObjectCoercible representation");
        };
        assert!(matches!(
            coercion.expr,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::ToObject,
                ..
            }
        ));
    }

    #[test]
    fn lowers_computed_key_object_destructuring_forms() {
        // A computed key contributes no bound name (8.6 BoundNames), so these bind
        // exactly what the literal-key spelling binds and lower through the semantic
        // `ObjectDestructure` node, which carries the key expression.
        for source in [
            "let { [key]: value } = source;",
            "let { ['value']: value } = source;",
            "const { [key]: value = 1 } = source;",
            "const { [key]: { nested } } = source;",
            "let { [key]: value, ...rest } = source;",
            "for (const { [key]: value } of source) print(value);",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "expected supported lowering for {source}: {:?}",
                program.diagnostics
            );
        }
    }

    #[test]
    fn lowers_object_assignment_patterns_in_loop_heads() {
        // 13.15.5 destructuring assignment is legal in a `for-in`/`for-of` head, and
        // the object shape reaches the same assignment-pattern lowering the array
        // shape does, so property-access and rest targets work there too.
        for source in [
            "let a; for ({ a } of source) print(a);",
            "let a; for ({ a = 1 } of source) print(a);",
            "const o = {}; for ({ a: o.x } of source) print(o.x);",
            "let r; for ({ ...r } of source) print(r);",
            "let a; for ({ length: a } in source) print(a);",
            "let a; for ([a] in source) print(a);",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "expected supported lowering for {source}: {:?}",
                program.diagnostics
            );
        }
    }

    #[test]
    fn lowers_nested_and_rest_object_destructuring_bindings() {
        for source in [
            "let { value: { nested } } = source;",
            "let { value, ...rest } = source;",
            "let { value: [first] } = source;",
            "var { value: [first], ...rest } = source;",
            "let a, b; ({ value: [a], ...b } = source);",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "expected supported lowering for {source}: {:?}",
                program.diagnostics
            );
        }
    }

    #[test]
    fn lowers_coercion_core_ir() {
        let program = lower_script("1 == \"1\"; \"2\" - 1; \"10\" > \"2\"; void 1; (1, 2);");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("spec_operations=1"));
        assert!(summary.contains("coercive_numeric_ops=1"));
        assert!(summary.contains("coercive_relational_ops=1"));
        assert!(summary.contains("void_uses=1"));
        assert!(summary.contains("comma_ops=1"));
    }

    #[test]
    fn lowers_object_function_property_call_arg_coercion_ir() {
        let program = lower_script(
            r#"var h = { get: function(_, key) { return key * 10; } }; h.get({}, "1");"#,
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("coercive_numeric_ops=2"));
    }

    #[test]
    fn lowers_object_seal_with_argument_result_shape() {
        let program = lower_script("var target = { value: 1 }; Object.seal(target);");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(expression) = &script.body.statements[1] else {
            panic!("expected Object.seal expression");
        };
        assert_eq!(expression.kind, ValueKind::Object);
        let function_id = StandardBuiltinId::ObjectSeal.function_id();
        let call = indirect_call_body(expression)
            .unwrap_or_else(|| panic!("expected Object.seal call: {expression:?}"));
        let ExprIr::CallIndirect { callee, .. } = &call.expr else {
            unreachable!("indirect_call_body only returns indirect calls");
        };
        assert!(callee.function_targets.contains(&function_id));
    }

    #[test]
    fn lowers_heap_loose_equality_ir() {
        let program = lower_script("let object = {}; object == undefined; null != object;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("spec_operations=2"));
    }

    #[test]
    fn lowers_typeof_unresolved_identifier() {
        let program = lower_script("typeof missingName;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        assert!(matches!(
            expr.expr,
            ExprIr::TypeOfUnresolvedIdentifier { .. }
        ));
        assert_eq!(expr.kind, ValueKind::String);
    }

    #[test]
    fn lowers_symbol_key_for_through_real_method() {
        // `Symbol.keyFor` now resolves through the real `Symbol` constructor
        // object's own `keyFor` method (backed by a runtime registry) rather
        // than a compile-time fold, so the call dispatches indirectly and its
        // result is typed `String | undefined`.
        let program = lower_script("Symbol.keyFor(Symbol.iterator);");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        assert!(indirect_call_body(expr).is_some());
        assert!(expr.possible_kinds.contains(ValueKind::String));
        assert!(expr.possible_kinds.contains(ValueKind::Undefined));
    }

    #[test]
    fn lowers_symbol_description_property() {
        let program = lower_script("Symbol.iterator.description.startsWith(\"Symbol.\");");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        assert_eq!(expr.kind, ValueKind::Boolean);
    }

    #[test]
    fn folds_temporal_ascii_identifier_helper_test() {
        let program = lower_script(
            "const ASCII_IDENTIFIER = /^[$_a-zA-Z][$_a-zA-Z0-9]*$/u; ASCII_IDENTIFIER.test(\"next\");",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        assert!(matches!(expr.expr, ExprIr::Boolean(true)));
    }

    #[test]
    fn lowers_temporal_zoned_date_time_from_with_instance_result_shape() {
        let program = lower_script("Temporal.ZonedDateTime.from(\"1970-01-01T00:00Z[UTC]\");");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(expression) = &script.body.statements[0] else {
            panic!("expected Temporal.ZonedDateTime.from expression");
        };
        assert_eq!(expression.kind, ValueKind::Object);
        assert!(expression.heap_shape.is_some());
        let function_id = StandardBuiltinId::TemporalZonedDateTimeFrom.function_id();
        let call = indirect_call_body(expression)
            .unwrap_or_else(|| panic!("expected Temporal.ZonedDateTime.from call: {expression:?}"));
        let ExprIr::CallIndirect { callee, .. } = &call.expr else {
            unreachable!("indirect_call_body only returns indirect calls");
        };
        assert!(callee.function_targets.contains(&function_id));
    }

    #[test]
    fn lowers_temporal_zoned_date_time_fixed_offset_accessors() {
        let program = lower_script(
            "const value = new Temporal.ZonedDateTime(0n, \"+01:30\"); \
             value.offset; value.offsetNanoseconds;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(offset) = &script.body.statements[1] else {
            panic!("expected Temporal.ZonedDateTime offset expression");
        };
        let StatementIr::Expression(offset_nanoseconds) = &script.body.statements[2] else {
            panic!("expected Temporal.ZonedDateTime offsetNanoseconds expression");
        };
        assert_eq!(offset.kind, ValueKind::String);
        assert_eq!(offset_nanoseconds.kind, ValueKind::Number);
    }

    #[test]
    fn lowers_temporal_instant_equals_with_boolean_result() {
        let program = lower_script("new Temporal.Instant(1n).equals(new Temporal.Instant(1n));");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(expression) = &script.body.statements[0] else {
            panic!("expected Temporal.Instant.prototype.equals expression");
        };
        assert_eq!(expression.kind, ValueKind::Boolean);
        let function_id = StandardBuiltinId::TemporalInstantPrototypeEquals.function_id();
        let call = indirect_call_body(expression).unwrap_or_else(|| {
            panic!("expected Temporal.Instant.prototype.equals call: {expression:?}")
        });
        let ExprIr::CallIndirect { callee, .. } = &call.expr else {
            unreachable!("indirect_call_body only returns indirect calls");
        };
        assert!(callee.function_targets.contains(&function_id));
    }

    #[test]
    fn lowers_switch_labels_and_debugger_ir() {
        let program = lower_script(
            "outer: while (true) { switch (2) { case 1: break; case 2: debugger; break outer; default: break; } }",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("switches=1"));
        assert!(summary.contains("labels=1"));
        assert!(summary.contains("debuggers=1"));
    }

    #[test]
    fn lowers_hoisted_var_ir() {
        let program = lower_script("x; var x = 1; if (true) { var y = 2; } y;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("vars=2"));
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(script.body.statements[1], StatementIr::Var(_)));
    }

    #[test]
    fn lowers_top_level_functions_and_calls() {
        let program = lower_script("add(1, 2); function add(x, y) { return x + y; }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 2);
        assert_eq!(script.functions[0].params.len(), 2);
        let summary = program.ir_summary();
        assert!(summary.contains("functions=2"));
        assert!(summary.contains("calls=1"));
        assert!(summary.contains("returns=2"));
    }

    #[test]
    fn lowers_array_binding_patterns_in_function_parameters() {
        let program = lower_script("function dstr(a, [b]) { return b; } dstr(1, [2]);");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "dstr")
            .expect("dstr should be lowered");
        assert_eq!(function.params[1].name, "$destructured.param.1");
        let StatementIr::ParameterInitialization {
            parameter_index: 1,
            statements,
        } = &function.body.statements[0]
        else {
            panic!("expected parameter initialization marker");
        };
        let StatementIr::Expression(TypedExpr {
            expr:
                ExprIr::ArrayDestructure {
                    pattern,
                    assignment: false,
                    ..
                },
            ..
        }) = &statements[0]
        else {
            panic!("expected parameter array destructuring inside the initialization marker");
        };
        assert!(matches!(
            pattern.elements.as_slice(),
            [ArrayDestructuringElementIr::Target {
                target: DestructuringTargetIr::Binding { name, .. },
                ..
            }] if name == "b"
        ));
    }

    #[test]
    fn lowers_object_rest_binding_in_generator_parameters() {
        let program =
            lower_script("var f = function* ({ a, ...rest } = { a: 1, b: 2 }) {}; f().next();");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.execution_kind == FunctionExecutionKind::Generator)
            .expect("generator expression should be lowered");
        assert!(function.params[0].default_init.is_some());
        let StatementIr::ParameterInitialization { statements, .. } = &function.body.statements[0]
        else {
            panic!("expected parameter initialization marker");
        };
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::ObjectDestructure { pattern, .. },
            ..
        }) = &statements[0]
        else {
            panic!("expected object destructuring parameter initialization");
        };
        assert_eq!(pattern.properties.len(), 1);
        assert!(matches!(
            pattern.rest,
            Some(DestructuringTargetIr::Binding { ref name, .. }) if name == "rest"
        ));
    }

    #[test]
    fn lowers_computed_object_keys_in_generator_parameters() {
        let program = lower_script(
            "var key = 'value'; var f = function* ({ [key]: value = 9 }) {}; f({}).next();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.execution_kind == FunctionExecutionKind::Generator)
            .expect("generator expression should be lowered");
        let StatementIr::ParameterInitialization { statements, .. } = &function.body.statements[0]
        else {
            panic!("expected parameter initialization marker");
        };
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::ObjectDestructure { pattern, .. },
            ..
        }) = &statements[0]
        else {
            panic!("expected object destructuring parameter initialization");
        };
        assert!(matches!(
            pattern.properties.as_slice(),
            [ObjectDestructuringPropertyIr {
                key: DestructuringPropertyKeyIr::Computed(_),
                default: Some(_),
                ..
            }]
        ));
    }

    #[test]
    fn lowers_nested_object_and_array_generator_parameters() {
        let program = lower_script(
            "var f = function* ([{ x }], { values: [y] }) {}; f([{ x: 1 }], { values: [2] }).next();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.execution_kind == FunctionExecutionKind::Generator)
            .expect("generator expression should be lowered");
        let StatementIr::ParameterInitialization {
            statements: first, ..
        } = &function.body.statements[0]
        else {
            panic!("expected first parameter initialization marker");
        };
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::ArrayDestructure { pattern: first, .. },
            ..
        }) = &first[0]
        else {
            panic!("expected nested object in array destructuring");
        };
        assert!(matches!(
            first.elements.as_slice(),
            [ArrayDestructuringElementIr::Target {
                target: DestructuringTargetIr::NestedObject(_),
                ..
            }]
        ));

        let StatementIr::ParameterInitialization {
            statements: second, ..
        } = &function.body.statements[1]
        else {
            panic!("expected second parameter initialization marker");
        };
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::ObjectDestructure {
                pattern: second, ..
            },
            ..
        }) = &second[0]
        else {
            panic!("expected nested array in object destructuring");
        };
        assert!(matches!(
            second.properties.as_slice(),
            [ObjectDestructuringPropertyIr {
                target: DestructuringTargetIr::NestedArray(_),
                ..
            }]
        ));
    }

    #[test]
    fn marks_function_body_use_strict_directive() {
        let program = lower_script(r#"function f() { "use strict"; return this; } f();"#);
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "f")
            .expect("function should be lowered");
        assert!(function.strict);
    }

    #[test]
    fn marks_script_use_strict_directive() {
        let program = lower_script(r#""use strict"; function f() { return this; } f();"#);
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.strict);
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "f")
            .expect("function should be lowered");
        assert!(function.strict);
    }

    #[test]
    fn lowers_block_function_declarations_as_hoisted_bindings() {
        let program = lower_script(
            "let value = 0; if (true) { value = nested(); function nested() { return 7; } } value;",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("functions=2"));
        assert!(summary.contains("calls=1"));
    }

    #[test]
    fn lowers_root_function_constructor_reference_inside_function_body() {
        let program = lower_script(
            "function Box(message) { this.message = message; } function make(message) { return new Box(message); } make(\"ok\");",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("constructs=1"));
    }

    #[test]
    fn exposes_gc_as_noop_host_builtin() {
        let program = lower_script("if (typeof gc === \"function\") { gc(); }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.host_builtins.contains(&HostBuiltinId::Gc));
    }

    #[test]
    fn prunes_unresolved_typeof_function_guard() {
        let program =
            lower_script("if (typeof __missingHostHook === \"function\") { __missingHostHook(); }");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("calls=0"));
    }

    #[test]
    fn prunes_unresolved_typeof_and_guard_rhs() {
        let program = lower_script(
            "if (typeof Symbol !== \"undefined\" && Symbol.iterator) { missingCall(); }",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("calls=0"));
    }

    #[test]
    fn marks_explicit_extending_class_constructor_as_derived() {
        let program =
            lower_script("class A {} class B extends A { constructor() { super(); } } B;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let derived = script
            .functions
            .iter()
            .find(|function| function.name == "B")
            .expect("derived constructor should be lowered");
        assert!(derived.is_derived_constructor);
        assert!(derived.super_constructor_target.is_some());
        assert_canonical_derived_activation(derived);
    }

    #[test]
    fn gives_default_derived_constructor_canonical_activation_but_not_base_constructor() {
        let program = lower_script("class A {} class B extends A {} B;");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let base = script
            .functions
            .iter()
            .find(|function| function.name == "A")
            .expect("base constructor should be lowered");
        assert!(!base.is_derived_constructor);
        assert!(base.lexical_derived_activation.is_none());
        assert!(base.owned_env_bindings.iter().all(|binding| {
            ![
                DERIVED_ACTIVATION_THIS_NAME,
                DERIVED_ACTIVATION_THIS_STATUS_NAME,
                DERIVED_ACTIVATION_NEW_TARGET_NAME,
                DERIVED_ACTIVATION_FUNCTION_NAME,
            ]
            .contains(&binding.name.as_str())
        }));

        let derived = script
            .functions
            .iter()
            .find(|function| function.name == "B")
            .expect("default derived constructor should be lowered");
        assert!(derived.is_derived_constructor);
        assert!(derived.is_synthetic_default_derived_constructor);
        assert_canonical_derived_activation(derived);
    }

    #[test]
    fn lowers_immediate_arrow_super_with_derived_activation_capture() {
        let program =
            lower_script("class A {} class B extends A { constructor() { (() => super())(); } }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let constructor = script
            .functions
            .iter()
            .find(|function| function.is_derived_constructor)
            .unwrap();
        let activation = constructor.lexical_derived_activation.as_ref().unwrap();
        assert_eq!(activation.owner_function_id, constructor.id);
        let arrow = script
            .functions
            .iter()
            .find(|function| function.flavor == FunctionFlavor::Arrow)
            .unwrap();
        assert!(arrow.uses_super);
        assert!(arrow
            .captured_bindings
            .iter()
            .any(|binding| binding.name == DERIVED_ACTIVATION_FUNCTION_NAME));
        assert!(arrow.body.statements.iter().any(|statement| matches!(
            statement,
            StatementIr::Return(TypedExpr {
                expr: ExprIr::SuperConstruct { .. },
                ..
            })
        )));
    }

    #[test]
    fn nested_arrows_share_derived_activation_this_and_new_target() {
        let program = lower_script(
            "class A {} class B extends A { constructor() { (() => (() => [this, new.target, super()]))(); } }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let constructor = script
            .functions
            .iter()
            .find(|function| function.is_derived_constructor)
            .unwrap();
        let activation = constructor.lexical_derived_activation.as_ref().unwrap();
        for name in [
            &activation.this_binding,
            &activation.this_status_binding,
            &activation.new_target_binding,
            &activation.active_function_binding,
        ] {
            assert!(constructor
                .owned_env_bindings
                .iter()
                .any(|binding| &binding.name == name));
        }
        let arrows = script
            .functions
            .iter()
            .filter(|function| function.flavor == FunctionFlavor::Arrow)
            .collect::<Vec<_>>();
        assert_eq!(arrows.len(), 2);
        let innermost = arrows.iter().find(|function| function.uses_super).unwrap();
        assert!(innermost.captures_lexical_this);
        assert!(innermost
            .captured_bindings
            .iter()
            .any(|binding| binding.name == LEXICAL_NEW_TARGET_NAME));
        assert!(innermost
            .captured_bindings
            .iter()
            .any(|binding| binding.name == DERIVED_ACTIVATION_THIS_STATUS_NAME));
    }

    #[test]
    fn derived_arrow_this_and_new_target_capture_activation_without_super_flag() {
        for source in [
            "class A {} class B extends A { constructor() { (() => this)(); } }",
            "class A {} class B extends A { constructor() { (() => new.target)(); } }",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().unwrap();
            let arrow = script
                .functions
                .iter()
                .find(|function| function.flavor == FunctionFlavor::Arrow)
                .unwrap();
            assert!(!arrow.uses_super);
            for name in [
                DERIVED_ACTIVATION_FUNCTION_NAME,
                DERIVED_ACTIVATION_NEW_TARGET_NAME,
                DERIVED_ACTIVATION_THIS_NAME,
                DERIVED_ACTIVATION_THIS_STATUS_NAME,
            ] {
                assert!(arrow
                    .captured_bindings
                    .iter()
                    .any(|binding| binding.name == name));
            }
        }
    }

    #[test]
    fn derived_arrow_super_property_captures_activation_and_preserves_slots() {
        let program = lower_script(
            "class A { get x() { return 1; } } class B extends A { constructor() { let $a = 1; (() => (() => super.x + $a))(); } }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().unwrap();
        let constructor = script
            .functions
            .iter()
            .find(|function| function.is_derived_constructor)
            .unwrap();
        let slots = constructor
            .owned_env_bindings
            .iter()
            .map(|binding| (binding.name.as_str(), binding.slot))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(slots.get(DERIVED_ACTIVATION_FUNCTION_NAME), Some(&0));
        assert_eq!(slots.get(DERIVED_ACTIVATION_NEW_TARGET_NAME), Some(&1));
        assert_eq!(slots.get(DERIVED_ACTIVATION_THIS_NAME), Some(&2));
        assert_eq!(slots.get(DERIVED_ACTIVATION_THIS_STATUS_NAME), Some(&3));
        assert_eq!(slots.get("$a"), Some(&4));
        let arrows = script
            .functions
            .iter()
            .filter(|function| function.flavor == FunctionFlavor::Arrow)
            .collect::<Vec<_>>();
        assert_eq!(arrows.len(), 2);
        assert!(arrows.iter().any(|function| function.uses_super));
    }

    #[test]
    fn derived_arrow_dynamic_super_method_call_uses_lexical_this() {
        let source = "class A { increment() {} } class B extends A { constructor() { super(); (() => super.increment())(); } }";
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let arrow = script
            .functions
            .iter()
            .find(|function| function.flavor == FunctionFlavor::Arrow)
            .expect("arrow should be lowered");
        let Some(StatementIr::Return(TypedExpr {
            expr:
                ExprIr::CallIndirect {
                    callee,
                    this_arg: Some(this_arg),
                    args,
                    ..
                },
            ..
        })) = arrow.body.statements.first()
        else {
            panic!("expected indirect super method call: {:?}", arrow.body);
        };
        assert!(matches!(
            callee.expr,
            ExprIr::SuperPropertyRead {
                key: PropertyKeyIr::StaticString(ref key),
            } if key == "increment"
        ));
        assert!(matches!(this_arg.expr, ExprIr::This));
        assert!(args.is_empty());
    }

    #[test]
    fn class_method_arrow_captures_lexical_home_object_and_this_for_super() {
        let source =
            "class A { method() {} } class B extends A { make() { return () => super.method(); } }";
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let method = script
            .functions
            .iter()
            .find(|function| function.name == "B.make")
            .expect("class method should be lowered");
        for name in [LEXICAL_THIS_NAME, LEXICAL_HOME_OBJECT_NAME] {
            assert!(method
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == name));
        }
        let arrow = script
            .functions
            .iter()
            .find(|function| function.flavor == FunctionFlavor::Arrow)
            .expect("arrow should be lowered");
        for name in [LEXICAL_THIS_NAME, LEXICAL_HOME_OBJECT_NAME] {
            assert!(arrow
                .captured_bindings
                .iter()
                .any(|binding| binding.name == name));
        }
        assert!(arrow.captures_lexical_this);
    }

    #[test]
    fn exact_context_specialization_preserves_escaped_closure_environment() {
        let source = "class B { make() { let x = 7; return () => () => x; } } let b = new B(); let outer = b.make(); let inner = outer(); inner();";
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let inner_init = script.body.statements.iter().find_map(|statement| {
            let StatementIr::Lexical { name, init, .. } = statement else {
                return None;
            };
            (name == "inner").then_some(init)
        });
        let Some(TypedExpr {
            expr: ExprIr::CallIndirect { callee, .. },
            ..
        }) = inner_init
        else {
            panic!("expected escaped outer closure call: {:?}", script.body);
        };
        assert!(matches!(
            callee.expr,
            ExprIr::Identifier(ref name) if name == "outer"
        ));
    }

    #[test]
    fn exact_context_specialization_preserves_escaped_callback_argument() {
        let source = "function invoke(callback) { return callback(); } function make() { let x = 1; return () => x; } let callback = make(); invoke(callback);";
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let callback_arg = script.body.statements.iter().find_map(|statement| {
            let StatementIr::Expression(TypedExpr {
                expr: ExprIr::CallIndirect { args, .. },
                ..
            }) = statement
            else {
                return None;
            };
            args.first()
        });
        assert!(matches!(
            callback_arg,
            Some(TypedExpr {
                expr: ExprIr::Identifier(name),
                ..
            }) if name == "callback"
        ));
    }

    #[test]
    fn array_callback_exact_context_preserves_this_argument_shape() {
        let source = "['source', 'flags'].forEach(function (key) { Object.defineProperty(this, key, { value: '' }); }, this);";
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
    }

    #[test]
    fn base_arrow_does_not_capture_derived_activation() {
        let program = lower_script("class B { constructor() { (() => this)(); } }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().unwrap();
        let arrow = script
            .functions
            .iter()
            .find(|function| function.flavor == FunctionFlavor::Arrow)
            .unwrap();
        assert!(!arrow.uses_super);
        assert!(arrow.lexical_derived_activation.is_none());
        assert!(!arrow
            .captured_bindings
            .iter()
            .any(|binding| binding.name == DERIVED_ACTIVATION_FUNCTION_NAME));
    }

    #[test]
    fn ordinary_function_is_a_lexical_super_boundary() {
        let source = "class A {} class B extends A { constructor() { (function () { return () => super(); })(); } }";
        assert!(parse(source, ParseOptions::script()).is_err());
    }

    #[test]
    fn infers_array_kind_for_direct_and_nested_default_subclasses() {
        for source in [
            "class Ar extends Array {} new Ar();",
            "class A extends Array {} class B extends A {} new B();",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
                panic!("expected constructed instance");
            };
            assert_eq!(instance.kind, ValueKind::Array, "{source}");
            assert!(matches!(
                instance.heap_shape.as_deref(),
                Some(HeapShape::Array(_))
            ));
        }

        let ordinary = lower_script("class C {} new C();");
        let script = ordinary.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
            panic!("expected constructed instance");
        };
        assert_eq!(instance.kind, ValueKind::Object);
        assert!(matches!(
            instance.heap_shape.as_deref(),
            Some(HeapShape::Object(_))
        ));
    }

    #[test]
    fn object_create_does_not_infer_a_null_prototype_from_a_mixed_prototype() {
        let program =
            lower_script("var prototype = globalThis.flag ? null : {}; Object.create(prototype);");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
            panic!("expected Object.create result");
        };
        assert_eq!(instance.kind, ValueKind::Object);
        assert!(instance.heap_shape.is_none());
    }

    #[test]
    fn preserves_private_brand_shape_for_array_subclass_instances() {
        let program =
            lower_script("class A extends Array { #x; has() { return #x in this; } } new A();");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
            panic!("expected class declaration");
        };
        let ExprIr::ClassDefinition(class) = &init.expr else {
            panic!("expected class definition");
        };
        let private_name_id = *class
            .private_name_ids
            .get("x")
            .expect("private brand should be assigned");
        let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
            panic!("expected constructed instance");
        };
        let HeapShape::Array(shape) = instance.heap_shape.as_deref().expect("array shape") else {
            panic!("expected ArrayShape");
        };
        assert!(shape
            .properties
            .contains_key(&private_brand_key(private_name_id)));
    }

    #[test]
    fn private_in_rhs_preserves_shift_precedence_and_runtime_global_resolution() {
        for (source, expected_rhs) in [
            (
                "class C { #field; probe() { try { #field in {} << 0; } catch (error) {} } }",
                "shift",
            ),
            (
                "class C { #field; probe() { try { #field in missingName; } catch (error) {} } }",
                "global-resolution",
            ),
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let probe = script
                .functions
                .iter()
                .find(|function| function.name == "C.probe")
                .expect("private-in probe should be lowered");
            let StatementIr::TryCatch { try_block, .. } = &probe.body.statements[0] else {
                panic!("expected private-in try/catch");
            };
            let StatementIr::Expression(TypedExpr {
                expr: ExprIr::PrivateIn { rhs, .. },
                ..
            }) = &try_block.statements[0]
            else {
                panic!("expected private-in expression");
            };

            match expected_rhs {
                "shift" => assert!(matches!(
                    &rhs.expr,
                    ExprIr::BitwiseNumber {
                        op: BitwiseBinaryOp::Shl,
                        ..
                    }
                )),
                "global-resolution" => assert!(matches!(
                    &rhs.expr,
                    ExprIr::GlobalIdentifierRead { name } if name == "missingName"
                )),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn private_in_captured_error_widens_the_enclosing_binding() {
        let program = lower_script(
            "let caught = null;
             class C {
                 #field;
                 constructor() {
                     try { #field in 0; } catch (error) { caught = error; }
                 }
             }
             new C();
             caught.constructor;",
        );

        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn same_spelling_in_distinct_nested_classes_has_distinct_private_identity() {
        let program = lower_script(
            "function first() { return class { #value; }; }
             function second() { return class { #value; }; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let private_name_id = |function_name: &str| {
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("{function_name} should be lowered"));
            let class = function
                .body
                .statements
                .iter()
                .find_map(|statement| match statement {
                    StatementIr::Return(TypedExpr {
                        expr: ExprIr::ClassDefinition(class),
                        ..
                    }) => Some(class),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{function_name} should return a class"));
            *class
                .private_name_ids
                .get("value")
                .unwrap_or_else(|| panic!("{function_name} should declare #value"))
        };

        assert_ne!(private_name_id("first"), private_name_id("second"));
    }

    #[test]
    fn resolves_private_names_through_nested_function_and_class_boundaries() {
        let program = lower_script(
            "class Outer {
                #value;
                make() {
                    function nested(receiver) { return receiver.#value; }
                    return class Inner extends (function Heritage(receiver) {
                        return receiver.#value;
                    }) {
                        #value;
                        read(receiver) { return receiver.#value; }
                    };
                }
            }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
            panic!("expected outer class declaration");
        };
        let ExprIr::ClassDefinition(outer_class) = &init.expr else {
            panic!("expected outer class definition");
        };
        let outer_private_name_id = outer_class.private_name_ids["value"];

        let make = script
            .functions
            .iter()
            .find(|function| function.name == "Outer.make")
            .expect("outer method should be lowered");
        let inner_class = make
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Return(TypedExpr {
                    expr: ExprIr::ClassDefinition(class),
                    ..
                }) => Some(class),
                _ => None,
            })
            .expect("outer method should return the inner class");
        let inner_private_name_id = inner_class.private_name_ids["value"];
        assert_ne!(inner_private_name_id, outer_private_name_id);

        let returned_private_name_id = |function_name: &str| {
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("function `{function_name}` should be lowered"));
            function
                .body
                .statements
                .iter()
                .find_map(|statement| match statement {
                    StatementIr::Return(TypedExpr {
                        expr:
                            ExprIr::PrivateRead {
                                private_name_id, ..
                            },
                        ..
                    }) => Some(*private_name_id),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("function `{function_name}` should read a private name"))
        };

        assert_eq!(returned_private_name_id("nested"), outer_private_name_id);
        assert_eq!(returned_private_name_id("Heritage"), outer_private_name_id);
        assert_eq!(
            returned_private_name_id("Inner.read"),
            inner_private_name_id
        );
    }

    #[test]
    fn resolves_shadowed_private_names_in_a_nested_class_field_initializer() {
        let program = lower_script(
            "class Outer {
                set #value(next) {}
                field = class Inner {
                    #value;
                    write(receiver, next) { receiver.#value = next; }
                    read() { return this.#value; }
                };
            }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");

        let outer_setter = script
            .functions
            .iter()
            .find(|function| function.name == "set #value")
            .expect("outer setter should be lowered");
        let write = script
            .functions
            .iter()
            .find(|function| function.name == "Inner.write")
            .expect("inner writer should be lowered");
        let read = script
            .functions
            .iter()
            .find(|function| function.name == "Inner.read")
            .expect("inner reader should be lowered");

        let write_private_name_id = write
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Expression(TypedExpr {
                    expr:
                        ExprIr::PrivateWrite {
                            private_name_id, ..
                        },
                    ..
                }) => Some(*private_name_id),
                _ => None,
            })
            .expect("inner writer should write a private name");
        let read_private_name_id = read
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Return(TypedExpr {
                    expr:
                        ExprIr::PrivateRead {
                            private_name_id, ..
                        },
                    ..
                }) => Some(*private_name_id),
                _ => None,
            })
            .expect("inner reader should read a private name");

        assert_eq!(write_private_name_id, read_private_name_id);
        assert_eq!(write.private_name_ids["value"], write_private_name_id);
        assert_eq!(read.private_name_ids["value"], read_private_name_id);
        assert_ne!(outer_setter.private_name_ids["value"], read_private_name_id);
        assert_eq!(read.return_kind, ValueKind::Dynamic);
    }

    #[test]
    fn resolves_private_names_in_field_initializers_and_static_blocks() {
        let program = lower_script(
            "class C {
                #instance;
                field = this.#instance;
                static #staticValue;
                static field = this.#staticValue;
                static { this.#staticValue; }
            }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
            panic!("expected class declaration");
        };
        let ExprIr::ClassDefinition(class) = &init.expr else {
            panic!("expected class definition");
        };

        let private_read_id = |execution_kind: ClassElementExecutionKind| {
            let function = script
                .functions
                .iter()
                .find(|function| function.class_element_execution_kind == execution_kind)
                .unwrap_or_else(|| panic!("{execution_kind:?} should be lowered"));
            function
                .body
                .statements
                .iter()
                .find_map(|statement| {
                    let expression = match statement {
                        StatementIr::Expression(expression) | StatementIr::Return(expression) => {
                            expression
                        }
                        _ => return None,
                    };
                    match &expression.expr {
                        ExprIr::PrivateRead {
                            private_name_id, ..
                        } => Some(*private_name_id),
                        _ => None,
                    }
                })
                .unwrap_or_else(|| panic!("{execution_kind:?} should read a private name"))
        };

        assert_eq!(
            private_read_id(ClassElementExecutionKind::InstanceFieldInitializer),
            class.private_name_ids["instance"]
        );
        assert_eq!(
            private_read_id(ClassElementExecutionKind::StaticFieldInitializer),
            class.private_name_ids["staticValue"]
        );
        assert_eq!(
            private_read_id(ClassElementExecutionKind::StaticBlock),
            class.private_name_ids["staticValue"]
        );
    }

    #[test]
    fn class_declarations_have_mutable_lexical_bindings() {
        let program = lower_script("class C {} C = 1;");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        assert!(matches!(
            script.body.statements[0],
            StatementIr::Lexical {
                mode: BindingMode::Let,
                ..
            }
        ));
    }

    #[test]
    fn class_setters_capture_the_immutable_inner_name_binding() {
        let program = lower_script("var C2; class C { set value(next) { C2 = C; } }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[1] else {
            panic!("expected class declaration");
        };
        let ExprIr::ClassDefinition(class) = &init.expr else {
            panic!("expected class definition");
        };
        let name_binding = class
            .name_binding
            .as_ref()
            .expect("named class should own an inner name binding");
        let setter = script
            .functions
            .iter()
            .find(|function| function.class_kind == ClassFunctionKind::Setter)
            .expect("class setter should be lowered");

        assert_eq!(name_binding.environment.bindings.len(), 1);
        assert_eq!(setter.captured_bindings.len(), 1);
        let capture = &setter.captured_bindings[0];
        assert_eq!(capture.source_name, "C");
        assert_eq!(capture.name, name_binding.storage_name);
        assert_eq!(capture.mode, BindingMode::Const);
    }

    #[test]
    fn class_shape_retains_paired_getter_and_setter() {
        let program = lower_script("class C { get value() { return C; } set value(next) {} }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
            panic!("expected class declaration");
        };
        let Some(HeapShape::Object(class_shape)) = init.heap_shape.as_deref() else {
            panic!("expected class shape");
        };
        let Some(ObjectShapeProperty::Data(prototype)) = class_shape.properties.get("prototype")
        else {
            panic!("expected class prototype");
        };
        let Some(HeapShape::Object(prototype_shape)) = prototype.heap_shape.as_deref() else {
            panic!("expected prototype shape");
        };

        assert!(matches!(
            prototype_shape.properties.get("value"),
            Some(ObjectShapeProperty::Accessor {
                getter: Some(_),
                setter: Some(_),
            })
        ));
    }

    #[test]
    fn array_subclass_overrides_lower_as_runtime_property_calls() {
        let program = lower_script(
            "class A extends Array {
                push() { return 'custom push'; }
                join() { return 'custom join'; }
             }
             const a = new A();
             a.push(1);
             a.join();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");

        for (statement, expected_key) in script
            .body
            .statements
            .iter()
            .rev()
            .take(2)
            .rev()
            .zip(["push", "join"])
        {
            let StatementIr::Expression(expression) = statement else {
                panic!("expected runtime indirect call for {expected_key}: {statement:?}");
            };
            assert_eq!(expression.kind, ValueKind::String);
            let Some(TypedExpr {
                expr:
                    ExprIr::CallIndirect {
                        callee,
                        this_arg: Some(this_arg),
                        ..
                    },
                ..
            }) = indirect_call_body(expression)
            else {
                panic!("expected runtime indirect call for {expected_key}: {statement:?}");
            };
            assert!(matches!(
                &callee.expr,
                ExprIr::SpecOperation {
                    operation: SpecOperationIr::GetV,
                    operands,
                } if operands.len() == 2
                    && matches!(&operands[1].expr, ExprIr::String(key) if key == expected_key)
            ));
            assert!(matches!(
                this_arg.heap_shape.as_deref(),
                Some(HeapShape::Array(shape)) if shape.prototype.is_some()
            ));
        }
    }

    #[test]
    fn private_method_call_materializes_receiver_before_brand_checked_read() {
        let program = lower_script(
            "class A extends Array {
                #method() { return 1; }
                call() { return this.#method(); }
             }
             new A().call();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "A.call")
            .expect("class method should be lowered");
        let StatementIr::Return(TypedExpr {
            expr: ExprIr::MaterializeBinding { name, value, body },
            ..
        }) = &function.body.statements[0]
        else {
            panic!(
                "expected materialized private method call: {:?}",
                function.body
            );
        };
        assert!(matches!(value.expr, ExprIr::This));
        let ExprIr::CallIndirect {
            callee,
            this_arg: Some(this_arg),
            ..
        } = &body.expr
        else {
            panic!("expected indirect private method call body: {body:?}");
        };
        let ExprIr::PrivateRead { target, .. } = &callee.expr else {
            panic!("expected brand-checked private read: {callee:?}");
        };
        assert!(matches!(
            &target.expr,
            ExprIr::Identifier(target_name) if target_name == name
        ));
        assert!(matches!(
            &this_arg.expr,
            ExprIr::Identifier(this_name) if this_name == name
        ));
    }

    #[test]
    fn ordinary_method_call_materializes_compound_receiver_before_property_read() {
        let program = lower_script(
            "function make() { return { method() { return this; } }; } make().method();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::MaterializeBinding { name, value, body },
            ..
        }) = script
            .body
            .statements
            .last()
            .expect("method call statement should exist")
        else {
            panic!("expected materialized ordinary method call");
        };
        assert!(matches!(
            value.expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));
        let ExprIr::CallIndirect {
            callee,
            this_arg: Some(this_arg),
            ..
        } = &body.expr
        else {
            panic!("expected indirect method call body: {body:?}");
        };
        let ExprIr::PropertyRead { target, .. } = &callee.expr else {
            panic!("expected property read callee: {callee:?}");
        };
        assert!(matches!(
            &target.expr,
            ExprIr::Identifier(target_name) if target_name == name
        ));
        assert!(matches!(
            &this_arg.expr,
            ExprIr::Identifier(this_name) if this_name == name
        ));
    }

    #[test]
    fn computed_method_call_materializes_base_before_key_evaluation() {
        let program = lower_script(
            "function make() { return { method() { return this; } }; } function key() { return 'method'; } make()[key()]();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::MaterializeBinding { name, body, .. },
            ..
        }) = script
            .body
            .statements
            .last()
            .expect("computed method call statement should exist")
        else {
            panic!("expected materialized computed method call");
        };
        let ExprIr::CallIndirect {
            callee,
            this_arg: Some(this_arg),
            ..
        } = &body.expr
        else {
            panic!("expected indirect computed method call body: {body:?}");
        };
        let ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } = &callee.expr
        else {
            panic!("expected GetV callee: {callee:?}");
        };
        assert!(matches!(
            &operands[0].expr,
            ExprIr::Identifier(target_name) if target_name == name
        ));
        assert!(matches!(
            &this_arg.expr,
            ExprIr::Identifier(this_name) if this_name == name
        ));
        assert!(matches!(
            operands[1].expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));
    }

    #[test]
    fn carries_dynamic_class_heritage_to_runtime() {
        let program =
            lower_script("function make(Base) { class Derived extends Base {} return Derived; }");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("class_extends=1"));
    }

    #[test]
    fn narrows_identifier_used_as_object_property_key_to_string() {
        let program =
            lower_script("function get(obj, name) { return obj[name]; } get({ x: 1 }, \"x\");");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("property_reads=2"));
    }

    #[test]
    fn keeps_dynamic_string_property_key_on_possible_array_targets() {
        let program = lower_script(
            "function read(desc, name) { return desc.get[name]; } read({ get: function () {} }, \"length\");",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("property_reads=4"));
    }

    #[test]
    fn treats_typed_array_constructor_parameter_bytes_per_element_as_number() {
        let program =
            lower_script("function f(TA) { return 4 * TA.BYTES_PER_ELEMENT; } f(Int8Array);");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
    }

    #[test]
    fn lowers_objects_arrays_and_properties() {
        let program = lower_script("let o = { x: 1 }; let a = [1]; a[2] = 4; o.x;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("objects=1"));
        assert!(summary.contains("arrays=1"));
        assert!(summary.contains("property_reads=1"));
        assert!(summary.contains("property_writes=1"));
    }

    #[test]
    fn lowers_only_colon_proto_properties_as_prototype_setters() {
        fn object_literal(source: &str) -> TypedExpr {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.expect("script ir should exist");
            let StatementIr::Expression(expr) = script.body.statements.last().expect("expression")
            else {
                panic!("expected object literal expression");
            };
            expr.clone()
        }

        let prototype_setter = object_literal("({ __proto__: { marker: 1 } });");
        let ExprIr::ObjectLiteral(properties) = &prototype_setter.expr else {
            panic!("expected object literal");
        };
        assert!(matches!(
            properties.as_slice(),
            [ObjectPropertyIr::PrototypeSetter { .. }]
        ));
        let Some(HeapShape::Object(shape)) = prototype_setter.heap_shape.as_deref() else {
            panic!("expected object shape");
        };
        assert!(shape.prototype.is_some());
        assert!(!shape.properties.contains_key("__proto__"));

        let shorthand = object_literal("let __proto__ = 1; ({ __proto__ });");
        let ExprIr::ObjectLiteral(properties) = &shorthand.expr else {
            panic!("expected object literal");
        };
        assert!(matches!(
            properties.as_slice(),
            [ObjectPropertyIr::Data { .. }]
        ));

        let computed = object_literal("({ ['__proto__']: 1 });");
        let ExprIr::ObjectLiteral(properties) = &computed.expr else {
            panic!("expected object literal");
        };
        assert!(matches!(
            properties.as_slice(),
            [ObjectPropertyIr::Data { .. }]
        ));

        let method = object_literal("({ __proto__() {} });");
        let ExprIr::ObjectLiteral(properties) = &method.expr else {
            panic!("expected object literal");
        };
        assert!(matches!(
            properties.as_slice(),
            [ObjectPropertyIr::Method { .. }]
        ));
    }

    #[test]
    fn lowers_object_spreads_in_property_evaluation_order() {
        let program =
            lower_script("let source = { copied: 2 }; ({ before: 1, ...source, after: 3 });");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script ir should exist");
        let StatementIr::Expression(object) =
            script.body.statements.last().expect("object expression")
        else {
            panic!("expected object literal expression");
        };
        let ExprIr::ObjectLiteral(properties) = &object.expr else {
            panic!("expected object literal");
        };
        assert!(matches!(
            properties.as_slice(),
            [
                ObjectPropertyIr::Data { key: before, .. },
                ObjectPropertyIr::Spread { .. },
                ObjectPropertyIr::Data { key: after, .. },
            ] if before == "before" && after == "after"
        ));
        assert!(
            object.heap_shape.is_none(),
            "spread keys make the object shape dynamic"
        );
    }

    #[test]
    fn lowers_fractional_array_keys_as_named_properties() {
        let program = lower_script("const arr = [39, 42]; arr[1.1] = 'other prop'; arr[1.1];");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(write) = &script.body.statements[1] else {
            panic!("expected array property write");
        };
        let ExprIr::PropertyWrite { key, .. } = &write.expr else {
            panic!("expected property write, got {:?}", write.expr);
        };
        assert_eq!(key, &PropertyKeyIr::StaticString("1.1".to_string()));

        let StatementIr::Expression(read) = &script.body.statements[2] else {
            panic!("expected array property read");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &read.expr
        else {
            panic!("expected GetV operation, got {:?}", read.expr);
        };
        assert_eq!(*operation, SpecOperationIr::GetV);
        assert_eq!(operands.len(), 2);
        assert!(matches!(operands[1].expr, ExprIr::Number(_)));
        assert_eq!(read.kind, ValueKind::Dynamic);
    }

    #[test]
    fn preserves_well_known_symbols_as_computed_property_keys() {
        let program = lower_script(
            r#"let target = { "Symbol.asyncIterator": "string" };
target[Symbol.asyncIterator] = "symbol";
target[Symbol.asyncIterator];"#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(write) = &script.body.statements[1] else {
            panic!("expected property write");
        };
        let ExprIr::PropertyWrite { key, .. } = &write.expr else {
            panic!("expected property write, got {:?}", write.expr);
        };
        assert!(
            matches!(key, PropertyKeyIr::StringExpr(key) if key.kind == ValueKind::Symbol),
            "well-known Symbol write key must retain its Symbol kind, got {key:?}"
        );

        let StatementIr::Expression(read) = &script.body.statements[2] else {
            panic!("expected property read");
        };
        let ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } = &read.expr
        else {
            panic!("expected GetV operation, got {:?}", read.expr);
        };
        assert!(
            matches!(operands.as_slice(), [_, key] if key.kind == ValueKind::Symbol),
            "well-known Symbol read key must retain its Symbol kind, got {operands:?}"
        );
    }

    #[test]
    fn symbol_and_similarly_named_string_properties_do_not_share_shape_facts() {
        let program = lower_script(
            r#"let target = {
                [Symbol.iterator]: 1,
                "Symbol.iterator": "ordinary"
            };
            target[Symbol.iterator];
            target["Symbol.iterator"];"#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(symbol_read) = &script.body.statements[1] else {
            panic!("expected symbol property read");
        };
        assert_eq!(symbol_read.kind, ValueKind::Dynamic);

        let StatementIr::Expression(string_read) = &script.body.statements[2] else {
            panic!("expected string property read");
        };
        assert_eq!(string_read.kind, ValueKind::String);
    }

    #[test]
    fn distinct_computed_symbols_are_omitted_from_string_key_shapes() {
        let program = lower_script(
            r#"const first = Symbol("first");
            const second = Symbol("second");
            let target = { [first]: 1, [second]: "second" };"#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[2] else {
            panic!("expected object declaration");
        };
        let Some(HeapShape::Object(shape)) = init.heap_shape.as_deref() else {
            panic!("expected object shape");
        };
        assert!(shape.properties.is_empty());
    }

    #[test]
    fn computed_property_keys_respect_a_shadowed_symbol_binding() {
        let program = lower_script(
            r#"const Symbol = { iterator: "actual" };
let target = { actual: 1, "Symbol.iterator": 2 };
target[Symbol.iterator];"#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(read) = &script.body.statements[2] else {
            panic!("expected property read");
        };
        let ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } = &read.expr
        else {
            panic!("expected GetV operation, got {:?}", read.expr);
        };
        assert!(
            !matches!(
                operands.as_slice(),
                [_, TypedExpr {
                    expr: ExprIr::String(key),
                    ..
                }] if key == "Symbol.iterator"
            ),
            "shadowed Symbol key must not resolve to the well-known Symbol marker"
        );
    }

    #[test]
    fn specialized_iterator_reads_preserve_symbol_keys() {
        let program = lower_script(
            r#"[][Symbol.iterator];
""[Symbol.iterator];
[]["Symbol.iterator"];"#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");

        for statement in &script.body.statements[..2] {
            let StatementIr::Expression(read) = statement else {
                panic!("expected property read");
            };
            match &read.expr {
                ExprIr::PropertyRead { key, .. } => assert!(
                    matches!(key, PropertyKeyIr::StringExpr(key) if key.kind == ValueKind::Symbol),
                    "well-known iterator reads must retain their Symbol key, got {key:?}"
                ),
                ExprIr::SpecOperation {
                    operation: SpecOperationIr::GetV,
                    operands,
                } => assert!(
                    matches!(operands.as_slice(), [_, key] if key.kind == ValueKind::Symbol),
                    "well-known iterator reads must retain their Symbol operand, got {operands:?}"
                ),
                _ => panic!("expected property read, got {:?}", read.expr),
            }
        }

        let StatementIr::Expression(literal_read) = &script.body.statements[2] else {
            panic!("expected literal-string property read");
        };
        match &literal_read.expr {
            ExprIr::PropertyRead {
                key: literal_key, ..
            } => assert_eq!(
                literal_key,
                &PropertyKeyIr::StaticString("Symbol.iterator".to_string())
            ),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                operands,
            } => assert!(
                matches!(
                    operands.as_slice(),
                    [_, TypedExpr {
                        kind: ValueKind::String,
                        expr: ExprIr::String(key),
                        ..
                    }] if key == "Symbol.iterator"
                ),
                "literal iterator key must remain a String operand, got {operands:?}"
            ),
            _ => panic!(
                "expected literal-string property read, got {:?}",
                literal_read.expr
            ),
        }
    }

    #[test]
    fn computed_object_literal_keys_preserve_symbol_identity() {
        let program = lower_script(
            r#"({
  [Symbol.iterator]: 1,
  [Symbol.toPrimitive]() { return 2; },
  ["Symbol.iterator"]: 3
});"#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(object) = &script.body.statements[0] else {
            panic!("expected object literal");
        };
        let ExprIr::ObjectLiteral(properties) = &object.expr else {
            panic!("expected object literal, got {:?}", object.expr);
        };
        assert!(
            matches!(
                properties.as_slice(),
                [
                    ObjectPropertyIr::ComputedData { key: symbol_data, .. },
                    ObjectPropertyIr::ComputedMethod { key: symbol_method, .. },
                    ObjectPropertyIr::Data {
                        key: string_key,
                        ..
                    }
                ] if symbol_data.kind == ValueKind::Symbol
                    && symbol_method.kind == ValueKind::Symbol
                    && string_key == "Symbol.iterator"
            ),
            "computed Symbol keys and ordinary string keys must stay distinct: {properties:?}"
        );
    }

    #[test]
    fn lowers_runtime_number_array_keys_through_to_property_key() {
        for key_init in ["1", "1.1", "-1", "NaN", "Infinity"] {
            let source =
                format!("let key = {key_init}; let array = []; array[key] = 7; array[key];");
            let program = lower_script(&source);
            assert!(program.is_wasm_supported(), "{source}");
            let script = program.script.as_ref().expect("script ir should exist");

            let StatementIr::Expression(write) = &script.body.statements[2] else {
                panic!("expected array property write for {source}");
            };
            let ExprIr::PropertyWrite { key, .. } = &write.expr else {
                panic!("expected property write for {source}, got {:?}", write.expr);
            };
            assert!(
                matches!(key, PropertyKeyIr::StringExpr(key) if key.kind == ValueKind::Number),
                "runtime numeric write key must use ToPropertyKey for {source}, got {key:?}"
            );

            let StatementIr::Expression(read) = &script.body.statements[3] else {
                panic!("expected array property read for {source}");
            };
            let ExprIr::SpecOperation {
                operation,
                operands,
            } = &read.expr
            else {
                panic!("expected GetV operation for {source}, got {:?}", read.expr);
            };
            assert!(
                *operation == SpecOperationIr::GetV
                    && operands.len() == 2
                    && operands[1].kind == ValueKind::Number,
                "runtime numeric read key must use GetV for {source}, got {operands:?}"
            );
        }
    }

    #[test]
    fn folds_iterator_from_nullish_symbol_iterator_fallback() {
        let program = lower_script(
            "function* g() { yield 0; yield 1; yield 2; }
             let iter = (function () {
               let n = g();
               return { [Symbol.iterator]: null, next: () => n.next() };
             })();
             let array = Array.from(Iterator.from(iter));",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let array_init = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "array" => Some(init),
                _ => None,
            })
            .expect("array lexical should be present");
        let ExprIr::ArrayLiteral(elements) = &array_init.expr else {
            panic!("expected folded array literal, got {:?}", array_init.expr);
        };
        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn folds_iterator_from_nullish_symbol_iterator_fallback_after_reassignment() {
        let program = lower_script(
            "function* g() { yield 0; yield 1; yield 2; }
             let iter = (function () {
               let n = g();
               return { [Symbol.iterator]: 0, next: () => n.next() };
             })();
             iter = (function () {
               let n = g();
               return { [Symbol.iterator]: null, next: () => n.next() };
             })();
             let array = Array.from(Iterator.from(iter));",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let array_init = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "array" => Some(init),
                _ => None,
            })
            .expect("array lexical should be present");
        let ExprIr::ArrayLiteral(elements) = &array_init.expr else {
            panic!("expected folded array literal, got {:?}", array_init.expr);
        };
        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn keeps_iterator_from_wrapper_return_observable() {
        let program = lower_script(
            "const iter = {};
             const wrapper = Iterator.from(iter);
             const result = wrapper.return();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let result_init = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "result" => Some(init),
                _ => None,
            })
            .expect("result lexical should be present");
        assert!(indirect_call_body(result_init).is_some());
    }

    #[test]
    fn gives_iterator_from_wrapper_the_wrapper_prototype_chain() {
        let program = lower_script(
            "const iter = { next() { return { done: true, value: undefined }; } };
             const wrapperPrototype = Object.getPrototypeOf(Iterator.from(iter));
             const iteratorPrototype = Object.getPrototypeOf(wrapperPrototype);",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let wrapper_prototype = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "wrapperPrototype" => Some(init),
                _ => None,
            })
            .expect("wrapper prototype lexical should be present");
        let iterator_prototype = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "iteratorPrototype" => {
                    Some(init)
                }
                _ => None,
            })
            .expect("iterator prototype lexical should be present");
        assert!(wrapper_prototype.heap_shape.is_some());
        assert_eq!(
            iterator_prototype.heap_shape.as_deref(),
            Some(ScriptLowerer::iterator_prototype_shape().as_ref())
        );
    }

    #[test]
    fn iterator_from_preserves_existing_iterator_instances() {
        let program = lower_script(
            "const GeneratorPrototype = Object.getPrototypeOf((function* () {})());
             const FromPrototype = Object.getPrototypeOf(Iterator.from((function* () {})()));",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let generator_prototype = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "GeneratorPrototype" => {
                    Some(init)
                }
                _ => None,
            })
            .expect("generator prototype lexical should be present");
        let from_prototype = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "FromPrototype" => Some(init),
                _ => None,
            })
            .expect("from prototype lexical should be present");
        assert_eq!(
            generator_prototype.heap_shape.as_deref(),
            from_prototype.heap_shape.as_deref()
        );
    }

    #[test]
    fn registers_zero_suspension_generator_declarations() {
        let program = lower_script(
            "function* empty() {}
             function* returns() { return 1; }
             function* throws() { throw 2; }",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let generators = script
            .functions
            .iter()
            .filter(|function| function.execution_kind == FunctionExecutionKind::Generator)
            .collect::<Vec<_>>();
        assert_eq!(generators.len(), 3);
        assert_eq!(
            generators
                .iter()
                .map(|function| function.name.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["empty", "returns", "throws"])
        );
        for function in generators {
            assert_zero_suspension_generator(function);
        }
    }

    #[test]
    fn lowers_zero_suspension_generator_expressions_as_function_values() {
        let program = lower_script(
            "let returns = function* named() { return 1; };
             let throws = function* () { throw 2; };",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let generator_ids = script
            .functions
            .iter()
            .filter(|function| function.execution_kind == FunctionExecutionKind::Generator)
            .map(|function| {
                assert_zero_suspension_generator(function);
                function.id.clone()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(generator_ids.len(), 2);
        let lexical_targets = script
            .body
            .statements
            .iter()
            .filter_map(|statement| match statement {
                StatementIr::Lexical { init, .. } => match &init.expr {
                    ExprIr::FunctionValue(function_id) => Some(function_id.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(lexical_targets, generator_ids);
    }

    #[test]
    fn records_explicit_inferred_and_anonymous_generator_expression_names() {
        let program = lower_script(
            "let inferred = function* () {};
             let values = [function* explicit() {}, function* () {}];",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let names = script
            .functions
            .iter()
            .filter(|function| function.execution_kind == FunctionExecutionKind::Generator)
            .map(|function| function.name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(names, BTreeSet::from(["", "explicit", "inferred"]));
    }

    #[test]
    fn lowers_arrow_function_from_generator_object_parameter_default() {
        let program = lower_script("let f = function* ({ arrow = () => 1 }) {};");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script
            .functions
            .iter()
            .any(|function| function.flavor == FunctionFlavor::Arrow && function.name == "arrow"));
    }

    #[test]
    fn lowers_function_expression_from_nested_generator_object_parameter_default() {
        let program = lower_script(
            "let f = function* ({ nested: { ordinary = function () { return 1; } } }) {};",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.functions.iter().any(|function| {
            function.execution_kind == FunctionExecutionKind::Ordinary
                && function.flavor == FunctionFlavor::Ordinary
                && function.is_expression
                && function.name == "ordinary"
        }));
    }

    #[test]
    fn lowers_generator_function_instanceof_without_requiring_constructability() {
        let program =
            lower_script("let generator = function* () {}; generator() instanceof generator;");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("instanceofs=1"));
    }

    #[test]
    fn records_generator_default_parameter_tdz_and_prior_binding_reads() {
        let program = lower_script(
            "let selfRead = function* (value = value) {};
             let laterRead = function* (value = later, later) {};
             let priorRead = function* (value = 1, later = value) {};",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");

        for function_name in ["selfRead", "laterRead"] {
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("missing `{function_name}`"));
            assert!(matches!(
                function.params[0]
                    .default_init
                    .as_ref()
                    .map(|init| &init.expr),
                Some(ExprIr::RuntimeThrow {
                    name: NativeErrorKind::ReferenceError,
                    ..
                })
            ));
        }

        let prior_read = script
            .functions
            .iter()
            .find(|function| function.name == "priorRead")
            .expect("missing `priorRead`");
        assert!(matches!(
            prior_read.params[1]
                .default_init
                .as_ref()
                .map(|init| &init.expr),
            Some(ExprIr::Identifier(name)) if name == "value"
        ));
    }

    #[test]
    fn records_zero_suspension_object_and_class_generator_methods() {
        let program = lower_script(
            "const object = {
                 *returns() { return 1; },
                 *throws() { throw 2; }
             };
             class Example {
                 *empty() {}
                 static *returns() { return 1; }
             }",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let generators = script
            .functions
            .iter()
            .filter(|function| function.execution_kind == FunctionExecutionKind::Generator)
            .collect::<Vec<_>>();
        assert_eq!(generators.len(), 4);
        for function in generators {
            assert_zero_suspension_generator(function);
        }
    }

    #[test]
    fn records_linear_generator_suspension_edges() {
        let program = lower_script("function* sequence() { yield 1; yield 2; return yield 3; }");
        assert!(program.is_wasm_supported());
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "sequence")
            .expect("generator should be registered");
        assert_eq!(function.execution_kind, FunctionExecutionKind::Generator);
        assert_eq!(
            function.generator_plan,
            Some(GeneratorPlanIr {
                entry_state: 0,
                state_count: 4,
                suspension_points: vec![
                    GeneratorSuspensionPointIr {
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    GeneratorSuspensionPointIr {
                        suspend_state: 1,
                        resume_state: 2,
                    },
                    GeneratorSuspensionPointIr {
                        suspend_state: 2,
                        resume_state: 3,
                    },
                ],
            })
        );
        assert_eq!(
            function
                .body
                .statements
                .iter()
                .filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. }))
                .count(),
            3
        );
    }

    #[test]
    fn rejects_generator_loops_with_unmodelled_loop_control() {
        for source in [
            "function* sequence() { for (let i = 0; i < 1; i++) { yield i; break; } }",
            "function* sequence() { while (true) { yield 1; continue; } }",
        ] {
            let program = lower_script(source);
            assert!(
                !program.is_wasm_supported(),
                "loop control must not enter the linear generator plan: {source}"
            );
        }
    }

    #[test]
    fn rejects_generator_loops_with_captured_per_iteration_bindings() {
        let uncaptured =
            lower_script("function* sequence() { for (let i = 0; i < 2; i++) { yield i; } }");
        assert!(
            uncaptured.is_wasm_supported(),
            "{:?}",
            uncaptured.diagnostics
        );

        let program = lower_script(
            "function* sequence() {
                 for (let i = 0; i < 2; i++) {
                     yield function () { return i; };
                 }
             }",
        );

        assert!(
            !program.is_wasm_supported(),
            "captured for-let bindings require a resumable per-iteration environment"
        );
    }

    #[test]
    fn records_nested_yield_as_two_linear_suspensions() {
        let program = lower_script("function* sequence() { yield yield 1; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "sequence")
            .expect("generator should be registered");
        assert_eq!(
            function.generator_plan,
            Some(GeneratorPlanIr {
                entry_state: 0,
                state_count: 3,
                suspension_points: vec![
                    GeneratorSuspensionPointIr {
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    GeneratorSuspensionPointIr {
                        suspend_state: 1,
                        resume_state: 2,
                    },
                ],
            })
        );
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical { .. },
                        StatementIr::GeneratorYield { .. },
                        StatementIr::GeneratorYield { .. }
                    ]
                )
        ));
    }

    #[test]
    fn records_regexp_yield_assignment_resume_mode() {
        let program = lower_script(
            "let received;
             function* sequence() { received = yield/abc/i; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "sequence")
            .expect("generator should be registered");
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::GeneratorYield {
                value: TypedExpr { expr: ExprIr::RegExpLiteral { .. }, .. },
                resume_mode: GeneratorResumeModeIr::AssignIdentifier(name),
                ..
            }] if name == "received"
        ));
    }

    #[test]
    fn records_discarded_generator_expression_suspensions() {
        let program = lower_script(
            "function* grouping() { (yield 1); }
             function* array() { [yield 1]; }
             function* block() { { yield 1; } }
             function* comma() { yield 1, yield 2; }
             function* add() { (yield 1) + (yield 2); }
             function* conditional() { (yield 1) ? yield 2 : yield 3; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        for (name, state_count, suspension_count) in [
            ("grouping", 2, 1),
            ("array", 2, 1),
            ("block", 2, 1),
            ("comma", 3, 2),
            ("add", 3, 2),
            ("conditional", 5, 3),
        ] {
            let function = script
                .functions
                .iter()
                .find(|function| function.name == name)
                .unwrap_or_else(|| panic!("generator `{name}` should be registered"));
            let plan = function
                .generator_plan
                .as_ref()
                .unwrap_or_else(|| panic!("generator `{name}` should have a suspension plan"));
            assert_eq!(plan.state_count, state_count, "generator `{name}`");
            assert_eq!(
                plan.suspension_points.len(),
                suspension_count,
                "generator `{name}`"
            );
        }
        let conditional = script
            .functions
            .iter()
            .find(|function| function.name == "conditional")
            .expect("conditional generator should be registered");
        assert!(matches!(
            conditional.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical { .. },
                        StatementIr::GeneratorYield { .. },
                        StatementIr::GeneratorIf { .. }
                    ]
                )
        ));
        let add = script
            .functions
            .iter()
            .find(|function| function.name == "add")
            .expect("add generator should be registered");
        assert!(matches!(
            add.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical { .. },
                        StatementIr::GeneratorYield { .. },
                        StatementIr::Lexical { .. },
                        StatementIr::Lexical { .. },
                        StatementIr::GeneratorYield { .. },
                        StatementIr::Expression(TypedExpr {
                            expr: ExprIr::CoerciveAdd { .. },
                            ..
                        })
                    ]
                )
        ));
    }

    #[test]
    fn records_generator_template_interpolation_suspensions() {
        let program = lower_script(
            "let output;
             function before() { return 2; }
             function* sequence() {
                 output = `1${before()}3${yield 4}5${yield 6}7`;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "sequence")
            .expect("generator should be registered");
        assert_eq!(
            function.generator_plan,
            Some(GeneratorPlanIr {
                entry_state: 0,
                state_count: 3,
                suspension_points: vec![
                    GeneratorSuspensionPointIr {
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    GeneratorSuspensionPointIr {
                        suspend_state: 1,
                        resume_state: 2,
                    },
                ],
            })
        );
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name.starts_with("$generator.template.")));
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if statements.iter().filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. })).count() == 2
        ));
    }

    #[test]
    fn records_generator_suspensions_inside_with() {
        let program = lower_script(
            "function* sequence() {
                 let x = 1;
                 yield x;
                 with ({ x: 2 }) {
                     yield x;
                     x = 3;
                     yield x;
                 }
                 yield x;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "sequence")
            .expect("generator should be registered");
        let plan = function
            .generator_plan
            .as_ref()
            .expect("generator should have a suspension plan");
        assert_eq!(plan.state_count, 5);
        assert_eq!(plan.suspension_points.len(), 4);
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name.starts_with("$generator.with.")));
    }

    #[test]
    fn records_linear_async_await_resume_state() {
        let program = lower_script(
            "async function resume() {
                 let retained = 40;
                 let received;
                 received = await 2;
                 return retained + received;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "resume")
            .expect("async function should be registered");

        assert_eq!(function.execution_kind, FunctionExecutionKind::Async);
        assert!(!function.constructable);
        assert!(function.body.statements.iter().any(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 0,
                    resume_state: 1,
                    resume_mode: AsyncResumeModeIr::AssignIdentifier(name),
                    ..
                } if name == "received"
            )
        }));
    }

    #[test]
    fn lowers_async_await_call_in_lexical_initializer() {
        let program = lower_script(
            "async function collect(Constructor) {
                 let result = await Array.fromAsync.call(Constructor, [1, 2]);
                 return result;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "collect")
            .expect("async function should be registered");
        let StatementIr::LexicalBlock(statements) = &function.body.statements[0] else {
            panic!(
                "awaited lexical initializer should lower through a lexical block: {:?}",
                function.body.statements
            );
        };
        let [StatementIr::Lexical {
            name: received_name,
            ..
        }, StatementIr::AsyncAwait {
            resume_mode: AsyncResumeModeIr::AssignIdentifier(resume_name),
            ..
        }, StatementIr::Lexical {
            name: result_name,
            init:
                TypedExpr {
                    expr: ExprIr::Identifier(init_name),
                    ..
                },
            ..
        }] = statements.as_slice()
        else {
            panic!(
                "awaited lexical initializer should stage its resumed value: {:?}",
                function.body.statements
            );
        };

        assert_eq!(received_name, resume_name);
        assert_eq!(received_name, init_name);
        assert_eq!(result_name, "result");
    }

    #[test]
    fn lowers_nested_async_await_in_expression_statement() {
        let program = lower_script(
            "async function inspect(promise) {
                 assert.sameValue((await promise).value, 1);
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "inspect")
            .expect("async function should be registered");
        let StatementIr::LexicalBlock(statements) = &function.body.statements[0] else {
            panic!(
                "nested await should lower through a lexical block: {:?}",
                function.body.statements
            );
        };

        assert!(statements.iter().any(|statement| matches!(
            statement,
            StatementIr::AsyncAwait {
                suspend_state: 0,
                resume_state: 1,
                resume_mode: AsyncResumeModeIr::AssignIdentifier(_),
                ..
            }
        )));
        assert!(matches!(
            statements.last(),
            Some(StatementIr::Expression(_))
        ));
    }

    #[test]
    fn lowers_direct_await_elements_in_lexical_array_initializer() {
        let program = lower_script(
            "async function collect(before, first, between, second, after) {
                 const reports = [
                     before(),
                     await first(),
                     between(),
                     await second(),
                     after(),
                 ];
                 return reports;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "collect")
            .expect("async function should be registered");
        let StatementIr::LexicalBlock(statements) = &function.body.statements[0] else {
            panic!(
                "awaited array initializer should lower through a lexical block: {:?}",
                function.body.statements
            );
        };
        let await_states = statements
            .iter()
            .filter_map(|statement| {
                let StatementIr::AsyncAwait {
                    suspend_state,
                    resume_state,
                    ..
                } = statement
                else {
                    return None;
                };
                Some((*suspend_state, *resume_state))
            })
            .collect::<Vec<_>>();
        assert_eq!(await_states, vec![(0, 1), (1, 2)]);
        let Some(StatementIr::Lexical {
            name,
            init:
                TypedExpr {
                    expr: ExprIr::ArrayLiteral(elements),
                    ..
                },
            ..
        }) = statements.last()
        else {
            panic!(
                "resumed array elements should initialize the declared binding: {:?}",
                function.body.statements
            );
        };
        assert_eq!(name, "reports");
        assert_eq!(elements.len(), 5);
        assert!(elements
            .iter()
            .all(|element| matches!(element.expr, ExprIr::Identifier(_))));
    }

    #[test]
    fn rejects_composite_and_spread_awaited_lexical_array_initializers() {
        let composite = lower_script(
            "async function collect(source) {
                 const reports = [consume(await source)];
                 return reports;
             }",
        );
        assert!(!composite.is_wasm_supported());
        assert!(composite.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("async lexical array initializer composite await element")));

        let spread = lower_script(
            "async function collect(source, rest) {
                 const reports = [await source, ...rest];
                 return reports;
             }",
        );
        assert!(!spread.is_wasm_supported());
        assert!(spread.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("async lexical array initializer spread")));
    }

    #[test]
    fn lowers_eager_arithmetic_await_declaration_initializers() {
        let program = lower_script(
            "async function calculate(x, first, second) {
                 let lexical = await first() * x;
                 var variable = -(await second()) + lexical;
                 return variable;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "calculate")
            .expect("async function should be registered");
        let await_states = function
            .body
            .statements
            .iter()
            .flat_map(|statement| match statement {
                StatementIr::LexicalBlock(statements) => statements.as_slice(),
                statement => std::slice::from_ref(statement),
            })
            .filter_map(|statement| {
                let StatementIr::AsyncAwait {
                    suspend_state,
                    resume_state,
                    ..
                } = statement
                else {
                    return None;
                };
                Some((*suspend_state, *resume_state))
            })
            .collect::<Vec<_>>();

        assert_eq!(await_states, vec![(0, 1), (1, 2)]);
        let StatementIr::LexicalBlock(var_statements) = &function.body.statements[1] else {
            panic!(
                "awaited var initializer should preserve its predeclared binding: {:?}",
                function.body.statements
            );
        };
        assert!(matches!(
            var_statements.first(),
            Some(StatementIr::Var(declarators))
                if matches!(
                    declarators.as_slice(),
                    [VarDeclaratorIr { name, init: None }] if name == "variable"
                )
        ));
        assert!(matches!(
            var_statements.last(),
            Some(StatementIr::Expression(TypedExpr {
                expr: ExprIr::AssignIdentifier { name, .. },
                ..
            })) if name == "variable"
        ));
    }

    #[test]
    fn stages_eager_arithmetic_operands_before_later_awaits() {
        let program = lower_script(
            "async function calculate(before, first, between, second) {
                 const result =
                     before() + await first() * between() + await second();
                 return result;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "calculate")
            .expect("async function should be registered");
        let StatementIr::LexicalBlock(statements) = &function.body.statements[0] else {
            panic!(
                "composite await initializer should lower through a lexical block: {:?}",
                function.body.statements
            );
        };
        let await_indexes = statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| {
                matches!(statement, StatementIr::AsyncAwait { .. }).then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(await_indexes.len(), 2);
        assert!(matches!(
            statements.first(),
            Some(StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::CallIndirect { .. },
                        ..
                    },
                ..
            }) if name.starts_with("$async.binary.lhs.")
        ));
        assert!(await_indexes[0] > 0);
        assert!(statements[await_indexes[0] + 1..await_indexes[1]]
            .iter()
            .any(|statement| matches!(
                statement,
                StatementIr::Lexical { name, .. }
                    if name.starts_with("$async.binary.lhs.")
            )));
        assert!(matches!(
            statements.last(),
            Some(StatementIr::Lexical { name, .. }) if name == "result"
        ));
    }

    #[test]
    fn rejects_branch_sensitive_await_declaration_initializers() {
        for source in [
            "async function inspect(source) { let value = source && await source; }",
            "async function inspect(source) { var value = source ? await source : 0; }",
            "async function inspect(source, key) { let value = source?.[await key]; }",
        ] {
            let program = lower_script(source);
            assert!(!program.is_wasm_supported());
            assert!(program.diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("branch-sensitive await expression")));
        }
    }

    #[test]
    fn lowers_awaited_optional_chain_in_var_initializer() {
        let program = lower_script(
            "async function inspect(source, key) {
                 var result = await source?.[key()];
                 return result;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "inspect")
            .expect("async function should be registered");
        let StatementIr::LexicalBlock(statements) = &function.body.statements[0] else {
            panic!(
                "awaited optional chain should lower through a lexical block: {:?}",
                function.body.statements
            );
        };

        assert!(statements.iter().any(|statement| matches!(
            statement,
            StatementIr::AsyncAwait {
                suspend_state: 0,
                resume_state: 1,
                resume_mode: AsyncResumeModeIr::AssignIdentifier(_),
                ..
            }
        )));
        assert!(matches!(
            statements.first(),
            Some(StatementIr::Var(declarators))
                if matches!(
                    declarators.as_slice(),
                    [VarDeclaratorIr {
                        name,
                        init: None,
                    }] if name == "result"
                )
        ));
    }

    #[test]
    fn awaited_var_redeclaration_discards_static_binding_facts() {
        let program = lower_script(
            "async function inspect(promise) {
                 var value = 'old';
                 var value = await promise;
                 return value.length;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "inspect")
            .expect("async function should be registered");
        let return_value = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Return(value) => Some(value),
                _ => None,
            })
            .expect("async function should return the awaited value length");

        assert!(matches!(
            return_value.expr,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                ..
            }
        ));
    }

    #[test]
    fn skips_await_in_statically_nullish_optional_chain_key() {
        let program = lower_script(
            "async function inspect(reject) {
                 assert.sameValue(undefined?.[await reject()], undefined);
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "inspect")
            .expect("async function should be registered");
        let StatementIr::LexicalBlock(statements) = &function.body.statements[0] else {
            panic!(
                "optional chain should retain its expression statement: {:?}",
                function.body.statements
            );
        };

        assert!(
            !statements
                .iter()
                .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })),
            "short-circuited key must not suspend: {statements:?}"
        );
        // Async operand staging may hoist side-effect-free receiver reads into
        // `$async.operand.N` temporaries ahead of the call, so the block is not
        // required to be exactly one statement. What matters is that the
        // short-circuited key never suspends (asserted above) and that the
        // chain still lowers to a single evaluated expression at the end.
        assert!(
            matches!(statements.last(), Some(StatementIr::Expression(_))),
            "optional chain should still end in its expression statement: {statements:?}"
        );
        assert!(
            statements.iter().rev().skip(1).all(
                |statement| matches!(statement, StatementIr::Lexical { name, .. }
                    if name.starts_with("$async.operand."))
            ),
            "only async operand staging may precede it: {statements:?}"
        );
    }

    #[test]
    fn skips_await_in_side_effecting_statically_nullish_optional_chain_key() {
        let program = lower_script(
            "async function inspect(reject) {
                 let calls = 0;
                 let value = (calls += 1, undefined)?.[await reject()];
                 return value;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "inspect")
            .expect("async function should be registered");
        assert!(
            !function
                .body
                .statements
                .iter()
                .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })),
            "short-circuited key must not suspend: {:?}",
            function.body.statements
        );
        assert!(matches!(
            &function.body.statements[1],
            StatementIr::Lexical {
                init: TypedExpr {
                    expr: ExprIr::MaterializeBinding { .. },
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn collects_nested_async_generator_declarations_with_exact_source() {
        let declaration = "async function* stream(source) { yield await source; }";
        let program = lower_script(&format!(
            "function outer() {{ {declaration} return stream; }}"
        ));
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("nested async generator declaration should be collected");

        assert_eq!(
            function.execution_kind,
            FunctionExecutionKind::AsyncGenerator
        );
        assert!(!function.constructable);
        assert_eq!(
            function.to_string_representation,
            CallableToStringRepresentation::ExactSource(declaration.to_string())
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical { .. },
                        StatementIr::AsyncAwait {
                            suspend_state: 0,
                            resume_state: 1,
                            ..
                        },
                        StatementIr::GeneratorYield {
                            suspend_state: 1,
                            resume_state: 2,
                            ..
                        }
                    ]
                )
        ));
    }

    #[test]
    fn async_generator_yield_await_stages_the_awaited_binding_into_yield() {
        let program = lower_script("async function* stream(source) { yield await source; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");

        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical { name: binding, .. },
                        StatementIr::AsyncAwait {
                            resume_mode: AsyncResumeModeIr::AssignIdentifier(await_binding),
                            ..
                        },
                        StatementIr::GeneratorYield {
                            value: TypedExpr {
                                expr: ExprIr::Identifier(yield_binding),
                                ..
                            },
                            delegate: false,
                            ..
                        }
                    ] if binding == await_binding
                        && await_binding == yield_binding
                )
        ));
    }

    #[test]
    fn async_generator_yield_star_preserves_the_delegation_boundary() {
        let program = lower_script("async function* outer(source) { yield* source; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "outer")
            .expect("async generator declaration should be collected");

        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::GeneratorYield {
                value: TypedExpr {
                    expr: ExprIr::Identifier(source),
                    ..
                },
                delegate: true,
                suspend_state: 0,
                resume_state: 1,
                ..
            }] if source == "source"
        ));
    }

    #[test]
    fn async_generator_yield_star_assigns_its_completion_to_var() {
        let program = lower_script(
            "async function* outer(source) { var completion = yield* source; return completion; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "outer")
            .expect("async generator declaration should be collected");

        assert!(
            matches!(
                function.body.statements.as_slice(),
                [
                    StatementIr::LexicalBlock(statements),
                    StatementIr::AsyncAwait {
                        value: TypedExpr {
                            expr: ExprIr::Identifier(completion),
                            ..
                        },
                        resume_mode: AsyncResumeModeIr::Return,
                        ..
                    }
                ] if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Var(declarations),
                        StatementIr::GeneratorYield {
                            delegate: true,
                            resume_mode: GeneratorResumeModeIr::AssignIdentifier(binding),
                            ..
                        }
                    ] if matches!(declarations.as_slice(), [VarDeclaratorIr { name, init: None }] if name == binding)
                        && binding == completion
                )
            ),
            "{:#?}",
            function.body.statements
        );
    }

    #[test]
    fn async_generator_yield_star_initializes_lexical_binding_after_delegation() {
        let program = lower_script(
            "async function* outer(source) { const completion = yield* source; return completion; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "outer")
            .expect("async generator declaration should be collected");

        assert!(
            matches!(
                function.body.statements.as_slice(),
                [
                    StatementIr::LexicalBlock(statements),
                    StatementIr::AsyncAwait {
                        value: TypedExpr {
                            expr: ExprIr::Identifier(returned_completion),
                            ..
                        },
                        resume_mode: AsyncResumeModeIr::Return,
                        ..
                    }
                ] if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical {
                            mode: BindingMode::Let,
                            name: staged_completion,
                            init: TypedExpr {
                                expr: ExprIr::Undefined,
                                ..
                            },
                        },
                        StatementIr::GeneratorYield {
                            delegate: true,
                            resume_mode: GeneratorResumeModeIr::AssignIdentifier(received_completion),
                            ..
                        },
                        StatementIr::Lexical {
                            mode: BindingMode::Const,
                            name: lexical_completion,
                            init: TypedExpr {
                                expr: ExprIr::Identifier(initializer_completion),
                                ..
                            },
                        }
                    ] if staged_completion == received_completion
                        && received_completion == initializer_completion
                        && lexical_completion == returned_completion
                )
            ),
            "{:#?}",
            function.body.statements
        );
    }

    #[test]
    fn nested_async_generator_yields_consume_resumable_states_in_execution_order() {
        let program = lower_script("async function* stream() { yield yield 1; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");

        assert_eq!(
            function.resumable_plan,
            Some(ResumablePlanIr {
                entry_state: 0,
                state_count: 3,
                suspension_points: vec![
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 1,
                        resume_state: 2,
                    },
                ],
            })
        );
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical { .. },
                        StatementIr::GeneratorYield {
                            suspend_state: 0,
                            resume_state: 1,
                            resume_mode: GeneratorResumeModeIr::AssignIdentifier(_),
                            ..
                        },
                        StatementIr::GeneratorYield {
                            suspend_state: 1,
                            resume_state: 2,
                            resume_mode: GeneratorResumeModeIr::Ignore,
                            ..
                        }
                    ]
                )
        ));
    }

    #[test]
    fn async_generator_yield_branches_reserve_a_distinct_merge_state() {
        let program = lower_script(
            "async function* choose(flag) { if (flag) yield 1; else yield 2; yield 3; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "choose")
            .expect("async generator declaration should be collected");

        assert_eq!(
            function.resumable_plan,
            Some(ResumablePlanIr {
                entry_state: 0,
                state_count: 5,
                suspension_points: vec![
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 1,
                        resume_state: 2,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 3,
                        resume_state: 4,
                    },
                ],
            })
        );
        assert!(matches!(
            function.body.statements.as_slice(),
            [
                StatementIr::GeneratorIf {
                    entry_state: 0,
                    then_resume_state: Some(1),
                    else_resume_state: Some(2),
                    exit_state: 3,
                    ..
                },
                StatementIr::GeneratorYield {
                    suspend_state: 3,
                    resume_state: 4,
                    ..
                }
            ]
        ));
    }

    #[test]
    fn async_generator_loop_exits_into_the_next_preplanned_suspension() {
        let program = lower_script(
            "async function* stream() { for (let i = 0; i < 3; i++) { yield i; } yield 9; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");

        let [StatementIr::GeneratorLoop {
            entry_state: 0,
            resume_state: 1,
            exit_state: 1,
            suspension_statement,
            ..
        }, StatementIr::GeneratorYield {
            suspend_state: 1,
            resume_state: 2,
            ..
        }] = function.body.statements.as_slice()
        else {
            panic!(
                "expected resumable loop followed by the next yield: {:#?}",
                function.body.statements
            );
        };
        assert!(matches!(
            suspension_statement.as_ref(),
            StatementIr::GeneratorYield {
                suspend_state: 0,
                resume_state: 1,
                delegate: false,
                ..
            }
        ));
    }

    #[test]
    fn plain_async_for_loop_await_lowers_to_a_resumable_loop() {
        // Without this the loop lowers to a straight-line `StatementIr::For`
        // holding the await: the async driver re-enters the body from the top,
        // so the loop restarts at iteration zero and the suspension, already
        // past its state guard, never fires again.
        let program = lower_script(
            "(async function(){ let t = 0; for (let i = 0; i < 3; i++) { t += await Promise.resolve(i); } print(t); })();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .first()
            .expect("async function expression should be collected");
        assert_eq!(function.execution_kind, FunctionExecutionKind::Async);
        assert!(function.resumable_plan.is_none());

        let [StatementIr::Lexical { name, .. }, StatementIr::GeneratorLoop {
            init: Some(ForInitIr::Lexical { .. }),
            test: Some(_),
            update: Some(_),
            suspension_statement,
            after_suspension,
            entry_state: 0,
            resume_state: 1,
            exit_state: 2,
            ..
        }, StatementIr::Expression(_)] = function.body.statements.as_slice()
        else {
            panic!(
                "expected a resumable await loop between the accumulator and the print: {:#?}",
                function.body.statements
            );
        };
        assert_eq!(name, "t");
        assert!(matches!(
            suspension_statement.as_ref(),
            StatementIr::AsyncAwait {
                suspend_state: 0,
                resume_state: 1,
                ..
            }
        ));
        // `t += <awaited>` has to land after the suspension, or the accumulator
        // is re-read from before the await on every resume.
        assert_eq!(after_suspension.len(), 1);
    }

    #[test]
    fn plain_async_while_loop_await_lowers_to_a_resumable_loop() {
        let program = lower_script(
            "(async function(){ let n = 0; while (n < 3) { n++; await Promise.resolve(0); } print(n); })();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .first()
            .expect("async function expression should be collected");
        assert!(
            function
                .body
                .statements
                .iter()
                .any(|statement| matches!(statement, StatementIr::GeneratorLoop { .. })),
            "{:#?}",
            function.body.statements
        );
    }

    #[test]
    fn plain_async_for_of_array_body_await_lowers_to_an_index_loop() {
        let program = lower_script(
            "(async function(){ const out = []; for (const x of [1,2,3]) { out.push(await Promise.resolve(x)); } print(out); })();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .first()
            .expect("async function expression should be collected");

        let Some(StatementIr::GeneratorLoop {
            init: Some(ForInitIr::LexicalBlock(head)),
            test: Some(_),
            update: Some(_),
            before_suspension,
            ..
        }) = function
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::GeneratorLoop { .. }))
        else {
            panic!(
                "for-of with a body await should become an index loop: {:#?}",
                function.body.statements
            );
        };
        // The array and the cursor both have to survive the suspension, so they
        // are hoisted into the loop head and given activation-record slots.
        assert_eq!(head.len(), 2);
        for binding in head {
            assert!(
                function
                    .owned_env_bindings
                    .iter()
                    .any(|owned| owned.name == binding.name),
                "`{}` must live in the activation record: {:?}",
                binding.name,
                function.owned_env_bindings
            );
        }
        assert!(matches!(
            before_suspension.first(),
            Some(StatementIr::Lexical { .. })
        ));
    }

    #[test]
    fn rejects_async_loop_awaits_with_no_resumable_shape() {
        // Each of these used to compile to a loop that ran its body once and
        // then reused the first resumed value for every later iteration.
        for (source, message) in [
            (
                "(async function(){ for (let i = 0; i < 2; i++) { try { await 0; } catch (e) {} } })();",
                "async loop body did not lower to one direct await",
            ),
            (
                "(async function(){ for (let i = 0; i < 2; i++) { await 0; break; } })();",
                "async loop with await requires an eager loop head without break or continue",
            ),
            (
                "(async function(){ for (let i = 0; i < 2; i++) { await 0; await 1; } })();",
                "async loop body did not lower to one direct await",
            ),
            (
                "(async function(){ let n = 0; do { n++; await 0; } while (n < 2); })();",
                "await inside a do-while loop",
            ),
            (
                "(async function(){ for (const k in { a: 1 }) { await 0; } })();",
                "await inside a for-in loop",
            ),
            (
                "(async function(){ for (const c of \"ab\") { await 0; } })();",
                "async for-of with a body await requires an array iterable and a plain binding",
            ),
        ] {
            let program = lower_script(source);
            assert!(!program.is_wasm_supported(), "{source} should not compile");
            assert!(
                program
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.message.contains(message)),
                "{source}: {:?}",
                program.diagnostics
            );
        }
    }

    #[test]
    fn async_generator_loop_reuses_one_await_state_across_iterations() {
        let program = lower_script(
            "async function* callAsync(iterations, pushAwait) {
                 for (let i = 0; i < iterations; i++) {
                     await pushAwait(i);
                 }
                 return 0;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "callAsync")
            .expect("async generator declaration should be collected");

        let [StatementIr::GeneratorLoop {
            init: Some(ForInitIr::Lexical { name, .. }),
            test: Some(_),
            update: Some(_),
            suspension_statement,
            entry_state: 0,
            resume_state: 1,
            exit_state: 1,
            ..
        }, StatementIr::AsyncAwait {
            suspend_state: 1,
            resume_state: 2,
            resume_mode: AsyncResumeModeIr::Return,
            ..
        }] = function.body.statements.as_slice()
        else {
            panic!(
                "expected a reusable loop Await followed by return Await: {:#?}",
                function.body.statements
            );
        };
        assert!(matches!(
            suspension_statement.as_ref(),
            StatementIr::AsyncAwait {
                suspend_state: 0,
                resume_state: 1,
                resume_mode: AsyncResumeModeIr::Ignore,
                ..
            }
        ));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == *name));
    }

    #[test]
    fn rejects_unplanned_resumable_async_loop_control() {
        for source in [
            "async function* stream() { for (;;) { await 0; break; } }",
            "async function* stream() { for (let i = 0; i < 2; i++) { await 0; continue; } }",
            "async function* stream() { for (let i = 0; i < 2; i++) { await 0; await 1; } }",
            "async function* stream() { for (let i = 0; i < 2; await 0) {} }",
        ] {
            let program = lower_script(source);
            assert!(!program.is_wasm_supported());
            assert!(
                program.diagnostics.iter().any(|diagnostic| diagnostic
                    .message
                    .contains("resumable async loop requires one direct body await")),
                "{source}: {:?}",
                program.diagnostics
            );
        }
    }

    #[test]
    fn stages_async_generator_array_spread_yields_in_source_order() {
        let program = lower_script("async function* stream() { yield [0, ...yield, 3]; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator should be registered");

        assert_eq!(
            function
                .resumable_plan
                .as_ref()
                .expect("async generator should have a resumable plan")
                .state_count,
            3
        );
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name.starts_with("$generator.array.spread.")));
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if statements.iter().filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. })).count() == 2
                    && statements.iter().any(|statement| matches!(
                        statement,
                        StatementIr::Lexical {
                            init: TypedExpr {
                                expr: ExprIr::CallIndirect { .. },
                                ..
                            },
                            ..
                        }
                    ))
        ));
    }

    #[test]
    fn stages_async_generator_object_spread_yields_in_source_order() {
        let program = lower_script(
            "async function* stream() { yield { ...yield, fixed: 1, ...yield yield }; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator should be registered");

        assert_eq!(
            function
                .resumable_plan
                .as_ref()
                .expect("async generator should have a resumable plan")
                .state_count,
            5
        );
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name.starts_with("$generator.object.spread.")));
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if statements.iter().filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. })).count() == 4
        ));
    }

    #[test]
    fn rejects_async_generator_spread_with_conditional_yield() {
        let program =
            lower_script("async function* stream(flag) { yield [...(flag ? yield [] : [])]; }");

        assert!(!program.is_wasm_supported());
        assert!(program.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("generator expression suspension")
        }));
    }

    #[test]
    fn first_async_generator_request_preserves_linear_body_start_states() {
        let program = lower_script(
            "let started = false;
             async function* stream(source) {
                 started = true;
                 await source;
                 yield source;
             }
             const iterator = stream(1);
             iterator.next();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");

        assert_eq!(
            function.resumable_plan,
            Some(ResumablePlanIr {
                entry_state: 0,
                state_count: 3,
                suspension_points: vec![
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Await,
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 1,
                        resume_state: 2,
                    },
                ],
            })
        );
        assert!(function.body.statements.iter().any(|statement| {
            matches!(statement, StatementIr::Expression(_))
                || matches!(
                    statement,
                    StatementIr::LexicalBlock(statements)
                        if statements
                            .iter()
                            .any(|statement| matches!(statement, StatementIr::Expression(_)))
                )
        }));
    }

    #[test]
    fn allocates_mixed_async_generator_suspension_states_without_collisions() {
        let program = lower_script(
            "async function* stream(source) {
                 await source.ready;
                 yield 1;
                 for await (const value of source) { yield value; }
                 await source.done;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");

        assert_eq!(
            function.resumable_plan,
            Some(ResumablePlanIr {
                entry_state: 0,
                state_count: 8,
                suspension_points: vec![
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Await,
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 1,
                        resume_state: 2,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::ForAwaitNext,
                        suspend_state: 2,
                        resume_state: 3,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 3,
                        resume_state: 4,
                    },
                    // State 5 is reserved so the iterator-close await suspends
                    // somewhere nothing else resumes into. Chaining it onto 4
                    // aliased `close_resume_state` with the body yield's resume
                    // state, and — for a body with no suspension at all — with
                    // `value_resume_state`, which made a `next()` resume replay
                    // the close path.
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::ForAwaitClose,
                        suspend_state: 5,
                        resume_state: 6,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Await,
                        suspend_state: 6,
                        resume_state: 7,
                    },
                ],
            })
        );
        let [StatementIr::AsyncAwait {
            suspend_state: 0,
            resume_state: 1,
            ..
        }, StatementIr::GeneratorYield {
            suspend_state: 1,
            resume_state: 2,
            ..
        }, StatementIr::ForOfIterator {
            body,
            async_plan: Some(async_plan),
            ..
        }, StatementIr::AsyncAwait {
            suspend_state: 6,
            resume_state: 7,
            ..
        }] = function.body.statements.as_slice()
        else {
            panic!(
                "async-generator body should preserve await, yield, for-await, await: {:?}",
                function.body.statements
            );
        };
        assert_eq!(async_plan.entry_state, 2);
        assert_eq!(async_plan.value_resume_state, 3);
        assert_eq!(async_plan.close_resume_state, 5);
        assert_eq!(async_plan.exit_state, 6);
        assert!(matches!(
            body.as_ref(),
            StatementIr::Block(block)
                if matches!(
                    block.statements.as_slice(),
                    [StatementIr::GeneratorYield {
                        suspend_state: 3,
                        resume_state: 4,
                        ..
                    }]
                )
        ));
    }

    #[test]
    fn allocates_disjoint_for_await_states_when_the_body_never_suspends() {
        // The four states a for-await loop re-enters on must be pairwise
        // distinct; the backend's entry test admits three of them and then
        // re-dispatches on which one it saw, so an alias silently routes a
        // `next()` resume into the iterator-close path.
        let program = lower_script(
            "async function* stream(source) {
                 for await (const value of source) { sink(value); }
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");

        let [StatementIr::ForOfIterator {
            async_plan: Some(async_plan),
            ..
        }] = function.body.statements.as_slice()
        else {
            panic!(
                "async-generator body should be a single for-await: {:?}",
                function.body.statements
            );
        };
        let states = [
            async_plan.entry_state,
            async_plan.value_resume_state,
            async_plan.close_resume_state,
            async_plan.exit_state,
        ];
        assert_eq!(states, [0, 1, 2, 3], "{async_plan:?}");
    }

    #[test]
    fn async_generator_return_without_value_completes_without_implicit_await() {
        let program = lower_script("async function* stream() { return; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");

        assert_eq!(
            function.resumable_plan,
            Some(ResumablePlanIr {
                entry_state: 0,
                state_count: 1,
                suspension_points: Vec::new(),
            })
        );
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::Return(TypedExpr {
                kind: ValueKind::Undefined,
                ..
            })]
        ));
    }

    #[test]
    fn async_generator_return_value_ends_with_implicit_await() {
        let program = lower_script("async function* stream(value) { return value; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");

        assert_eq!(
            function.resumable_plan,
            Some(ResumablePlanIr {
                entry_state: 0,
                state_count: 2,
                suspension_points: vec![ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Await,
                    suspend_state: 0,
                    resume_state: 1,
                }],
            })
        );
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::AsyncAwait {
                suspend_state: 0,
                resume_state: 1,
                resume_mode: AsyncResumeModeIr::Return,
                ..
            }]
        ));
    }

    #[test]
    fn async_generator_return_await_yield_orders_yield_before_both_awaits() {
        let program = lower_script("async function* stream(value) { return await (yield value); }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");
        let plan = function
            .resumable_plan
            .as_ref()
            .expect("async generator should have a resumable plan");

        assert_eq!(plan.state_count, 4);
        assert_eq!(
            plan.suspension_points
                .iter()
                .map(|suspension| suspension.kind)
                .collect::<Vec<_>>(),
            vec![
                ResumableSuspensionKindIr::Yield,
                ResumableSuspensionKindIr::Await,
                ResumableSuspensionKindIr::Await,
            ]
        );
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical { .. },
                        StatementIr::GeneratorYield {
                            suspend_state: 0,
                            resume_state: 1,
                            resume_mode: GeneratorResumeModeIr::AssignIdentifier(_),
                            ..
                        },
                        StatementIr::Lexical { .. },
                        StatementIr::AsyncAwait {
                            suspend_state: 1,
                            resume_state: 2,
                            resume_mode: AsyncResumeModeIr::AssignIdentifier(_),
                            ..
                        },
                        StatementIr::AsyncAwait {
                            suspend_state: 2,
                            resume_state: 3,
                            resume_mode: AsyncResumeModeIr::Return,
                            ..
                        }
                    ]
                )
        ));
    }

    #[test]
    fn async_generator_return_yield_await_orders_await_before_yield_and_return_await() {
        let program = lower_script("async function* stream(value) { return yield (await value); }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");
        let plan = function
            .resumable_plan
            .as_ref()
            .expect("async generator should have a resumable plan");

        assert_eq!(plan.state_count, 4);
        assert_eq!(
            plan.suspension_points
                .iter()
                .map(|suspension| suspension.kind)
                .collect::<Vec<_>>(),
            vec![
                ResumableSuspensionKindIr::Await,
                ResumableSuspensionKindIr::Yield,
                ResumableSuspensionKindIr::Await,
            ]
        );
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(statements)]
                if matches!(
                    statements.as_slice(),
                    [
                        StatementIr::Lexical { .. },
                        StatementIr::AsyncAwait {
                            suspend_state: 0,
                            resume_state: 1,
                            resume_mode: AsyncResumeModeIr::AssignIdentifier(_),
                            ..
                        },
                        StatementIr::Lexical { .. },
                        StatementIr::GeneratorYield {
                            suspend_state: 1,
                            resume_state: 2,
                            resume_mode: GeneratorResumeModeIr::AssignIdentifier(_),
                            ..
                        },
                        StatementIr::AsyncAwait {
                            suspend_state: 2,
                            resume_state: 3,
                            resume_mode: AsyncResumeModeIr::Return,
                            ..
                        }
                    ]
                )
        ));
    }

    /// A composite `return` operand used to be refused outright; it now stages
    /// through the ordinary async prefix, so the `await` inside it becomes its
    /// own suspension and the residual `+` is what the implicit return awaits.
    #[test]
    fn async_generator_return_stages_composite_suspension_boundaries() {
        let program =
            lower_script("async function* stream(left, right) { return (await left) + right; }");

        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "stream")
            .expect("async generator declaration should be collected");
        let plan = function
            .resumable_plan
            .as_ref()
            .expect("async generator should have a resumable plan");

        assert_eq!(
            plan.suspension_points
                .iter()
                .map(|suspension| suspension.kind)
                .collect::<Vec<_>>(),
            vec![
                ResumableSuspensionKindIr::Await,
                ResumableSuspensionKindIr::Await,
            ]
        );
        let [StatementIr::LexicalBlock(statements)] = function.body.statements.as_slice() else {
            panic!(
                "staged return should lower to one block: {:?}",
                function.body.statements
            );
        };
        let awaits = statements
            .iter()
            .filter_map(|statement| match statement {
                StatementIr::AsyncAwait {
                    suspend_state,
                    resume_state,
                    resume_mode,
                    ..
                } => Some((*suspend_state, *resume_state, resume_mode)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches!(
                awaits.as_slice(),
                [
                    (0, 1, AsyncResumeModeIr::AssignIdentifier(_)),
                    (1, 2, AsyncResumeModeIr::Return),
                ]
            ),
            "{awaits:?} from {statements:?}"
        );
    }

    #[test]
    fn records_named_async_function_expression() {
        let program = lower_script(
            "const resume = async function inner(value) {
                 await Promise.resolve();
                 return value;
             };",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "inner")
            .expect("async function expression should be registered");

        assert_eq!(function.execution_kind, FunctionExecutionKind::Async);
        assert!(!function.constructable);
        assert!(function.is_named_expression);
        assert!(function
            .body
            .statements
            .iter()
            .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
    }

    #[test]
    fn records_async_object_method_as_non_constructable_async_function() {
        let program = lower_script(
            "const holder = {
                 marker: 40,
                 async method(delta) {
                     await Promise.resolve();
                     return this.marker + delta;
                 }
             };",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "method")
            .expect("async object method should be registered");

        assert_eq!(function.execution_kind, FunctionExecutionKind::Async);
        assert!(!function.constructable);
        assert!(function.body.statements.iter().any(|statement| matches!(
            statement,
            StatementIr::AsyncAwait {
                suspend_state: 0,
                resume_state: 1,
                ..
            }
        )));

        let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
            panic!("expected object binding");
        };
        let ExprIr::ObjectLiteral(properties) = &init.expr else {
            panic!("expected object literal");
        };
        assert!(properties.iter().any(|property| matches!(
            property,
            ObjectPropertyIr::Method {
                key,
                function: TypedExpr {
                    expr: ExprIr::FunctionValue(function_id),
                    ..
                },
            } if key == "method" && function_id == &function.id
        )));
    }

    #[test]
    fn lowers_async_object_method_later_parameter_read_as_tdz_throw() {
        let program = lower_script("const holder = { async method(value = later, later) {} };");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "method")
            .expect("async object method should be registered");
        let default_init = function.params[0]
            .default_init
            .as_ref()
            .expect("first parameter should have a default initializer");

        assert!(matches!(
            default_init.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            }
        ));
    }

    #[test]
    fn records_instance_and_static_async_class_methods_with_resume_state() {
        let program = lower_script(
            "class Base {
                 method() { return this.marker; }
                 static staticMethod() { return this.marker; }
             }
             class Derived extends Base {
                 async method(delta = later, later) {
                     await Promise.resolve();
                     return super.method() + delta;
                 }
                 static async staticMethod(delta) {
                     await Promise.resolve();
                     return super.staticMethod() + delta;
                 }
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let async_methods = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .filter(|function| function.execution_kind == FunctionExecutionKind::Async)
            .collect::<Vec<_>>();

        assert_eq!(async_methods.len(), 2);
        assert!(async_methods.iter().all(|function| !function.constructable));
        assert!(async_methods.iter().all(|function| {
            function.body.statements.iter().any(|statement| {
                matches!(
                    statement,
                    StatementIr::AsyncAwait {
                        suspend_state: 0,
                        resume_state: 1,
                        ..
                    }
                )
            })
        }));
        let instance_method = async_methods
            .iter()
            .find(|function| !function.is_static_class_member)
            .expect("instance async method should be registered");
        assert!(matches!(
            instance_method.params[0]
                .default_init
                .as_ref()
                .map(|init| &init.expr),
            Some(ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            })
        ));
    }

    #[test]
    fn records_async_arrows_with_lexical_captures_and_parameter_tdz() {
        let program = lower_script(
            "function make(marker) {
                 const expression = async delta =>
                     this.value + arguments[0] + delta + (new.target === undefined ? 1 : 0);
                 const block = async (value = later, later) => {
                     await Promise.resolve();
                     return this.value + arguments[0] + value
                         + (new.target === undefined ? 1 : 0);
                 };
                 return [expression, block];
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let async_arrows = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .filter(|function| {
                function.flavor == FunctionFlavor::Arrow
                    && function.execution_kind == FunctionExecutionKind::Async
            })
            .collect::<Vec<_>>();

        assert_eq!(async_arrows.len(), 2);
        assert!(async_arrows.iter().all(|function| !function.constructable));
        assert!(async_arrows
            .iter()
            .all(|function| function.captures_lexical_this));
        assert!(async_arrows
            .iter()
            .all(|function| function.captures_lexical_arguments));
        assert!(async_arrows.iter().all(|function| {
            function
                .captured_bindings
                .iter()
                .any(|binding| binding.name == LEXICAL_NEW_TARGET_NAME)
        }));
        assert!(async_arrows.iter().any(|function| {
            function
                .body
                .statements
                .iter()
                .any(|statement| matches!(statement, StatementIr::Return(_)))
        }));
        let block = async_arrows
            .iter()
            .find(|function| {
                function
                    .body
                    .statements
                    .iter()
                    .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
            })
            .expect("block-bodied async arrow should suspend");
        assert!(matches!(
            block.params[0].default_init.as_ref().map(|init| &init.expr),
            Some(ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            })
        ));
    }

    #[test]
    fn lowers_expression_bodied_async_arrow_awaits_in_source_order() {
        let program = lower_script("const add = async () => await 1 + await 2;");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| {
                function.flavor == FunctionFlavor::Arrow
                    && function.execution_kind == FunctionExecutionKind::Async
            })
            .expect("async arrow should be lowered");
        let StatementIr::Block(block) = &function.body.statements[0] else {
            panic!("expression-bodied async arrow should contain a linear await block");
        };

        assert!(matches!(
            &block.statements[1],
            StatementIr::AsyncAwait {
                suspend_state: 0,
                resume_state: 1,
                resume_mode: AsyncResumeModeIr::AssignIdentifier(_),
                ..
            }
        ));
        assert!(matches!(
            &block.statements[3],
            StatementIr::AsyncAwait {
                suspend_state: 1,
                resume_state: 2,
                resume_mode: AsyncResumeModeIr::AssignIdentifier(_),
                ..
            }
        ));
        assert!(matches!(&block.statements[4], StatementIr::Return(_)));
    }

    #[test]
    fn records_async_try_catch_finally_resume_boundaries() {
        let program = lower_script(
            "const settle = async function() {
                 try { await Promise.reject(\"early\"); }
                 catch (error) { await Promise.resolve(error); }
                 finally { return await Promise.resolve(\"override\"); }
             };",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.execution_kind == FunctionExecutionKind::Async)
            .expect("async function expression should be registered");
        let StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            async_plan,
            ..
        } = &function.body.statements[0]
        else {
            panic!("expected async try/catch/finally statement");
        };

        assert_eq!(
            *async_plan,
            Some(AsyncTryPlanIr {
                entry_state: 0,
                try_exit_state: 2,
                catch_entry_state: Some(2),
                catch_exit_state: Some(4),
                finally_entry_state: Some(4),
                finally_exit_state: Some(6),
                exit_state: 6,
            })
        );
        assert!(matches!(
            try_block.statements[0],
            StatementIr::AsyncAwait {
                suspend_state: 0,
                resume_state: 1,
                resume_mode: AsyncResumeModeIr::Ignore,
                ..
            }
        ));
        assert!(matches!(
            catch_block.statements[0],
            StatementIr::AsyncAwait {
                suspend_state: 2,
                resume_state: 3,
                resume_mode: AsyncResumeModeIr::Ignore,
                ..
            }
        ));
        assert!(matches!(
            finally_block.statements[0],
            StatementIr::AsyncAwait {
                suspend_state: 4,
                resume_state: 5,
                resume_mode: AsyncResumeModeIr::Return,
                ..
            }
        ));
    }

    #[test]
    fn async_generator_try_catch_shares_preplanned_clause_boundaries() {
        let program = lower_script(
            "async function* outer(source) {
                 let caught;
                 try { yield* source; }
                 catch (error) { caught = error; }
                 return caught;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "outer")
            .expect("async generator should be registered");
        assert_eq!(
            function.resumable_plan,
            Some(ResumablePlanIr {
                entry_state: 0,
                state_count: 3,
                suspension_points: vec![
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Yield,
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    ResumableSuspensionPointIr {
                        kind: ResumableSuspensionKindIr::Await,
                        suspend_state: 1,
                        resume_state: 2,
                    },
                ],
            })
        );
        let [StatementIr::Lexical { .. }, StatementIr::TryCatch {
            generator_plan: Some(generator_plan),
            async_plan: Some(async_plan),
            ..
        }, StatementIr::AsyncAwait {
            suspend_state: 1,
            resume_state: 2,
            resume_mode: AsyncResumeModeIr::Return,
            ..
        }] = function.body.statements.as_slice()
        else {
            panic!(
                "expected planned async-generator try/catch followed by terminal Await: {:#?}",
                function.body.statements
            );
        };
        let expected_try_plan = AsyncTryPlanIr {
            entry_state: 0,
            try_exit_state: 1,
            catch_entry_state: Some(1),
            catch_exit_state: Some(1),
            finally_entry_state: None,
            finally_exit_state: None,
            exit_state: 1,
        };
        assert_eq!(*async_plan, expected_try_plan);
        assert_eq!(
            *generator_plan,
            GeneratorTryPlanIr {
                entry_state: expected_try_plan.entry_state,
                try_exit_state: expected_try_plan.try_exit_state,
                catch_entry_state: expected_try_plan.catch_entry_state,
                catch_exit_state: expected_try_plan.catch_exit_state,
                finally_entry_state: expected_try_plan.finally_entry_state,
                finally_exit_state: expected_try_plan.finally_exit_state,
                exit_state: expected_try_plan.exit_state,
            }
        );
    }

    #[test]
    fn records_for_await_array_iterator_resume_boundaries_and_owned_state() {
        let program = lower_script(
            "async function collect() {
                 let total = 0;
                 for await (const value of [Promise.resolve(1), 2]) {
                     total += value;
                 }
                 return total;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| {
                function.name == "collect"
                    && function.execution_kind == FunctionExecutionKind::Async
            })
            .expect("async function should be registered");
        let StatementIr::ForOfIterator {
            async_plan: Some(plan),
            ..
        } = &function.body.statements[1]
        else {
            panic!("expected planned iterator-backed for-await-of statement");
        };

        assert_eq!(plan.entry_state, 0);
        assert_eq!(plan.value_resume_state, 1);
        assert_eq!(plan.close_resume_state, 2);
        assert_eq!(plan.exit_state, 3);
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.record.iterator().as_str()));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.record.next_method().as_str()));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.record.done().as_str()));
    }

    #[test]
    fn records_for_await_sync_iterator_resume_boundaries_and_owned_state() {
        let program = lower_script(
            "async function collect(iterable) {
                 for await (const value of iterable) return value;
             }
             let iterable = {};
             iterable[Symbol.iterator] = function () { return this; };
             collect(iterable);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| {
                function.name == "collect"
                    && function.execution_kind == FunctionExecutionKind::Async
            })
            .expect("async function should be registered");
        let StatementIr::ForOfIterator {
            async_plan: Some(plan),
            ..
        } = &function.body.statements[0]
        else {
            panic!("expected planned sync-iterator for-await-of statement");
        };

        assert_eq!(plan.entry_state, 0);
        assert_eq!(plan.value_resume_state, 1);
        assert_eq!(plan.close_resume_state, 2);
        assert_eq!(plan.exit_state, 3);
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.record.iterator().as_str()));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.record.next_method().as_str()));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.async_iterator_binding));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.record.done().as_str()));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.close_on_rejection_binding));
    }

    #[test]
    fn records_for_await_async_iterator_mode_as_owned_state() {
        let program = lower_script(
            "async function collect(iterable) {
                 for await (const value of iterable) return value;
             }
             let iterable = {};
             iterable[Symbol.asyncIterator] = function () { return this; };
             collect(iterable);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| {
                function.name == "collect"
                    && function.execution_kind == FunctionExecutionKind::Async
            })
            .expect("async function should be registered");
        let StatementIr::ForOfIterator {
            async_plan: Some(plan),
            ..
        } = &function.body.statements[0]
        else {
            panic!("expected planned async-iterator for-await-of statement");
        };

        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == plan.async_iterator_binding));
    }

    #[test]
    fn for_await_iterator_method_specializes_strict_primitive_this() {
        let program = lower_script(
            "String.prototype[Symbol.asyncIterator] = function strictAsyncIterator() {
                 'use strict';
                 return this;
             };
             String.prototype['Symbol.asyncIterator'] = function ordinaryStringProperty() {
                 return this;
             };
             async function collect() {
                 for await (const value of 'source') return value;
             }
             collect();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "strictAsyncIterator")
            .expect("strict async iterator method should be registered");
        let return_value = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Return(return_value) => Some(return_value),
                _ => None,
            })
            .expect("strict async iterator method should return this");

        assert!(matches!(return_value.expr, ExprIr::This));
        assert_eq!(return_value.kind, ValueKind::String);
    }

    #[test]
    fn records_generator_try_catch_finally_resume_state_boundaries() {
        let program = lower_script(
            "function* sequence() {
                 try { yield 1; }
                 catch (error) { yield error; }
                 finally { yield 3; }
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "sequence")
            .expect("generator should be registered");

        assert_eq!(
            function.generator_plan,
            Some(GeneratorPlanIr {
                entry_state: 0,
                state_count: 7,
                suspension_points: vec![
                    GeneratorSuspensionPointIr {
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    GeneratorSuspensionPointIr {
                        suspend_state: 2,
                        resume_state: 3,
                    },
                    GeneratorSuspensionPointIr {
                        suspend_state: 4,
                        resume_state: 5,
                    },
                ],
            })
        );
        let StatementIr::TryCatchFinally { generator_plan, .. } = &function.body.statements[0]
        else {
            panic!("expected generator try/catch/finally statement");
        };
        assert_eq!(
            *generator_plan,
            Some(GeneratorTryPlanIr {
                entry_state: 0,
                try_exit_state: 2,
                catch_entry_state: Some(2),
                catch_exit_state: Some(4),
                finally_entry_state: Some(4),
                finally_exit_state: Some(6),
                exit_state: 6,
            })
        );
    }

    #[test]
    fn records_nested_generator_try_finally_resume_state_boundaries() {
        let program = lower_script(
            "function* nested() {
                 try {
                     try { yield 1; }
                     finally { yield 2; }
                 } finally { yield 3; }
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "nested")
            .expect("generator should be registered");

        assert_eq!(
            function.generator_plan,
            Some(GeneratorPlanIr {
                entry_state: 0,
                state_count: 8,
                suspension_points: vec![
                    GeneratorSuspensionPointIr {
                        suspend_state: 0,
                        resume_state: 1,
                    },
                    GeneratorSuspensionPointIr {
                        suspend_state: 2,
                        resume_state: 3,
                    },
                    GeneratorSuspensionPointIr {
                        suspend_state: 5,
                        resume_state: 6,
                    },
                ],
            })
        );
        let StatementIr::TryFinally {
            try_block,
            generator_plan,
            ..
        } = &function.body.statements[0]
        else {
            panic!("expected outer generator try/finally statement");
        };
        assert_eq!(
            *generator_plan,
            Some(GeneratorTryPlanIr {
                entry_state: 0,
                try_exit_state: 5,
                catch_entry_state: None,
                catch_exit_state: None,
                finally_entry_state: Some(5),
                finally_exit_state: Some(7),
                exit_state: 7,
            })
        );
        let StatementIr::TryFinally { generator_plan, .. } = &try_block.statements[0] else {
            panic!("expected nested generator try/finally statement");
        };
        assert_eq!(
            *generator_plan,
            Some(GeneratorTryPlanIr {
                entry_state: 0,
                try_exit_state: 2,
                catch_entry_state: None,
                catch_exit_state: None,
                finally_entry_state: Some(2),
                finally_exit_state: Some(4),
                exit_state: 4,
            })
        );
    }

    #[test]
    fn generator_activation_owns_bindings_that_survive_a_yield() {
        let program = lower_script(
            "function* activation(parameter) { let local = parameter; if (local) { yield local; local += arguments[0]; } }",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "activation")
            .expect("generator should be registered");
        let owned_names = function
            .owned_env_bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<BTreeSet<_>>();
        assert!(owned_names.contains("parameter"));
        assert!(owned_names.contains("local"));
        assert!(matches!(
            function.body.statements[1],
            StatementIr::GeneratorIf { .. }
        ));
    }

    #[test]
    fn stages_generator_return_calls_across_argument_yields() {
        let program = lower_script(
            "const generator = function* g() { return (function(value) { return value + 1; }(yield)); };",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "g")
            .expect("generator should be registered");
        assert_eq!(function.generator_plan.as_ref().unwrap().state_count, 2);
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(_)]
        ));
    }

    #[test]
    fn stages_generator_object_spreads_in_source_order() {
        let program = lower_script(
            "const generator = function* g() { yield { ...yield yield, ...(function(value) { return {...value}; }(yield)), ...yield }; };",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "g")
            .expect("generator should be registered");
        assert_eq!(function.generator_plan.as_ref().unwrap().state_count, 6);
        assert!(matches!(
            function.body.statements.as_slice(),
            [StatementIr::LexicalBlock(_)]
        ));
    }

    #[test]
    fn rejects_generator_suspensions_without_a_structured_resume_plan() {
        for source in [
            "function* nestedOperand() { return 1 + (yield 2); }",
            "function* scopedBranch(flag) { if (flag) { let value = 1; yield value; } }",
        ] {
            let program = lower_script(source);
            assert!(
                !program.is_wasm_supported(),
                "source should be rejected: {source}"
            );
        }
    }

    #[test]
    fn lowers_a_loop_body_lexical_declaration_across_a_suspension() {
        // This shape used to be rejected alongside the cases above. A loop body
        // that redeclares a lexical binding on every iteration and suspends
        // while it is live now gets a structured resume plan, so it lowers
        // instead of being refused. Verified end to end: the generator yields
        // 0, 2, 4 and then completes.
        let program = lower_script(
            "function* g() { let i = 0; while (i < 3) { let d = i * 2; yield d; i += 1; } }",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "g")
            .expect("generator should be registered");
        assert!(function.generator_plan.is_some());
    }

    #[test]
    fn allows_subclassing_iterator_constructor() {
        let program = lower_script("class SubIterator extends Iterator {}");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
            panic!("expected class lexical declaration");
        };
        assert!(matches!(init.kind, ValueKind::Function));
    }

    #[test]
    fn typed_array_constructor_prototype_is_inferred_as_a_function() {
        let program = lower_script("var TypedArray = Object.getPrototypeOf(Int8Array);");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Var(declarators) = &script.body.statements[0] else {
            panic!("expected variable declaration");
        };
        let init = declarators[0]
            .init
            .as_ref()
            .expect("TypedArray should have an initializer");
        assert_eq!(init.kind, ValueKind::Function);
        assert_eq!(init.possible_kinds, KindSet::from_kind(ValueKind::Function));
    }

    #[test]
    fn lowers_heap_shapes_and_array_length() {
        let program = lower_script(
            "function box() { let o = { inner: { x: 2 } }; return o; } let a = [1, 2, 3]; box().inner.x + a.length;",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions[0].return_kind, ValueKind::Object);
        assert!(script.functions[0].return_shape.is_some());
        let summary = program.ir_summary();
        assert!(summary.contains("array_lengths=1"));
        assert!(summary.contains("heap_shapes="));
    }

    #[test]
    fn lowers_property_access_on_dynamic_after_kind_merge() {
        let program = lower_script("let v; if (true) { v = 1; } else { v = { x: 1 }; } v.x;");
        assert!(program.is_wasm_supported());
        assert_eq!(
            program
                .script
                .as_ref()
                .expect("script ir should exist")
                .result_kind(),
            ValueKind::Dynamic
        );
    }

    #[test]
    fn lowers_nested_function_declaration() {
        let program =
            lower_script("function outer() { function inner() { return 1; } return inner(); }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 3);
        assert!(script.functions.iter().any(|function| function.is_nested));
        let summary = program.ir_summary();
        assert!(summary.contains("nested_functions=2"));
    }

    #[test]
    fn lowers_closure_capture_and_function_expression() {
        let program = lower_script(
            "function outer() { let x = 2; return function (y) { return x + y; }; } let f = outer(); f(3);",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 3);
        assert!(script
            .functions
            .iter()
            .any(|function| function.is_expression));
        assert!(script
            .functions
            .iter()
            .any(|function| !function.captured_bindings.is_empty()));
        let summary = program.ir_summary();
        assert!(summary.contains("function_exprs=1"));
        assert!(summary.contains("closures=3"));
        assert!(summary.contains("captures=1"));
    }

    #[test]
    fn lowers_script_closure_capture() {
        let program = lower_script("let x = 1; function f() { return x; } f();");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.owned_env_bindings.len(), 1);
        assert_eq!(script.owned_env_bindings[0].name, "x");
        assert_eq!(script.functions[0].captured_bindings[0].name, "x");
        assert_eq!(
            script.functions[0].captured_bindings[0].slot,
            script.owned_env_bindings[0].slot
        );
        assert_eq!(script.functions[0].captured_bindings[0].hops, 0);
    }

    #[test]
    fn with_body_captures_outer_binding_for_property_fallback() {
        let program = lower_script(
            "const fallback = { value: 17 }; function observe(view) { with (view) { return fallback.value; } }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let observe = script
            .functions
            .iter()
            .find(|function| function.name == "observe")
            .expect("observe function should be lowered");

        assert!(observe
            .captured_bindings
            .iter()
            .any(|binding| binding.name == "fallback"));
    }

    #[test]
    fn supported_block_patterns_share_span_stable_capture_storage() {
        for declaration in ["let { value } = { value: 2 };", "let [value] = [2];"] {
            let source = format!(
                "function owner() {{ let value = 1; {{ {declaration} return (() => value)(); }} }} owner();"
            );
            assert_function_capture_storage_contract(&source, "owner", None, "$scoped.lex.");
        }
    }

    #[test]
    fn object_catch_pattern_shares_span_stable_capture_storage() {
        assert_function_capture_storage_contract(
            "function owner() { let value = 1; try { throw { value: 2 }; } catch ({ value }) { return (() => value)(); } } owner();",
            "owner",
            None,
            "$scoped.lex.",
        );
    }

    #[test]
    fn script_supported_patterns_keep_source_named_capture_storage() {
        for source in [
            "let { value } = { value: 2 }; (() => value)();",
            "let [value] = [2]; (() => value)();",
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let capture = script
                .functions
                .iter()
                .flat_map(|function| &function.captured_bindings)
                .find(|binding| binding.name == "value")
                .expect("arrow should capture the root lexical binding");
            assert!(script
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == capture.name && binding.slot == capture.slot));
            assert!(collect_binding_storage_names(&script.body).contains(&capture.name));
        }
    }

    #[test]
    fn pattern_initializer_closures_capture_eventual_lexical_storage() {
        for source in [
            "let { value } = { value: (() => value)() };",
            "let [value] = [(() => value)()];",
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let capture = script
                .functions
                .iter()
                .flat_map(|function| &function.captured_bindings)
                .find(|binding| binding.name == "value")
                .expect("initializer arrow should capture the eventual lexical binding");
            assert!(script
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == capture.name && binding.slot == capture.slot));
        }
    }

    #[test]
    fn object_var_pattern_keeps_owner_capture_storage() {
        assert_function_capture_storage_contract(
            "function owner() { var { value } = { value: 2 }; return () => value; } owner()();",
            "owner",
            None,
            "value",
        );
    }

    #[test]
    fn object_for_of_pattern_shares_dedicated_loop_capture_storage() {
        assert_function_capture_storage_contract(
            "function owner() { let value = 1; let read; for (let { value } of [{ value: 2 }]) { read = () => value; break; } return read(); } owner();",
            "owner",
            None,
            "$forof.lex.",
        );
    }

    #[test]
    fn classic_for_head_shares_span_stable_capture_storage() {
        assert_function_capture_storage_contract(
            "function owner() { let value = 1; let read; for (let value = 2; value < 3; value++) { read = () => value; break; } return read(); } owner();",
            "owner",
            None,
            "$scoped.lex.",
        );
    }

    #[test]
    fn classic_for_update_targets_lexical_storage() {
        let program = lower_script("for (let value = 0; value < 2; value++) {}");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::For {
            init: Some(ForInitIr::Lexical { name, .. }),
            update: Some(update),
            ..
        } = &script.body.statements[0]
        else {
            panic!("expected classic for lexical initializer and update");
        };
        assert!(name.starts_with("$scoped.lex."));
        assert!(matches!(
            &update.expr,
            ExprIr::UpdateIdentifier {
                name: update_name,
                ..
            } if update_name == name
        ));
    }

    #[test]
    fn classic_for_initializers_capture_eventual_physical_bindings() {
        for source in [
            "function owner() { let value = 1; let read; for (let value = 2, unused = read = () => value; false;) {} return read(); } owner();",
            "function owner() { let read; for (let inner = read = () => later, later = 2; false;) {} return read(); } owner();",
        ] {
            assert_function_capture_storage_contract(source, "owner", None, "$scoped.lex.");
        }
    }

    #[test]
    fn classic_for_direct_self_initializer_read_uses_tdz() {
        let program = lower_script(
            "function owner() { let value = 1; for (let value = value;;) { break; } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let init = owner
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::For {
                    init: Some(ForInitIr::Lexical { init, .. }),
                    ..
                } => Some(init),
                _ => None,
            })
            .expect("classic for lexical initializer should be lowered");
        assert!(matches!(
            &init.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            }
        ));
    }

    #[test]
    fn nested_root_declarations_preserve_transitive_creation_aliases() {
        assert_function_capture_storage_contract(
            "function owner() { let value = 0; { let value = 2; function middle() { function inner() { return value; } return inner(); } return middle(); } } owner();",
            "owner",
            Some("inner"),
            "$scoped.lex.",
        );
    }

    #[test]
    fn lowers_for_in_let_closure_capture_metadata() {
        let program = lower_script(
            "function fn(x) { let callbacks = []; for (let p in x) { callbacks.push(function () { return p; }); } return callbacks[0](); } fn({ a: 1 });",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let outer = script
            .functions
            .iter()
            .find(|function| function.name == "fn")
            .expect("outer function should be lowered");
        let loop_binding = outer
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::ForInObject {
                    lexical_environment: Some(environment),
                    ..
                } => environment
                    .iteration_environment
                    .as_ref()
                    .and_then(|environment| {
                        environment.bindings.iter().find(|binding| {
                            binding.name.starts_with("$forin.lex.") && binding.name.ends_with(".p")
                        })
                    }),
                _ => None,
            })
            .expect("loop binding should own an iteration environment slot");
        let closure = script
            .functions
            .iter()
            .find(|function| function.is_expression)
            .expect("loop closure should be lowered");
        assert_eq!(closure.captured_bindings.len(), 1);
        assert_eq!(closure.captured_bindings[0].name, loop_binding.name);
        assert_eq!(closure.captured_bindings[0].slot, loop_binding.slot);
    }

    #[test]
    fn lowers_shadowed_for_in_let_closure_to_loop_binding() {
        let program = lower_script(
            "let x = 'outside'; var f; for (let x in { a: 0 }) { f = function () { return x; }; } f();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let closure = script
            .functions
            .iter()
            .find(|function| function.is_expression)
            .expect("loop closure should be lowered");
        assert_eq!(closure.captured_bindings.len(), 1);
        let captured = &closure.captured_bindings[0];
        assert!(captured.name.starts_with("$forin.lex."));
        assert!(captured.name.ends_with(".x"));
        assert_ne!(captured.name, "x");
        let StatementIr::ForInObject {
            lexical_environment: Some(environment),
            ..
        } = &script.body.statements[2]
        else {
            panic!("expected captured for-in iteration environment");
        };
        assert!(environment
            .iteration_environment
            .as_ref()
            .is_some_and(|environment| environment
                .bindings
                .iter()
                .any(|binding| binding.name == captured.name && binding.slot == captured.slot)));
    }

    #[test]
    fn lowers_for_in_head_lexical_tdz_for_target_expression() {
        fn has_reference_error_throw(expr: &TypedExpr) -> bool {
            match &expr.expr {
                ExprIr::RuntimeThrow {
                    name: NativeErrorKind::ReferenceError,
                    ..
                } => true,
                ExprIr::ObjectLiteral(properties) => {
                    properties.iter().any(|property| match property {
                        ObjectPropertyIr::PrototypeSetter { value }
                        | ObjectPropertyIr::Spread { source: value }
                        | ObjectPropertyIr::Data { value, .. }
                        | ObjectPropertyIr::NonEnumerableData { value, .. } => {
                            has_reference_error_throw(value)
                        }
                        ObjectPropertyIr::ComputedData { key, value } => {
                            has_reference_error_throw(key) || has_reference_error_throw(value)
                        }
                        ObjectPropertyIr::ComputedMethod { key, function }
                        | ObjectPropertyIr::ComputedGetter { key, function }
                        | ObjectPropertyIr::ComputedSetter { key, function } => {
                            has_reference_error_throw(key) || has_reference_error_throw(function)
                        }
                        ObjectPropertyIr::Method { function, .. }
                        | ObjectPropertyIr::Getter { function, .. }
                        | ObjectPropertyIr::Setter { function, .. } => {
                            has_reference_error_throw(function)
                        }
                    })
                }
                ExprIr::TypeOf { expr } => has_reference_error_throw(expr),
                _ => false,
            }
        }

        let program = lower_script("let x = 1; for (let x in { x }) {}");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::ForInObject { target, .. } = &script.body.statements[1] else {
            panic!("expected for-in object statement");
        };
        assert!(has_reference_error_throw(target));
    }

    #[test]
    fn lowers_for_of_object_pattern_head_under_lexical_tdz() {
        let program = lower_script("let x = 1; for (let { x } of [{ x }]) {}");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::ForOfArray { iterable, .. } = &script.body.statements[1] else {
            panic!("expected for-of array statement");
        };
        let ExprIr::ArrayLiteral(elements) = &iterable.expr else {
            panic!("expected array iterable");
        };
        let ExprIr::ObjectLiteral(properties) = &elements[0].expr else {
            panic!("expected object element");
        };
        let ObjectPropertyIr::Data { value, .. } = &properties[0] else {
            panic!("expected shorthand data property");
        };
        assert!(matches!(
            value.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            }
        ));
    }

    #[test]
    fn lowers_for_in_head_function_capture_to_tdz_binding() {
        let program = lower_script(
            "let x = 'outside'; var f; for (let x in { i: f = function () { return typeof x; } }) {} f();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let closure = script
            .functions
            .iter()
            .find(|function| function.is_expression)
            .expect("head closure should be lowered");
        assert_eq!(closure.captured_bindings.len(), 1);
        assert!(closure.captured_bindings[0]
            .name
            .starts_with(TDZ_BINDING_STORAGE_PREFIX));
        assert!(closure.captured_bindings[0].name.ends_with(".x"));
    }

    #[test]
    fn infers_array_buffer_new_shape_for_closure_capture() {
        let program = lower_script(
            "const rab = new ArrayBuffer(64, { maxByteLength: 1024 }); const f = () => rab.resize; f();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let arrow = script
            .functions
            .iter()
            .find(|function| function.flavor == FunctionFlavor::Arrow)
            .expect("arrow function should be lowered");
        let Some(StatementIr::Return(expr)) = arrow.body.statements.first() else {
            panic!("expression arrow should lower to return");
        };
        assert!(expr
            .function_targets
            .contains(&StandardBuiltinId::ArrayBufferPrototypeResize.function_id()));
    }

    #[test]
    fn infers_inherited_typed_array_to_string_target() {
        let program = lower_script("Uint8Array.prototype.toString;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = script.body.statements.last().unwrap() else {
            panic!("property read should remain the script result");
        };
        assert!(
            expr.function_targets
                .contains(&StandardBuiltinId::TypedArrayPrototypeToString.function_id()),
            "unexpected property expression: {expr:?}"
        );
    }

    #[test]
    fn lowers_script_class_closure_capture() {
        let program = lower_script("class C {} function f() { return C; } f();");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == "C"));
        assert!(script.functions.iter().any(|function| {
            function
                .captured_bindings
                .iter()
                .any(|binding| binding.name == "C")
        }));
    }

    #[test]
    fn lowers_class_constructor_closure_capture() {
        let program = lower_script(
            "function outer(Base, args) { let called = 0; class Derived extends Base { constructor() { ++called; super(...args); } } return Derived; }",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let constructor = script
            .functions
            .iter()
            .find(|function| function.name == "Derived")
            .expect("derived constructor should be lowered");
        assert!(constructor
            .captured_bindings
            .iter()
            .any(|binding| binding.name == "called"));
        assert!(constructor
            .captured_bindings
            .iter()
            .any(|binding| binding.name == "args"));
    }

    #[test]
    fn class_members_preserve_scoped_capture_source_names() {
        for (source, member_name) in [
            (
                "function owner() { let x = \"outer\"; { let x = 2; class C { m() { return x + 3; } } return new C().m(); } } owner();",
                "C.m",
            ),
            (
                "function owner() { let x = \"outer\"; { let x = 2; class C { constructor() { this.value = x + 3; } } return new C().value; } } owner();",
                "C",
            ),
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let owner = script
                .functions
                .iter()
                .find(|function| function.name == "owner")
                .expect("owner function should be lowered");
            let member = script
                .functions
                .iter()
                .find(|function| function.name == member_name)
                .expect("class member should be lowered");
            let capture = member
                .captured_bindings
                .iter()
                .find(|binding| binding.source_name == "x")
                .expect("class member should capture the scoped binding");

            assert!(capture.name.starts_with("$scoped.lex."));
            assert_eq!(capture.hops, 1);
            assert!(
                !owner
                    .owned_env_bindings
                    .iter()
                    .any(|binding| binding.name == capture.name)
            );
            assert!(block_environment_owns_binding(
                &owner.body,
                &capture.name,
                capture.slot
            ));
            if member_name == "C.m" {
                assert_eq!(member.return_kind, ValueKind::Dynamic);
            } else {
                assert!(member.body.statements.iter().any(|statement| {
                    matches!(
                        statement,
                        StatementIr::Expression(TypedExpr {
                            expr: ExprIr::PropertyWrite { value, .. },
                            ..
                        }) if value.kind == ValueKind::Dynamic
                    )
                }));
            }
        }
    }

    #[test]
    fn class_members_retain_transitive_root_function_captures() {
        for (source, member_name) in [
            (
                "function owner() { let x = \"outer\"; { let x = 2; class C { m() { function inner() { return x + 3; } return inner(); } } return new C().m(); } } owner();",
                "C.m",
            ),
            (
                "function owner() { let x = \"outer\"; { let x = 2; class C { constructor() { function inner() { return x + 3; } this.value = inner(); } } return new C().value; } } owner();",
                "C",
            ),
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let member = script
                .functions
                .iter()
                .find(|function| function.name == member_name)
                .expect("class member should be lowered");
            let member_capture = member
                .captured_bindings
                .iter()
                .find(|binding| binding.source_name == "x")
                .expect("class member should retain the nested root capture");
            let root_function = script
                .functions
                .iter()
                .find(|function| function.name == "inner")
                .expect("nested root function should be lowered");
            let root_capture = root_function
                .captured_bindings
                .iter()
                .find(|binding| binding.source_name == "x")
                .expect("nested root function should capture the scoped binding");

            assert!(member_capture.name.starts_with("$scoped.lex."));
            assert_eq!(member_capture.name, root_capture.name);
            assert_eq!(member_capture.slot, root_capture.slot);
            assert_eq!(member_capture.hops, 1);
            assert_eq!(root_capture.hops, 1);
            assert_eq!(root_function.return_kind, ValueKind::Dynamic);
        }
    }

    #[test]
    fn instance_field_initializer_retains_transitive_block_capture_from_class_definition() {
        let program = lower_script(
            "function owner() { { const blockValue = 7; class C { value = (() => blockValue)(); } return C; } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let field = script
            .functions
            .iter()
            .find(|function| function.name == "C.field.value")
            .expect("field initializer should be lowered");
        let constructor = script
            .functions
            .iter()
            .find(|function| function.name == "C")
            .expect("default constructor should be lowered");
        let arrow = script
            .functions
            .iter()
            .find(|function| function.flavor == FunctionFlavor::Arrow)
            .expect("nested arrow should be lowered");
        let field_capture = field
            .captured_bindings
            .iter()
            .find(|binding| binding.source_name == "blockValue")
            .expect("field initializer should retain the nested arrow capture");
        let arrow_capture = arrow
            .captured_bindings
            .iter()
            .find(|binding| binding.source_name == "blockValue")
            .expect("nested arrow should capture the block binding");

        assert_eq!(field_capture.mode, BindingMode::Const);
        assert_eq!(field_capture.name, arrow_capture.name);
        assert_eq!(field_capture.slot, arrow_capture.slot);
        assert!(!constructor
            .captured_bindings
            .iter()
            .any(|binding| binding.source_name == "blockValue"));
    }

    #[test]
    fn instance_field_capture_uses_the_class_definition_environment() {
        let program = lower_script(
            "function owner() { let x = 7; class C { constructor() { let local = 1; (() => local)(); } value = x; } return C; } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let field = script
            .functions
            .iter()
            .find(|function| function.name == "C.field.value")
            .expect("field initializer should be lowered");
        let constructor = script
            .functions
            .iter()
            .find(|function| function.name == "C")
            .expect("explicit constructor should be lowered");
        let field_capture = field
            .captured_bindings
            .iter()
            .find(|binding| binding.source_name == "x")
            .expect("field initializer should capture the outer binding");
        assert!(constructor
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == "local"));
        assert_eq!(field_capture.hops, 1);
        assert!(!constructor
            .captured_bindings
            .iter()
            .any(|binding| binding.source_name == "x"));
    }

    #[test]
    fn static_field_initializer_captures_switch_environment_binding() {
        let program = lower_script(
            "switch (0) { case 0: let switchValue = 3; class C { static value = switchValue; } }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let field = script
            .functions
            .iter()
            .find(|function| function.name == "C.field.value")
            .expect("static field initializer should be lowered");
        let capture = field
            .captured_bindings
            .iter()
            .find(|binding| binding.source_name == "switchValue")
            .expect("static field should capture the switch binding");

        assert_eq!(capture.mode, BindingMode::Let);
        assert_ne!(capture.name, capture.source_name);
    }

    #[test]
    fn static_block_retains_nested_function_capture_from_catch_environment() {
        let program = lower_script(
            "try { throw 1; } catch (caught) { class C { static { function readCaught() { return caught; } this.value = readCaught(); } } }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let static_block = script
            .functions
            .iter()
            .find(|function| function.name == "C.<static>")
            .expect("static block should be lowered");
        let nested = script
            .functions
            .iter()
            .find(|function| function.name == "readCaught")
            .expect("nested function should be lowered");
        let static_capture = static_block
            .captured_bindings
            .iter()
            .find(|binding| binding.source_name == "caught")
            .expect("static block should retain the nested function capture");
        let nested_capture = nested
            .captured_bindings
            .iter()
            .find(|binding| binding.source_name == "caught")
            .expect("nested function should capture the catch binding");

        assert_eq!(static_capture.mode, BindingMode::Let);
        assert_eq!(static_capture.name, nested_capture.name);
        assert_eq!(static_capture.slot, nested_capture.slot);
    }

    #[test]
    fn class_execution_ids_are_distinct_for_multiple_fields_and_static_blocks() {
        let program = lower_script("class C { first = 1; second = 2; static {} static {} }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let class = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical {
                    init:
                        TypedExpr {
                            expr: ExprIr::ClassDefinition(class),
                            ..
                        },
                    ..
                } => Some(class.as_ref()),
                _ => None,
            })
            .expect("class definition should be lowered");
        let constructor = script
            .functions
            .iter()
            .find(|function| function.id == class.constructor_function_id)
            .expect("class constructor should be lowered");
        let instance_plan = constructor
            .class_instance_element_plan
            .as_ref()
            .expect("constructor should own its instance element plan");
        let execution_ids =
            instance_plan
                .fields
                .iter()
                .filter_map(|field| field.init_function_id.clone())
                .chain(class.element_plan.static_elements.iter().filter_map(
                    |element| match element {
                        ClassStaticElementIr::Field(field) => field.init_function_id.clone(),
                        ClassStaticElementIr::Block(block) => Some(block.function_id.clone()),
                    },
                ))
                .collect::<BTreeSet<_>>();

        assert_eq!(instance_plan.fields.len(), 2);
        assert_eq!(class.element_plan.static_elements.len(), 2);
        assert_eq!(execution_ids.len(), 4);
    }

    #[test]
    fn class_element_plan_preserves_definition_and_field_source_order() {
        let program = lower_script(
            "function first() { return 'first'; }
             function second() { return 'second'; }
             function third() { return 'third'; }
             class C {
                 [first()]() {}
                 #method() {}
                 static publicField = 1;
                 static {}
                 static #privateField = 2;
                 get [second()]() {}
                 set [third()](value) {}
                 publicInstance = 3;
                 #privateInstance = 4;
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let class = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical {
                    init:
                        TypedExpr {
                            expr: ExprIr::ClassDefinition(class),
                            ..
                        },
                    ..
                } => Some(class.as_ref()),
                _ => None,
            })
            .expect("class definition should be lowered");

        assert!(matches!(
            class.element_plan.definitions.as_slice(),
            [
                ClassElementDefinitionIr::PublicMethod(first),
                ClassElementDefinitionIr::PrivateMethod(_),
                ClassElementDefinitionIr::PublicMethod(second),
                ClassElementDefinitionIr::PublicMethod(third),
            ] if first.kind == ClassFunctionKind::Method
                && matches!(&first.key, PropertyKeyIr::StringExpr(_))
                && second.kind == ClassFunctionKind::Getter
                && matches!(&second.key, PropertyKeyIr::StringExpr(_))
                && third.kind == ClassFunctionKind::Setter
                && matches!(&third.key, PropertyKeyIr::StringExpr(_))
        ));
        assert!(matches!(
            class.element_plan.static_elements.as_slice(),
            [
                ClassStaticElementIr::Field(public),
                ClassStaticElementIr::Block(_),
                ClassStaticElementIr::Field(private),
            ] if matches!(&public.key, ClassFieldKeyIr::Public(key) if key == "publicField")
                && matches!(&private.key, ClassFieldKeyIr::Private(_))
        ));
        let constructor = script
            .functions
            .iter()
            .find(|function| function.id == class.constructor_function_id)
            .expect("class constructor should be lowered");
        let instance_plan = constructor
            .class_instance_element_plan
            .as_ref()
            .expect("constructor should own its instance element plan");
        assert!(matches!(
            instance_plan.fields.as_slice(),
            [public, private]
                if matches!(&public.key, ClassFieldKeyIr::Public(key) if key == "publicInstance")
                    && matches!(&private.key, ClassFieldKeyIr::Private(_))
        ));
        assert_eq!(instance_plan.private_method_brands.len(), 1);
    }

    #[test]
    fn computed_class_field_keys_use_definition_order_cache_slots() {
        let program = lower_script(
            "function first() { return 'instance'; }
             function second() { return 'static'; }
             class C { [first()] = 1; static [second()] = 2; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let class = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical {
                    init:
                        TypedExpr {
                            expr: ExprIr::ClassDefinition(class),
                            ..
                        },
                    ..
                } => Some(class.as_ref()),
                _ => None,
            })
            .expect("class definition should be lowered");
        assert!(matches!(
            class.element_plan.definitions.as_slice(),
            [
                ClassElementDefinitionIr::ComputedFieldKey { slot: 0, .. },
                ClassElementDefinitionIr::ComputedFieldKey { slot: 1, .. },
            ]
        ));
        let constructor = script
            .functions
            .iter()
            .find(|function| function.id == class.constructor_function_id)
            .expect("class constructor should be lowered");
        let instance_plan = constructor
            .class_instance_element_plan
            .as_ref()
            .expect("constructor should own its instance field plan");
        assert!(matches!(
            instance_plan.fields.as_slice(),
            [ClassFieldInitIr {
                key: ClassFieldKeyIr::ComputedPublic(0),
                ..
            }]
        ));
        assert!(matches!(
            class.element_plan.static_elements.as_slice(),
            [ClassStaticElementIr::Field(ClassFieldInitIr {
                key: ClassFieldKeyIr::ComputedPublic(1),
                ..
            })]
        ));
    }

    #[test]
    fn computed_class_field_key_preserves_nested_runtime_global_resolution() {
        let program = lower_script(
            "function evaluate() { class C { [missingComputedName] = 1; } } evaluate();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let evaluate = script
            .functions
            .iter()
            .find(|function| function.name == "evaluate")
            .expect("evaluate function should be lowered");
        let class = evaluate
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical {
                    init:
                        TypedExpr {
                            expr: ExprIr::ClassDefinition(class),
                            ..
                        },
                    ..
                } => Some(class.as_ref()),
                _ => None,
            })
            .expect("class definition should be lowered");
        let [ClassElementDefinitionIr::ComputedFieldKey {
            key: PropertyKeyIr::StringExpr(key),
            ..
        }] = class.element_plan.definitions.as_slice()
        else {
            panic!("expected one computed field key");
        };
        assert!(matches!(
            &key.expr,
            ExprIr::GlobalIdentifierRead { name } if name == "missingComputedName"
        ));
    }

    #[test]
    fn later_instance_field_can_call_an_earlier_private_function_field() {
        let program = lower_script(
            "class C { #callable = () => 42; value = this.#callable(); } new C().value;",
        );

        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn generated_class_elements_record_their_execution_kind_and_strictness() {
        let program = lower_script(
            "function ordinary() {}
             class C {
                 instance = 1;
                 #private = 2;
                 static shared = 3;
                 static {}
                 method() {}
                 get value() {}
                 set value(next) {}
                 static sharedMethod() {}
                 static get sharedValue() {}
                 static set sharedValue(next) {}
             }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let class = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical {
                    init:
                        TypedExpr {
                            expr: ExprIr::ClassDefinition(class),
                            ..
                        },
                    ..
                } => Some(class.as_ref()),
                _ => None,
            })
            .expect("class definition should be lowered");

        let constructor = script
            .functions
            .iter()
            .find(|function| function.id == class.constructor_function_id)
            .expect("class constructor should be lowered");
        let instance_plan = constructor
            .class_instance_element_plan
            .as_ref()
            .expect("constructor should own its instance element plan");
        for field in &instance_plan.fields {
            let initializer_id = field
                .init_function_id
                .as_ref()
                .expect("field should have an initializer");
            let initializer = script
                .functions
                .iter()
                .find(|function| &function.id == initializer_id)
                .expect("field initializer function should be lowered");
            assert_eq!(
                initializer.class_element_execution_kind,
                ClassElementExecutionKind::InstanceFieldInitializer
            );
            assert!(initializer.strict);
            assert_eq!(initializer.class_kind, ClassFunctionKind::None);
        }
        for field in class
            .element_plan
            .static_elements
            .iter()
            .filter_map(|element| match element {
                ClassStaticElementIr::Field(field) => Some(field),
                ClassStaticElementIr::Block(_) => None,
            })
        {
            let initializer_id = field
                .init_function_id
                .as_ref()
                .expect("field should have an initializer");
            let initializer = script
                .functions
                .iter()
                .find(|function| &function.id == initializer_id)
                .expect("field initializer function should be lowered");
            assert_eq!(
                initializer.class_element_execution_kind,
                ClassElementExecutionKind::StaticFieldInitializer
            );
            assert!(initializer.strict);
            assert_eq!(initializer.class_kind, ClassFunctionKind::None);
        }

        let static_block_id = class
            .element_plan
            .static_elements
            .iter()
            .find_map(|element| match element {
                ClassStaticElementIr::Block(block) => Some(&block.function_id),
                ClassStaticElementIr::Field(_) => None,
            })
            .expect("static block should be planned");
        let static_block = script
            .functions
            .iter()
            .find(|function| &function.id == static_block_id)
            .expect("static block function should be lowered");
        assert_eq!(
            static_block.class_element_execution_kind,
            ClassElementExecutionKind::StaticBlock
        );
        assert!(static_block.strict);

        for function in script.functions.iter().filter(|function| {
            function.id == class.constructor_function_id
                || class.element_plan.definitions.iter().any(|definition| {
                    matches!(
                        definition,
                        ClassElementDefinitionIr::PublicMethod(method)
                            if method.function_id == function.id
                    )
                })
                || function.name == "ordinary"
        }) {
            assert_eq!(
                function.class_element_execution_kind,
                ClassElementExecutionKind::None
            );
            if function.name == "ordinary" {
                assert!(!function.strict);
            } else {
                assert!(function.strict, "{} should be strict", function.name);
            }
        }
    }

    #[test]
    fn nested_arrows_in_class_elements_capture_the_home_object() {
        let program = lower_script(
            "class Base {} Base.prototype.marker = 1; Base.marker = 2; class C extends Base { instance = (() => super.marker)(); static shared = (() => super.marker)(); static { this.block = (() => super.marker)(); } }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let arrows = script
            .functions
            .iter()
            .filter(|function| function.flavor == FunctionFlavor::Arrow)
            .collect::<Vec<_>>();

        assert_eq!(arrows.len(), 3);
        assert!(arrows.iter().all(|arrow| {
            arrow
                .captured_bindings
                .iter()
                .any(|binding| binding.source_name == LEXICAL_HOME_OBJECT_NAME)
        }));
    }

    #[test]
    fn class_parameter_defaults_capture_outer_bindings() {
        let program = lower_script(
            "function owner() { const fallback = 1; class C { constructor(value = fallback) {} method(value = fallback) { return value; } } return C; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        for member_name in ["C", "C.method"] {
            let member = script
                .functions
                .iter()
                .find(|function| function.name == member_name)
                .unwrap_or_else(|| panic!("class member `{member_name}` should be lowered"));
            assert!(member.captured_bindings.iter().any(|binding| {
                binding.source_name == "fallback" && binding.mode == BindingMode::Const
            }));
        }
    }

    #[test]
    fn private_class_callable_names_preserve_source_spelling_and_accessor_prefixes() {
        let program = lower_script(
            "class C {
                #instanceMethod() {}
                static #staticMethod() {}
                get #value() { return 1; }
                set #value(next) {}
                publicMethod() {}
            }
            class D { #instanceMethod() {} }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let function_names = script
            .functions
            .iter()
            .map(|function| function.name.as_str())
            .collect::<BTreeSet<_>>();

        for private_name in [
            "#instanceMethod",
            "#staticMethod",
            "get #value",
            "set #value",
        ] {
            assert!(
                function_names.contains(private_name),
                "missing private callable name `{private_name}` in {function_names:?}"
            );
        }
        assert!(function_names.contains("C.publicMethod"));

        let same_spelling = script
            .functions
            .iter()
            .filter(|function| function.name == "#instanceMethod")
            .collect::<Vec<_>>();
        assert_eq!(same_spelling.len(), 2);
        assert_ne!(same_spelling[0].id, same_spelling[1].id);
    }

    #[test]
    fn lowers_script_global_update_from_class_constructor() {
        let program = lower_script(
            "var count = 0; class Base { constructor() { count++; } } class Derived extends Base { constructor() { (_ => super())(); } } new Derived();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let base_constructor = script
            .functions
            .iter()
            .find(|function| function.name == "Base")
            .expect("base constructor should be lowered");
        assert!(matches!(
            base_constructor.body.statements.first(),
            Some(StatementIr::Expression(TypedExpr {
                expr: ExprIr::GlobalPropertyUpdate { name, .. },
                ..
            })) if name == "count"
        ));
    }

    #[test]
    fn carries_dynamic_function_apply_argument_list_to_runtime() {
        let program = lower_script(
            "function target() {} function call(args) { return target.apply(null, args); }",
        );
        assert!(program.is_wasm_supported());
    }

    #[test]
    fn carries_dynamic_method_receiver_to_runtime() {
        let program =
            lower_script("function call(receiver, name, args) { return receiver[name](...args); }");
        assert!(program.is_wasm_supported());
    }

    #[test]
    fn keeps_top_level_lexicals_out_of_script_global_bindings() {
        let program = lower_script("let x = 1; const y = 2; var z = 3; function f() {}");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(!script
            .global_bindings
            .iter()
            .any(|binding| binding.name == "x" || binding.name == "y"));
        assert!(script.global_bindings.iter().any(|binding| {
            binding.name == "z" && matches!(binding.kind, ScriptGlobalBindingKind::Var)
        }));
        assert!(script.global_bindings.iter().any(|binding| {
            binding.name == "f" && matches!(binding.kind, ScriptGlobalBindingKind::Function)
        }));
    }

    #[test]
    fn lowers_assignment_to_const_as_runtime_type_error_after_rhs_evaluation() {
        let program = lower_script("const x = 1; x = 2;");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expression) = &script.body.statements[1] else {
            panic!("expected assignment expression");
        };
        let ExprIr::Comma { lhs, rhs } = &expression.expr else {
            panic!("expected evaluated RHS before immutable-binding throw");
        };
        assert!(matches!(lhs.expr, ExprIr::Number(_)));
        assert!(matches!(
            rhs.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::TypeError,
                ..
            }
        ));
    }

    #[test]
    fn preserves_sloppy_named_function_binding_assignment_through_a_capture() {
        let program = lower_script(
            "const outer = function named() {
                 return function write() { named = 2; };
             };",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let write = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "write")
            .expect("capturing function should be lowered");
        assert!(write.body.statements.iter().any(|statement| matches!(
            statement,
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::Number(_),
                ..
            })
        )));
    }

    #[test]
    fn lowers_string_compound_assignment() {
        let program = lower_script("let s = \"a\"; s += \"b\";");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("compound_assigns=1"));
        assert_eq!(
            program
                .script
                .as_ref()
                .expect("script ir should exist")
                .result_kind(),
            ValueKind::String
        );
    }

    #[test]
    fn lowers_lone_surrogate_string_literal_with_internal_marker() {
        let program = lower_script("\"\\uD800\";");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::String(value) = &expr.expr else {
            panic!("expected string literal expression");
        };
        assert_eq!(value, &format!("{JS_STRING_SURROGATE_SENTINEL}D800"));
    }

    #[test]
    fn lowers_label_on_expression_statement() {
        let program = lower_script("label: 1;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(
            script.body.statements.first(),
            Some(StatementIr::Labelled { .. })
        ));
    }

    #[test]
    fn lowers_string_or_number_binding_addition_as_coercive_add() {
        let program = lower_script("var x; if (true) { x = 1; } else { x = \"a\"; } x + 1;");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("heap_coercions=1"));
    }

    #[test]
    fn lowers_dynamic_plus_proven_string_as_coercive_add() {
        let program = lower_script(
            "function choose(flag) { if (flag) return 1; return {}; } function format(message) { return message + \" suffix\"; } format(choose(true));",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("heap_coercions=2"));
    }

    #[test]
    fn lowers_maybe_string_plus_as_coercive_add() {
        let program = lower_script(
            "function choose(flag) { if (flag) return \"name\"; return 1; } choose(true) + 1;",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("heap_coercions=1"));
    }

    #[test]
    fn lowers_nested_maybe_string_plus_as_coercive_add() {
        let program = lower_script(
            "function choose(flag) { if (flag) return \"name\"; return 1; } choose(true) + 1 + \" suffix\";",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("heap_coercions=1"));
        assert!(summary.contains("string_concats=1"));
    }

    #[test]
    fn function_var_shadows_same_named_script_global_var() {
        let program = lower_script(
            "function helper() { var index = 0; index = index + 1; return index; } var index; for (var index in []) {} helper();",
        );
        assert!(program.is_wasm_supported());
    }

    #[test]
    fn merges_nested_function_arguments_into_script_global_value_info() {
        let program = lower_script(
            "var args = null; var close = function() { args = arguments; }; close(); args.length;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn lowers_arguments_reads_with_computed_string_keys() {
        let program = lower_script(
            "function readArgument(propertyKey) { return arguments[propertyKey]; } readArgument(\"0\");",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn lowers_arguments_symbol_iterator_reads() {
        let program = lower_script(
            "function readArgument(propertyKey) { return arguments[propertyKey]; } readArgument(\"0\"); readArgument(Symbol.iterator) === Array.prototype.values;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn lowers_arguments_object_to_string_reads() {
        let program = lower_script(
            "function describeArguments() { return arguments.toString(); } describeArguments();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn retains_nested_function_script_global_value_info_after_root_update() {
        let program = lower_script(
            "var args = null; var close = function() { args = arguments; }; close(); args = 1; args.length;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expression) = script.body.statements.last().unwrap() else {
            panic!("expected final expression");
        };
        let ExprIr::SpecOperation { operands, .. } = &expression.expr else {
            panic!(
                "expected final property read operation: {:?}",
                expression.expr
            );
        };
        let target = operands.first().expect("property read target");
        assert!(target.possible_kinds.contains(ValueKind::Arguments));
        assert!(target.possible_kinds.contains(ValueKind::Number));
    }

    #[test]
    fn nested_arrow_writes_script_global_before_property_read() {
        let program = lower_script(
            "var args = null; var close = () => { args = []; }; close(); args.length;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn nested_script_global_write_is_known_before_later_root_function_lowering() {
        let program = lower_script(
            "var args = null; function reader() { args.length; } function writer() { args = arguments; } reader(); writer(); args.length;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn nested_writer_in_later_root_function_is_known_to_earlier_reader() {
        let program = lower_script(
            "var args = null; function reader() { args.length; } function container() { var writer = function() { args = arguments; }; writer(); } reader(); container(); args.length;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    }

    #[test]
    fn read_only_nested_script_global_does_not_widen_later_root_update() {
        let program = lower_script(
            "var value = null; function reader() { value; } reader(); value = 1; value + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expression) = script.body.statements.last().unwrap() else {
            panic!("expected final expression");
        };
        assert_eq!(
            expression.possible_kinds,
            KindSet::from_kind(ValueKind::Number)
        );
    }

    #[test]
    fn lowers_unbound_identifier_read_as_runtime_global_resolution() {
        let program = lower_script("try { missingName; } catch (e) {}");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::TryCatch { try_block, .. } = &script.body.statements[0] else {
            panic!("expected try/catch statement");
        };
        let StatementIr::Expression(expr) = &try_block.statements[0] else {
            panic!("expected try expression statement");
        };
        assert!(matches!(
            expr.expr,
            ExprIr::GlobalIdentifierRead { ref name } if name == "missingName"
        ));
    }

    #[test]
    fn lowers_catch_parameter_source_alias_for_redeclared_var() {
        let program =
            lower_script("foo = \"prior\"; try { throw 1; } catch (foo) { var foo = \"init\"; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let (catch_name, catch_source_name, catch_block) = script
            .body
            .statements
            .iter()
            .find_map(|statement| {
                if let StatementIr::TryCatch {
                    catch_name,
                    catch_source_name,
                    catch_block,
                    ..
                } = statement
                {
                    Some((catch_name, catch_source_name, catch_block))
                } else {
                    None
                }
            })
            .expect("expected try/catch statement");

        assert_eq!(catch_source_name, "foo");
        assert_ne!(catch_name, catch_source_name);
        let StatementIr::Var(declarators) = &catch_block.statements[0] else {
            panic!("expected var declaration in catch block");
        };
        assert_eq!(declarators[0].name, "foo");
    }

    #[test]
    fn lowers_static_class_field_as_static_with_initializer_function() {
        let program = lower_script("class C { static x = 1; } C.x;");
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 2);
        let class_expr = match &script.body.statements[0] {
            StatementIr::Lexical { init, .. } => &init.expr,
            other => panic!("expected class lexical statement, got {other:?}"),
        };
        let ExprIr::ClassDefinition(class) = class_expr else {
            panic!("expected class definition");
        };
        let ClassStaticElementIr::Field(field) = &class.element_plan.static_elements[0] else {
            panic!("expected static field");
        };
        assert!(matches!(&field.key, ClassFieldKeyIr::Public(key) if key == "x"));
        assert!(field.init_function_id.is_some());
    }

    #[test]
    fn does_not_fold_number_wrapper_to_boolean_to_string() {
        let program = lower_script("throw (new Number()).toString();");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Throw(value) = &script.body.statements[0] else {
            panic!("expected throw statement");
        };
        let ExprIr::String(value) = &value.expr else {
            panic!("expected static string throw value, got {:?}", value.expr);
        };
        assert_eq!(value, "0");
    }

    #[test]
    fn lowers_json_parse_reviver_with_static_string_binding() {
        let program =
            lower_script("var json = \"[1, 2]\"; JSON.parse(json, function(k, v) { return v; });");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected JSON.parse expression");
        };
        assert!(matches!(expr.expr, ExprIr::JsonParseStaticReviver { .. }));
    }

    #[test]
    fn dynamic_json_parse_observes_reviver_holder_kinds() {
        let program = lower_script(
            "function parse(text) { return JSON.parse(text, function reviver(key, value) { this[1] = value; return value; }); } parse('[1, 2]');",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let reviver = script
            .functions
            .iter()
            .find(|function| function.name == "reviver")
            .expect("reviver should be lowered");
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::PropertyWrite { target, key, .. },
            ..
        }) = &reviver.body.statements[0]
        else {
            panic!("expected reviver holder write");
        };
        assert_eq!(target.kind, ValueKind::Dynamic);
        assert_eq!(
            target.possible_kinds,
            KindSet::from_kind(ValueKind::Object).union(KindSet::from_kind(ValueKind::Array))
        );
        assert!(matches!(key, PropertyKeyIr::ArrayIndex(_)));
    }

    #[test]
    fn does_not_fold_static_regexp_literal_exec() {
        let program = lower_script("/]/.exec(' ]{}')[0]; /\\c0/.exec('\\x0f\\x10\\x11');");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(first) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::PropertyRead { target, .. } = &first.expr else {
            panic!(
                "expected static regexp match array read, got {:?}",
                first.expr
            );
        };
        assert!(
            matches!(target.expr, ExprIr::CallMethod { .. }),
            "literal exec must retain observable RegExp.prototype.exec lookup, got {:?}",
            target.expr
        );
        let StatementIr::Expression(second) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        assert!(
            matches!(second.expr, ExprIr::CallMethod { .. }),
            "literal exec must not be folded to a static result, got {:?}",
            second.expr
        );
    }

    #[test]
    fn lowers_regexp_literals_as_intrinsic_leaves() {
        let program = lower_script("/t[a-b|q-s]/g; /a|b/;");
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(supported) = &script.body.statements[0] else {
            panic!("expected supported regexp literal expression");
        };
        let ExprIr::RegExpLiteral {
            source,
            flags,
            program,
        } = &supported.expr
        else {
            panic!(
                "expected intrinsic regexp literal, got {:?}",
                supported.expr
            );
        };
        assert_eq!(source, "t[a-b|q-s]");
        assert_eq!(flags, "g");
        assert_eq!(
            program.as_ref(),
            Some(&RegExpProgram::compile("t[a-b|q-s]", "g").expect("supported matcher"))
        );
        assert!(!matches!(supported.expr, ExprIr::Construct { .. }));

        let StatementIr::Expression(supported_alternation) = &script.body.statements[1] else {
            panic!("expected supported regexp literal expression");
        };
        let ExprIr::RegExpLiteral { program, .. } = &supported_alternation.expr else {
            panic!(
                "expected intrinsic regexp literal, got {:?}",
                supported_alternation.expr
            );
        };
        assert!(program.is_some());
        assert!(!matches!(
            supported_alternation.expr,
            ExprIr::Construct { .. }
        ));
    }

    #[test]
    fn annotates_only_supported_constant_regexp_construction() {
        let program = lower_script(
            r#"new RegExp("(.|\r|\n)*", ""); new RegExp("World"); new RegExp(); let pattern = "a"; new RegExp(pattern, ""); let expression = /a/; new RegExp(expression); new RegExp("(?=a)", ""); new RegExp("[", ""); new Date("2020-01-01");"#,
        );
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(constructed) = &script.body.statements[0] else {
            panic!("expected constructed regexp");
        };
        let ExprIr::Construct {
            static_regexp_compilation: Some(StaticRegExpCompilation::Program(static_program)),
            ..
        } = &constructed.expr
        else {
            panic!("expected supported constant constructed regexp annotation");
        };
        assert_eq!(
            static_program,
            &RegExpProgram::compile("(.|\r|\n)*", "").expect("program should compile")
        );
        assert!(static_program
            .instructions
            .iter()
            .any(|instruction| instruction.opcode == REGEXP_OPCODE_DOT));
        assert_eq!(static_program.capture_count, 1);

        let StatementIr::Expression(constructed_with_default_flags) = &script.body.statements[1]
        else {
            panic!("expected one-argument constructed regexp");
        };
        let ExprIr::Construct {
            static_regexp_compilation: Some(StaticRegExpCompilation::Program(static_program)),
            ..
        } = &constructed_with_default_flags.expr
        else {
            panic!("expected one-argument constructed regexp annotation");
        };
        assert_eq!(
            static_program,
            &RegExpProgram::compile("World", "").expect("program should compile")
        );

        for index in [2, 4, 6, 9] {
            let StatementIr::Expression(expr) = &script.body.statements[index] else {
                panic!("expected construct expression at {index}");
            };
            assert!(matches!(
                expr.expr,
                ExprIr::Construct {
                    static_regexp_compilation: None,
                    ..
                }
            ));
        }
        let StatementIr::Expression(constructed_lookahead) = &script.body.statements[7] else {
            panic!("expected lookahead construct expression");
        };
        assert!(matches!(
            constructed_lookahead.expr,
            ExprIr::Construct {
                static_regexp_compilation: Some(StaticRegExpCompilation::Program(_)),
                ..
            }
        ));
        let StatementIr::Expression(constructed_invalid) = &script.body.statements[8] else {
            panic!("expected invalid construct expression");
        };
        assert!(matches!(
            constructed_invalid.expr,
            ExprIr::Construct {
                static_regexp_compilation: Some(StaticRegExpCompilation::InvalidSyntax { .. }),
                ..
            }
        ));
    }

    #[test]
    fn annotates_only_direct_constant_regexp_calls() {
        let static_program_for = |source: &str| {
            let program = lower_script(source);
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(TypedExpr {
                expr:
                    ExprIr::CallIndirect {
                        static_regexp_compilation,
                        ..
                    },
                ..
            }) = script.body.statements.last().expect("expected RegExp call")
            else {
                panic!("expected indirect RegExp call");
            };
            static_regexp_compilation.clone()
        };

        assert_eq!(
            static_program_for(r#"RegExp("(?<name>a)");"#),
            Some(StaticRegExpCompilation::Program(
                RegExpProgram::compile("(?<name>a)", "").expect("program should compile")
            ))
        );
        assert_eq!(
            static_program_for(r#"RegExp("(?<π>a)", "u");"#),
            Some(StaticRegExpCompilation::Program(
                RegExpProgram::compile("(?<π>a)", "u").expect("program should compile")
            ))
        );
        assert!(
            static_program_for(r#"var pattern = "a"; RegExp(pattern);"#).is_none(),
            "dynamic patterns must not be annotated"
        );
        assert!(
            static_program_for(r#"var expression = /a/; RegExp(expression);"#).is_none(),
            "RegExp object identity calls must not be annotated"
        );
        assert!(
            static_program_for(r#"function RegExp() {} RegExp("a");"#).is_none(),
            "shadowed RegExp calls must not be annotated"
        );
        assert!(
            static_program_for(r#"RegExp = function () {}; RegExp("a");"#).is_none(),
            "reassigned RegExp calls must not be annotated"
        );
    }

    #[test]
    fn annotates_constant_regexp_prototype_compile_calls() {
        let program =
            lower_script(r#"let subject = /original/; subject.compile("[\ud834\udf06]", "u");"#);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expression) = script
            .body
            .statements
            .last()
            .expect("expected compile call")
        else {
            panic!("expected compile expression");
        };
        let Some(TypedExpr {
            expr:
                ExprIr::CallIndirect {
                    static_regexp_compilation:
                        Some(StaticRegExpCompilation::Program(static_program)),
                    ..
                },
            ..
        }) = indirect_call_body(expression)
        else {
            panic!(
                "expected annotated indirect RegExp.prototype.compile call, got {:#?}",
                script.body.statements.last()
            );
        };
        assert_eq!(
            static_program.instructions[0],
            RegExpInstruction::literal_code_point(0x1d306)
        );
    }

    #[test]
    fn annotates_invalid_constant_regexp_prototype_compile_calls() {
        let program = lower_script(r#"let subject = /original/; subject.compile(".{2,1}");"#);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expression) = script
            .body
            .statements
            .last()
            .expect("expected compile call")
        else {
            panic!("expected compile expression");
        };
        let Some(TypedExpr {
            expr:
                ExprIr::CallIndirect {
                    static_regexp_compilation:
                        Some(StaticRegExpCompilation::InvalidSyntax { message }),
                    ..
                },
            ..
        }) = indirect_call_body(expression)
        else {
            panic!("expected invalid static RegExp compilation annotation");
        };
        assert!(message.contains("regular-expression quantifier bounds are reversed"));
    }

    #[test]
    fn lowers_numbered_capture_programs_with_alternation() {
        let program = lower_script(r"/(\d+)/g; /(a|b)/;");
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(supported) = &script.body.statements[0] else {
            panic!("expected supported regexp literal expression");
        };
        let ExprIr::RegExpLiteral {
            program: Some(capture_program),
            ..
        } = &supported.expr
        else {
            panic!("expected first-slice capture matcher program");
        };
        assert_eq!(capture_program.capture_count, 1);

        let StatementIr::Expression(supported_alternation) = &script.body.statements[1] else {
            panic!("expected supported regexp literal expression");
        };
        let ExprIr::RegExpLiteral { program, .. } = &supported_alternation.expr else {
            panic!("expected intrinsic regexp literal");
        };
        assert!(program.is_some());
    }

    #[test]
    fn does_not_fold_stateful_regexp_literal_exec_or_test() {
        let program = lower_script("/b/y.exec('ab'); /a/g.test('a');");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(exec) = &script.body.statements[0] else {
            panic!("expected exec expression statement");
        };
        assert!(
            matches!(exec.expr, ExprIr::CallMethod { .. }),
            "sticky exec must retain runtime lastIndex semantics, got {:?}",
            exec.expr
        );

        let StatementIr::Expression(test) = &script.body.statements[1] else {
            panic!("expected test expression statement");
        };
        assert!(
            matches!(test.expr, ExprIr::CallMethod { .. }),
            "global test must retain runtime lastIndex semantics, got {:?}",
            test.expr
        );
    }

    #[test]
    fn lowers_optional_property_chain_as_one_ir_expression() {
        let program = lower_script("let a; a?.b;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected optional-chain expression statement");
        };
        let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
            panic!("expected one optional-property-chain IR expression");
        };
        assert!(matches!(target.expr, ExprIr::Identifier(_)));
        assert_eq!(chain.len(), 1);
        let OptionalChainOperationIr::Property { key, shorted } = &chain[0] else {
            panic!("expected optional property operation");
        };
        assert_eq!(key, &PropertyKeyIr::StaticString("b".to_string()));
        assert!(*shorted);
    }

    #[test]
    fn optional_property_chain_preserves_each_operation_shorted_flag() {
        for (source, expected_flags) in [
            ("let a; a?.b.c;", vec![true, false]),
            ("let a; a?.b?.c;", vec![true, true]),
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{source}");
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(expr) = &script.body.statements[1] else {
                panic!("expected optional-chain expression statement");
            };
            let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
                panic!("expected optional-property-chain IR expression");
            };
            assert_eq!(
                chain
                    .iter()
                    .map(|operation| match operation {
                        OptionalChainOperationIr::Property { shorted, .. }
                        | OptionalChainOperationIr::PrivateProperty { shorted, .. }
                        | OptionalChainOperationIr::Call { shorted, .. } => *shorted,
                    })
                    .collect::<Vec<_>>(),
                expected_flags,
                "{source}"
            );
        }
    }

    #[test]
    fn optional_private_access_preserves_chain_order_receiver_and_private_identity() {
        let source = "
            class A {
                #field = 1;
                #method() { return this; }
                read(o) { return o?.c.#field; }
                call(o) { return o?.#method(); }
            }
            class B {
                #field = 2;
                read(o) { return o?.c.#field; }
            }
        ";
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let mut field_ids = Vec::new();

        for function_name in ["A.read", "B.read"] {
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("missing `{function_name}`"));
            let StatementIr::Return(expr) = &function.body.statements[0] else {
                panic!("expected return from `{function_name}`");
            };
            let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
                panic!("expected optional chain from `{function_name}`");
            };
            assert!(matches!(
                chain.first(),
                Some(OptionalChainOperationIr::Property {
                    key: PropertyKeyIr::StaticString(key),
                    shorted: true,
                }) if key == "c"
            ));
            let Some(OptionalChainOperationIr::PrivateProperty {
                private_name_id,
                shorted: false,
            }) = chain.get(1)
            else {
                panic!("expected private tail from `{function_name}`: {chain:?}");
            };
            field_ids.push(*private_name_id);
        }
        assert_ne!(field_ids[0], field_ids[1]);

        let call = script
            .functions
            .iter()
            .find(|function| function.name == "A.call")
            .expect("missing `A.call`");
        let StatementIr::Return(expr) = &call.body.statements[0] else {
            panic!("expected return from `A.call`");
        };
        let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
            panic!("expected optional chain from `A.call`");
        };
        assert!(matches!(
            chain.as_slice(),
            [
                OptionalChainOperationIr::PrivateProperty { shorted: true, .. },
                OptionalChainOperationIr::Call {
                    args,
                    receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    shorted: false,
                    boundary_before: false,
                },
            ] if args.is_empty()
        ));
    }

    #[test]
    fn optional_property_chain_keeps_computed_key_inside_chain() {
        let program = lower_script("function key() { return 'x'; } let a; a?.[key()];");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[2] else {
            panic!("expected optional-chain expression statement");
        };
        let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
            panic!("expected optional-property-chain IR expression");
        };
        let OptionalChainOperationIr::Property { key, .. } = &chain[0] else {
            panic!("expected optional property operation, got {:?}", chain[0]);
        };
        let PropertyKeyIr::StringExpr(key) = key else {
            panic!("expected deferred computed key, got {key:?}");
        };
        assert!(
            matches!(
                key.expr,
                ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
            ),
            "expected the deferred key expression to contain the call, got {:?}",
            key.expr
        );
    }

    #[test]
    fn optional_call_is_retained_in_ordered_chain_ir() {
        let program = lower_script("function arg() { return 1; } let fn; fn?.(arg());");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[2] else {
            panic!("expected optional-call expression statement");
        };
        let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
            panic!("expected ordered optional-chain IR expression");
        };
        assert!(matches!(target.expr, ExprIr::Identifier(_)));
        assert_eq!(chain.len(), 1);
        let OptionalChainOperationIr::Call {
            args,
            receiver,
            shorted,
            boundary_before,
        } = &chain[0]
        else {
            panic!("expected optional call operation, got {:?}", chain[0]);
        };
        assert!(*shorted);
        assert_eq!(*receiver, OptionalChainCallReceiverIr::ReferenceOrUndefined);
        assert!(!*boundary_before);
        assert_eq!(args.len(), 1);
        assert!(matches!(
            args[0].expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));
    }

    #[test]
    fn optional_method_calls_preserve_reference_and_shorted_flags() {
        for (source, expected) in [
            (
                "let obj; obj?.method();",
                vec![("property", true), ("call", false)],
            ),
            (
                "let obj; obj.method?.();",
                vec![("property", false), ("call", true)],
            ),
            (
                "let obj; obj?.method?.().x;",
                vec![("property", true), ("call", true), ("property", false)],
            ),
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{source}");
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(expr) = &script.body.statements[1] else {
                panic!("expected optional-chain expression for {source}");
            };
            let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
                panic!("expected ordered optional-chain IR for {source}");
            };
            assert!(
                matches!(target.expr, ExprIr::Identifier(_)),
                "member-call base must be stored once for {source}, got {:?}",
                target.expr
            );
            assert_eq!(chain.len(), expected.len(), "{source}");
            assert_eq!(
                chain
                    .iter()
                    .map(|operation| match operation {
                        OptionalChainOperationIr::Property { shorted, .. } => {
                            ("property", *shorted)
                        }
                        OptionalChainOperationIr::PrivateProperty { shorted, .. } => {
                            ("private", *shorted)
                        }
                        OptionalChainOperationIr::Call { shorted, .. } => ("call", *shorted),
                    })
                    .collect::<Vec<_>>(),
                expected,
                "{source}"
            );
            let OptionalChainOperationIr::Property { key, .. } = &chain[0] else {
                unreachable!();
            };
            assert_eq!(key, &PropertyKeyIr::StaticString("method".to_string()));
            if source.ends_with(".x;") {
                let OptionalChainOperationIr::Property { key, .. } = &chain[2] else {
                    unreachable!();
                };
                assert_eq!(key, &PropertyKeyIr::StaticString("x".to_string()));
            }
        }
    }

    #[test]
    fn optional_member_call_keeps_effectful_base_once() {
        let program =
            lower_script("function base() { return { method() {} }; } base().method?.();");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = script.body.statements.last().unwrap() else {
            panic!("expected optional-call expression statement");
        };
        let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
            panic!("expected ordered optional-chain IR expression");
        };
        assert!(matches!(
            target.expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));
        assert!(matches!(
            chain.as_slice(),
            [
                OptionalChainOperationIr::Property {
                    key: PropertyKeyIr::StaticString(key),
                    shorted: false,
                },
                OptionalChainOperationIr::Call {
                    args,
                    receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    shorted: true,
                    boundary_before: false,
                },
            ] if key == "method" && args.is_empty()
        ));
    }

    #[test]
    fn grouped_optional_chain_calls_mark_new_short_circuit_boundaries() {
        for (source, expected_call_flags) in [
            ("let a; (a?.b)();", vec![(false, true)]),
            ("let a; (a?.b)?.();", vec![(true, true)]),
            ("let a; ((a?.b)())();", vec![(false, true), (false, true)]),
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(expr) = &script.body.statements[1] else {
                panic!("expected grouped optional-chain expression for {source}");
            };
            let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
                panic!("expected flattened optional-chain IR for {source}");
            };
            assert!(matches!(target.expr, ExprIr::Identifier(_)), "{source}");
            assert!(matches!(
                chain.first(),
                Some(OptionalChainOperationIr::Property {
                    key: PropertyKeyIr::StaticString(key),
                    shorted: true,
                }) if key == "b"
            ));
            assert_eq!(
                chain
                    .iter()
                    .filter_map(|operation| match operation {
                        OptionalChainOperationIr::Call {
                            shorted,
                            boundary_before,
                            ..
                        } => Some((*shorted, *boundary_before)),
                        OptionalChainOperationIr::Property { .. }
                        | OptionalChainOperationIr::PrivateProperty { .. } => None,
                    })
                    .collect::<Vec<_>>(),
                expected_call_flags,
                "{source}"
            );
        }
    }

    #[test]
    fn grouped_ordinary_call_keeps_arguments_after_optional_segment_boundary() {
        let program = lower_script("function arg() { return 1; } let a; (a?.b)(arg());");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[2] else {
            panic!("expected grouped optional-chain expression");
        };
        let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
            panic!("expected flattened optional-chain IR");
        };
        let Some(OptionalChainOperationIr::Call {
            args,
            receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
            shorted: false,
            boundary_before: true,
        }) = chain.last()
        else {
            panic!("expected ordinary grouped call boundary, got {chain:?}");
        };
        assert!(matches!(
            args.as_slice(),
            [TypedExpr {
                expr: ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. },
                ..
            }]
        ));
    }

    #[test]
    fn optional_primitive_method_calls_specialize_strict_this_as_primitive() {
        for (source, function_name, expected_kind) in [
            (
                "String.prototype.q = function stringQ() { 'use strict'; return this === 'z'; }; 'z'?.q();",
                "stringQ",
                ValueKind::String,
            ),
            (
                "Number.prototype.q = function numberQ() { 'use strict'; return this === 3; }; (3)?.q();",
                "numberQ",
                ValueKind::Number,
            ),
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("missing {function_name} for {source}"));
            let this_operand = function.body.statements.iter().find_map(|statement| {
                let StatementIr::Return(TypedExpr {
                    expr:
                        ExprIr::SpecOperation {
                            operation: SpecOperationIr::StrictEqualityComparison,
                            operands,
                        },
                    ..
                }) = statement
                else {
                    return None;
                };
                operands
                    .iter()
                    .find(|operand| matches!(operand.expr, ExprIr::This))
            });
            assert_eq!(
                this_operand.map(|operand| operand.kind),
                Some(expected_kind),
                "strict this specialization for {source}"
            );
        }
    }

    #[test]
    fn optional_factory_calls_converge_before_primitive_method_this_analysis() {
        for (source, factory_name, function_name, expected_kind) in [
            (
                "String.prototype.q = function stringFactoryQ() { 'use strict'; return this === 's'; }; function makeString() { return 's'; } makeString?.().q();",
                "makeString",
                "stringFactoryQ",
                ValueKind::String,
            ),
            (
                "Number.prototype.q = function numberFactoryQ() { 'use strict'; return this === 3; }; function makeNumber() { return 3; } makeNumber?.().q();",
                "makeNumber",
                "numberFactoryQ",
                ValueKind::Number,
            ),
            (
                "Boolean.prototype.q = function booleanFactoryQ() { 'use strict'; return this === true; }; function makeBoolean() { return true; } makeBoolean?.().q();",
                "makeBoolean",
                "booleanFactoryQ",
                ValueKind::Boolean,
            ),
            (
                "BigInt.prototype.q = function bigintFactoryQ() { 'use strict'; return this === 3n; }; function makeBigInt() { return 3n; } makeBigInt?.().q();",
                "makeBigInt",
                "bigintFactoryQ",
                ValueKind::BigInt,
            ),
            (
                "Symbol.prototype.q = function symbolFactoryQ() { 'use strict'; return this === this; }; function makeSymbol() { return Symbol('marker'); } makeSymbol?.().q();",
                "makeSymbol",
                "symbolFactoryQ",
                ValueKind::Symbol,
            ),
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let factory = script
                .functions
                .iter()
                .find(|function| function.name == factory_name)
                .unwrap_or_else(|| panic!("missing {factory_name} for {source}"));
            assert_eq!(
                factory.return_kind, expected_kind,
                "factory return for {source}"
            );
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("missing {function_name} for {source}"));
            let this_operand = function.body.statements.iter().find_map(|statement| {
                let StatementIr::Return(TypedExpr {
                    expr:
                        ExprIr::SpecOperation {
                            operation: SpecOperationIr::StrictEqualityComparison,
                            operands,
                        },
                    ..
                }) = statement
                else {
                    return None;
                };
                operands
                    .iter()
                    .find(|operand| matches!(operand.expr, ExprIr::This))
            });
            assert_eq!(
                this_operand.map(|operand| operand.kind),
                Some(expected_kind),
                "factory return must converge before method this analysis for {source}"
            );
        }
    }

    #[test]
    fn optional_eval_call_remains_explicitly_unsupported() {
        let program = lower_script("eval?.('source');");
        assert!(!program.is_wasm_supported());
        assert!(program.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("dynamic eval through optional call")
        }));
    }

    #[test]
    fn map_iterable_construction_preserves_the_map_instance_shape() {
        for source in [
            "new Map([]);",
            "function makeMap(iterable) { return new Map(iterable); } makeMap([]);",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
                panic!("expected constructed Map instance for {source}");
            };
            let Some(HeapShape::Object(instance_shape)) = instance.heap_shape.as_deref() else {
                panic!("expected Map instance shape for {source}");
            };
            let Some(HeapShape::Object(prototype_shape)) = instance_shape.prototype.as_deref()
            else {
                panic!("expected Map prototype shape for {source}");
            };
            assert!(prototype_shape.properties.contains_key("set"), "{source}");
            assert!(
                prototype_shape.properties.contains_key("forEach"),
                "{source}"
            );
        }
    }

    #[test]
    fn set_iterable_construction_preserves_the_set_instance_shape() {
        for source in [
            "new Set([]);",
            "function makeSet(iterable) { return new Set(iterable); } makeSet([]);",
            "new Set('ab');",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
                panic!("expected constructed Set instance for {source}");
            };
            let Some(HeapShape::Object(instance_shape)) = instance.heap_shape.as_deref() else {
                panic!("expected Set instance shape for {source}");
            };
            let Some(HeapShape::Object(prototype_shape)) = instance_shape.prototype.as_deref()
            else {
                panic!("expected Set prototype shape for {source}");
            };
            assert!(prototype_shape.properties.contains_key("add"), "{source}");
            assert!(
                prototype_shape.properties.contains_key("forEach"),
                "{source}"
            );
        }
    }

    #[test]
    fn set_algebra_preserves_the_set_instance_shape() {
        for method in ["difference", "intersection", "symmetricDifference", "union"] {
            let source = format!("new Set().{method}(new Set());");
            let program = lower_script(&source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
                panic!("expected Set algebra result for {source}");
            };
            let Some(HeapShape::Object(instance_shape)) = instance.heap_shape.as_deref() else {
                panic!("expected Set instance shape for {source}");
            };
            let Some(HeapShape::Object(prototype_shape)) = instance_shape.prototype.as_deref()
            else {
                panic!("expected Set prototype shape for {source}");
            };
            assert!(prototype_shape.properties.contains_key("add"), "{source}");
            assert!(prototype_shape.properties.contains_key("union"), "{source}");
        }
    }

    #[test]
    fn set_predicates_have_boolean_result_kind() {
        for method in ["isDisjointFrom", "isSubsetOf", "isSupersetOf"] {
            let source = format!("new Set().{method}(new Set());");
            let program = lower_script(&source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
                panic!("expected Set predicate result for {source}");
            };
            assert_eq!(result.kind, ValueKind::Boolean, "{source}");
            assert_eq!(
                result.possible_kinds,
                KindSet::from_kind(ValueKind::Boolean),
                "{source}"
            );
        }
    }

    #[test]
    fn optional_super_method_call_uses_current_this_receiver() {
        let source = "class Base { method() { return this; } } class Derived extends Base { call() { return super.method?.(); } }";
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "Derived.call")
            .expect("derived call method should be lowered");
        let StatementIr::Return(expr) = &function.body.statements[0] else {
            panic!(
                "expected return statement, got {:?}",
                function.body.statements
            );
        };
        let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
            panic!("expected optional super call IR, got {:?}", expr.expr);
        };
        assert!(matches!(
            &target.expr,
            ExprIr::SuperPropertyRead {
                key: PropertyKeyIr::StaticString(key),
            } if key == "method"
        ));
        assert!(matches!(
            chain.as_slice(),
            [OptionalChainOperationIr::Call {
                args,
                receiver: OptionalChainCallReceiverIr::CurrentThis,
                shorted: true,
                boundary_before: false,
            }] if args.is_empty()
        ));
    }

    #[test]
    fn optional_private_call_remains_explicitly_unsupported() {
        let source = "class C { #method() {} call() { return this.#method?.(); } }";
        let program = lower_script(source);
        assert!(!program.is_wasm_supported(), "{source}");
        assert!(
            program
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("optional private call")),
            "expected explicit optional private-call diagnostic: {:?}",
            program.diagnostics
        );
    }

    #[test]
    fn annex_b_block_functions_create_undefined_owner_bindings_and_copy_when_selected() {
        let program = lower_script(
            "if (false) { function unselected() {} } if (true) { function selected() {} }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        for name in ["unselected", "selected"] {
            assert!(script.global_bindings.iter().any(|binding| {
                binding.name == name && binding.kind == ScriptGlobalBindingKind::Var
            }));
        }
        let copies = collect_annex_b_copies(&script.body);
        assert_eq!(copies.len(), 2);
        assert!(copies.iter().any(|(source, block, target)| {
            source == "unselected" && block.starts_with("$annexb.block.") && target == source
        }));
        assert!(copies.iter().any(|(source, block, target)| {
            source == "selected" && block.starts_with("$annexb.block.") && target == source
        }));
    }

    #[test]
    fn annex_b_block_function_self_reference_captures_the_block_binding() {
        let program =
            lower_script("function owner() { { function f() { return f; } } return f; } owner();");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "f")
            .expect("block function should be lowered");
        assert!(function
            .captured_bindings
            .iter()
            .any(|binding| binding.name.starts_with("$annexb.block.")));
    }

    #[test]
    fn script_annex_b_block_function_self_reference_captures_the_block_binding() {
        let program = lower_script("if (true) function f() { return f; } f();");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "f")
            .expect("block function should be lowered");
        assert!(function.captured_bindings.iter().any(|binding| {
            binding.name.starts_with("$annexb.block.") && binding.source_name == "f"
        }));
    }

    #[test]
    fn annex_b_sibling_block_function_captures_the_block_binding() {
        let program = lower_script(
            "function owner() { let f = function () { return 2; }; { function f() { return 1; } function g() { return f(); } return g(); } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let sibling = script
            .functions
            .iter()
            .find(|function| function.name == "g")
            .expect("sibling block function should be lowered");
        let captured = sibling
            .captured_bindings
            .iter()
            .find(|binding| binding.name.starts_with("$annexb.block."))
            .expect("sibling should capture the block function binding");
        assert_ne!(captured.name, "f");
        assert!(!owner
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == captured.name));
        assert!(block_environment_owns_binding(
            &owner.body,
            &captured.name,
            captured.slot
        ));
    }

    #[test]
    fn block_function_captures_same_block_let_binding() {
        let program = lower_script(
            "function owner() { 'use strict'; let value = 2; { let value = 1; function read() { return value; } return read(); } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let reader = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("block function should be lowered");
        let captured = reader
            .captured_bindings
            .iter()
            .find(|binding| binding.name.starts_with("$scoped.lex."))
            .expect("block function should capture the block lexical binding");
        assert!(!owner
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == captured.name));
        assert!(block_environment_owns_binding(
            &owner.body,
            &captured.name,
            captured.slot
        ));
    }

    #[test]
    fn block_function_captures_same_block_class_binding() {
        let program = lower_script(
            "function owner() { 'use strict'; class Outer {} { class Local {} function read() { return Local; } return read(); } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let reader = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("block function should be lowered");
        let captured = reader
            .captured_bindings
            .iter()
            .find(|binding| binding.name.starts_with("$scoped.lex."))
            .expect("block function should capture the block class binding");
        assert!(!owner
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == captured.name));
        assert!(block_environment_owns_binding(
            &owner.body,
            &captured.name,
            captured.slot
        ));
    }

    #[test]
    fn nested_block_function_capture_uses_the_nearest_shadowing_binding() {
        let program = lower_script(
            "function owner() { 'use strict'; let value = 0; { let value = 1; { const value = 2; function read() { return value; } return read(); } } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let reader = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("nested block function should be lowered");
        let captured = reader
            .captured_bindings
            .iter()
            .find(|binding| binding.name.starts_with("$scoped.lex."))
            .expect("nested block function should capture a scoped lexical binding");
        assert_ne!(captured.name, "value");
        assert!(!owner
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == captured.name));
        assert!(block_environment_owns_binding(
            &owner.body,
            &captured.name,
            captured.slot
        ));
    }

    #[test]
    fn captured_block_bindings_use_nested_environment_hops_without_owner_activation() {
        let program = lower_script(
            "function owner() { { let outer = 1; { let inner = 2; function read() { return outer + inner; } return read; } } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let reader = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("reader function should be lowered");
        let outer = reader
            .captured_bindings
            .iter()
            .find(|binding| binding.source_name == "outer")
            .expect("outer block binding should be captured");
        let inner = reader
            .captured_bindings
            .iter()
            .find(|binding| binding.source_name == "inner")
            .expect("inner block binding should be captured");

        assert!(owner.owned_env_bindings.is_empty());
        assert_eq!(outer.hops, 1);
        assert_eq!(inner.hops, 0);
        assert!(block_environment_owns_binding(
            &owner.body,
            &outer.name,
            outer.slot
        ));
        assert!(block_environment_owns_binding(
            &owner.body,
            &inner.name,
            inner.slot
        ));
    }

    #[test]
    fn captured_for_of_binding_uses_one_hop_from_the_body_block() {
        let program = lower_script(
            "function owner() { let saved; for (let value of [1]) { let body = 2; function read() { return value + body; } saved = read; } return saved; } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let reader = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("reader function should be lowered");
        let capture_hops = reader
            .captured_bindings
            .iter()
            .map(|binding| (binding.source_name.as_str(), binding.hops))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(capture_hops.get("body"), Some(&0));
        assert_eq!(capture_hops.get("value"), Some(&1));
    }

    #[test]
    fn captured_block_bindings_skip_to_owner_activation_when_it_exists() {
        let program = lower_script(
            "function owner(argument) { { let outer = 1; { let inner = 2; function read() { return argument + outer + inner; } return read; } } } owner(3);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let reader = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("reader function should be lowered");
        let capture_hops = reader
            .captured_bindings
            .iter()
            .map(|binding| (binding.source_name.as_str(), binding.hops))
            .collect::<BTreeMap<_, _>>();

        assert!(owner
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == "argument"));
        assert_eq!(capture_hops.get("inner"), Some(&0));
        assert_eq!(capture_hops.get("outer"), Some(&1));
        assert_eq!(capture_hops.get("argument"), Some(&2));
    }

    #[test]
    fn switch_case_block_functions_share_lexical_capture_aliases() {
        let program = lower_script(
            "function owner() { 'use strict'; let value = 0; switch (1) { case 1: let value = 1; case 2: function read() { return value; } return read(); } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let reader = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("case block function should be lowered");
        let captured = reader
            .captured_bindings
            .iter()
            .find(|binding| binding.name.starts_with("$scoped.lex."))
            .expect("case block function should capture the case lexical binding");
        assert!(owner.owned_env_bindings.is_empty());
        let StatementIr::Switch {
            lexical_environment,
            ..
        } = owner
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Switch { .. }))
            .expect("owner should contain switch statement")
        else {
            panic!("expected switch statement");
        };
        assert!(lexical_environment.as_ref().is_some_and(|environment| {
            environment
                .bindings
                .iter()
                .any(|binding| binding.name == captured.name && binding.slot == captured.slot)
        }));
        assert!(block_environment_owns_binding(
            &owner.body,
            &captured.name,
            captured.slot
        ));
    }

    #[test]
    fn switch_selector_reads_its_shared_lexical_environment_in_tdz() {
        let program = lower_script(
            "function select() { let value = 1; switch (value) { case value: let value = 2; } } select();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let select = script
            .functions
            .iter()
            .find(|function| function.name == "select")
            .expect("select function should be lowered");
        let StatementIr::Switch { cases, .. } = select
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Switch { .. }))
            .expect("select should contain switch statement")
        else {
            panic!("expected switch statement");
        };
        let condition = cases[0]
            .condition
            .as_ref()
            .expect("case should have a selector");

        assert!(matches!(condition.expr, ExprIr::RuntimeThrow { .. }));
    }

    #[test]
    fn block_shadow_read_before_declaration_uses_the_inner_tdz_binding() {
        let program =
            lower_script("function owner() { let value = 1; { value; let value = 2; } } owner();");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let block = owner
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Block(block) => Some(block),
                _ => None,
            })
            .expect("owner should contain the shadowing block");

        assert!(matches!(
            &block.statements[0],
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::RuntimeThrow {
                    name: NativeErrorKind::ReferenceError,
                    ..
                },
                ..
            })
        ));
        let StatementIr::Lexical { name, .. } = &block.statements[1] else {
            panic!("expected the inner lexical declaration");
        };
        assert!(name.starts_with("$scoped.lex."));
    }

    #[test]
    fn block_self_initializer_uses_the_inner_tdz_binding() {
        let program =
            lower_script("function owner() { let value = 1; { let value = value; } } owner();");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let block = owner
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Block(block) => Some(block),
                _ => None,
            })
            .expect("owner should contain the shadowing block");
        let StatementIr::Lexical { init, .. } = &block.statements[0] else {
            panic!("expected the inner lexical declaration");
        };

        assert!(matches!(
            &init.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            }
        ));
    }

    #[test]
    fn switch_later_selector_reads_its_shared_lexical_environment_in_tdz() {
        let program = lower_script(
            "function select() { switch (1) { case 0: let value = 1; break; case value: } } select();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let select = script
            .functions
            .iter()
            .find(|function| function.name == "select")
            .expect("select function should be lowered");
        let StatementIr::Switch { cases, .. } = select
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Switch { .. }))
            .expect("select should contain switch statement")
        else {
            panic!("expected switch statement");
        };
        let condition = cases[1]
            .condition
            .as_ref()
            .expect("second case should have a selector");

        assert!(matches!(
            &condition.expr,
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            }
        ));
    }

    #[test]
    fn catch_block_functions_capture_catch_scope_lexical_bindings() {
        let program = lower_script(
            "function owner() { 'use strict'; let value = 0; try { throw 1; } catch (error) { let value = 1; function read() { return value; } return read(); } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let reader = script
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("catch block function should be lowered");
        let captured = reader
            .captured_bindings
            .iter()
            .find(|binding| binding.name.starts_with("$scoped.lex."))
            .expect("catch block function should capture the catch lexical binding");
        assert!(owner.owned_env_bindings.is_empty());
        assert!(block_environment_owns_binding(
            &owner.body,
            &captured.name,
            captured.slot
        ));
    }

    #[test]
    fn lowers_parent_linked_try_catch_finally_environment_layouts_and_hops() {
        let program = lower_script(
            "function owner() { try { let tried = 1; function readTried() { return tried; } throw 2; } catch (error) { let handled = error; function readHandled() { return error + handled; } return readHandled; } finally { let cleaned = 3; function readCleaned() { return cleaned; } } } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function should be lowered");
        let StatementIr::TryCatchFinally {
            try_block,
            catch_parameter_environment,
            catch_block,
            finally_block,
            ..
        } = owner
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::TryCatchFinally { .. }))
            .expect("owner should contain try/catch/finally")
        else {
            panic!("expected try/catch/finally statement");
        };

        assert!(try_block.lexical_environment.is_some());
        assert!(catch_parameter_environment.is_some());
        assert!(catch_block.lexical_environment.is_some());
        assert!(finally_block.lexical_environment.is_some());

        let handled = script
            .functions
            .iter()
            .find(|function| function.name == "readHandled")
            .expect("catch reader should be lowered");
        let capture_hops = handled
            .captured_bindings
            .iter()
            .map(|binding| (binding.source_name.as_str(), binding.hops))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(capture_hops.get("handled"), Some(&0));
        assert_eq!(capture_hops.get("error"), Some(&1));
    }

    #[test]
    fn strict_block_bindings_do_not_leak_to_sibling_functions() {
        let program = lower_script(
            "function owner() { 'use strict'; { function hidden() { return 1; } } function outside() { return typeof hidden; } return outside(); } owner();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let outside = script
            .functions
            .iter()
            .find(|function| function.name == "outside")
            .expect("sibling function should be lowered");
        assert!(outside.captured_bindings.is_empty());
    }

    #[test]
    fn annex_b_function_owner_parameter_and_arguments_bindings_block_outer_copies() {
        for source in [
            "function owner(f) { { function f() {} } return f; }",
            "function owner() { { function arguments() {} } return arguments; }",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script IR should exist");
            assert!(script
                .functions
                .iter()
                .all(|function| collect_annex_b_copies(&function.body).is_empty()));
        }
    }

    #[test]
    fn annex_b_top_level_lexical_binding_blocks_outer_copy() {
        let program = lower_script("let f = 1; { function f() {} } f;");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        assert!(collect_annex_b_copies(&script.body).is_empty());
        assert!(!script
            .global_bindings
            .iter()
            .any(|binding| binding.name == "f"));
    }

    #[test]
    fn annex_b_blocked_candidates_do_not_create_script_owner_bindings() {
        for (shape, source) in [
            (
                "block",
                "{ let f; { function f() {} } } typeof f; f; function outside() { return typeof f; }",
            ),
            (
                "switch",
                "switch (0) { default: let f; { function f() {} } } typeof f; f; function outside() { return typeof f; }",
            ),
        ] {
            let mut interner = Interner::default();
            let scope = Scope::new_global();
            let parsed_script = Parser::new(Source::from_bytes(source.as_bytes()))
                .parse_script(&scope, &mut interner)
                .expect("script should parse");
            let analysis = AnalysisBuilder::default().finish(&parsed_script, &interner, source);
            assert!(
                !analysis.owner_plans[SCRIPT_OWNER_ID]
                    .root_bindings
                    .contains("f"),
                "{shape}: blocked Annex B candidate must not create a script owner binding"
            );

            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{shape}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script IR should exist");
            assert!(
                !script
                    .global_bindings
                    .iter()
                    .any(|binding| binding.name == "f"),
                "{shape}: blocked Annex B candidate must not create a script global binding"
            );
            assert!(
                collect_annex_b_copies(&script.body)
                    .iter()
                    .all(|(source, _, _)| source != "f"),
                "{shape}: blocked Annex B candidate must not copy to the variable environment"
            );
            assert!(
                script.body.statements.iter().any(|statement| matches!(
                    statement,
                    StatementIr::Expression(TypedExpr {
                        expr: ExprIr::TypeOfUnresolvedIdentifier { .. },
                        ..
                    })
                )),
                "{shape}: outer typeof f must be unresolved"
            );
            assert!(
                script.body.statements.iter().any(|statement| matches!(
                    statement,
                    StatementIr::Expression(TypedExpr {
                        expr: ExprIr::GlobalIdentifierRead { name },
                        ..
                    }) if name == "f"
                )),
                "{shape}: outer f read must use runtime global resolution"
            );

            let outside = script
                .functions
                .iter()
                .find(|function| function.name == "outside")
                .expect("outside function should be lowered");
            assert!(
                outside.captured_bindings.is_empty(),
                "{shape}: outside callback must not capture blocked f"
            );
            assert!(
                outside.body.statements.iter().any(|statement| matches!(
                    statement,
                    StatementIr::Return(TypedExpr {
                        expr: ExprIr::TypeOfUnresolvedIdentifier { .. },
                        ..
                    })
                )),
                "{shape}: outside callback typeof f must be unresolved"
            );
        }
    }

    #[test]
    fn annex_b_existing_var_and_function_bindings_are_reused() {
        let program = lower_script(
            "function owner() { var f = 1; { function f() {} } function g() {} { function g() {} } return f; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner should be lowered");
        let copies = collect_annex_b_copies(&owner.body);
        assert_eq!(copies.len(), 2);
        assert!(copies
            .iter()
            .any(|(source, _, target)| source == "f" && target == "f"));
        assert!(copies
            .iter()
            .any(|(source, _, target)| source == "g" && target == "g"));
    }

    #[test]
    fn annex_b_copy_bypasses_a_same_named_catch_binding() {
        let program = lower_script(
            "function owner() { try { throw 1; } catch (f) { { function f() {} } } return f; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner should be lowered");
        let copies = collect_annex_b_copies(&owner.body);
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].0, "f");
        assert_eq!(copies[0].2, "f");
    }

    #[test]
    fn annex_b_duplicate_declarations_share_the_last_block_binding() {
        let program = lower_script(
            "function owner() { { function f() { return 1; } function f() { return 2; } } return f; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner should be lowered");
        let copies = collect_annex_b_copies(&owner.body);
        assert_eq!(copies.len(), 2);
        assert_eq!(copies[0].1, copies[1].1);
    }

    #[test]
    fn annex_b_switch_declarations_share_one_case_block_binding() {
        let program = lower_script(
            "function owner(v) { switch (v) { case 0: function f() { return 1; } break; default: function f() { return 2; } } return f; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner should be lowered");
        let switch = owner
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Switch {
                    lexical_declarations,
                    cases,
                    ..
                } => Some((lexical_declarations, cases)),
                _ => None,
            })
            .expect("switch should be lowered");
        assert_eq!(switch.0.len(), 1);
        assert_eq!(collect_annex_b_copies(&owner.body).len(), 2);
    }

    #[test]
    fn nested_labelled_function_declaration_remains_block_scoped() {
        let program = lower_script(
            "function owner() { var result; { label: function f() { return 6; } result = f(); } return result; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner should be lowered");
        assert!(collect_annex_b_copies(&owner.body).is_empty());
        assert!(script.functions.iter().any(|function| function.name == "f"));
    }

    #[test]
    fn annex_b_for_lexical_binding_blocks_outer_copy() {
        let program = lower_script(
            "function owner() { for (let f;;) { if (false) function _f() {} else function f() {} break; } return typeof f; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let owner = script
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner should be lowered");
        let copies = collect_annex_b_copies(&owner.body);
        assert!(copies.iter().any(|(source, _, _)| source == "_f"));
        assert!(copies.iter().all(|(source, _, _)| source != "f"));
    }

    #[test]
    fn tagged_template_lowers_its_tag_and_template_site() {
        let program = lower_script("(function(template) { return template; })`a${1}b`;");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(expression) = &script.body.statements[0] else {
            panic!("tagged template should lower as an expression statement");
        };
        let ExprIr::CallIndirect { callee, args, .. } = &expression.expr else {
            panic!("plain tagged template should lower as an indirect call");
        };
        assert!(matches!(callee.expr, ExprIr::FunctionValue(_)));
        let ExprIr::TemplateObject(template) = &args[0].expr else {
            panic!("first tagged-template argument should identify its template site");
        };
        assert_eq!(
            template.cooked,
            vec![Some("a".to_string()), Some("b".to_string())]
        );
        assert_eq!(template.raw, vec!["a".to_string(), "b".to_string()]);
    }
}
