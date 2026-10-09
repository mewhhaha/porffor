use super::*;
use lila_intl::{CustomIntlProfile, CustomProfileId, IntlCompilationProfile, IntlDataSelection};
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
    cache::RuntimeIdentity::admit(&IntlDataSelection::new(IntlCompilationProfile::Minimal)).unwrap()
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
    assert_eq!(admitted.key(), package.key());
    assert_eq!(admitted.bytes().as_ref(), package.wasm());
    assert_eq!(admitted.layout(), package.runtime.layout());
    let target = std::thread::Builder::new()
        .name("runtime-build-package-target-parity".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            emit_runtime_artifact(&IntlDataSelection::new(IntlCompilationProfile::Minimal)).unwrap()
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
fn build_package_rejects_source_profile_abi_content_and_pool_corruption() {
    let package = fixture();
    let identity = minimal_identity();
    assert!(decode(package.package_bytes(), &identity, &[20; 32]).is_none());
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
    let other = cache::RuntimeIdentity::admit(&IntlDataSelection::new(
        IntlCompilationProfile::CustomProjection(profile),
    ))
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
