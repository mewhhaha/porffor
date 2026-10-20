use std::sync::Arc;

use scriptw_runtime::{
    EmbeddedModuleBytesInput, EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph,
    EmbeddedModuleGraphError, EmbeddedModuleInput, EmbeddedModuleKind, EmbeddedModuleReferrer,
    EmbeddedModuleRequest, EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput,
    ModuleLoadingPolicy,
};

#[test]
fn raw_payload_and_requested_kind_bind_identity_without_changing_the_url() {
    let make = |payload: Vec<u8>| {
        EmbeddedModuleGraph::try_new_typed(
            entry(EmbeddedModuleGoal::Module),
            vec![EmbeddedModuleInput::Bytes(EmbeddedModuleBytesInput {
                identity: "case/entry.js".into(),
                bytes: payload,
                meta_url: "scriptw://metadata/entry.js".into(),
            })],
            vec![edge(
                EmbeddedModuleReferrer::Module("case/entry.js".into()),
                "./entry.js",
                vec![("type".into(), "bytes".into())],
                "case/entry.js",
            )],
        )
    };
    let graph = make(vec![0, 255, 128]).unwrap();
    assert_eq!(graph.modules().len(), 2);
    assert_eq!(graph.dependency_modules().count(), 1);
    let raw = graph
        .resolve(
            &EmbeddedModuleReferrer::Module("case/entry.js".into()),
            &request("./entry.js", vec![("type".into(), "bytes".into())]),
        )
        .unwrap();
    assert_eq!(raw.kind(), EmbeddedModuleKind::Bytes);
    assert_eq!(raw.payload_bytes(), &[0, 255, 128]);
    assert_eq!(raw.source_text(), None);
    assert_eq!(raw.meta_url(), graph.entry().meta_url());
    assert_ne!(
        graph.fingerprint(),
        make(vec![0, 255, 129]).unwrap().fingerprint()
    );
    assert_ne!(graph.fingerprint(), make(Vec::new()).unwrap().fingerprint());
}

#[test]
fn bytes_are_not_source_or_json_and_do_not_accept_extra_attributes() {
    for attributes in [
        Vec::new(),
        vec![("type".into(), "json".into())],
        vec![
            ("type".into(), "bytes".into()),
            ("extra".into(), "no".into()),
        ],
    ] {
        let graph = EmbeddedModuleGraph::try_new_typed(
            entry(EmbeddedModuleGoal::Script),
            vec![EmbeddedModuleInput::Bytes(EmbeddedModuleBytesInput {
                identity: "raw".into(),
                bytes: vec![255],
                meta_url: "scriptw://raw".into(),
            })],
            vec![edge(
                EmbeddedModuleReferrer::Script("case/entry.js".into()),
                "raw",
                attributes,
                "raw",
            )],
        );
        assert!(matches!(
            graph,
            Err(EmbeddedModuleGraphError::RecordKindMismatch { .. })
        ));
    }
}

#[test]
fn declared_json_kind_is_fingerprinted_and_requires_its_exact_type() {
    let make = |json: bool, attributes: Vec<(String, String)>| {
        EmbeddedModuleGraph::try_new_typed(
            entry(EmbeddedModuleGoal::Script),
            vec![if json {
                EmbeddedModuleInput::Json(module("case/data.json", "0"))
            } else {
                EmbeddedModuleInput::SourceText(module("case/data.json", "0"))
            }],
            vec![edge(
                EmbeddedModuleReferrer::Script("case/entry.js".into()),
                "./data.json",
                attributes,
                "case/data.json",
            )],
        )
    };
    let json = make(true, vec![("type".into(), "json".into())]).unwrap();
    assert_eq!(
        json.module("case/data.json", EmbeddedModuleKind::Json)
            .unwrap()
            .kind(),
        EmbeddedModuleKind::Json
    );
    let js = make(false, Vec::new()).unwrap();
    assert_ne!(json.fingerprint(), js.fingerprint());
    assert!(matches!(
        make(true, Vec::new()),
        Err(EmbeddedModuleGraphError::RecordKindMismatch { .. })
    ));
    assert!(matches!(
        make(false, vec![("type".into(), "json".into())]),
        Err(EmbeddedModuleGraphError::RecordKindMismatch { .. })
    ));
    assert!(make(
        false,
        vec![
            ("type".into(), "javascript".into()),
            ("mode".into(), "stable".into())
        ]
    )
    .is_ok());
    // Record kind alone changes the policy identity, even without a request row.
    let empty_rows = |input| {
        EmbeddedModuleGraph::try_new_typed(
            entry(EmbeddedModuleGoal::Script),
            vec![input],
            Vec::new(),
        )
        .unwrap()
    };
    assert_ne!(
        empty_rows(EmbeddedModuleInput::Json(module("case/data.json", "0"))).fingerprint(),
        empty_rows(EmbeddedModuleInput::SourceText(module(
            "case/data.json",
            "0"
        )))
        .fingerprint()
    );
}

