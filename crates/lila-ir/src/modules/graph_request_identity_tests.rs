use super::super::graph_build::build_graph;
use super::*;

fn request(attributes: &[(&str, &str)]) -> ModuleRequestKeyIr {
    ModuleRequestKeyIr::try_new(
        "./dep.js",
        attributes.iter().map(|(key, value)| ImportAttributeIr {
            key: (*key).to_owned(),
            value: (*value).to_owned(),
        }),
    )
    .unwrap()
}

fn graph(source: &str, attributes: &[(&str, &str)], text_record: bool) -> ModuleGraphIr {
    let key = ModuleKey::from_host("/root/dep.js");
    let payload = "export const value = 41;".to_owned();
    let url = "file:///root/dep.js".to_owned();
    let dependency = if text_record {
        ModuleSourceIr::text(key, payload, url)
    } else {
        ModuleSourceIr::new(key, payload, url)
    };
    let input = ModuleGraphSources {
        realm_requests: Default::default(),
        modules: vec![
            ModuleSourceIr::new(
                ModuleKey::from_host("/root/entry.js"),
                source.to_owned(),
                "file:///root/entry.js".to_owned(),
            ),
            dependency,
        ],
        entry: 0,
        resolutions: vec![(0, request(attributes), 1)],
    };
    let mut graph = build_graph(&input).expect("explicit host records parse");
    link(&mut graph);
    graph
}

#[test]
fn public_host_request_identity_keeps_custom_attribute_values_exact() {
    // An embedder can define its own metadata for Source Text records. The
    // filesystem host has a narrower supported-attribute set; no policy is
    // inferred from these public in-memory rows or the dependency's filename.
    for source in [
        "import { value } from './dep.js' with { charset:'utf8', syntax:'javascript' }; value;",
        "export { value as renamed } from './dep.js' with { charset:'utf8', syntax:'javascript' };",
    ] {
        let exact = graph(
            source,
            &[("syntax", "javascript"), ("charset", "utf8")],
            false,
        );
        assert!(exact.link_errors.is_empty(), "{:?}", exact.link_errors);
        for wrong in [
            vec![("syntax", "javascript")],
            vec![("syntax", "javascript"), ("charset", "ascii")],
            vec![
                ("syntax", "javascript"),
                ("charset", "utf8"),
                ("extra", "ignored"),
            ],
        ] {
            let wrong = graph(source, &wrong, false);
            assert_eq!(
                wrong.link_errors,
                vec![ModuleLinkErrorIr::UnresolvedModule {
                    referrer: 0,
                    request: request(&[("charset", "utf8"), ("syntax", "javascript")]),
                }],
                "every source attribute remains in the exact request identity"
            );
        }
    }
}

#[test]
fn public_host_text_rows_require_exact_type_attributes_and_actual_text_records() {
    for source in [
        "import value from './dep.js' with { type:'text' }; value;",
        "export { default as renamed } from './dep.js' with { type:'text' };",
    ] {
        let exact = graph(source, &[("type", "text")], true);
        assert!(exact.link_errors.is_empty(), "{:?}", exact.link_errors);
        assert_eq!(exact.units[1].record.kind(), ModuleKindIr::Text);
        assert_eq!(
            exact.units[1].record.text_source(),
            Some("export const value = 41;")
        );
        assert_eq!(exact.exported_names(1), vec![ExportName::default_export()]);
        let resolved = exact.resolve_export(1, &ExportName::default_export());
        assert!(matches!(
            resolved,
            ResolvedBindingIr::Resolved { module: 1, .. }
        ));
        for (attributes, text_record) in [
            (vec![("type", "text")], false),
            (Vec::new(), true),
            (vec![("type", "json")], true),
            (vec![("type", "bytes")], true),
            (vec![("type", "text"), ("charset", "utf8")], true),
        ] {
            let wrong = graph(source, &attributes, text_record);
            assert_eq!(
                wrong.link_errors,
                vec![ModuleLinkErrorIr::UnresolvedModule {
                    referrer: 0,
                    request: request(&[("type", "text")]),
                }]
            );
        }
    }
    // Preserve both obsolete positive inputs as explicit rejection controls:
    // extra charset is not a text-module selector, and named JS exports cannot
    // satisfy the text record contract at the same locator.
    for source in [
        "import { value } from './dep.js' with { charset:'utf8', type:'text' }; value;",
        "export { value as renamed } from './dep.js' with { charset:'utf8', type:'text' };",
    ] {
        for text_record in [false, true] {
            let rejected = graph(
                source,
                &[("type", "text"), ("charset", "utf8")],
                text_record,
            );
            assert_eq!(
                rejected.link_errors,
                vec![ModuleLinkErrorIr::UnresolvedModule {
                    referrer: 0,
                    request: request(&[("charset", "utf8"), ("type", "text")]),
                }]
            );
        }
    }
}
