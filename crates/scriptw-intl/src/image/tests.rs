use super::*;

const MARKERS: &[&str] = &["locale/aliases/v1", "locale/parents/v1"];
fn frame() -> Arc<[u8]> {
    DataImageEnvelope::encode(
        DataImageComponent::LocaleTransforms,
        &IntlDataProfile::Minimal,
        MARKERS,
        b"actual typed consumer payload follows framing",
    )
    .unwrap()
}
fn rewrite_manifest(bytes: &[u8], edit: impl FnOnce(&mut serde_json::Value)) -> Arc<[u8]> {
    let len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let mut manifest: serde_json::Value = serde_json::from_slice(&bytes[12..12 + len]).unwrap();
    edit(&mut manifest);
    // Value serialises keys alphabetically; re-encode through the typed manifest
    // so edits that stay schema-valid keep the canonical field order. Edits that
    // break the schema (unknown fields) are written as edited.
    let manifest = match serde_json::from_value::<Manifest>(manifest.clone()) {
        Ok(typed) => serde_json::to_vec(&typed),
        Err(_) => serde_json::to_vec(&manifest),
    }
    .unwrap();
    let mut changed = MAGIC.to_vec();
    changed.extend_from_slice(&u32::try_from(manifest.len()).unwrap().to_le_bytes());
    changed.extend_from_slice(&manifest);
    changed.extend_from_slice(&bytes[12 + len..bytes.len() - 32]);
    let digest: [u8; 32] = Sha256::digest(&changed).into();
    changed.extend_from_slice(&digest);
    changed.into()
}
#[test]
fn deterministic_frames_bind_component_profile_and_complete_marker_inventory() {
    let bytes = frame();
    assert_eq!(bytes, frame());
    let image =
        DataImageEnvelope::decode(bytes.clone(), DataImageComponent::LocaleTransforms, MARKERS)
            .unwrap();
    assert_eq!(image.profile(), &IntlDataProfile::Minimal);
    assert_eq!(image.bytes(), bytes);
    assert!(matches!(
        DataImageEnvelope::decode(bytes.clone(), DataImageComponent::ListFormat, MARKERS),
        Err(IntlDataImageError::Component)
    ));
    assert!(matches!(
        DataImageEnvelope::decode(bytes, DataImageComponent::LocaleTransforms, &[MARKERS[0]]),
        Err(IntlDataImageError::MarkerInventory)
    ));
}
#[test]
fn inert_framing_rejects_corruption_truncation_trailing_bytes_and_version_drift() {
    let bytes = frame();
    let mut corrupt = bytes.to_vec();
    corrupt[20] ^= 1;
    assert!(matches!(
        DataImageEnvelope::decode(
            corrupt.into(),
            DataImageComponent::LocaleTransforms,
            MARKERS
        ),
        Err(IntlDataImageError::Digest)
    ));
    for extent in [0, 7, 12, bytes.len() - 1] {
        assert!(DataImageEnvelope::decode(
            Arc::from(&bytes[..extent]),
            DataImageComponent::LocaleTransforms,
            MARKERS
        )
        .is_err());
    }
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert!(matches!(
        DataImageEnvelope::decode(
            trailing.into(),
            DataImageComponent::LocaleTransforms,
            MARKERS
        ),
        Err(IntlDataImageError::Framing(_))
    ));
    let changed = rewrite_manifest(&bytes, |manifest| {
        manifest["versions"]["cldr"] = "unbound".into()
    });
    assert!(matches!(
        DataImageEnvelope::decode(changed, DataImageComponent::LocaleTransforms, MARKERS),
        Err(IntlDataImageError::Version)
    ));
}
#[test]
fn incomplete_conformance_and_self_declared_open_manifest_fields_are_refused() {
    let incomplete = DataImageEnvelope::encode(
        DataImageComponent::LocaleTransforms,
        &IntlDataProfile::Conformance,
        MARKERS,
        b"data",
    )
    .unwrap();
    // A profile label is valid framing, but cannot publish typed source data.
    assert!(crate::LocaleDataImage::from_bytes(incomplete).is_err());
    let changed = rewrite_manifest(&frame(), |manifest| {
        manifest["profile"]["kind"] = "conformance".into()
    });
    assert_eq!(
        DataImageEnvelope::decode(
            changed.clone(),
            DataImageComponent::LocaleTransforms,
            MARKERS
        )
        .unwrap()
        .profile(),
        &IntlDataProfile::Conformance
    );
    assert!(crate::LocaleDataImage::from_bytes(changed).is_err());
    let changed = rewrite_manifest(&frame(), |manifest| manifest["host_locale"] = "de".into());
    assert!(
        DataImageEnvelope::decode(changed, DataImageComponent::LocaleTransforms, MARKERS).is_err()
    );
}
