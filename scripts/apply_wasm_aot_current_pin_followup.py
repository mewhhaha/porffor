#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return (ROOT / path).read_text()


def write(path, text):
    p = ROOT / path
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text)


def replace_once(text, old, new, label):
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


def function_region(text, start_marker, end_marker):
    start = text.index(start_marker)
    end = text.index(end_marker, start)
    return start, end, text[start:end]


# 1. Extend only the generic Array callback owner with forEach.
path = "crates/lila-aot-wasm/src/builtins/array/callback_iteration.rs"
s = read(path)
s = replace_once(s, "//! The shared observable loop for Array map/filter/every/some.\n", "//! The shared observable loop for Array forEach/map/filter/every/some.\n", "callback doc")
s = replace_once(s, "    Map,\n    Filter,\n    Every,\n", "    ForEach,\n    Map,\n    Filter,\n    Every,\n", "callback enum")
s = replace_once(s, "        match self {\n            Self::Map =>", "        match self {\n            Self::ForEach => \"Array.prototype.forEach callback is not callable\",\n            Self::Map =>", "forEach callback error")
s = replace_once(s, "            ArrayCallbackIterationKind::Every | ArrayCallbackIterationKind::Some => {}\n", "            ArrayCallbackIterationKind::ForEach\n            | ArrayCallbackIterationKind::Every\n            | ArrayCallbackIterationKind::Some => {}\n", "forEach no species")
s = replace_once(s, "        match kind {\n            ArrayCallbackIterationKind::Map => {", "        match kind {\n            ArrayCallbackIterationKind::ForEach => {}\n            ArrayCallbackIterationKind::Map => {", "forEach callback result policy")
s = replace_once(s, "        match kind {\n            ArrayCallbackIterationKind::Map | ArrayCallbackIterationKind::Filter => {", "        match kind {\n            ArrayCallbackIterationKind::ForEach => {\n                function.instruction(&Instruction::I64Const(0));\n                function.instruction(&Instruction::LocalSet(self.result_local));\n                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));\n                function.instruction(&Instruction::LocalSet(self.result_tag_local));\n            }\n            ArrayCallbackIterationKind::Map | ArrayCallbackIterationKind::Filter => {", "forEach final result")
write(path, s)

# 2. Generic find* gets the same LengthOfArrayLike and direct indexed dispatch.
path = "crates/lila-aot-wasm/src/builtins/array/find_via_predicate.rs"
s = read(path)
start, end, fn = function_region(s, "    fn compile_array_find_with_kind(\n", "\n    }\n}")
init = "        self.emit_initialize_find_result(&projection, function);\n"
predicate = "        let predicate =\n            self.emit_validate_find_predicate(predicate_not_callable_message, function)?;\n"
a = fn.index(init) + len(init)
b = fn.index(predicate, a)
replacement = "        // Generic find methods observe LengthOfArrayLike before IsCallable.\n        self.emit_array_iteration_length_before_callback_validation(\n            receiver_payload_local,\n            receiver_tag_local,\n            key_local,\n            len_local,\n            element_tag_local,\n            function,\n        )?;\n        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));\n        function.instruction(&Instruction::LocalSet(number_tag_local));\n\n"
fn = fn[:a] + replacement + fn[b:]
loop_anchor = "        self.emit_initialize_find_index(&direction, len_local, index_local, function);\n"
loop_pos = fn.index(loop_anchor)
read_start_marker = "        function.instruction(&Instruction::LocalGet(receiver_tag_local));\n        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));\n"
read_start = fn.index(read_start_marker, loop_pos)
read_end = fn.index("        self.emit_propagate_throw_from_locals_if_needed(\n            element_payload_local,", read_start)
read_replacement = "        self.emit_typed_array_or_object_index_read_from_locals(\n            receiver_payload_local,\n            receiver_tag_local,\n            index_local,\n            element_payload_local,\n            element_tag_local,\n            function,\n        )?;\n"
fn = fn[:read_start] + read_replacement + fn[read_end:]
# Private typed-array snapshot locals are no longer generic find state.
for old in [
    "        let typed_receiver_local = self.reserve_temp_local();\n",
    "        let typed_buffer_payload_local = self.reserve_temp_local();\n",
    "        let typed_byte_offset_local = self.reserve_temp_local();\n",
    "        let typed_stored_byte_length_local = self.reserve_temp_local();\n",
    "        let typed_bytes_per_element_local = self.reserve_temp_local();\n",
    "        self.release_temp_local(typed_bytes_per_element_local);\n",
    "        self.release_temp_local(typed_stored_byte_length_local);\n",
    "        self.release_temp_local(typed_byte_offset_local);\n",
    "        self.release_temp_local(typed_buffer_payload_local);\n",
    "        self.release_temp_local(typed_receiver_local);\n",
]:
    if old not in fn:
        raise SystemExit(f"generic find cleanup missing: {old.strip()}")
    fn = fn.replace(old, "", 1)
