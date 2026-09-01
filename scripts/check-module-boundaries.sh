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

require_exact_line_count() {
  file="$1"
  line="$2"
  expected="$3"
  description="$4"
  count="$(grep -Fxc "$line" "$file" || true)"
  if [ "$count" -ne "$expected" ]; then
    fail "$file must contain $expected exact $description lines (found $count)"
  fi
}

require_regex_count() {
  file="$1"
  pattern="$2"
  expected="$3"
  description="$4"
  count="$(grep -Ec "$pattern" "$file" || true)"
  if [ "$count" -ne "$expected" ]; then
    fail "$file must contain $expected $description lines (found $count)"
  fi
}

require_text_regex_count() {
  text="$1"
  pattern="$2"
  expected="$3"
  description="$4"
  count="$(printf '%s\n' "$text" | grep -Ec "$pattern" || true)"
  if [ "$count" -ne "$expected" ]; then
    fail "text must contain $expected $description lines (found $count)"
  fi
}

require_tree_regex_count() {
  root="$1"
  pattern="$2"
  expected="$3"
  description="$4"
  count="$({ grep -RhE --include='*.rs' "$pattern" "$root" || true; } | wc -l | tr -d '[:space:]')"
  if [ "$count" -ne "$expected" ]; then
    fail "$root must contain $expected $description declarations (found $count)"
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

if command -v sha256sum >/dev/null 2>&1; then
  sha256_stream() {
    sha256sum | cut -d ' ' -f 1
  }
elif command -v shasum >/dev/null 2>&1; then
  sha256_stream() {
    shasum -a 256 | cut -d ' ' -f 1
  }
else
  printf 'check-module-boundaries: sha256sum or shasum is required\n' >&2
  exit 1
fi

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
# T02's assignment-expression boundary owns the exhaustive AssignOp/target
# dispatch across identifier, property, private, destructuring, logical and
# eager compound writes. Its specialized Reference lifecycles remain in their
# typed child modules; the parent expression dispatcher cannot regrow a second
# assignment implementation.
ir_assignment_lowering="crates/lila-ir/src/lowering/assignment.rs"
require_file "$ir_assignment_lowering"
require_module_decl "$ir_lowering" "assignment"
require_fixed_string_count \
  "$ir_assignment_lowering" \
  'pub(super) fn lower_assign(' \
  1 \
  'assignment-expression lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_assign(' \
  0 \
  'assignment-expression lowering outside child module'
check_no_inline_legacy_includes "$ir_assignment_lowering"
# Measured after formatting the extraction: 716 raw lines. The margin is for
# maintenance of this exhaustive dispatcher, not unrelated lowering.
check_raw_line_budget "$ir_assignment_lowering" 770
# T02's delete-expression boundary owns the complete target dispatch for
# property, private, super, identifier and value deletion. The unary
# dispatcher remains its sole caller and cannot regrow a second implementation.
ir_delete_expression_lowering="crates/lila-ir/src/lowering/delete_expression.rs"
require_file "$ir_delete_expression_lowering"
require_module_decl "$ir_lowering" "delete_expression"
require_fixed_string_count \
  "$ir_delete_expression_lowering" \
  'pub(super) fn lower_delete(' \
  1 \
  'delete-expression lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_delete(' \
  0 \
  'delete-expression lowering outside child module'
check_no_inline_legacy_includes "$ir_delete_expression_lowering"
# Measured after formatting the extraction: 217 raw lines. The margin is for
# maintenance of this exhaustive dispatcher, not unrelated lowering.
check_raw_line_budget "$ir_delete_expression_lowering" 250
# T02's new-expression boundary owns constructor target resolution, argument
# lowering, builtin/user result typing, dynamic-source rejection and static
# RegExp compilation. The parent expression dispatcher is its sole caller.
ir_new_expression_lowering="crates/lila-ir/src/lowering/new_expression.rs"
require_file "$ir_new_expression_lowering"
require_module_decl "$ir_lowering" "new_expression"
require_fixed_string_count \
  "$ir_new_expression_lowering" \
  'pub(super) fn lower_new(' \
  1 \
  'new-expression lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_new(' \
  0 \
  'new-expression lowering outside child module'
check_no_inline_legacy_includes "$ir_new_expression_lowering"
# Measured after formatting the extraction: 248 raw lines. The margin is for
# maintenance of constructor-expression lowering only.
check_raw_line_budget "$ir_new_expression_lowering" 290
# T02's statement boundary owns the exhaustive Statement dispatcher and its
# resumable expression-statement specialization. Control-flow implementations
# remain in their focused owners; the parent cannot regrow a second dispatcher.
ir_statement_lowering="crates/lila-ir/src/lowering/statement.rs"
require_file "$ir_statement_lowering"
require_module_decl "$ir_lowering" "statement"
require_fixed_string_count \
  "$ir_statement_lowering" \
  'pub(super) fn lower_statement(' \
  1 \
  'statement lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_statement(' \
  0 \
  'statement lowering outside child module'
check_no_inline_legacy_includes "$ir_statement_lowering"
# Measured after formatting the extraction: 259 raw lines. The margin is for
# maintenance of the exhaustive dispatcher, not statement implementations.
check_raw_line_budget "$ir_statement_lowering" 300
# T02's for-in boundary owns the complete initializer/target/body lowering
# family and its Test262-specific empty/non-enumerable recognizers. Only the
# statement-facing wrapper crosses this private child boundary; environment,
# TDZ, expression and static-analysis helpers remain in their shared owners.
ir_for_in_lowering="crates/lila-ir/src/lowering/for_in.rs"
require_file "$ir_for_in_lowering"
require_exact_line_count "$ir_lowering" 'mod for_in;' 1 'private for-in module declaration'
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+lower_for_in_loop[[:space:]]*\(' \
  1 \
  'for-in statement-facing owner'
require_regex_count \
  "$ir_lowering" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+lower_for_in_loop[[:space:]]*\(' \
  0 \
  'for-in lowerer outside child module'
for private_owner in \
  lower_for_in_initializer_prefix \
  prepend_statement \
  for_in_initializer_binding \
  for_in_known_empty_target \
  for_in_global_non_enumerable_guard_only \
  for_in_builtin_non_enumerable_assert_only \
  for_in_static_builtin_target \
  for_in_initializer_name \
  for_in_non_enumerable_guarded_assignment \
  for_in_non_enumerable_guard_name \
  for_in_not_same_value_guard_name \
  statement_is_simple_false_assignment
do
  require_regex_count \
    "$ir_for_in_lowering" \
    "^[[:space:]]*fn[[:space:]]+${private_owner}[[:space:]]*[<(]" \
    1 \
    "private ${private_owner} owner"
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${private_owner}[[:space:]]*[<(]" \
    0 \
    "${private_owner} outside child module"
done
# Twelve unmodified private methods plus the one reviewed pub(super) wrapper is
# the whole child surface. The total accepts every Rust function modifier so a
# new const/async/unsafe/extern helper cannot hide from the private-fn count.
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*fn[[:space:]]+' \
  12 \
  'private method'
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*((pub(\([^)]*\))?|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+[[:alnum:]_]+' \
  13 \
  'total function declaration'
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' \
  1 \
  'Rust-visible item'
for shared_parent_owner in \
  lower_for_in_of_environment \
  lower_for_head_expression_with_tdz \
  for_in_global_target \
  single_statement \
  expr_is_identifier_named \
  static_string_expression \
  plain_async_entry_state \
  lower_loop_body \
  lower_web_compat_loop_assignment_target \
  lower_var_declarator \
  unwrap_parenthesized_expr \
  is_known_non_enumerable_global \
  is_known_non_enumerable_builtin_property
do
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*fn[[:space:]]+${shared_parent_owner}[[:space:]]*[<(]" \
    1 \
    "parent-owned shared ${shared_parent_owner} helper"
  require_regex_count \
    "$ir_for_in_lowering" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${shared_parent_owner}[[:space:]]*[<(]" \
    0 \
    "shared ${shared_parent_owner} helper copied into for-in owner"
done
for shared_free_owner in supported_bound_names for_in_loop_binding_storage_name; do
  require_regex_count \
    "crates/lila-ir/src/lowering_helpers.rs" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${shared_free_owner}[[:space:]]*[<(]" \
    1 \
    "shared ${shared_free_owner} free-helper owner"
  require_regex_count \
    "$ir_for_in_lowering" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${shared_free_owner}[[:space:]]*[<(]" \
    0 \
    "shared ${shared_free_owner} free helper copied into for-in owner"
done
# Boa's generic containment visitor is imported rather than locally owned, but
# copying one into the child would fork the grammar predicate this family uses.
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+contains[[:space:]]*[<(]' \
  0 \
  'generic contains helper copied into for-in owner'
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?(struct|enum|union|type)[[:space:]]+ForInOfEnvironmentIr([[:space:]]|<|\{|\(|=|;|:)' \
  0 \
  'shared for-in/of environment type declaration copied into for-in owner'
require_tree_regex_count \
  "crates/lila-ir/src" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?(struct|enum|union|type)[[:space:]]+ForInOfEnvironmentIr([[:space:]]|<|\{|\(|=|;|:)' \
  1 \
  'shared for-in/of environment type'
check_no_inline_legacy_includes "$ir_for_in_lowering"
# Measured after formatting the extraction: 571 raw lines. The margin is for
# maintenance of this closed family, not unrelated lowering or shared helpers.
check_raw_line_budget "$ir_for_in_lowering" 650
# T02's classic-for boundary owns the complete head/environment/resumption
# lifecycle and the final For/GeneratorLoop choice. The statement dispatcher
# remains its sole caller and the parent cannot regrow a second implementation.
ir_for_loop_lowering="crates/lila-ir/src/lowering/for_loop.rs"
require_file "$ir_for_loop_lowering"
require_module_decl "$ir_lowering" "for_loop"
require_fixed_string_count \
  "$ir_for_loop_lowering" \
  'pub(super) fn lower_for_loop(' \
  1 \
  'classic-for lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_for_loop(' \
  0 \
  'classic-for lowering outside child module'
check_no_inline_legacy_includes "$ir_for_loop_lowering"
# Measured after formatting the extraction: 213 raw lines. The margin is for
# maintenance of the classic-for lifecycle, not unrelated loop lowering.
check_raw_line_budget "$ir_for_loop_lowering" 250
# T02's for-of boundary owns every specialization decision and the
# lowering-only protocol carrier. The statement dispatcher is the sole caller;
# shared loop/environment helpers and public statement/protocol IR remain in
# their existing owners.
ir_for_of_lowering="crates/lila-ir/src/lowering/for_of.rs"
ir_for_of_protocol_lowering="crates/lila-ir/src/lowering/for_of/protocol.rs"
require_file "$ir_for_of_lowering"
require_file "$ir_for_of_protocol_lowering"
require_exact_line_count "$ir_lowering" 'mod for_of;' 1 'private for-of module declaration'
require_exact_line_count \
  "$ir_for_of_lowering" \
  'mod protocol;' \
  1 \
  'private for-of protocol child declaration'
require_fixed_string_count \
  "$ir_for_of_lowering" \
  'pub(super) fn lower_for_of_loop(' \
  1 \
  'for-of statement-facing owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_for_of_loop(' \
  0 \
  'for-of lowering outside child module'
for private_owner in lower_async_function_for_of_iterator_with_body_await lower_for_of_head; do
  require_fixed_string_count \
    "$ir_for_of_lowering" \
    "fn ${private_owner}(" \
    1 \
    "private ${private_owner} owner"
  require_fixed_string_count \
    "$ir_lowering" \
    "fn ${private_owner}(" \
    0 \
    "${private_owner} outside child module"
done
require_fixed_string_count \
  "$ir_for_of_lowering" \
  'enum AsyncForOfArrayWalkForm' \
  0 \
  'retired resumable array-walk classification carrier'
require_fixed_string_count \
  "crates/lila-ir/src/lowering_helpers.rs" \
  'enum AsyncForOfArrayWalkForm' \
  0 \
  'resumable array-walk carrier declarations in the former helper owner'
require_tree_regex_count \
  "crates/lila-ir/src" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?enum[[:space:]]+AsyncForOfArrayWalkForm([[:space:]]|\{)' \
  0 \
  'retired resumable array-walk carrier'
require_fixed_string_count \
  "crates/lila-ir/src/ir.rs" \
  'pub struct AsyncFunctionForOfIteratorPlanIr' \
  1 \
  'resumable synchronous for-of plan owner'
require_fixed_string_count \
  "$ir_for_of_lowering" \
  'struct AsyncFunctionForOfIteratorPlanIr' \
  0 \
  'resumable synchronous for-of plan declarations outside the IR owner'
require_tree_regex_count \
  "crates/lila-ir/src" \
  '^[[:space:]]*pub[[:space:]]+struct[[:space:]]+AsyncFunctionForOfIteratorPlanIr([[:space:]]|\{)' \
  1 \
  'resumable synchronous for-of plan'
require_fixed_string_count \
  "$ir_for_of_protocol_lowering" \
  'pub(super) struct ForOfLoweringIr' \
  1 \
  'private for-of protocol carrier'
require_fixed_string_count \
  "$ir_for_of_lowering" \
  'struct ForOfLoweringIr' \
  0 \
  'for-of protocol carrier declarations outside the protocol child'
require_fixed_string_count \
  "crates/lila-ir/src/ir.rs" \
  'struct ForOfLoweringIr' \
  0 \
  'for-of protocol carrier declarations in the former IR owner'
require_tree_regex_count \
  "crates/lila-ir/src" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+ForOfLoweringIr([[:space:]]|\{)' \
  1 \
  'for-of protocol carrier'
# The statement-facing wrapper is the main for-of module's only Rust-visible
# item. The protocol carrier remains visible only to its private parent module.
require_regex_count \
  "$ir_for_of_lowering" \
  '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' \
  1 \
  'Rust-visible item'
for shared_helper in \
  plain_async_entry_state \
  split_resumable_loop_body \
  lower_loop_body \
  lower_for_in_of_environment \
  lower_for_head_expression_with_tdz \
  for_of_loop_binding_storage_name
do
  require_fixed_string_count \
    "$ir_for_of_lowering" \
    "fn ${shared_helper}(" \
    0 \
    "shared ${shared_helper} helper copied into for-of owner"
done
require_fixed_string_count \
  "$ir_for_of_lowering" \
  'fn generator_loop_has_unsupported_control' \
  0 \
  'shared generic generator_loop_has_unsupported_control helper copied into for-of owner'
require_fixed_string_count \
  "$ir_for_of_lowering" \
  'enum LoweredForOfHeadKind' \
  0 \
  'async-disposable head-kind type copied into for-of owner'
check_no_inline_legacy_includes "$ir_for_of_lowering"
check_no_inline_legacy_includes "$ir_for_of_protocol_lowering"
# Measured after separating the protocol witness carrier: 731 raw lines. The
# margin is for maintenance of the complete for-of lowering family, not
# unrelated lowering.
check_raw_line_budget "$ir_for_of_lowering" 760
# Measured after extraction: 98 raw lines. The child owns only the protocol
# witness carrier and its constructors.
check_raw_line_budget "$ir_for_of_protocol_lowering" 120
# T02's if-statement boundary owns branch lowering, flow-fact joins and the
# generator split/merge lifecycle. The shared static expression helpers remain
# parent-owned and the parent cannot regrow a second if implementation.
ir_if_statement_lowering="crates/lila-ir/src/lowering/if_statement.rs"
require_file "$ir_if_statement_lowering"
require_module_decl "$ir_lowering" "if_statement"
require_fixed_string_count \
  "$ir_if_statement_lowering" \
  'pub(super) fn lower_if_statement(' \
  1 \
  'if-statement lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_if_statement(' \
  0 \
  'if-statement lowering outside child module'
require_fixed_string_count \
  "$ir_if_statement_lowering" \
  'fn split_generator_if_branch(' \
  1 \
  'generator if-branch split helper'
require_fixed_string_count \
  "$ir_lowering" \
  'fn split_generator_if_branch(' \
  0 \
  'generator if-branch split helper outside child module'
require_fixed_string_count \
  "$ir_if_statement_lowering" \
  'fn statement_completes_by_throw(' \
  1 \
  'if-branch abrupt-completion helper'
require_fixed_string_count \
  "$ir_lowering" \
  'fn statement_completes_by_throw(' \
  0 \
  'if-branch abrupt-completion helper outside child module'
require_fixed_string_count \
  "$ir_lowering" \
  'fn static_bool_expr(' \
  1 \
  'shared static boolean helper in parent'
require_fixed_string_count \
  "$ir_if_statement_lowering" \
  'fn static_bool_expr(' \
  0 \
  'static boolean helper copied into if-statement owner'
check_no_inline_legacy_includes "$ir_if_statement_lowering"
# Measured after formatting the extraction: 141 raw lines. The margin is for
# maintenance of if-statement lowering only.
check_raw_line_budget "$ir_if_statement_lowering" 180
# T02's labelled-statement boundary owns nested label collection, target-kind
# classification and final Labelled IR assembly. The active-label stack types
# remain parent-owned because break/continue lowering also consumes them.
ir_labelled_statement_lowering="crates/lila-ir/src/lowering/labelled_statement.rs"
require_file "$ir_labelled_statement_lowering"
require_module_decl "$ir_lowering" "labelled_statement"
require_fixed_string_count \
  "$ir_labelled_statement_lowering" \
  'pub(super) fn lower_labelled(' \
  1 \
  'labelled-statement lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_labelled(' \
  0 \
  'labelled-statement lowering outside child module'
require_fixed_string_count \
  "$ir_labelled_statement_lowering" \
  'fn collect_labels' \
  1 \
  'nested-label collection helper'
require_fixed_string_count \
  "$ir_lowering" \
  'fn collect_labels' \
  0 \
  'nested-label collection helper outside child module'
require_fixed_string_count \
  "$ir_labelled_statement_lowering" \
  'fn label_target_kind' \
  1 \
  'label target-kind helper'
require_fixed_string_count \
  "$ir_lowering" \
  'fn label_target_kind' \
  0 \
  'label target-kind helper outside child module'
require_fixed_string_count "$ir_lowering" 'struct ActiveLabel' 1 'shared active-label type in parent'
require_fixed_string_count "$ir_labelled_statement_lowering" 'struct ActiveLabel' 0 'active-label type copied into labelled owner'
require_fixed_string_count "$ir_lowering" 'enum LabelTargetKind' 1 'shared label-target type in parent'
require_fixed_string_count "$ir_labelled_statement_lowering" 'enum LabelTargetKind' 0 'label-target type copied into labelled owner'
check_no_inline_legacy_includes "$ir_labelled_statement_lowering"
# Measured after formatting the extraction: 72 raw lines. The margin is for
# maintenance of labelled-statement lowering only.
check_raw_line_budget "$ir_labelled_statement_lowering" 100
# T02's abrupt loop-control boundary owns all labelled/unlabelled break and
# continue validation plus final abrupt-control IR assembly. The active-label
# types stay parent-owned because labelled_statement produces them while this
# child consumes them.
ir_break_continue_lowering="crates/lila-ir/src/lowering/break_continue.rs"
require_file "$ir_break_continue_lowering"
require_module_decl "$ir_lowering" "break_continue"
require_fixed_string_count \
  "$ir_break_continue_lowering" \
  'pub(super) fn lower_break(' \
  1 \
  'break lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_break(' \
  0 \
  'break lowering outside child module'
require_fixed_string_count \
  "$ir_break_continue_lowering" \
  'pub(super) fn lower_continue(' \
  1 \
  'continue lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_continue(' \
  0 \
  'continue lowering outside child module'
require_fixed_string_count \
  "$ir_break_continue_lowering" \
  'struct ActiveLabel' \
  0 \
  'active-label type copied into break/continue owner'
require_fixed_string_count \
  "$ir_break_continue_lowering" \
  'enum LabelTargetKind' \
  0 \
  'label-target type copied into break/continue owner'
check_no_inline_legacy_includes "$ir_break_continue_lowering"
# Measured after formatting the extraction: 45 raw lines. The margin is for
# maintenance of break/continue lowering only.
check_raw_line_budget "$ir_break_continue_lowering" 70
# T02's while-family boundary owns ordinary/resumable while lowering and the
# explicit do-while suspension refusal. Shared loop resumption helpers remain
# parent-owned and the parent cannot regrow either loop implementation.
ir_while_loop_lowering="crates/lila-ir/src/lowering/while_loop.rs"
require_file "$ir_while_loop_lowering"
require_module_decl "$ir_lowering" "while_loop"
require_fixed_string_count \
  "$ir_while_loop_lowering" \
  'pub(super) fn lower_while_loop(' \
  1 \
  'while-loop lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_while_loop(' \
  0 \
  'while-loop lowering outside child module'
require_fixed_string_count \
  "$ir_while_loop_lowering" \
  'pub(super) fn lower_do_while_loop(' \
  1 \
  'do-while lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_do_while_loop(' \
  0 \
  'do-while lowering outside child module'
require_fixed_string_count \
  "$ir_lowering" \
  'fn plain_async_entry_state(' \
  1 \
  'shared plain-async loop-state helper in parent'
require_fixed_string_count \
  "$ir_while_loop_lowering" \
  'fn plain_async_entry_state(' \
  0 \
  'plain-async loop-state helper copied into while owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn split_resumable_loop_body(' \
  1 \
  'shared resumable-loop split helper in parent'
require_fixed_string_count \
  "$ir_while_loop_lowering" \
  'fn split_resumable_loop_body(' \
  0 \
  'resumable-loop split helper copied into while owner'
check_no_inline_legacy_includes "$ir_while_loop_lowering"
# Measured after formatting the extraction: 106 raw lines. The margin is for
# maintenance of while/do-while lowering only.
check_raw_line_budget "$ir_while_loop_lowering" 130
# T02's switch-statement boundary owns discriminant and selector evaluation,
# the one shared CaseBlock lexical environment, case-body fact joins and final
# Switch IR assembly. Statement-list and environment materialization helpers
# stay parent-owned because other statement families consume them too.
ir_switch_statement_lowering="crates/lila-ir/src/lowering/switch_statement.rs"
require_file "$ir_switch_statement_lowering"
require_module_decl "$ir_lowering" "switch_statement"
require_fixed_string_count \
  "$ir_switch_statement_lowering" \
  'pub(super) fn lower_switch(' \
  1 \
  'switch-statement lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_switch(' \
  0 \
  'switch-statement lowering outside child module'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_statement_items_without_function_initialization(' \
  1 \
  'shared statement-list helper in parent'
require_fixed_string_count \
  "$ir_switch_statement_lowering" \
  'fn lower_statement_items_without_function_initialization(' \
  0 \
  'statement-list helper copied into switch owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_materialized_lexical_environment(' \
  1 \
  'shared lexical-environment materializer in parent'
require_fixed_string_count \
  "$ir_switch_statement_lowering" \
  'fn lower_materialized_lexical_environment(' \
  0 \
  'lexical-environment materializer copied into switch owner'
check_no_inline_legacy_includes "$ir_switch_statement_lowering"
# Measured after formatting the extraction: 90 raw lines. The margin is for
# maintenance of switch-statement lowering only.
check_raw_line_budget "$ir_switch_statement_lowering" 120
# T02's with-statement boundary owns the full Object Environment lifecycle:
# outer object evaluation, hidden binding materialization, ordered chain entry
# and exit, body lowering and lexical-block assembly. Shared allocation and
# suspension helpers plus reference lifecycle types remain in their owners.
ir_with_statement_lowering="crates/lila-ir/src/lowering/with_statement.rs"
require_file "$ir_with_statement_lowering"
require_module_decl "$ir_lowering" "with_statement"
require_fixed_string_count \
  "$ir_with_statement_lowering" \
  'pub(super) fn lower_with_statement(' \
  1 \
  'with-statement lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_with_statement(' \
  0 \
  'with-statement lowering outside child module'
for shared_helper in object_like_kind_set alloc_temp_binding_name add_suspension_owned_binding; do
  require_fixed_string_count \
    "$ir_with_statement_lowering" \
    "fn ${shared_helper}(" \
    0 \
    "shared ${shared_helper} helper copied into with-statement owner"
done
for reference_type in ObjectEnvironmentBindingObject CurrentScopeDepth OrderedWithEnvironmentChain; do
  require_fixed_string_count \
    "$ir_with_statement_lowering" \
    "struct ${reference_type}" \
    0 \
    "shared ${reference_type} lifecycle type copied into with-statement owner"
done
require_fixed_string_count \
  "$ir_with_statement_lowering" \
  'with_environment_chain.enter_current(' \
  1 \
  'with-environment chain entry'
require_fixed_string_count \
  "$ir_with_statement_lowering" \
  'with_environment_chain.leave_current();' \
  1 \
  'with-environment chain exit'
check_no_inline_legacy_includes "$ir_with_statement_lowering"
# Measured after formatting the extraction: 103 raw lines. The margin is for
# maintenance of with-statement lowering only.
check_raw_line_budget "$ir_with_statement_lowering" 130
# T02's property-access boundary owns ordinary, private and super access
# dispatch plus the primitive/exotic target-kind split. Keep that split
# exhaustive so a future ValueKind cannot silently inherit Number's currently
# unsupported behavior through a catch-all arm.
ir_property_access_lowering="crates/lila-ir/src/lowering/property_access.rs"
require_file "$ir_property_access_lowering"
require_module_decl "$ir_lowering" "property_access"
require_fixed_string_count \
  "$ir_property_access_lowering" \
  'pub(super) fn lower_property_access(' \
  1 \
  'property-access lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_property_access(' \
  0 \
  'property-access lowering outside child module'
require_fixed_string_count \
  "$ir_property_access_lowering" \
  'ValueKind::Number => {' \
  1 \
  'explicit Number property-access arm'
if grep -Fq '_ => self.unsupported_expr("property access on non-object target")' "$ir_property_access_lowering"; then
  fail "$ir_property_access_lowering must exhaust ValueKind instead of hiding future variants behind a catch-all"
fi
check_no_inline_legacy_includes "$ir_property_access_lowering"
# Measured after formatting the extraction: 223 raw lines. The margin is for
# maintenance of property-access lowering only.
check_raw_line_budget "$ir_property_access_lowering" 260
# T02's call-expression boundary keeps the public entry and direct
# identifier/property recognition in one child family. Its private nested child
# owns only the terminal non-property call path. The parent owns expression
# dispatch and reusable helpers, but cannot regrow a second call implementation.
ir_call_expression_lowering="crates/lila-ir/src/lowering/call_expression.rs"
ir_non_property_call_lowering="crates/lila-ir/src/lowering/call_expression/non_property_call.rs"
require_file "$ir_call_expression_lowering"
require_file "$ir_non_property_call_lowering"
require_module_decl "$ir_lowering" "call_expression"
require_exact_line_count \
  "$ir_call_expression_lowering" \
  'mod non_property_call;' \
  1 \
  'private non-property call child declaration'
require_fixed_string_count \
  "$ir_call_expression_lowering" \
  'pub(super) fn lower_call(' \
  1 \
  'call-expression lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_call(' \
  0 \
  'call-expression lowering body outside child module'
require_fixed_string_count \
  "$ir_non_property_call_lowering" \
  'pub(super) fn lower_non_property_call(' \
  1 \
  'non-property call lowering owner'
require_tree_regex_count \
  "crates/lila-ir/src" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+lower_non_property_call[[:space:]]*\(' \
  1 \
  'non-property call lowering owner'
require_fixed_string_count \
  "$ir_call_expression_lowering" \
  'self.lower_non_property_call(callee, args)' \
  1 \
  'non-property call lowering dispatch'
require_tree_regex_count \
  "crates/lila-ir/src" \
  '^[[:space:]]*self\.lower_non_property_call\(callee, args\)' \
  1 \
  'non-property call lowering dispatch'
call_expression_terminal_suffix="$(tail -n 3 "$ir_call_expression_lowering")"
expected_call_expression_terminal_suffix=$'        self.lower_non_property_call(callee, args)\n    }\n}'
if [ "$call_expression_terminal_suffix" != "$expected_call_expression_terminal_suffix" ]; then
  fail "$ir_call_expression_lowering must end by dispatching the non-property call owner"
fi
require_fixed_string_count \
  "$ir_call_expression_lowering" \
  'unsupported_call' \
  0 \
  'unreachable constructor fallback in property calls'
require_fixed_string_count \
  "$ir_non_property_call_lowering" \
  'unsupported_call' \
  0 \
  'unreachable constructor fallback in non-property calls'
check_no_inline_legacy_includes "$ir_call_expression_lowering"
check_no_inline_legacy_includes "$ir_non_property_call_lowering"
# Measured after closing forwarded and conversion effects: 3,070 raw lines. The
# margin is for maintenance of direct identifier/property recognition.
check_raw_line_budget "$ir_call_expression_lowering" 3100
# Measured after extraction: 315 raw lines. This private child owns optional,
# erased, multi-target and exact-target calls after callee-value lowering.
check_raw_line_budget "$ir_non_property_call_lowering" 350
# T02's builtin call-result boundary owns the exhaustive StandardBuiltinId
# result analysis and its narrowly related observation updates. Construct,
# direct-call, RegExp-literal and well-known-symbol routing remain consumers;
# the parent orchestration file cannot regrow a second result table.
ir_builtin_call_info_lowering="crates/lila-ir/src/lowering/builtin_call_info.rs"
require_file "$ir_builtin_call_info_lowering"
require_module_decl "$ir_lowering" "builtin_call_info"
require_fixed_string_count \
  "$ir_builtin_call_info_lowering" \
  'pub(super) fn standard_builtin_call_info(' \
  1 \
  'builtin call-result analysis owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn standard_builtin_call_info(' \
  0 \
  'builtin call-result analysis outside child module'
check_no_inline_legacy_includes "$ir_builtin_call_info_lowering"
# Promise-specific catalog bypasses are a closed policy owned outside the
# exhaustive result table. The caller consumes one policy for executor
# observation and caller-flow invalidation; ad hoc booleans cannot regrow.
ir_promise_caller_flow_lowering="crates/lila-ir/src/lowering/promise_caller_flow.rs"
require_file "$ir_promise_caller_flow_lowering"
require_module_decl "$ir_lowering" "promise_caller_flow"
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+enum[[:space:]]+PromiseInvocationPolicy[[:space:]]*\{' \
  1 \
  'Promise invocation-policy owner'
require_fixed_string_count \
  "$ir_builtin_call_info_lowering" \
  'PromiseInvocationPolicy::for_call(builtin, args, &context)' \
  1 \
  'Promise invocation-policy consumption'
for retired_promise_effect_boolean in \
  promise_constructor_may_invoke_executor \
  promise_builtin_cannot_call_user_code
do
  require_tree_regex_count \
    crates/lila-ir/src \
    "$retired_promise_effect_boolean" \
    0 \
    'retired ad hoc Promise invocation booleans'
done
check_no_inline_legacy_includes "$ir_promise_caller_flow_lowering"
check_raw_line_budget "$ir_promise_caller_flow_lowering" 70
# Invocation-effect accounting is a linear proof owned outside the exhaustive
# result table. There is one private module, one raw unattached-proof producer,
# and no compatibility re-export through the former owner.
ir_invocation_effects_lowering="crates/lila-ir/src/lowering/invocation_effects.rs"
require_file "$ir_invocation_effects_lowering"
require_exact_line_count \
  "$ir_lowering" \
  'mod invocation_effects;' \
  1 \
  'private invocation-effects module declaration'
require_fixed_string_count \
  "$ir_lowering" \
  'pub use invocation_effects::' \
  0 \
  'public invocation-effects compatibility re-exports'
for invocation_effects_consumer in "$ir_lowering" "$ir_builtin_call_info_lowering"; do
  require_regex_count \
    "$invocation_effects_consumer" \
    '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+use[[:space:]]+([^;]*::)?invocation_effects(::|[[:space:]]*;)' \
    0 \
    'visibility-qualified invocation-effects compatibility re-exports'
done
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+AccountedInvocationEffects[[:space:]]*\{' \
  1 \
  'AccountedInvocationEffects owner'
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?enum[[:space:]]+StandardBuiltinCallAnalysis[[:space:]]*\{' \
  1 \
  'StandardBuiltinCallAnalysis owner'
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?enum[[:space:]]+AnalyzedInvocationEffects[[:space:]]*\{' \
  1 \
  'AnalyzedInvocationEffects owner'
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+InvocationCallerFlowEffects\(InvocationCallerFlowState\);' \
  1 \
  'opaque invocation caller-flow owner'
require_regex_count \
  "$ir_builtin_call_info_lowering" \
  '^[[:space:]]*(pub\([^)]*\)[[:space:]]+)?(struct|enum)[[:space:]]+(AccountedInvocationEffects|StandardBuiltinCallAnalysis|AnalyzedInvocationEffects)' \
  0 \
  'invocation-effects declarations in the former owner'
require_fixed_string_count \
  "$ir_invocation_effects_lowering" \
  'attached_to_emitted_call: false,' \
  1 \
  'raw unattached invocation-effects proof producers'
require_fixed_string_count \
  "$ir_invocation_effects_lowering" \
  'pub(super) fn recorded() -> Self {' \
  1 \
  'canonical invocation-effects proof constructor'
require_fixed_string_count \
  "$ir_invocation_effects_lowering" \
  'impl Drop for AccountedInvocationEffects {' \
  1 \
  'unconsumed invocation-effects rejection boundary'
if grep -Eq '#\[derive\([^]]*(Clone|Copy)' "$ir_invocation_effects_lowering" \
  || grep -Eq 'impl[[:space:]]+(Clone|Copy)[[:space:]]+for[[:space:]]+AccountedInvocationEffects' "$ir_invocation_effects_lowering"; then
  fail "$ir_invocation_effects_lowering must keep AccountedInvocationEffects nonduplicable"
fi
check_no_inline_legacy_includes "$ir_invocation_effects_lowering"
# Measured after moving indexed-receiver mutation into catalog metadata: 2,225
# raw lines.
# The margin is for maintenance of this exhaustive result table, not unrelated
# lowering.
check_raw_line_budget "$ir_builtin_call_info_lowering" 2250
# Measured after adding the opaque source/host caller-flow aggregate: 192 raw
# lines. This owner must remain a bounded lifecycle, not become a second
# call-analysis implementation store.
check_raw_line_budget "$ir_invocation_effects_lowering" 210
# Argument evaluation carries one must-consume authority. Raw predecessor
# snapshot variants remain private to its consumption methods so call owners
# cannot partially apply the invalidation result.
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*struct[[:space:]]+LoweredCallArguments[[:space:]]*\{' \
  1 \
  'lowered-call-argument authority owner'
require_exact_line_count \
  "$ir_lowering" \
  '#[must_use = "lowered call arguments must account for values captured before their evaluation"]' \
  1 \
  'lowered-call-argument must-use contract'
for predecessor_snapshot_variant in \
  'PreArgumentHeapShapeSnapshots::NoHeapShapes 1' \
  'PreArgumentHeapShapeSnapshots::OneHeapShape 2' \
  'PreArgumentHeapShapeSnapshots::TwoHeapShapes 2'
do
  set -- $predecessor_snapshot_variant
  require_fixed_string_count \
    "$ir_lowering" \
    "$1" \
    "$2" \
    'private predecessor snapshot consumption'
done
# Source-call caller-flow preservation is admitted only by an exhaustive walk
# over the finalized parameters and IR body. The public-to-lowering carrier
# hides its state; only the nonduplicable proof token can mint the proven-safe
# state.
ir_source_call_flow_proof="crates/lila-ir/src/source_call_flow_proof.rs"
require_file "$ir_source_call_flow_proof"
require_exact_line_count \
  "$ir_lib" \
  'mod source_call_flow_proof;' \
  1 \
  'private source-call flow-proof module declaration'
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*pub\(crate\)[[:space:]]+struct[[:space:]]+SourceCallFlowEffects\(SourceCallFlowState\);' \
  1 \
  'opaque source-call flow state owner'
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*pub\(crate\)[[:space:]]+struct[[:space:]]+ProvenNoCallerFlowInvalidation[[:space:]]*\{' \
  1 \
  'caller-flow proof token owner'
source_call_flow_proof_token="$(sed -n '/#\[must_use = "caller-flow preservation must be consumed by source-call admission"\]/,/^}/p' "$ir_source_call_flow_proof")"
require_text_regex_count \
  "$source_call_flow_proof_token" \
  '^#\[must_use = "caller-flow preservation must be consumed by source-call admission"\]$' \
  1 \
  'caller-flow proof consumption obligation'
require_text_regex_count \
  "$source_call_flow_proof_token" \
  '^[[:space:]]*_private: \(\),$' \
  1 \
  'private caller-flow proof constructor field'
if grep -Eq '#\[derive\([^]]*(Clone|Copy)' <<<"$source_call_flow_proof_token" \
  || grep -Eq 'impl[[:space:]]+(Clone|Copy)[[:space:]]+for[[:space:]]+ProvenNoCallerFlowInvalidation' "$ir_source_call_flow_proof"; then
  fail "$ir_source_call_flow_proof must keep ProvenNoCallerFlowInvalidation nonduplicable"
fi
require_fixed_string_count \
  "$ir_source_call_flow_proof" \
  'Self(SourceCallFlowState::ProvenNoFlowInvalidation)' \
  2 \
  'proof-preserving source-call state constructors'
require_fixed_string_count \
  "$ir_source_call_flow_proof" \
  'Some(proof) => Self::from_proof(proof),' \
  1 \
  'finalized-invocation proof admission'
require_fixed_string_count \
  "$ir_source_call_flow_proof" \
  'pub(crate) fn for_finalized_invocation(params: &[FunctionParamIr], body: &BlockIr) -> Self {' \
  1 \
  'parameter-and-body source-call proof boundary'
require_fixed_string_count \
  "$ir_source_call_flow_proof" \
  'for_finalized_body' \
  0 \
  'body-only source-call proof admission'
for source_call_flow_variant_spec in 'StatementIr|34' 'ExprIr|83' 'SpecOperationIr|29'; do
  source_call_flow_variant_domain="${source_call_flow_variant_spec%%|*}"
  expected_source_call_flow_variants="${source_call_flow_variant_spec#*|}"
  observed_source_call_flow_variants="$({
    grep -oE "${source_call_flow_variant_domain}::[A-Za-z0-9_]+" "$ir_source_call_flow_proof" || true
  } | sort -u | wc -l | tr -d ' ')"
  if [ "$observed_source_call_flow_variants" != "$expected_source_call_flow_variants" ]; then
    fail "$ir_source_call_flow_proof must exhaust ${expected_source_call_flow_variants} ${source_call_flow_variant_domain} variants (found $observed_source_call_flow_variants)"
  fi
done
if grep -Eq '(^|[|,(])[[:space:]]*_[[:space:]]*=>' "$ir_source_call_flow_proof" \
  || grep -Eq '\{[[:space:]]*\.\.[[:space:]]*\}' "$ir_source_call_flow_proof"; then
  fail "$ir_source_call_flow_proof must not hide new IR variants behind catch-all patterns"
fi
check_no_inline_legacy_includes "$ir_source_call_flow_proof"
# Measured after including parameter-default execution: 769 raw lines. The
# margin is only for exhaustive IR variants and focused proof tests.
check_raw_line_budget "$ir_source_call_flow_proof" 800
# T02's class-definition boundary keeps the complete element planning,
# generated-function scheduling and typed ClassDefinitionIr construction in
# one child module. The parent retains only declaration/expression
# orchestration and cannot regrow a second copy of the implementation.
ir_class_definition_lowering="crates/lila-ir/src/lowering/class_definition.rs"
require_file "$ir_class_definition_lowering"
require_module_decl "$ir_lowering" "class_definition"
require_fixed_string_count \
  "$ir_class_definition_lowering" \
  'pub(super) fn lower_class_common_in_name_scope(' \
  1 \
  'class-definition lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_class_common_in_name_scope(' \
  0 \
  'class-definition lowering body outside child module'
check_no_inline_legacy_includes "$ir_class_definition_lowering"
# Measured after constructor invocation-effect finalization: 1,458 raw lines.
# The margin is for maintenance of this class-definition family, not unrelated
# lowering.
check_raw_line_budget "$ir_class_definition_lowering" 1500
# T02's ordinary-function boundary keeps the nested-lowerer lifecycle,
# parameter/body lowering, capture transfer, signature updates and final
# FunctionIr assembly together. The parent owns the seven orchestration calls
# and shared helpers used by generated iterators, class methods and object
# methods, but cannot regrow a second ordinary-function implementation.
ir_function_definition_lowering="crates/lila-ir/src/lowering/function_definition.rs"
require_file "$ir_function_definition_lowering"
require_module_decl "$ir_lowering" "function_definition"
require_fixed_string_count \
  "$ir_function_definition_lowering" \
  'pub(super) fn lower_function(' \
  1 \
  'ordinary-function lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_function(' \
  0 \
  'ordinary-function lowering outside child module'
check_no_inline_legacy_includes "$ir_function_definition_lowering"
# Measured after formatting the extraction: 721 raw lines. The margin is for
# maintenance of this lifecycle, not unrelated lowering.
check_raw_line_budget "$ir_function_definition_lowering" 780
# T02's try-statement boundary owns catch-parameter environment construction,
# resumable entry/exit planning and final TryCatch/TryFinally IR assembly. The
# catch and finally lifecycle records are named so their generator and async
# states cannot be transposed through positional tuple access.
ir_try_statement_lowering="crates/lila-ir/src/lowering/try_statement.rs"
require_file "$ir_try_statement_lowering"
require_module_decl "$ir_lowering" "try_statement"
require_fixed_string_count \
  "$ir_try_statement_lowering" \
  'pub(super) fn lower_try(' \
  1 \
  'try-statement lowering owner'
require_fixed_string_count \
  "$ir_lowering" \
  'fn lower_try(' \
  0 \
  'try-statement lowering outside child module'
if ! grep -q '^struct LoweredCatchClause {' "$ir_try_statement_lowering" \
  || ! grep -q '^struct LoweredFinallyClause {' "$ir_try_statement_lowering"; then
  fail "$ir_try_statement_lowering must carry named catch/finally lifecycle records"
fi
if sed '/^[[:space:]]*\/\//d' "$ir_try_statement_lowering" | grep -Eq '\.[0-9]+'; then
  fail "$ir_try_statement_lowering must not recover lifecycle state through tuple positions"
fi
check_no_inline_legacy_includes "$ir_try_statement_lowering"
# Measured after formatting the extraction and named-record cleanup: 264 raw
# lines. The margin is for maintenance of try-statement lowering only.
check_raw_line_budget "$ir_try_statement_lowering" 320
# T02's throw-inference boundary owns the complete recursive statement,
# expression and property-key analysis closure. Only block inference crosses
# the private boundary for try-statement lowering; shared value/type algebra
# remains with its existing owners.
ir_throw_inference_lowering="crates/lila-ir/src/lowering/throw_inference.rs"
require_file "$ir_throw_inference_lowering"
require_exact_line_count \
  "$ir_lowering" \
  'mod throw_inference;' \
  1 \
  'private throw-inference module declaration'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+infer_block_throw_info[[:space:]]*\(' \
  1 \
  'block throw-inference entry point'
for private_owner in \
  merge_optional_value_info \
  infer_statement_throw_info \
  infer_expr_throw_info \
  infer_expr_operand_throw_info \
  infer_property_key_throw_info
do
  require_regex_count \
    "$ir_throw_inference_lowering" \
    "^[[:space:]]*fn[[:space:]]+${private_owner}[[:space:]]*[<(]" \
    1 \
    "private ${private_owner} owner"
done
for throw_owner in \
  merge_optional_value_info \
  infer_block_throw_info \
  infer_statement_throw_info \
  infer_expr_throw_info \
  infer_expr_operand_throw_info \
  infer_property_key_throw_info
do
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${throw_owner}[[:space:]]*[<(]" \
    0 \
    "${throw_owner} outside throw-inference child"
done
# Five private methods plus the reviewed pub(super) entry point are the entire
# child method surface. Count modifier-qualified declarations as well, so a
# const/async/unsafe/extern/default helper cannot evade the closed owner set.
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*fn[[:space:]]+' \
  5 \
  'private method'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*[<(]' \
  6 \
  'total function declaration'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' \
  1 \
  'Rust-visible item'
require_fixed_string_count \
  "$ir_try_statement_lowering" \
  'infer_block_throw_info' \
  1 \
  'throw-inference identifier use'
require_fixed_string_count \
  "$ir_lowering" \
  'infer_block_throw_info' \
  0 \
  'throw-inference identifier use outside child module'
while IFS= read -r caller; do
  case "$caller" in
    "$ir_throw_inference_lowering"|"$ir_try_statement_lowering") continue ;;
  esac
  if grep -Fq 'infer_block_throw_info' "$caller"; then
    fail "unexpected infer_block_throw_info identifier use: $caller"
  fi
done < <(find crates/lila-ir/src/lowering -type f -name '*.rs' -print)
for shared_parent_owner in \
  merge_value_infos \
  resolve_single_function_target \
  object_like_kind_set
do
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${shared_parent_owner}[[:space:]]*[<(]" \
    1 \
    "parent-owned shared ${shared_parent_owner} helper"
  require_regex_count \
    "$ir_throw_inference_lowering" \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${shared_parent_owner}[[:space:]]*[<(]" \
    0 \
    "shared ${shared_parent_owner} helper copied into throw-inference owner"
done
require_regex_count \
  "$ir_lowering" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+unknown_runtime_value_info[[:space:]]*[<(]' \
  1 \
  'parent-owned unknown-runtime helper'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+unknown_runtime_value_info[[:space:]]*[<(]' \
  0 \
  'unknown-runtime helper copied into throw-inference owner'
for shared_owner_spec in \
  'crates/lila-ir/src/lowering/builtin_shapes.rs:standard_error_instance_info' \
  'crates/lila-ir/src/reference.rs:carried_put_value_failure'
do
  shared_owner_file="${shared_owner_spec%%:*}"
  shared_owner_name="${shared_owner_spec##*:}"
  require_regex_count \
    "$shared_owner_file" \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${shared_owner_name}[[:space:]]*[<(]" \
    1 \
    "shared ${shared_owner_name} helper owner"
  require_regex_count \
    "$ir_throw_inference_lowering" \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${shared_owner_name}[[:space:]]*[<(]" \
    0 \
    "shared ${shared_owner_name} helper copied into throw-inference owner"
done
require_regex_count \
  "crates/lila-ir/src/reference.rs" \
  '^[[:space:]]*pub[[:space:]]+enum[[:space:]]+PutValueFailure([[:space:]]|<|\{|\(|=|;|:)' \
  1 \
  'closed PutValueFailure owner'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?((unsafe|auto)[[:space:]]+)*(struct|enum|union|type|trait)[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*([[:space:]]|<|\{|\(|=|;|:)' \
  0 \
  'local type or trait declaration'
# This owner needs no textual runtime data. Keeping non-line-comment block
# comments and string literals out also makes the executable shape checks below
# immune to code-looking text in /* ... */ or raw/multiline strings.
throw_non_line_comment_source="$(sed '/^[[:space:]]*\/\//d' "$ir_throw_inference_lowering")"
if printf '%s\n' "$throw_non_line_comment_source" | grep -Eq '/\*|\*/|"'; then
  fail "$ir_throw_inference_lowering must express throw analysis through typed values, not block comments or string literals"
fi
# A catch-all silently converts the closed IR/type algebra back into an open
# domain. Inspect each match-arm prefix, including inline and parenthesized
# arms, and every top-level or-pattern alternative. Guarded bindings are not
# catch-alls; `true` and `false` are exhaustive values rather than bindings.
throw_catch_all_arms="$(awk '
  function trim(value) {
    sub(/^[[:space:]]+/, "", value)
    sub(/[[:space:]]+$/, "", value)
    return value
  }
  function unparen(atom) {
    atom = trim(atom)
    while (atom ~ /^\(.*\)$/) {
      atom = trim(substr(atom, 2, length(atom) - 2))
    }
    return atom
  }
  function is_binding(atom) {
    atom = unparen(atom)
    if (atom == "true" || atom == "false") {
      return 0
    }
    return atom ~ /^(&[[:space:]]*(mut[[:space:]]+)?)?((ref[[:space:]]+mut|ref|mut)[[:space:]]+)?(r#)?[a-z][[:alnum:]_]*$/
  }
  function is_simple_catch_all(atom) {
    atom = unparen(atom)
    if (atom == "_" || atom == "..") {
      return 1
    }
    return is_binding(atom)
  }
  function is_catch_all(atom, inner, parts, count, part_index, at, binding, pattern) {
    atom = trim(atom)
    if (atom ~ /^\(.*\)$/) {
      inner = trim(substr(atom, 2, length(atom) - 2))
      if (inner ~ /,/) {
        count = split(inner, parts, /,/)
        for (part_index = 1; part_index <= count; part_index += 1) {
          if (!is_simple_catch_all(parts[part_index])) {
            return 0
          }
        }
        return count > 1
      }
      atom = inner
    }
    if ((at = index(atom, "@")) != 0) {
      binding = trim(substr(atom, 1, at - 1))
      pattern = unparen(substr(atom, at + 1))
      if (!is_binding(binding)) {
        return 0
      }
      count = split(pattern, parts, /\|/)
      for (part_index = 1; part_index <= count; part_index += 1) {
        if (is_simple_catch_all(parts[part_index])) {
          return 1
        }
      }
      return 0
    }
    count = split(atom, parts, /\|/)
    for (part_index = 1; part_index <= count; part_index += 1) {
      if (is_simple_catch_all(parts[part_index])) {
        return 1
      }
    }
    return 0
  }
  {
    rest = $0
    sub(/[[:space:]]*\/\/.*$/, "", rest)
    while ((arrow = index(rest, "=>")) != 0) {
      prefix = substr(rest, 1, arrow - 1)
      start = 0
      paren_depth = 0
      bracket_depth = 0
      brace_depth = 0
      for (i = length(prefix); i >= 1; i -= 1) {
        delimiter = substr(prefix, i, 1)
        if (delimiter == ")") {
          paren_depth += 1
        } else if (delimiter == "(" && paren_depth > 0) {
          paren_depth -= 1
        } else if (delimiter == "]") {
          bracket_depth += 1
        } else if (delimiter == "[" && bracket_depth > 0) {
          bracket_depth -= 1
        } else if (delimiter == "}") {
          brace_depth += 1
        } else if (delimiter == "{" && brace_depth > 0) {
          brace_depth -= 1
        } else if (paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 \
            && (delimiter == "{" || delimiter == ",")) {
          start = i
          break
        }
      }
      arm = trim(substr(prefix, start + 1))
      if (arm !~ /[[:space:]]if[[:space:]]/ && is_catch_all(arm)) {
        print NR ":" arm
      }
      rest = substr(rest, arrow + 2)
    }
  }
' "$ir_throw_inference_lowering")"
if [ -n "$throw_catch_all_arms" ]; then
  fail "$ir_throw_inference_lowering must keep every match exhaustive without catch-all arms"
fi
require_fixed_string_count \
  "$ir_throw_inference_lowering" \
  'unreachable!' \
  0 \
  'unreachable escape hatch'
require_fixed_string_count \
  "$ir_throw_inference_lowering" \
  'macro_rules!' \
  0 \
  'local macro definition'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^(    )?(::[[:space:]]*)?((r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*::[[:space:]]*)*(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*!' \
  0 \
  'module-or-impl-level generated helper invocation'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*let[[:space:]]+strict_put_value_throw[[:space:]]*=[[:space:]]*match[[:space:]]+carried_put_value_failure\(&expr\.expr\)[[:space:]]*\{' \
  1 \
  'executable carried PutValue failure read'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*self\.infer_expr_operand_throw_info\(expr\),[[:space:]]*$' \
  1 \
  'executable wrapper-to-operand delegation'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*PutValueFailure::TypeErrorOnly[[:space:]]*=>[[:space:]]*type_error,[[:space:]]*$' \
  1 \
  'executable TypeError-only PutValue arm'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*PutValueFailure::TypeErrorOrReferenceError[[:space:]]*=>[[:space:]]*self\.merge_value_infos\([[:space:]]*$' \
  1 \
  'executable TypeError-or-ReferenceError PutValue arm'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*Some\(\(Strictness::Sloppy, _\)\)[[:space:]]*\|[[:space:]]*None[[:space:]]*=>[[:space:]]*None,[[:space:]]*$' \
  1 \
  'executable sloppy-or-absent PutValue arm'
check_no_inline_legacy_includes "$ir_throw_inference_lowering"
# Measured after formatting the extraction: 895 raw lines. The margin is for
# maintenance of this closed recursive analysis owner, not general lowering.
check_raw_line_budget "$ir_throw_inference_lowering" 950
# T02's static-JSON parse boundary owns the ordered static-reviver protocol,
# its prepared-value proof and the complete private parser. Dynamic reviver
# target discovery/observation remains in the parent because the ordinary
# JSON.parse path consumes it too.
ir_static_json_parse_lowering="crates/lila-ir/src/lowering/static_json_parse.rs"
ir_static_string_binding_facts_lowering="crates/lila-ir/src/lowering/static_string_binding_facts.rs"
require_file "$ir_static_json_parse_lowering"
require_file "$ir_static_string_binding_facts_lowering"
require_exact_line_count \
  "$ir_lowering" \
  'mod static_json_parse;' \
  1 \
  'private static-JSON parse module declaration'
require_regex_count \
  "$ir_lowering" \
  '^(pub(\([^)]*\))?[[:space:]]+)?mod[[:space:]]+static_json_parse;' \
  1 \
  'total static-JSON parse module declaration'
static_json_module_context="$(sed -n '/^mod statement;$/,+3p' "$ir_lowering")"
if [ "$static_json_module_context" != $'mod statement;\nmod static_json_parse;\nmod static_string_binding_facts;\nmod super_property_mutation;' ]; then
  fail "$ir_lowering must keep static_json_parse and static_string_binding_facts as private module declarations between statement and super_property_mutation"
fi
# Static String facts are keyed by the binding storage identity, never the
# source spelling. The child-private raw map makes it impossible for the parent
# or a sibling lowerer to insert or query a fact without a BindingInfo proof.
require_exact_line_count \
  "$ir_lowering" \
  'use static_string_binding_facts::StaticStringBindingFacts;' \
  1 \
  'narrow static-string fact owner import'
require_exact_line_count \
  "$ir_lowering" \
  '    static_string_bindings: StaticStringBindingFacts,' \
  2 \
  'binding-owned static-string flow fields'
require_tree_regex_count \
  'crates/lila-ir/src' \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+StaticStringBindingFacts[[:space:]]*\{' \
  1 \
  'sole StaticStringBindingFacts owner'
require_exact_line_count \
  "$ir_static_string_binding_facts_lowering" \
  '    by_storage_name: BTreeMap<String, String>,' \
  1 \
  'child-private storage-identity fact map'
for static_string_fact_method in get insert remove clear equal_intersection; do
  require_regex_count \
    "$ir_static_string_binding_facts_lowering" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${static_string_fact_method}[[:space:]]*[<(]" \
    1 \
    "StaticStringBindingFacts::${static_string_fact_method} owner"
done
require_regex_count \
  "$ir_static_string_binding_facts_lowering" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+' \
  5 \
  'closed static-string fact operation inventory'
require_exact_line_count \
  "$ir_static_string_binding_facts_lowering" \
  '    pub(super) fn get(&self, binding: &BindingInfo) -> Option<&String> {' \
  1 \
  'binding-owned static-string fact read'
require_exact_line_count \
  "$ir_static_string_binding_facts_lowering" \
  '    pub(super) fn insert(&mut self, binding: &BindingInfo, value: String) {' \
  1 \
  'binding-owned static-string fact write'
require_exact_line_count \
  "$ir_static_string_binding_facts_lowering" \
  '    pub(super) fn remove(&mut self, binding: &BindingInfo) {' \
  1 \
  'binding-owned static-string fact invalidation'
while IFS= read -r static_string_fact_consumer; do
  if [ "$static_string_fact_consumer" != "$ir_static_string_binding_facts_lowering" ] \
    && grep -Fq '.by_storage_name' "$static_string_fact_consumer"; then
    fail "raw static-string storage map escaped into $static_string_fact_consumer"
  fi
done < <(find crates/lila-ir/src -type f -name '*.rs' -print)
check_no_inline_legacy_includes "$ir_static_string_binding_facts_lowering"
# Measured after replacing source-spelling keys with binding-owned storage
# identities: 33 raw lines. The margin is for the fact lifecycle only.
check_raw_line_budget "$ir_static_string_binding_facts_lowering" 45
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  'use super::*;' \
  1 \
  'parent import'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+prepare_static_json_parse_reviver[[:space:]]*\(' \
  1 \
  'static JSON.parse preparation entry point'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+finish_static_json_parse_reviver[[:space:]]*\(' \
  1 \
  'static JSON.parse finishing entry point'
for moved_owner in prepare_static_json_parse_reviver finish_static_json_parse_reviver; do
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${moved_owner}[[:space:]]*[<(]" \
    0 \
    "${moved_owner} outside static-JSON parse child"
  require_tree_regex_count \
    'crates/lila-ir/src' \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${moved_owner}[[:space:]]*[<(]" \
    1 \
    "sole ${moved_owner} owner"
done
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+PreparedStaticJsonParseReviver[[:space:]]*\{' \
  1 \
  'prepared static JSON.parse proof owner'
require_tree_regex_count \
  'crates/lila-ir/src' \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+PreparedStaticJsonParseReviver[[:space:]]*\{' \
  1 \
  'sole prepared static JSON.parse proof owner'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '#[must_use = "a prepared static JSON.parse reviver must be emitted or rejected"]' \
  1 \
  'prepared static JSON.parse must-use contract'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '    parsed_value: JsonStaticValueIr,' \
  1 \
  'private prepared static JSON.parse payload'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  "^[[:space:]]*struct[[:space:]]+JsonStaticParser<'a>[[:space:]]*\\{" \
  1 \
  'private static-JSON parser type'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  "^[[:space:]]*impl<'a>[[:space:]]+JsonStaticParser<'a>[[:space:]]*\\{" \
  1 \
  'private static-JSON parser implementation'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  "^[[:space:]]*impl<'a>[[:space:]]+ScriptLowerer<'a>[[:space:]]*\\{" \
  1 \
  'ScriptLowerer static-JSON implementation'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*((default|const|unsafe)[[:space:]]+)*impl([[:space:]]|<)' \
  2 \
  'total inherent implementation block'
static_json_lowerer_impl="$(sed -n "/^impl<'a> ScriptLowerer<'a> {$/,/^}$/p" "$ir_static_json_parse_lowering")"
static_json_parser_impl="$(sed -n "/^impl<'a> JsonStaticParser<'a> {$/,/^}$/p" "$ir_static_json_parse_lowering")"
require_text_regex_count \
  "$static_json_lowerer_impl" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+prepare_static_json_parse_reviver[[:space:]]*\(' \
  1 \
  'ScriptLowerer preparation entry point'
require_text_regex_count \
  "$static_json_lowerer_impl" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+finish_static_json_parse_reviver[[:space:]]*\(' \
  1 \
  'ScriptLowerer finishing entry point'
require_text_regex_count \
  "$static_json_lowerer_impl" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*[<(]' \
  2 \
  'total ScriptLowerer function declaration'
for parser_method in \
  new \
  parse \
  parse_value \
  parse_array \
  parse_object \
  parse_string_literal \
  parse_number \
  consume_keyword \
  consume_byte \
  peek_byte \
  skip_ws
do
  require_text_regex_count \
    "$static_json_parser_impl" \
    "^[[:space:]]*fn[[:space:]]+${parser_method}[[:space:]]*[<(]" \
    1 \
    "private JsonStaticParser::${parser_method} method"
done
require_text_regex_count \
  "$static_json_parser_impl" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*[<(]' \
  11 \
  'total JsonStaticParser function declaration'
# Eleven parser methods are private; the prepared proof and two protocol
# methods are the entire Rust-visible child surface. The modifier-aware total
# prevents const/async/unsafe/extern/default additions from evading the closed
# inventory.
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*fn[[:space:]]+' \
  11 \
  'private function'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*[<(]' \
  13 \
  'total function declaration'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' \
  3 \
  'Rust-visible item'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?((unsafe|auto)[[:space:]]+)*(struct|enum|union|type|trait)[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*([[:space:]]|<|\{|\(|=|;|:)' \
  2 \
  'local type or trait declaration'
require_regex_count \
  "$ir_lowering" \
  "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?struct[[:space:]]+JsonStaticParser([[:space:]]|<|\\{|\\(|=|;|:)" \
  0 \
  'static-JSON parser type outside child'
require_tree_regex_count \
  'crates/lila-ir/src' \
  "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?struct[[:space:]]+JsonStaticParser([[:space:]]|<|\\{|\\(|=|;|:)" \
  1 \
  'static-JSON parser type owner'
require_tree_regex_count \
  'crates/lila-ir/src' \
  "^[[:space:]]*impl<'a>[[:space:]]+JsonStaticParser<'a>[[:space:]]*\\{" \
  1 \
  'static-JSON parser implementation owner'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?enum[[:space:]]+JsonStaticValueIr([[:space:]]|<|\{|\(|=|;|:)' \
  0 \
  'JsonStaticValueIr type copied into static-JSON parse child'
require_tree_regex_count \
  'crates/lila-ir/src' \
  '^[[:space:]]*pub[[:space:]]+enum[[:space:]]+JsonStaticValueIr([[:space:]]|<|\{|\(|=|;|:)' \
  1 \
  'shared JsonStaticValueIr owner'
for retained_owner in known_json_parse_reviver_targets observe_json_parse_reviver_targets; do
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*fn[[:space:]]+${retained_owner}[[:space:]]*[<(]" \
    1 \
    "parent-owned ${retained_owner} helper"
  require_regex_count \
    "$ir_static_json_parse_lowering" \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${retained_owner}[[:space:]]*[<(]" \
    0 \
    "${retained_owner} helper copied into static-JSON parse child"
  require_tree_regex_count \
    'crates/lila-ir/src' \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${retained_owner}[[:space:]]*[<(]" \
    1 \
    "${retained_owner} helper owner"
done
require_exact_line_count \
  "$ir_lowering" \
  '            let reviver_targets = self.known_json_parse_reviver_targets(&lowered_args);' \
  1 \
  'dynamic JSON.parse target discovery'
require_exact_line_count \
  "$ir_lowering" \
  '            self.observe_json_parse_reviver_targets(reviver_targets, &helper_context_id);' \
  1 \
  'dynamic JSON.parse target observation'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '        let input = self.static_string_expression(&arguments[0]).or_else(|| {' \
  1 \
  'pre-argument static JSON.parse input recovery'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '            if !self.with_environment_chain.is_empty() {' \
  1 \
  'with-environment static-input rejection'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '            self.static_string_bindings.get(&binding).cloned()' \
  1 \
  'binding-owned ordinary identifier static-input recovery'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '        let parsed_value = JsonStaticParser::new(&input).parse()?;' \
  1 \
  'static JSON.parse parser invocation'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '        if self.known_json_parse_reviver_targets(arguments).is_empty() {' \
  1 \
  'static JSON.parse known-target proof'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '                callee: Box::new(callee.clone()),' \
  1 \
  'static JSON.parse callee acquisition operand'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '                input: Box::new(input.clone()),' \
  1 \
  'static JSON.parse input evaluation operand'
require_fixed_string_count \
  "$ir_static_json_parse_lowering" \
  'observe_json_parse_reviver_targets' \
  0 \
  'dynamic target observation copied into static-JSON parse child'
require_exact_line_count \
  "$ir_call_expression_lowering" \
  '                        self.prepare_static_json_parse_reviver(&function_id, source_arguments);' \
  1 \
  'direct-function static JSON.parse preparation call'
require_exact_line_count \
  "$ir_non_property_call_lowering" \
  '            self.prepare_static_json_parse_reviver(&function_id, args);' \
  1 \
  'effective-function static JSON.parse preparation call'
for static_json_caller in "$ir_call_expression_lowering" "$ir_non_property_call_lowering"; do
  require_fixed_string_count \
    "$static_json_caller" \
    'prepare_static_json_parse_reviver' \
    1 \
    'static JSON.parse preparation use'
  require_fixed_string_count \
    "$static_json_caller" \
    'finish_static_json_parse_reviver' \
    1 \
    'static JSON.parse finishing use'
  static_json_prepare_line="$(grep -nF 'prepare_static_json_parse_reviver' "$static_json_caller" | cut -d: -f1 || true)"
  static_json_finish_line="$(grep -nF 'finish_static_json_parse_reviver' "$static_json_caller" | cut -d: -f1 || true)"
  if [ -n "$static_json_prepare_line" ] \
    && [ -n "$static_json_finish_line" ] \
    && [ "$static_json_prepare_line" -lt "$static_json_finish_line" ]; then
    static_json_ordered_slice="$(sed -n "${static_json_prepare_line},${static_json_finish_line}p" "$static_json_caller")"
    require_text_regex_count \
      "$static_json_ordered_slice" \
      'lower_call_args(_with_target)?[[:space:]]*\(' \
      1 \
      'argument lowering between static JSON.parse preparation and finishing'
  else
    fail "$static_json_caller must prepare static JSON.parse before finishing it"
  fi
done
for static_json_protocol_method in prepare_static_json_parse_reviver finish_static_json_parse_reviver; do
  require_fixed_string_count \
    "$ir_lowering" \
    "$static_json_protocol_method" \
    0 \
    "static JSON.parse ${static_json_protocol_method} use outside child module"
done
while IFS= read -r caller; do
  case "$caller" in
    "$ir_static_json_parse_lowering"|"$ir_call_expression_lowering"|"$ir_non_property_call_lowering") continue ;;
  esac
  for static_json_protocol_method in prepare_static_json_parse_reviver finish_static_json_parse_reviver; do
    if grep -Fq "$static_json_protocol_method" "$caller"; then
      fail "unexpected static JSON.parse protocol use in $caller: $static_json_protocol_method"
    fi
  done
done < <(find crates/lila-ir/src/lowering -type f -name '*.rs' -print)
require_fixed_string_count \
  "$ir_static_json_parse_lowering" \
  'macro_rules!' \
  0 \
  'local macro definition'
static_json_compact_source="$(tr -d '[:space:]' < "$ir_static_json_parse_lowering")"
case "$static_json_compact_source" in
  *macro_rules\!*) fail "$ir_static_json_parse_lowering must not contain a whitespace-split macro_rules definition" ;;
esac
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]{0,7}(::[[:space:]]*)?((r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*::[[:space:]]*)*(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*!' \
  0 \
  'module-or-impl-level generated helper invocation'
check_no_inline_legacy_includes "$ir_static_json_parse_lowering"
# Measured after making the ordered operand and binding-lifecycle proofs
# explicit: 295 raw lines. The margin is for maintenance of static JSON parsing
# only, not dynamic target analysis.
check_raw_line_budget "$ir_static_json_parse_lowering" 315
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

# T15's array-destructuring result/declaration semantics are a closed operation
# carried from five lowering contexts into six semantic consumers. Generic IR
# visitors that only transport the field are deliberately outside this census.
braced_rust_item_source() {
  source_file="$1"
  item_start_pattern="$2"
  awk -v item_start_pattern="$item_start_pattern" '
    !capturing {
      trimmed_line = $0
      sub(/^[[:space:]]*/, "", trimmed_line)
      if (match(trimmed_line, item_start_pattern) != 1) {
        next
      }
      capturing = 1
    }
    {
      print
      opening_line = $0
      closing_line = $0
      openings = gsub(/\{/, "", opening_line)
      closings = gsub(/\}/, "", closing_line)
      depth += openings - closings
      if (openings > 0) {
        body_started = 1
      }
      if (body_started && depth == 0) {
        exit
      }
    }
  ' "$source_file"
}

rust_item_attributes_source() {
  source_file="$1"
  item_start_pattern="$2"
  awk -v item_start_pattern="$item_start_pattern" '
    {
      trimmed_line = $0
      sub(/^[[:space:]]*/, "", trimmed_line)
    }
    trimmed_line ~ /^#\[/ {
      attributes = attributes trimmed_line "\n"
      next
    }
    match(trimmed_line, item_start_pattern) == 1 {
      printf "%s", attributes
      exit
    }
    trimmed_line == "" || trimmed_line ~ /^\/\// { next }
    { attributes = "" }
  ' "$source_file"
}

rust_source_without_inline_tests() {
  awk '
    skipping_test_item {
      opening_line = $0
      closing_line = $0
      openings = gsub(/\{/, "", opening_line)
      closings = gsub(/\}/, "", closing_line)
      depth += openings - closings
      if (openings > 0) {
        body_started = 1
      }
      if ((body_started && depth == 0) || (!body_started && /;/)) {
        skipping_test_item = 0
        body_started = 0
        depth = 0
      }
      next
    }
    /^[[:space:]]*#\[test\][[:space:]]*$/ {
      skipping_test_item = 1
      next
    }
    { print }
  ' "$1"
}

require_active_wasm_cli_rust_test() {
  source_file="$1"
  function_name="$2"
  test_description="$3"
  item_start_pattern="^fn[[:space:]]+${function_name}[[:space:]]*[(]"
  require_regex_count \
    "$source_file" \
    "^[[:space:]]*fn[[:space:]]+${function_name}[[:space:]]*\\(" \
    1 \
    "${test_description} test owner"
  test_attributes="$(rust_item_attributes_source "$source_file" "$item_start_pattern")"
  require_text_regex_count "$test_attributes" '^#\[test\]$' 1 "${test_description} live test attribute"
  if grep -Eq '^#\[(cfg|cfg_attr|ignore)([^[:alnum:]_]|$)' <<<"$test_attributes"; then
    fail "${test_description} must not be disabled by an attached cfg/ignore attribute"
  fi
  test_source="$(braced_rust_item_source "$source_file" "$item_start_pattern")"
  require_text_regex_count "$test_source" '^[[:space:]]*\.arg\("run"\)$' 1 "${test_description} run command"
  require_text_regex_count "$test_source" '^[[:space:]]*\.arg\("--execution-backend"\)$' 1 "${test_description} backend option"
  require_text_regex_count "$test_source" '^[[:space:]]*\.arg\("wasm"\)$' 1 "${test_description} Wasm backend"
  require_text_regex_count "$test_source" '^[[:space:]]*output\.status\.success\(\),$' 1 "${test_description} process-status assertion"
  require_text_regex_count "$test_source" '^[[:space:]]*assert!\(stdout\.contains\("backend_used: WasmAot"\)\);$' 1 "${test_description} backend assertion"
  require_text_regex_count "$test_source" '^[[:space:]]*assert!\(stdout\.contains\("boolean\(true\)"\), "\{stdout\}"\);$' 1 "${test_description} boolean-result assertion"
}

array_destructuring_evaluation_match_source() {
  awk '
    !capturing {
      if (!match($0, /match[[:space:]]+\*?evaluation[[:space:]]*\{/)) {
        next
      }
      $0 = substr($0, RSTART)
      capturing = 1
    }
    {
      print
      opening_line = $0
      closing_line = $0
      openings = gsub(/\{/, "", opening_line)
      closings = gsub(/\}/, "", closing_line)
      depth += openings - closings
      if (depth == 0) {
        exit
      }
    }
  '
}

require_array_destructuring_producer() {
  source_file="$1"
  function_name="$2"
  evaluation_variant="$3"
  item_start_pattern="^(pub([(][^)]*[)])?[[:space:]]+)?fn[[:space:]]+${function_name}[[:space:]]*[(]"
  require_regex_count "$source_file" "^[[:space:]]*${item_start_pattern#^}" 1 "${function_name} producer declaration"
  producer_source="$(braced_rust_item_source "$source_file" "$item_start_pattern")"
  require_text_regex_count \
    "$producer_source" \
    'evaluation:[[:space:]]*ArrayDestructuringEvaluationIr::[[:alnum:]_]+,' \
    1 \
    "${function_name} array-destructuring evaluation producer"
  require_text_regex_count \
    "$producer_source" \
    "evaluation:[[:space:]]*ArrayDestructuringEvaluationIr::${evaluation_variant}," \
    1 \
    "${function_name} ${evaluation_variant} producer"
}

require_array_destructuring_consumer() {
  source_file="$1"
  function_name="$2"
  item_start_pattern="^(pub([(][^)]*[)])?[[:space:]]+)?fn[[:space:]]+${function_name}[[:space:]]*[(]"
  require_regex_count "$source_file" "^[[:space:]]*${item_start_pattern#^}" 1 "${function_name} semantic consumer declaration"
  consumer_source="$(braced_rust_item_source "$source_file" "$item_start_pattern")"
  require_text_regex_count \
    "$consumer_source" \
    'match[[:space:]]+\*?evaluation[[:space:]]*\{' \
    1 \
    "${function_name} array-destructuring evaluation match"
  evaluation_match="$(printf '%s\n' "$consumer_source" | array_destructuring_evaluation_match_source)"
  require_text_regex_count \
    "$evaluation_match" \
    '^[[:space:]]*ArrayDestructuringEvaluationIr::BindingInitialization[[:space:]]*=>' \
    1 \
    "${function_name} BindingInitialization match arm"
  require_text_regex_count \
    "$evaluation_match" \
    '^[[:space:]]*ArrayDestructuringEvaluationIr::AssignmentEvaluation[[:space:]]*=>' \
    1 \
    "${function_name} AssignmentEvaluation match arm"
  require_text_regex_count \
    "$evaluation_match" \
    '^[[:space:]]*_[[:space:]]*=>' \
    0 \
    "${function_name} array-destructuring wildcard arm"
  require_text_regex_count \
    "$consumer_source" \
    'ArrayDestructuringEvaluationIr::' \
    2 \
    "${function_name} direct evaluation-variant use"
}

ir_ir="crates/lila-ir/src/ir.rs"
require_fixed_string_count "$ir_ir" 'pub enum ArrayDestructuringEvaluationIr {' 1 'public array-destructuring evaluation enum'
array_destructuring_evaluation_derive="$(awk '/^pub enum ArrayDestructuringEvaluationIr \{$/ { print preceding_line; exit } { preceding_line = $0 }' "$ir_ir")"
if [ "$array_destructuring_evaluation_derive" != '#[derive(Debug, Clone, Copy, PartialEq, Eq)]' ]; then
  fail 'ArrayDestructuringEvaluationIr must keep its exact non-Default derives'
fi
array_destructuring_evaluation_enum="$(braced_rust_item_source "$ir_ir" '^pub[[:space:]]+enum[[:space:]]+ArrayDestructuringEvaluationIr[[:space:]]*[{]')"
array_destructuring_evaluation_enum_code="$(printf '%s\n' "$array_destructuring_evaluation_enum" | sed '/^[[:space:]]*\/\//d; /^[[:space:]]*$/d')"
require_text_regex_count "$array_destructuring_evaluation_enum_code" '^    BindingInitialization,$' 1 'BindingInitialization unit variant'
require_text_regex_count "$array_destructuring_evaluation_enum_code" '^    AssignmentEvaluation,$' 1 'AssignmentEvaluation unit variant'
if [ "$(printf '%s\n' "$array_destructuring_evaluation_enum_code" | wc -l | tr -d '[:space:]')" -ne 4 ]; then
  fail "$ir_ir must keep ArrayDestructuringEvaluationIr to its public declaration, two unit variants and closing brace"
fi
if grep -Eq 'bool|Default' <<<"$array_destructuring_evaluation_enum_code" \
  || grep -Eq 'impl[[:space:]]+Default[[:space:]]+for[[:space:]]+ArrayDestructuringEvaluationIr' "$ir_ir"; then
  fail 'ArrayDestructuringEvaluationIr must not regain a bool field or Default implementation'
fi

require_array_destructuring_producer "$ir_lowering" lower_parameter_binding_pattern BindingInitialization
require_array_destructuring_producer "$ir_lowering" lower_pattern_assign_value AssignmentEvaluation
require_array_destructuring_producer "$ir_lowering" lower_pattern_lexical_binding BindingInitialization
require_array_destructuring_producer "$ir_lowering" lower_pattern_var_binding_from_value BindingInitialization
require_array_destructuring_producer "$ir_lowering" lower_pattern_lexical_binding_from_value_with_storage_names BindingInitialization
require_fixed_string_count "$ir_lowering" 'evaluation: ArrayDestructuringEvaluationIr::' 5 'reviewed array-destructuring evaluation producers'

require_array_destructuring_consumer "$ir_ir" validate_async_function_for_of_initialization
require_array_destructuring_consumer "$ir_lib" collect_binding_storage_names
require_array_destructuring_consumer crates/lila-aot-wasm/src/control_flow.rs initialize_direct_lexical_bindings
require_array_destructuring_consumer crates/lila-aot-wasm/src/control_flow.rs compile_array_destructure_to_locals
require_array_destructuring_consumer crates/lila-aot-wasm/src/planning.rs expr_result_tag_is_runtime_dynamic
require_array_destructuring_consumer crates/lila-aot-wasm/src/planning.rs count_statement_lexicals
require_array_destructuring_consumer crates/lila-aot-wasm/src/planning.rs collect_hoisted_vars_statement

array_destructuring_variant_product_files="$({
  while IFS= read -r product_source_file; do
    if rust_source_without_inline_tests "$product_source_file" \
      | grep -F 'ArrayDestructuringEvaluationIr::' >/dev/null; then
      printf '%s\n' "$product_source_file"
    fi
  done < <(find crates -type f -path '*/src/*.rs' -print | sort)
})"
expected_array_destructuring_variant_product_files='crates/lila-aot-wasm/src/control_flow.rs
crates/lila-aot-wasm/src/planning.rs
crates/lila-ir/src/ir.rs
crates/lila-ir/src/lib.rs
crates/lila-ir/src/lowering.rs'
if [ "$array_destructuring_variant_product_files" != "$expected_array_destructuring_variant_product_files" ]; then
  fail "direct ArrayDestructuringEvaluationIr variant use must stay in the reviewed five product files: $array_destructuring_variant_product_files"
fi
for product_variant_spec in \
  'crates/lila-ir/src/lowering.rs|5' \
  'crates/lila-ir/src/ir.rs|2' \
  'crates/lila-ir/src/lib.rs|2' \
  'crates/lila-aot-wasm/src/control_flow.rs|4' \
  'crates/lila-aot-wasm/src/planning.rs|6'
do
  product_source_file="${product_variant_spec%%|*}"
  expected_variant_uses="${product_variant_spec#*|}"
  product_variant_uses="$({
    rust_source_without_inline_tests "$product_source_file" \
      | grep -Fc 'ArrayDestructuringEvaluationIr::'
  } || true)"
  if [ "$product_variant_uses" -ne "$expected_variant_uses" ]; then
    fail "$product_source_file must contain $expected_variant_uses non-test direct array-destructuring evaluation variant uses (found $product_variant_uses)"
  fi
done

array_destructuring_cli="crates/lila-cli/tests/cli/array.rs"
array_destructuring_fixture="crates/lila-cli/tests/fixtures/wasm_array_destructuring_iterators.js"
require_file "$array_destructuring_cli"
require_file "$array_destructuring_fixture"
require_active_wasm_cli_rust_test \
  "$array_destructuring_cli" \
  run_wasm_backend_uses_iterators_for_array_destructuring \
  'array-destructuring CLI regression'
array_destructuring_cli_test="$(braced_rust_item_source "$array_destructuring_cli" '^fn[[:space:]]+run_wasm_backend_uses_iterators_for_array_destructuring[[:space:]]*[(]')"
require_text_regex_count "$array_destructuring_cli_test" 'fixture_path\("wasm_array_destructuring_iterators\.js"\)' 1 'array-destructuring CLI fixture wiring'
check_no_inline_legacy_includes "$ir_lowering"
# Measured after formatting the static-JSON parse extraction: 20,748 raw lines.
# This leaves modest orchestration headroom while preventing the former
# 32k-line implementation store from regrowing.
check_raw_line_budget "$ir_lowering" 21750

# T02's StandardBuiltinId registry. One macro row owns declaration order,
# function-index order, global installation order and every metadata field.
# Keeping the invocation in a real child module preserves an ownership seam;
# `include!` would merely hide the same monolith from line counts.
ir_callable_to_string="crates/lila-ir/src/builtins/callable_to_string.rs"
require_file "$ir_callable_to_string"
require_exact_line_count \
  "$ir_builtins" \
  'mod callable_to_string;' \
  1 \
  'private callable-to-string module declaration'
require_pub_use \
  "$ir_builtins" \
  '^pub use callable_to_string::CallableToStringRepresentation;$' \
  'the callable-to-string representation'
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*pub[[:space:]]+enum[[:space:]]+CallableToStringRepresentation[[:space:]]*\{' \
  1 \
  'CallableToStringRepresentation owner'
require_regex_count \
  "$ir_builtins" \
  '^[[:space:]]*pub[[:space:]]+enum[[:space:]]+CallableToStringRepresentation[[:space:]]*\{' \
  0 \
  'callable-to-string representations in the parent'
require_fixed_string_count \
  "$ir_callable_to_string" \
  'impl CallableToStringRepresentation {' \
  1 \
  'callable-to-string materializer owner'
require_tree_regex_count \
  crates/lila-ir/src \
  '^[[:space:]]*fn[[:space:]]+callable_to_string_representations_materialize_spec_shapes[[:space:]]*\(' \
  1 \
  'colocated callable-to-string behavior test'
require_fixed_string_count \
  "$ir_builtins" \
  'fn callable_to_string_representations_materialize_spec_shapes(' \
  0 \
  'callable-to-string behavior tests in the parent'
if grep -Eq '^[[:space:]]*_[[:space:]]*=>' "$ir_callable_to_string"; then
  fail "$ir_callable_to_string must materialize every representation exhaustively"
fi
check_no_inline_legacy_includes "$ir_callable_to_string"
# Measured after extraction: 38 raw lines. This child owns only the closed
# representation and its materializer/test.
check_raw_line_budget "$ir_callable_to_string" 50
ir_builtin_catalog="crates/lila-ir/src/builtins/catalog.rs"
ir_builtin_catalog_contract_tests="crates/lila-ir/src/builtins/catalog_contract_tests.rs"
require_file "$ir_builtin_catalog"
require_file "$ir_builtin_catalog_contract_tests"
require_module_decl "$ir_builtins" "catalog"
require_exact_line_count \
  "$ir_builtins" \
  'mod catalog_contract_tests;' \
  1 \
  'test-only builtin catalog contract module declaration'
require_exact_line_count \
  "$ir_builtin_catalog_contract_tests" \
  'fn indexed_receiver_mutation_is_owned_by_the_builtin_catalog() {' \
  1 \
  'indexed-receiver mutation catalog contract'
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
check_no_inline_legacy_includes "$ir_builtin_catalog_contract_tests"
check_raw_line_budget "$ir_builtin_catalog_contract_tests" 40
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
require_fixed_string_count \
  "$ir_host_builtin_catalog" \
  'pub(crate) const fn may_invalidate_caller_flow(self) -> bool {' \
  1 \
  'host caller-flow classification owner'
require_fixed_string_count \
  "$ir_host_builtin_catalog" \
  'Self::CreateRealm => false,' \
  1 \
  'sole caller-flow-preserving host builtin'
host_caller_flow_classifier="$(sed -n '/pub(crate) const fn may_invalidate_caller_flow(self)/,/^    }/p' "$ir_host_builtin_catalog")"
if grep -Eq '(^|[|,(])[[:space:]]*_[[:space:]]*=>' <<<"$host_caller_flow_classifier"; then
  fail "$ir_host_builtin_catalog must exhaust host caller-flow effects without a catch-all"
fi
# Measured after adding the catalog-owned indexed-receiver mutation contract:
# 1,748 raw lines.
# raw lines.
# Metadata rows belong in their catalogs; shared machinery should shrink rather
# than regrow.
check_raw_line_budget "$ir_builtins" 1760

for module in abi arguments_protocol control_flow data emit environments expressions functions gc_types heap module modules objects operations planning; do
  require_file "crates/lila-aot-wasm/src/${module}.rs"
  require_module_decl "$wasm_lib" "$module"
done

# T02 gives the complete resumable synchronous for-of emitter one real private
# child owner. Its crate visibility is required by emission_sites.rs, which
# names the method as the obligation ledger witness.
wasm_control_flow="crates/lila-aot-wasm/src/control_flow.rs"
wasm_async_function_for_of_iterator="crates/lila-aot-wasm/src/control_flow/async_function_for_of_iterator.rs"
wasm_emission_sites="crates/lila-aot-wasm/src/emission_sites.rs"
require_file "$wasm_async_function_for_of_iterator"
require_exact_line_count \
  "$wasm_control_flow" \
  'mod async_function_for_of_iterator;' \
  1 \
  'private async-function for-of child declarations'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*mod[[:space:]]+async_function_for_of_iterator;' \
  1 \
  'private async-function for-of child declarations'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+async_function_for_of_iterator;' "$wasm_control_flow"; then
  fail "$wasm_control_flow must keep async_function_for_of_iterator private"
fi
require_exact_line_count \
  "$wasm_async_function_for_of_iterator" \
  '    pub(crate) fn compile_async_function_for_of_iterator(' \
  1 \
  'crate-visible resumable synchronous for-of owners'
require_fixed_string_count \
  "$wasm_async_function_for_of_iterator" \
  'fn compile_async_function_for_of_iterator(' \
  1 \
  'complete resumable synchronous for-of owner declarations'
require_fixed_string_count \
  "$wasm_control_flow" \
  'fn compile_async_function_for_of_iterator(' \
  0 \
  'resumable synchronous for-of owner declarations outside the child'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+compile_async_function_for_of_iterator[[:space:]]*\(' \
  1 \
  'complete resumable synchronous for-of owners'
require_fixed_string_count \
  "$wasm_control_flow" \
  'self.compile_async_function_for_of_iterator(iterable, plan, function)?;' \
  1 \
  'resumable synchronous for-of statement-dispatch calls'
require_exact_line_count \
  "$wasm_emission_sites" \
  '            let _ = FunctionBuilder::compile_async_function_for_of_iterator;' \
  1 \
  'resumable synchronous for-of obligation-ledger references'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'compile_async_function_for_of_iterator' \
  3 \
  'resumable synchronous for-of owner, dispatch and obligation-ledger sites'
check_no_inline_legacy_includes "$wasm_control_flow"
check_no_inline_legacy_includes "$wasm_async_function_for_of_iterator"

async_function_for_of_iterator_owner="$({
  sed -n \
    '/^    pub(crate) fn compile_async_function_for_of_iterator(/,/^    }$/p' \
    "$wasm_async_function_for_of_iterator"
})"
async_function_for_of_iterator_owner_lines="$(printf '%s\n' "$async_function_for_of_iterator_owner" | wc -l | tr -d '[:space:]')"
if [ "$async_function_for_of_iterator_owner_lines" -ne 416 ]; then
  fail "$wasm_async_function_for_of_iterator must retain the reviewed 416-line complete owner (found $async_function_for_of_iterator_owner_lines)"
fi
async_function_for_of_iterator_owner_sha256="$(printf '%s\n' "$async_function_for_of_iterator_owner" | sha256_stream)"
if [ "$async_function_for_of_iterator_owner_sha256" != 'd722dc0abbfda6aea0f1bec2b8fd15cd40f32c34eb443ac082e62744950dcec5' ]; then
  fail "$wasm_async_function_for_of_iterator complete owner changed from the reviewed synchronous-iterator consumer SHA-256 (found $async_function_for_of_iterator_owner_sha256)"
fi
if ! awk '
  index($0, "let activation_local =") && !activation { activation = NR }
  index($0, "self.emit_get_iterator_from_value_locals(") && !acquire { acquire = NR }
  index($0, "let loop_frame = self.open_frame") && !loop { loop = NR }
  index($0, "self.emit_sync_iterator_step_value(") && !step { step = NR }
  index($0, "ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment)") && !environment { environment = NR }
  index($0, "let (value_storage, value_is_entry_local) = match plan.value_storage()") && !value { value = NR }
  index($0, "self.finally_stack.push(close_frame);") && !close_frame { close_frame = NR }
  index($0, "self.compile_statement(plan.await_statement(), function)?;") && !await_statement { await_statement = NR }
  index($0, "self.save_current_completion(") && !save { save = NR }
  index($0, "self.emit_leave_lexical_environment(function);") && !leave_environment { leave_environment = NR }
  index($0, "self.emit_iterator_close_preserving_current_throw(") && !close_throw { close_throw = NR }
  index($0, "self.emit_dispatch_async_completion(function)?;") && !dispatch { dispatch = NR }
  index($0, "self.release_sync_iterator_locals(iterator_locals);") && !release { release = NR }
  END {
    exit !(activation < acquire && acquire < loop && loop < step \
      && step < environment && environment < value && value < close_frame \
      && close_frame < await_statement && await_statement < save \
      && save < leave_environment && leave_environment < close_throw \
      && close_throw < dispatch && dispatch < release)
  }
' <<<"$async_function_for_of_iterator_owner"; then
  fail "$wasm_async_function_for_of_iterator must retain acquisition, iteration, close, dispatch and release order"
fi

# Measured immediately after extraction: 13,220 parent lines and 424 child
# lines. The margins admit narrow maintenance without letting the owner return
# to the parent or become another control-flow monolith.
check_raw_line_budget "$wasm_control_flow" 13260
check_raw_line_budget "$wasm_async_function_for_of_iterator" 440

# T05's typed Wasm-GC schema is the sole raw struct-instruction boundary. The
# encoder dependency necessarily accepts interchangeable u32 immediates, so
# direct StructNew/Get/Set construction anywhere else would discard the
# owner/field/target types before the final encoding step.
wasm_gc_types="crates/lila-aot-wasm/src/gc_types.rs"
compiled_module_package="crates/lila-aot-wasm/src/module/compiled_module_package.rs"
require_file "$compiled_module_package"
require_exact_line_count \
  crates/lila-aot-wasm/src/module.rs \
  'mod compiled_module_package;' \
  1 \
  'private compiled-module-package owner declarations'
if grep -Eq '^pub(\(crate\))? mod compiled_module_package;' crates/lila-aot-wasm/src/module.rs; then
  fail 'the compiled-module-package owner must remain private'
fi
compiled_package_reexport="$(
  sed -n '/^pub(crate) use compiled_module_package::{/,/^};/p' crates/lila-aot-wasm/src/module.rs
)"
for surface in ModuleAssemblySections ModuleGlobalSectionBuilder ModuleTypeRegistry; do
  require_text_regex_count "$compiled_package_reexport" "[ ,]${surface}[, ]" 1 "compiled-module-package ${surface} re-exports"
done
for private_state in FinalizedModuleSections CompiledModulePackage CallableFunctionTableSections; do
  if printf '%s\n' "$compiled_package_reexport" | grep -Fq "$private_state"; then
    fail "$private_state must remain private to $compiled_module_package"
  fi
done
for sole_owner in ModuleTypeRegistry FinalizedModuleSections CompiledModulePackage \
                  CallableFunctionTableSections ModuleAssemblySections ModuleTypeSectionBuilder \
                  ModuleGlobalSectionBuilder; do
  require_fixed_string_count "$compiled_module_package" "struct ${sole_owner}" 1 "${sole_owner} owners"
  if sed '/^#\[cfg(test)\]/,$d' crates/lila-aot-wasm/src/module.rs \
    | grep -Fq "struct ${sole_owner}"; then
    fail "$sole_owner must be owned only by $compiled_module_package"
  fi
done
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

# T05's runtime root is derived only while the complete scalar/dynamic global
# section is consumed. Keeping its raw constructor at one site prevents the old
# independently predicted u32 slot from returning under another name.
require_fixed_string_count \
  "$wasm_gc_types" \
  'GcRootGlobal::new(' \
  1 \
  'typed GC root construction site in the global-section finalizer'
if grep -R -Eq 'runtime_gc_root_global_index|fn bind_root\(' crates/lila-aot-wasm/src; then
  fail 'the runtime GC root must be derived from the finalized global section, not a planned raw index'
fi
if ! grep -Fq 'pub(crate) fn finalize_globals(' "$wasm_gc_types" \
  || ! grep -Fq 'gc_anchor_root: GcRootGlobal::new(globals.len()),' "$wasm_gc_types" \
  || ! grep -Fq 'impl Section for FinalizedModuleGlobals' "$wasm_gc_types"; then
  fail "$wasm_gc_types must derive the root from the actual global count and encode the opaque sealed section"
fi
if grep -Eq 'pub\(crate\).*RuntimeModuleSchema|fn (runtime_schema|section)\(&self\).*GlobalSection' "$wasm_gc_types"; then
  fail "$wasm_gc_types must not expose a copyable runtime schema or the cloneable raw global section"
fi
gc_schema_escapes="$(
  find crates/lila-aot-wasm/src -type f -name '*.rs' ! -path "$wasm_gc_types" -print0 \
    | xargs -0 grep -Fn 'RuntimeModuleSchema' || true
)"
if [ -n "$gc_schema_escapes" ]; then
  fail "the private runtime GC schema must not escape $wasm_gc_types: $gc_schema_escapes"
fi
require_fixed_string_count \
  "$compiled_module_package" \
  'runtime.finalize_globals(self.section)' \
  1 \
  'global-section builder finalization through the typed runtime registry'
require_fixed_string_count \
  crates/lila-aot-wasm/src/emit.rs \
  'module_types.finalize_globals(globals)' \
  1 \
  'complete module global-section finalization site'
if ! grep -Fq "Main(&'a FinalizedModuleGlobals)" crates/lila-aot-wasm/src/emit.rs \
  || ! grep -Fq 'module_sections.compile_main(MainFunctionCompilation::new(' crates/lila-aot-wasm/src/emit.rs \
  || ! grep -Fq 'compilation.compile_into(&self.globals, &mut code)?' "$compiled_module_package" \
  || ! grep -Fq 'code.push(EmittedFunction::new(FunctionIdentity::Main, main));' crates/lila-aot-wasm/src/emit.rs \
  || ! grep -Fq 'CompiledModulePackage::append_remaining_functions;' "$compiled_module_package"; then
  fail 'main must compile into package-owned code through its exact finalized globals'
fi
for rejected_surface in runtime_globals push_main_to append_types_to append_globals_to; do
  if grep -Fq "${rejected_surface}(" "$compiled_module_package"; then
    fail "the finalized module package must not expose split assembly surface: ${rejected_surface}"
  fi
done
if grep -Fq 'impl FnOnce(&FinalizedModuleGlobals)' "$compiled_module_package"; then
  fail 'the finalized module package must use the closed main compiler, not an arbitrary callback'
fi
if grep -Fq 'CompilingModulePackage' "$compiled_module_package"; then
  fail 'main compilation must return the one compiled package, not an independently consumable code package'
fi
require_fixed_string_count \
  "$compiled_module_package" \
  'pub(crate) fn append_to_module(' \
  1 \
  'consume-once compiled-package assembly transition'
require_fixed_string_count \
  crates/lila-aot-wasm/src/emit.rs \
  'module_package.append_to_module(' \
  1 \
  'compiled-package assembly consumer'
for sealed_section in types globals code; do
  sealed_section_escapes="$(
    find crates/lila-aot-wasm/src -type f -name '*.rs' \
      ! -path "$compiled_module_package" \
      ! -path 'crates/lila-aot-wasm/src/module.rs' -print0 \
      | xargs -0 grep -Fn "module.section(&${sealed_section})" || true
  )"
  module_parent_escape="$(
    sed '/^#\[cfg(test)\]/,$d' crates/lila-aot-wasm/src/module.rs \
      | grep -Fn "module.section(&${sealed_section})" || true
  )"
  if [ -n "$sealed_section_escapes" ] || [ -n "$module_parent_escape" ]; then
    fail "sealed runtime ${sealed_section} section escaped consume-once package assembly: ${sealed_section_escapes}"
  fi
done
global_section_constructor_escapes="$(
  find crates/lila-aot-wasm/src -type f -name '*.rs' \
    ! -path "$compiled_module_package" \
    ! -path "$wasm_gc_types" -print0 \
    | xargs -0 grep -Fn 'GlobalSection::new()' || true
)"
if [ -n "$global_section_constructor_escapes" ] \
  || sed '/^#\[cfg(test)\]/,$d' "$wasm_gc_types" | grep -Fq 'GlobalSection::new()'; then
  fail "production GlobalSection construction must stay in $compiled_module_package: $global_section_constructor_escapes"
fi

for module in array atomics bigint binary_data boolean bootstrap date errors function \
              global_numeric host iterators json math number object proxy reflect \
              standard string symbol uri; do
  require_file "crates/lila-aot-wasm/src/builtins/${module}.rs"
  require_module_decl "$wasm_builtins_mod" "$module"
done

wasm_builtin_bootstrap="crates/lila-aot-wasm/src/builtins/bootstrap.rs"
if ! grep -q 'match builtin\.intrinsic_installer()' "$wasm_builtin_bootstrap"; then
  fail "$wasm_builtin_bootstrap must dispatch through the catalog installer class"
fi


# T02's Object, Proxy, Math, Symbol, BigInt, Boolean, Number, Function, Atomics,
# global numeric, URI, Error and JSON
# builtin body boundaries. The exhaustive StandardBuiltinId dispatch remains in
# standard.rs, but family bodies are one-line delegates so unrelated builtin
# work no longer collides with ~11k lines of Object descriptor/prototype
# implementation, the Proxy lifecycle, the Math emitter family, Symbol's
# registry/prototype implementation or BigInt's constructor, fixed-width and
# prototype implementation, Boolean's constructor and prototype receiver logic,
# Number's constructor, predicates and prototype methods, Function's constructor,
# four prototype methods and hidden bound-function invoker, the Error intrinsic
# family, the Atomics integer/wait family, or JSON's parse/stringify/raw-JSON
# wrappers. The two coercing global numeric predicates
# and the six global URI and Annex-B codec wrappers likewise stay out of the
# shared dispatcher.
check_no_inline_legacy_includes "$wasm_standard_builtins"
# Measured after the Atomics extraction: 30,567 raw lines before formatting.
# This margin is dispatch-maintenance headroom; substantive bodies belong in
# family modules.
check_raw_line_budget "$wasm_standard_builtins" 30800

wasm_atomics_builtins="crates/lila-aot-wasm/src/builtins/atomics.rs"
check_no_inline_legacy_includes "$wasm_atomics_builtins"
if ! grep -q '^enum AtomicsBuiltin' "$wasm_atomics_builtins" \
  || ! grep -q '^enum AtomicsIntegerOperation' "$wasm_atomics_builtins" \
  || ! grep -q '^enum AtomicsRmwOperation' "$wasm_atomics_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_atomics_builtins"; then
  fail "$wasm_atomics_builtins must dispatch through the closed Atomics builtin/integer/RMW domains"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+AtomicsBuiltin' "$wasm_atomics_builtins"; then
  fail "$wasm_atomics_builtins must keep AtomicsBuiltin private"
fi
if grep -Eq 'AtomicsBuiltin|emit_atomics_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Atomics entries"
fi
require_fixed_string_count \
  "$wasm_atomics_builtins" \
  'self.emit_atomics_builtin(' \
  14 \
  'fixed Atomics entry call'
require_fixed_string_count "$wasm_atomics_builtins" 'fn emit_atomics_builtin(' 1 'private Atomics emitter'
require_fixed_string_count "$wasm_atomics_builtins" 'pub(super) fn emit_atomics_' 15 'fixed Atomics entries plus the TypedArray BigInt-kind predicate'
for atomics_builtin in add and compare_exchange exchange is_lock_free load notify or pause store sub wait wait_async xor; do
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "self.emit_atomics_${atomics_builtin}_builtin(function)?" \
    1 \
    "fixed Atomics ${atomics_builtin} route"
done
require_fixed_string_count \
  "$wasm_atomics_builtins" \
  'pub(super) const ATOMICS_PUBLICATION_ORDER: [StandardBuiltinId; 14]' \
  1 \
  'ordered Atomics publication surface'
for atomics_publication_owner in "$wasm_builtin_bootstrap" crates/lila-aot-wasm/src/builtins/host.rs; do
  require_fixed_string_count \
    "$atomics_publication_owner" \
    'for builtin in ATOMICS_PUBLICATION_ORDER' \
    1 \
    'Atomics publication loop'
  if grep -Eq 'AtomicsBuiltin|atomics_standard_builtin' "$atomics_publication_owner"; then
    fail "$atomics_publication_owner must publish fixed StandardBuiltinId entries without raw Atomics policy"
  fi
done
require_fixed_string_count \
  "$wasm_atomics_builtins" \
  'pub(super) fn emit_atomics_bigint_element_kind_i32(' \
  1 \
  'cross-family TypedArray BigInt-kind predicate'
require_fixed_string_count \
  "$wasm_atomics_builtins" \
  'pub(crate) fn emit_drain_atomics_wait_async_timeouts(' \
  1 \
  'event-loop Atomics waiter drain hook'
require_fixed_string_count \
  "$wasm_atomics_builtins" \
  'pub(crate) fn emit_poll_atomics_wait_async_timeouts(' \
  1 \
  'promise-checkpoint Atomics waiter poll hook'
require_fixed_string_count \
  "$wasm_atomics_builtins" \
  'StandardBuiltinId' \
  15 \
  'Atomics publication type and entries'
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' "$wasm_atomics_builtins"; then
  fail "$wasm_atomics_builtins must keep family matches exhaustive without catch-all arms"
fi
# Measured immediately after extraction: 2,767 raw lines before formatting.
check_raw_line_budget "$wasm_atomics_builtins" 2850

wasm_boolean_builtins="crates/lila-aot-wasm/src/builtins/boolean.rs"
check_no_inline_legacy_includes "$wasm_boolean_builtins"
if ! grep -q '^enum BooleanBuiltin' "$wasm_boolean_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_boolean_builtins"; then
  fail "$wasm_boolean_builtins must dispatch through the closed BooleanBuiltin domain"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+BooleanBuiltin' "$wasm_boolean_builtins"; then
  fail "$wasm_boolean_builtins must keep BooleanBuiltin private"
fi
if grep -Eq 'BooleanBuiltin|emit_boolean_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Boolean entries"
fi
require_fixed_string_count "$wasm_boolean_builtins" 'fn emit_boolean_builtin(' 1 'private Boolean emitter'
require_fixed_string_count "$wasm_boolean_builtins" 'self.emit_boolean_builtin(' 3 'fixed Boolean entry calls'
# Measured immediately after extraction: 139 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_boolean_builtins" 200

wasm_math_builtins="crates/lila-aot-wasm/src/builtins/math.rs"
check_no_inline_legacy_includes "$wasm_math_builtins"
if ! grep -q '^enum MathBuiltin' "$wasm_math_builtins" \
  || ! grep -q '^enum MathUnaryBuiltin' "$wasm_math_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_math_builtins" \
  || ! grep -q '^                match unary {' "$wasm_math_builtins"; then
  fail "$wasm_math_builtins must dispatch through the closed nested Math domains"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+(MathBuiltin|MathUnaryBuiltin)' "$wasm_math_builtins"; then
  fail "$wasm_math_builtins must keep both Math domains private"
fi
if grep -Eq 'MathBuiltin|MathUnaryBuiltin|MathFn|UnaryMathFn|emit_math\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Math entries"
fi
require_fixed_string_count "$wasm_math_builtins" 'fn emit_math(' 1 'private Math emitter'
require_fixed_string_count "$wasm_math_builtins" 'self.emit_math(' 37 'fixed Math entry calls'
require_fixed_string_count "$wasm_math_builtins" 'pub(super) fn emit_math_' 37 'fixed Math entry definitions'
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_math_' 37 'fixed Math routes'
if grep -q 'StandardBuiltinId::' "$wasm_math_builtins"; then
  fail "$wasm_math_builtins must accept only its closed family domains, not StandardBuiltinId"
fi
# Measured after closing the 37 fixed entries: 2,393 raw lines. The margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_math_builtins" 2430

wasm_number_builtins="crates/lila-aot-wasm/src/builtins/number.rs"
check_no_inline_legacy_includes "$wasm_number_builtins"
if ! grep -q '^enum NumberBuiltin' "$wasm_number_builtins" \
  || ! grep -q '^enum NumberPrototypeOperation' "$wasm_number_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_number_builtins" \
  || ! grep -q '^        match operation {' "$wasm_number_builtins"; then
  fail "$wasm_number_builtins must dispatch through the closed Number builtin/prototype domains"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+NumberBuiltin' "$wasm_number_builtins"; then
  fail "$wasm_number_builtins must keep NumberBuiltin private"
fi
if grep -Eq 'NumberBuiltin|emit_number_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Number entries"
fi
require_fixed_string_count \
  "$wasm_number_builtins" \
  'fn emit_number_constructor_result(' \
  1 \
  'Number constructor body'
require_fixed_string_count \
  "$wasm_number_builtins" \
  'fn emit_number_prototype_builtin(' \
  1 \
  'shared Number prototype receiver/body dispatch'
require_fixed_string_count \
  "$wasm_number_builtins" \
  'fn emit_number_builtin(' \
  1 \
  'private Number builtin emitter'
require_fixed_string_count \
  "$wasm_number_builtins" \
  'self.emit_number_builtin(' \
  11 \
  'fixed Number entry calls'
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
# Measured after closing the eleven fixed entries: 400 raw lines. The narrow
# margin is for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_number_builtins" 430

wasm_function_builtins="crates/lila-aot-wasm/src/builtins/function.rs"
check_no_inline_legacy_includes "$wasm_function_builtins"
if ! grep -q '^enum FunctionBuiltin' "$wasm_function_builtins" \
  || ! grep -q '^    BoundFunctionInvoker,$' "$wasm_function_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_function_builtins"; then
  fail "$wasm_function_builtins must dispatch through the closed FunctionBuiltin domain"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+FunctionBuiltin' "$wasm_function_builtins"; then
  fail "$wasm_function_builtins must keep FunctionBuiltin private"
fi
if grep -Eq 'FunctionBuiltin|emit_function_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Function entries"
fi
require_fixed_string_count \
  "$wasm_function_builtins" \
  'self.emit_function_builtin(' \
  8 \
  'fixed Function entry calls'
require_fixed_string_count "$wasm_function_builtins" 'fn emit_function_builtin(' 1 'private Function emitter'
bound_function_invoker_delegate="$(
  sed -n \
    '/^            StandardBuiltinId::BoundFunctionInvoker => {$/,/^            }$/p' \
    "$wasm_standard_builtins"
)"
if [ "$(printf '%s\n' "$bound_function_invoker_delegate" | wc -l)" -ne 3 ] \
  || ! grep -Fqx \
    '                self.emit_bound_function_invoker_builtin(function)?' \
    <<<"$bound_function_invoker_delegate"; then
  fail "$wasm_standard_builtins must keep BoundFunctionInvoker as one fixed delegate"
fi
require_fixed_string_count \
  "$wasm_function_builtins" \
  'FunctionBuiltin::BoundFunctionInvoker => {' \
  1 \
  'bound-function invoker body'
if grep -q 'StandardBuiltinId::BoundFunctionInvoker' "$wasm_function_builtins"; then
  fail "$wasm_function_builtins must own the bound-function invoker through FunctionBuiltin"
fi
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' "$wasm_function_builtins"; then
  fail "$wasm_function_builtins must keep its family match exhaustive without catch-all arms"
fi
require_fixed_string_count \
  crates/lila-cli/tests/cli/functions.rs \
  'fn run_wasm_backend_succeeds_for_supported_bind_builtin_fixture()' \
  1 \
  'bound-function call/construct regression'
require_fixed_string_count \
  crates/lila-cli/tests/cli/language_errors.rs \
  'fn run_wasm_backend_succeeds_for_bound_construct_new_target_identity_fixture()' \
  1 \
  'bound-function new.target regression'
require_fixed_string_count \
  crates/lila-cli/tests/cli/heap.rs \
  'fn run_wasm_backend_succeeds_for_heap_rooted_bound_function_fixture()' \
  1 \
  'bound-function heap-rooting regression'
# Measured after closing eight fixed entries: 508 raw lines. The narrow margin
# is for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_function_builtins" 525

wasm_date_builtins="crates/lila-aot-wasm/src/builtins/date.rs"
wasm_date_local_string="crates/lila-aot-wasm/src/builtins/date/local_string.rs"
require_file "$wasm_date_local_string"
check_no_inline_legacy_includes "$wasm_date_builtins"
check_no_inline_legacy_includes "$wasm_date_local_string"
if ! grep -q '^enum DateComponentSetterOperation' "$wasm_date_builtins" \
  || ! grep -q '^        match operation {' "$wasm_date_builtins"; then
  fail "$wasm_date_builtins must privately dispatch through DateComponentSetterOperation"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+DateComponentSetterOperation' "$wasm_date_builtins"; then
  fail "$wasm_date_builtins must keep DateComponentSetterOperation private"
fi
if grep -Eq 'DateComponentSetterOperation|emit_date_component_setter\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Date setter entries"
fi
require_fixed_string_count "$wasm_date_builtins" 'fn emit_date_component_setter(' 1 'private Date setter emitter'
require_fixed_string_count "$wasm_date_builtins" 'self.emit_date_component_setter(' 7 'fixed Date setter entry calls'
require_exact_line_count \
  "$wasm_date_builtins" \
  'mod local_string;' \
  1 \
  'private Date local-string module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+local_string;' "$wasm_date_builtins"; then
  fail "$wasm_date_builtins must keep local_string private"
fi
if grep -Eq 'DateLocalStringFormat|DateTimeValueSource|emit_date_time_value_from_source\(|wall_clock_millis_import_function_index|local_string::' "$wasm_date_builtins"; then
  fail "$wasm_date_builtins must not name, construct, project or import the private Date local-string and time-source policies"
fi
require_regex_count \
  "$wasm_date_local_string" \
  '^enum[[:space:]]+DateTimeValueSource[[:space:]]*\{' \
  1 \
  'private Date time-value source owner'
require_fixed_string_count \
  "$wasm_date_local_string" \
  'DateTimeValueSource' \
  10 \
  'Date time-value source owner uses'
require_fixed_string_count \
  "$wasm_date_local_string" \
  'DateTimeValueSource::' \
  7 \
  'Date time-value source exhaustive arms and producers'
require_fixed_string_count \
  "$wasm_date_local_string" \
  'emit_date_time_value_from_source(' \
  3 \
  'Date time-value source consumer and typed callers'
require_fixed_string_count \
  "$wasm_date_local_string" \
  '.wall_clock_millis_import_function_index()' \
  1 \
  'sole Date clock-import access'
require_regex_count \
  "$wasm_date_local_string" \
  '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_date_current_time_payload[[:space:]]*\(' \
  1 \
  'Date current-time semantic wrapper owner'
require_fixed_string_count \
  "$wasm_date_builtins" \
  'self.emit_date_current_time_payload(self.result_local, function)?;' \
  1 \
  'unchanged Date.now semantic delegate'
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.emit_date_current_time_payload(value_payload_local, function)?;' \
  1 \
  'unchanged Date constructor current-time semantic delegate'
require_regex_count \
  "$wasm_date_local_string" \
  '^enum[[:space:]]+DateLocalStringFormat[[:space:]]*\{' \
  1 \
  'private Date local-string format owner'
require_fixed_string_count \
  "$wasm_date_local_string" \
  'DateLocalStringFormat' \
  9 \
  'Date local-string format owner uses'
require_fixed_string_count \
  "$wasm_date_local_string" \
  'emit_date_local_string(' \
  5 \
  'Date local-string consumer and producer sites'
for date_local_string_surface in \
  emit_date_function_call \
  emit_date_to_date_string \
  emit_date_to_time_string \
  emit_date_to_string
do
  require_regex_count \
    "$wasm_date_local_string" \
    "^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+${date_local_string_surface}[[:space:]]*\(" \
    1 \
    "$date_local_string_surface semantic wrapper owner"
done
for date_local_string_call in \
  'self.emit_date_function_call(function)?;' \
  'self.emit_date_to_date_string(function)?;' \
  'self.emit_date_to_time_string(function)?;' \
  'self.emit_date_to_string(function)?;'
do
  case "$date_local_string_call" in
    *emit_date_function_call*) expected_calls=1 ;;
    *) expected_calls=2 ;;
  esac
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "$date_local_string_call" \
    "$expected_calls" \
    'unchanged Date local-string semantic delegates'
done
# Measured immediately after the time-source extraction: 1,675 parent lines and 339 child
# lines. The narrow margins are for maintenance of each owner.
check_raw_line_budget "$wasm_date_builtins" 1725
check_raw_line_budget "$wasm_date_local_string" 370

wasm_bigint_builtins="crates/lila-aot-wasm/src/builtins/bigint.rs"
wasm_bigint_radix_formatting="crates/lila-aot-wasm/src/builtins/bigint/radix_formatting.rs"
wasm_numeric_operations="crates/lila-aot-wasm/src/operations.rs"
wasm_emit="crates/lila-aot-wasm/src/emit.rs"
wasm_host_builtins="crates/lila-aot-wasm/src/builtins/host.rs"
wasm_created_realm_weak_ref_intrinsics="crates/lila-aot-wasm/src/builtins/host/created_realm_weak_ref_intrinsics.rs"
require_file "$wasm_created_realm_weak_ref_intrinsics"
check_no_inline_legacy_includes "$wasm_host_builtins"
check_no_inline_legacy_includes "$wasm_created_realm_weak_ref_intrinsics"
require_exact_line_count \
  "$wasm_host_builtins" \
  'mod created_realm_weak_ref_intrinsics;' \
  1 \
  'private created-Realm WeakRef intrinsic lifecycle module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+created_realm_weak_ref_intrinsics;' "$wasm_host_builtins"; then
  fail "$wasm_host_builtins must keep created_realm_weak_ref_intrinsics private"
fi
if grep -Eq 'CreatedRealmWeakRefIntrinsics|created_realm_weak_ref_intrinsics::' "$wasm_host_builtins"; then
  fail "$wasm_host_builtins must not name, construct, project or import the created-Realm WeakRef lifecycle carrier"
fi
if ! grep -q '^pub(super) struct CreatedRealmWeakRefIntrinsics {$' "$wasm_created_realm_weak_ref_intrinsics" \
  || grep -Eq '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+(prototype_local|constructor_local):' "$wasm_created_realm_weak_ref_intrinsics" \
  || ! grep -q '^    pub(super) fn emit_materialize_created_realm_weak_ref_intrinsics(' "$wasm_created_realm_weak_ref_intrinsics" \
  || ! grep -q '^    pub(super) fn emit_publish_created_realm_weak_ref_intrinsics(' "$wasm_created_realm_weak_ref_intrinsics"; then
  fail "$wasm_created_realm_weak_ref_intrinsics must own the opaque carrier and its sibling-visible producer/consumer"
fi
require_fixed_string_count \
  "$wasm_created_realm_weak_ref_intrinsics" \
  'CreatedRealmWeakRefIntrinsics' \
  5 \
  'created-Realm WeakRef carrier lifecycle sites'
for weak_ref_lifecycle_method in \
  emit_materialize_created_realm_weak_ref_intrinsics \
  emit_publish_created_realm_weak_ref_intrinsics
do
  require_fixed_string_count \
    "$wasm_created_realm_weak_ref_intrinsics" \
    "fn $weak_ref_lifecycle_method(" \
    1 \
    "created-Realm WeakRef lifecycle definition $weak_ref_lifecycle_method"
  require_fixed_string_count \
    "$wasm_host_builtins" \
    "self.$weak_ref_lifecycle_method(" \
    1 \
    "created-Realm WeakRef lifecycle call $weak_ref_lifecycle_method"
done
check_no_inline_legacy_includes "$wasm_bigint_builtins"
check_no_inline_legacy_includes "$wasm_bigint_radix_formatting"
if ! grep -q '^enum BigIntBuiltin' "$wasm_bigint_builtins" \
  || ! grep -q '^enum BigIntPrototypeResultPolicy' "$wasm_bigint_builtins" \
  || ! grep -q '^enum BigIntFixedWidthOperation' "$wasm_bigint_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_bigint_builtins" \
  || ! grep -q '^                match result_policy {' "$wasm_bigint_builtins"; then
  fail "$wasm_bigint_builtins must dispatch through the closed BigInt builtin/result domains"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+(struct|enum)[[:space:]]+BigInt(ValueResult|RadixStringResult|LocaleStringFallbackResult|PrototypeResultPolicy|FixedWidthOperation|Builtin)' "$wasm_bigint_builtins"; then
  fail "$wasm_bigint_builtins must keep its BigInt policy and result-authority domains private"
fi
if grep -Eq 'BigIntBuiltin|BigIntFixedWidthOperation|emit_bigint_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed BigInt entries"
fi
require_fixed_string_count "$wasm_bigint_builtins" 'fn emit_bigint_builtin(' 1 'private BigInt emitter'
require_fixed_string_count "$wasm_bigint_builtins" 'self.emit_bigint_builtin(' 6 'fixed BigInt entry calls'
require_fixed_string_count "$wasm_bigint_builtins" 'pub(super) fn emit_bigint_' 6 'fixed BigInt entry definitions'
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_bigint_' 6 'fixed BigInt routes'
require_exact_line_count \
  "$wasm_bigint_builtins" \
  'mod radix_formatting;' \
  1 \
  'private BigInt radix-formatting module declaration'
if grep -Eq 'PreparedBigIntRadixLocal|emit_prepare_bigint_radix\(|radix_formatting::' "$wasm_bigint_builtins"; then
  fail "$wasm_bigint_builtins must not name, construct, project or import the private prepared-radix lifecycle"
fi
if ! grep -q '^struct PreparedBigIntRadixLocal(u32);$' "$wasm_bigint_radix_formatting" \
  || ! grep -q '^    pub(super) fn emit_bigint_radix_string_result(' "$wasm_bigint_radix_formatting" \
  || ! grep -q '^    fn emit_prepare_bigint_radix(' "$wasm_bigint_radix_formatting" \
  || grep -Eq '^pub|^    pub\((crate|super)\) fn emit_prepare_bigint_radix' "$wasm_bigint_radix_formatting"; then
  fail "$wasm_bigint_radix_formatting must privately own the prepared-radix carrier and producer behind one sibling-visible semantic wrapper"
fi
require_fixed_string_count \
  "$wasm_bigint_radix_formatting" \
  'fn emit_prepare_bigint_radix(' \
  1 \
  'private prepared-radix stage'
require_fixed_string_count \
  "$wasm_bigint_builtins" \
  'emit_bigint_radix_string_result(' \
  1 \
  'parent semantic radix-result wrapper call'

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

# Measured after the prepared-radix lifecycle extraction: 807 parent lines and
# 94 child lines. The narrow margins are for maintenance of each owner.
# Measured after closing the six fixed entries: 855 raw lines. The narrow
# margin is for maintenance of this family, not adjacent builtin work.
check_raw_line_budget "$wasm_bigint_builtins" 875
check_raw_line_budget "$wasm_bigint_radix_formatting" 120
# Measured immediately after the created-Realm WeakRef lifecycle extraction:
# 8,941 parent lines and 163 child lines. The margins are for narrow owner
# maintenance without letting the created-Realm bootstrap reinflate silently.
check_raw_line_budget "$wasm_host_builtins" 9000
check_raw_line_budget "$wasm_created_realm_weak_ref_intrinsics" 180

wasm_intl_locale="crates/lila-aot-wasm/src/builtins/intl.rs"
wasm_intl_locale_construction="crates/lila-aot-wasm/src/builtins/intl/construction_lifecycle.rs"
intl_locale_functions="crates/lila-aot-wasm/src/functions.rs"
require_file "$wasm_intl_locale_construction"
require_file "$intl_locale_functions"
check_no_inline_legacy_includes "$wasm_intl_locale"
check_no_inline_legacy_includes "$wasm_intl_locale_construction"
require_exact_line_count \
  "$wasm_intl_locale" \
  'mod construction_lifecycle;' \
  1 \
  'private Intl.Locale construction-lifecycle module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+construction_lifecycle;' "$wasm_intl_locale"; then
  fail "$wasm_intl_locale must keep construction_lifecycle private"
fi
intl_locale_production="$(awk '/^#\[cfg\(test\)\]/ { exit } { print }' "$wasm_intl_locale")"
for lifecycle_state in \
  ReservedIntlLocaleObjectLocal \
  InitializedIntlLocaleObjectLocal
do
  state_parent_count="$(printf '%s\n' "$intl_locale_production" | grep -Fc "$lifecycle_state" || true)"
  if [ "$state_parent_count" -ne 0 ]; then
    fail "$wasm_intl_locale production owner must not name $lifecycle_state (found $state_parent_count)"
  fi
  require_fixed_string_count \
    "$wasm_intl_locale_construction" \
    "$lifecycle_state" \
    4 \
    "$lifecycle_state child-only uses"
  require_regex_count \
    "$wasm_intl_locale_construction" \
    "^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+${lifecycle_state}\(u32\);" \
    1 \
    "$lifecycle_state private-field owner"
done
for lifecycle_transition in \
  emit_reserve_intl_locale_object \
  emit_initialize_intl_locale_object \
  emit_publish_intl_locale_object
do
  require_regex_count \
    "$wasm_intl_locale_construction" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${lifecycle_transition}[[:space:]]*\(" \
    1 \
    "$lifecycle_transition private-child owner"
  transition_parent_count="$(printf '%s\n' "$intl_locale_production" | grep -Fc ".${lifecycle_transition}(" || true)"
  if [ "$transition_parent_count" -ne 1 ]; then
    fail "$wasm_intl_locale production constructor must call $lifecycle_transition once (found $transition_parent_count)"
  fi
done
intl_locale_direct_returning_domain="$(awk '
  /let direct_returning_constructor_table_indices: Vec<i64> = \[/ { within_domain = 1 }
  within_domain { print }
  within_domain && /\.into_iter\(\)/ { exit }
' "$intl_locale_functions")"
intl_locale_direct_returning_count="$(printf '%s\n' "$intl_locale_direct_returning_domain" | grep -Fc 'StandardBuiltinId::IntlLocaleConstructor,' || true)"
if [ "$intl_locale_direct_returning_count" -ne 1 ]; then
  fail "$intl_locale_functions must classify Intl.Locale as direct-returning exactly once (found $intl_locale_direct_returning_count)"
fi
if ! grep -q '^enum IntlLocaleStringSlot' "$wasm_intl_locale"; then
  fail "$wasm_intl_locale must own the private Intl.Locale string-slot domain"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+IntlLocaleStringSlot' "$wasm_intl_locale"; then
  fail "$wasm_intl_locale must keep IntlLocaleStringSlot private"
fi
require_fixed_string_count \
  "$wasm_intl_locale" \
  'fn emit_intl_locale_string_slot(' \
  1 \
  'private Intl.Locale string-slot emitter'
require_fixed_string_count \
  "$wasm_intl_locale" \
  'self.emit_intl_locale_string_slot(' \
  5 \
  'fixed Intl.Locale string-slot entry call'
for intl_locale_method in language_getter script_getter region_getter base_name_getter to_string; do
  require_fixed_string_count \
    "$wasm_intl_locale" \
    "pub(super) fn emit_intl_locale_${intl_locale_method}_builtin(" \
    1 \
    "fixed Intl.Locale $intl_locale_method entry"
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "self.emit_intl_locale_${intl_locale_method}_builtin(function)?;" \
    1 \
    "fixed Intl.Locale $intl_locale_method route"
done
if grep -Eq 'IntlLocaleStringSlot|emit_intl_locale_string_slot\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Intl.Locale string entries"
fi
require_fixed_string_count \
  "$wasm_intl_locale_construction" \
  'reserved.0' \
  1 \
  'reserved Intl.Locale projection'
require_fixed_string_count \
  "$wasm_intl_locale_construction" \
  'initialized.0' \
  2 \
  'initialized Intl.Locale projections'
intl_locale_production_lines="$(awk '/^#\[cfg\(test\)\]/ { exit } { lines += 1 } END { print lines + 0 }' "$wasm_intl_locale")"
if [ "$intl_locale_production_lines" -gt 2225 ]; then
  fail "$wasm_intl_locale has $intl_locale_production_lines pre-test lines; expected at most 2225"
fi
# Measured after closing the five string-slot entries: 2,205 pre-test parent
# lines and 117 child lines. The narrow margins are for maintenance of each
# owner.
check_raw_line_budget "$wasm_intl_locale_construction" 145

wasm_intl_date_time_format="crates/lila-aot-wasm/src/builtins/intl_datetimeformat.rs"
wasm_intl_date_time_format_construction="crates/lila-aot-wasm/src/builtins/intl_datetimeformat/construction_lifecycle.rs"
require_file "$wasm_intl_date_time_format_construction"
check_no_inline_legacy_includes "$wasm_intl_date_time_format"
check_no_inline_legacy_includes "$wasm_intl_date_time_format_construction"
require_exact_line_count \
  "$wasm_intl_date_time_format" \
  'enum DtfFormatMode {' \
  1 \
  'owner-private DateTimeFormat output-mode declaration'
require_exact_line_count \
  "$wasm_intl_date_time_format" \
  'struct DtfFormatTimes {' \
  1 \
  'owner-private DateTimeFormat times declaration'
require_regex_count \
  "$wasm_intl_date_time_format" \
  '^[[:space:]]{4}fn[[:space:]]+emit_intl_dtf_build_format_with_kind[[:space:]]*\(' \
  1 \
  'owner-private DateTimeFormat raw formatter'
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+(enum[[:space:]]+DtfFormatMode|struct[[:space:]]+DtfFormatTimes|fn[[:space:]]+emit_intl_dtf_build_format_with_kind)' "$wasm_intl_date_time_format"; then
  fail "$wasm_intl_date_time_format must keep the DateTimeFormat mode, times and raw formatter owner-private"
fi
require_fixed_string_count \
  "$wasm_intl_date_time_format" \
  'self.emit_intl_dtf_build_format_with_kind(' \
  3 \
  'fixed DateTimeFormat raw formatter calls'
for intl_dtf_private_temporal_declaration in \
  'struct IntlDtfTemporalKind {' \
  'enum DtfTimeBasis {' \
  'const INTL_DTF_TEMPORAL_KINDS: &[IntlDtfTemporalKind] = &[' \
  'enum DtfBrandedKind {' \
  'enum DtfValueKind {'
do
  require_exact_line_count \
    "$wasm_intl_date_time_format" \
    "$intl_dtf_private_temporal_declaration" \
    1 \
    'owner-private DateTimeFormat Temporal-kind declaration'
done
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+(struct[[:space:]]+IntlDtfTemporalKind|enum[[:space:]]+(DtfTimeBasis|DtfBrandedKind|DtfValueKind)|const[[:space:]]+INTL_DTF_TEMPORAL_KINDS)' "$wasm_intl_date_time_format"; then
  fail "$wasm_intl_date_time_format must keep the Temporal-kind family owner-private"
fi
intl_dtf_temporal_kind_record="$(sed -n '/^struct IntlDtfTemporalKind {$/,/^}$/p' "$wasm_intl_date_time_format")"
if grep -Eq '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' <<<"$intl_dtf_temporal_kind_record"; then
  fail "$wasm_intl_date_time_format must keep every IntlDtfTemporalKind field private"
fi
require_fixed_string_count \
  "$wasm_intl_date_time_format" \
  '    IntlDtfTemporalKind {' \
  6 \
  'DateTimeFormat Temporal-kind table rows'
for intl_dtf_private_extension_declaration in \
  'enum IntlDtfExtensionResolution {' \
  'enum IntlDtfRelevantExtensionKey {'
do
  require_exact_line_count \
    "$wasm_intl_date_time_format" \
    "$intl_dtf_private_extension_declaration" \
    1 \
    'owner-private DateTimeFormat extension declaration'
done
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+(IntlDtfExtensionResolution|IntlDtfRelevantExtensionKey)' "$wasm_intl_date_time_format"; then
  fail "$wasm_intl_date_time_format must keep the extension-key domains owner-private"
fi
require_exact_line_count \
  "$wasm_intl_date_time_format" \
  '    const ALL: [Self; 3] = [Self::Ca, Self::Hc, Self::Nu];' \
  1 \
  'owner-private DateTimeFormat relevant-extension key list'
require_exact_line_count \
  "$wasm_intl_date_time_format" \
  'struct IntlDtfKeywordNeedle {' \
  1 \
  'owner-private DateTimeFormat keyword needle'
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+struct[[:space:]]+IntlDtfKeywordNeedle' "$wasm_intl_date_time_format"; then
  fail "$wasm_intl_date_time_format must keep IntlDtfKeywordNeedle owner-private"
fi
require_exact_line_count \
  "$wasm_intl_date_time_format" \
  'enum TimeZoneNameStyle {' \
  1 \
  'owner-private DateTimeFormat time-zone-name style declaration'
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+TimeZoneNameStyle' "$wasm_intl_date_time_format"; then
  fail "$wasm_intl_date_time_format must keep TimeZoneNameStyle owner-private"
fi
require_exact_line_count \
  "$wasm_intl_date_time_format" \
  'struct IntlDtfOption {' \
  1 \
  'owner-private DateTimeFormat option record'
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+(struct[[:space:]]+IntlDtfOption|const[[:space:]]+INTL_DTF_(COMPONENT_OPTIONS|HOUR_CYCLE_OPTION|DATE_STYLE_OPTION|TIME_STYLE_OPTION))' "$wasm_intl_date_time_format"; then
  fail "$wasm_intl_date_time_format must keep the DateTimeFormat option family owner-private"
fi
intl_dtf_option_record="$(sed -n '/^struct IntlDtfOption {$/,/^}$/p' "$wasm_intl_date_time_format")"
if grep -Eq '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' <<<"$intl_dtf_option_record"; then
  fail "$wasm_intl_date_time_format must keep every IntlDtfOption field private"
fi
for intl_dtf_private_time_zone_declaration in \
  'struct TzOffsetMinutes(i16);' \
  'struct IntlDtfNamedZone {' \
  'const INTL_DTF_NAMED_ZONES: &[IntlDtfNamedZone] = &[' \
  'struct DtfCanonicalTimeZone {' \
  'struct DtfResolvedTimeZone(DtfCanonicalTimeZone);'
do
  require_exact_line_count \
    "$wasm_intl_date_time_format" \
    "$intl_dtf_private_time_zone_declaration" \
    1 \
    'owner-private DateTimeFormat time-zone authority declaration'
done
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+(struct[[:space:]]+(TzOffsetMinutes|IntlDtfNamedZone|DtfCanonicalTimeZone|DtfResolvedTimeZone)|const[[:space:]]+INTL_DTF_NAMED_ZONES)' "$wasm_intl_date_time_format"; then
  fail "$wasm_intl_date_time_format must keep the DateTimeFormat time-zone authority owner-private"
fi
intl_dtf_named_zone_record="$(sed -n '/^struct IntlDtfNamedZone {$/,/^}$/p' "$wasm_intl_date_time_format")"
if grep -Eq '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' <<<"$intl_dtf_named_zone_record"; then
  fail "$wasm_intl_date_time_format must keep every IntlDtfNamedZone field private"
fi
require_exact_line_count \
  "$wasm_intl_date_time_format" \
  'mod construction_lifecycle;' \
  1 \
  'private Intl.DateTimeFormat construction-lifecycle module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+construction_lifecycle;' "$wasm_intl_date_time_format"; then
  fail "$wasm_intl_date_time_format must keep construction_lifecycle private"
fi
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'construction_lifecycle::' \
  0 \
  'Intl.DateTimeFormat construction-lifecycle imports or re-exports'
for lifecycle_state in \
  ReservedIntlDateTimeFormatObjectLocal \
  InitializedIntlDateTimeFormatObjectLocal
do
  require_fixed_string_count \
    "$wasm_intl_date_time_format" \
    "$lifecycle_state" \
    0 \
    "$lifecycle_state parent names"
  require_regex_count \
    "$wasm_intl_date_time_format_construction" \
    "^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+${lifecycle_state}\(u32\);" \
    1 \
    "$lifecycle_state private-field owner"
done
for lifecycle_transition in \
  emit_reserve_intl_date_time_format_object \
  emit_initialize_intl_date_time_format_object \
  emit_publish_intl_date_time_format_object
do
  require_regex_count \
    "$wasm_intl_date_time_format_construction" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${lifecycle_transition}[[:space:]]*\(" \
    1 \
    "$lifecycle_transition private-child owner"
  require_fixed_string_count \
    "$wasm_intl_date_time_format" \
    ".${lifecycle_transition}(" \
    1 \
    "$lifecycle_transition parent constructor call"
done
# Measured immediately after extraction: 7,093 parent lines and 94 child
# lines. The narrow margins are for maintenance of each lifecycle owner.
check_raw_line_budget "$wasm_intl_date_time_format" 7150
check_raw_line_budget "$wasm_intl_date_time_format_construction" 125

wasm_global_numeric_builtins="crates/lila-aot-wasm/src/builtins/global_numeric.rs"
check_no_inline_legacy_includes "$wasm_global_numeric_builtins"
if ! grep -q '^enum GlobalNumericBuiltin' "$wasm_global_numeric_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_global_numeric_builtins"; then
  fail "$wasm_global_numeric_builtins must dispatch through the closed GlobalNumericBuiltin domain"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+GlobalNumericBuiltin' "$wasm_global_numeric_builtins"; then
  fail "$wasm_global_numeric_builtins must keep GlobalNumericBuiltin private"
fi
if grep -Eq 'GlobalNumericBuiltin|emit_global_numeric_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed global numeric entries"
fi
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.emit_global_is_finite_builtin(function)?' \
  1 \
  'fixed global isFinite delegate'
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.emit_global_is_nan_builtin(function)?' \
  1 \
  'fixed global isNaN delegate'
require_fixed_string_count \
  "$wasm_global_numeric_builtins" \
  'self.emit_global_numeric_builtin(' \
  2 \
  'private global numeric producer calls'
# Measured immediately after extraction: 51 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_global_numeric_builtins" 90

wasm_symbol_builtins="crates/lila-aot-wasm/src/builtins/symbol.rs"
check_no_inline_legacy_includes "$wasm_symbol_builtins"
if ! grep -q '^enum SymbolBuiltin' "$wasm_symbol_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_symbol_builtins"; then
  fail "$wasm_symbol_builtins must dispatch through the closed SymbolBuiltin domain"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+SymbolBuiltin' "$wasm_symbol_builtins"; then
  fail "$wasm_symbol_builtins must keep SymbolBuiltin private"
fi
if grep -Eq 'SymbolBuiltin|SymbolFn|emit_symbol\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Symbol entries"
fi
require_fixed_string_count "$wasm_symbol_builtins" 'fn emit_symbol(' 1 'private Symbol emitter'
require_fixed_string_count "$wasm_symbol_builtins" 'self.emit_symbol(' 7 'fixed Symbol entry calls'
# Measured immediately after extraction: 518 raw lines. The narrow margin is
# for maintenance of this family, not adjacent builtin implementations.
check_raw_line_budget "$wasm_symbol_builtins" 550

wasm_uri_builtins="crates/lila-aot-wasm/src/builtins/uri.rs"
check_no_inline_legacy_includes "$wasm_uri_builtins"
if ! grep -q '^enum UriBuiltin' "$wasm_uri_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_uri_builtins"; then
  fail "$wasm_uri_builtins must privately dispatch through the closed UriBuiltin domain"
fi
if grep -Fq 'UriBuiltin' "$wasm_standard_builtins" \
  || grep -Fq 'self.emit_uri_builtin(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed URI operations instead of the raw policy"
fi
require_fixed_string_count "$wasm_uri_builtins" 'fn emit_uri_builtin(' 1 'private URI compiler'
require_fixed_string_count "$wasm_uri_builtins" 'self.emit_uri_builtin(' 6 'fixed URI wrapper calls'
require_regex_count \
  "$wasm_uri_builtins" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_(escape|unescape|encode_uri|encode_uri_component|decode_uri|decode_uri_component)_builtin[[:space:]]*\(' \
  6 \
  'fixed URI semantic wrappers'
for uri_wrapper in \
  emit_escape_builtin \
  emit_unescape_builtin \
  emit_encode_uri_builtin \
  emit_encode_uri_component_builtin \
  emit_decode_uri_builtin \
  emit_decode_uri_component_builtin
do
  require_fixed_string_count "$wasm_standard_builtins" "self.${uri_wrapper}(function)?" 1 "URI dispatcher call to $uri_wrapper"
done
check_raw_line_budget "$wasm_uri_builtins" 165

wasm_error_builtins="crates/lila-aot-wasm/src/builtins/errors.rs"
if ! grep -q '^enum ErrorBuiltin' "$wasm_error_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_error_builtins"; then
  fail "$wasm_error_builtins must privately dispatch through the closed ErrorBuiltin domain"
fi
if grep -Eq 'ErrorBuiltin|NativeErrorKind|self\.emit_error_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Error-family operations instead of the raw policy"
fi
require_fixed_string_count "$wasm_error_builtins" 'fn emit_error_builtin(' 1 'private Error-family compiler'
require_fixed_string_count "$wasm_error_builtins" 'self.emit_error_builtin(' 11 'fixed Error-family entry calls'
for error_wrapper in \
  emit_error_constructor_builtin \
  emit_error_is_error_builtin \
  emit_eval_error_constructor_builtin \
  emit_aggregate_error_constructor_builtin \
  emit_suppressed_error_constructor_builtin \
  emit_range_error_constructor_builtin \
  emit_syntax_error_constructor_builtin \
  emit_type_error_constructor_builtin \
  emit_uri_error_constructor_builtin \
  emit_reference_error_constructor_builtin \
  emit_error_prototype_to_string_builtin
do
  require_fixed_string_count "$wasm_error_builtins" "pub(super) fn ${error_wrapper}(" 1 "fixed Error-family entry $error_wrapper"
  require_fixed_string_count "$wasm_standard_builtins" "self.${error_wrapper}(function)?" 1 "standard call to fixed Error-family entry $error_wrapper"
done
wasm_aggregate_error_preparation="crates/lila-aot-wasm/src/builtins/errors/aggregate_error_preparation.rs"
require_file "$wasm_aggregate_error_preparation"
require_exact_line_count \
  "$wasm_error_builtins" \
  'mod aggregate_error_preparation;' \
  1 \
  'private AggregateError preparation module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+aggregate_error_preparation;' "$wasm_error_builtins"; then
  fail "$wasm_error_builtins must keep aggregate_error_preparation private"
fi
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'aggregate_error_preparation::' \
  0 \
  'AggregateError preparation imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+PreparedAggregateErrorLocal[[:space:]]*\{' \
  1 \
  'prepared AggregateError witness owner'
require_fixed_string_count \
  "$wasm_error_builtins" \
  'PreparedAggregateErrorLocal' \
  0 \
  'prepared AggregateError parent names'
require_exact_line_count \
  "$wasm_aggregate_error_preparation" \
  '    object: u32,' \
  1 \
  'private prepared AggregateError object field'
require_fixed_string_count \
  "$wasm_aggregate_error_preparation" \
  'Ok(PreparedAggregateErrorLocal {' \
  2 \
  'prepared AggregateError construction sites'
require_fixed_string_count \
  "$wasm_aggregate_error_preparation" \
  'let PreparedAggregateErrorLocal {' \
  1 \
  'prepared AggregateError consuming projection'

for aggregate_error_preparation_method in \
  emit_prepare_aggregate_error_instance \
  emit_prepare_promise_any_aggregate_error_instance \
  emit_finish_aggregate_error_instance
do
  require_regex_count \
    "$wasm_aggregate_error_preparation" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+${aggregate_error_preparation_method}[[:space:]]*\\(" \
    1 \
    "$aggregate_error_preparation_method private-child owner"
  require_regex_count \
    "$wasm_error_builtins" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${aggregate_error_preparation_method}[[:space:]]*\\(" \
    0 \
    "$aggregate_error_preparation_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${aggregate_error_preparation_method}[[:space:]]*\\(" \
    1 \
    "$aggregate_error_preparation_method backend owner"
done

for aggregate_error_preparation_call_census in \
  'emit_prepare_aggregate_error_instance 1' \
  'emit_prepare_promise_any_aggregate_error_instance 1' \
  'emit_finish_aggregate_error_instance 2'
do
  set -- $aggregate_error_preparation_call_census
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "\\.${1}[[:space:]]*\\(" \
    "$2" \
    "prepared AggregateError $1 calls"
done

check_no_inline_legacy_includes "$wasm_error_builtins"
check_no_inline_legacy_includes "$wasm_aggregate_error_preparation"
if ! grep -q '^enum ErrorBuiltin' "$wasm_error_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_error_builtins" \
  || ! grep -q '^            ErrorBuiltin::Constructor(error_kind) => match error_kind {' "$wasm_error_builtins"; then
  fail "$wasm_error_builtins must privately dispatch through the closed ErrorBuiltin domain"
fi
# Measured immediately after extracting the prepared AggregateError lifecycle:
# 1,443 parent lines and 118 child lines. Batch AQ adds 101 lines for eleven
# fixed semantic entries. The narrow margins are for maintenance of these
# families, not adjacent builtin implementations.
check_raw_line_budget "$wasm_error_builtins" 1590
check_raw_line_budget "$wasm_aggregate_error_preparation" 150

wasm_promise_builtins="crates/lila-aot-wasm/src/builtins/promise.rs"
wasm_promise_internal_function_materialization="crates/lila-aot-wasm/src/builtins/promise/promise_internal_function_materialization.rs"
require_file "$wasm_promise_internal_function_materialization"
check_no_inline_legacy_includes "$wasm_promise_internal_function_materialization"
require_exact_line_count \
  "$wasm_promise_builtins" \
  'mod promise_internal_function_materialization;' \
  1 \
  'private Promise internal-function materialization module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+promise_internal_function_materialization;' "$wasm_promise_builtins"; then
  fail "$wasm_promise_builtins must keep promise_internal_function_materialization private"
fi
require_exact_line_count \
  "$wasm_promise_builtins" \
  'use self::promise_internal_function_materialization::PromiseInternalFunctionMaterializationContext;' \
  1 \
  'private Promise internal-function carrier import'
if grep -Eq '^pub([^[:space:]]*[[:space:]]+)?use[[:space:]]+.*promise_internal_function_materialization' "$wasm_promise_builtins"; then
  fail "$wasm_promise_builtins must not re-export the Promise internal-function carrier"
fi
if ! grep -q '^pub(super) struct PromiseInternalFunctionMaterializationContext {$' "$wasm_promise_internal_function_materialization" \
  || grep -Eq '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+(realm_local|function_prototype_local|type_error_prototype_local|range_error_prototype_local):' "$wasm_promise_internal_function_materialization"; then
  fail "$wasm_promise_internal_function_materialization must own the opaque Promise internal-function carrier with private fields"
fi
require_fixed_string_count \
  "$wasm_promise_internal_function_materialization" \
  'PromiseInternalFunctionMaterializationContext' \
  8 \
  'Promise internal-function carrier child sites'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'PromiseInternalFunctionMaterializationContext' \
  11 \
  'Promise internal-function carrier recursive sites'
for promise_internal_method_and_count in \
  'emit_promise_internal_function_materialization_context_from_realm 4' \
  'emit_current_function_promise_internal_function_materialization_context 7' \
  'emit_promise_record_internal_function_materialization_context 2' \
  'emit_promise_internal_function_value 11' \
  'emit_load_promise_internal_function_context 9' \
  'release_promise_internal_function_materialization_context 9' \
  'emit_load_promise_internal_function_realm_intrinsics 2'
do
  promise_internal_method="${promise_internal_method_and_count% *}"
  promise_internal_count="${promise_internal_method_and_count##* }"
  require_regex_count \
    "$wasm_promise_internal_function_materialization" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+$promise_internal_method[[:space:]]*\(" \
    1 \
    "Promise internal-function owner method $promise_internal_method"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "$promise_internal_method[[:space:]]*\(" \
    "$promise_internal_count" \
    "Promise internal-function recursive method census $promise_internal_method"
done
if grep -q 'materialization_context.realm_local' crates/lila-aot-wasm/src/builtins/promise/promise_resolve_realm_context.rs; then
  fail "PromiseResolve must load the materialization Realm only through the child-owned capability"
fi
wasm_promise_try_callback_type_error="crates/lila-aot-wasm/src/builtins/promise/promise_try_callback_type_error.rs"
require_file "$wasm_promise_try_callback_type_error"
require_exact_line_count \
  "$wasm_promise_builtins" \
  'mod promise_try_callback_type_error;' \
  1 \
  'private Promise.try callback TypeError module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+promise_try_callback_type_error;' "$wasm_promise_builtins"; then
  fail "$wasm_promise_builtins must keep promise_try_callback_type_error private"
fi
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'promise_try_callback_type_error::' \
  0 \
  'Promise.try callback TypeError imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+PromiseTryCallbackTypeErrorPrototypeLocal\(u32\);' \
  1 \
  'Promise.try callback TypeError proof owner'
require_fixed_string_count \
  "$wasm_promise_builtins" \
  'PromiseTryCallbackTypeErrorPrototypeLocal' \
  0 \
  'Promise.try callback TypeError parent names'
require_fixed_string_count \
  "$wasm_promise_try_callback_type_error" \
  'PromiseTryCallbackTypeErrorPrototypeLocal(prototype_local)' \
  1 \
  'Promise.try callback TypeError proof construction sites'
require_fixed_string_count \
  "$wasm_promise_try_callback_type_error" \
  'prototype.0' \
  2 \
  'Promise.try callback TypeError proof projections'

for promise_try_callback_type_error_method in \
  emit_load_promise_try_callback_type_error_prototype \
  emit_throw_promise_try_non_callable_callback
do
  require_regex_count \
    "$wasm_promise_try_callback_type_error" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+${promise_try_callback_type_error_method}[[:space:]]*\\(" \
    1 \
    "$promise_try_callback_type_error_method private-child owner"
  require_regex_count \
    "$wasm_promise_builtins" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${promise_try_callback_type_error_method}[[:space:]]*\\(" \
    0 \
    "$promise_try_callback_type_error_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${promise_try_callback_type_error_method}[[:space:]]*\\(" \
    1 \
    "$promise_try_callback_type_error_method backend owner"
  require_fixed_string_count \
    "$wasm_promise_builtins" \
    ".${promise_try_callback_type_error_method}(" \
    1 \
    "$promise_try_callback_type_error_method Promise.try call"
done

check_no_inline_legacy_includes "$wasm_promise_builtins"
check_no_inline_legacy_includes "$wasm_promise_try_callback_type_error"
# Measured after extracting the Promise internal-function materialization
# authority: 7,111 parent lines and 212 child lines. The narrow margins are for
# maintenance of each owner, not adjacent Promise implementations.
check_raw_line_budget "$wasm_promise_builtins" 7180
check_raw_line_budget "$wasm_promise_internal_function_materialization" 240
check_raw_line_budget "$wasm_promise_try_callback_type_error" 80

wasm_promise_prototype_receiver_type_error="crates/lila-aot-wasm/src/builtins/promise/promise_prototype_receiver_type_error.rs"
require_file "$wasm_promise_prototype_receiver_type_error"
require_exact_line_count \
  "$wasm_promise_builtins" \
  'mod promise_prototype_receiver_type_error;' \
  1 \
  'private Promise prototype receiver TypeError module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+promise_prototype_receiver_type_error;' "$wasm_promise_builtins"; then
  fail "$wasm_promise_builtins must keep promise_prototype_receiver_type_error private"
fi
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'promise_prototype_receiver_type_error::' \
  0 \
  'Promise prototype receiver TypeError imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+PromisePrototypeReceiverTypeErrorPrototypeLocal\(u32\);' \
  1 \
  'Promise prototype receiver TypeError proof owner'
require_fixed_string_count \
  "$wasm_promise_builtins" \
  'PromisePrototypeReceiverTypeErrorPrototypeLocal' \
  0 \
  'Promise prototype receiver TypeError parent names'
require_fixed_string_count \
  "$wasm_promise_builtins" \
  'PromisePrototypeReceiverError' \
  0 \
  'Promise prototype receiver raw error policy parent names'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'PromisePrototypeReceiverError' \
  5 \
  'Promise prototype receiver raw error policy private-child uses'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*enum[[:space:]]+PromisePrototypeReceiverError[[:space:]]*\{' \
  1 \
  'Promise prototype receiver raw error policy private-child owner'
require_fixed_string_count \
  "$wasm_promise_prototype_receiver_type_error" \
  'PromisePrototypeReceiverTypeErrorPrototypeLocal(prototype_local)' \
  1 \
  'Promise prototype receiver TypeError proof construction sites'
require_fixed_string_count \
  "$wasm_promise_prototype_receiver_type_error" \
  'prototype.0' \
  2 \
  'Promise prototype receiver TypeError proof projections'

require_regex_count \
  "$wasm_promise_prototype_receiver_type_error" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_load_promise_prototype_receiver_type_error_prototype[[:space:]]*\(' \
  1 \
  'Promise prototype receiver TypeError proof factory private-child owner'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*(pub(\([^)]*\))[[:space:]]+)?fn[[:space:]]+emit_load_promise_prototype_receiver_type_error_prototype[[:space:]]*\(' \
  1 \
  'Promise prototype receiver TypeError proof factory backend owner'
require_fixed_string_count \
  "$wasm_promise_builtins" \
  '.emit_load_promise_prototype_receiver_type_error_prototype(' \
  2 \
  'Promise prototype receiver TypeError proof factory then/finally calls'
require_regex_count \
  "$wasm_promise_prototype_receiver_type_error" \
  '^[[:space:]]*fn[[:space:]]+emit_throw_promise_prototype_receiver_error[[:space:]]*\(' \
  1 \
  'Promise prototype receiver raw error consumer private-child owner'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*(pub(\([^)]*\))[[:space:]]+)?fn[[:space:]]+emit_throw_promise_prototype_receiver_error[[:space:]]*\(' \
  1 \
  'Promise prototype receiver raw error consumer backend owner'
require_fixed_string_count \
  "$wasm_promise_prototype_receiver_type_error" \
  'emit_throw_promise_prototype_receiver_error(' \
  3 \
  'Promise prototype receiver raw consumer and semantic wrapper calls'
require_fixed_string_count \
  "$wasm_promise_builtins" \
  'emit_throw_promise_prototype_receiver_error(' \
  0 \
  'Promise prototype receiver raw error parent calls'

for promise_prototype_receiver_error_wrapper in \
  emit_throw_promise_then_incompatible_receiver_error \
  emit_throw_promise_finally_non_object_receiver_error
do
  require_regex_count \
    "$wasm_promise_prototype_receiver_type_error" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+${promise_prototype_receiver_error_wrapper}[[:space:]]*\\(" \
    1 \
    "$promise_prototype_receiver_error_wrapper private-child owner"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))[[:space:]]+)?fn[[:space:]]+${promise_prototype_receiver_error_wrapper}[[:space:]]*\\(" \
    1 \
    "$promise_prototype_receiver_error_wrapper backend owner"
  require_fixed_string_count \
    "$wasm_promise_builtins" \
    ".${promise_prototype_receiver_error_wrapper}(" \
    1 \
    "$promise_prototype_receiver_error_wrapper sole semantic caller"
done

check_no_inline_legacy_includes "$wasm_promise_prototype_receiver_type_error"
# Measured after closing the raw diagnostic-policy boundary: 101 raw lines. The
# narrow margin is for maintenance of this proof lifecycle, not adjacent Promise
# implementations.
check_raw_line_budget "$wasm_promise_prototype_receiver_type_error" 120

wasm_promise_prototype_then_invocation="crates/lila-aot-wasm/src/builtins/promise/promise_prototype_then_invocation.rs"
require_file "$wasm_promise_prototype_then_invocation"
require_exact_line_count \
  "$wasm_promise_builtins" \
  'mod promise_prototype_then_invocation;' \
  1 \
  'private Promise prototype then-invocation module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+promise_prototype_then_invocation;' "$wasm_promise_builtins"; then
  fail "$wasm_promise_builtins must keep promise_prototype_then_invocation private"
fi
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'promise_prototype_then_invocation::' \
  0 \
  'Promise prototype then-invocation imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+ValidatedPromisePrototypeThenInvocationLocals[[:space:]]*\{' \
  1 \
  'validated Promise prototype then-invocation carrier owner'
require_fixed_string_count \
  "$wasm_promise_builtins" \
  'ValidatedPromisePrototypeThenInvocationLocals' \
  0 \
  'validated Promise prototype then-invocation parent names'
require_fixed_string_count \
  "$wasm_promise_prototype_then_invocation" \
  'ValidatedPromisePrototypeThenInvocationLocals' \
  5 \
  'validated Promise prototype then-invocation carrier uses'
require_fixed_string_count \
  "$wasm_promise_prototype_then_invocation" \
  'Ok(ValidatedPromisePrototypeThenInvocationLocals { method, receiver })' \
  1 \
  'validated Promise prototype then-invocation construction sites'
require_fixed_string_count \
  "$wasm_promise_prototype_then_invocation" \
  'let ValidatedPromisePrototypeThenInvocationLocals { method, receiver }' \
  1 \
  'validated Promise prototype then-invocation consuming projections'

for promise_prototype_then_invocation_method in \
  emit_validate_promise_prototype_then_invocation \
  emit_call_validated_promise_prototype_then_invocation
do
  require_regex_count \
    "$wasm_promise_prototype_then_invocation" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+${promise_prototype_then_invocation_method}[[:space:]]*\\(" \
    1 \
    "$promise_prototype_then_invocation_method private-child owner"
  require_regex_count \
    "$wasm_promise_builtins" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${promise_prototype_then_invocation_method}[[:space:]]*\\(" \
    0 \
    "$promise_prototype_then_invocation_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${promise_prototype_then_invocation_method}[[:space:]]*\\(" \
    1 \
    "$promise_prototype_then_invocation_method backend owner"
  require_fixed_string_count \
    "$wasm_promise_builtins" \
    ".${promise_prototype_then_invocation_method}(" \
    2 \
    "$promise_prototype_then_invocation_method catch/finally calls"
done

check_no_inline_legacy_includes "$wasm_promise_prototype_then_invocation"
# Measured immediately after extraction: 55 raw lines. The narrow margin is for
# maintenance of this carrier lifecycle, not adjacent Promise implementations.
check_raw_line_budget "$wasm_promise_prototype_then_invocation" 80

wasm_promise_settlement_record_allocation="crates/lila-aot-wasm/src/builtins/promise/promise_settlement_record_allocation.rs"
require_file "$wasm_promise_settlement_record_allocation"
require_exact_line_count \
  "$wasm_promise_builtins" \
  'mod promise_settlement_record_allocation;' \
  1 \
  'private Promise settlement-record allocation module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+promise_settlement_record_allocation;' "$wasm_promise_builtins"; then
  fail "$wasm_promise_builtins must keep promise_settlement_record_allocation private"
fi
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'promise_settlement_record_allocation::' \
  0 \
  'Promise settlement-record allocation imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+PromiseSettlementRecordAllocationContext[[:space:]]*\{' \
  1 \
  'Promise settlement-record allocation context owner'
require_fixed_string_count \
  "$wasm_promise_builtins" \
  'PromiseSettlementRecordAllocationContext' \
  0 \
  'Promise settlement-record allocation parent names'
require_fixed_string_count \
  "$wasm_promise_settlement_record_allocation" \
  'PromiseSettlementRecordAllocationContext' \
  4 \
  'Promise settlement-record allocation context uses'
require_fixed_string_count \
  "$wasm_promise_settlement_record_allocation" \
  'PromiseSettlementRecordAllocationContext { prototype_local }' \
  1 \
  'Promise settlement-record allocation context construction sites'
require_fixed_string_count \
  "$wasm_promise_settlement_record_allocation" \
  'context.prototype_local' \
  2 \
  'Promise settlement-record allocation context projections'

for promise_settlement_record_allocation_method in \
  emit_self_backed_promise_settlement_record_allocation_context \
  emit_alloc_promise_settlement_record
do
  require_regex_count \
    "$wasm_promise_settlement_record_allocation" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+${promise_settlement_record_allocation_method}[[:space:]]*\\(" \
    1 \
    "$promise_settlement_record_allocation_method private-child owner"
  require_regex_count \
    "$wasm_promise_builtins" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${promise_settlement_record_allocation_method}[[:space:]]*\\(" \
    0 \
    "$promise_settlement_record_allocation_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${promise_settlement_record_allocation_method}[[:space:]]*\\(" \
    1 \
    "$promise_settlement_record_allocation_method backend owner"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "\\.${promise_settlement_record_allocation_method}\\(" \
    2 \
    "$promise_settlement_record_allocation_method standard/keyed calls"
done

check_no_inline_legacy_includes "$wasm_promise_settlement_record_allocation"
# Measured immediately after extraction: 70 raw lines. The narrow margin is for
# maintenance of this allocation lifecycle, not adjacent Promise implementations.
check_raw_line_budget "$wasm_promise_settlement_record_allocation" 100

wasm_promise_with_resolvers_result_allocation="crates/lila-aot-wasm/src/builtins/promise/promise_with_resolvers_result_allocation.rs"
require_file "$wasm_promise_with_resolvers_result_allocation"
require_exact_line_count \
  "$wasm_promise_builtins" \
  'mod promise_with_resolvers_result_allocation;' \
  1 \
  'private Promise.withResolvers result-allocation module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+promise_with_resolvers_result_allocation;' "$wasm_promise_builtins"; then
  fail "$wasm_promise_builtins must keep promise_with_resolvers_result_allocation private"
fi
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'promise_with_resolvers_result_allocation::' \
  0 \
  'Promise.withResolvers result-allocation imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+PromiseWithResolversResultAllocationContext[[:space:]]*\{' \
  1 \
  'Promise.withResolvers result-allocation context owner'
require_fixed_string_count \
  "$wasm_promise_builtins" \
  'PromiseWithResolversResultAllocationContext' \
  0 \
  'Promise.withResolvers result-allocation parent names'
require_fixed_string_count \
  "$wasm_promise_with_resolvers_result_allocation" \
  'PromiseWithResolversResultAllocationContext' \
  4 \
  'Promise.withResolvers result-allocation context uses'
require_fixed_string_count \
  "$wasm_promise_with_resolvers_result_allocation" \
  'PromiseWithResolversResultAllocationContext { prototype_local }' \
  1 \
  'Promise.withResolvers result-allocation context construction sites'
require_fixed_string_count \
  "$wasm_promise_with_resolvers_result_allocation" \
  'context.prototype_local' \
  2 \
  'Promise.withResolvers result-allocation context projections'

for promise_with_resolvers_result_allocation_method in \
  emit_current_function_promise_with_resolvers_result_allocation_context \
  emit_install_promise_with_resolvers_result_prototype
do
  require_regex_count \
    "$wasm_promise_with_resolvers_result_allocation" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+${promise_with_resolvers_result_allocation_method}[[:space:]]*\\(" \
    1 \
    "$promise_with_resolvers_result_allocation_method private-child owner"
  require_regex_count \
    "$wasm_promise_builtins" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${promise_with_resolvers_result_allocation_method}[[:space:]]*\\(" \
    0 \
    "$promise_with_resolvers_result_allocation_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${promise_with_resolvers_result_allocation_method}[[:space:]]*\\(" \
    1 \
    "$promise_with_resolvers_result_allocation_method backend owner"
  require_fixed_string_count \
    "$wasm_promise_builtins" \
    ".${promise_with_resolvers_result_allocation_method}(" \
    1 \
    "$promise_with_resolvers_result_allocation_method Promise.withResolvers call"
done

check_no_inline_legacy_includes "$wasm_promise_with_resolvers_result_allocation"
# Measured immediately after extraction: 83 raw lines. The narrow margin is for
# maintenance of this allocation lifecycle, not adjacent Promise implementations.
check_raw_line_budget "$wasm_promise_with_resolvers_result_allocation" 110

wasm_json_builtins="crates/lila-aot-wasm/src/builtins/json.rs"
wasm_json_parse_frame_state="crates/lila-aot-wasm/src/builtins/json/parse_frame_state.rs"
require_file "$wasm_json_parse_frame_state"
check_no_inline_legacy_includes "$wasm_json_builtins"
check_no_inline_legacy_includes "$wasm_json_parse_frame_state"
require_exact_line_count \
  "$wasm_json_builtins" \
  'mod parse_frame_state;' \
  1 \
  'private JSON parse-frame-state module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+parse_frame_state;' "$wasm_json_builtins"; then
  fail "$wasm_json_builtins must keep parse_frame_state private"
fi
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'parse_frame_state::' \
  0 \
  'JSON parse-frame-state imports or re-exports'
require_fixed_string_count \
  "$wasm_json_builtins" \
  'ValidatedJsonParseFrameStateLocal' \
  0 \
  'JSON parse-frame-state parent carrier names'
require_fixed_string_count \
  "$wasm_json_parse_frame_state" \
  'ValidatedJsonParseFrameStateLocal' \
  7 \
  'JSON parse-frame-state child carrier names'
if ! grep -q '^pub(super) struct ValidatedJsonParseFrameStateLocal(u32);$' "$wasm_json_parse_frame_state" \
  || grep -q '^pub(super) struct ValidatedJsonParseFrameStateLocal(pub' "$wasm_json_parse_frame_state"; then
  fail "$wasm_json_parse_frame_state must own the sibling-visible carrier with a private tuple field"
fi
require_fixed_string_count \
  "$wasm_json_parse_frame_state" \
  'self.0' \
  2 \
  'JSON parse-frame-state raw carrier projections'
for json_parse_frame_method_and_count in \
  'emit_validate_json_parse_frame_state_local 5' \
  'emit_json_parse_frame_state_is_i32 9' \
  'emit_push_json_parse_frame 4' \
  'release_validated_json_parse_frame_state_local 2'
do
  json_parse_frame_method="${json_parse_frame_method_and_count% *}"
  json_parse_frame_count="${json_parse_frame_method_and_count##* }"
  require_regex_count \
    "$wasm_json_parse_frame_state" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+$json_parse_frame_method[[:space:]]*\(" \
    1 \
    "JSON parse-frame-state child method $json_parse_frame_method"
  require_regex_count \
    "$wasm_json_builtins" \
    "^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+$json_parse_frame_method[[:space:]]*\(" \
    0 \
    "JSON parse-frame-state parent method $json_parse_frame_method"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "$json_parse_frame_method[[:space:]]*\(" \
    "$json_parse_frame_count" \
    "JSON parse-frame-state recursive method census $json_parse_frame_method"
done
if grep -q 'frame_state.into_local()' "$wasm_json_builtins"; then
  fail "$wasm_json_builtins must release validated JSON parse-frame state through the child owner"
fi
if ! grep -q '^enum JsonBuiltin' "$wasm_json_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_json_builtins"; then
  fail "$wasm_json_builtins must dispatch through the closed JsonBuiltin domain"
fi
if grep -Eq 'JsonBuiltin|emit_json_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must not import, construct or call the private raw JSON builtin policy"
fi
require_fixed_string_count "$wasm_json_builtins" 'JsonBuiltin' 10 'private JSON builtin policy census'
for json_builtin_wrapper in \
  emit_json_parse_builtin \
  emit_json_stringify_builtin \
  emit_json_raw_json_builtin \
  emit_json_is_raw_json_builtin
do
  require_fixed_string_count \
    "$wasm_json_builtins" \
    "pub(super) fn ${json_builtin_wrapper}(" \
    1 \
    "private fixed JSON wrapper $json_builtin_wrapper"
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "self.${json_builtin_wrapper}(function)?" \
    1 \
    "standard JSON route $json_builtin_wrapper"
done
require_fixed_string_count \
  "$wasm_json_builtins" \
  'self.emit_json_builtin(' \
  4 \
  'private fixed JSON wrapper producers'
# Measured after extracting the validated JSON parse-frame-state lifecycle:
# 8,307 parent lines and 184 child lines. The narrow margins are for maintenance
# of each owner, not adjacent JSON implementations.
check_raw_line_budget "$wasm_json_builtins" 8380
check_raw_line_budget "$wasm_json_parse_frame_state" 220

# T02's Map/WeakMap get-or-insert owner. The four crate-visible semantic entry
# points remain product-callable, but only the private child may construct the
# raw value-source policy or call the shared parameterized emitter.
wasm_collections_builtins="crates/lila-aot-wasm/src/builtins/collections.rs"
wasm_map_get_or_insert="crates/lila-aot-wasm/src/builtins/collections/map_get_or_insert.rs"
require_file "$wasm_map_get_or_insert"
check_no_inline_legacy_includes "$wasm_collections_builtins"
check_no_inline_legacy_includes "$wasm_map_get_or_insert"
require_exact_line_count \
  "$wasm_collections_builtins" \
  'mod map_get_or_insert;' \
  1 \
  'private Map get-or-insert module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+map_get_or_insert;' "$wasm_collections_builtins"; then
  fail "$wasm_collections_builtins must keep map_get_or_insert private"
fi
if grep -Eq 'MapGetOrInsertValueSource|emit_map_prototype_get_or_insert_inner\(|map_get_or_insert::' "$wasm_collections_builtins"; then
  fail "$wasm_collections_builtins must not name, construct, project or import the private Map get-or-insert policy"
fi
require_fixed_string_count \
  "$wasm_map_get_or_insert" \
  'MapGetOrInsertValueSource' \
  10 \
  'Map get-or-insert value-source owner lines'
require_fixed_string_count \
  "$wasm_map_get_or_insert" \
  'MapGetOrInsertValueSource::' \
  8 \
  'Map get-or-insert qualified value-source uses'
require_fixed_string_count \
  "$wasm_map_get_or_insert" \
  'emit_map_prototype_get_or_insert_inner(' \
  5 \
  'private Map get-or-insert emitter definition and calls'
for semantic_get_or_insert in \
  emit_map_prototype_get_or_insert \
  emit_map_prototype_get_or_insert_computed \
  emit_weak_map_prototype_get_or_insert \
  emit_weak_map_prototype_get_or_insert_computed
do
  require_regex_count \
    "$wasm_map_get_or_insert" \
    "^[[:space:]]*pub\\(crate\\)[[:space:]]+fn[[:space:]]+$semantic_get_or_insert[[:space:]]*\\(" \
    1 \
    "Map get-or-insert semantic surface $semantic_get_or_insert"
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "self.$semantic_get_or_insert(function)?;" \
    1 \
    "Map get-or-insert product call $semantic_get_or_insert"
done
# Measured immediately after extraction: 6,491 parent lines and 322 child
# lines. The narrow margins are for maintenance of each owner.
check_raw_line_budget "$wasm_collections_builtins" 6560
check_raw_line_budget "$wasm_map_get_or_insert" 360

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

wasm_temporal_plain_date="crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs"
require_exact_line_count \
  "$wasm_temporal_plain_date" \
  'enum TemporalCalendarCarrier {' \
  1 \
  'owner-private Temporal calendar-carrier declaration'
require_exact_line_count \
  "$wasm_temporal_plain_date" \
  '    fn emit_temporal_calendar_slot_fast_path(' \
  1 \
  'owner-private Temporal calendar-slot raw fast path'
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+(enum[[:space:]]+TemporalCalendarCarrier|fn[[:space:]]+emit_temporal_calendar_slot_fast_path)' "$wasm_temporal_plain_date"; then
  fail "$wasm_temporal_plain_date must keep the Temporal calendar carrier and raw fast path owner-private"
fi
temporal_calendar_carrier_impl="$(sed -n '/^impl TemporalCalendarCarrier {$/,/^}$/p' "$wasm_temporal_plain_date")"
if grep -Eq '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' <<<"$temporal_calendar_carrier_impl"; then
  fail "$wasm_temporal_plain_date must keep every TemporalCalendarCarrier projection private"
fi
require_fixed_string_count \
  "$wasm_temporal_plain_date" \
  'self.emit_temporal_calendar_slot_fast_path(' \
  2 \
  'fixed Temporal calendar-slot raw fast-path calls'
wasm_temporal_plain_month_day="crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs"
require_exact_line_count \
  "$wasm_temporal_plain_month_day" \
  'struct TemporalParsedMonthDayYear {' \
  1 \
  'owner-private Temporal.PlainMonthDay parsed-year carrier'
require_exact_line_count \
  "$wasm_temporal_plain_month_day" \
  '    fn emit_temporal_parse_month_day_string(' \
  1 \
  'owner-private Temporal.PlainMonthDay raw parser'
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+(struct[[:space:]]+TemporalParsedMonthDayYear|fn[[:space:]]+emit_temporal_parse_month_day_string)' "$wasm_temporal_plain_month_day"; then
  fail "$wasm_temporal_plain_month_day must keep its parsed-year carrier and raw parser owner-private"
fi
require_exact_line_count \
  "$wasm_temporal_plain_month_day" \
  '        let parsed = self.emit_temporal_parse_month_day_string(' \
  1 \
  'fixed Temporal.PlainMonthDay raw parser call'
wasm_temporal_duration="crates/lila-aot-wasm/src/builtins/temporal_duration.rs"
wasm_temporal_plain_date_time="crates/lila-aot-wasm/src/builtins/temporal_plain_date_time.rs"
require_exact_line_count \
  "$wasm_temporal_duration" \
  'const TEMPORAL_DURATION_FIELD_OFFSETS: [u64; 10] = [' \
  1 \
  'owner-private Temporal.Duration field-offset table'
require_exact_line_count \
  "$wasm_temporal_plain_date_time" \
  'const TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS: [u64; 9] = [' \
  1 \
  'owner-private Temporal.PlainDateTime field-offset table'
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+const[[:space:]]+TEMPORAL_DURATION_FIELD_OFFSETS' "$wasm_temporal_duration"; then
  fail "$wasm_temporal_duration must keep its field-offset table owner-private"
fi
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+const[[:space:]]+TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS' "$wasm_temporal_plain_date_time"; then
  fail "$wasm_temporal_plain_date_time must keep its field-offset table owner-private"
fi
require_fixed_string_count \
  "$wasm_temporal_duration" \
  'TEMPORAL_DURATION_FIELD_OFFSETS' \
  3 \
  'Temporal.Duration field-offset declaration and consumers'
require_fixed_string_count \
  "$wasm_temporal_plain_date_time" \
  'TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS' \
  3 \
  'Temporal.PlainDateTime field-offset declaration and consumers'
wasm_temporal_instant="crates/lila-aot-wasm/src/builtins/temporal_instant.rs"
for instant_diagnostic in \
  TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE \
  TEMPORAL_INSTANT_VALUE_OF_MESSAGE
do
  require_regex_count \
    "$wasm_temporal_instant" \
    "^const[[:space:]]+${instant_diagnostic}:[[:space:]]*&str[[:space:]]*=" \
    1 \
    "owner-private Temporal.Instant diagnostic $instant_diagnostic"
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+const[[:space:]]+${instant_diagnostic}" "$wasm_temporal_instant"; then
    fail "$wasm_temporal_instant must keep $instant_diagnostic owner-private"
  fi
  require_fixed_string_count \
    "$wasm_temporal_instant" \
    "$instant_diagnostic" \
    2 \
    "Temporal.Instant diagnostic declaration and consumer $instant_diagnostic"
done

wasm_string_trim="crates/lila-aot-wasm/src/operations/string_trim.rs"
require_exact_line_count \
  "$wasm_string_trim" \
  'const ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8: [&[u8]; 19] = [' \
  1 \
  'owner-private ECMAScript non-ASCII whitespace table'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+const[[:space:]]+ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8' "$wasm_string_trim"; then
  fail "$wasm_string_trim must keep its ECMAScript whitespace table owner-private"
fi
require_fixed_string_count \
  "$wasm_string_trim" \
  'ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8' \
  3 \
  'ECMAScript whitespace table declaration and scan consumers'
require_fixed_string_count \
  "$wasm_builtins_mod" \
  'ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8' \
  0 \
  'obsolete broad builtin whitespace authority'
for obsolete_builtin_emitter in \
  emit_date_time_within_day \
  emit_throw_if_shared_array_buffer \
  emit_string_match_all_global_ascii_word_iterator_from_string_locals
do
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "\\b${obsolete_builtin_emitter}\\b" \
    0 \
    "obsolete builtin emitter $obsolete_builtin_emitter"
done
for obsolete_core_backend_api in \
  static_number_expr_value \
  buffer_memarg32 \
  buffer_memarg16 \
  emit_store_realm_type_error_prototype \
  standard_builtin_prototype_global_index
do
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "\\b${obsolete_core_backend_api}\\b" \
    0 \
    "obsolete core backend API $obsolete_core_backend_api"
done
wasm_planning="crates/lila-aot-wasm/src/planning.rs"
for obsolete_planning_api in \
  is_large_deferred_standard_builtin \
  script_uses_env \
  script_uses_calls \
  script_uses_function_heap \
  script_uses_function_table \
  block_uses_function_table \
  block_uses_calls \
  statement_uses_calls \
  for_init_uses_calls \
  statement_uses_function_table \
  for_init_uses_function_table \
  expr_uses_function_table \
  expr_uses_calls
do
  require_fixed_string_count \
    "$wasm_planning" \
    "$obsolete_planning_api" \
    0 \
    "obsolete planning API $obsolete_planning_api"
done
require_fixed_string_count \
  "$wasm_planning" \
  'super_constructor_target' \
  0 \
  'obsolete Wasm metadata super-constructor projection'
require_exact_line_count \
  "$wasm_planning" \
  '    pub(crate) fn iter(&self) -> impl Iterator<Item = (&FunctionId, &WasmFunctionMeta)> {' \
  0 \
  'obsolete function-meta registry iterator'

wasm_objects_property_read="$(sed -n '/    pub(crate) fn compile_property_read_from_locals(/,/    fn compile_dynamic_property_read_from_locals(/p' crates/lila-aot-wasm/src/objects.rs)"
require_text_regex_count \
  "$wasm_objects_property_read" \
  '^[[:space:]]*ValueKind::Dynamic => \{$' \
  1 \
  'dynamic property-read owner'
require_text_regex_count \
  "$wasm_objects_property_read" \
  '^[[:space:]]*ValueKind::String => match key \{$' \
  1 \
  'String property-read owner'
require_text_regex_count \
  "$wasm_objects_property_read" \
  '^[[:space:]]*ValueKind::String => \{$' \
  0 \
  'shadowed String property-read arms'
require_exact_line_count \
  "$wasm_lib" \
  'pub(crate) use functions::RealmRecordLocal;' \
  0 \
  'obsolete RealmRecordLocal crate-root re-export'
require_exact_line_count \
  crates/lila-aot-wasm/src/operations.rs \
  'use lila_ir::StaticRegExpCompilation;' \
  1 \
  'owner-local StaticRegExpCompilation import'

ir_lowering="crates/lila-ir/src/lowering.rs"
for obsolete_lowering_specialization in \
  target_has_private_brand \
  lower_generated_iterator_function_expression \
  lower_generated_iterator_function \
  lower_this_range_generator_function_body \
  single_lexical_number_binding \
  single_lexical_expression_binding \
  expression_is_this_unsigned_right_shift_zero \
  while_body_yields_and_increments \
  alloc_generated_iterator_values_name \
  lower_generator_body_as_array_iterator \
  lower_yield_star_generator_iife \
  delegate_method_returns_non_object \
  static_generator_declaration_elements \
  static_generator_statement_list_elements \
  static_generator_yield_string_element \
  static_generator_for_loop_string_elements \
  static_generator_string_for_loop_initializer \
  static_generator_string_for_loop_body \
  static_string_from_char_code_yield_name \
  static_generator_yield_identifier_name \
  static_string_from_char_code_arg_is_named \
  static_string_from_char_code_arg_name \
  static_negated_string_match_regex \
  static_string_from_char_code_value \
  static_generator_declaration_elements_by_name \
  merge_operand_shapes
do
  require_fixed_string_count \
    "$ir_lowering" \
    "$obsolete_lowering_specialization" \
    0 \
    "obsolete lowering specialization $obsolete_lowering_specialization"
done
require_fixed_string_count \
  crates/lila-ir/src/lowering_helpers.rs \
  'StaticStringGeneratorLoopBody' \
  0 \
  'obsolete String-generator loop domain'
generated_function_output="$(sed -n '/pub(crate) struct GeneratedFunctionOutput {/,/^}/p' "$ir_lowering")"
require_text_regex_count \
  "$generated_function_output" \
  '^[[:space:]]*pub\(crate\) [a-z_][a-z_]*:' \
  2 \
  'observed generated-function output fields'
require_text_regex_count \
  "$generated_function_output" \
  '^[[:space:]]*pub\(crate\) (function_id|this_info):' \
  0 \
  'unread generated-function output fields'
require_exact_line_count \
  crates/lila-ir/src/lib.rs \
  'use regress::Regex;' \
  0 \
  'obsolete broad Regex import'
require_fixed_string_count \
  crates/lila-ir/src/regexp.rs \
  'use regress::{' \
  1 \
  'direct regexp compiler import'

for obsolete_static_generator_cache_surface in \
  static_generator_sum_values \
  static_generator_element_values \
  prepare_static_generator_declarations \
  is_static_generator_declaration \
  static_generator_call_values \
  static_generator_call_elements_owned \
  static_generator_call_name \
  static_generator_call_is_known \
  array_iterator_from_static_generator_values \
  array_iterator_from_lowered_elements
do
  for static_generator_source in \
    "$ir_lowering" \
    crates/lila-ir/src/lowering/assignment.rs \
    crates/lila-ir/src/lowering/call_expression.rs \
    crates/lila-ir/src/lowering/for_of.rs
  do
    require_fixed_string_count \
      "$static_generator_source" \
      "$obsolete_static_generator_cache_surface" \
      0 \
      "obsolete static-generator cache surface $obsolete_static_generator_cache_surface"
  done
done
require_fixed_string_count \
  "$ir_lowering" \
  'fn static_generator_declaration_values_by_name(' \
  1 \
  'live static-generator declaration fold owner'
require_fixed_string_count \
  crates/lila-ir/src/lowering/call_expression.rs \
  'self.static_generator_call_overrides.get(&name)' \
  1 \
  'live generator-expression call override'
require_fixed_string_count \
  crates/lila-ir/src/lowering/assignment.rs \
  'self.static_object_iterator_literal_values(rhs)' \
  1 \
  'live assignment object-iterator fold'
require_fixed_string_count \
  crates/lila-ir/src/lowering/for_of.rs \
  'let element_info = if plain_async_await_body && iterable_is_array {' \
  0 \
  'obsolete resumable Array-walk element analysis boundary'
require_fixed_string_count \
  crates/lila-ir/src/lowering/for_of.rs \
  'let element_info = ValueInfo {' \
  1 \
  'generic synchronous iterator result analysis boundary'
require_fixed_string_count \
  crates/lila-ir/src/lowering/for_of.rs \
  'kind: ValueKind::Dynamic,' \
  1 \
  'generic synchronous iterator dynamic result kind'

test262_differential="crates/lila-test262/src/differential.rs"
require_fixed_string_count \
  "$test262_differential" \
  '#[cfg(any(test, feature = "spec-exec-oracle"))]' \
  32 \
  'test-or-oracle compile boundaries'
require_exact_line_count \
  crates/lila-test262/src/lib.rs \
  'fn skip_template_source(bytes: &[u8], mut idx: usize) -> usize {' \
  0 \
  'obsolete template-source scanner'
require_exact_line_count \
  crates/lila-test262/src/lib.rs \
  '    fn values_mut(&mut self) -> Option<&mut Vec<T>> {' \
  1 \
  'test-only wire-list mutation entry'
require_exact_line_count \
  "$test262_differential" \
  '    fn module_loader_context_sources(specifier: &str) -> Vec<(&'"'"'static str, String)> {' \
  1 \
  'feature-only module-loader fixture'

wasm_temporal_zoned_date_time_methods="crates/lila-aot-wasm/src/builtins/temporal_zoned_date_time_methods.rs"
for direction_domain in ZonedDateTimeArithmetic ZonedDateTimeDifference; do
  require_fixed_string_count \
    "$wasm_temporal_zoned_date_time_methods" \
    "enum $direction_domain" \
    1 \
    "private ZonedDateTime direction domain $direction_domain"
  if grep -Eq "^pub(\\([^)]*\\))?[[:space:]]+enum[[:space:]]+$direction_domain" "$wasm_temporal_zoned_date_time_methods"; then
    fail "$wasm_temporal_zoned_date_time_methods must keep $direction_domain private"
  fi
  if grep -Eq "$direction_domain" "$wasm_standard_builtins" "$wasm_builtins_mod"; then
    fail "$direction_domain must not escape its ZonedDateTime method owner"
  fi
done
for zoned_method in add subtract until since; do
  require_fixed_string_count \
    "$wasm_temporal_zoned_date_time_methods" \
    "pub(super) fn emit_temporal_zoned_date_time_${zoned_method}_builtin(" \
    1 \
    "fixed ZonedDateTime $zoned_method entry"
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "self.emit_temporal_zoned_date_time_${zoned_method}_builtin(function)?;" \
    1 \
    "fixed ZonedDateTime $zoned_method route"
done
require_fixed_string_count \
  "$wasm_temporal_zoned_date_time_methods" \
  'fn emit_temporal_zoned_date_time_add_or_subtract(' \
  1 \
  'private ZonedDateTime arithmetic emitter'
require_fixed_string_count \
  "$wasm_temporal_zoned_date_time_methods" \
  'fn emit_temporal_zoned_date_time_until_or_since(' \
  1 \
  'private ZonedDateTime difference emitter'
if grep -Eq 'emit_temporal_zoned_date_time_(add_or_subtract|until_or_since)\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed ZonedDateTime arithmetic and difference entries"
fi
check_raw_line_budget "$wasm_temporal_zoned_date_time_methods" 700

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

# T02's RegExp range-search owner. The parent matcher may request the semantic
# mismatch operation, but only the private child may select encoded range
# bounds or read their raw offsets.
wasm_regexp_builtins="crates/lila-aot-wasm/src/builtins/regexp.rs"
wasm_regexp_range_search="crates/lila-aot-wasm/src/builtins/regexp/range_search.rs"
require_file "$wasm_regexp_range_search"
check_no_inline_legacy_includes "$wasm_regexp_builtins"
check_no_inline_legacy_includes "$wasm_regexp_range_search"
require_exact_line_count \
  "$wasm_regexp_builtins" \
  'mod range_search;' \
  1 \
  'private RegExp range-search module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+range_search;' "$wasm_regexp_builtins"; then
  fail "$wasm_regexp_builtins must keep range_search private"
fi
if grep -Eq 'RegExpRangeBound|emit_regexp_range_bound_load\(|range_search::' "$wasm_regexp_builtins"; then
  fail "$wasm_regexp_builtins must not name, construct, project or import the private RegExp range-bound policy"
fi
require_fixed_string_count \
  "$wasm_regexp_range_search" \
  'RegExpRangeBound' \
  5 \
  'RegExp range-bound owner uses'
require_fixed_string_count \
  "$wasm_regexp_range_search" \
  'RegExpRangeBound::' \
  2 \
  'RegExp range-bound producer selections'
require_fixed_string_count \
  "$wasm_regexp_range_search" \
  'emit_regexp_range_bound_load(' \
  3 \
  'RegExp range-bound reader and consumers'
require_regex_count \
  "$wasm_regexp_range_search" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_regexp_unicode_property_mismatch[[:space:]]*\(' \
  1 \
  'RegExp semantic range-search surface'
require_fixed_string_count \
  "$wasm_regexp_builtins" \
  'self.emit_regexp_unicode_property_mismatch(' \
  2 \
  'unchanged forward and reverse RegExp range-search calls'
# Measured immediately after extraction: 3,661 parent lines and 120 child
# lines. The narrow margins are for maintenance of each owner.
check_raw_line_budget "$wasm_regexp_builtins" 3710
check_raw_line_budget "$wasm_regexp_range_search" 145

# T02's RegExp substitution owner. The String parent may request the semantic
# GetSubstitution operation, but only the private child may recognize, encode
# or exhaust the six raw substitution kinds.
wasm_string_builtins="crates/lila-aot-wasm/src/builtins/string.rs"
wasm_regexp_substitution="crates/lila-aot-wasm/src/builtins/string/regexp_substitution.rs"
require_file "$wasm_regexp_substitution"
check_no_inline_legacy_includes "$wasm_string_builtins"
check_no_inline_legacy_includes "$wasm_regexp_substitution"
if ! grep -q '^enum StringSymbolHookOperation' "$wasm_string_builtins" \
  || ! grep -q '^        let symbol_key = match &operation {' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must privately dispatch through StringSymbolHookOperation"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+StringSymbolHookOperation' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must keep StringSymbolHookOperation private"
fi
if grep -Eq 'StringSymbolHookOperation|emit_string_symbol_hook_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed String symbol-hook entries"
fi
require_fixed_string_count "$wasm_string_builtins" 'fn emit_string_symbol_hook_builtin(' 1 'private String symbol-hook emitter'
require_fixed_string_count "$wasm_string_builtins" 'self.emit_string_symbol_hook_builtin(' 5 'fixed String symbol-hook entry calls'
for string_symbol_hook_entry in match match_all replace replace_all search; do
  require_fixed_string_count \
    "$wasm_string_builtins" \
    "pub(super) fn emit_string_${string_symbol_hook_entry}_builtin(" \
    1 \
    "fixed String symbol-hook entry ${string_symbol_hook_entry}"
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "self.emit_string_${string_symbol_hook_entry}_builtin(function)?;" \
    1 \
    "fixed String symbol-hook route ${string_symbol_hook_entry}"
done
if ! grep -q '^enum RegExpFlagGetter' "$wasm_string_builtins" \
  || ! grep -q '^        let flag = match &getter {' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must privately dispatch through RegExpFlagGetter"
fi
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+RegExpFlagGetter' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must keep RegExpFlagGetter private"
fi
if grep -Eq 'RegExpFlagGetter|emit_regexp_prototype_flag_getter_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed RegExp flag-getter entries"
fi
require_fixed_string_count "$wasm_string_builtins" 'fn emit_regexp_prototype_flag_getter_builtin(' 1 'private RegExp flag-getter emitter'
require_fixed_string_count "$wasm_string_builtins" 'self.emit_regexp_prototype_flag_getter_builtin(' 8 'fixed RegExp flag-getter entry calls'
for regexp_flag_getter_entry in has_indices global ignore_case multiline dot_all unicode unicode_sets sticky; do
  require_fixed_string_count \
    "$wasm_string_builtins" \
    "pub(super) fn emit_regexp_prototype_${regexp_flag_getter_entry}_getter_builtin(" \
    1 \
    "fixed RegExp flag-getter entry ${regexp_flag_getter_entry}"
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "self.emit_regexp_prototype_${regexp_flag_getter_entry}_getter_builtin(function)?;" \
    1 \
    "fixed RegExp flag-getter route ${regexp_flag_getter_entry}"
done
require_exact_line_count \
  "$wasm_string_builtins" \
  'mod regexp_substitution;' \
  1 \
  'private RegExp substitution module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+regexp_substitution;' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must keep regexp_substitution private"
fi
if grep -Eq 'RegExpSubstitutionKind|regexp_substitution::' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must not name, construct, project or import the private RegExp substitution policy"
fi
require_fixed_string_count \
  "$wasm_regexp_substitution" \
  'RegExpSubstitutionKind' \
  15 \
  'RegExp substitution-kind owner uses'
require_fixed_string_count \
  "$wasm_regexp_substitution" \
  'runtime_code()' \
  4 \
  'RegExp substitution runtime-code projections'
require_regex_count \
  "$wasm_regexp_substitution" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_regexp_get_substitution[[:space:]]*\(' \
  1 \
  'RegExp semantic GetSubstitution surface'
require_fixed_string_count \
  "$wasm_string_builtins" \
  'self.emit_regexp_get_substitution(' \
  1 \
  'unchanged RegExp replacement consumer call'
# Measured immediately after extraction: 20,970 parent lines and 483 child
# lines. The narrow margin is for maintenance of the child owner.
check_raw_line_budget "$wasm_regexp_substitution" 520

# T02's duplicate-named-group pattern owner. The String parent may request
# either complete semantic matcher, but only the private child may name the raw
# pattern or call the pattern-parameterized emitter.
wasm_duplicate_named_group_pattern="crates/lila-aot-wasm/src/builtins/string/duplicate_named_group_pattern.rs"
require_file "$wasm_duplicate_named_group_pattern"
check_no_inline_legacy_includes "$wasm_duplicate_named_group_pattern"
require_exact_line_count \
  "$wasm_string_builtins" \
  'mod duplicate_named_group_pattern;' \
  1 \
  'private duplicate-named-group pattern module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+duplicate_named_group_pattern;' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must keep duplicate_named_group_pattern private"
fi
if grep -Eq 'DuplicateNamedGroupPattern|emit_string_match_duplicate_named_groups_from_string_locals\(|duplicate_named_group_pattern::' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must not name, construct, project or import the private duplicate-named-group policy"
fi
require_fixed_string_count \
  "$wasm_duplicate_named_group_pattern" \
  'DuplicateNamedGroupPattern' \
  6 \
  'duplicate-named-group pattern owner uses'
require_fixed_string_count \
  "$wasm_duplicate_named_group_pattern" \
  'emit_string_match_duplicate_named_groups_from_string_locals(' \
  3 \
  'private duplicate-named-group emitter definition and calls'
require_regex_count \
  "$wasm_duplicate_named_group_pattern" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_string_match_duplicate_named_group_alternative_captures[[:space:]]*\(' \
  1 \
  'alternative-captures semantic matcher surface'
require_regex_count \
  "$wasm_duplicate_named_group_pattern" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_string_match_duplicate_named_group_iterated_backreference[[:space:]]*\(' \
  1 \
  'iterated-backreference semantic matcher surface'
require_fixed_string_count \
  "$wasm_string_builtins" \
  'self.emit_string_match_duplicate_named_group_alternative_captures(' \
  1 \
  'alternative-captures semantic matcher call'
require_fixed_string_count \
  "$wasm_string_builtins" \
  'self.emit_string_match_duplicate_named_group_iterated_backreference(' \
  1 \
  'iterated-backreference semantic matcher call'
# Measured immediately after extraction: 20,883 parent lines and 125 child
# lines. The narrow margin is for maintenance of the child owner.
check_raw_line_budget "$wasm_duplicate_named_group_pattern" 150

# T02's global ASCII class quantifier owner. The String parent may request one
# of the three complete semantic matchers, but only the private child may name
# the raw width/polarity policy or call the parameterized emitter.
wasm_global_ascii_class_quantifier="crates/lila-aot-wasm/src/builtins/string/global_ascii_class_quantifier.rs"
require_file "$wasm_global_ascii_class_quantifier"
check_no_inline_legacy_includes "$wasm_global_ascii_class_quantifier"
require_exact_line_count \
  "$wasm_string_builtins" \
  'mod global_ascii_class_quantifier;' \
  1 \
  'private global ASCII class quantifier module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+global_ascii_class_quantifier;' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must keep global_ascii_class_quantifier private"
fi
if grep -Eq 'GlobalAsciiClassQuantifier|emit_string_match_global_ascii_class_quantifier_from_string_locals\(|global_ascii_class_quantifier::' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must not name, construct, project or import the private global ASCII class quantifier policy"
fi
require_fixed_string_count \
  "$wasm_global_ascii_class_quantifier" \
  'GlobalAsciiClassQuantifier' \
  9 \
  'global ASCII class quantifier owner lines'
require_fixed_string_count \
  "$wasm_global_ascii_class_quantifier" \
  'emit_string_match_global_ascii_class_quantifier_from_string_locals(' \
  4 \
  'private global ASCII class quantifier emitter definition and calls'
for semantic_matcher in \
  emit_string_match_global_ascii_digit_once_from_string_locals \
  emit_string_match_global_ascii_digit_twice_from_string_locals \
  emit_string_match_global_ascii_non_digit_twice_from_string_locals
do
  require_regex_count \
    "$wasm_global_ascii_class_quantifier" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+$semantic_matcher[[:space:]]*\\(" \
    1 \
    "global ASCII class semantic matcher surface $semantic_matcher"
  require_fixed_string_count \
    "$wasm_string_builtins" \
    "self.$semantic_matcher(" \
    1 \
    "global ASCII class semantic matcher call $semantic_matcher"
done
# Measured immediately after extraction: 20,671 parent lines and 261 child
# lines. The narrow margin is for maintenance of the child owner.
check_raw_line_budget "$wasm_global_ascii_class_quantifier" 300

# T02's postal-code match-result-shape owner. The String parent may request
# either complete semantic matcher, but only the private child may name the raw
# result shape or call the shape-parameterized emitter.
wasm_postal_code_match_result_shape="crates/lila-aot-wasm/src/builtins/string/postal_code_match_result_shape.rs"
require_file "$wasm_postal_code_match_result_shape"
check_no_inline_legacy_includes "$wasm_postal_code_match_result_shape"
require_exact_line_count \
  "$wasm_string_builtins" \
  'mod postal_code_match_result_shape;' \
  1 \
  'private postal-code match-result-shape module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+postal_code_match_result_shape;' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must keep postal_code_match_result_shape private"
fi
if grep -Eq 'PostalCodeMatchResultShape|emit_string_match_postal_code_from_string_locals\(|postal_code_match_result_shape::' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must not name, construct, project or import the private postal-code match-result-shape policy"
fi
require_fixed_string_count \
  "$wasm_postal_code_match_result_shape" \
  'PostalCodeMatchResultShape' \
  8 \
  'postal-code match-result-shape owner uses'
require_fixed_string_count \
  "$wasm_postal_code_match_result_shape" \
  'GlobalMatchArray' \
  4 \
  'postal-code global result-shape uses'
require_fixed_string_count \
  "$wasm_postal_code_match_result_shape" \
  'ExecMatchArray' \
  4 \
  'postal-code exec result-shape uses'
require_fixed_string_count \
  "$wasm_postal_code_match_result_shape" \
  'emit_string_match_postal_code_from_string_locals(' \
  3 \
  'private postal-code emitter definition and calls'
for semantic_matcher in \
  emit_string_match_postal_code_global_from_string_locals \
  emit_string_match_postal_code_exec_from_string_locals
do
  require_regex_count \
    "$wasm_postal_code_match_result_shape" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+$semantic_matcher[[:space:]]*\\(" \
    1 \
    "postal-code semantic matcher surface $semantic_matcher"
  require_fixed_string_count \
    "$wasm_string_builtins" \
    "self.$semantic_matcher(" \
    1 \
    "postal-code semantic matcher call $semantic_matcher"
done
# Measured immediately after extraction: 20,307 parent lines and 398 child
# lines. The narrow margin is for maintenance of the child owner.
check_raw_line_budget "$wasm_postal_code_match_result_shape" 440

# T02's literal-replacement scope owner. The String parent may request either
# complete semantic replacement loop, but only the private child may name the
# raw scope or call the scope-parameterized emitter.
wasm_string_literal_replacement_scope="crates/lila-aot-wasm/src/builtins/string/string_literal_replacement_scope.rs"
require_file "$wasm_string_literal_replacement_scope"
check_no_inline_legacy_includes "$wasm_string_literal_replacement_scope"
require_exact_line_count \
  "$wasm_string_builtins" \
  'mod string_literal_replacement_scope;' \
  1 \
  'private String literal-replacement scope module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+string_literal_replacement_scope;' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must keep string_literal_replacement_scope private"
fi
if grep -Eq 'StringLiteralReplacementScope|emit_string_replace_literal_from_string_locals\(|string_literal_replacement_scope::' "$wasm_string_builtins"; then
  fail "$wasm_string_builtins must not name, construct, project or import the private literal-replacement scope"
fi
require_fixed_string_count \
  "$wasm_string_literal_replacement_scope" \
  'StringLiteralReplacementScope' \
  6 \
  'String literal-replacement scope owner uses'
require_fixed_string_count \
  "$wasm_string_literal_replacement_scope" \
  'FirstOccurrence' \
  3 \
  'first-occurrence replacement scope uses'
require_fixed_string_count \
  "$wasm_string_literal_replacement_scope" \
  'AllOccurrences' \
  3 \
  'all-occurrences replacement scope uses'
require_fixed_string_count \
  "$wasm_string_literal_replacement_scope" \
  'emit_string_replace_literal_from_string_locals(' \
  3 \
  'private literal-replacement emitter definition and calls'
for semantic_replacement in \
  emit_string_replace_literal_first_occurrence_from_string_locals \
  emit_string_replace_literal_all_occurrences_from_string_locals
do
  require_regex_count \
    "$wasm_string_literal_replacement_scope" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+$semantic_replacement[[:space:]]*\\(" \
    1 \
    "String literal-replacement semantic surface $semantic_replacement"
  require_fixed_string_count \
    "$wasm_string_builtins" \
    "self.$semantic_replacement(" \
    1 \
    "String literal-replacement semantic call $semantic_replacement"
done
# Measured immediately after extraction: 19,860 parent lines and 489 child
# lines. The narrow margins are for maintenance of each owner.
# Measured after closing the String symbol-hook and RegExp flag-getter entries:
# 19,951 raw lines. The narrow margin is for this family, not adjacent work.
check_raw_line_budget "$wasm_string_builtins" 19980
check_raw_line_budget "$wasm_string_literal_replacement_scope" 530

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
require_fixed_string_count "$math_extremum_file" 'fn identity(&self) -> f64 {' 1 'Math extremum identity projection'
require_fixed_string_count "$math_extremum_file" 'fn emit_combine(&self,' 1 'Math extremum reduction projection'
require_fixed_string_count "$math_extremum_file" 'emit_math_extremum_builtin(' 3 'Math extremum definition/min/max consumers'

math_extremum_body="$(sed -n \
  '/^    fn emit_math_extremum_builtin(/,/^    pub(super) fn emit_math_abs_builtin(/p' \
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

# T05's passive Temporal.PlainDateTime layout has one private typed owner.
temporal_plain_date_time_layout_parent="crates/lila-aot-wasm/src/lib.rs"
temporal_plain_date_time_layout_heap="crates/lila-aot-wasm/src/heap.rs"
temporal_plain_date_time_layout_file="crates/lila-aot-wasm/src/heap_temporal_plain_date_time_layout.rs"
require_file "$temporal_plain_date_time_layout_file"
check_no_inline_legacy_includes "$temporal_plain_date_time_layout_file"
require_exact_line_count \
  "$temporal_plain_date_time_layout_parent" \
  'mod heap_temporal_plain_date_time_layout;' \
  1 \
  'private Temporal.PlainDateTime layout module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+heap_temporal_plain_date_time_layout;' "$temporal_plain_date_time_layout_parent"; then
  fail "$temporal_plain_date_time_layout_parent must keep heap_temporal_plain_date_time_layout private"
fi
require_fixed_string_count \
  "$temporal_plain_date_time_layout_file" \
  'pub(crate) enum TemporalPlainDateTimeHeapSlot {' \
  1 \
  'Temporal.PlainDateTime heap-slot identity domain'
require_fixed_string_count \
  "$temporal_plain_date_time_layout_file" \
  'record: "temporal-plain-date-time-record"' \
  10 \
  'Temporal.PlainDateTime typed layout rows'
if grep -Fq 'record: "temporal-plain-date-time-record"' "$temporal_plain_date_time_layout_heap" \
  || grep -Fq 'HEAP_TEMPORAL_PLAIN_DATE_TIME_RECORD_LAYOUT: &[HeapLayoutSlot]' "$temporal_plain_date_time_layout_heap"; then
  fail "$temporal_plain_date_time_layout_heap must not regain free-form Temporal.PlainDateTime layout rows"
fi
check_raw_line_budget "$temporal_plain_date_time_layout_file" 185

# T05's passive Temporal.Duration layout has one private typed owner.
temporal_duration_layout_parent="crates/lila-aot-wasm/src/lib.rs"
temporal_duration_layout_heap="crates/lila-aot-wasm/src/heap.rs"
temporal_duration_layout_file="crates/lila-aot-wasm/src/heap_temporal_duration_layout.rs"
require_file "$temporal_duration_layout_file"
check_no_inline_legacy_includes "$temporal_duration_layout_file"
require_exact_line_count \
  "$temporal_duration_layout_parent" \
  'mod heap_temporal_duration_layout;' \
  1 \
  'private Temporal.Duration layout module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+heap_temporal_duration_layout;' "$temporal_duration_layout_parent"; then
  fail "$temporal_duration_layout_parent must keep heap_temporal_duration_layout private"
fi
require_fixed_string_count \
  "$temporal_duration_layout_file" \
  'pub(crate) enum TemporalDurationHeapSlot {' \
  1 \
  'Temporal.Duration heap-slot identity domain'
require_fixed_string_count \
  "$temporal_duration_layout_file" \
  'record: "temporal-duration-record"' \
  10 \
  'Temporal.Duration typed layout rows'
if grep -Fq 'record: "temporal-duration-record"' "$temporal_duration_layout_heap" \
  || grep -Fq 'HEAP_TEMPORAL_DURATION_RECORD_LAYOUT: &[HeapLayoutSlot]' "$temporal_duration_layout_heap"; then
  fail "$temporal_duration_layout_heap must not regain free-form Temporal.Duration layout rows"
fi
check_raw_line_budget "$temporal_duration_layout_file" 180

# T05/T23's passive Intl.DateTimeFormat layout has one private typed owner.
intl_date_time_format_layout_parent="crates/lila-aot-wasm/src/lib.rs"
intl_date_time_format_layout_heap="crates/lila-aot-wasm/src/heap.rs"
intl_date_time_format_layout_file="crates/lila-aot-wasm/src/heap_intl_date_time_format_layout.rs"
require_file "$intl_date_time_format_layout_file"
check_no_inline_legacy_includes "$intl_date_time_format_layout_file"
require_exact_line_count \
  "$intl_date_time_format_layout_parent" \
  'mod heap_intl_date_time_format_layout;' \
  1 \
  'private Intl.DateTimeFormat layout module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+heap_intl_date_time_format_layout;' "$intl_date_time_format_layout_parent"; then
  fail "$intl_date_time_format_layout_parent must keep heap_intl_date_time_format_layout private"
fi
require_fixed_string_count \
  "$intl_date_time_format_layout_file" \
  'pub(crate) enum IntlDateTimeFormatHeapSlot {' \
  1 \
  'Intl.DateTimeFormat heap-slot identity domain'
require_fixed_string_count \
  "$intl_date_time_format_layout_file" \
  'record: "intl-date-time-format-record"' \
  23 \
  'Intl.DateTimeFormat typed layout rows'
if grep -Fq 'record: "intl-date-time-format-record"' "$intl_date_time_format_layout_heap" \
  || grep -Fq 'HEAP_INTL_DATE_TIME_FORMAT_RECORD_LAYOUT: &[HeapLayoutSlot]' "$intl_date_time_format_layout_heap"; then
  fail "$intl_date_time_format_layout_heap must not regain free-form Intl.DateTimeFormat layout rows"
fi
check_raw_line_budget "$intl_date_time_format_layout_file" 290

# T16's named Array string-key selection is private to two raw consumers. The
# Object builtins may call only four fixed count/write operations.
array_named_key_owner="crates/lila-aot-wasm/src/builtins/array.rs"
array_named_key_caller="crates/lila-aot-wasm/src/builtins/object.rs"
require_fixed_string_count "$array_named_key_owner" 'enum ArrayNamedStringKeySelection {' 1 'private Array named-key policy'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+ArrayNamedStringKeySelection' "$array_named_key_owner"; then
  fail "$array_named_key_owner must keep ArrayNamedStringKeySelection private"
fi
if grep -Eq 'ArrayNamedStringKeySelection|emit_array_named_string_props_(count|write_keys)\(' "$array_named_key_caller"; then
  fail "$array_named_key_caller must use fixed Array named-key operations"
fi
require_fixed_string_count "$array_named_key_owner" 'fn emit_array_named_string_props_count(' 1 'private Array named-key count consumer'
require_fixed_string_count "$array_named_key_owner" 'fn emit_array_named_string_props_write_keys(' 1 'private Array named-key write consumer'
require_fixed_string_count "$array_named_key_owner" 'self.emit_array_named_string_props_count(' 2 'fixed Array named-key count wrappers'
require_fixed_string_count "$array_named_key_owner" 'self.emit_array_named_string_props_write_keys(' 2 'fixed Array named-key write wrappers'
for array_named_key_wrapper in \
  emit_array_all_named_string_props_count \
  emit_array_enumerable_named_string_props_count \
  emit_array_all_named_string_props_write_keys \
  emit_array_enumerable_named_string_props_write_keys
do
  require_fixed_string_count "$array_named_key_owner" "pub(super) fn ${array_named_key_wrapper}(" 1 "fixed Array named-key wrapper $array_named_key_wrapper"
  require_fixed_string_count "$array_named_key_caller" "self.${array_named_key_wrapper}(" 1 "Object call to fixed Array named-key wrapper $array_named_key_wrapper"
done

# T16's raw sort output policy is private to fixed sort and toSorted entries.
array_sort_owner="crates/lila-aot-wasm/src/builtins/array.rs"
array_sort_caller="crates/lila-aot-wasm/src/builtins/standard.rs"
require_fixed_string_count "$array_sort_owner" 'enum ArraySortOutput {' 1 'private Array sort output policy'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+ArraySortOutput' "$array_sort_owner"; then
  fail "$array_sort_owner must keep ArraySortOutput private"
fi
if grep -Eq 'ArraySortOutput|compile_array_sort_with_output\(' "$array_sort_caller"; then
  fail "$array_sort_caller must use fixed Array sort entries"
fi
require_fixed_string_count "$array_sort_owner" 'fn compile_array_sort_with_output(' 1 'private shared Array sort compiler'
require_fixed_string_count "$array_sort_owner" 'self.compile_array_sort_with_output(' 2 'fixed Array sort entry calls'
for array_sort_wrapper in \
  compile_array_prototype_sort_builtin \
  compile_array_prototype_to_sorted_builtin
do
  require_fixed_string_count "$array_sort_owner" "pub(super) fn ${array_sort_wrapper}(" 1 "fixed Array sort entry $array_sort_wrapper"
  require_fixed_string_count "$array_sort_caller" "self.${array_sort_wrapper}(function)?" 1 "standard call to fixed Array sort entry $array_sort_wrapper"
done

# T16's find-family kind and raw compilers are private to eight fixed entries.
array_find_parent="crates/lila-aot-wasm/src/builtins/array.rs"
array_find_owner="crates/lila-aot-wasm/src/builtins/array/find_via_predicate.rs"
array_find_caller="crates/lila-aot-wasm/src/builtins/standard.rs"
require_fixed_string_count "$array_find_owner" 'enum FindViaPredicateKind {' 1 'private find-family kind'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+FindViaPredicateKind' "$array_find_owner"; then
  fail "$array_find_owner must keep FindViaPredicateKind private"
fi
if grep -Eq 'FindViaPredicateKind|compile_(array|typed_array)_find_with_kind\(' "$array_find_parent" "$array_find_caller"; then
  fail 'Array parent and standard catalog must use fixed find-family entries'
fi
require_fixed_string_count "$array_find_owner" 'fn compile_array_find_with_kind(' 1 'private Array find compiler'
require_fixed_string_count "$array_find_owner" 'fn compile_typed_array_find_with_kind(' 1 'private TypedArray find compiler'
require_fixed_string_count "$array_find_owner" 'self.compile_array_find_with_kind(' 4 'fixed Array find entry calls'
require_fixed_string_count "$array_find_owner" 'self.compile_typed_array_find_with_kind(' 4 'fixed TypedArray find entry calls'
for array_find_family in array typed_array; do
  for array_find_method in find find_index find_last find_last_index; do
    array_find_wrapper="compile_${array_find_family}_prototype_${array_find_method}_builtin"
    require_fixed_string_count "$array_find_owner" "pub(in crate::builtins) fn ${array_find_wrapper}(" 1 "fixed find-family entry $array_find_wrapper"
    require_fixed_string_count "$array_find_caller" "self.${array_find_wrapper}(function)?" 1 "standard call to fixed find-family entry $array_find_wrapper"
  done
done

# T16's callback receiver policy is private to six fixed reducer/forEach
# entries. The reducer entries are audited with their direction below.
array_callback_owner="crates/lila-aot-wasm/src/builtins/array.rs"
array_callback_caller="crates/lila-aot-wasm/src/builtins/standard.rs"
require_fixed_string_count "$array_callback_owner" 'enum ArrayCallbackReceiverKind {' 1 'private Array callback receiver policy'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+ArrayCallbackReceiverKind' "$array_callback_owner"; then
  fail "$array_callback_owner must keep ArrayCallbackReceiverKind private"
fi
if grep -Eq 'ArrayCallbackReceiverKind|compile_array_like_for_each_builtin\(' "$array_callback_caller"; then
  fail "$array_callback_caller must use fixed Array callback entries"
fi
require_fixed_string_count "$array_callback_owner" 'fn compile_array_like_for_each_builtin(' 1 'private shared Array forEach compiler'
require_fixed_string_count "$array_callback_owner" 'self.compile_array_like_for_each_builtin(' 2 'fixed Array forEach entry calls'
for array_for_each_wrapper in \
  compile_array_prototype_for_each_builtin \
  compile_typed_array_prototype_for_each_builtin
do
  require_fixed_string_count "$array_callback_owner" "pub(super) fn ${array_for_each_wrapper}(" 1 "fixed Array forEach entry $array_for_each_wrapper"
  require_fixed_string_count "$array_callback_caller" "self.${array_for_each_wrapper}(function)?" 1 "standard call to fixed Array forEach entry $array_for_each_wrapper"
done

# T16's raw reducer direction is private to four fixed semantic entries.
array_reduce_owner="crates/lila-aot-wasm/src/builtins/array.rs"
array_reduce_caller="crates/lila-aot-wasm/src/builtins/standard.rs"
require_fixed_string_count "$array_reduce_owner" 'enum ArrayReduceDirection {' 1 'private Array reduce direction'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+ArrayReduceDirection' "$array_reduce_owner"; then
  fail "$array_reduce_owner must keep ArrayReduceDirection private"
fi
if grep -Eq 'ArrayReduceDirection|compile_array_like_reduce_builtin\(' "$array_reduce_caller"; then
  fail "$array_reduce_caller must use fixed Array reducer entries"
fi
require_fixed_string_count "$array_reduce_owner" 'fn compile_array_like_reduce_builtin(' 1 'private shared Array reducer'
require_fixed_string_count "$array_reduce_owner" 'self.compile_array_like_reduce_builtin(' 4 'fixed Array reducer entry calls'
for array_reduce_wrapper in \
  compile_array_reduce_builtin \
  compile_array_reduce_right_builtin \
  compile_typed_array_reduce_builtin \
  compile_typed_array_reduce_right_builtin
do
  require_fixed_string_count "$array_reduce_owner" "pub(super) fn ${array_reduce_wrapper}(" 1 "fixed Array reducer entry $array_reduce_wrapper"
  require_fixed_string_count "$array_reduce_caller" "self.${array_reduce_wrapper}(function)?" 1 "standard call to fixed Array reducer entry $array_reduce_wrapper"
done

# T16/T17's raw Array/TypedArray at policy is private to two fixed entries.
array_at_owner="crates/lila-aot-wasm/src/builtins/array.rs"
array_at_caller="crates/lila-aot-wasm/src/builtins/standard.rs"
require_fixed_string_count "$array_at_owner" 'enum ArrayAtReceiverPolicy {' 1 'private Array at receiver policy'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+ArrayAtReceiverPolicy' "$array_at_owner"; then
  fail "$array_at_owner must keep ArrayAtReceiverPolicy private"
fi
if grep -Eq 'ArrayAtReceiverPolicy|compile_array_like_at_builtin\(' "$array_at_caller"; then
  fail "$array_at_caller must use fixed Array at entries"
fi
require_fixed_string_count "$array_at_owner" 'fn compile_array_like_at_builtin(' 1 'private shared Array at compiler'
require_fixed_string_count "$array_at_owner" 'self.compile_array_like_at_builtin(' 2 'fixed Array at entry calls'
require_fixed_string_count "$array_at_owner" 'fn emit_array_at_from_locals(' 1 'private Array at policy consumer'
for array_at_wrapper in \
  compile_array_prototype_at_builtin \
  compile_typed_array_prototype_at_builtin
do
  require_fixed_string_count "$array_at_owner" "pub(super) fn ${array_at_wrapper}(" 1 "fixed Array at entry $array_at_wrapper"
  require_fixed_string_count "$array_at_caller" "self.${array_at_wrapper}(function)?" 1 "standard call to fixed Array at entry $array_at_wrapper"
done

# T10's complete Object.defineProperty descriptor family has one private owner.
# The standard dispatcher may call the fixed builtin entry, but neither it nor
# the parent may regain the raw descriptor carriers or Arguments-specialized
# implementation helpers.
object_define_property_parent="crates/lila-aot-wasm/src/builtins/object.rs"
object_define_property_file="crates/lila-aot-wasm/src/builtins/object/define_property.rs"
require_file "$object_define_property_file"
check_no_inline_legacy_includes "$object_define_property_file"
require_exact_line_count \
  "$object_define_property_parent" \
  'mod define_property;' \
  1 \
  'private Object.defineProperty module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+define_property;' "$object_define_property_parent"; then
  fail "$object_define_property_parent must keep define_property private"
fi
for object_define_property_raw_owner in \
  'enum ObjectDefinePropertyDescriptorLocals {' \
  'struct ArgumentsCalleeDescriptorLocals {' \
  'fn emit_arguments_define_index_descriptor(' \
  'fn emit_arguments_define_callee('
do
  require_fixed_string_count \
    "$object_define_property_file" \
    "$object_define_property_raw_owner" \
    1 \
    "Object.defineProperty owner marker $object_define_property_raw_owner"
  if grep -Fq "$object_define_property_raw_owner" "$object_define_property_parent" \
    || grep -Fq "$object_define_property_raw_owner" "$wasm_standard_builtins"; then
    fail "Object.defineProperty raw owner marker $object_define_property_raw_owner escaped its private child"
  fi
done
require_regex_count \
  "$object_define_property_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_define_property_builtin[[:space:]]*\(' \
  1 \
  'Object.defineProperty builtin entry visibility'
if grep -Fq 'compile_object_define_property_builtin(' "$object_define_property_parent"; then
  fail "$object_define_property_parent must not regain the extracted Object.defineProperty entry"
fi
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.compile_object_define_property_builtin(function)?' \
  1 \
  'standard dispatcher Object.defineProperty call'
check_raw_line_budget "$object_define_property_parent" 5840
check_raw_line_budget "$object_define_property_file" 2560

# T10's complete Object.getOwnPropertyDescriptor compiler has one private
# owner. The parent retains only its module declaration and the standard
# dispatcher retains one fixed builtin call.
object_get_own_descriptor_parent="crates/lila-aot-wasm/src/builtins/object.rs"
object_get_own_descriptor_file="crates/lila-aot-wasm/src/builtins/object/get_own_property_descriptor.rs"
require_file "$object_get_own_descriptor_file"
check_no_inline_legacy_includes "$object_get_own_descriptor_file"
require_exact_line_count \
  "$object_get_own_descriptor_parent" \
  'mod get_own_property_descriptor;' \
  1 \
  'private Object.getOwnPropertyDescriptor module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+get_own_property_descriptor;' "$object_get_own_descriptor_parent"; then
  fail "$object_get_own_descriptor_parent must keep get_own_property_descriptor private"
fi
if grep -Fq 'compile_object_get_own_property_descriptor_builtin(' "$object_get_own_descriptor_parent"; then
  fail "$object_get_own_descriptor_parent must not regain the extracted Object.getOwnPropertyDescriptor entry"
fi
require_regex_count \
  "$object_get_own_descriptor_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_get_own_property_descriptor_builtin[[:space:]]*\(' \
  1 \
  'Object.getOwnPropertyDescriptor builtin entry visibility'
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.compile_object_get_own_property_descriptor_builtin(function)?' \
  1 \
  'standard dispatcher Object.getOwnPropertyDescriptor call'
check_raw_line_budget "$object_get_own_descriptor_parent" 4400
check_raw_line_budget "$object_get_own_descriptor_file" 1480

# T10's complete Object.getOwnPropertyDescriptors compiler has one private
# owner. The parent retains only its module declaration and the standard
# dispatcher retains one fixed builtin call.
object_get_own_descriptors_parent="crates/lila-aot-wasm/src/builtins/object.rs"
object_get_own_descriptors_file="crates/lila-aot-wasm/src/builtins/object/get_own_property_descriptors.rs"
require_file "$object_get_own_descriptors_file"
check_no_inline_legacy_includes "$object_get_own_descriptors_file"
require_exact_line_count \
  "$object_get_own_descriptors_parent" \
  'mod get_own_property_descriptors;' \
  1 \
  'private Object.getOwnPropertyDescriptors module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+get_own_property_descriptors;' "$object_get_own_descriptors_parent"; then
  fail "$object_get_own_descriptors_parent must keep get_own_property_descriptors private"
fi
if grep -Fq 'compile_object_get_own_property_descriptors_builtin(' "$object_get_own_descriptors_parent"; then
  fail "$object_get_own_descriptors_parent must not regain the extracted Object.getOwnPropertyDescriptors entry"
fi
require_regex_count \
  "$object_get_own_descriptors_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_get_own_property_descriptors_builtin[[:space:]]*\(' \
  1 \
  'Object.getOwnPropertyDescriptors builtin entry visibility'
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.compile_object_get_own_property_descriptors_builtin(function)?' \
  1 \
  'standard dispatcher Object.getOwnPropertyDescriptors call'
check_raw_line_budget "$object_get_own_descriptors_parent" 4220
check_raw_line_budget "$object_get_own_descriptors_file" 220

# T10's complete Object.assign compiler has one private owner. The parent
# retains only its module declaration and standard dispatch retains one fixed
# builtin call.
object_assign_parent="crates/lila-aot-wasm/src/builtins/object.rs"
object_assign_file="crates/lila-aot-wasm/src/builtins/object/assign.rs"
require_file "$object_assign_file"
check_no_inline_legacy_includes "$object_assign_file"
require_exact_line_count \
  "$object_assign_parent" \
  'mod assign;' \
  1 \
  'private Object.assign module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+assign;' "$object_assign_parent"; then
  fail "$object_assign_parent must keep assign private"
fi
if grep -Fq 'compile_object_assign_builtin(' "$object_assign_parent"; then
  fail "$object_assign_parent must not regain the extracted Object.assign entry"
fi
require_regex_count \
  "$object_assign_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_assign_builtin[[:space:]]*\(' \
  1 \
  'Object.assign builtin entry visibility'
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.compile_object_assign_builtin(function)?' \
  1 \
  'standard dispatcher Object.assign call'
check_raw_line_budget "$object_assign_parent" 3950
check_raw_line_budget "$object_assign_file" 300

# T10's user-facing own-descriptor predicates. These three builtins have
# different input sources and observable coercion orders, but consume the same
# public [[GetOwnProperty]] protocol. Keep those decisions in one closed Rust
# domain and prevent the deleted Array/arguments/ordinary representation scans
# from returning in any wrapper.
own_descriptor_predicate_parent="crates/lila-aot-wasm/src/builtins/object.rs"
own_descriptor_predicate_file="crates/lila-aot-wasm/src/builtins/object/own_descriptor_predicate.rs"
require_file "$own_descriptor_predicate_file"
check_no_inline_legacy_includes "$own_descriptor_predicate_file"
require_exact_line_count \
  "$own_descriptor_predicate_parent" \
  'mod own_descriptor_predicate;' \
  1 \
  'private own-descriptor predicate module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+own_descriptor_predicate;' "$own_descriptor_predicate_parent"; then
  fail "$own_descriptor_predicate_parent must keep own_descriptor_predicate private"
fi
if grep -Eq 'OwnDescriptorPredicateBuiltin|compile_object_own_descriptor_predicate_builtin\(|own_descriptor_predicate::' "$own_descriptor_predicate_parent"; then
  fail "$own_descriptor_predicate_parent must not name, construct, call or import the private own-descriptor predicate policy"
fi
require_fixed_string_count \
  "$own_descriptor_predicate_file" \
  'enum OwnDescriptorPredicateBuiltin {' \
  1 \
  'closed own-descriptor predicate builtin domain'
require_fixed_string_count \
  "$own_descriptor_predicate_file" \
  'fn compile_object_own_descriptor_predicate_builtin(' \
  1 \
  'shared own-descriptor predicate compiler'
require_fixed_string_count \
  "$own_descriptor_predicate_file" \
  'compile_object_own_descriptor_predicate_builtin(' \
  4 \
  'own-descriptor predicate compiler definition and three wrapper calls'
require_fixed_string_count "$own_descriptor_predicate_file" 'OwnDescriptorPredicateBuiltin' 14 'own-descriptor predicate policy uses'
for own_predicate_variant in ObjectHasOwn PrototypeHasOwnProperty PrototypePropertyIsEnumerable
do
  require_fixed_string_count "$own_descriptor_predicate_file" "$own_predicate_variant" 5 "own-descriptor predicate variant $own_predicate_variant"
done
require_regex_count \
  "$own_descriptor_predicate_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_.*_builtin[[:space:]]*\(' \
  3 \
  'own-descriptor predicate semantic wrapper visibility'

own_descriptor_predicate_body="$(sed -n \
  '/^    fn compile_object_own_descriptor_predicate_builtin(/,/^    pub(in crate::builtins) fn compile_object_has_own_builtin(/p' \
  "$own_descriptor_predicate_file")"
if [ "$(grep -Fc 'match &builtin {' <<<"$own_descriptor_predicate_body" || true)" -ne 3 ]; then
  fail 'own-descriptor predicate compiler must exhaustively select source, order and projection'
fi
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' <<<"$own_descriptor_predicate_body"; then
  fail 'own-descriptor predicate compiler must not escape its closed domain with a catch-all'
fi
require_own_predicate_body_count() {
  needle="$1"
  expected="$2"
  description="$3"
  count="$(grep -Fc "$needle" <<<"$own_descriptor_predicate_body" || true)"
  if [ "$count" -ne "$expected" ]; then
    fail "own-descriptor predicate compiler must contain $expected $description sites (found $count)"
  fi
}
require_own_predicate_body_count \
  'StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id()' \
  1 \
  'canonical Object.getOwnPropertyDescriptor metadata lookup'
require_own_predicate_body_count 'self.emit_direct_js_call(' 1 'canonical descriptor call'
require_own_predicate_body_count \
  'self.emit_object_own_data_field_read(' \
  1 \
  'non-observable enumerable projection'
require_own_predicate_body_count \
  'self.emit_value_to_current_function_realm_object_locals(' \
  3 \
  'ToObject conversion'
require_own_predicate_body_count \
  'self.emit_value_to_property_key_locals(' \
  3 \
  'ToPropertyKey conversion'

for raw_own_predicate_scan in \
  'HEAP_' \
  'PROXY_HANDLER_' \
  'emit_object_own_property_present' \
  'emit_known_array_index_from_property_key' \
  'emit_array_' \
  'emit_arguments_' \
  'load_i64_to_local_from_offset'
do
  if grep -Fq "$raw_own_predicate_scan" <<<"$own_descriptor_predicate_body"; then
    fail "own-descriptor predicate compiler must not rebuild representation storage through $raw_own_predicate_scan"
  fi
done

own_descriptor_order_body="$(awk '
  /match &builtin \{/ { matches += 1 }
  matches == 2 { print }
  matches == 3 { exit }
' <<<"$own_descriptor_predicate_body")"
if ! awk '
  /OwnDescriptorPredicateBuiltin::ObjectHasOwn =>/ && !object_arm { object_arm = NR }
  /compile_nullish_tagged_i32/ && object_arm && !object_nullish { object_nullish = NR }
  /emit_value_to_current_function_realm_object_locals/ && object_arm && !object_to_object { object_to_object = NR }
  /emit_value_to_property_key_locals/ && object_arm && !object_to_key { object_to_key = NR }
  /OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty =>/ { hop_arm = NR }
  /emit_value_to_property_key_locals/ && hop_arm && !hop_to_key { hop_to_key = NR }
  /compile_nullish_tagged_i32/ && hop_arm && !hop_nullish { hop_nullish = NR }
  /emit_value_to_current_function_realm_object_locals/ && hop_arm && !hop_to_object { hop_to_object = NR }
  /OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable =>/ { pie_arm = NR }
  /emit_value_to_property_key_locals/ && pie_arm && !pie_to_key { pie_to_key = NR }
  /compile_nullish_tagged_i32/ && pie_arm && !pie_nullish { pie_nullish = NR }
  /emit_value_to_current_function_realm_object_locals/ && pie_arm && !pie_to_object { pie_to_object = NR }
  END {
    exit !(object_arm && object_nullish && object_to_object && object_to_key &&
      hop_arm && hop_to_key && hop_nullish && hop_to_object &&
      pie_arm && pie_to_key && pie_nullish && pie_to_object &&
      object_nullish < object_to_object && object_to_object < object_to_key &&
      hop_to_key < hop_nullish && hop_nullish < hop_to_object &&
      pie_to_key < pie_nullish && pie_nullish < pie_to_object)
  }
' <<<"$own_descriptor_order_body"; then
  fail 'own-descriptor predicates must preserve Object.hasOwn object-first and prototype key-first conversion order'
fi

for own_predicate_wrapper_spec in \
  'compile_object_has_own_builtin|compile_object_prototype_has_own_property_builtin|OwnDescriptorPredicateBuiltin::ObjectHasOwn' \
  'compile_object_prototype_has_own_property_builtin|compile_object_prototype_property_is_enumerable_builtin|OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty' \
  'compile_object_prototype_property_is_enumerable_builtin|END|OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable'
do
  wrapper="${own_predicate_wrapper_spec%%|*}"
  rest="${own_predicate_wrapper_spec#*|}"
  next_wrapper="${rest%%|*}"
  variant="${rest#*|}"
  if [ "$next_wrapper" = END ]; then
    wrapper_body="$(sed -n \
      "/^    pub(in crate::builtins) fn ${wrapper}(/,/^}/p" \
      "$own_descriptor_predicate_file")"
  else
    wrapper_body="$(sed -n \
      "/^    pub(in crate::builtins) fn ${wrapper}(/,/^    pub(in crate::builtins) fn ${next_wrapper}(/p" \
      "$own_descriptor_predicate_file")"
  fi
  if [ "$(grep -Fc 'self.compile_object_own_descriptor_predicate_builtin(' <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc "$variant" <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc 'self.' <<<"$wrapper_body" || true)" -ne 1 ]; then
    fail "$wrapper must be a one-call selection of $variant"
  fi
  if grep -Eq 'Instruction::|HEAP_|emit_|reserve_temp_local|StandardBuiltinId::' <<<"$wrapper_body"; then
    fail "$wrapper must not contain a representation-specific descriptor path"
  fi
done

# Measured after the later EnumerableOwnProperties extraction: 8,248 parent lines. The
# own-descriptor child remains 230 lines. The narrow margins are for maintenance
# of each owner.
check_raw_line_budget "$own_descriptor_predicate_parent" 8330
check_raw_line_budget "$own_descriptor_predicate_file" 260

require_fixed_string_count \
  crates/lila-cli/tests/cli/object.rs \
  'fn run_wasm_backend_succeeds_for_object_own_descriptor_predicates()' \
  1 \
  'exact own-descriptor-predicate CLI regression'
require_fixed_string_count \
  crates/lila-cli/tests/cli/object.rs \
  '"wasm_object_own_descriptor_predicates.js"' \
  1 \
  'own-descriptor-predicate fixture wiring'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_object_own_descriptor_predicates.js ]; then
  fail 'own-descriptor-predicate fixture must remain present'
fi
if [ ! -f docs/rust-rewrite/contracts/own-descriptor-predicates.md ]; then
  fail 'own-descriptor-predicate contract must remain present'
fi

# T10's Object entries/values policy has one private owner. The parent and
# standard dispatcher may invoke fixed semantic operations, but cannot construct
# or project the raw policy controlling diagnostics and result shape.
enumerable_own_properties_parent="crates/lila-aot-wasm/src/builtins/object.rs"
enumerable_own_properties_file="crates/lila-aot-wasm/src/builtins/object/enumerable_own_properties.rs"
require_file "$enumerable_own_properties_file"
check_no_inline_legacy_includes "$enumerable_own_properties_file"
require_exact_line_count \
  "$enumerable_own_properties_parent" \
  'mod enumerable_own_properties;' \
  1 \
  'private enumerable-own-properties module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+enumerable_own_properties;' "$enumerable_own_properties_parent"; then
  fail "$enumerable_own_properties_parent must keep enumerable_own_properties private"
fi
for enumerable_own_properties_non_owner in "$enumerable_own_properties_parent" "$wasm_standard_builtins"
do
  if grep -Eq 'EnumerableOwnProperties|compile_object_enumerable_own_properties_builtin\(|enumerable_own_properties::' "$enumerable_own_properties_non_owner"; then
    fail "$enumerable_own_properties_non_owner must not name, construct, call or import the private enumerable-own-properties policy"
  fi
done
require_fixed_string_count "$enumerable_own_properties_file" 'enum EnumerableOwnProperties {' 1 'closed enumerable-own-properties domain'
require_fixed_string_count "$enumerable_own_properties_file" 'EnumerableOwnProperties' 8 'enumerable-own-properties policy uses'
require_fixed_string_count "$enumerable_own_properties_file" 'EnumerableOwnProperties::Entries' 3 'entries policy uses'
require_fixed_string_count "$enumerable_own_properties_file" 'EnumerableOwnProperties::Values' 3 'values policy uses'
require_fixed_string_count \
  "$enumerable_own_properties_file" \
  'compile_object_enumerable_own_properties_builtin(' \
  3 \
  'private enumerable-own-properties compiler definition and wrapper calls'
require_regex_count \
  "$enumerable_own_properties_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_(entries|values)_builtin[[:space:]]*\(' \
  2 \
  'enumerable-own-properties semantic wrapper visibility'
enumerable_own_properties_body="$(sed -n \
  '/^    fn compile_object_enumerable_own_properties_builtin(/,/^    pub(in crate::builtins) fn compile_object_entries_builtin(/p' \
  "$enumerable_own_properties_file")"
if [ "$(grep -Fc 'match &mode {' <<<"$enumerable_own_properties_body" || true)" -ne 2 ] \
  || grep -Eq 'match mode|mode[[:space:]]*[!=]=|^[[:space:]]*_ =>|unreachable!\(' <<<"$enumerable_own_properties_body"; then
  fail 'enumerable-own-properties compiler must borrow and exhaustively project both policy decisions'
fi
for enumerable_own_properties_capability in Clone Copy Debug PartialEq Eq PartialOrd Ord Hash Default
do
  if grep -Eq "#\[derive\([^]]*${enumerable_own_properties_capability}|impl[[:space:]]+${enumerable_own_properties_capability}[[:space:]]+for[[:space:]]+EnumerableOwnProperties" "$enumerable_own_properties_file"; then
    fail "EnumerableOwnProperties must not gain ${enumerable_own_properties_capability} capability"
  fi
done
for enumerable_own_properties_wrapper_spec in \
  'compile_object_entries_builtin|compile_object_values_builtin|EnumerableOwnProperties::Entries' \
  'compile_object_values_builtin|END|EnumerableOwnProperties::Values'
do
  wrapper="${enumerable_own_properties_wrapper_spec%%|*}"
  rest="${enumerable_own_properties_wrapper_spec#*|}"
  next_wrapper="${rest%%|*}"
  variant="${rest#*|}"
  if [ "$next_wrapper" = END ]; then
    wrapper_body="$(sed -n "/^    pub(in crate::builtins) fn ${wrapper}(/,/^}/p" "$enumerable_own_properties_file")"
  else
    wrapper_body="$(sed -n "/^    pub(in crate::builtins) fn ${wrapper}(/,/^    pub(in crate::builtins) fn ${next_wrapper}(/p" "$enumerable_own_properties_file")"
  fi
  if [ "$(grep -Fc 'self.compile_object_enumerable_own_properties_builtin(' <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc "$variant" <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc 'self.' <<<"$wrapper_body" || true)" -ne 1 ]; then
    fail "$wrapper must be a one-call selection of $variant"
  fi
  if grep -Eq 'Instruction::|emit_|reserve_temp_local|StandardBuiltinId::' <<<"$wrapper_body"; then
    fail "$wrapper must not contain enumerable-own-properties implementation policy"
  fi
  require_fixed_string_count "$wasm_standard_builtins" "self.${wrapper}(" 1 "standard dispatcher call to $wrapper"
done
check_raw_line_budget "$enumerable_own_properties_file" 380

# T10's Object integrity-test policy has one private owner. The parent and
# standard dispatcher may invoke fixed isSealed/isFrozen operations, but cannot
# construct or project the raw policy that controls the writability branch.
integrity_test_parent="crates/lila-aot-wasm/src/builtins/object.rs"
integrity_test_file="crates/lila-aot-wasm/src/builtins/object/integrity_test.rs"
require_file "$integrity_test_file"
check_no_inline_legacy_includes "$integrity_test_file"
require_exact_line_count \
  "$integrity_test_parent" \
  'mod integrity_test;' \
  1 \
  'private integrity-test module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+integrity_test;' "$integrity_test_parent"; then
  fail "$integrity_test_parent must keep integrity_test private"
fi
for integrity_test_non_owner in "$integrity_test_parent" "$wasm_standard_builtins"
do
  if grep -Eq 'IntegrityTest|compile_object_integrity_test_builtin\(|integrity_test::' "$integrity_test_non_owner"; then
    fail "$integrity_test_non_owner must not name, construct, call or import the private integrity-test policy"
  fi
done
require_fixed_string_count "$integrity_test_file" 'enum IntegrityTest {' 1 'closed integrity-test domain'
require_fixed_string_count "$integrity_test_file" 'IntegrityTest' 6 'integrity-test policy uses'
require_fixed_string_count "$integrity_test_file" 'IntegrityTest::Sealed' 2 'sealed policy uses'
require_fixed_string_count "$integrity_test_file" 'IntegrityTest::Frozen' 2 'frozen policy uses'
require_fixed_string_count \
  "$integrity_test_file" \
  'compile_object_integrity_test_builtin(' \
  3 \
  'private integrity-test compiler definition and wrapper calls'
require_regex_count \
  "$integrity_test_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_is_(sealed|frozen)_builtin[[:space:]]*\(' \
  2 \
  'integrity-test semantic wrapper visibility'
integrity_test_body="$(sed -n \
  '/^    fn compile_object_integrity_test_builtin(/,/^    pub(in crate::builtins) fn compile_object_is_sealed_builtin(/p' \
  "$integrity_test_file")"
if [ "$(grep -Fc 'match &mode {' <<<"$integrity_test_body" || true)" -ne 1 ] \
  || grep -Eq 'match mode|mode[[:space:]]*[!=]=|^[[:space:]]*_ =>|unreachable!\(' <<<"$integrity_test_body"; then
  fail 'integrity-test compiler must borrow and exhaustively project its closed policy'
fi
for integrity_test_capability in Clone Copy Debug PartialEq Eq PartialOrd Ord Hash Default
do
  if grep -Eq "#\[derive\([^]]*${integrity_test_capability}|impl[[:space:]]+${integrity_test_capability}[[:space:]]+for[[:space:]]+IntegrityTest" "$integrity_test_file"; then
    fail "IntegrityTest must not gain ${integrity_test_capability} capability"
  fi
done
for integrity_test_wrapper_spec in \
  'compile_object_is_sealed_builtin|compile_object_is_frozen_builtin|IntegrityTest::Sealed' \
  'compile_object_is_frozen_builtin|END|IntegrityTest::Frozen'
do
  wrapper="${integrity_test_wrapper_spec%%|*}"
  rest="${integrity_test_wrapper_spec#*|}"
  next_wrapper="${rest%%|*}"
  variant="${rest#*|}"
  if [ "$next_wrapper" = END ]; then
    wrapper_body="$(sed -n "/^    pub(in crate::builtins) fn ${wrapper}(/,/^}/p" "$integrity_test_file")"
  else
    wrapper_body="$(sed -n "/^    pub(in crate::builtins) fn ${wrapper}(/,/^    pub(in crate::builtins) fn ${next_wrapper}(/p" "$integrity_test_file")"
  fi
  if [ "$(grep -Fc 'self.compile_object_integrity_test_builtin(' <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc "$variant" <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc 'self.' <<<"$wrapper_body" || true)" -ne 1 ]; then
    fail "$wrapper must be a one-call selection of $variant"
  fi
  if grep -Eq 'Instruction::|emit_|reserve_temp_local|StandardBuiltinId::' <<<"$wrapper_body"; then
    fail "$wrapper must not contain integrity-test implementation policy"
  fi
  require_fixed_string_count "$wasm_standard_builtins" "self.${wrapper}(" 1 "standard dispatcher call to $wrapper"
done
check_raw_line_budget "$integrity_test_file" 250

# T10's Annex-B prototype accessor lookup policy has one private owner. The
# parent and standard dispatcher may invoke the two fixed semantic operations,
# but cannot construct or project the raw getter/setter selection.
prototype_lookup_parent="crates/lila-aot-wasm/src/builtins/object.rs"
prototype_lookup_file="crates/lila-aot-wasm/src/builtins/object/prototype_lookup.rs"
require_file "$prototype_lookup_file"
check_no_inline_legacy_includes "$prototype_lookup_file"
require_exact_line_count \
  "$prototype_lookup_parent" \
  'mod prototype_lookup;' \
  1 \
  'private prototype-lookup module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+prototype_lookup;' "$prototype_lookup_parent"; then
  fail "$prototype_lookup_parent must keep prototype_lookup private"
fi
if grep -Eq 'PrototypeLookup|compile_object_prototype_lookup_builtin\(|prototype_lookup::' "$prototype_lookup_parent"; then
  fail "$prototype_lookup_parent must not name, construct, call or import the private prototype-lookup policy"
fi
if grep -Eq '(^|::)PrototypeLookup(::|[[:space:]]*(,|;|}|as[[:space:]]))|compile_object_prototype_lookup_builtin\(|prototype_lookup::' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must not construct, call or import the private prototype-lookup policy"
fi
require_fixed_string_count "$prototype_lookup_file" 'enum PrototypeLookup {' 1 'closed prototype-lookup domain'
require_fixed_string_count "$prototype_lookup_file" 'PrototypeLookup' 6 'prototype-lookup policy uses'
require_fixed_string_count "$prototype_lookup_file" 'PrototypeLookup::Getter' 2 'getter policy uses'
require_fixed_string_count "$prototype_lookup_file" 'PrototypeLookup::Setter' 2 'setter policy uses'
require_fixed_string_count \
  "$prototype_lookup_file" \
  'compile_object_prototype_lookup_builtin(' \
  3 \
  'private prototype-lookup compiler definition and wrapper calls'
require_regex_count \
  "$prototype_lookup_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_prototype_lookup_(getter|setter)_builtin[[:space:]]*\(' \
  2 \
  'prototype-lookup semantic wrapper visibility'
prototype_lookup_body="$(sed -n \
  '/^    fn compile_object_prototype_lookup_builtin(/,/^    pub(in crate::builtins) fn compile_object_prototype_lookup_getter_builtin(/p' \
  "$prototype_lookup_file")"
if [ "$(grep -Fc 'match &mode {' <<<"$prototype_lookup_body" || true)" -ne 1 ] \
  || grep -Eq 'match mode|mode[[:space:]]*[!=]=|^[[:space:]]*_ =>|unreachable!\(' <<<"$prototype_lookup_body"; then
  fail 'prototype-lookup compiler must borrow and exhaustively project its closed policy'
fi
for prototype_lookup_capability in Clone Copy Debug PartialEq Eq PartialOrd Ord Hash Default
do
  if grep -Eq "#\[derive\([^]]*${prototype_lookup_capability}|impl[[:space:]]+${prototype_lookup_capability}[[:space:]]+for[[:space:]]+PrototypeLookup" "$prototype_lookup_file"; then
    fail "PrototypeLookup must not gain ${prototype_lookup_capability} capability"
  fi
done
for prototype_lookup_wrapper_spec in \
  'compile_object_prototype_lookup_getter_builtin|compile_object_prototype_lookup_setter_builtin|PrototypeLookup::Getter' \
  'compile_object_prototype_lookup_setter_builtin|END|PrototypeLookup::Setter'
do
  wrapper="${prototype_lookup_wrapper_spec%%|*}"
  rest="${prototype_lookup_wrapper_spec#*|}"
  next_wrapper="${rest%%|*}"
  variant="${rest#*|}"
  if [ "$next_wrapper" = END ]; then
    wrapper_body="$(sed -n "/^    pub(in crate::builtins) fn ${wrapper}(/,/^}/p" "$prototype_lookup_file")"
  else
    wrapper_body="$(sed -n "/^    pub(in crate::builtins) fn ${wrapper}(/,/^    pub(in crate::builtins) fn ${next_wrapper}(/p" "$prototype_lookup_file")"
  fi
  if [ "$(grep -Fc 'self.compile_object_prototype_lookup_builtin(' <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc "$variant" <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc 'self.' <<<"$wrapper_body" || true)" -ne 1 ]; then
    fail "$wrapper must be a one-call selection of $variant"
  fi
  if grep -Eq 'Instruction::|emit_|reserve_temp_local|StandardBuiltinId::' <<<"$wrapper_body"; then
    fail "$wrapper must not contain prototype-lookup implementation policy"
  fi
  require_fixed_string_count "$wasm_standard_builtins" "self.${wrapper}(" 1 "standard dispatcher call to $wrapper"
done
check_raw_line_budget "$prototype_lookup_file" 180
require_fixed_string_count \
  crates/lila-cli/tests/cli/object.rs \
  'fn run_wasm_backend_preserves_object_builtin_policy_domains()' \
  1 \
  'exact Object policy-domain CLI regression'
require_fixed_string_count \
  crates/lila-cli/tests/cli/object.rs \
  'fn run_wasm_backend_succeeds_for_object_prototype_accessor_lookup_fixture()' \
  1 \
  'exact prototype-accessor lookup CLI regression'
if [ ! -f docs/rust-rewrite/contracts/object-builtin-policy-domains.md ]; then
  fail 'Object builtin policy-domain contract must remain present'
fi

# T10's complete [[HasProperty]] entry is crate-visible, while the
# representation dispatcher stays private to objects.rs. The branch order is
# declared once and consumed exhaustively both here and by the direct
# [[GetOwnProperty]] authority below.
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'pub(crate) fn emit_object_has_property_i32(' \
  1 \
  'crate-visible HasProperty entry'
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'pub(crate) fn emit_object_has_property_with_key_tag_i32(' \
  1 \
  'crate-visible tagged-key HasProperty entry'
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  '    fn emit_has_property_dispatch_with_key_tag_i32(' \
  1 \
  'private HasProperty representation dispatcher'
object_internal_method_macro="$(sed -n \
  '/^macro_rules! object_internal_method_branches {/,/^}$/p' \
  crates/lila-aot-wasm/src/objects.rs | tr -d '[:space:]')"
if [ "$(grep -Fo 'enumObjectInternalMethodBranch{$($branch),+}' <<<"$object_internal_method_macro" | wc -l || true)" -ne 1 ] \
  || [ "$(grep -Fo 'constORDER:&'"'"'static[Self]=&[$(Self::$branch),+];' <<<"$object_internal_method_macro" | wc -l || true)" -ne 1 ]; then
  fail 'object internal-method enum and order must remain generated from the same branch sequence'
fi
object_internal_method_order="$(sed -n \
  '/^object_internal_method_branches!(/,/^);/p' \
  crates/lila-aot-wasm/src/objects.rs | tr -d '[:space:]')"
if [ "$object_internal_method_order" != 'object_internal_method_branches!(Proxy,IntegerIndexed,Array,Arguments,BoxedString,Ordinary,);' ]; then
  fail 'object internal methods must retain the reviewed exotic-to-ordinary dispatch order'
fi
has_property_dispatch_body="$(sed -n \
  '/^    fn emit_has_property_dispatch_with_key_tag_i32(/,/^    pub(crate) fn emit_data_property_read_no_call(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
if [ "$(grep -Fc 'for branch in ObjectInternalMethodBranch::ORDER.iter().copied() {' <<<"$has_property_dispatch_body" || true)" -ne 1 ]; then
  fail 'HasProperty must consume the closed object-internal-method branch order once'
fi
for branch in Proxy IntegerIndexed Array Arguments BoxedString Ordinary; do
  if [ "$(grep -Fc "ObjectInternalMethodBranch::$branch => {" <<<"$has_property_dispatch_body" || true)" -ne 1 ]; then
    fail "HasProperty must exhaustively emit the $branch branch"
  fi
done
if grep -Eq '^[[:space:]]*_ =>|unreachable!|todo!|unimplemented!' <<<"$has_property_dispatch_body"; then
  fail 'HasProperty dispatch must not escape exhaustive representation handling'
fi
require_fixed_string_count \
  crates/lila-engine/src/lib.rs \
  'fn wasm_backend_has_property_dispatches_every_live_exotic_branch()' \
  1 \
  'complete HasProperty runtime regression'

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
require_fixed_string_count crates/lila-aot-wasm/src/builtins/object/get_own_property_descriptor.rs "$own_descriptor_fact" 2 'Object.getOwnPropertyDescriptor invariant call'
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
require_regex_count crates/lila-aot-wasm/src/objects.rs '^[[:space:]]*struct[[:space:]]+ProxySetDescriptorLocals[[:space:]]*\{' 1 'complete Proxy-Set descriptor projection'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'enum DescriptorAccessorProjectionLocals {' 1 'closed getter/setter endpoint projection'
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'fn emit_direct_own_descriptor(' \
  1 \
  'direct-own-descriptor representation authority'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_direct_own_descriptor(' 4 'direct-own-descriptor definition/fact/Proxy Get/Proxy Set call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_direct_own_descriptor_for_proxy_get(' 2 'typed Proxy-Get descriptor wrapper definition/call'
require_regex_count crates/lila-aot-wasm/src/objects.rs '^[[:space:]]*fn[[:space:]]+emit_direct_own_descriptor_for_proxy_set[[:space:]]*\(' 1 'typed Proxy-Set descriptor wrapper definition'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_proxy_get_invariant_check(' 2 'Proxy-Get invariant definition/object-read call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_proxy_set_invariant_check(' 2 'Proxy-Set invariant definition/object-write call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/reflect.rs 'emit_proxy_set_invariant_check(' 1 'Reflect.set typed Proxy-Set invariant call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'DirectOwnDescriptorProjectionLocals::ProxyGet(' 1 'complete Proxy-Get descriptor projection construction'
require_regex_count crates/lila-aot-wasm/src/objects.rs '^[[:space:]]*DirectOwnDescriptorProjectionLocals::ProxySet\(descriptor\),$' 1 'complete Proxy-Set descriptor projection construction'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'emit_normal_proxy_get_trap_result(' 2 'pending-to-normal Proxy-Get result transition definition/call'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'PendingProxyGetTrapResultLocals::new(' 1 'pending Proxy-Get result construction'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'NormalProxyGetTrapResultLocals(' 2 'normal Proxy-Get type declaration/guarded construction'
require_fixed_string_count crates/lila-cli/tests/cli/object.rs 'fn run_wasm_backend_succeeds_for_proxy_get_direct_descriptor_invariants()' 1 'exact Proxy-Get direct-descriptor CLI regression'
require_fixed_string_count crates/lila-cli/tests/cli/object.rs '"wasm_proxy_get_direct_descriptor_invariants.js"' 1 'Proxy-Get direct-descriptor fixture wiring'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_proxy_get_direct_descriptor_invariants.js ]; then
  fail 'Proxy [[Get]] direct-descriptor invariant fixture must remain present'
fi
proxy_set_cli="crates/lila-cli/tests/cli/object.rs"
require_active_wasm_cli_rust_test \
  "$proxy_set_cli" \
  run_wasm_backend_succeeds_for_proxy_set_direct_descriptor_invariants \
  'Proxy-Set direct-descriptor CLI regression'
proxy_set_cli_test="$(braced_rust_item_source "$proxy_set_cli" '^fn[[:space:]]+run_wasm_backend_succeeds_for_proxy_set_direct_descriptor_invariants[[:space:]]*[(]')"
require_text_regex_count "$proxy_set_cli_test" '^[[:space:]]*"wasm_proxy_set_direct_descriptor_invariants\.js",$' 1 'Proxy-Set direct-descriptor fixture wiring'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_proxy_set_direct_descriptor_invariants.js ]; then
  fail 'Proxy [[Set]] direct-descriptor invariant fixture must remain present'
fi
require_active_wasm_cli_rust_test \
  "$proxy_set_cli" \
  run_wasm_backend_succeeds_for_proxy_reflect_set_handler_protocol \
  'direct Reflect Set handler-protocol CLI regression'
proxy_reflect_set_cli_test="$(braced_rust_item_source "$proxy_set_cli" '^fn[[:space:]]+run_wasm_backend_succeeds_for_proxy_reflect_set_handler_protocol[[:space:]]*[(]')"
require_text_regex_count "$proxy_reflect_set_cli_test" 'fixture_path\("wasm_proxy_reflect_set_handler_protocol\.js"\)' 1 'direct Reflect Set handler-protocol fixture wiring'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_proxy_reflect_set_handler_protocol.js ]; then
  fail 'direct Reflect Set handler-protocol fixture must remain present'
fi
builtin_arg_presence='emit_builtin_arg_is_present_i32('
require_fixed_string_count \
  crates/lila-aot-wasm/src/functions.rs \
  "$builtin_arg_presence" \
  2 \
  'builtin optional-argument presence authority definition/use'
require_fixed_string_count \
  crates/lila-aot-wasm/src/builtins/reflect.rs \
  "$builtin_arg_presence" \
  3 \
  'three Reflect optional-argument presence consumers'
require_active_wasm_cli_rust_test \
  "$proxy_set_cli" \
  run_wasm_backend_distinguishes_omitted_reflect_optional_arguments \
  'Reflect optional-argument presence CLI regression'
reflect_optional_presence_cli_test="$(braced_rust_item_source "$proxy_set_cli" '^fn[[:space:]]+run_wasm_backend_distinguishes_omitted_reflect_optional_arguments[[:space:]]*[(]')"
require_text_regex_count "$reflect_optional_presence_cli_test" 'fixture_path\("wasm_reflect_optional_argument_presence\.js"\)' 1 'Reflect optional-argument presence fixture wiring'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_reflect_optional_argument_presence.js ]; then
  fail 'Reflect optional-argument presence fixture must remain present'
fi
require_fixed_string_count \
  crates/lila-aot-wasm/src/builtins/reflect.rs \
  'emit_value_to_property_key_locals(' \
  5 \
  'five Reflect full ToPropertyKey consumers'
require_fixed_string_count \
  crates/lila-aot-wasm/src/builtins/reflect.rs \
  'emit_value_to_property_key_payload(' \
  0 \
  'legacy payload-only Reflect ToPropertyKey consumers'
require_active_wasm_cli_rust_test \
  "$proxy_set_cli" \
  run_wasm_backend_preserves_reflect_property_key_conversion \
  'Reflect property-key conversion CLI regression'
reflect_property_key_cli_test="$(braced_rust_item_source "$proxy_set_cli" '^fn[[:space:]]+run_wasm_backend_preserves_reflect_property_key_conversion[[:space:]]*[(]')"
require_text_regex_count "$reflect_property_key_cli_test" 'fixture_path\("wasm_reflect_property_key_conversion\.js"\)' 1 'Reflect property-key conversion fixture wiring'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_reflect_property_key_conversion.js ]; then
  fail 'Reflect property-key conversion fixture must remain present'
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
require_text_regex_count "$proxy_set_invariant_body" '^[[:space:]]*target:[[:space:]]+ProxyTargetLocals,$' 1 'Proxy-Set typed target signature role'
require_text_regex_count "$proxy_set_invariant_body" '^[[:space:]]*key:[[:space:]]+PropertyKeyLocals,$' 1 'Proxy-Set typed property-key signature role'
require_text_regex_count "$proxy_set_invariant_body" '^[[:space:]]*incoming:[[:space:]]+ProxySetValueLocals,$' 1 'Proxy-Set typed incoming-value signature role'
require_text_regex_count \
  "$proxy_set_invariant_body" \
  '^[[:space:]]*self\.emit_direct_own_descriptor_for_proxy_set\(target, key, descriptor, function\)\?;$' \
  1 \
  'Proxy-Set typed direct-own-descriptor projection call'
require_text_regex_count \
  "$proxy_set_invariant_body" \
  '^[[:space:]]*self\.emit_proxy_set_descriptor_same_value_i32\(incoming, descriptor\.data_value, function\)\?;$' \
  1 \
  'Proxy-Set exact frozen-data SameValue consumer'
require_text_regex_count \
  "$proxy_set_invariant_body" \
  '^[[:space:]]*descriptor\.setter\.emit_undefined_i32\(function\);$' \
  1 \
  'Proxy-Set exact tagged-undefined setter consumer'

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
  /A function-like value with no materialized entry still/ { function_fallback = NR }
  END { exit !(entry && intrinsic && function_fallback && entry < intrinsic && intrinsic < function_fallback) }
' <<<"$ordinary_direct_descriptor_body"; then
  fail 'ordinary descriptor precedence must be entry storage, intrinsic fallback, then Function prototype fallback'
fi
function_prototype_fallback="$(sed -n \
  '/A function-like value with no materialized entry still/,/self.release_temp_local(function_like_local)/p' \
  <<<"$ordinary_direct_descriptor_body")"
if ! grep -Fq 'Instruction::LocalGet(fact.present)' <<<"$function_prototype_fallback" \
  || ! grep -Fq 'Instruction::I64Eqz' <<<"$function_prototype_fallback" \
  || ! grep -Fq 'StoredPropertyAttributes::Data {' <<<"$function_prototype_fallback" \
  || ! grep -Fq 'writable: true,' <<<"$function_prototype_fallback" \
  || ! grep -Fq 'enumerable: false,' <<<"$function_prototype_fallback" \
  || ! grep -Fq 'configurable: false,' <<<"$function_prototype_fallback"; then
  fail 'Function prototype fallback must be gated on an absent real entry'
fi

# T11's Proxy record has one typed writer and one typed live reader. Keep the
# raw handler-tag offset private to objects.rs (apart from its heap declaration)
# and keep the reviewed HasProperty, Delete, GetPrototypeOf, SetPrototypeOf,
# IsExtensible, PreventExtensions, DefineOwnProperty, OwnPropertyKeys, direct
# Reflect Set and public descriptor consumers on the reader so no path can
# silently reconstruct an Object tag.
proxy_slot_reader='emit_load_live_proxy_slots('
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'pub(crate) fn emit_load_live_proxy_slots(' \
  1 \
  'typed live-Proxy-slot reader authority'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs "$proxy_slot_reader" 9 'live-Proxy-slot reader definition/internal call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/object/get_own_property_descriptor.rs "$proxy_slot_reader" 1 'public descriptor live-Proxy-slot reader call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/reflect.rs "$proxy_slot_reader" 1 'live-Proxy-slot reader call in Reflect builtins'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'HEAP_PROXY_HANDLER_TAG_OFFSET' 2 'Proxy handler-tag writer/reader authority'
proxy_handler_tag_files="$(grep -RFl --include='*.rs' 'HEAP_PROXY_HANDLER_TAG_OFFSET' crates/lila-aot-wasm/src | sort || true)"
expected_proxy_handler_tag_files="$(printf '%s\n' \
  crates/lila-aot-wasm/src/heap.rs \
  crates/lila-aot-wasm/src/objects.rs | sort)"
if [ "$proxy_handler_tag_files" != "$expected_proxy_handler_tag_files" ]; then
  fail 'Proxy handler-tag heap access must stay inside the typed slot authority'
fi

proxy_prevent_extensions_dispatch="$(sed -n \
  '/pub(crate) fn emit_object_prevent_extensions(/,/pub(crate) fn emit_object_is_extensible_i32(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
if ! grep -Fq 'self.emit_load_live_proxy_slots(' <<<"$proxy_prevent_extensions_dispatch"; then
  fail 'Proxy PreventExtensions must retain target and handler tags through the typed live-slot reader'
fi

proxy_set_prototype_of_dispatch="$(sed -n \
  '/pub(crate) fn emit_object_set_prototype_of_i32(/,/pub(crate) fn emit_ordinary_set_prototype_of_i32(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
for required_proxy_set_prototype_of_seam in \
  'self.emit_load_live_proxy_slots(' \
  'ProxySlotLocals::new(' \
  'ProxyTargetLocals::new(' \
  'ProxyHandlerLocals::new(' \
  'ProxyRevocationRoute::ObjectMutationRealmToActiveHandler' \
  'self.emit_object_read_without_throw_propagation(' \
  'self.emit_return_current_completion_if_throw(function);' \
  'self.emit_is_callable_i32(' \
  'self.emit_function_or_proxy_call_with_throw_propagation(' \
  'self.emit_object_mutation_type_error_to_active_handler('; do
  if ! grep -Fq "$required_proxy_set_prototype_of_seam" <<<"$proxy_set_prototype_of_dispatch"; then
    fail "Proxy SetPrototypeOf must retain $required_proxy_set_prototype_of_seam"
  fi
done
for forbidden_proxy_set_prototype_of_seam in \
  'HEAP_OBJECT_BOXED_PAYLOAD_OFFSET' \
  'HEAP_OBJECT_BOXED_TAG_OFFSET' \
  'Instruction::LocalSet(handler_tag_local)' \
  'self.emit_object_read_ordinary(' \
  'self.emit_throw_runtime_error_to_active_handler('; do
  if grep -Fq "$forbidden_proxy_set_prototype_of_seam" <<<"$proxy_set_prototype_of_dispatch"; then
    fail "Proxy SetPrototypeOf must not reconstruct or bypass $forbidden_proxy_set_prototype_of_seam"
  fi
done

proxy_reflect_set_dispatch="$(sed -n \
  '/pub(crate) fn compile_reflect_set_builtin(/,/pub(crate) fn compile_reflect_has_builtin(/p' \
  crates/lila-aot-wasm/src/builtins/reflect.rs)"
for required_proxy_reflect_set_seam in \
  'self.emit_load_live_proxy_slots(' \
  'ProxySlotLocals::new(' \
  'ProxyTargetLocals::new(' \
  'ProxyHandlerLocals::new(' \
  'ProxyRevocationRoute::CurrentFunctionRealm' \
  'self.emit_object_read_without_throw_propagation(' \
  'self.emit_return_current_completion_if_throw(function);' \
  'self.emit_is_callable_i32(' \
  'self.emit_function_or_proxy_call_with_throw_propagation(' \
  'self.emit_proxy_set_invariant_check('; do
  if ! grep -Fq "$required_proxy_reflect_set_seam" <<<"$proxy_reflect_set_dispatch"; then
    fail "Reflect Set must retain $required_proxy_reflect_set_seam"
  fi
done
for forbidden_proxy_reflect_set_seam in \
  'HEAP_OBJECT_BOXED_PAYLOAD_OFFSET' \
  'HEAP_OBJECT_BOXED_TAG_OFFSET' \
  'Instruction::LocalSet(handler_tag_local)' \
  'self.emit_object_read_ordinary(' \
  '"Proxy handler is null"'; do
  if grep -Fq "$forbidden_proxy_reflect_set_seam" <<<"$proxy_reflect_set_dispatch"; then
    fail "Reflect Set must not reconstruct or bypass $forbidden_proxy_reflect_set_seam"
  fi
done
proxy_reflect_set_trap_acquisition="$(sed -n \
  '/self.emit_load_live_proxy_slots(/,/self.compile_truthy_tagged_i32(/p' \
  <<<"$proxy_reflect_set_dispatch")"
if grep -Fq 'self.emit_function_handle_call(' <<<"$proxy_reflect_set_trap_acquisition" \
  || grep -Fq 'self.emit_propagate_throw_from_locals_if_needed(' <<<"$proxy_reflect_set_trap_acquisition"; then
  fail 'Reflect Set trap acquisition must use the Function-or-Proxy call owner and its one throw route'
fi

proxy_delete_dispatch="$(sed -n \
  '/pub(crate) fn emit_object_delete(/,/pub(crate) fn emit_delete_ordinary_by_tag(/p' \
  crates/lila-aot-wasm/src/objects.rs)"
for required_proxy_delete_seam in \
  'ProxyRevocationRoute::CurrentCompletion' \
  'self.emit_object_read_without_throw_propagation(' \
  'self.emit_propagate_throw_from_locals_if_needed(' \
  'self.emit_is_callable_i32(' \
  'self.emit_function_or_proxy_call_with_throw_propagation(' \
  'ProxyTargetLocals::new(' \
  'PropertyKeyLocals::new('; do
  if ! grep -Fq "$required_proxy_delete_seam" <<<"$proxy_delete_dispatch"; then
    fail "Proxy Delete must retain its typed live-slot, full GetMethod and callable-call seam: $required_proxy_delete_seam"
  fi
done
if grep -Fq 'Instruction::LocalSet(handler_tag_local)' <<<"$proxy_delete_dispatch"; then
  fail 'Proxy Delete must not reconstruct its handler as Object'
fi

# T02 gives T09's private-element environment, storage and access lifecycle one
# private backend owner. Keep the complete family together: widening only some
# methods back into objects.rs would recreate the shared ownership surface this
# boundary removes.
wasm_objects="crates/lila-aot-wasm/src/objects.rs"
wasm_private_elements="crates/lila-aot-wasm/src/objects/private_elements.rs"
wasm_set_path_realm="crates/lila-aot-wasm/src/objects/set_path_realm.rs"
require_file "$wasm_private_elements"
require_file "$wasm_set_path_realm"
require_exact_line_count "$wasm_objects" 'mod private_elements;' 1 'private-elements module declaration'
require_exact_line_count "$wasm_objects" 'mod set_path_realm;' 1 'set-path Realm module declaration'
require_regex_count \
  "$wasm_objects" \
  '^(pub(\([^)]*\))?[[:space:]]+)?mod[[:space:]]+private_elements;' \
  1 \
  'private-elements module declarations'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+private_elements;' "$wasm_objects"; then
  fail "$wasm_objects must keep private_elements private"
fi
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+set_path_realm;' "$wasm_objects"; then
  fail "$wasm_objects must keep set_path_realm private"
fi

private_element_builder_impl="$(sed -n "/^impl<'a> FunctionBuilder<'a> {$/,/^}$/p" "$wasm_private_elements")"
require_text_regex_count \
  "$private_element_builder_impl" \
  '^[[:space:]]*(pub\(crate\)[[:space:]]+)?fn[[:space:]]+[a-z0-9_]+[[:space:]]*\(' \
  18 \
  'private-element FunctionBuilder methods'

for private_element_owner in \
  emit_current_private_environment_to_local \
  emit_private_name_token_to_local \
  emit_private_brand_add \
  emit_private_field_add \
  emit_private_setter_definition_add \
  emit_private_method_definition_add \
  emit_private_getter_definition_add \
  emit_private_element_entry_add \
  emit_private_receiver_kind_guard \
  emit_private_definition_kind_guard \
  emit_private_element_find \
  emit_private_element_definition_find \
  emit_private_brand_has_i32 \
  compile_private_read_to_locals \
  emit_private_read_from_locals \
  compile_private_write_to_locals \
  emit_private_write_from_locals \
  emit_private_brand_guard
do
  require_regex_count \
    "$wasm_private_elements" \
    "^[[:space:]]*(pub\\(crate\\)[[:space:]]+)?fn[[:space:]]+${private_element_owner}[[:space:]]*\\(" \
    1 \
    "$private_element_owner owner"
  require_regex_count \
    "$wasm_objects" \
    "^[[:space:]]*(pub\\(crate\\)[[:space:]]+)?fn[[:space:]]+${private_element_owner}[[:space:]]*\\(" \
    0 \
    "$private_element_owner parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${private_element_owner}[[:space:]]*\\(" \
    1 \
    "$private_element_owner backend owners"
done

for private_element_cross_owner in \
  emit_current_private_environment_to_local \
  emit_private_name_token_to_local \
  emit_private_brand_add \
  emit_private_field_add \
  emit_private_setter_definition_add \
  emit_private_method_definition_add \
  emit_private_getter_definition_add \
  emit_private_element_find \
  emit_private_brand_has_i32 \
  compile_private_read_to_locals \
  compile_private_write_to_locals \
  emit_private_write_from_locals \
  emit_private_brand_guard
do
  require_regex_count \
    "$wasm_private_elements" \
    "^[[:space:]]*pub\\(crate\\)[[:space:]]+fn[[:space:]]+${private_element_cross_owner}[[:space:]]*\\(" \
    1 \
    "$private_element_cross_owner reviewed cross-module visibility"
done

for private_element_internal_owner in \
  emit_private_element_entry_add \
  emit_private_receiver_kind_guard \
  emit_private_definition_kind_guard \
  emit_private_element_definition_find \
  emit_private_read_from_locals
do
  require_regex_count \
    "$wasm_private_elements" \
    "^[[:space:]]*fn[[:space:]]+${private_element_internal_owner}[[:space:]]*\\(" \
    1 \
    "$private_element_internal_owner private visibility"
done

require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?enum[[:space:]]+PrivateElementEntryLocals[[:space:]]*\{' \
  1 \
  'private-element entry carrier'
private_element_entry_domain="$(sed -n '/^enum PrivateElementEntryLocals {$/,/^}$/p' "$wasm_private_elements")"
require_text_regex_count \
  "$private_element_entry_domain" \
  '^[[:space:]]{4}[[:upper:]][[:alnum:]]*[[:space:]]*\{' \
  5 \
  'complete closed private-element entry domain'
for private_element_variant in Brand Field SetterDefinition MethodDefinition GetterDefinition; do
  require_text_regex_count \
    "$private_element_entry_domain" \
    "^[[:space:]]*${private_element_variant}[[:space:]]*\\{" \
    1 \
    "$private_element_variant entry variant"
done

private_element_entry_projection="$(sed -n '/^impl PrivateElementEntryLocals {$/,/^}$/p' "$wasm_private_elements")"
if grep -Eq '(^|[^[:alnum:]])_[[:space:]]*=>' <<<"$private_element_entry_projection"; then
  fail "$wasm_private_elements must project every private-element entry variant exhaustively"
fi
for private_element_variant in Brand Field SetterDefinition MethodDefinition GetterDefinition; do
  require_text_regex_count \
    "$private_element_entry_projection" \
    "PrivateElementEntryLocals::${private_element_variant}|Self::${private_element_variant}" \
    3 \
    "$private_element_variant kind/receiver/value projections"
done

check_no_inline_legacy_includes "$wasm_private_elements"
check_no_inline_legacy_includes "$wasm_set_path_realm"
check_raw_line_budget "$wasm_objects" 21800
check_raw_line_budget "$wasm_private_elements" 1050
check_raw_line_budget "$wasm_set_path_realm" 150

# The Arguments ParameterMap fact is captured, borrowed by indexed operations,
# and consumed by one private child. No parent or sibling can construct its
# paired mapped/slot locals directly.
wasm_functions=crates/lila-aot-wasm/src/functions.rs
wasm_arguments_index_mapping=crates/lila-aot-wasm/src/functions/arguments_index_mapping.rs
require_file "$wasm_arguments_index_mapping"
require_exact_line_count \
  "$wasm_functions" \
  'mod arguments_index_mapping;' \
  1 \
  'private arguments_index_mapping module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+arguments_index_mapping;' "$wasm_functions"; then
  fail "$wasm_functions must keep arguments_index_mapping private"
fi
require_fixed_string_count \
  "$wasm_functions" \
  'arguments_index_mapping::' \
  0 \
  'arguments_index_mapping imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+ArgumentsIndexMappingLocals[[:space:]]*\{' \
  1 \
  'ArgumentsIndexMappingLocals backend owner'
require_regex_count \
  "$wasm_functions" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+ArgumentsIndexMappingLocals[[:space:]]*\{' \
  0 \
  'ArgumentsIndexMappingLocals parent copies'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*ArgumentsIndexMappingLocals[[:space:]]*\{[[:space:]]*mapped,[[:space:]]*slot[[:space:]]*\}' \
  1 \
  'ArgumentsIndexMappingLocals construction sites'

for arguments_mapping_method in \
  emit_arguments_index_mapping_from_descriptor_word \
  emit_arguments_parameter_map_read \
  emit_arguments_parameter_map_write \
  emit_arguments_mapping_restore_on_data_descriptor \
  release_arguments_index_mapping
do
  require_regex_count \
    "$wasm_arguments_index_mapping" \
    "^[[:space:]]*pub\\(crate\\)[[:space:]]+fn[[:space:]]+${arguments_mapping_method}[[:space:]]*\\(" \
    1 \
    "$arguments_mapping_method private-child owner"
  require_regex_count \
    "$wasm_functions" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${arguments_mapping_method}[[:space:]]*\\(" \
    0 \
    "$arguments_mapping_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${arguments_mapping_method}[[:space:]]*\\(" \
    1 \
    "$arguments_mapping_method backend owner"
done

require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_arguments_index_mapping_from_descriptor_word[[:space:]]*\(' \
  5 \
  'Arguments mapping capture calls'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_arguments_parameter_map_read[[:space:]]*\(' \
  3 \
  'Arguments ParameterMap read calls'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_arguments_parameter_map_write[[:space:]]*\(' \
  4 \
  'Arguments ParameterMap write calls'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_arguments_mapping_restore_on_data_descriptor[[:space:]]*\(' \
  1 \
  'Arguments mapping restore calls'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.release_arguments_index_mapping[[:space:]]*\(' \
  5 \
  'Arguments mapping release calls'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'mapping\.mapped' \
  4 \
  'Arguments mapped-field owner accesses'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'mapping\.slot' \
  6 \
  'Arguments slot-field owner accesses'

check_no_inline_legacy_includes "$wasm_arguments_index_mapping"
check_raw_line_budget "$wasm_arguments_index_mapping" 180

# A created realm's Array prototype progresses from reserved storage to an
# initialized Array exotic object in one private child. The parent and siblings
# can use the inferred states, but cannot name or construct either state.
wasm_created_realm_array_prototype=crates/lila-aot-wasm/src/functions/created_realm_array_prototype.rs
require_file "$wasm_created_realm_array_prototype"
require_exact_line_count \
  "$wasm_functions" \
  'mod created_realm_array_prototype;' \
  1 \
  'private created_realm_array_prototype module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+created_realm_array_prototype;' "$wasm_functions"; then
  fail "$wasm_functions must keep created_realm_array_prototype private"
fi
require_fixed_string_count \
  "$wasm_functions" \
  'created_realm_array_prototype::' \
  0 \
  'created-Realm Array prototype imports or re-exports'

for created_realm_array_prototype_state in \
  ReservedRealmArrayPrototypeLocal \
  RealmArrayPrototypeLocal
do
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?struct[[:space:]]+${created_realm_array_prototype_state}\\(u32\\);" \
    1 \
    "$created_realm_array_prototype_state backend owner"
  require_fixed_string_count \
    "$wasm_functions" \
    "$created_realm_array_prototype_state" \
    0 \
    "$created_realm_array_prototype_state parent names"
done

require_fixed_string_count \
  "$wasm_created_realm_array_prototype" \
  'ReservedRealmArrayPrototypeLocal(self.reserve_temp_local())' \
  1 \
  'created-Realm Array prototype reserved construction sites'
require_fixed_string_count \
  "$wasm_created_realm_array_prototype" \
  'RealmArrayPrototypeLocal(reserved.0)' \
  1 \
  'created-Realm Array prototype initialized construction sites'
require_fixed_string_count \
  "$wasm_created_realm_array_prototype" \
  'reserved.0' \
  4 \
  'created-Realm Array prototype reserved projections'
require_fixed_string_count \
  "$wasm_created_realm_array_prototype" \
  'prototype.0' \
  5 \
  'created-Realm Array prototype initialized projections'

for created_realm_array_prototype_method in \
  reserve_realm_array_prototype_local \
  emit_initialize_realm_array_prototype \
  emit_store_realm_array_prototype \
  emit_define_realm_array_prototype_data_with_flags \
  emit_bind_realm_array_constructor_prototype \
  release_realm_array_prototype_local
do
  require_regex_count \
    "$wasm_created_realm_array_prototype" \
    "^[[:space:]]*pub\\(crate\\)[[:space:]]+fn[[:space:]]+${created_realm_array_prototype_method}[[:space:]]*\\(" \
    1 \
    "$created_realm_array_prototype_method private-child owner"
  require_regex_count \
    "$wasm_functions" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${created_realm_array_prototype_method}[[:space:]]*\\(" \
    0 \
    "$created_realm_array_prototype_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${created_realm_array_prototype_method}[[:space:]]*\\(" \
    1 \
    "$created_realm_array_prototype_method backend owner"
done

for created_realm_array_prototype_call_census in \
  'reserve_realm_array_prototype_local 1' \
  'emit_initialize_realm_array_prototype 1' \
  'emit_store_realm_array_prototype 1' \
  'emit_define_realm_array_prototype_data_with_flags 3' \
  'emit_bind_realm_array_constructor_prototype 1' \
  'release_realm_array_prototype_local 1'
do
  set -- $created_realm_array_prototype_call_census
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "\\.${1}[[:space:]]*\\(" \
    "$2" \
    "created-Realm Array prototype $1 calls"
done

check_no_inline_legacy_includes "$wasm_created_realm_array_prototype"
check_raw_line_budget "$wasm_created_realm_array_prototype" 220

# Required ordinary default prototypes have one closed selector and one typed
# resolved-Realm witness owner. The parent sees the witness only as an inferred
# value; its raw field and construction stay private to this child.
wasm_required_resolved_realm_ordinary_prototype=crates/lila-aot-wasm/src/functions/required_resolved_realm_ordinary_prototype.rs
require_file "$wasm_required_resolved_realm_ordinary_prototype"
require_exact_line_count \
  "$wasm_functions" \
  'mod required_resolved_realm_ordinary_prototype;' \
  1 \
  'private required_resolved_realm_ordinary_prototype module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+required_resolved_realm_ordinary_prototype;' "$wasm_functions"; then
  fail "$wasm_functions must keep required_resolved_realm_ordinary_prototype private"
fi
require_exact_line_count \
  "$wasm_functions" \
  'pub(crate) use required_resolved_realm_ordinary_prototype::OrdinaryDefaultPrototype;' \
  1 \
  'OrdinaryDefaultPrototype narrow re-export'
require_fixed_string_count \
  "$wasm_functions" \
  'required_resolved_realm_ordinary_prototype::' \
  1 \
  'required resolved-Realm ordinary prototype imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?enum[[:space:]]+OrdinaryDefaultPrototype[[:space:]]*\{' \
  1 \
  'OrdinaryDefaultPrototype backend owner'
require_regex_count \
  "$wasm_functions" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?enum[[:space:]]+OrdinaryDefaultPrototype[[:space:]]*\{' \
  0 \
  'OrdinaryDefaultPrototype parent copies'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*pub\(super\)[[:space:]]+struct[[:space:]]+ResolvedRealmOrdinaryPrototypeLocal\(u32\);' \
  1 \
  'ResolvedRealmOrdinaryPrototypeLocal backend owner'
require_fixed_string_count \
  "$wasm_functions" \
  'ResolvedRealmOrdinaryPrototypeLocal' \
  0 \
  'ResolvedRealmOrdinaryPrototypeLocal parent names'
require_fixed_string_count \
  "$wasm_required_resolved_realm_ordinary_prototype" \
  'ResolvedRealmOrdinaryPrototypeLocal(prototype_local)' \
  1 \
  'resolved-Realm ordinary prototype construction sites'
require_fixed_string_count \
  "$wasm_required_resolved_realm_ordinary_prototype" \
  'prototype.0' \
  2 \
  'resolved-Realm ordinary prototype projections'

ordinary_default_prototype_domain="$(sed -n '/^pub(crate) enum OrdinaryDefaultPrototype {$/,/^}$/p' "$wasm_required_resolved_realm_ordinary_prototype")"
require_text_regex_count \
  "$ordinary_default_prototype_domain" \
  '^[[:space:]]{4}([[:alnum:]]+|MessageError\(ErrorMessageConstructorKind\)),[[:space:]]*$' \
  9 \
  'complete ordinary default-prototype domain'
ordinary_default_prototype_offsets="$(sed -n '/^impl OrdinaryDefaultPrototype {$/,/^}$/p' "$wasm_required_resolved_realm_ordinary_prototype")"
if grep -Eq '(^|[^[:alnum:]])_[[:space:]]*=>' <<<"$ordinary_default_prototype_offsets"; then
  fail "$wasm_required_resolved_realm_ordinary_prototype must map every ordinary default prototype exhaustively"
fi
for ordinary_default_prototype_variant in Object MessageError String Number Boolean Date Iterator RegExp Promise; do
  require_text_regex_count \
    "$ordinary_default_prototype_offsets" \
    "Self::${ordinary_default_prototype_variant}(\\(kind\\))?[[:space:]]*=>" \
    1 \
    "$ordinary_default_prototype_variant ordinary default-prototype offset"
done

for required_ordinary_prototype_method_visibility in \
  'emit_load_required_resolved_realm_ordinary_prototype pub\(super\)' \
  'emit_required_new_target_realm_ordinary_prototype pub\(crate\)' \
  'emit_install_resolved_realm_ordinary_prototype pub\(super\)'
do
  set -- $required_ordinary_prototype_method_visibility
  require_regex_count \
    "$wasm_required_resolved_realm_ordinary_prototype" \
    "^[[:space:]]*${2}[[:space:]]+fn[[:space:]]+${1}[[:space:]]*\\(" \
    1 \
    "$1 private-child owner"
  require_regex_count \
    "$wasm_functions" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${1}[[:space:]]*\\(" \
    0 \
    "$1 parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${1}[[:space:]]*\\(" \
    1 \
    "$1 backend owner"
done

for required_ordinary_prototype_call_census in \
  'emit_load_required_resolved_realm_ordinary_prototype 5' \
  'emit_install_resolved_realm_ordinary_prototype 5' \
  'emit_required_new_target_realm_ordinary_prototype 3'
do
  set -- $required_ordinary_prototype_call_census
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "\\.${1}[[:space:]]*\\(" \
    "$2" \
    "required resolved-Realm ordinary prototype $1 calls"
done

check_no_inline_legacy_includes "$wasm_required_resolved_realm_ordinary_prototype"
check_raw_line_budget "$wasm_required_resolved_realm_ordinary_prototype" 180

# The active function Realm's Array prototype is a one-shot proof: one private
# child constructs it, and exactly two bounded consumers install it with an
# Array payload. Iterator-toArray owns one consumer; Proxy dispatch owns the
# other so trap-visible argument Arrays use the execution Realm.
wasm_current_function_realm_array_prototype=crates/lila-aot-wasm/src/functions/current_function_realm_array_prototype.rs
wasm_proxy_execution_realm=crates/lila-aot-wasm/src/functions/proxy_execution_realm.rs
require_file "$wasm_current_function_realm_array_prototype"
require_file "$wasm_proxy_execution_realm"
require_exact_line_count \
  "$wasm_functions" \
  'mod current_function_realm_array_prototype;' \
  1 \
  'private current_function_realm_array_prototype module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+current_function_realm_array_prototype;' "$wasm_functions"; then
  fail "$wasm_functions must keep current_function_realm_array_prototype private"
fi
require_fixed_string_count \
  "$wasm_functions" \
  'current_function_realm_array_prototype::' \
  0 \
  'current-function Realm Array prototype imports or re-exports'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?struct[[:space:]]+CurrentFunctionRealmArrayPrototypeLocal\(u32\);' \
  1 \
  'CurrentFunctionRealmArrayPrototypeLocal backend owner'
require_fixed_string_count \
  "$wasm_functions" \
  'CurrentFunctionRealmArrayPrototypeLocal' \
  0 \
  'CurrentFunctionRealmArrayPrototypeLocal parent names'
require_fixed_string_count \
  "$wasm_current_function_realm_array_prototype" \
  'CurrentFunctionRealmArrayPrototypeLocal(prototype_local)' \
  1 \
  'current-function Realm Array prototype construction sites'
require_fixed_string_count \
  "$wasm_current_function_realm_array_prototype" \
  'prototype.0' \
  2 \
  'current-function Realm Array prototype projections'

for current_realm_array_prototype_method in \
  emit_load_current_function_realm_array_prototype \
  emit_install_current_function_realm_array_prototype
do
  require_regex_count \
    "$wasm_current_function_realm_array_prototype" \
    "^[[:space:]]*pub\\(crate\\)[[:space:]]+fn[[:space:]]+${current_realm_array_prototype_method}[[:space:]]*\\(" \
    1 \
    "$current_realm_array_prototype_method private-child owner"
  require_regex_count \
    "$wasm_functions" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${current_realm_array_prototype_method}[[:space:]]*\\(" \
    0 \
    "$current_realm_array_prototype_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${current_realm_array_prototype_method}[[:space:]]*\\(" \
    1 \
    "$current_realm_array_prototype_method backend owner"
done

require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_load_current_function_realm_array_prototype[[:space:]]*\(' \
  2 \
  'current-function Realm Array prototype load calls'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_install_current_function_realm_array_prototype[[:space:]]*\(' \
  2 \
  'current-function Realm Array prototype install calls'
for current_realm_array_prototype_consumer in \
  crates/lila-aot-wasm/src/builtins/array.rs \
  "$wasm_proxy_execution_realm"
do
  require_fixed_string_count \
    "$current_realm_array_prototype_consumer" \
    'emit_load_current_function_realm_array_prototype(function)' \
    1 \
    'current-function Realm Array prototype load consumer'
  require_fixed_string_count \
    "$current_realm_array_prototype_consumer" \
    'emit_install_current_function_realm_array_prototype(' \
    1 \
    'current-function Realm Array prototype install consumer'
done
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_alloc_array_payload_with_length_in_current_function_realm[[:space:]]*\(' \
  2 \
  'current-function Realm Array allocator consumers'

check_no_inline_legacy_includes "$wasm_current_function_realm_array_prototype"
check_raw_line_budget "$wasm_current_function_realm_array_prototype" 110

# GetFunctionRealm's raw result lifecycle has one private owner. The parent
# exposes only the route selected by sibling consumers and privately imports
# the resolved witness used by its retained allocation paths.
wasm_function_realm=crates/lila-aot-wasm/src/functions/function_realm.rs
require_file "$wasm_function_realm"
require_exact_line_count \
  "$wasm_functions" \
  'mod function_realm;' \
  1 \
  'private function_realm module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+function_realm;' "$wasm_functions"; then
  fail "$wasm_functions must keep function_realm private"
fi
require_exact_line_count \
  "$wasm_functions" \
  'pub(crate) use function_realm::FunctionRealmRevokedRoute;' \
  1 \
  'FunctionRealmRevokedRoute re-export'
require_regex_count \
  "$wasm_functions" \
  '^pub(\([^)]*\))?[[:space:]]+use[[:space:]]+function_realm::' \
  1 \
  'function_realm public re-exports'
require_exact_line_count \
  "$wasm_functions" \
  'use function_realm::ResolvedFunctionRealmLocal;' \
  1 \
  'private ResolvedFunctionRealmLocal import'

for function_realm_type in \
  FunctionRealmOutcome \
  FunctionRealmResultLocals \
  ResolvedFunctionRealmLocal \
  FunctionRealmRevokedRoute
do
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?(enum|struct)[[:space:]]+${function_realm_type}([[:space:](<{]|$)" \
    1 \
    "$function_realm_type backend owner"
  require_regex_count \
    "$wasm_functions" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?(enum|struct)[[:space:]]+${function_realm_type}([[:space:](<{]|$)" \
    0 \
    "$function_realm_type parent copies"
done

for function_realm_method in \
  emit_get_function_realm \
  emit_route_function_realm_result \
  release_resolved_function_realm_local
do
  require_regex_count \
    "$wasm_function_realm" \
    "^[[:space:]]*pub\\(crate\\)[[:space:]]+fn[[:space:]]+${function_realm_method}[[:space:]]*\\(" \
    1 \
    "$function_realm_method private-child owner"
  require_regex_count \
    "$wasm_functions" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${function_realm_method}[[:space:]]*\\(" \
    0 \
    "$function_realm_method parent copies"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${function_realm_method}[[:space:]]*\\(" \
    1 \
    "$function_realm_method backend owner"
done

require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_get_function_realm[[:space:]]*\(' \
  5 \
  'GetFunctionRealm product calls'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.emit_route_function_realm_result[[:space:]]*\(' \
  5 \
  'GetFunctionRealm route calls'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '\.release_resolved_function_realm_local[[:space:]]*\(' \
  5 \
  'resolved FunctionRealm release calls'

check_no_inline_legacy_includes "$wasm_function_realm"
check_raw_line_budget "$wasm_functions" 12363
check_raw_line_budget "$wasm_function_realm" 310

# The write-never static-generator IR cache had one synthetic backend protocol.
# Its names, producer and close-time marker consumer must not regrow separately.
require_tree_regex_count \
  crates/lila-ir/src \
  'LILA_STATIC_GENERATOR_' \
  0 \
  'retired static-generator IR names'
for retired_static_generator_backend_spelling in \
  'LILA_STATIC_GENERATOR_' \
  '[$]LilaStaticGenerator' \
  'StaticGeneratorValues' \
  'emit_exhaust_static_generator_iterator_if_marked'
do
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "$retired_static_generator_backend_spelling" \
    0 \
    'retired static-generator backend protocol'
done

# Every ordinary, generator and async catch/finally clause seeds its own empty
# statement-list completion after preserving the incoming completion. The three
# combined owners contain one catch seed and one finally seed.
try_clause_seed_pattern='^[[:space:]]*self\.emit_statement_result\(function, ValueKind::Undefined\);[[:space:]]*$'
for try_clause_seed_owner in \
  'compile_try_catch emit_generator_state_in_range 1' \
  'compile_generator_try_catch compile_generator_try_finally 1' \
  'compile_generator_try_finally compile_generator_try_catch_finally 1' \
  'compile_generator_try_catch_finally compile_async_try_catch 2' \
  'compile_async_try_catch compile_async_try_catch_finally 1' \
  'compile_async_try_catch_finally compile_async_try_finally 2' \
  'compile_async_try_finally compile_try_finally 1' \
  'compile_try_finally compile_async_disposable_scope 1' \
  'compile_try_catch_finally compile_while 2'
do
  set -- $try_clause_seed_owner
  try_clause_seed_owner_source="$(sed -n "/fn ${1}(/,/fn ${2}(/p" "$wasm_control_flow")"
  require_text_regex_count \
    "$try_clause_seed_owner_source" \
    "$try_clause_seed_pattern" \
    "$3" \
    "$1 empty-completion clause-entry seeds"
done

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-module-boundaries: ok\n'
