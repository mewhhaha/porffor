#![cfg(test)]

use super::*;
use wasm_encoder::{CustomSection, EntityType, ImportSection, Module, TypeSection};

const COMPILERS: [&str; 3] = [
    "compile_dynamic_function",
    "compile_global_script",
    "compile_direct_eval",
];

// Metadata controls do not claim that these () -> () imports are the native
// GC ABI. Real product validation and Engine linking retain that obligation.
fn module(profile: &[u8], imports: &[(&str, &str)], frames: usize) -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([], []);
    module.section(&types);
    let mut section = ImportSection::new();
    for (namespace, name) in imports {
        section.import(namespace, name, EntityType::Function(0));
    }
    module.section(&section);
    for _ in 0..frames {
        module.section(&CustomSection {
            name: RUNTIME_SOURCE_COMPILATION_PROFILE_CUSTOM_SECTION.into(),
            data: profile.into(),
        });
    }
    module.finish()
}

#[test]
fn source_profile_requires_exact_physical_authority_and_compiler_import_set() {
    let native: Vec<_> = COMPILERS.iter().map(|name| ("lila_host", *name)).collect();
    assert_eq!(
        RuntimeSourceCompilationProfile::from_module(&module(&[0], &[], 1)).unwrap(),
        RuntimeSourceCompilationProfile::Portable
    );
    assert_eq!(
        RuntimeSourceCompilationProfile::from_module(&module(&[1], &native, 1)).unwrap(),
        RuntimeSourceCompilationProfile::NativeHost
    );
    for profile in [vec![], vec![2], vec![1, 0], vec![0, 0]] {
        assert!(RuntimeSourceCompilationProfile::from_module(&module(&profile, &[], 1)).is_err());
    }
    for frames in [0, 2] {
        assert!(RuntimeSourceCompilationProfile::from_module(&module(&[0], &[], frames)).is_err());
    }
    assert!(RuntimeSourceCompilationProfile::from_module(&module(&[0], &native, 1)).is_err());
    assert!(RuntimeSourceCompilationProfile::from_module(&module(&[1], &[], 1)).is_err());
    assert!(RuntimeSourceCompilationProfile::from_module(&module(&[1], &native[..2], 1)).is_err());
    let mut duplicate = native.clone();
    duplicate.push(native[0]);
    assert!(RuntimeSourceCompilationProfile::from_module(&module(&[1], &duplicate, 1)).is_err());
    let mut foreign = native;
    foreign[0].0 = "other_host";
    assert!(RuntimeSourceCompilationProfile::from_module(&module(&[1], &foreign, 1)).is_err());
}

#[test]
fn source_profile_separates_raw_cache_package_and_residency_identity() {
    use crate::runtime_artifact::cache::RuntimeIdentity;
    use lila_intl::{IntlCompilationProfile, IntlDataSelection};
    let selection = IntlDataSelection::new(IntlCompilationProfile::Minimal);
    let portable =
        RuntimeIdentity::admit(&selection, RuntimeSourceCompilationProfile::Portable).unwrap();
    let native =
        RuntimeIdentity::admit(&selection, RuntimeSourceCompilationProfile::NativeHost).unwrap();
    assert_ne!(portable.digest(), native.digest());
    assert_eq!(
        portable.digest(),
        RuntimeIdentity::admit(&selection, RuntimeSourceCompilationProfile::Portable)
            .unwrap()
            .digest()
    );
    assert_eq!(
        native.digest(),
        RuntimeIdentity::admit(&selection, RuntimeSourceCompilationProfile::NativeHost)
            .unwrap()
            .digest()
    );
}