view_start = fn.index("        let typed_view = TypedArrayViewLocals::new(\n")
view_end = fn.index("        );\n", view_start) + len("        );\n")
fn = fn[:view_start] + fn[view_end:]
s = s[:start] + fn + s[end:]
write(path, s)

# 3. Array monolith: generic forEach/reduce, species copies, concat LengthOfArrayLike.
path = "crates/lila-aot-wasm/src/builtins/array.rs"
s = read(path)
s = replace_once(
    s,
    "    pub(super) fn compile_array_prototype_for_each_builtin(\n        &mut self,\n        function: &mut Function,\n    ) -> Result<(), EmitError> {\n        self.compile_array_like_for_each_builtin(function, ArrayCallbackReceiverKind::ArrayLike)\n    }\n",
    "    pub(super) fn compile_array_prototype_for_each_builtin(\n        &mut self,\n        function: &mut Function,\n    ) -> Result<(), EmitError> {\n        self.compile_array_callback_iteration(function, ArrayCallbackIterationKind::ForEach)\n    }\n",
    "generic forEach owner",
)

# Generic reduce: always observable Get(length)+ToLength, after ToObject and before IsCallable.
rstart, rend, rfn = function_region(s, "    fn compile_array_like_reduce_builtin(\n", "\n    pub(super) fn compile_array_reduce_builtin(")
private_len = "                function.instruction(&Instruction::LocalGet(receiver_tag_local));\n                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));\n"
a = rfn.index(private_len)
branch_end_marker = "                function.instruction(&Instruction::End);\n                function.instruction(&Instruction::End);\n            }\n            ArrayCallbackReceiverKind::TypedArray => {"
b = rfn.index(branch_end_marker, a)
length_replacement = "                function.instruction(&Instruction::I64Const(self.strings.payload(\"length\")));\n                function.instruction(&Instruction::LocalSet(key_local));\n                self.emit_object_read(\n                    receiver_payload_local,\n                    receiver_tag_local,\n                    receiver_payload_local,\n                    receiver_tag_local,\n                    key_local,\n                    element_payload_local,\n                    element_tag_local,\n                    function,\n                )?;\n                self.emit_propagate_throw_from_locals_if_needed(\n                    element_payload_local,\n                    element_tag_local,\n                    function,\n                )?;\n                self.emit_to_length_i64_from_value_locals(\n                    element_tag_local,\n                    element_payload_local,\n                    len_local,\n                    function,\n                )?;\n"
rfn = rfn[:a] + length_replacement + rfn[b:]
s = s[:rstart] + rfn + s[rend:]

# Reduce HasProperty/Get use the same exhaustive generic dispatch as the shared callback owner.
hstart, hend, _ = function_region(s, "    fn emit_array_reduce_has_property(\n", "\n    fn emit_array_reduce_get_index(")
hnew = '''    fn emit_array_reduce_has_property(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        _typed_receiver_local: u32,
        _typed_view: &TypedArrayViewLocals,
        _index_local: u32,
        key_local: u32,
        has_property_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_has_property_i32(
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            has_property_local,
            function,
        )
    }
'''
s = s[:hstart] + hnew + s[hend:]
gstart, gend, _ = function_region(s, "    fn emit_array_reduce_get_index(\n", "\n    pub(super) fn compile_array_prototype_for_each_builtin(")
gnew = '''    fn emit_array_reduce_get_index(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        _typed_receiver_local: u32,
        index_local: u32,
        _key_local: u32,
        element_payload_local: u32,
        element_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_typed_array_or_object_index_read_from_locals(
            receiver_payload_local,
            receiver_tag_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        )
    }
'''
s = s[:gstart] + gnew + s[gend:]

