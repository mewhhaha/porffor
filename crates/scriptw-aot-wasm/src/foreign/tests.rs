use super::*;
use scriptw_ir::foreign::{ForeignErrorTag, ForeignModuleDeclaration};
use std::borrow::Cow;
use wasm_encoder::{CustomSection, Module, TypeSection, ValType};

fn registry(error_tag: bool) -> ForeignModuleRegistry {
    let function = ForeignFunctionDeclaration::new(
        "add",
        "dd",
        "add",
        vec![ForeignScalarType::Number; 2],
        ForeignResultType::Number,
    )
    .unwrap();
    let module = ForeignModuleDeclaration::new("dd:host", vec![function]).unwrap();
    let registry = ForeignModuleRegistry::new(vec![module]).unwrap();
    if error_tag {
        registry.with_error_tag(ForeignErrorTag::code_v1())
    } else {
        registry
    }
}

fn module(
    registry: Option<&ForeignModuleRegistry>,
    scalar: ValType,
    tag_payload: Option<ValType>,
    common_after_foreign: bool,
    duplicate_section: bool,
) -> Vec<u8> {
    let mut types = TypeSection::new();
    types.ty().function([scalar, scalar], [scalar]);
    if let Some(payload) = tag_payload {
        types.ty().function([payload], []);
    }
    let mut imports = ImportSection::new();
    imports.import("dd", "add", EntityType::Function(0));
    if common_after_foreign {
        imports.import(
            crate::RUNTIME_IMPORT_NAMESPACE,
            "f0",
            EntityType::Function(0),
        );
    }
    if tag_payload.is_some() {
        imports.import(
            "scriptw_foreign",
            "error_v1",
            EntityType::Tag(TagType {
                kind: TagKind::Exception,
                func_type_idx: 1,
            }),
        );
    }
    let mut module = Module::new();
    module.section(&types);
    module.section(&imports);
    if let Some(registry) = registry {
        let metadata = registry.to_normalized_json();
        let section = CustomSection {
            name: Cow::Borrowed(FOREIGN_MODULES_CUSTOM_SECTION),
            data: Cow::Borrowed(&metadata),
        };
        module.section(&section);
        if duplicate_section {
            module.section(&section);
        }
    }
    let bytes = module.finish();
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&bytes)
        .expect("actual encoded fixture validates before metadata admission");
    bytes
}

#[test]
fn foreign_metadata_admits_exact_scalar_imports_and_optional_error_tag() {
    let ordinary = registry(false);
    let fallible = registry(true);
    assert_eq!(
        foreign_registry_from_module(&module(Some(&ordinary), ValType::F64, None, false, false))
            .unwrap(),
        ordinary
    );
    assert_eq!(
        foreign_registry_from_module(&module(
            Some(&fallible),
            ValType::F64,
            Some(ValType::I32),
            false,
            false
        ))
        .unwrap(),
        fallible
    );
    assert!(foreign_registry_from_module(&Module::new().finish())
        .unwrap()
        .is_empty());
}

#[test]
fn foreign_metadata_rejects_missing_authority_wrong_scalar_types_and_duplicate_sections() {
    let registry = registry(false);
    for bytes in [
        module(None, ValType::F64, None, false, false),
        module(Some(&registry), ValType::I64, None, false, false),
        module(Some(&registry), ValType::F64, None, false, true),
        module(Some(&registry), ValType::F64, None, true, false),
    ] {
        assert!(foreign_registry_from_module(&bytes).is_err());
    }
}

#[test]
fn foreign_metadata_rejects_tag_inventory_and_payload_mismatches() {
    let ordinary = registry(false);
    let fallible = registry(true);
    for bytes in [
        module(
            Some(&ordinary),
            ValType::F64,
            Some(ValType::I32),
            false,
            false,
        ),
        module(Some(&fallible), ValType::F64, None, false, false),
        module(
            Some(&fallible),
            ValType::F64,
            Some(ValType::F64),
            false,
            false,
        ),
    ] {
        assert!(foreign_registry_from_module(&bytes).is_err());
    }
}

#[test]
fn foreign_metadata_rejects_empty_or_unsupported_version_sections() {
    let mut module = Module::new();
    module.section(&CustomSection {
        name: Cow::Borrowed(FOREIGN_MODULES_CUSTOM_SECTION),
        data: Cow::Owned(ForeignModuleRegistry::default().to_normalized_json()),
    });
    assert!(foreign_registry_from_module(&module.finish()).is_err());
    let bytes = self::module(Some(&registry(false)), ValType::F64, None, false, false);
    let version = b"\"abi_version\":1";
    let mut changed = bytes.clone();
    let matches = bytes
        .windows(version.len())
        .enumerate()
        .filter(|(_, part)| *part == version)
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "canonical actual metadata has one ABI version"
    );
    changed[matches[0] + version.len() - 1] = b'2';
    assert!(foreign_registry_from_module(&changed).is_err());
}
