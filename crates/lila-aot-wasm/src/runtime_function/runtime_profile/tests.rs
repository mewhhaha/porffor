use super::*;
use std::borrow::Cow;
use wasm_encoder::{CustomSection, Module};

fn image(selection: &IntlDataSelection, corrupt: bool, duplicate: bool) -> Vec<u8> {
    let selected = selection.selected().unwrap();
    let mut module = Module::new();
    for (name, bytes) in selected.component_sections() {
        module.section(&CustomSection {
            name: Cow::Borrowed(name),
            data: Cow::Borrowed(bytes.as_ref()),
        });
    }
    let artifact_identity = selected.identity().artifact_identity();
    let identity = artifact_identity.as_bytes();
    module.section(&CustomSection {
        name: Cow::Borrowed(INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION),
        data: Cow::Borrowed(if corrupt {
            b"different physical image"
        } else {
            identity
        }),
    });
    if duplicate {
        module.section(&CustomSection {
            name: Cow::Borrowed(INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION),
            data: Cow::Borrowed(identity),
        });
    }
    if let Some(services) = selected.service_selection() {
        module.section(&CustomSection {
            name: Cow::Borrowed(INTL_SERVICE_SELECTION_CUSTOM_SECTION),
            data: Cow::Owned(services.wire().to_le_bytes().to_vec()),
        });
    }
    module.finish()
}

#[test]
fn current_runtime_requires_exact_physical_profile_and_complete_unique_frames() {
    let selection = IntlDataSelection::new(lila_intl::IntlCompilationProfile::Minimal);
    assert!(require_runtime_profile(&image(&selection, false, false), &selection).is_ok());
    assert!(require_runtime_profile(&image(&selection, true, false), &selection).is_err());
    assert!(require_runtime_profile(&image(&selection, false, true), &selection).is_err());
    assert!(require_runtime_profile(&Module::new().finish(), &selection).is_err());
}