fn entry(goal: EmbeddedModuleGoal) -> EmbeddedModuleEntryInput {
    EmbeddedModuleEntryInput {
        goal,
        identity: "case/entry.js".into(),
        source: "import('./shared.js');".into(),
        meta_url: "scriptw://metadata/entry.js".into(),
    }
}

fn module(identity: &str, source: &str) -> EmbeddedModuleSourceInput {
    EmbeddedModuleSourceInput {
        identity: identity.into(),
        source: source.into(),
        meta_url: format!("scriptw://metadata/{identity}"),
    }
}

fn edge(
    referrer: EmbeddedModuleReferrer,
    specifier: &str,
    attributes: Vec<(String, String)>,
    target: &str,
) -> EmbeddedModuleResolutionInput {
    EmbeddedModuleResolutionInput {
        referrer,
        specifier: specifier.into(),
        attributes,
        target: target.into(),
    }
}

fn request(specifier: &str, attributes: Vec<(String, String)>) -> EmbeddedModuleRequest {
    EmbeddedModuleRequest::try_new(specifier, attributes).unwrap()
}

#[test]
fn realm_requests_are_independent_of_source_and_unlocated_domains_and_fingerprinted() {
    let make = |origin| {
        EmbeddedModuleGraph::try_new(
            entry(EmbeddedModuleGoal::Script),
            vec![module("case/value.js", "export const value = 1;")],
            vec![edge(origin, "selected", Vec::new(), "case/value.js")],
        )
        .unwrap()
    };
    let realm = make(EmbeddedModuleReferrer::Realm);
    let key = request("selected", Vec::new());
    assert!(realm
        .resolve(&EmbeddedModuleReferrer::Realm, &key)
        .is_some());
    for origin in [
        EmbeddedModuleReferrer::Unlocated,
        EmbeddedModuleReferrer::Script("case/entry.js".into()),
    ] {
        assert!(realm.resolve(&origin, &key).is_none());
        assert_ne!(realm.fingerprint(), make(origin).fingerprint());
    }
}

#[test]
fn script_module_and_unlocated_requests_have_exact_distinct_referrers() {
    let script = EmbeddedModuleReferrer::Script("case/entry.js".into());
    let same_locator_module = EmbeddedModuleReferrer::Module("case/entry.js".into());
    let graph = EmbeddedModuleGraph::try_new(
        entry(EmbeddedModuleGoal::Script),
        vec![
            module("case/entry.js", "export const role = 'module';"),
            module("case/leaf.js", "export const role = 'leaf';"),
        ],
        vec![
            edge(script.clone(), "./entry.js", vec![], "case/entry.js"),
            edge(
                same_locator_module.clone(),
                "./entry.js",
                vec![],
                "case/leaf.js",
            ),
            edge(
                EmbeddedModuleReferrer::Unlocated,
                "declared-worker-request",
                vec![],
                "case/leaf.js",
            ),
        ],
    )
    .unwrap();
    let self_import = request("./entry.js", vec![]);
    assert_eq!(
        graph
            .resolve(&script, &self_import)
            .unwrap()
            .source_text()
            .unwrap(),
        "export const role = 'module';"
    );
    assert_eq!(
        graph
            .resolve(&same_locator_module, &self_import)
            .unwrap()
            .source_text()
            .unwrap(),
        "export const role = 'leaf';"
    );
    assert!(graph
        .resolve(&EmbeddedModuleReferrer::Unlocated, &self_import)
        .is_none());
    assert!(graph
        .resolve(
            &EmbeddedModuleReferrer::Script("other/entry.js".into()),
            &self_import,
        )
        .is_none());
    assert_eq!(
        graph
            .resolve(
                &EmbeddedModuleReferrer::Unlocated,
                &request("declared-worker-request", vec![]),
            )
            .unwrap()
            .identity(),
        "case/leaf.js"
    );
    // Metadata is an independently declared observation, not a resolution base.
    assert_eq!(graph.entry().meta_url(), "scriptw://metadata/entry.js");
    assert!(graph
        .resolve(
            &script,
            &request("scriptw://metadata/case/entry.js", vec![])
        )
        .is_none());
    assert_eq!(graph.dependency_modules().count(), 2);
}