# Shared ArraySpeciesCreate is exactly the native Array owner used by map/filter/flatMap.
def replace_species_copy(text, fn_marker, next_marker, start_marker, end_marker, replacement):
    fs, fe, body = function_region(text, fn_marker, next_marker)
    a = body.index(start_marker)
    b = body.index(end_marker, a)
    body = body[:a] + replacement + body[b:]
    # Remove species-only locals and metadata now owned by emit_array_species_create.
    for name in ["constructor_payload_local", "constructor_tag_local", "constructor_table_index_local", "skip_species_local", "species_payload_local", "species_tag_local", "argc_local", "argv_local"]:
        body = body.replace(f"        let {name} = self.reserve_temp_local();\n", "", 1)
        body = body.replace(f"        self.release_temp_local({name});\n", "", 1)
    meta = "        let array_constructor_table_index = self\n            .functions\n            .get(&StandardBuiltinId::ArrayConstructor.function_id())\n            .map(|meta| meta.table_index as i64)\n            .ok_or_else(|| {\n                EmitError::unsupported(\n                    \"unsupported in lila wasm-aot first slice: missing builtin meta `Array`\",\n                )\n            })?;\n"
    if meta not in body:
        raise SystemExit(f"{fn_marker}: species metadata block missing")
    body = body.replace(meta, "", 1)
    return text[:fs] + body + text[fe:]

flat_start = "        function.instruction(&Instruction::LocalGet(this_tag_local));\n        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));\n        function.instruction(&Instruction::I64Eq);\n        function.instruction(&Instruction::If(BlockType::Empty));\n        self.emit_array_constructor_read(\n"
flat_end = "        self.emit_alloc_array_payload_with_length(zero_local, stack_values_local, function)?;\n"
flat_repl = "        self.emit_array_species_create(\n            this_payload_local,\n            this_tag_local,\n            zero_local,\n            target_payload_local,\n            target_tag_local,\n            function,\n        )?;\n"
s = replace_species_copy(s, "    pub(crate) fn compile_array_prototype_flat_builtin(\n", "\n    pub(crate) fn emit_flat_append_depth_one_value(", flat_start, flat_end, flat_repl)

concat_start = "        self.emit_is_array_i64(\n            this_payload_local,\n            this_tag_local,\n            spreadable_flag_local,\n            function,\n        )?;\n"
concat_end = "        function.instruction(&Instruction::I64Const(0));\n        function.instruction(&Instruction::LocalSet(item_index_local));\n"
concat_repl = "        self.emit_array_species_create(\n            this_payload_local,\n            this_tag_local,\n            zero_local,\n            target_payload_local,\n            target_tag_local,\n            function,\n        )?;\n\n"
s = replace_species_copy(s, "    pub(crate) fn compile_array_prototype_concat_builtin(\n", "\n    pub(crate) fn compile_array_prototype_flat_map_builtin(", concat_start, concat_end, concat_repl)

# Concat spreadable inputs use generic observable LengthOfArrayLike, including arguments overrides.
cs, ce, _ = function_region(s, "    pub(crate) fn emit_concat_length_of_array_like(\n", "\n    pub(crate) fn emit_concat_typed_array_has_index_i32(")
concat_len = '''    pub(crate) fn emit_concat_length_of_array_like(
        &mut self,
        item_payload_local: u32,
        item_tag_local: u32,
        length_local: u32,
        object_length_payload_local: u32,
        object_length_tag_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            item_payload_local,
            item_tag_local,
            item_payload_local,
            item_tag_local,
            key_local,
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            object_length_payload_local,
            object_length_tag_local,
            function,
        )?;
        self.emit_to_length_i64_from_value_locals(
            object_length_tag_local,
            object_length_payload_local,
            length_local,
            function,
        )
    }
'''
s = s[:cs] + concat_len + s[ce:]
write(path, s)

