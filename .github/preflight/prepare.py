from pathlib import Path


def replace_once(path, old, new):
    p = Path(path)
    text = p.read_text()
    if text.count(old) != 1:
        raise ValueError(f'{path}: expected exactly one replacement boundary: {old[:90]}')
    p.write_text(text.replace(old, new, 1))


def replace_region(path, start, end, replacement):
    p = Path(path)
    text = p.read_text()
    if text.count(start) != 1 or text.count(end) != 1:
        raise ValueError(f'{path}: ambiguous region boundaries')
    left = text.index(start)
    right = text.index(end, left)
    p.write_text(text[:left] + replacement + text[right:])


array = 'crates/lila-aot-wasm/src/builtins/array.rs'
replace_region(array,
    '    pub(crate) fn compile_array_prototype_flat_map_builtin(',
    '    pub(crate) fn emit_array_iteration_length_before_callback_validation(',
    Path('.github/preflight/flatmap-replacement.rs').read_text() + '\n')

owner = 'crates/lila-aot-wasm/tests/array_flat_map_algorithm_owner_structure.rs'
replace_once(owner, '        "    pub(crate) fn compile_array_prototype_map_builtin(",',
    '        "    fn emit_flat_map_append(",')
replace_once(owner, 'assert_eq!(canonical.matches("self.argc_param_local()").count(), 2);',
    'assert_eq!(canonical.matches("self.argc_param_local()").count(), 0);')
replace_region(owner, '    for (earlier, later) in [', '        assert_before(canonical, earlier, later);', '''    for (earlier, later) in [
        ("self.emit_array_iteration_length_before_callback_validation(", "self.emit_builtin_arg_to_locals(0,"),
        ("self.emit_builtin_arg_to_locals(0,", "self.emit_is_callable_i32("),
        ("self.emit_is_callable_i32(", "self.emit_builtin_arg_to_locals(1,"),
        ("self.emit_builtin_arg_to_locals(1,", "self.emit_array_species_create("),
        ("self.emit_array_species_create(", "self.emit_object_has_property_i32("),
        ("self.emit_object_has_property_i32(", "self.emit_object_read("),
        ("self.emit_object_read(", "self.emit_function_handle_call_with_argv("),
        ("self.emit_function_handle_call_with_argv(", "self.emit_is_array_i64("),
    ] {
''')
with Path(owner).open('a') as f:
    f.write('''
#[test]
fn one_append_owner_bounds_the_index_before_defining_and_incrementing() {
    let append = bounded(
        ARRAY_SOURCE,
        "    fn emit_flat_map_append(",
        "    pub(crate) fn emit_array_iteration_length_before_callback_validation(",
    );
    assert_eq!(append.matches("emit_array_target_create_data_property_or_throw(").count(), 1);
    assert_before(append, "Instruction::I64Const(MAX_SAFE_INTEGER as i64)", "emit_array_target_create_data_property_or_throw(");
    assert_before(append, "emit_array_target_create_data_property_or_throw(", "emit_return_current_completion_if_throw(");
    assert_before(append, "emit_return_current_completion_if_throw(", "Instruction::I64Add");
    assert!(!append.contains("emit_object_write("));
}
''')

witness = 'crates/lila-aot-wasm/tests/array_flat_map_typed_array_witness_structure.rs'
replace_once(witness, '        "pub(crate) fn compile_array_prototype_map_builtin(",',
    '        "fn emit_flat_map_append(",')
