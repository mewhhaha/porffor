//! Index/declaration controls use synthetic bodies, without claiming Wasm execution.
use super::*;
use crate::{
    code_sink::LocalDeclarations, function_layout::FunctionIndexLayout,
    runtime_artifact::ModuleKind,
};
use wasm_encoder::Instruction;

fn body() -> Function {
    let mut body = Function::new(LocalDeclarations::from_types([]));
    body.instruction(&Instruction::End);
    body
}

fn identity(index: u32) -> FunctionIdentity {
    FunctionIdentity::Script {
        id: format!("index-{index}"),
        name: String::new(),
    }
}

#[test]
fn runtime_program_and_standalone_layouts_match_actual_emission_order() {
    let runtime = crate::runtime_artifact(&lila_intl::IntlDataSelection::new(
        lila_intl::IntlCompilationProfile::Minimal,
    ))
    .unwrap();
    for kind in [
        ModuleKind::Runtime,
        ModuleKind::Program(&runtime),
        ModuleKind::Standalone,
    ] {
        for imported in [0, 3] {
            for source_count in [0, 2] {
                for helper_count in [0, 2] {
                    let r_count = if kind.uses_heap() { source_count } else { 0 };
                    let r_helpers = if kind.uses_heap() { helper_count } else { 0 };
                    let p_helpers = if kind.uses_heap() { helper_count } else { 0 };
                    let layout = FunctionIndexLayout::new(
                        imported,
                        r_count,
                        0,
                        0,
                        r_helpers,
                        source_count,
                        p_helpers,
                        kind,
                    );
                    let boundary = imported + r_count as u32 + r_helpers as u32;
                    assert_eq!(layout.first_program_index(), boundary);
                    let linked = matches!(kind, ModuleKind::Program(_));
                    assert_eq!(layout.program_initialize_index().is_some(), linked);
                    assert_eq!(layout.program_prefix_count(), 1 + usize::from(linked));
                    if let Some(initialize) = layout.program_initialize_index() {
                        assert_eq!(initialize.raw(), boundary + 1);
                    }
                    if source_count > 0 {
                        assert_eq!(
                            layout.program_callable_index(0),
                            boundary + 1 + u32::from(linked)
                        );
                    }
                    assert_eq!(
                        layout.defined_functions().end,
                        boundary + 1 + u32::from(linked) + source_count as u32 + p_helpers as u32
                    );
                    let first = if kind.compiles_runtime() {
                        imported
                    } else {
                        boundary
                    };
                    let mut code = ModuleCode::new(first);
                    let mut expected = Vec::new();
                    if kind.compiles_runtime() {
                        for index in imported..boundary {
                            let id = identity(index);
                            expected.push(id.clone());
                            code.push(EmittedFunction::new(id, body()));
                        }
                    }
                    if kind.compiles_program() {
                        expected.push(FunctionIdentity::Main);
                        code.push(EmittedFunction::main(body()));
                        if let Some(index) = layout.program_initialize_index() {
                            let mut initialize =
                                EmittedFunction::new(FunctionIdentity::ProgramInitialize, body());
                            initialize.declaration = FunctionDeclaration::ProgramInitialize(index);
                            code.push(initialize);
                            expected.push(FunctionIdentity::ProgramInitialize);
                        }
                        for index in 0..source_count {
                            let id = identity(layout.program_callable_index(index));
                            expected.push(id.clone());
                            code.push(EmittedFunction::new(id, body()));
                        }
                        for helper in RuntimeHelperId::ALL.iter().take(p_helpers) {
                            expected.push(FunctionIdentity::RuntimeHelper(*helper));
                            code.push(EmittedFunction::runtime_helper(*helper, body()));
                        }
                    }
                    let (_, _, table) = code.finish();
                    assert_eq!(
                        table
                            .records()
                            .iter()
                            .map(|record| record.identity.clone())
                            .collect::<Vec<_>>(),
                        expected
                    );
                    assert_eq!(
                        table
                            .records()
                            .iter()
                            .map(|record| record.wasm_index)
                            .collect::<Vec<_>>(),
                        (first..first + expected.len() as u32).collect::<Vec<_>>()
                    );
                    if linked {
                        let initialize = layout.program_initialize_index().unwrap();
                        assert_eq!(
                            table.type_indices()[(initialize.raw() - first) as usize],
                            StaticSignature::ProgramInitialize.type_index()
                        );
                    }
                    if matches!(kind, ModuleKind::Runtime) {
                        assert_eq!(table.records().len(), r_count + r_helpers);
                    }
                    if matches!(kind, ModuleKind::Standalone) {
                        assert_eq!(table.records().len(), 1 + source_count);
                    }
                }
            }
        }
    }
}

#[test]
#[should_panic(expected = "P initializer emitted outside its planned index")]
fn a_bootstrap_body_cannot_be_published_at_main_position() {
    let runtime = crate::runtime_artifact(&lila_intl::IntlDataSelection::new(
        lila_intl::IntlCompilationProfile::Minimal,
    ))
    .unwrap();
    let layout = FunctionIndexLayout::new(0, 0, 0, 0, 0, 0, 0, ModuleKind::Program(&runtime));
    let index = layout.program_initialize_index().unwrap();
    let mut initialize = EmittedFunction::new(FunctionIdentity::ProgramInitialize, body());
    initialize.declaration = FunctionDeclaration::ProgramInitialize(index);
    ModuleCode::new(layout.main_index()).push(initialize);
}