# 4. Direct-dispatch/materializer planning: species-created flat/concat targets may be ordinary objects.
path = "crates/lila-aot-wasm/src/planning.rs"
s = read(path)
s = replace_once(
    s,
    "            StandardBuiltinId::ArrayPrototypeFlatMap\n            | StandardBuiltinId::ArrayPrototypeMap\n",
    "            StandardBuiltinId::ArrayPrototypeFlat\n            | StandardBuiltinId::ArrayPrototypeConcat\n            | StandardBuiltinId::ArrayPrototypeFlatMap\n            | StandardBuiltinId::ArrayPrototypeMap\n",
    "Array result materializer roots",
)
write(path, s)

# 5. Exact executable regressions.
path = "crates/lila-engine/tests/aot_array_callback_iteration.rs"
s = read(path)
append = r'''

#[test]
fn generic_for_each_observes_borrowed_typed_array_length() {
    assert_wasm_true(r#"
        const value = new Uint8Array([4, 5]);
        Object.defineProperty(value, "length", { value: 1 });
        let calls = 0;
        Array.prototype.forEach.call(value, () => { calls += 1; });
        calls === 1
    "#);
}

#[test]
fn generic_for_each_observes_inherited_sparse_indexes() {
    assert_wasm_true(r#"
        Array.prototype[0] = 17;
        const value = [, 2];
        const seen = [];
        Array.prototype.forEach.call(value, x => seen.push(x));
        delete Array.prototype[0];
        seen.length === 2 && seen[0] === 17 && seen[1] === 2
    "#);
}

#[test]
fn generic_find_observes_borrowed_typed_array_length_and_holes() {
    assert_wasm_true(r#"
        const value = new Uint8Array([7, 9]);
        Object.defineProperty(value, "length", { value: 1 });
        let typedCalls = 0;
        Array.prototype.find.call(value, () => { typedCalls += 1; return false; });
        const holes = [];
        Array.prototype.find.call([, 1], (v, i) => { holes.push(i + ":" + v); return false; });
        typedCalls === 1 && holes.length === 2 && holes[0] === "0:undefined"
    "#);
}

#[test]
fn generic_reduce_observes_length_and_inherited_indexes() {
    assert_wasm_true(r#"
        const value = new Uint8Array([7, 9]);
        Object.defineProperty(value, "length", { value: 1 });
        const typed = Array.prototype.reduce.call(value, (a, v) => a + v, 0);
        Array.prototype[0] = 5;
        const sparse = Array.prototype.reduce.call([, 2], (a, v) => a + v, 0);
        delete Array.prototype[0];
        typed === 7 && sparse === 7
    "#);
}

#[test]
fn generic_callback_length_is_observed_before_callback_type_error() {
    assert_wasm_true(r#"
        let log = "";
        const value = { get length() { log += "L"; return 0; } };
        try { Array.prototype.forEach.call(value, null); } catch (e) { log += e instanceof TypeError ? "F" : "f"; }
        try { Array.prototype.find.call(value, null); } catch (e) { log += e instanceof TypeError ? "D" : "d"; }
        try { Array.prototype.reduce.call(value, null); } catch (e) { log += e instanceof TypeError ? "R" : "r"; }
        log === "LFLDLR"
    "#);
}

#[test]
fn flat_and_concat_accept_constructable_proxy_species() {
    assert_wasm_true(r#"
        function Species() {}
        const species = new Proxy(Species, { construct() { return []; } });
        const ctor = {};
        Object.defineProperty(ctor, Symbol.species, { value: species });
        const flatInput = [[1]];
        flatInput.constructor = ctor;
        const flatResult = flatInput.flat();
        const concatInput = [1];
        concatInput.constructor = ctor;
        const concatResult = concatInput.concat([2]);
        Array.isArray(flatResult) && flatResult[0] === 1 &&
        Array.isArray(concatResult) && concatResult.length === 2 && concatResult[1] === 2
    "#);
}

#[test]
fn concat_spreadable_arguments_observes_overridden_length() {
    assert_wasm_true(r#"
        (function () {
            arguments[Symbol.isConcatSpreadable] = true;
            Object.defineProperty(arguments, "length", { value: 1 });
            const result = [].concat(arguments);
            return result.length === 1 && result[0] === 3;
        })(3, 4)
    "#);
}
'''
if "generic_for_each_observes_borrowed_typed_array_length" in s:
    raise SystemExit("regressions already present")