replace_region(witness,
    '#[test]\nfn flat_map_uses_one_view_for_its_snapshot_and_live_property_observation()',
    '#[test]\nfn focused_fixture_couples_each_buffer_transition_to_one_failure_bit()',
    '''#[test]
fn flat_map_uses_observable_length_and_shared_live_property_operations() {
    let body = flat_map_body();
    for (needle, expected) in [
        ("emit_array_iteration_length_before_callback_validation(", 1),
        ("emit_object_has_property_i32(", 2),
        ("emit_object_read(", 3),
        ("emit_is_array_i64(", 1),
        ("emit_array_species_create(", 1),
    ] {
        assert_eq!(body.matches(needle).count(), expected, "{needle}");
    }
    for forbidden in [
        "TypedArrayViewLocals", "TypedArrayWitnessUse",
        "emit_load_typed_array_private_state(",
        "emit_typed_array_current_byte_length(",
        "emit_validate_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(", "emit_load_array_buffer_data(",
        "HEAP_TYPED_ARRAY_", "HEAP_LEN_OFFSET", "HEAP_OBJECT_BOXED_",
        "Instruction::I64TruncF64U", "Instruction::I64DivU",
    ] {
        assert!(!body.contains(forbidden), "flatMap must delegate property semantics, not use {forbidden}");
    }
    assert_eq!(ARRAY_SOURCE.matches("emit_typed_array_current_byte_length(").count(), 0,
        "Array builtins must not reintroduce the legacy raw current-length observer");
}

#[test]
fn flat_map_keeps_the_snapshot_presence_read_and_mapper_order() {
    let body = flat_map_body();
    let snapshot = unique_position(body,
        "emit_array_iteration_length_before_callback_validation(", "ToObject/LengthOfArrayLike");
    let validate = unique_position(body, "emit_is_callable_i32(", "IsCallable");
    let target = unique_position(body, "emit_array_species_create(", "ArraySpeciesCreate");
    let first_loop = body.find("Instruction::Loop(BlockType::Empty)").expect("source loop");
    let presence = body.find("emit_object_has_property_i32(").expect("source HasProperty");
    let read = body.find("emit_object_read(").expect("source Get");
    let mapper = unique_position(body, "emit_function_handle_call_with_argv(", "mapper Call");
    let is_array = unique_position(body, "emit_is_array_i64(", "mapped IsArray");
    assert!(snapshot < validate && validate < target && target < first_loop &&
        first_loop < presence && presence < read && read < mapper && mapper < is_array);
    let normalized = without_whitespace(body);
    for receiver in ["this", "mapped"] {
        let snippet = format!(
            "self.emit_object_has_property_i32({receiver}_payload_local,{receiver}_tag_local,key_local,present_local,function,)?;self.emit_return_current_completion_if_throw(function);"
        );
        assert!(normalized.contains(&snippet), "{receiver} HasProperty must propagate abrupt completion");
    }
    let dispatcher = without_whitespace(STANDARD_SOURCE);
    unique_normalized_position(&dispatcher, r#"
        StandardBuiltinId::ArrayPrototypeFlatMap => {
            self.compile_array_prototype_flat_map_builtin(function)?;
        }
    "#, "Array.prototype.flatMap dispatcher edge");
}

''')

species = 'crates/lila-aot-wasm/tests/array_species_create_operation_evidence_structure.rs'
replace_once(species, 'fn array_species_create_emitter_has_only_slice_and_splice_callers()',
    'fn array_species_create_emitter_has_flat_map_slice_and_splice_callers()')
replace_once(species, 'ARRAY_SOURCE.matches("emit_array_species_create(").count(),\n        3',
    'ARRAY_SOURCE.matches("emit_array_species_create(").count(),\n        4')
replace_once(species, '    let slice = bounded(', '''    let flat_map = bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_prototype_flat_map_builtin(",
        "    fn emit_flat_map_append(",
    );
    assert_eq!(flat_map.matches("self.emit_array_species_create(").count(), 1);
    let slice = bounded(''')
replace_once(species, 'fn symbol_species_reads_remain_a_reviewed_nine_site_census()',
    'fn symbol_species_reads_remain_a_reviewed_eight_site_census()')
replace_once(species, 'assert_eq!(ARRAY_SOURCE.matches(SPECIES_READ).count(), 9);',
    'assert_eq!(ARRAY_SOURCE.matches(SPECIES_READ).count(), 8);')
