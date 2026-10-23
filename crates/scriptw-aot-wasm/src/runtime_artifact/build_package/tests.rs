use super::*;
use scriptw_intl::{CustomIntlProfile, CustomProfileId, IntlCompilationProfile, IntlDataSelection};
use std::cell::Cell;
use std::sync::OnceLock;

const SOURCE: [u8; 32] = [19; 32];

fn fixture() -> &'static RuntimeBuildArtifact {
    static FIXTURE: OnceLock<RuntimeBuildArtifact> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        std::thread::Builder::new()
            .name("runtime-build-package-fixture".to_owned())
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                build_runtime_artifact(
                    &IntlDataSelection::new(IntlCompilationProfile::Minimal),
                    &SOURCE,
                )
                .expect("real build package emits and admits")
            })
            .expect("compiler worker starts")
            .join()
            .expect("build package emission completes")
    })
}

fn minimal_identity() -> cache::RuntimeIdentity {
    cache::RuntimeIdentity::admit(
        &IntlDataSelection::new(IntlCompilationProfile::Minimal),
        crate::RuntimeSourceCompilationProfile::Portable,
    )
    .unwrap()
}

fn resign(package: &mut [u8]) {
    let end = package.len() - DIGEST_BYTES;
    let digest = Sha256::digest(&package[..end]);
    package[end..].copy_from_slice(&digest);
}

