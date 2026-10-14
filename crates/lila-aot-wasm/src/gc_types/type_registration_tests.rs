use super::*;
use crate::module::StaticSignature;
use crate::runtime_artifact::ModuleKind;
use crate::runtime_helpers::{HelperOwner, HelperTypeRegion, ProgramHook, RuntimeHelperId};
use wasmparser::{CompositeInnerType, Parser, Payload, SubType, Validator, WasmFeatures};

fn encoded(kind: ModuleKind<'_>) -> (FinalizedRuntimeModule, Vec<u8>) {
    let runtime =
        RuntimeModuleTypes::register(kind).finalize_globals(GlobalLedger::new(), false, 0);
    let mut module = wasm_encoder::Module::new();
    module.section(runtime.types());
    module.section(&runtime.globals().defined_section(0));
    let bytes = module.finish();
    Validator::new_with_features(WasmFeatures::all())
        .validate_all(&bytes)
        .expect("registered concrete signatures, GC graph and hook globals validate");
    (runtime, bytes)
}

fn declarations(bytes: &[u8]) -> (Vec<SubType>, Vec<usize>, Vec<wasmparser::GlobalType>) {
    let mut types = Vec::new();
    let mut groups = Vec::new();
    let mut globals = Vec::new();
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.expect("registered module decodes") {
            Payload::TypeSection(reader) => {
                for group in reader {
                    let group = group.expect("type group decodes");
                    groups.push(group.types().len());
                    types.extend(group.types().cloned());
                }
            }
            Payload::GlobalSection(reader) => {
                globals.extend(reader.into_iter().map(|global| global.unwrap().ty));
            }
            _ => {}
        }
    }
    (types, groups, globals)
}

#[test]
fn direct_program_signatures_follow_the_exact_core_graph_and_hooks_keep_core_types() {
    let (runtime, runtime_bytes) = encoded(ModuleKind::Runtime);
    let (_, program_bytes) = encoded(ModuleKind::Standalone);
    let (core, core_groups, core_globals) = declarations(&runtime_bytes);
    let (program, program_groups, program_globals) = declarations(&program_bytes);
    let core_end = StaticSignature::ALL.len()
        + RuntimeHelperId::type_count(HelperTypeRegion::Core)
        + GcLayout::ALL.len();
    let program_count = RuntimeHelperId::type_count(HelperTypeRegion::Program);
    assert_eq!(core.len(), core_end, "R has no direct program signatures");
    assert_eq!(program.len(), core_end + program_count);
    assert_eq!(
        &program[..core_end],
        &core,
        "every exact core type is identical"
    );
    assert_eq!(&program_groups[..core_groups.len()], &core_groups);
    assert_eq!(program_groups.len(), core_groups.len() + program_count);
    assert!(
        program_groups[core_groups.len()..]
            .iter()
            .all(|size| *size == 1),
        "direct program signatures cannot enlarge the core recursion group"
    );
    assert_eq!(
        program_globals, core_globals,
        "the suffix cannot renumber core roots"
    );

    let mut core_indexes = std::collections::BTreeSet::new();
    let mut program_indexes = std::collections::BTreeSet::new();
    for helper in RuntimeHelperId::ALL {
        let index = helper.type_index() as usize;
        let expected = helper.definition(runtime.globals().schema().layouts());
        let CompositeInnerType::Func(actual) = &program[index].composite_type.inner else {
            panic!("{helper:?} must name its actual function type");
        };
        let convert = |values: &[wasmparser::ValType]| {
            values
                .iter()
                .copied()
                .map(|value| ValType::try_from(value).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            convert(actual.params()).as_slice(),
            expected.parameters(),
            "{helper:?} parameters"
        );
        assert_eq!(
            convert(actual.results()).as_slice(),
            expected.results(),
            "{helper:?} results"
        );
        match helper.owner() {
            HelperOwner::Runtime | HelperOwner::Hook(_) => {
                assert!(index < core_end && core_indexes.insert(index));
                assert_eq!(core[index], program[index]);
            }
            HelperOwner::Program => {
                assert!(index >= core_end && program_indexes.insert(index));
            }
        }
    }
    assert_eq!(
        core_indexes.into_iter().collect::<Vec<_>>(),
        (StaticSignature::ALL.len()..core_end - GcLayout::ALL.len()).collect::<Vec<_>>()
    );
    assert_eq!(
        program_indexes.into_iter().collect::<Vec<_>>(),
        (core_end..program.len()).collect::<Vec<_>>()
    );

    for hook in ProgramHook::ALL {
        let helper = hook.helper();
        assert!(
            helper.is_program_owned(),
            "the hook body remains program-owned"
        );
        assert_eq!(helper.type_region(), HelperTypeRegion::Core);
        let index = runtime.globals().schema().program_hook_global(hook) as usize;
        let global = core_globals[index];
        assert!(global.mutable);
        assert_eq!(
            ValType::try_from(global.content_type).unwrap(),
            ValType::Ref(RefType {
                nullable: true,
                heap_type: HeapType::Concrete(helper.type_index())
            }),
            "R's {hook:?} global consumes the exact core hook signature"
        );
    }
}
