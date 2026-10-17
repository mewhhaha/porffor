use super::*;
use crate::gc_types::GcStructHeapType;

#[test]
fn native_global_script_compiler_has_a_distinct_exact_gc_request_and_status_domain() {
    let host = StaticSignature::HostCompileGlobalScript;
    assert!(matches!(
        host.parameter_types(),
        [
            I32,
            Gc(GcLayout::StringValue, NonNullable),
            Gc(GcLayout::RealmRecord, NonNullable),
            Gc(GcLayout::ProgramInstance, NonNullable)
        ]
    ));
    assert_eq!(&host.result_types()[..5], &COMPLETION_TYPES);
    assert!(matches!(
        &host.result_types()[5..],
        [I32, Gc(GcLayout::StringValue, Nullable)]
    ));
    assert_ne!(
        host.type_index(),
        StaticSignature::HostCompileDynamicFunction.type_index()
    );
    assert_eq!(
        crate::GcHostImport::CompileGlobalScript.name(),
        "compile_global_script"
    );
    assert_ne!(
        crate::GcHostImport::CompileGlobalScript.name(),
        crate::GcHostImport::CompileDynamicFunction.name()
    );
    assert_eq!(
        crate::GcHostLayout::RealmRecord.field_count(),
        Some(crate::gc_types::RealmRecord::FIELD_COUNT)
    );
    assert!(crate::GcHostLayout::RealmRecord.array_element().is_none());
}