s += append
write(path, s)

# 6. Source boundary test: strict TypedArray slice/map/filter remain independently branded/species-owned.
write("crates/lila-aot-wasm/tests/array_owner_followup_structure.rs", r'''fn region<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).unwrap_or_else(|| panic!("missing start marker: {start}"));
    let rest = &source[start..];
    let end = rest.find(end).unwrap_or_else(|| panic!("missing end marker: {end}"));
    &rest[..end]
}

#[test]
fn generic_for_each_uses_shared_callback_owner_but_strict_typedarray_does_not() {
    let source = include_str!("../src/builtins/array.rs");
    let generic = region(source, "fn compile_array_prototype_for_each_builtin", "fn compile_typed_array_prototype_for_each_builtin");
    assert!(generic.contains("compile_array_callback_iteration"));
    assert!(generic.contains("ArrayCallbackIterationKind::ForEach"));
    let strict = region(source, "fn compile_typed_array_prototype_for_each_builtin", "fn compile_array_like_for_each_builtin");
    assert!(strict.contains("ArrayCallbackReceiverKind::TypedArray"));
}

#[test]
fn strict_typedarray_slice_map_filter_keep_validated_method_entry_contracts() {
    let source = include_str!("../src/builtins/array.rs");
    for (start, end) in [
        ("fn compile_typed_array_prototype_slice_builtin", "fn compile_typed_array_prototype_map_builtin"),
        ("fn compile_typed_array_prototype_map_builtin", "fn compile_typed_array_prototype_filter_builtin"),
        ("fn compile_typed_array_prototype_filter_builtin", "fn compile_array_prototype_filter_builtin"),
    ] {
        let body = region(source, start, end);
        assert!(body.contains("TypedArrayWitnessUse::ValidatedMethodEntry"), "{start} lost strict entry validation");
    }
}

#[test]
fn flat_and_concat_delegate_native_array_species_creation() {
    let source = include_str!("../src/builtins/array.rs");
    let flat = region(source, "fn compile_array_prototype_flat_builtin", "fn emit_flat_append_depth_one_value");
    let concat = region(source, "fn compile_array_prototype_concat_builtin", "fn compile_array_prototype_flat_map_builtin");
    for body in [flat, concat] {
        assert!(body.contains("emit_array_species_create"));
        assert!(!body.contains("Symbol.species"));
        assert!(!body.contains("emit_mark_skip_species_for_cross_realm_array_constructor"));
    }
}

#[test]
fn direct_dispatch_materializes_species_result_property_support_for_flat_and_concat() {
    let planning = include_str!("../src/planning.rs");
    let roots = region(planning, "StandardBuiltinId::ArrayPrototypeFlat", "=> {\n                self.builtin_roots.insert(BuiltinRoot::ObjectDefineProperty);");
    assert!(roots.contains("StandardBuiltinId::ArrayPrototypeConcat"));
    assert!(roots.contains("StandardBuiltinId::ArrayPrototypeFlatMap"));
}
''')

