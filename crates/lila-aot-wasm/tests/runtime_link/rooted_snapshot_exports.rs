use std::collections::BTreeSet;
use std::sync::Arc;

use lila_aot_wasm::{
    emit, emit_with_rooted_snapshot, GcSnapshotLayout, PromiseRejectionPolicy,
    SNAPSHOT_ENTRY_REALM_EXPORT, SNAPSHOT_REALMS_EXPORT, SNAPSHOT_SYMBOLS_EXPORT,
};
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{ExternalKind, HeapType, Parser, Payload, ValType};

fn exports(
    bytes: &[u8],
) -> (
    Vec<(String, ExternalKind, u32)>,
    Vec<wasmparser::GlobalType>,
) {
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
}

fn snapshot_export_names(rows: &[(String, ExternalKind, u32)]) -> BTreeSet<&str> {
    rows.iter()
        .map(|(name, _, _)| name.as_str())
        .filter(|name| name.starts_with("lila_snapshot_"))
        .collect()
}

/// The runtime module declares the snapshot witnesses and roots for every
/// program; a program module never does. Asking for a snapshot only forces
/// the runtime link of a program that would otherwise be self-contained.
#[test]
fn snapshot_witness_exports_are_opt_in_actual_gc_globals() {
    let heap = lower(&parse("({ value: 7 })", ParseOptions::script()).unwrap());
    let normal = emit(&heap).unwrap();
    let graph = emit_with_rooted_snapshot(
        &heap,
        PromiseRejectionPolicy::default(),
        &lila_intl::IntlCompilationProfile::Minimal,
    )
    .unwrap();
    let runtime = normal.runtime().expect("heap program links R");
    assert!(Arc::ptr_eq(
        runtime,
        graph.runtime().expect("snapshot links R")
    ));
    assert_eq!(
        normal.bytes, graph.bytes,
        "a heap program's snapshot is the same program module"
    );

    let (program_exports, _) = exports(&normal.bytes);
    assert!(snapshot_export_names(&program_exports).is_empty());

    let (runtime_exports, runtime_globals) = exports(runtime.bytes());
    let expected = GcSnapshotLayout::ALL
        .iter()
        .map(|layout| layout.witness_export())
        .chain([
            SNAPSHOT_ENTRY_REALM_EXPORT,
            SNAPSHOT_REALMS_EXPORT,
            SNAPSHOT_SYMBOLS_EXPORT,
        ])
        .collect::<BTreeSet<_>>();
    assert_eq!(
        snapshot_export_names(&runtime_exports),
        expected,
        "R exports only the three roots and the declaration-derived witnesses"
    );
    for name in expected {
        assert_eq!(
            runtime_exports
                .iter()
                .filter(|(export, _, _)| export == name)
                .count(),
            1
        );
        let (_, kind, index) = runtime_exports
            .iter()
            .find(|(export, _, _)| export == name)
            .unwrap();
        assert_eq!(*kind, ExternalKind::Global, "{name}");
        let global = &runtime_globals[*index as usize];
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

    let free = lower(&parse("1;", ParseOptions::script()).unwrap());
    let plain = emit(&free).unwrap();
    assert!(
        plain.runtime().is_none(),
        "runtime-free IR stays self-contained"
    );
    assert!(snapshot_export_names(&exports(&plain.bytes).0).is_empty());
    let observed = emit_with_rooted_snapshot(
        &free,
        PromiseRejectionPolicy::default(),
        &lila_intl::IntlCompilationProfile::Minimal,
    )
    .unwrap();
    assert!(Arc::ptr_eq(
        runtime,
        observed.runtime().expect("a snapshot links R")
    ));
    assert!(snapshot_export_names(&exports(&observed.bytes).0).is_empty());
}