replace_once(species, '''        (
            "    pub(crate) fn compile_array_prototype_flat_map_builtin(",
            "    pub(crate) fn emit_array_iteration_length_before_callback_validation(",
        ),
''', '')
replace_once(species, 'assert_eq!(live_array_copies.len(), 5);',
    'assert_eq!(live_array_copies.len(), 4);')

replace_once('.github/workflows/ci.yaml',
    '          cargo test -p lila-test262 --lib -- wasm_agents_run_test262_wait_until_with_exact_assertions\n', '''
      - name: Execute the pinned Test262 agent regression (non-vacuous)
        shell: bash
        run: |
          set -euo pipefail
          test_name=tests::wasm_agents_run_pinned_test262_case_with_exact_host_order
          cargo test --locked -p lila-test262 --lib "$test_name" -- --exact --list | tee /tmp/agent-inventory.txt
          grep -Fxq "$test_name: test" /tmp/agent-inventory.txt
          cargo test --locked -p lila-test262 --lib "$test_name" -- --exact --nocapture | tee /tmp/agent-result.txt
          grep -Eq '^test result: ok\\. 1 passed; 0 failed; 0 ignored;' /tmp/agent-result.txt
''')

replace_once('README.md', '## Current Status\n', '''Array `flatMap` now uses shared observable length, callability, species and
property operations. Its source length is captured before mapper validation and
species side effects; sparse properties and Proxy traps remain live during
mapping. TypedArray receivers use ordinary `length` access rather than a private
extent shortcut. The focused Wasm-AOT regressions, pinned subtree command and
remaining work are documented in
[the flatMap conformance follow-up](docs/rust-rewrite/aot-flat-map.md).
This does not change the published full-suite status or denominator.

## Current Status
''')

Path('docs/rust-rewrite/contracts/array-flat-map-algorithm-owner.md').write_text('''# `Array.prototype.flatMap` algorithm ownership

The Wasm-AOT implementation has one canonical Array algorithm,
`compile_array_prototype_flat_map_builtin`. The static Array entry continues to
use the shared argument-vector call boundary; the Iterator-helper branch is
unchanged. The removed direct Array wrapper must not return.

## Observable algorithm

All call-site argument expressions, including unused arguments and spreads,
finish before builtin execution. Inside the builtin, ToObject and one observable
LengthOfArrayLike precede IsCallable and ArraySpeciesCreate. Missing argument zero
is undefined, not a reason to skip observable length access. The length snapshot
survives species getters and construction without being refreshed.

Source indices below the snapshot use live HasProperty, then Get, then Call with
(value, index, boxed source). Callable Proxies and thisArg use the shared call
owner. A mapped value passes through the shared IsArray operation. Only Arrays
flatten, at depth one; their original Proxy receiver is retained for length,
presence and element access. Holes are skipped without mapper calls. Abrupt
completion stops later observable work.

A private append owner checks the maximum safe integer bound before shared
CreateDataPropertyOrThrow, propagates an abrupt definition, and increments only
on success. Custom species targets do not receive a synthetic length write.

The algorithm no longer reconstructs TypedArray private slots or implements its
own numeric length conversion, Proxy classification or species construction.
Those operations retain their existing semantic owners. The shared
ArraySpeciesCreate emitter now serves flatMap, slice and splice; its structural
census is updated without changing the operation catalog's evidence categories.

## Durable guards and execution evidence

The owner structure target pins one dispatcher/algorithm, complete argument
forwarding, operation order and append ordering. The TypedArray structure target
forbids private representation bypasses and retains the exact existing
resizable-buffer fixture matrix. The new `lila-engine` `aot_flat_map` target
executes observable programs through WasmAot, not an interpreter oracle.

See [the conformance follow-up](../aot-flat-map.md) for commands, evidence layers
and remaining work. Historical August 28 results described the direct-call
boundary only; they are not execution evidence for this replacement algorithm.
This change does not repair neighboring Array methods or the static branch's
broader property-lookup/classification policy.
''')

