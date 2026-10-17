#!/usr/bin/env python3
"""Keep immutable builtin recipes on the real lowering path, with owned deltas."""
import re
import sys
from pathlib import Path
sys.dont_write_bytecode = True


def check(root: Path, overrides: dict | None = None) -> list[str]:
    overrides = overrides or {}
    errors = []
    def source(path):
        if path in overrides:
            return overrides[path]
        try:
            return (root / path).read_text()
        except FileNotFoundError:
            errors.append('missing catalog owner: ' + path)
            return ''
    def need(value, message):
        if not value:
            errors.append(message)
    prefix = 'crates/lila-ir/src/lowering/'
    lowering = source('crates/lila-ir/src/lowering.rs')
    catalog = source(prefix + 'builtin_catalog.rs')
    overlay = source(prefix + 'catalog_overlay.rs')
    flow = source(prefix + 'conditional_flow.rs')
    signatures = source(prefix + 'signature_evidence.rs')
    compact = lambda text: re.sub(r'\s+', '', text)
    need('mod catalog_overlay;' in lowering, 'real lowering must own the overlay module')
    need(lowering.count('function_signatures: FunctionSignatures,') == 1
         and lowering.count('global_properties: GlobalProperties,') == 2,
         'all lowering and executed-class facts must retain typed overlays')
    need('self.function_signatures = builtin_catalog::function_signatures();' in lowering,
         'real signature initialization must use the shared catalog')
    need('GlobalProperties::new(global_property_baseline(policy))' in catalog
         and 'FunctionSignatures::new(function_signature_baseline())' in catalog,
         'catalog accessors must borrow the immutable recipes')
    need('global_property_baseline(policy).clone()' not in catalog
         and 'function_signature_baseline().clone()' not in catalog,
         'catalog accessors cannot copy whole builtin maps')
    need("baseline:Option<&'staticBTreeMap<String,V>>" in compact(overlay)
         and 'changes:BTreeMap<String,Option<V>>' in compact(overlay),
         'baseline storage must be immutable and changes must retain tombstones')
    need('pub(super) baseline' not in overlay and 'DerefMut' not in overlay,
         'shared recipes cannot expose mutable storage')
    need('self.changes.insert(key.clone(),Some(value.clone()))' in compact(overlay)
         and 'self.baseline?.get_key_value(key)?' in compact(overlay),
         'writes must copy just their selected baseline entry')
    need('impl<V:PartialEq' in compact(overlay)
         and 'self.same_baseline(other)' in overlay
         and 'self.changes.keys().chain(other.changes.keys())' in compact(overlay),
         'fixpoint equality must compare effective deviations')
    need('left.same_baseline(right)' in flow and 'left.empty_changes()' in flow
         and 'left.changed_keys().chain(right.changed_keys())' in compact(flow)
         and 'merged.set_effective(name,None)' in compact(flow),
         'branch joins must preserve shared baselines and absence')
    need('propagated: FunctionSignatures' in signatures,
         'signature propagation cannot flatten overlays into an owned catalog')
    need('FnMut(KindSet,&FunctionTargetKnowledge,&mutHeapShape)->bool' in compact(lowering)
         and 'signature.has_live_shape_roots()' in lowering
         and 'self.function_signatures.values_mut()' not in lowering,
         'shape invalidation must own only typed present roots')
    need('#[path = "aot_native_program_templates/catalog_overlays.rs"]\nmod catalog_overlays;' in source('crates/lila-engine/tests/aot_native_program_templates.rs'),
         'exact existing integration target must own the catalog controls')
    for path, name in [
        (prefix + 'catalog_overlay/tests.rs', 'untouched_catalog_and_prepass_forks_never_clone_recipe_values'),
        (prefix + 'catalog_overlay/tests.rs', 'merged_iteration_preserves_order_overrides_and_deleted_baseline_entries'),
        (prefix + 'builtin_catalog/flow_tests.rs', 'global_branch_join_retains_shadowing_tombstones_and_intersection_presence'),
        (prefix + 'builtin_catalog/flow_tests.rs', 'present_shape_invalidation_owns_only_live_rows_and_preserves_fresh_returns'),
        ('crates/lila-engine/tests/aot_native_program_templates/catalog_overlays.rs', 'native_catalog_overlays_preserve_shadowing_alias_writes_and_branch_joins'),
        ('crates/lila-engine/tests/aot_native_program_templates/catalog_overlays.rs', 'native_catalog_changes_do_not_escape_into_a_later_unseen_source'),
    ]:
        need('fn ' + name + '()' in source(path), 'missing semantic control: ' + name)
    return errors


if __name__ == '__main__':
    failures = check(Path(sys.argv[1]))
    for failure in failures:
        print(failure, file=sys.stderr)
    raise SystemExit(bool(failures))
