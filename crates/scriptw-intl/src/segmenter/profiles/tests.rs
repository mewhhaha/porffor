use super::*;

#[test]
fn malformed_profile_source_versions_catalogue_and_unknown_fields_are_rejected() {
    let original: serde_json::Value = serde_json::from_slice(SEGMENTER_PROFILE).unwrap();
    for (key, value) in [
        ("schema_version", serde_json::json!(2)),
        ("algorithm_version", serde_json::json!("2.0.0")),
        ("data_version", serde_json::json!("2.0.1")),
        ("unicode_version", serde_json::json!("17.0.0")),
        ("source_manifest_sha256", serde_json::json!("0".repeat(64))),
        ("default_locale", serde_json::json!("en")),
        ("unknown", serde_json::json!(true)),
    ] {
        let mut changed = original.clone();
        changed[key] = value;
        assert!(
            SegmenterProfiles::parse_profile(&serde_json::to_vec(&changed).unwrap()).is_err(),
            "{key}"
        );
    }
    for locales in [
        serde_json::json!(["en"]),
        serde_json::json!(["en", "en"]),
        serde_json::json!(["EN"]),
    ] {
        let mut changed = original.clone();
        changed["locales"] = locales;
        assert!(SegmenterProfiles::parse_profile(&serde_json::to_vec(&changed).unwrap()).is_err());
    }
}
struct Missing<'a> {
    provider: &'a SegmenterImageProvider,
    marker: DataMarkerInfo,
    attribute: Option<&'static str>,
    locale: Option<&'static str>,
}
impl<M: DataMarker> DataProvider<M> for Missing<'_>
where
    SegmenterImageProvider: DataProvider<M>,
{
    fn load(&self, req: DataRequest) -> Result<DataResponse<M>, DataError> {
        if M::INFO == self.marker
            && self
                .attribute
                .is_none_or(|a| req.id.marker_attributes.as_str() == a)
            && self.locale.is_none_or(|l| req.id.locale.to_string() == l)
        {
            return Err(DataErrorKind::IdentifierNotFound.with_req(M::INFO, req));
        }
        DataProvider::<M>::load(self.provider, req)
    }
}
#[test]
fn admission_rejects_each_missing_complex_model_before_optional_icu_fallback() {
    let profiles = crate::embedded_segmenter_data_image().unwrap().profiles();
    let provider = profiles._data.as_ref();
    certify(provider).expect("all genuine complex models must load before omission probes");
    for attribute in ["Burmese_", "Khmer_", "Lao_", "Thai_"] {
        assert!(
            certify(&Missing {
                provider,
                marker: SegmenterLstmAutoV1::INFO,
                attribute: Some(attribute),
                locale: None
            })
            .is_err(),
            "{attribute}"
        );
    }
    assert!(certify(&Missing {
        provider,
        marker: SegmenterDictionaryAutoV1::INFO,
        attribute: Some("cjdict"),
        locale: None
    })
    .is_err());
}
#[test]
fn admission_rejects_each_missing_base_table_or_required_locale_override() {
    let profiles = crate::embedded_segmenter_data_image().unwrap().profiles();
    let provider = profiles._data.as_ref();
    certify(provider).expect("complete admitted image before missing-table probes");
    for marker in [
        SegmenterBreakGraphemeClusterV1::INFO,
        SegmenterBreakWordV1::INFO,
        SegmenterBreakSentenceV1::INFO,
    ] {
        assert!(certify(&Missing {
            provider,
            marker,
            attribute: None,
            locale: None
        })
        .is_err());
    }
    for locale in ["fi", "sv"] {
        assert!(certify(&Missing {
            provider,
            marker: SegmenterBreakWordOverrideV1::INFO,
            attribute: None,
            locale: Some(locale)
        })
        .is_err());
    }
    assert!(certify(&Missing {
        provider,
        marker: SegmenterBreakSentenceOverrideV1::INFO,
        attribute: None,
        locale: Some("el")
    })
    .is_err());
}