Path('docs/rust-rewrite/contracts/array-flat-map-typed-array-buffer-witness.md').write_text('''# Generic Array `flatMap` and TypedArray observation

## Ordinary length access, live integer-indexed properties

Array.prototype.flatMap is a generic Array method. It first obtains one
LengthOfArrayLike through the receiver's observable length property. A
TypedArray's own length property or an inherited override therefore participates
in Get and ToLength. The private element count is not a valid substitute.
Length access and coercion happen before mapper validation and species effects.

When the normal TypedArray length accessor is selected by property lookup, that
accessor owns backing-buffer validation and its detached/out-of-bounds policy.
An override can report a different length, resize or detach the buffer, or throw.
FlatMap captures the resulting ToLength value once and does not grow its loop
bound if a callback grows the buffer.

Each visited source index delegates to shared HasProperty and, when present, Get.
Those integer-indexed operation owners observe the current buffer state. Shrink,
out-of-bounds views or detachment can make subsequent indices absent without
changing the captured loop bound. The mapper receives the value read at that
iteration, not a previously captured element.

## Ownership change

The August 24 implementation constructed a TypedArrayViewLocals directly inside
flatMap for a private length projection and a live presence projection. It
explicitly left ordinary length shadowing and mapper/length ordering unresolved.
The new algorithm removes that specialization rather than adding a second length
policy. There are no private TypedArray loads, raw length conversions or direct
buffer witness projections in the flatMap owner. Generic property owners remain
responsible for their existing witness capabilities.

The structural target retains its filename but now enforces delegation and
Get/HasProperty/Call ordering. It rejects private-slot reconstruction and raw
length helpers. Its six-case fixture coupling remains intact: odd-byte tracking,
growth, shrink, fixed out-of-bounds, fixed regrowth and detached views. The
existing CLI fixture is not weakened or rewritten.

## Execution boundaries

The new engine target adds own and inherited length accessors, fractional length
coercion, resize during length access, resize during mapping, and detached views
with explicit length overrides. Those tests complement the retained CLI matrix;
source-structure guards alone do not prove buffer semantics.

See [the conformance follow-up](../aot-flat-map.md) for reproducible verification.
Other generic Array methods and TypedArray-specific methods are not changed by
this ownership migration. Historical fixture pass counts are not reused as
current-head evidence.
''')

with Path('tasks/16-arrays-and-array-builtins.md').open('a') as f:
    f.write('''

## 2026-09-06: Generic flatMap observable-operation closure

The canonical Wasm-AOT flatMap compiler now delegates ToObject/LengthOfArrayLike,
IsCallable, ArraySpeciesCreate, IsArray, HasProperty, Get and target property
creation to shared operation owners. Length is observable and captured before
mapper validation or species side effects. Missing callbacks no longer bypass
length getters, huge numeric lengths use bounded ToLength, and TypedArray private
extents no longer bypass own or inherited length properties.

The source and mapped-array loops retain live property observations, nested Proxy
traps, sparse behavior, one-level flattening and abrupt-completion order. A single
append owner guards the maximum safe integer bound before data-property creation.
The three affected structural targets track this ownership change; existing CLI
fixture bodies remain unchanged. Sixteen new engine regression programs select
WasmAot explicitly. CI includes a nonempty compiled-inventory check and the entire
pinned flatMap subtree, and repairs the previously stale pinned-agent test filter.

[The flatMap follow-up](../docs/rust-rewrite/aot-flat-map.md) gives the verification
commands and follow-on priorities. This is not closure of T16 or T26, not a change
to the Test262 denominator, and not a claim that the full current-pin suite is
green. Runtime results must be attached to the exact tested PR revision.
''')