#[test]
fn canonical_attributes_and_row_reordering_preserve_the_complete_snapshot() {
    let referrer = EmbeddedModuleReferrer::Module("case/entry.js".into());
    let attributes = vec![
        ("\u{e000}".into(), "bmp".into()),
        ("\u{10000}".into(), "astral".into()),
    ];
    let build = |reverse: bool| {
        let mut modules = vec![
            module("case/shared.js", "export default 1;"),
            module("case/unused.js", ""),
        ];
        let mut attributes = attributes.clone();
        let mut resolutions = vec![
            edge(
                referrer.clone(),
                "./shared.js",
                attributes.clone(),
                "case/shared.js",
            ),
            edge(referrer.clone(), "unused", vec![], "case/unused.js"),
        ];
        if reverse {
            modules.reverse();
            attributes.reverse();
            resolutions[0].attributes = attributes;
            resolutions.reverse();
        }
        EmbeddedModuleGraph::try_new(entry(EmbeddedModuleGoal::Module), modules, resolutions)
            .unwrap()
    };
    let graph = build(false);
    let reordered = build(true);
    assert_eq!(graph, reordered);
    assert_eq!(graph.fingerprint(), reordered.fingerprint());
    assert_eq!(graph.modules().len(), 3);
    assert_eq!(graph.dependency_modules().count(), 2);
    let projected = EmbeddedModuleGraph::try_new(
        EmbeddedModuleEntryInput {
            goal: graph.entry().goal(),
            identity: graph.entry().identity().into(),
            source: graph.entry().source().into(),
            meta_url: graph.entry().meta_url().into(),
        },
        graph
            .dependency_modules()
            .map(|source| EmbeddedModuleSourceInput {
                identity: source.identity().into(),
                source: source.source_text().unwrap().into(),
                meta_url: source.meta_url().into(),
            })
            .collect(),
        graph
            .resolutions()
            .iter()
            .map(|row| EmbeddedModuleResolutionInput {
                referrer: row.referrer().clone(),
                specifier: row.request().specifier().into(),
                attributes: row.request().attributes().to_vec(),
                target: row.target().into(),
            })
            .collect(),
    )
    .unwrap();
    assert_eq!(graph, projected);
    let canonical = request("./shared.js", attributes);
    assert_eq!(canonical.attributes()[0].0, "\u{10000}");
    assert_eq!(
        graph.resolve(&referrer, &canonical).unwrap().identity(),
        "case/shared.js"
    );
    assert!(graph
        .resolve(&referrer, &request("./shared.js", vec![]))
        .is_none());
    assert!(graph
        .resolve(
            &referrer,
            &request(
                "./shared.js",
                vec![
                    ("\u{10000}".into(), "different".into()),
                    ("\u{e000}".into(), "bmp".into())
                ]
            ),
        )
        .is_none());
    let policy = ModuleLoadingPolicy::Embedded(Arc::clone(&graph));
    let ModuleLoadingPolicy::Embedded(shared) = policy.clone() else {
        panic!("cloning a sealed policy must retain its graph");
    };
    assert!(Arc::ptr_eq(&graph, &shared));
    assert_eq!(
        ModuleLoadingPolicy::default(),
        ModuleLoadingPolicy::Filesystem
    );
}