#[test]
fn build_package_round_trips_real_runtime_and_matches_fresh_target_emission() {
    let package = fixture();
    let identity = minimal_identity();
    let admitted = decode(package.package_bytes(), &identity, &SOURCE)
        .expect("all actual Intl sections/imports/data/layout admit");
    assert_eq!(
        admitted.source_compilation_profile(),
        crate::RuntimeSourceCompilationProfile::Portable
    );
    assert_eq!(
        crate::RuntimeSourceCompilationProfile::from_module(package.wasm()).unwrap(),
        crate::RuntimeSourceCompilationProfile::Portable
    );
    assert_eq!(admitted.key(), package.key());
    assert_eq!(admitted.bytes().as_ref(), package.wasm());
    assert_eq!(admitted.layout(), package.runtime.layout());
    let concurrent = decode_with_jobs(
        package.package_bytes(),
        &identity,
        &SOURCE,
        std::num::NonZeroUsize::new(4).unwrap(),
    )
    .expect("every raw function and layout also admits with explicit jobs");
    assert_eq!(concurrent.bytes(), admitted.bytes());
    assert_eq!(concurrent.layout(), admitted.layout());
    // SAFETY: the static fixture is the exact immutable output of the producer
    // above, which fully admitted every function before returning its package.
    let immutable = unsafe {
        RuntimeArtifactInputs::new(None)
            .with_immutable_build_package(package.package_bytes(), &SOURCE)
    }
    .load_or_emit(&identity, || {
        panic!("immutable producer package must admit")
    })
    .unwrap();
    assert_eq!(immutable.key(), admitted.key());
    assert_eq!(immutable.bytes(), admitted.bytes());
    assert_eq!(immutable.layout(), admitted.layout());
    let target = std::thread::Builder::new()
        .name("runtime-build-package-target-parity".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            emit_runtime_artifact(
                &IntlDataSelection::new(IntlCompilationProfile::Minimal),
                crate::RuntimeSourceCompilationProfile::Portable,
            )
            .unwrap()
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(target.key(), package.key());
    assert_eq!(
        target.bytes().as_ref(),
        package.wasm(),
        "normal target emission owns the exact same raw R"
    );
    assert_eq!(target.layout(), admitted.layout());
}

#[test]
fn safe_package_replacement_rejects_self_checksummed_invalid_bodies_after_a_trusted_candidate() {
    let package = fixture();
    let identity = minimal_identity();
    let mut bytes = package.wasm().to_vec();
    let offset = wasmparser::Parser::new(0)
        .parse_all(&bytes)
        .find_map(|payload| match payload.unwrap() {
            wasmparser::Payload::CodeSectionEntry(body) => {
                Some(body.get_operators_reader().unwrap().original_position())
            }
            _ => None,
        })
        .unwrap();
    // Preserve every section, index, layout and data byte, but replace one body
    // opcode. Re-sign both the runtime key and package checksum, so only full
    // body validation can reject this otherwise well-formed envelope.
    bytes[offset] = 0xff;
    let layout = package.runtime.layout();
    let invalid = RuntimeArtifact::unadmitted_fixture(
        Arc::from(bytes),
        super::super::RuntimeLayout::new(
            layout.imported_functions,
            layout.function_types.clone(),
            layout.globals.clone(),
            layout.pool,
        ),
    );
    let invalid_package = encode(&invalid, &identity, &SOURCE);
    assert!(decode(&invalid_package, &identity, &SOURCE).is_none());
    let emitted = Cell::new(false);
    // SAFETY: only the first, unchanged static producer fixture is trusted.
    // Replacing it through the safe setter must restore complete validation.
    let inputs = unsafe {
        RuntimeArtifactInputs::new(None)
            .with_immutable_build_package(package.package_bytes(), &SOURCE)
    }
    .with_build_package(&invalid_package, &SOURCE)
    .with_admission_jobs(std::num::NonZeroUsize::new(4).unwrap());
    let admitted = inputs
        .load_or_emit(&identity, || {
            emitted.set(true);
            Ok(package.runtime.clone())
        })
        .unwrap();
    assert!(emitted.get());
    assert_eq!(admitted.key(), package.key());
    assert_eq!(admitted.bytes().as_ref(), package.wasm());
}

#[test]
fn admitted_build_package_resolves_without_emission_or_a_disk_cache() {
    let emitted = Cell::new(false);
    let runtime = RuntimeArtifactInputs::new(None)
        .with_build_package(fixture().package_bytes(), &SOURCE)
        .load_or_emit(&minimal_identity(), || {
            emitted.set(true);
            Err(EmitError::unsupported("unexpected emission"))
        })
        .unwrap();
    assert!(!emitted.get());
    assert_eq!(runtime.key(), fixture().key());
}

#[test]
fn previous_layout_build_package_falls_back_and_current_real_runtime_package_admits() {
    let package = fixture();
    let identity = minimal_identity();
    let mut previous = package.package_bytes().to_vec();
    let previous_version = 15u32;
    assert_ne!(LAYOUT_VERSION, previous_version);
    // Preserve the actual current R, source, profile and runtime key while
    // isolating the previous version field under a valid package checksum.
    previous[MAGIC.len()..MAGIC.len() + 4].copy_from_slice(&previous_version.to_le_bytes());
    resign(&mut previous);
    assert!(decode(&previous, &identity, &SOURCE).is_none());
    let emissions = Cell::new(0);
    let replacement = RuntimeArtifactInputs::new(None)
        .with_build_package(&previous, &SOURCE)
        .load_or_emit(&identity, || {
            emissions.set(emissions.get() + 1);
            Ok(package.runtime.clone())
        })
        .expect("a previous-layout package falls back to the current producer");
    assert_eq!(emissions.get(), 1);

    let current = RuntimeArtifactInputs::new(None)
        .with_build_package(package.package_bytes(), &SOURCE)
        .load_or_emit(&identity, || {
            panic!("the current real runtime package must admit directly")
        })
        .expect("the current real R passes raw package admission");
    for admitted in [&replacement, &current] {
        assert_eq!(admitted.key(), package.key());
        assert_eq!(admitted.bytes().as_ref(), package.wasm());
        assert_eq!(admitted.layout(), package.runtime.layout());
    }
}

// Envelope controls preserve current payload/checksum; no old-schema native proof.
#[test]
fn abi6_build_package_envelope_is_rejected_before_abi7_real_runtime_admission() {
    let package = fixture();
    let identity = minimal_identity();
    let mut previous = package.package_bytes().to_vec();
    let previous_version = 6u32;
    assert_ne!(LAYOUT_VERSION, previous_version);
    // Preserve the actual current R, source, profile and runtime key while
    // isolating the previous version field under a valid package checksum.
    previous[MAGIC.len()..MAGIC.len() + 4].copy_from_slice(&previous_version.to_le_bytes());
    resign(&mut previous);
    assert!(decode(&previous, &identity, &SOURCE).is_none());
    let emissions = Cell::new(0);
    let replacement = RuntimeArtifactInputs::new(None)
        .with_build_package(&previous, &SOURCE)
        .load_or_emit(&identity, || {
            emissions.set(emissions.get() + 1);
            Ok(package.runtime.clone())
        })
        .expect("a previous-layout package falls back to the current producer");
    assert_eq!(emissions.get(), 1);

    let current = RuntimeArtifactInputs::new(None)
        .with_build_package(package.package_bytes(), &SOURCE)
        .load_or_emit(&identity, || {
            panic!("the current real runtime package must admit directly")
        })
        .expect("the current real R passes raw package admission");
    for admitted in [&replacement, &current] {
        assert_eq!(admitted.key(), package.key());
        assert_eq!(admitted.bytes().as_ref(), package.wasm());
        assert_eq!(admitted.layout(), package.runtime.layout());
    }
}

#[test]
fn build_package_rejects_source_profile_abi_content_and_pool_corruption() {
    let package = fixture();
    let identity = minimal_identity();
    assert!(decode(package.package_bytes(), &identity, &[20; 32]).is_none());
    let native_identity = cache::RuntimeIdentity::admit(
        &IntlDataSelection::new(IntlCompilationProfile::Minimal),
        crate::RuntimeSourceCompilationProfile::NativeHost,
    )
    .unwrap();
    assert!(decode(package.package_bytes(), &native_identity, &SOURCE).is_none());
    let profile = CustomIntlProfile::new(
        CustomProfileId::parse("build-runtime-alternative").unwrap(),
        Some(&["en-US"]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    for end in [
        0,
        7,
        8,
        11,
        12,
        43,
        44,
        75,
        76,
        107,
        108,
        135,
        136,
        package.package_bytes().len() - 1,
    ] {
        assert!(decode(&package.package_bytes()[..end], &identity, &SOURCE).is_none());
    }
    let mut trailing = package.package_bytes().to_vec();
    trailing.push(0);
    assert!(decode(&trailing, &identity, &SOURCE).is_none());
    let other = cache::RuntimeIdentity::admit(
        &IntlDataSelection::new(IntlCompilationProfile::CustomProjection(profile)),
        crate::RuntimeSourceCompilationProfile::Portable,
    )
    .unwrap();
    assert!(decode(package.package_bytes(), &other, &SOURCE).is_none());
    for (offset, checked_digest) in [
        (0, true),
        (8, true),
        (12, true),
        (44, true),
        (76, true),
        (108, true),
        (128, true),
        (136, false),
        (136, true),
    ] {
        let mut damaged = package.package_bytes().to_vec();
        damaged[offset] ^= 0xff;
        if checked_digest {
            resign(&mut damaged);
        }
        assert!(
            decode(&damaged, &identity, &SOURCE).is_none(),
            "rejected field offset {offset}, resigned={checked_digest}"
        );
        let emitted = Cell::new(false);
        let result = RuntimeArtifactInputs::new(None)
            .with_build_package(&damaged, &SOURCE)
            .load_or_emit(&identity, || {
                emitted.set(true);
                Err(EmitError::unsupported("expected safe fallback"))
            });
        assert!(emitted.get());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expected safe fallback"));
    }
}

#[test]
fn abi7_build_package_falls_back_and_current_real_runtime_package_admits() {
    let previous_version = 7u32;
    assert_previous_build_package_replaced(previous_version);
}

#[test]
fn abi8_build_package_falls_back_before_host_prototype_layout_is_admitted() {
    let previous_version = 8u32;
    assert_previous_build_package_replaced(previous_version);
}

#[test]
fn abi9_artifact_is_replaced_before_program_payload_layout_is_admitted() {
    let previous_version = 9u32;
    assert_previous_build_package_replaced(previous_version);
}

#[test]
fn abi10_package_is_replaced_before_shared_naming_and_regexp_helpers_are_admitted() {
    assert_previous_build_package_replaced(10);
}

fn assert_previous_build_package_replaced(previous_version: u32) {
    let package = fixture();
    let identity = minimal_identity();
    let mut previous = package.package_bytes().to_vec();
    assert_ne!(LAYOUT_VERSION, previous_version);
    // Preserve the actual current R, source, profile and runtime key while
    // isolating the previous version field under a valid package checksum.
    previous[MAGIC.len()..MAGIC.len() + 4].copy_from_slice(&previous_version.to_le_bytes());
    resign(&mut previous);
    assert!(decode(&previous, &identity, &SOURCE).is_none());
    let emissions = Cell::new(0);
    let replacement = RuntimeArtifactInputs::new(None)
        .with_build_package(&previous, &SOURCE)
        .load_or_emit(&identity, || {
            emissions.set(emissions.get() + 1);
            Ok(package.runtime.clone())
        })
        .expect("a previous-layout package falls back to the current producer");
    assert_eq!(emissions.get(), 1);

    let current = RuntimeArtifactInputs::new(None)
        .with_build_package(package.package_bytes(), &SOURCE)
        .load_or_emit(&identity, || {
            panic!("the current real runtime package must admit directly")
        })
        .expect("the current real R passes raw package admission");
    for admitted in [&replacement, &current] {
        assert_eq!(admitted.key(), package.key());
        assert_eq!(admitted.bytes().as_ref(), package.wasm());
        assert_eq!(admitted.layout(), package.runtime.layout());
    }
}
