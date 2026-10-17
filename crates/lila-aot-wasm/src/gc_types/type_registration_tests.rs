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
        .expect("registered concrete signatures and typed ProgramInstance graph validate");
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
    let (runtime, runtime_bytes) = encoded(ModuleKind::Runtime(
        crate::RuntimeSourceCompilationProfile::Portable,
    ));
    let (_, native_bytes) = encoded(ModuleKind::Runtime(
        crate::RuntimeSourceCompilationProfile::NativeHost,
    ));
    assert_eq!(
        runtime_bytes, native_bytes,
        "source compiler authority cannot change the core GC schema"
    );
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
            HelperOwner::Runtime | HelperOwner::Hook(_) | HelperOwner::ContextHook => {
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

    let instance_index = runtime
        .globals()
        .schema()
        .layouts()
        .layout_index(GcLayout::ProgramInstance) as usize;
    let CompositeInnerType::Struct(instance) = &core[instance_index].composite_type.inner else {
        panic!("ProgramInstance must be an actual GC struct");
    };
    assert_eq!(instance.fields.len(), 11);
    let policy_index = ProgramInstanceSchema::LANGUAGE_FEATURE_POLICY
        .ordinal()
        .raw() as usize;
    assert_eq!(policy_index, 9);
    assert_eq!(
        instance.fields[policy_index].element_type,
        wasmparser::StorageType::Val(wasmparser::ValType::I32)
    );
    assert!(!instance.fields[policy_index].mutable);
    let mut next_group_start = 0;
    let instance_group = core_groups
        .iter()
        .find_map(|size| {
            let range = next_group_start..next_group_start + *size;
            next_group_start = range.end;
            range.contains(&instance_index).then_some(range)
        })
        .expect("the actual ProgramInstance belongs to one emitted recursion group");
    assert!(instance_group.len() > 1);

    for hook in ProgramHook::ALL {
        let helper = hook.helper();
        assert!(
            helper.is_program_owned(),
            "the hook body remains program-owned"
        );
        assert_eq!(helper.type_region(), HelperTypeRegion::Core);
        assert!(
            instance_group.contains(&(helper.type_index() as usize)),
            "{hook:?} and ProgramInstance must be in the same actual recursion group"
        );
        let CompositeInnerType::Func(signature) =
            &core[helper.type_index() as usize].composite_type.inner
        else {
            panic!("the typed hook must reference its actual registered function type");
        };
        assert_eq!(
            ValType::try_from(signature.params()[0]).unwrap(),
            ValType::Ref(RefType {
                nullable: false,
                heap_type: HeapType::Concrete(instance_index as u32),
            }),
            "every typed hook receives the actual non-null ProgramInstance first"
        );

        let field = &instance.fields[hook as usize];
        assert!(!field.mutable, "a published P hook cannot be replaced");
        let wasmparser::StorageType::Val(actual) = field.element_type else {
            panic!("a ProgramInstance hook must be a typed function reference");
        };
        assert_eq!(
            ValType::try_from(actual).unwrap(),
            ValType::Ref(RefType {
                nullable: true,
                heap_type: HeapType::Concrete(helper.type_index()),
            }),
            "P's {hook:?} field consumes the exact registered core hook signature"
        );
        assert!(
            core_globals.iter().all(|global| {
                ValType::try_from(global.content_type).unwrap()
                    != ValType::Ref(RefType {
                        nullable: true,
                        heap_type: HeapType::Concrete(helper.type_index()),
                    })
            }),
            "the shared runtime must not retain a hook global"
        );
    }
}
