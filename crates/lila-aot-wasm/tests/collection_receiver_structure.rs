const COLLECTIONS_SOURCE: &str = include_str!("../src/builtins/collections.rs");

fn impl_body(type_name: &str, next_impl: &str) -> &'static str {
    COLLECTIONS_SOURCE
        .split_once(&format!("impl {type_name} {{"))
        .unwrap_or_else(|| panic!("{type_name} impl"))
        .1
        .split_once(&format!("impl {next_impl} {{"))
        .unwrap_or_else(|| panic!("{type_name} impl end"))
        .0
}

#[test]
fn collection_data_brands_have_one_mapping_authority() {
    let authority = impl_body(
        "CollectionDataReceiverKind",
        "CollectionReceiverRequirement",
    );
    for mapping in [
        "Self::Map => OBJECT_INTERNAL_BRAND_MAP,",
        "Self::WeakMap => OBJECT_INTERNAL_BRAND_WEAK_MAP,",
        "Self::Set => OBJECT_INTERNAL_BRAND_SET,",
        "Self::WeakSet => OBJECT_INTERNAL_BRAND_WEAK_SET,",
    ] {
        assert_eq!(authority.matches(mapping).count(), 1, "{mapping}");
    }

    for brand in [
        "OBJECT_INTERNAL_BRAND_MAP",
        "OBJECT_INTERNAL_BRAND_WEAK_MAP",
        "OBJECT_INTERNAL_BRAND_SET",
        "OBJECT_INTERNAL_BRAND_WEAK_SET",
    ] {
        let exact_uses = COLLECTIONS_SOURCE
            .match_indices(brand)
            .filter(|(offset, _)| {
                COLLECTIONS_SOURCE[*offset + brand.len()..]
                    .chars()
                    .next()
                    .map(|next| !next.is_ascii_alphanumeric() && next != '_')
                    .unwrap_or(true)
            })
            .count();
        assert_eq!(exact_uses, 1, "{brand} must have one mapping authority");
    }

    let set_kind = impl_body("SetCollectionKind", "MapCollectionKind");
    let map_kind = COLLECTIONS_SOURCE
        .split_once("impl MapCollectionKind {")
        .expect("MapCollectionKind impl")
        .1
        .split_once("enum SetAlgebraOperation")
        .expect("MapCollectionKind impl end")
        .0;
    for kind_impl in [set_kind, map_kind] {
        assert!(!kind_impl.contains("OBJECT_INTERNAL_BRAND_"));
    }
    assert_eq!(
        COLLECTIONS_SOURCE
            .matches("fn brand(self) -> u64 {\n        self.receiver_kind().brand()\n    }")
            .count(),
        2,
        "MapCollectionKind and SetCollectionKind must delegate brand selection"
    );
}
