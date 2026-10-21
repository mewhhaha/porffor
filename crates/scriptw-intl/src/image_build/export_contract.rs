//! Exact identifier census required before exporting a marker payload.
use icu_provider::prelude::*;
use std::collections::BTreeSet;

pub(crate) fn require_identifiers(
    info: DataMarkerInfo,
    ids: &BTreeSet<DataIdentifierCow<'_>>,
) -> Result<(), &'static str> {
    if ids.is_empty() {
        return Err("empty pinned export marker inventory");
    }
    if info.is_singleton
        && (ids.len() != 1
            || ids
                .iter()
                .any(|id| !id.locale.is_unknown() || !id.marker_attributes.is_empty()))
    {
        return Err("singleton export marker requires exactly the empty-attribute und identifier");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_collator::provider::{CollationMetadataV1, CollationRootV1};
    #[test]
    fn export_census_rejects_empty_and_noncanonical_singletons() {
        let singleton = BTreeSet::from([DataIdentifierCow::default()]);
        require_identifiers(CollationRootV1::INFO, &singleton).unwrap();
        assert!(require_identifiers(CollationRootV1::INFO, &BTreeSet::new()).is_err());
        let locale = DataIdentifierCow::from_owned(
            DataMarkerAttributes::empty().to_owned(),
            "en".parse().unwrap(),
        );
        let attrs = DataMarkerAttributes::try_from_str("search")
            .unwrap()
            .to_owned();
        let attributed = DataIdentifierCow::from_owned(attrs, DataLocale::default());
        for ids in [
            BTreeSet::from([locale.clone()]),
            BTreeSet::from([attributed]),
            BTreeSet::from([DataIdentifierCow::default(), locale.clone()]),
        ] {
            assert!(require_identifiers(CollationRootV1::INFO, &ids).is_err());
        }
        require_identifiers(
            CollationMetadataV1::INFO,
            &BTreeSet::from([DataIdentifierCow::default(), locale]),
        )
        .unwrap();
    }
}