# 7. Separate current-pin inventory workflow. It never invokes publish-status and never edits the canonical aggregate.
write(".github/workflows/wasm-aot-current-pin-inventory.yaml", r'''name: Wasm-AOT current-pin failure inventory

on:
  workflow_dispatch:

permissions:
  contents: read

jobs:
  inventory:
    runs-on: ubuntu-latest
    timeout-minutes: 360
    env:
      CARGO_BUILD_JOBS: 2
      LILA_MODULE_MEMORY_CACHE_ENTRIES: 2
      SNAPSHOT: current-pin-wasm-aot-${{ github.sha }}
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
      - name: Record exact pin and policy boundary
        shell: bash
        run: |
          set -euo pipefail
          git rev-parse HEAD | tee /tmp/current-pin-sha.txt
          git submodule status 2>/dev/null | tee /tmp/current-pin-submodules.txt || true
          grep -nE 'eval|new Function|cross-realm Function|dynamic source' tasks/26-zero-failure-conformance-closure.md > /tmp/dynamic-source-policy.txt
      - name: Build the Wasm-AOT Test262 product
        run: cargo build --locked -p lila-cli
      - name: Run the complete resumable current-pin matrix
        shell: bash
        run: |
          set -o pipefail
          ./target/debug/lila test262 report-all --execution-backend wasm-aot --snapshot-name "$SNAPSHOT" --resume 2>&1 | tee /tmp/report-all.txt
          report_rc=${PIPESTATUS[0]}
          ./target/debug/lila test262 progress-status --execution-backend wasm-aot --snapshot-name "$SNAPSHOT" 2>&1 | tee /tmp/progress-status.txt
          progress_rc=${PIPESTATUS[0]}
          ./target/debug/lila test262 triage-status --execution-backend wasm-aot --snapshot-name "$SNAPSHOT" 2>&1 | tee /tmp/triage-status.txt
          triage_rc=${PIPESTATUS[0]}
          printf 'report-all=%s\nprogress-status=%s\ntriage-status=%s\n' "$report_rc" "$progress_rc" "$triage_rc" > /tmp/current-pin-command-status.txt
          # The inventory is evidence even while failures remain; command failures are retained, not converted to passes.
          test -s /tmp/report-all.txt
          test -s /tmp/progress-status.txt
          test -s /tmp/triage-status.txt
      - name: Render semantic-owner and dynamic-source views
        shell: bash
        run: |
          set -euo pipefail
          {
            echo '# Current-pin Wasm-AOT failure inventory'
            echo
            echo "Pin: $(cat /tmp/current-pin-sha.txt)"
            echo
            echo '## Shared semantic-owner triage'
            echo
            echo '```text'
            cat /tmp/triage-status.txt
            echo '```'
            echo
            echo '## Explicit dynamic-source unsupported policy'
            echo
            echo 'These cases remain non-passing and are intentionally separated from semantic engine failures.'
            echo
            echo '```text'
            cat /tmp/dynamic-source-policy.txt
            echo '```'
            echo
            echo 'The canonical published aggregate is not rewritten by this workflow.'
          } > /tmp/current-pin-inventory.md
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: wasm-aot-current-pin-failure-inventory
          path: |
            /tmp/current-pin-inventory.md
            /tmp/current-pin-sha.txt
            /tmp/current-pin-submodules.txt
            /tmp/current-pin-command-status.txt
            /tmp/report-all.txt
            /tmp/progress-status.txt
            /tmp/triage-status.txt
            /tmp/dynamic-source-policy.txt
            test262/snapshots/current-pin-wasm-aot-*
          retention-days: 14
''')

