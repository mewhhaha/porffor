use lila_ir::{
    DynamicImportAttributesIr, ImportAttributeIr, ModuleKey, ModuleRequestKeyIr, ModuleSourceIr,
};

fn request(specifier: &str, value: &str) -> ModuleRequestKeyIr {
    ModuleRequestKeyIr::try_new(
        specifier,
        [ImportAttributeIr {
            key: "type".into(),
            value: value.into(),
        }],
    )
    .unwrap()
}

#[test]
fn retained_sites_distinguish_literal_runtime_attributes_from_computed_known_attributes() {
    let source = "var options = {}; import('./data.json', options); import('./known.json', {with:{type:'json'}}); import(path, {with:{type:'css'}}); import('./proxy.json', {with:new Proxy({}, {})}); import('./abrupt.json', {get with(){throw 1;}}); // import(ignored)";
    let parsed = lila_front::parse(source, lila_front::ParseOptions::script()).unwrap();
    let lila_front::ParsedSource::Script(script) = parsed else {
        panic!("Script")
    };
    for unit in [
        ModuleSourceIr::from_parsed_script(
            ModuleKey::from_host("entry.js"),
            "lila://entry".into(),
            script,
        ),
        ModuleSourceIr::new(
            ModuleKey::from_host("entry.js"),
            source.into(),
            "lila://entry".into(),
        ),
    ] {
        let sites = unit.dynamic_import_sites().unwrap();
        assert_eq!(sites.len(), 5, "comments never declare call sites");
        assert_eq!(sites[0].static_specifier.as_deref(), Some("./data.json"));
        assert!(matches!(
            sites[0].attributes,
            DynamicImportAttributesIr::Runtime
        ));
        for kind in ["json", "bytes", "text"] {
            assert!(sites[0].permits_catalog_key(&request("./data.json", kind)));
            assert!(!sites[0].permits_catalog_key(&request("./other.json", kind)));
        }
        assert!(matches!(
            sites[1].attributes,
            DynamicImportAttributesIr::Known(_)
        ));
        assert!(sites[1].permits_catalog_key(&request("./known.json", "json")));
        assert!(!sites[1].permits_catalog_key(&request("./known.json", "bytes")));
        assert_eq!(sites[2].static_specifier, None);
        assert!(matches!(
            sites[2].attributes,
            DynamicImportAttributesIr::Known(_)
        ));
        assert!(sites[2].permits_catalog_key(&request("./any.json", "css")));
        assert!(!sites[2].permits_catalog_key(&request("./any.json", "json")));
        for (site, specifier) in sites[3..].iter().zip(["./proxy.json", "./abrupt.json"]) {
            assert_eq!(site.static_specifier.as_deref(), Some(specifier));
            assert!(matches!(
                site.attributes,
                DynamicImportAttributesIr::Runtime
            ));
            assert!(site.permits_catalog_key(&request(specifier, "json")));
        }
    }
}

#[test]
fn retained_site_queries_keep_unrepresentable_and_synthetic_sources_closed() {
    let source = ModuleSourceIr::new(
        ModuleKey::from_host("entry.js"),
        r#"import('\uD800', options);"#.into(),
        "lila://entry".into(),
    );
    let sites = source.dynamic_import_sites().unwrap();
    assert_eq!(sites.len(), 1);
    let spelling = sites[0].static_specifier.as_ref().unwrap();
    assert!(
        !sites[0].permits_catalog_key(&request(spelling, "json")),
        "display spelling cannot represent a lone-surrogate host key"
    );
    let text = ModuleSourceIr::text(
        ModuleKey::from_host("raw"),
        "import('./ignored', options)".into(),
        "lila://raw".into(),
    );
    assert_eq!(text.dynamic_import_sites(), Some(Vec::new()));
    let rejected = ModuleSourceIr::new(
        ModuleKey::from_host("bad.js"),
        "invalid syntax!".into(),
        "lila://bad".into(),
    );
    assert_eq!(rejected.dynamic_import_sites(), None);
}