#[test]
fn every_source_url_attribute_and_resolution_affects_the_fingerprint() {
    let build = |entry_source: &str,
                 entry_url: &str,
                 module_source: &str,
                 module_url: &str,
                 attribute: &str,
                 target: &str,
                 unused_source: &str| {
        let mut root = entry(EmbeddedModuleGoal::Module);
        root.source = entry_source.into();
        root.meta_url = entry_url.into();
        let mut shared = module("case/shared.js", module_source);
        shared.meta_url = module_url.into();
        EmbeddedModuleGraph::try_new(
            root,
            vec![shared, module("case/unused.js", unused_source)],
            vec![edge(
                EmbeddedModuleReferrer::Module("case/entry.js".into()),
                "./shared.js",
                vec![("mode".into(), attribute.into())],
                target,
            )],
        )
        .unwrap()
    };
    let baseline = build(
        "import('./shared.js');",
        "scriptw://root/entry.js",
        "export default 1;",
        "scriptw://shared/original.js",
        "one",
        "case/shared.js",
        "",
    );
    let variants = [
        build(
            "import('./shared.js'); void 0;",
            "scriptw://root/entry.js",
            "export default 1;",
            "scriptw://shared/original.js",
            "one",
            "case/shared.js",
            "",
        ),
        build(
            "import('./shared.js');",
            "scriptw://root/changed.js",
            "export default 1;",
            "scriptw://shared/original.js",
            "one",
            "case/shared.js",
            "",
        ),
        build(
            "import('./shared.js');",
            "scriptw://root/entry.js",
            "export default 2;",
            "scriptw://shared/original.js",
            "one",
            "case/shared.js",
            "",
        ),
        build(
            "import('./shared.js');",
            "scriptw://root/entry.js",
            "export default 1;",
            "scriptw://shared/changed.js",
            "one",
            "case/shared.js",
            "",
        ),
        build(
            "import('./shared.js');",
            "scriptw://root/entry.js",
            "export default 1;",
            "scriptw://shared/original.js",
            "two",
            "case/shared.js",
            "",
        ),
        build(
            "import('./shared.js');",
            "scriptw://root/entry.js",
            "export default 1;",
            "scriptw://shared/original.js",
            "one",
            "case/unused.js",
            "",
        ),
        build(
            "import('./shared.js');",
            "scriptw://root/entry.js",
            "export default 1;",
            "scriptw://shared/original.js",
            "one",
            "case/shared.js",
            "export default 'unused';",
        ),
    ];
    for changed in variants {
        assert_ne!(baseline.fingerprint(), changed.fingerprint());
    }
    // Two tuples with the same unframed concatenation must remain distinct.
    let first = EmbeddedModuleGraph::try_new(
        EmbeddedModuleEntryInput {
            goal: EmbeddedModuleGoal::Script,
            identity: "ab".into(),
            source: "c".into(),
            meta_url: "scriptw://entry.js".into(),
        },
        vec![],
        vec![],
    )
    .unwrap();
    let second = EmbeddedModuleGraph::try_new(
        EmbeddedModuleEntryInput {
            goal: EmbeddedModuleGoal::Script,
            identity: "a".into(),
            source: "bc".into(),
            meta_url: "scriptw://entry.js".into(),
        },
        vec![],
        vec![],
    )
    .unwrap();
    assert_ne!(first.fingerprint(), second.fingerprint());
    let without_unused_edge =
        EmbeddedModuleGraph::try_new(entry(EmbeddedModuleGoal::Module), vec![], vec![]).unwrap();
    let with_unused_edge = EmbeddedModuleGraph::try_new(
        entry(EmbeddedModuleGoal::Module),
        vec![],
        vec![edge(
            EmbeddedModuleReferrer::Unlocated,
            "unused",
            vec![],
            "case/entry.js",
        )],
    )
    .unwrap();
    assert_ne!(
        without_unused_edge.fingerprint(),
        with_unused_edge.fingerprint()
    );
    let script_goal = EmbeddedModuleGraph::try_new(
        entry(EmbeddedModuleGoal::Script),
        vec![module("case/entry.js", "import('./shared.js');")],
        vec![],
    )
    .unwrap();
    assert_ne!(without_unused_edge.fingerprint(), script_goal.fingerprint());
}

