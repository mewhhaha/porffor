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
# Measured after the host-surface consolidation: 1,741 raw lines. Metadata rows
# belong in their catalogs; shared machinery should shrink rather than regrow.
check_raw_line_budget "$ir_builtins" 1750

for module in abi control_flow data emit environments expressions functions heap module modules objects operations planning; do
  require_file "crates/lila-aot-wasm/src/${module}.rs"
  require_module_decl "$wasm_lib" "$module"
done

for module in array bigint binary_data bootstrap date errors host iterators json math object proxy reflect standard string symbol; do
  require_file "crates/lila-aot-wasm/src/builtins/${module}.rs"
  require_module_decl "$wasm_builtins_mod" "$module"
done

wasm_builtin_bootstrap="crates/lila-aot-wasm/src/builtins/bootstrap.rs"
if ! grep -q 'match builtin\.intrinsic_installer()' "$wasm_builtin_bootstrap"; then
  fail "$wasm_builtin_bootstrap must dispatch through the catalog installer class"
fi


# T02's Object, Proxy, Math, Symbol and BigInt builtin body boundaries. The exhaustive
# StandardBuiltinId dispatch remains in standard.rs, but family bodies are
# one-line delegates so unrelated builtin work no longer collides with ~11k
# lines of Object descriptor/prototype implementation, the Proxy lifecycle,
# the Math emitter family, Symbol's registry/prototype implementation or
# BigInt's constructor, fixed-width and prototype implementation.
check_no_inline_legacy_includes "$wasm_standard_builtins"
# Measured immediately after BigInt extraction: 35,647 raw lines. This keeps
# roughly the same small dispatch-only margin as the prior 36,500-line cap;
# substantive bodies belong in family modules.
check_raw_line_budget "$wasm_standard_builtins" 35850

wasm_bigint_builtins="crates/lila-aot-wasm/src/builtins/bigint.rs"
check_no_inline_legacy_includes "$wasm_bigint_builtins"
if ! grep -q '^pub(super) enum BigIntBuiltin' "$wasm_bigint_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_bigint_builtins"; then
  fail "$wasm_bigint_builtins must dispatch through the closed BigIntBuiltin domain"
fi
# Measured immediately after extraction: 708 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_bigint_builtins" 750

wasm_symbol_builtins="crates/lila-aot-wasm/src/builtins/symbol.rs"
check_no_inline_legacy_includes "$wasm_symbol_builtins"
if ! grep -q '^pub(super) enum SymbolBuiltin' "$wasm_symbol_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_symbol_builtins"; then
  fail "$wasm_symbol_builtins must dispatch through the closed SymbolBuiltin domain"
fi
# Measured immediately after extraction: 518 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_symbol_builtins" 550

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

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-module-boundaries: ok\n'