# 8. Real-subtree base/head evidence, focused and non-publishing.
write(".github/workflows/array-owner-delta.yaml", r'''name: Array owner real-subtree delta

on:
  pull_request:
    paths:
      - 'crates/lila-aot-wasm/src/builtins/array.rs'
      - 'crates/lila-aot-wasm/src/builtins/array/**'
      - 'crates/lila-aot-wasm/src/planning.rs'
      - 'crates/lila-engine/tests/aot_array_callback_iteration.rs'
      - '.github/workflows/array-owner-delta.yaml'

permissions:
  contents: read

jobs:
  regressions:
    runs-on: ubuntu-latest
    timeout-minutes: 50
    env:
      CARGO_BUILD_JOBS: 2
      LILA_MODULE_MEMORY_CACHE_ENTRIES: 2
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
      - run: cargo fmt --all -- --check
      - run: cargo test --locked -p lila-aot-wasm --test array_owner_followup_structure
      - run: python3 scripts/run_engine_regression_inventory.py aot_array_callback_iteration --output-dir /tmp/array-owner-engine
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: array-owner-exact-programs
          path: /tmp/array-owner-engine
          retention-days: 14

  build-products:
    runs-on: ubuntu-latest
    timeout-minutes: 45
    env:
      CARGO_BUILD_JOBS: 2
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
          persist-credentials: false
      - name: Build exact base and head products
        shell: bash
        env:
          BASE_SHA: ${{ github.event.pull_request.base.sha }}
          HEAD_SHA: ${{ github.event.pull_request.head.sha }}
        run: |
          set -euo pipefail
          test "$(git rev-parse HEAD)" = "$HEAD_SHA"
          git worktree add /tmp/porffor-base "$BASE_SHA"
          CARGO_TARGET_DIR=/tmp/base-target cargo build --locked --manifest-path /tmp/porffor-base/Cargo.toml -p lila-cli
          cargo build --locked -p lila-cli
          mkdir -p /tmp/array-owner-products
          cp /tmp/base-target/debug/lila /tmp/array-owner-products/lila-base
          cp target/debug/lila /tmp/array-owner-products/lila-head
          printf '%s\n' "$BASE_SHA" > /tmp/array-owner-products/base.sha
          printf '%s\n' "$HEAD_SHA" > /tmp/array-owner-products/head.sha
      - uses: actions/upload-artifact@v4
        with:
          name: array-owner-products
          path: /tmp/array-owner-products
          retention-days: 14

  real-subtree-delta:
    needs: build-products
    runs-on: ubuntu-latest
    timeout-minutes: 90
    strategy:
      fail-fast: false
      matrix:
        include:
          - owner: callback-iteration
            path: built-ins/Array/prototype/forEach/
          - owner: find-via-predicate
            path: built-ins/Array/prototype/find/
          - owner: reduce
            path: built-ins/Array/prototype/reduce/
          - owner: array-species-flat
            path: built-ins/Array/prototype/flat/
          - owner: array-species-concat
            path: built-ins/Array/prototype/concat/
          - owner: strict-typedarray-slice
            path: built-ins/TypedArray/prototype/slice/
          - owner: strict-typedarray-map
            path: built-ins/TypedArray/prototype/map/
          - owner: strict-typedarray-filter
            path: built-ins/TypedArray/prototype/filter/
    env:
      LILA_MODULE_MEMORY_CACHE_ENTRIES: 2
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
      - uses: actions/download-artifact@v4
        with:
          name: array-owner-products
          path: /tmp/array-owner-products
      - name: Execute exact real subtree on base and head
        shell: bash
        env:
          OWNER: ${{ matrix.owner }}
          TEST_PATH: ${{ matrix.path }}
        run: |
          set -uo pipefail
          chmod +x /tmp/array-owner-products/lila-base /tmp/array-owner-products/lila-head
          mkdir -p "/tmp/$OWNER/base" "/tmp/$OWNER/head"
          /tmp/array-owner-products/lila-base test262 list "$TEST_PATH" | tee "/tmp/$OWNER/base-inventory.txt"
          /tmp/array-owner-products/lila-head test262 list "$TEST_PATH" | tee "/tmp/$OWNER/head-inventory.txt"
          diff -u "/tmp/$OWNER/base-inventory.txt" "/tmp/$OWNER/head-inventory.txt" | tee "/tmp/$OWNER/inventory.diff"
          test ${PIPESTATUS[0]} -eq 0
          /tmp/array-owner-products/lila-base test262 shard 1/1 "$TEST_PATH" --execution-backend wasm --threads 2 --jobs 2 --timeout-ms 60000 --snapshot-dir "/tmp/$OWNER/base" --snapshot-name "$OWNER-base" 2>&1 | tee "/tmp/$OWNER/base.txt"
          base_rc=${PIPESTATUS[0]}
          /tmp/array-owner-products/lila-head test262 shard 1/1 "$TEST_PATH" --execution-backend wasm --threads 2 --jobs 2 --timeout-ms 60000 --snapshot-dir "/tmp/$OWNER/head" --snapshot-name "$OWNER-head" 2>&1 | tee "/tmp/$OWNER/head.txt"
          head_rc=${PIPESTATUS[0]}
          printf 'base=%s\nhead=%s\n' "$base_rc" "$head_rc" | tee "/tmp/$OWNER/command-status.txt"
          grep -Eq '^total: [1-9][0-9]*$' "/tmp/$OWNER/base.txt"
          grep -Eq '^total: [1-9][0-9]*$' "/tmp/$OWNER/head.txt"
          # Semantic failures are evidence, not workflow-control flow. Exact logs and snapshots are retained.
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: ${{ matrix.owner }}-real-subtree-before-after
          path: /tmp/${{ matrix.owner }}
          retention-days: 14
''')

print("staged Wasm-AOT current-pin follow-up")