#[test]
fn constructor_rejects_unowned_and_ambiguous_rows_before_policy_creation() {
    let duplicate_root = EmbeddedModuleGraph::try_new(
        entry(EmbeddedModuleGoal::Module),
        vec![module("case/entry.js", "import('./shared.js');")],
        vec![],
    );
    assert!(matches!(
        duplicate_root,
        Err(EmbeddedModuleGraphError::DuplicateModule { .. })
    ));
    let unknown_referrer = EmbeddedModuleGraph::try_new(
        entry(EmbeddedModuleGoal::Script),
        vec![module("case/shared.js", "")],
        vec![edge(
            EmbeddedModuleReferrer::Module("case/entry.js".into()),
            "x",
            vec![],
            "case/shared.js",
        )],
    );
    assert!(matches!(
        unknown_referrer,
        Err(EmbeddedModuleGraphError::UnknownReferrer { .. })
    ));
    let missing = EmbeddedModuleGraph::try_new(
        entry(EmbeddedModuleGoal::Module),
        vec![],
        vec![edge(
            EmbeddedModuleReferrer::Module("case/entry.js".into()),
            "x",
            vec![],
            "case/missing.js",
        )],
    );
    assert!(matches!(
        missing,
        Err(EmbeddedModuleGraphError::MissingTarget { .. })
    ));
    let duplicate_attrs = EmbeddedModuleRequest::try_new(
        "x",
        vec![("type".into(), "one".into()), ("type".into(), "two".into())],
    );
    assert!(matches!(
        duplicate_attrs,
        Err(EmbeddedModuleGraphError::DuplicateAttribute { .. })
    ));
    let duplicate_edges = EmbeddedModuleGraph::try_new(
        entry(EmbeddedModuleGoal::Module),
        vec![module("case/shared.js", "")],
        vec![
            edge(
                EmbeddedModuleReferrer::Module("case/entry.js".into()),
                "x",
                vec![],
                "case/shared.js",
            ),
            edge(
                EmbeddedModuleReferrer::Module("case/entry.js".into()),
                "x",
                vec![],
                "case/entry.js",
            ),
        ],
    );
    assert!(matches!(
        duplicate_edges,
        Err(EmbeddedModuleGraphError::DuplicateResolution { .. })
    ));
    for identity in [
        "../entry.js",
        "case//entry.js",
        "/entry.js",
        "case/./entry.js",
        "case\\entry.js",
    ] {
        let mut root = entry(EmbeddedModuleGoal::Script);
        root.identity = identity.into();
        assert!(matches!(
            EmbeddedModuleGraph::try_new(root, vec![], vec![]),
            Err(EmbeddedModuleGraphError::InvalidIdentity { .. })
        ));
    }
    let mut root = entry(EmbeddedModuleGoal::Script);
    root.meta_url = "file:///ambient/entry.js".into();
    assert!(matches!(
        EmbeddedModuleGraph::try_new(root, vec![], vec![]),
        Err(EmbeddedModuleGraphError::InvalidMetaUrl { .. })
    ));
}

