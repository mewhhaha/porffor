#!/usr/bin/env bash
set -euo pipefail

failures=0

fail() {
  printf 'check-module-boundaries: %s\n' "$*" >&2
  failures=$((failures + 1))
}

require_file() {
  if [ ! -f "$1" ]; then
    fail "missing file: $1"
    return 1
  fi
}

require_module_decl() {
  file="$1"
  module="$2"
  if ! grep -Eq "^(pub\\(crate\\) |pub )?mod ${module};$" "$file"; then
    fail "$file must declare module: $module"
  fi
}

require_pub_use() {
  file="$1"
  pattern="$2"
  description="$3"
  if ! grep -Eq "$pattern" "$file"; then
    fail "$file must re-export $description"
  fi
}

require_fixed_string_count() {
  file="$1"
  needle="$2"
  expected="$3"
  description="$4"
  count="$(grep -Fc "$needle" "$file" || true)"
  if [ "$count" -ne "$expected" ]; then
    fail "$file must contain $expected $description sites (found $count)"
  fi
}

# Non-test CODE lines: everything before the crate's `#[cfg(test)]` block, minus
# blank lines and minus whole-line comments (`//`, `///`, `//!` and lines inside
# a whole-line `/* ... */` block).
#
# Blanks and comments are excluded because of what this budget is FOR: it exists
# so implementation cannot creep back into a crate root that is supposed to hold
# nothing but `mod`, `use` and `pub use`. Counting documentation against that
# budget makes the guard punish the one thing a re-export surface most needs.
# Measured at batch 6: `lila-ir/src/lib.rs` was 169 raw lines and RED against
# a budget of 140, while its code was 140 lines exactly — every line over the
# limit was a doc comment pointing a re-exported contract type at its
# `docs/rust-rewrite/contracts/` file, added by the theory rounds. Raising the
# number instead would have ratcheted the budget for a file that had not grown.
#
# THIS IS A LOOSENING OF EVERY `check_orchestration_surface` BUDGET, not only of
# the one that motivated it. Each budget below is now read against a code-only
# count; a number chosen against the old raw count is therefore no longer the
# limit it was written to be, and each is annotated at its call site with what it
# measures today.
#
# The block-comment rule is a state machine rather than the `^[[:space:]]*\*`
# heuristic it replaces. That heuristic dropped any line whose first non-space
# character is `*` — a `*slot = value;` deref statement, a continued expression —
# so the count could silently UNDER-report real code for any file this script
# guards, in the one direction that turns a red budget green without anyone
# editing the budget. Only whole-line block comments are skipped; a `/* ... */`
# that opens after code on the same line still counts that line, which is the
# conservative direction.
non_test_lines() {
  awk '
    /^#\[cfg\(test\)\]/ { exit }
    in_block { if ($0 ~ /\*\//) { in_block = 0 } ; next }
    /^[[:space:]]*$/ { next }
    /^[[:space:]]*\/\// { next }
    /^[[:space:]]*\/\*/ { if ($0 !~ /\*\//) { in_block = 1 } ; next }
    { count += 1 }
    END { print count + 0 }
  ' "$1"
}

check_orchestration_surface() {
  file="$1"
  max_lines="$2"
  lines="$(non_test_lines "$file")"
  if [ "$lines" -gt "$max_lines" ]; then
    fail "$file has $lines non-test code lines; expected at most $max_lines"
  fi
}

check_no_inline_legacy_includes() {
  file="$1"
  if grep -Eq 'include!|#\[path' "$file"; then
    fail "$file must not reassemble legacy implementation through include!/#[path]"
  fi
}

check_raw_line_budget() {
  file="$1"
  max_lines="$2"
  lines="$(wc -l < "$file")"
  if [ "$lines" -gt "$max_lines" ]; then
    fail "$file has $lines raw lines; expected at most $max_lines"
  fi
}

ir_lib="crates/lila-ir/src/lib.rs"
ir_builtins="crates/lila-ir/src/builtins.rs"
ir_lowering="crates/lila-ir/src/lowering.rs"
wasm_lib="crates/lila-aot-wasm/src/lib.rs"
wasm_builtins_mod="crates/lila-aot-wasm/src/builtins/mod.rs"
wasm_standard_builtins="crates/lila-aot-wasm/src/builtins/standard.rs"
wasm_intrinsics_mod="crates/lila-aot-wasm/src/intrinsics/mod.rs"

require_file "$ir_lib"
require_file "$wasm_lib"
require_file "$wasm_builtins_mod"

for module in analysis builtins diagnostics early_errors ir lowering lowering_helpers names operations; do
  require_file "crates/lila-ir/src/${module}.rs"
  require_module_decl "$ir_lib" "$module"
done

require_pub_use "$ir_lib" '^pub use ir::\*;' 'IR data types'
require_pub_use "$ir_lib" '^pub use lowering::\{?lower' 'the lowering entry point'
require_pub_use "$ir_lib" '^pub use operations::' 'shared operation enums'
# T12's module subsystem. `modules/` is a directory module, so the flat-file
# loop above cannot cover it: declaring `mod modules;` without the directory,
# or adding a submodule without registering it, is exactly the failure this
# catches.
ir_modules_mod="crates/lila-ir/src/modules/mod.rs"
require_file "$ir_modules_mod"
require_module_decl "$ir_lib" "modules"
for module in dynamic early graph link namespace record source; do
  require_file "crates/lila-ir/src/modules/${module}.rs"
  require_module_decl "$ir_modules_mod" "$module"
done
check_no_inline_legacy_includes "$ir_modules_mod"
require_pub_use "$ir_lib" '^pub use modules::\{' 'the module-record surface'

# 160 against a CODE-ONLY count, measured 140 at batch 6.
#
# 140 was the budget for the RAW line count, and after `non_test_lines` started
# excluding blanks and comments this file sat at exactly 140 of 140 — zero
# headroom, so the next `mod`/`use`/`pub use` line any lane adds to this crate
# root reddens a SHARED script for a reason unrelated to that lane. 160 is 20
# lines of headroom over the measurement and still far below the 169 raw lines
# the old number rejected. Re-tighten it the next time this crate root actually
# shrinks; do not raise it again without saying here what it was measured at.
check_orchestration_surface "$ir_lib" 160
check_no_inline_legacy_includes "$ir_lib"

# T02's pure builtin-shape boundary. Keeping these 98 metadata constructors in
# a child module leaves lowering.rs responsible for orchestration and semantic
# lowering rather than making it the mandatory edit point for every builtin.
ir_builtin_shapes="crates/lila-ir/src/lowering/builtin_shapes.rs"
require_file "$ir_builtin_shapes"
require_module_decl "$ir_lowering" "builtin_shapes"
# T15's two array-literal lowerers share one typed ArrayAccumulation seam. Keep
# the ordinary and staged-generator walkers together in their child module so
# the 32k-line orchestration boundary does not become the edit point again.
ir_array_literal_lowering="crates/lila-ir/src/lowering/array_literal.rs"
require_file "$ir_array_literal_lowering"
require_module_decl "$ir_lowering" "array_literal"
require_fixed_string_count "$ir_array_literal_lowering" 'fn lower_array_literal(' 1 'ordinary array-literal lowerer'
require_fixed_string_count "$ir_array_literal_lowering" 'fn lower_staged_generator_array_literal(' 1 'staged array-literal lowerer'
require_fixed_string_count "$ir_lowering" 'fn lower_array_literal(' 0 'array-literal lowerer outside child module'
require_fixed_string_count "$ir_lowering" 'fn lower_staged_generator_array_literal(' 0 'staged array-literal lowerer outside child module'
check_no_inline_legacy_includes "$ir_lowering"
# Measured immediately after extraction: 31,979 raw lines. This deliberately
# leaves only 21 lines of headroom; new builtin shape metadata belongs in the
# child, and further lowering families should be extracted rather than growing
# the remaining store again.
check_raw_line_budget "$ir_lowering" 32000

# T02's StandardBuiltinId registry. One macro row owns declaration order,
# function-index order, global installation order and every metadata field.
# Keeping the invocation in a real child module preserves an ownership seam;
# `include!` would merely hide the same monolith from line counts.
ir_builtin_catalog="crates/lila-ir/src/builtins/catalog.rs"
require_file "$ir_builtin_catalog"
require_module_decl "$ir_builtins" "catalog"
require_pub_use "$ir_builtins" '^pub use catalog::StandardBuiltinId;' 'the standard builtin ID'
require_pub_use "$ir_builtins" '^pub use catalog::StandardBuiltinInstaller;' 'the standard builtin installer class'
check_no_inline_legacy_includes "$ir_builtins"
if ! grep -q '^macro_rules! standard_builtin_catalog' "$ir_builtins"; then
  fail "$ir_builtins must generate StandardBuiltinId from standard_builtin_catalog"
fi
if ! grep -q '^standard_builtin_catalog!' "$ir_builtin_catalog"; then
  fail "$ir_builtin_catalog must be the single standard builtin catalog invocation"
fi
if ! grep -q 'function: FunctionOrdinal(' "$ir_builtin_catalog" \
  || ! grep -q 'global: GlobalOrdinal(' "$ir_builtin_catalog" \
  || ! grep -q 'installer: None' "$ir_builtin_catalog"; then
  fail "$ir_builtin_catalog must encode dense function/global ordinals and mandatory installer classes"
fi
# T24's host-builtin surface registry. Identity, callable/global name, function
# id, exposure class and realm scope come from one row source; the machinery
# stays in builtins.rs while the rows live in a real child module.
ir_host_builtin_catalog="crates/lila-ir/src/builtins/host_catalog.rs"
require_file "$ir_host_builtin_catalog"
require_module_decl "$ir_builtins" "host_catalog"
require_pub_use "$ir_builtins" '^pub use host_catalog::HostBuiltinId;' 'the host builtin ID'
check_no_inline_legacy_includes "$ir_host_builtin_catalog"
if ! grep -q '^macro_rules! host_builtin_catalog' "$ir_builtins"; then
  fail "$ir_builtins must generate HostBuiltinId from host_builtin_catalog"
fi
if ! grep -q '^host_builtin_catalog!' "$ir_host_builtin_catalog"; then
  fail "$ir_host_builtin_catalog must be the single host builtin catalog invocation"
fi
host_builtin_catalog_rows="$(grep -Ec '^    [A-Za-z][A-Za-z0-9]* \{$' "$ir_host_builtin_catalog")"
if [[ "$host_builtin_catalog_rows" != "19" ]]; then
  fail "$ir_host_builtin_catalog must contain the reviewed 19-row host builtin catalog (found $host_builtin_catalog_rows)"
fi
# Measured after the Date host-clock catalog correction: 1,751 raw lines.
# Metadata rows belong in their catalogs; shared machinery should shrink rather
# than regrow.
check_raw_line_budget "$ir_builtins" 1760

for module in abi arguments_protocol control_flow data emit environments expressions functions gc_types heap module modules objects operations planning; do
  require_file "crates/lila-aot-wasm/src/${module}.rs"
  require_module_decl "$wasm_lib" "$module"
done

# T05's typed Wasm-GC schema is the sole raw struct-instruction boundary. The
# encoder dependency necessarily accepts interchangeable u32 immediates, so
# direct StructNew/Get/Set construction anywhere else would discard the
# owner/field/target types before the final encoding step.
wasm_gc_types="crates/lila-aot-wasm/src/gc_types.rs"
gc_instruction_escapes="$(
  find crates/lila-aot-wasm/src -type f -name '*.rs' ! -path "$wasm_gc_types" -print0 \
    | xargs -0 grep -En 'Instruction::Struct(New|Get|Set)' || true
)"
if [ -n "$gc_instruction_escapes" ]; then
  fail "raw Wasm-GC struct instructions must stay in $wasm_gc_types: $gc_instruction_escapes"
fi
require_fixed_string_count \
  "$wasm_gc_types" \
  'function.instruction(&Instruction::StructNew(' \
  1 \
  'typed StructNew encoder boundary'
require_fixed_string_count \
  "$wasm_gc_types" \
  'function.instruction(&Instruction::StructGet {' \
  1 \
  'typed StructGet encoder boundary'
require_fixed_string_count \
  "$wasm_gc_types" \
  'function.instruction(&Instruction::StructSet {' \
  0 \
  'typed StructSet encoder boundary before a mutable GC field exists'

for module in array bigint binary_data boolean bootstrap date errors function \
              global_numeric host iterators json math number object proxy reflect \
              standard string symbol uri; do
  require_file "crates/lila-aot-wasm/src/builtins/${module}.rs"
  require_module_decl "$wasm_builtins_mod" "$module"
done

wasm_builtin_bootstrap="crates/lila-aot-wasm/src/builtins/bootstrap.rs"
if ! grep -q 'match builtin\.intrinsic_installer()' "$wasm_builtin_bootstrap"; then
  fail "$wasm_builtin_bootstrap must dispatch through the catalog installer class"
fi


# T02's Object, Proxy, Math, Symbol, BigInt, Boolean, Number, Function, global
# numeric, URI, Error and JSON
# builtin body boundaries. The exhaustive StandardBuiltinId dispatch remains in
# standard.rs, but family bodies are one-line delegates so unrelated builtin
# work no longer collides with ~11k lines of Object descriptor/prototype
# implementation, the Proxy lifecycle, the Math emitter family, Symbol's
# registry/prototype implementation or BigInt's constructor, fixed-width and
# prototype implementation, Boolean's constructor and prototype receiver logic,
# Number's constructor, predicates and prototype methods, Function's constructor
# and four prototype methods, the Error intrinsic family, or JSON's
# parse/stringify/raw-JSON wrappers. The two coercing global numeric predicates
# and the six global URI and Annex-B codec wrappers likewise stay out of the
# shared dispatcher.
check_no_inline_legacy_includes "$wasm_standard_builtins"
# Measured immediately after Number extraction: 33,512 raw lines. This leaves
# 213 lines of dispatch-maintenance headroom and removes most of the prior cap's
# drift; substantive bodies belong in family modules.
check_raw_line_budget "$wasm_standard_builtins" 33725

wasm_boolean_builtins="crates/lila-aot-wasm/src/builtins/boolean.rs"
check_no_inline_legacy_includes "$wasm_boolean_builtins"
if ! grep -q '^pub(super) enum BooleanBuiltin' "$wasm_boolean_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_boolean_builtins"; then
  fail "$wasm_boolean_builtins must dispatch through the closed BooleanBuiltin domain"
fi
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_boolean_builtin(' 3 'Boolean builtin delegate'
# Measured immediately after extraction: 139 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_boolean_builtins" 175

wasm_number_builtins="crates/lila-aot-wasm/src/builtins/number.rs"
check_no_inline_legacy_includes "$wasm_number_builtins"
if ! grep -q '^pub(super) enum NumberBuiltin' "$wasm_number_builtins" \
  || ! grep -q '^enum NumberPrototypeOperation' "$wasm_number_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_number_builtins" \
  || ! grep -q '^        match operation {' "$wasm_number_builtins"; then
  fail "$wasm_number_builtins must dispatch through the closed Number builtin/prototype domains"
fi
require_fixed_string_count \
  "$wasm_number_builtins" \
  'fn emit_number_constructor_builtin(' \
  1 \
  'Number constructor body'
require_fixed_string_count \
  "$wasm_number_builtins" \
  'fn emit_number_prototype_builtin(' \
  1 \
  'shared Number prototype receiver/body dispatch'
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'emit_number_builtin(' \
  11 \
  'Number builtin delegate'
if grep -q 'StandardBuiltinId::' "$wasm_number_builtins"; then
  fail "$wasm_number_builtins must accept only its closed family domains, not StandardBuiltinId"
fi
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' "$wasm_number_builtins"; then
  fail "$wasm_number_builtins must keep both family matches exhaustive without catch-all arms"
fi
require_fixed_string_count \
  crates/lila-cli/tests/cli/language_numerics.rs \
  'fn run_wasm_backend_succeeds_for_number_builtin_family_fixture()' \
  1 \
  'Number builtin-family CLI regression'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_number_builtin_family.js ]; then
  fail 'Number builtin-family fixture must remain present'
fi
# Measured immediately after extraction: 328 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_number_builtins" 370

wasm_function_builtins="crates/lila-aot-wasm/src/builtins/function.rs"
check_no_inline_legacy_includes "$wasm_function_builtins"
if ! grep -q '^pub(super) enum FunctionBuiltin' "$wasm_function_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_function_builtins"; then
  fail "$wasm_function_builtins must dispatch through the closed FunctionBuiltin domain"
fi
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.emit_function_builtin(' \
  5 \
  'Function builtin delegate'
# Measured immediately after extraction: 411 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_function_builtins" 450

wasm_bigint_builtins="crates/lila-aot-wasm/src/builtins/bigint.rs"
wasm_numeric_operations="crates/lila-aot-wasm/src/operations.rs"
wasm_emit="crates/lila-aot-wasm/src/emit.rs"
wasm_host_builtins="crates/lila-aot-wasm/src/builtins/host.rs"
check_no_inline_legacy_includes "$wasm_bigint_builtins"
if ! grep -q '^pub(super) enum BigIntBuiltin' "$wasm_bigint_builtins" \
  || ! grep -q '^pub(super) enum BigIntPrototypeResultPolicy' "$wasm_bigint_builtins" \
  || ! grep -q '^struct PreparedBigIntRadixLocal' "$wasm_bigint_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_bigint_builtins" \
  || ! grep -q '^                match result_policy {' "$wasm_bigint_builtins"; then
  fail "$wasm_bigint_builtins must dispatch through the closed BigInt builtin/result/radix domains"
fi
require_fixed_string_count \
  "$wasm_bigint_builtins" \
  'fn emit_prepare_bigint_radix(' \
  1 \
  'shared prepared-radix stage'

if ! grep -q '^pub(crate) enum NumericErrorRealmSource' "$wasm_emit" \
  || ! grep -q '^    numeric_error_realm_source: NumericErrorRealmSource,$' "$wasm_emit" \
  || ! grep -q '^    pub(crate) const fn numeric_error_realm_source' "$wasm_emit" \
  || ! grep -Fq 'RuntimeHelperId::ValueToNumber | RuntimeHelperId::ValueToNumeric' "$wasm_emit" \
  || ! grep -Fq 'self.numeric_error_realm_source = NumericErrorRealmSource::for_runtime_helper(helper);' "$wasm_emit"; then
  fail "$wasm_emit must carry the closed standard-builtin/numeric-helper/global-fallback Realm source"
fi
require_fixed_string_count \
  "$wasm_emit" \
  '            NumericErrorRealmSource::GlobalFallback,' \
  4 \
  'main/user/host/runtime-helper fallback constructor'
main_numeric_realm_source="$(
  sed -n '/^    fn new_main(/,/^    fn new_function(/p' "$wasm_emit"
)"
main_fallback_count="$(
  printf '%s\n' "$main_numeric_realm_source" \
    | grep -Fc 'NumericErrorRealmSource::GlobalFallback' \
    || true
)"
if [ "$main_fallback_count" -ne 1 ] \
  || printf '%s\n' "$main_numeric_realm_source" \
    | grep -Eq 'NumericErrorRealmSource::(StandardBuiltinEnvironment|NumericConversionHelperArgument)'; then
  fail "$wasm_emit new_main must select exactly one GlobalFallback numeric-error Realm source"
fi
require_fixed_string_count \
  "$wasm_emit" \
  '            NumericErrorRealmSource::StandardBuiltinEnvironment,' \
  1 \
  'trusted standard-builtin constructor'
require_fixed_string_count \
  "$wasm_numeric_operations" \
  'self.emit_outlined_numeric_realm_argument(function);' \
  2 \
  'realm-aware ToNumber/ToNumeric helper ABI consumer'

bigint_prototype_realm_slots="$(
  sed -n \
    '/for (name, meta) in &bigint_prototype_method_metas {/,/self.release_temp_local(method_payload_local);/p' \
    "$wasm_host_builtins"
)"
for slot in \
  HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET \
  HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET; do
  slot_count="$(printf '%s\n' "$bigint_prototype_realm_slots" | grep -Fc "$slot" || true)"
  if [ "$slot_count" -ne 1 ]; then
    fail "$wasm_host_builtins BigInt prototype-method loop must store exactly one $slot (found $slot_count)"
  fi
done

# Measured after T20's closed result/radix policy: 882 raw lines. The 38-line
# margin is for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_bigint_builtins" 920

wasm_global_numeric_builtins="crates/lila-aot-wasm/src/builtins/global_numeric.rs"
check_no_inline_legacy_includes "$wasm_global_numeric_builtins"
if ! grep -q '^pub(super) enum GlobalNumericBuiltin' "$wasm_global_numeric_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_global_numeric_builtins"; then
  fail "$wasm_global_numeric_builtins must dispatch through the closed GlobalNumericBuiltin domain"
fi
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.emit_global_numeric_builtin(' \
  2 \
  'global numeric builtin delegate'
# Measured immediately after extraction: 51 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_global_numeric_builtins" 75

wasm_symbol_builtins="crates/lila-aot-wasm/src/builtins/symbol.rs"
check_no_inline_legacy_includes "$wasm_symbol_builtins"
if ! grep -q '^pub(super) enum SymbolBuiltin' "$wasm_symbol_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_symbol_builtins"; then
  fail "$wasm_symbol_builtins must dispatch through the closed SymbolBuiltin domain"
fi
# Measured immediately after extraction: 518 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_symbol_builtins" 550

wasm_uri_builtins="crates/lila-aot-wasm/src/builtins/uri.rs"
check_no_inline_legacy_includes "$wasm_uri_builtins"
if ! grep -q '^pub(super) enum UriBuiltin' "$wasm_uri_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_uri_builtins"; then
  fail "$wasm_uri_builtins must dispatch through the closed UriBuiltin domain"
fi
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_uri_builtin(' 6 'URI builtin delegate'
# Measured immediately after extraction: 82 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_uri_builtins" 115

wasm_error_builtins="crates/lila-aot-wasm/src/builtins/errors.rs"
check_no_inline_legacy_includes "$wasm_error_builtins"
if ! grep -q '^pub(super) enum ErrorBuiltin' "$wasm_error_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_error_builtins" \
  || ! grep -q '^            ErrorBuiltin::Constructor(error_kind) => match error_kind {' "$wasm_error_builtins"; then
  fail "$wasm_error_builtins must dispatch through the closed ErrorBuiltin domain"
fi
require_fixed_string_count \
  "$wasm_standard_builtins" \
  '.emit_error_builtin(' \
  11 \
  'Error builtin delegate'
# Measured after making the nested NativeErrorKind match exhaustive: 1,646 raw
# lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_error_builtins" 1700

wasm_json_builtins="crates/lila-aot-wasm/src/builtins/json.rs"
check_no_inline_legacy_includes "$wasm_json_builtins"
if ! grep -q '^pub(super) enum JsonBuiltin' "$wasm_json_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_json_builtins"; then
  fail "$wasm_json_builtins must dispatch through the closed JsonBuiltin domain"
fi
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.emit_json_builtin(' \
  4 \
  'JSON builtin delegate'
# Measured immediately after extraction: 9,274 raw lines. The narrow margin is
# for maintenance of JSON machinery, not adjacent builtin implementations.
check_raw_line_budget "$wasm_json_builtins" 9400

# The Temporal record/constructor/accessor vs prototype-method-body boundary.
# `temporal.rs` and `temporal_plain_date_time.rs` hold the heap record, the
# constructor and the accessors; the `*_methods.rs` files hold the prototype
# method bodies. Both halves of each pair are required here so a lane adding a
# prototype method cannot quietly reinflate the record file — `temporal.rs` was
# already 7,402 lines before the batch-6 ZonedDateTime arithmetic surface was
# written, and that surface is the third `*_methods.rs` split of the same kind.
for module in temporal temporal_plain_date_time temporal_plain_date_time_methods \
              temporal_zoned_date_time_methods; do
  require_file "crates/lila-aot-wasm/src/builtins/${module}.rs"
  require_module_decl "$wasm_builtins_mod" "$module"
done

# T02's realm-bootstrap boundary. These files hold the per-family property and
# descriptor installation extracted out of the single
# init_builtin_constructor_object function, which every builtin lane previously
# had to edit. Requiring them keeps that split from silently collapsing back.
require_file "$wasm_intrinsics_mod"
require_module_decl "$wasm_lib" "intrinsics"
for module in array binary_data collections date errors function iterator numeric object promise proxy regexp string symbol temporal; do
  require_file "crates/lila-aot-wasm/src/intrinsics/${module}.rs"
  require_module_decl "$wasm_intrinsics_mod" "$module"
done
check_no_inline_legacy_includes "$wasm_intrinsics_mod"

require_pub_use "$wasm_lib" '^pub use emit::emit;' 'the Wasm emit entry point'
# 180 against a CODE-ONLY count, measured 101 at batch 6 (118 raw). Unlike the
# `lila-ir` budget above this one was never near its limit, so the switch to a
# code-only count did not need a matching adjustment.
check_orchestration_surface "$wasm_lib" 180
check_no_inline_legacy_includes "$wasm_lib"
check_no_inline_legacy_includes "$wasm_builtins_mod"

# T20's Number-to-32-bit residue boundary. The binary64 modulo must remain in
# one backend emitter: integer typed arrays, DataView setters and Math methods
# previously grew local conversions that trapped or discarded finite values at
# and above 2^63. Exact call counts make removing one route a static failure;
# adding a new consumer intentionally requires reviewing this inventory.
wasm_uint32_authority="crates/lila-aot-wasm/src/operations.rs"
uint32_modulus='Instruction::F64Const(Ieee64::from(4_294_967_296.0))'
uint32_modulus_files="$(grep -RFl --include='*.rs' "$uint32_modulus" crates/lila-aot-wasm/src || true)"
if [ "$uint32_modulus_files" != "$wasm_uint32_authority" ]; then
  fail "the exact modulo-2^32 implementation must exist only in $wasm_uint32_authority (found: ${uint32_modulus_files:-none})"
fi
require_fixed_string_count "$wasm_uint32_authority" "$uint32_modulus" 2 'modulo-2^32 constant'

uint32_call='self.emit_to_uint32_i64_from_number_payload('
uint32_consumer_files="$(grep -RFl --include='*.rs' "$uint32_call" crates/lila-aot-wasm/src | sort || true)"
expected_uint32_consumer_files="$(printf '%s\n' \
  crates/lila-aot-wasm/src/builtins/array.rs \
  crates/lila-aot-wasm/src/builtins/math.rs \
  crates/lila-aot-wasm/src/builtins/standard.rs \
  crates/lila-aot-wasm/src/builtins/string.rs \
  crates/lila-aot-wasm/src/expressions.rs \
  crates/lila-aot-wasm/src/objects.rs \
  crates/lila-aot-wasm/src/operations.rs | sort)"
if [ "$uint32_consumer_files" != "$expected_uint32_consumer_files" ]; then
  fail "the reviewed modulo-2^32 consumer inventory changed"
fi
require_fixed_string_count crates/lila-aot-wasm/src/builtins/array.rs "$uint32_call" 1 'ToUint32 authority call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/math.rs "$uint32_call" 3 'ToUint32 authority call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/standard.rs "$uint32_call" 3 'ToUint32 authority call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/string.rs "$uint32_call" 1 'ToUint32 authority call'
require_fixed_string_count crates/lila-aot-wasm/src/expressions.rs "$uint32_call" 1 'ToUint32 authority call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs "$uint32_call" 1 'ToUint32 authority call'
require_fixed_string_count crates/lila-aot-wasm/src/operations.rs "$uint32_call" 4 'ToUint32 authority call'

# T20's variadic Math extremum walk. The call ABI already owns an arbitrary
# argc/argv domain, so min/max must consume the runtime vector rather than grow
# another reviewed-looking finite prefix. The private enum owns the paired
# identity/reduction decisions; the loop owns every argument conversion.
math_extremum_file="crates/lila-aot-wasm/src/builtins/math.rs"
require_fixed_string_count "$math_extremum_file" 'enum MathExtremum {' 1 'closed Math extremum domain'
require_fixed_string_count "$math_extremum_file" 'fn identity(self) -> f64 {' 1 'Math extremum identity projection'
require_fixed_string_count "$math_extremum_file" 'fn emit_combine(' 1 'Math extremum reduction projection'
require_fixed_string_count "$math_extremum_file" 'emit_math_extremum_builtin(' 3 'Math extremum definition/min/max consumers'

math_extremum_body="$(sed -n \
  '/^    fn emit_math_extremum_builtin(/,/^    pub(super) fn emit_math(/p' \
  "$math_extremum_file")"
for variadic_extremum_step in \
  'extremum.identity()' \
  'Instruction::Loop(BlockType::Empty)' \
  'self.argv_param_local()' \
  'self.argc_param_local()' \
  'Instruction::I64GeU' \
  'Instruction::BrIf(1)' \
  'self.emit_value_to_number_payload(' \
  'self.emit_return_current_completion_if_throw(function)' \
  'extremum.emit_combine('
do
  if ! grep -Fq "$variadic_extremum_step" <<<"$math_extremum_body"; then
    fail "Math min/max variadic walk lost $variadic_extremum_step"
  fi
done
if grep -Fq 'emit_builtin_arg_to_locals(' <<<"$math_extremum_body"; then
  fail 'Math min/max must not reconstruct a fixed argument-index prefix'
fi
for extremum_instruction_count in \
  'Instruction::LocalGet(argument_index_local)|2' \
  'Instruction::I64Const(1)|1' \
  'Instruction::I64Add|1' \
  'Instruction::LocalSet(argument_index_local)|2' \
  'Instruction::Br(0)|1'
do
  extremum_instruction="${extremum_instruction_count%|*}"
  expected_extremum_count="${extremum_instruction_count##*|}"
  actual_extremum_count="$(grep -Fc "$extremum_instruction" <<<"$math_extremum_body" || true)"
  if [ "$actual_extremum_count" -ne "$expected_extremum_count" ]; then
    fail "Math min/max variadic walk must contain exactly $expected_extremum_count $extremum_instruction instruction(s)"
  fi
done
math_extremum_increment_sequence='        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(argument_index_local));
        function.instruction(&Instruction::Br(0));'
if ! grep -Fq "$math_extremum_increment_sequence" <<<"$math_extremum_body"; then
  fail 'Math min/max reduction must advance the argument index and take the loop backedge as one exact sequence'
fi
if ! awk '
  /self\.emit_value_to_number_payload\(/ && !convert { convert = NR }
  /Instruction::LocalSet\(arg_payload_local\)/ && convert && !store { store = NR }
  /self\.emit_return_current_completion_if_throw\(function\)/ && !route { route = NR }
  /extremum\.emit_combine\(/ && !combine { combine = NR }
  /Instruction::LocalGet\(argument_index_local\)/ && combine && !increment_get { increment_get = NR }
  /Instruction::I64Const\(1\)/ && increment_get && !increment_one { increment_one = NR }
  /Instruction::I64Add/ && increment_one && !increment_add { increment_add = NR }
  /Instruction::LocalSet\(argument_index_local\)/ && increment_add && !increment_store { increment_store = NR }
  /Instruction::Br\(0\)/ && increment_store && !backedge { backedge = NR }
  END {
    exit !(convert && store && route && combine && increment_get && increment_one &&
      increment_add && increment_store && backedge && convert < store &&
      store < route && route < combine && combine < increment_get &&
      increment_get < increment_one && increment_one < increment_add &&
      increment_add < increment_store && increment_store < backedge)
  }
' <<<"$math_extremum_body"; then
  fail 'Math min/max must route abrupt ToNumber completion, reduce, advance and branch in exact order'
fi
require_fixed_string_count \
  crates/lila-cli/tests/cli/language_numerics.rs \
  'fn run_wasm_backend_succeeds_for_math_extremum_argument_reduction()' \
  1 \
  'Math extremum variadic CLI regression'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_math_min_max_arity.js ]; then
  fail 'Math extremum variadic fixture must remain present'
fi

# T11's direct [[GetOwnProperty]] observations. One typed authority owns the
# representation split used by the value-free public descriptor/Has/Delete
# fact and the richer Proxy-Get/Proxy-Set projections. Array-only or
# ordinary-entry mirrors would let a new exotic silently escape one consumer
# again, so keep the closed branch order, projection domain and reviewed call
# sites exact.
own_descriptor_fact='emit_direct_own_descriptor_fact('
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'pub(crate) fn emit_direct_own_descriptor_fact(' \
  1 \
  'typed own-descriptor fact authority'
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'for branch in ObjectInternalMethodBranch::ORDER.iter().copied() {' \
  2 \
  'closed object-internal-method branch consumer'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs "$own_descriptor_fact" 3 'own-descriptor fact definition/HasProperty/Proxy Delete call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/object.rs "$own_descriptor_fact" 2 'Object.getOwnPropertyDescriptor invariant call'
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'enum DirectOwnDescriptorProjectionLocals {' \
  1 \
  'closed direct-own-descriptor projection domain'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'pub(crate) struct PropertyKeyLocals(TaggedLocals);' 1 'typed Proxy-Get/Set property key role'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'pub(crate) struct ProxySetValueLocals(TaggedLocals);' 1 'typed Proxy-Set incoming value role'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'struct PendingProxyGetTrapResultLocals(TaggedLocals);' 1 'pending Proxy-Get trap-result role'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'struct NormalProxyGetTrapResultLocals(TaggedLocals);' 1 'normal Proxy-Get trap-result role'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'struct DescriptorDataValueLocals(TaggedLocals);' 1 'typed Proxy-Get/Set descriptor data-value role'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'struct DescriptorGetterLocals(TaggedLocals);' 1 'typed descriptor getter role'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'struct DescriptorSetterLocals(TaggedLocals);' 1 'typed descriptor setter role'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'struct ProxyGetDescriptorLocals {' 1 'complete Proxy-Get descriptor projection'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'enum DescriptorAccessorProjectionLocals {' 1 'closed getter/setter endpoint projection'
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'fn emit_direct_own_descriptor(' \
  1 \
  'direct-own-descriptor representation authority'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_direct_own_descriptor(' 4 'direct-own-descriptor definition/fact/Proxy Get/Proxy Set call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_direct_own_descriptor_for_proxy_get(' 2 'typed Proxy-Get descriptor wrapper definition/call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_direct_own_descriptor_for_proxy_set(' 2 'typed Proxy-Set descriptor wrapper definition/call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_proxy_get_invariant_check(' 2 'Proxy-Get invariant definition/object-read call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_proxy_set_invariant_check(' 2 'Proxy-Set invariant definition/object-write call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/reflect.rs 'emit_proxy_set_invariant_check(' 1 'Reflect.set typed Proxy-Set invariant call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'DirectOwnDescriptorProjectionLocals::ProxyGet(' 1 'complete Proxy-Get descriptor projection construction'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'DirectOwnDescriptorProjectionLocals::ProxySet(' 1 'complete Proxy-Set descriptor projection construction'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_normal_proxy_get_trap_result(' 2 'pending-to-normal Proxy-Get result transition definition/call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'PendingProxyGetTrapResultLocals::new(' 1 'pending Proxy-Get result construction'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'NormalProxyGetTrapResultLocals(' 2 'normal Proxy-Get type declaration/guarded construction'
require_fixed_string_count crates/lila-cli/tests/cli/object.rs 'fn run_wasm_backend_succeeds_for_proxy_get_direct_descriptor_invariants()' 1 'exact Proxy-Get direct-descriptor CLI regression'
require_fixed_string_count crates/lila-cli/tests/cli/object.rs '"wasm_proxy_get_direct_descriptor_invariants.js"' 1 'Proxy-Get direct-descriptor fixture wiring'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_proxy_get_direct_descriptor_invariants.js ]; then
  fail 'Proxy [[Get]] direct-descriptor invariant fixture must remain present'
fi
if grep -RFl --include='*.rs' 'emit_proxy_array_target_own_descriptor_flags' crates/lila-aot-wasm/src >/dev/null; then
  fail 'Array-only Proxy own-descriptor mirrors must not bypass the typed authority'
fi

normal_proxy_get_transition="$(sed -n \
  '/^    fn emit_normal_proxy_get_trap_result(/,/^    fn emit_proxy_get_descriptor_same_value_i32(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
if ! grep -Fq 'pending: PendingProxyGetTrapResultLocals' <<<"$normal_proxy_get_transition" \
  || ! grep -Fq 'self.emit_return_current_completion_if_throw(function);' <<<"$normal_proxy_get_transition" \
  || ! grep -Fq 'NormalProxyGetTrapResultLocals(pending.0)' <<<"$normal_proxy_get_transition"; then
  fail 'only the abrupt-routing transition may construct a normal Proxy [[Get]] trap result'
fi

proxy_get_invariant_body="$(sed -n \
  '/^    fn emit_proxy_get_invariant_check(/,/^    pub(crate) fn reserve_own_descriptor_fact_locals(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
for raw_proxy_get_scan in \
  'emit_is_object_entry_backed_tag_i32' \
  'HEAP_PTR_OFFSET' \
  'HEAP_LEN_OFFSET' \
  'HEAP_OBJECT_ENTRY_SIZE' \
  'HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET' \
  'HEAP_OBJECT_DATA_' \
  'HEAP_OBJECT_GETTER_'
do
  if grep -Fq "$raw_proxy_get_scan" <<<"$proxy_get_invariant_body"; then
    fail "Proxy [[Get]] invariant must not rebuild descriptor storage through $raw_proxy_get_scan"
  fi
done
if ! grep -Fq 'target: ProxyTargetLocals' <<<"$proxy_get_invariant_body" \
  || ! grep -Fq 'key: PropertyKeyLocals' <<<"$proxy_get_invariant_body" \
  || ! grep -Fq 'trap_result: NormalProxyGetTrapResultLocals' <<<"$proxy_get_invariant_body"; then
  fail 'Proxy [[Get]] invariant must accept only typed target/key/normal-result roles'
fi
if ! grep -Fq 'emit_direct_own_descriptor_for_proxy_get(' <<<"$proxy_get_invariant_body"; then
  fail 'Proxy [[Get]] invariant must consume the typed direct-own-descriptor projection'
fi
if ! grep -Fq 'descriptor.getter.emit_undefined_i32(' <<<"$proxy_get_invariant_body"; then
  fail 'Proxy [[Get]] accessor invariant must normalize raw-zero and tagged-undefined getters'
fi

descriptor_getter_predicate="$(sed -n \
  '/^impl DescriptorGetterLocals {/,/^}/p' \
  crates/lila-aot-wasm/src/objects.rs)"
if ! grep -Fq 'Instruction::I64Eqz' <<<"$descriptor_getter_predicate" \
  || ! grep -Fq 'ValueKind::Undefined.tag()' <<<"$descriptor_getter_predicate" \
  || ! grep -Fq 'Instruction::I32Or' <<<"$descriptor_getter_predicate"; then
  fail 'descriptor getter absence must accept both raw zero and tagged undefined'
fi

proxy_set_invariant_body="$(sed -n \
  '/^    pub(crate) fn emit_proxy_set_invariant_check(/,/^    pub(crate) fn emit_object_delete_ordinary(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
for raw_proxy_set_scan in \
  'emit_is_object_entry_backed_tag_i32' \
  'HEAP_PTR_OFFSET' \
  'HEAP_LEN_OFFSET' \
  'HEAP_OBJECT_ENTRY_SIZE' \
  'HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET' \
  'HEAP_OBJECT_DATA_' \
  'HEAP_OBJECT_SETTER_'
do
  if grep -Fq "$raw_proxy_set_scan" <<<"$proxy_set_invariant_body"; then
    fail "Proxy [[Set]] invariant must not rebuild descriptor storage through $raw_proxy_set_scan"
  fi
done
if ! grep -Fq 'emit_direct_own_descriptor_for_proxy_set(' <<<"$proxy_set_invariant_body"; then
  fail 'Proxy [[Set]] invariant must consume the typed direct-own-descriptor projection'
fi
if ! grep -Fq 'descriptor.setter.emit_undefined_i32(' <<<"$proxy_set_invariant_body"; then
  fail 'Proxy [[Set]] accessor invariant must test exactly tagged undefined'
fi

direct_own_descriptor_body="$(sed -n \
  '/^    fn emit_direct_own_descriptor(/,/^    pub(crate) fn emit_proxy_has_invariant_check(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
for observable_descriptor_read in \
  'emit_object_read(' \
  'emit_object_read_ordinary(' \
  'emit_array_index_get(' \
  'emit_array_sparse_present_get(' \
  'emit_arguments_callee_read(' \
  'emit_arguments_length_read(' \
  'emit_function_handle_call(' \
  'emit_function_or_proxy_call_leave_throw_completion('
do
  if grep -Fq "$observable_descriptor_read" <<<"$direct_own_descriptor_body"; then
    fail "direct own-descriptor observation must not invoke getters through $observable_descriptor_read"
  fi
done
if ! grep -Fq 'DescriptorAccessorProjectionLocals::Getter(' <<<"$direct_own_descriptor_body" \
  || ! grep -Fq 'HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET' <<<"$direct_own_descriptor_body" \
  || ! grep -Fq 'HEAP_ARGUMENTS_CALLEE_VALUE_PAYLOAD_OFFSET' <<<"$direct_own_descriptor_body"; then
  fail 'direct own-descriptor observation must project getter storage for every accessor representation'
fi

entry_descriptor_projection_body="$(sed -n \
  '/^    fn emit_own_descriptor_from_entries(/,/^    pub(crate) fn emit_direct_own_descriptor_fact(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
if ! grep -Fq 'HEAP_OBJECT_GETTER_PAYLOAD_OFFSET' <<<"$entry_descriptor_projection_body" \
  || ! grep -Fq 'HEAP_OBJECT_GETTER_TAG_OFFSET' <<<"$entry_descriptor_projection_body"; then
  fail 'ordinary descriptor entry projection must retain the stored getter without invoking it'
fi

ordinary_direct_descriptor_body="$(sed -n \
  '/ObjectInternalMethodBranch::Ordinary => {/,/self.release_temp_local(function_like_local)/p' \
  <<<"$direct_own_descriptor_body")"
if [ "$(grep -Fc 'self.emit_own_descriptor_from_entries(' <<<"$ordinary_direct_descriptor_body" || true)" -ne 1 ]; then
  fail 'ordinary direct descriptors must have one exact entry-storage projection'
fi
if ! awk '
  /self\.emit_own_descriptor_from_entries\(/ && !entry { entry = NR }
  /for \(target_tag, target_global, key, descriptor\) in \[/ { intrinsic = NR }
  /DescriptorWord::of_data\(true, false, false\)\.as_i64\(\)/ { function_fallback = NR }
  END { exit !(entry && intrinsic && function_fallback && entry < intrinsic && intrinsic < function_fallback) }
' <<<"$ordinary_direct_descriptor_body"; then
  fail 'ordinary descriptor precedence must be entry storage, intrinsic fallback, then Function prototype fallback'
fi
function_prototype_fallback="$(sed -n \
  '/A function-like value with no materialized entry still/,/DescriptorWord::of_data(true, false, false).as_i64()/p' \
  <<<"$ordinary_direct_descriptor_body")"
if ! grep -Fq 'Instruction::LocalGet(fact.present)' <<<"$function_prototype_fallback" \
  || ! grep -Fq 'Instruction::I64Eqz' <<<"$function_prototype_fallback"; then
  fail 'Function prototype fallback must be gated on an absent real entry'
fi

# T11's Proxy record has one typed writer and one typed live reader. Keep the
# raw handler-tag offset private to objects.rs (apart from its heap declaration)
# and keep the reviewed HasProperty, GetPrototypeOf, IsExtensible and public
# descriptor consumers on the reader so no path can silently reconstruct an
# Object tag.
proxy_slot_reader='emit_load_live_proxy_slots('
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'pub(crate) fn emit_load_live_proxy_slots(' \
  1 \
  'typed live-Proxy-slot reader authority'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs "$proxy_slot_reader" 4 'live-Proxy-slot reader definition/internal call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/object.rs "$proxy_slot_reader" 1 'public descriptor live-Proxy-slot reader call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'HEAP_PROXY_HANDLER_TAG_OFFSET' 2 'Proxy handler-tag writer/reader authority'
proxy_handler_tag_files="$(grep -RFl --include='*.rs' 'HEAP_PROXY_HANDLER_TAG_OFFSET' crates/lila-aot-wasm/src | sort || true)"
expected_proxy_handler_tag_files="$(printf '%s\n' \
  crates/lila-aot-wasm/src/heap.rs \
  crates/lila-aot-wasm/src/objects.rs | sort)"
if [ "$proxy_handler_tag_files" != "$expected_proxy_handler_tag_files" ]; then
  fail 'Proxy handler-tag heap access must stay inside the typed slot authority'
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-module-boundaries: ok\n'
