use lila_aot_wasm::{
    emit, emit_with_rooted_snapshot, GcSnapshotLayout, PromiseRejectionPolicy,
    SNAPSHOT_ENTRY_REALM_EXPORT, SNAPSHOT_REALMS_EXPORT, SNAPSHOT_SYMBOLS_EXPORT,
};
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{ExternalKind, HeapType, Parser, Payload, ValType};

#[test]
fn snapshot_witness_exports_are_opt_in_actual_gc_globals() {
    let parsed = parse("({ value: 7 })", ParseOptions::script()).unwrap();
    let ir = lower(&parsed);
    let normal = emit(&ir).unwrap();
    let graph = emit_with_rooted_snapshot(
        &ir,
        PromiseRejectionPolicy::default(),
        &lila_intl::IntlCompilationProfile::Minimal,
    )
    .unwrap();
    let exports = |bytes: &[u8]| {
        let mut rows = Vec::new();
        let mut globals = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            match payload.unwrap() {
                Payload::ExportSection(section) => {
                    rows.extend(section.into_iter().map(|row| {
                        let row = row.unwrap();
                        (row.name.to_owned(), row.kind, row.index)
                    }));
                }
                Payload::GlobalSection(section) => {
                    globals.extend(section.into_iter().map(|global| global.unwrap().ty));
                }
                _ => {}
            }
        }
        (rows, globals)
    };
    let (normal_exports, normal_globals) = exports(&normal.bytes);
    assert!(!normal_exports
        .iter()
        .any(|(name, _, _)| name.starts_with("lila_snapshot_")));
    let (graph_exports, graph_globals) = exports(&graph.bytes);
    assert_eq!(
        graph_globals.len() - normal_globals.len(),
        GcSnapshotLayout::ALL.len() + 2,
        "snapshot mode adds only the two roots and declaration-derived witnesses"
    );
    for name in GcSnapshotLayout::ALL
        .iter()
        .map(|layout| layout.witness_export())
        .chain([
            SNAPSHOT_ENTRY_REALM_EXPORT,
            SNAPSHOT_REALMS_EXPORT,
            SNAPSHOT_SYMBOLS_EXPORT,
        ])
    {
        assert_eq!(
            graph_exports
                .iter()
                .filter(|(export, _, _)| export == name)
                .count(),
            1
        );
        assert!(graph_exports
            .iter()
            .any(|(export, kind, _)| export == name && *kind == ExternalKind::Global));
        let index = graph_exports
            .iter()
            .find(|(export, _, _)| export == name)
            .unwrap()
            .2;
        let global = &graph_globals[index as usize];
        let ValType::Ref(reference) = global.content_type else {
            panic!("{name} must expose its actual typed GC reference");
        };
        assert!(reference.is_nullable(), "{name}");
        assert!(
            matches!(reference.heap_type(), HeapType::Concrete(_)),
            "{name}"
        );
        assert_eq!(
            global.mutable,
            !GcSnapshotLayout::ALL
                .iter()
                .any(|layout| layout.witness_export() == name),
            "type witnesses are immutable; retained execution roots are mutable"
        );
    }
    assert_ne!(normal.bytes, graph.bytes);
}