#[test]
fn text_payload_identity_preserves_exact_strings_and_mixed_self_records() {
    let make = |text: &str| {
        EmbeddedModuleGraph::try_new_typed(
            entry(EmbeddedModuleGoal::Module),
            vec![EmbeddedModuleInput::Text(EmbeddedModuleSourceInput {
                identity: "case/entry.js".into(),
                source: text.into(),
                meta_url: "scriptw://metadata/entry.js".into(),
            })],
            vec![edge(
                EmbeddedModuleReferrer::Module("case/entry.js".into()),
                "./entry.js",
                vec![("type".into(), "text".into())],
                "case/entry.js",
            )],
        )
    };
    let text = "\u{feff}invalid syntax!\0𝄞";
    let graph = make(text).unwrap();
    assert_eq!(graph.modules().len(), 2);
    assert_eq!(graph.dependency_modules().count(), 1);
    let raw = graph
        .resolve(
            &EmbeddedModuleReferrer::Module("case/entry.js".into()),
            &request("./entry.js", vec![("type".into(), "text".into())]),
        )
        .unwrap();
    assert_eq!(raw.kind(), EmbeddedModuleKind::Text);
    assert_eq!(raw.source_text(), Some(text));
    assert_eq!(raw.payload_bytes(), text.as_bytes());
    assert_ne!(
        graph.fingerprint(),
        make("different").unwrap().fingerprint()
    );
    assert!(graph
        .module("case/entry.js", EmbeddedModuleKind::SourceText)
        .is_some());
    for attributes in [
        vec![("type".into(), "json".into())],
        vec![("type".into(), "bytes".into())],
        vec![
            ("type".into(), "text".into()),
            ("extra".into(), "no".into()),
        ],
    ] {
        assert!(EmbeddedModuleGraph::try_new_typed(
            entry(EmbeddedModuleGoal::Script),
            vec![EmbeddedModuleInput::Text(EmbeddedModuleSourceInput {
                identity: "raw".into(),
                source: text.into(),
                meta_url: "scriptw://raw".into()
            })],
            vec![edge(
                EmbeddedModuleReferrer::Script("case/entry.js".into()),
                "raw",
                attributes,
                "raw"
            )],
        )
        .is_err());
    }
}

#[test]
fn opaque_module_source_records_bind_actual_kind_tag_and_exact_resolution() {
    use scriptw_runtime::{EmbeddedModuleContents, EmbeddedModuleSourceValueInput};
    let make = |tag: &str| {
        EmbeddedModuleGraph::try_new_typed(
            entry(EmbeddedModuleGoal::Script),
            vec![EmbeddedModuleInput::ModuleSource(
                EmbeddedModuleSourceValueInput {
                    identity: "host/source".into(),
                    source_tag: tag.into(),
                    meta_url: "scriptw://host/source".into(),
                },
            )],
            vec![edge(
                EmbeddedModuleReferrer::Script("case/entry.js".into()),
                "virtual-source",
                Vec::new(),
                "host/source",
            )],
        )
        .unwrap()
    };
    let graph = make("源🌐");
    let target = graph
        .resolve(
            &EmbeddedModuleReferrer::Script("case/entry.js".into()),
            &request("virtual-source", Vec::new()),
        )
        .unwrap();
    assert_eq!(target.kind(), EmbeddedModuleKind::ModuleSource);
    assert_eq!(
        graph.resolutions()[0].target_kind(),
        EmbeddedModuleKind::ModuleSource
    );
    assert_eq!(target.source_text(), None);
    assert!(
        matches!(target.contents(), EmbeddedModuleContents::ModuleSource { source_tag } if source_tag == "源🌐")
    );
    assert_eq!(target.payload_bytes(), "源🌐".as_bytes());
    assert_ne!(graph.fingerprint(), make("changed").fingerprint());
    assert!(graph
        .resolve(
            &EmbeddedModuleReferrer::Realm,
            &request("virtual-source", Vec::new())
        )
        .is_none());
}

#[test]
fn untyped_target_cannot_silently_select_between_source_text_and_native_source() {
    use scriptw_runtime::EmbeddedModuleSourceValueInput;
    let result = EmbeddedModuleGraph::try_new_typed(
        entry(EmbeddedModuleGoal::Script),
        vec![
            EmbeddedModuleInput::SourceText(module("host/source", "export default 7;")),
            EmbeddedModuleInput::ModuleSource(EmbeddedModuleSourceValueInput {
                identity: "host/source".into(),
                source_tag: "Source".into(),
                meta_url: "scriptw://host/source".into(),
            }),
        ],
        vec![edge(
            EmbeddedModuleReferrer::Script("case/entry.js".into()),
            "virtual-source",
            Vec::new(),
            "host/source",
        )],
    );
    assert!(matches!(
        result,
        Err(EmbeddedModuleGraphError::RecordKindMismatch { .. })
    ));
}
