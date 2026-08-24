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
# T02's for-of boundary owns every specialization decision, the lowering-only
# protocol carrier and the closed resumable array-walk classification. The
# statement dispatcher is the sole caller; shared loop/environment helpers and
# public statement/protocol IR remain in their existing owners.
ir_for_of_lowering="crates/lila-ir/src/lowering/for_of.rs"
require_file "$ir_for_of_lowering"
require_exact_line_count "$ir_lowering" 'mod for_of;' 1 'private for-of module declaration'
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
for private_owner in lower_async_for_of_array_with_body_await lower_for_of_head; do
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
  1 \
  'private resumable array-walk classification carrier'
require_fixed_string_count \
  "crates/lila-ir/src/lowering_helpers.rs" \
  'enum AsyncForOfArrayWalkForm' \
  0 \
  'resumable array-walk carrier declarations in the former helper owner'
require_tree_regex_count \
  "crates/lila-ir/src" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?enum[[:space:]]+AsyncForOfArrayWalkForm([[:space:]]|\{)' \
  1 \
  'resumable array-walk carrier'
require_fixed_string_count \
  "$ir_for_of_lowering" \
  'struct ForOfLoweringIr' \
  1 \
  'private for-of protocol carrier'
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
# The statement-facing wrapper is the child's only Rust-visible item. This one
# count makes leaking either carrier, any field or any helper fail closed.
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
# Measured after formatting the extraction: 1,036 raw lines. The margin is for
# maintenance of the complete for-of lowering family, not unrelated lowering.
check_raw_line_budget "$ir_for_of_lowering" 1100
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
# T02's call-expression boundary keeps direct-call recognition and lowering in
# one child module. The parent owns expression dispatch and reusable helpers,
# but cannot regrow a second implementation of its largest former method.
ir_call_expression_lowering="crates/lila-ir/src/lowering/call_expression.rs"
require_file "$ir_call_expression_lowering"
require_module_decl "$ir_lowering" "call_expression"
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
check_no_inline_legacy_includes "$ir_call_expression_lowering"
# Measured immediately after extraction: 3,144 raw lines. The margin is for
# maintenance of the direct-call family, not unrelated lowering.
check_raw_line_budget "$ir_call_expression_lowering" 3200
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
# Measured after formatting the extraction: 2,150 raw lines. The margin is for
# maintenance of this exhaustive result table, not unrelated lowering.
check_raw_line_budget "$ir_builtin_call_info_lowering" 2250
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
# Measured immediately after extraction: 1,327 raw lines. The margin is for
# maintenance of this class-definition family, not unrelated lowering.
check_raw_line_budget "$ir_class_definition_lowering" 1400
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
# T02's static-JSON parse boundary owns only the static reviver specialization,
# its static-string input recovery and the complete private parser. Dynamic
# reviver target discovery/observation remains in the parent because the
# ordinary JSON.parse path consumes it too.
ir_static_json_parse_lowering="crates/lila-ir/src/lowering/static_json_parse.rs"
require_file "$ir_static_json_parse_lowering"
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
static_json_module_context="$(sed -n '/^mod statement;$/,+2p' "$ir_lowering")"
if [ "$static_json_module_context" != $'mod statement;\nmod static_json_parse;\nmod super_property_mutation;' ]; then
  fail "$ir_lowering must keep static_json_parse as one private module declaration between statement and super_property_mutation"
fi
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  'use super::*;' \
  1 \
  'parent import'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+try_lower_static_json_parse_reviver[[:space:]]*\(' \
  1 \
  'static JSON.parse specialization entry point'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*fn[[:space:]]+static_json_parse_input[[:space:]]*\(' \
  1 \
  'private static JSON.parse input recovery'
for moved_owner in try_lower_static_json_parse_reviver static_json_parse_input; do
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
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+try_lower_static_json_parse_reviver[[:space:]]*\(' \
  1 \
  'ScriptLowerer specialization entry point'
require_text_regex_count \
  "$static_json_lowerer_impl" \
  '^[[:space:]]*fn[[:space:]]+static_json_parse_input[[:space:]]*\(' \
  1 \
  'ScriptLowerer static-input helper'
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
# The static input helper plus eleven parser methods are private; the one
# specialization entry point is the entire Rust-visible child surface. The
# modifier-aware total prevents const/async/unsafe/extern/default additions
# from evading the closed inventory.
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*fn[[:space:]]+' \
  12 \
  'private function'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*[<(]' \
  13 \
  'total function declaration'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' \
  1 \
  'Rust-visible item'
require_regex_count \
  "$ir_static_json_parse_lowering" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?((unsafe|auto)[[:space:]]+)*(struct|enum|union|type|trait)[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*([[:space:]]|<|\{|\(|=|;|:)' \
  1 \
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
  '            self.observe_json_parse_reviver_targets(reviver_targets);' \
  1 \
  'dynamic JSON.parse target observation'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '        let input = self.static_json_parse_input(&args[0])?;' \
  1 \
  'static JSON.parse input recovery'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '        let parsed_value = JsonStaticParser::new(&input).parse()?;' \
  1 \
  'static JSON.parse parser invocation'
require_exact_line_count \
  "$ir_static_json_parse_lowering" \
  '        if self.known_json_parse_reviver_targets(args).is_empty() {' \
  1 \
  'static JSON.parse known-target proof'
require_fixed_string_count \
  "$ir_static_json_parse_lowering" \
  'observe_json_parse_reviver_targets' \
  0 \
  'dynamic target observation copied into static-JSON parse child'
require_exact_line_count \
  'crates/lila-ir/src/lowering/call_expression.rs' \
  '                        self.try_lower_static_json_parse_reviver(&function_id, &args)' \
  1 \
  'direct-function static JSON.parse sibling call'
require_exact_line_count \
  'crates/lila-ir/src/lowering/call_expression.rs' \
  '            self.try_lower_static_json_parse_reviver(&effective_function_id, &args)' \
  1 \
  'effective-function static JSON.parse sibling call'
require_regex_count \
  'crates/lila-ir/src/lowering/call_expression.rs' \
  '^[[:space:]]*self\.try_lower_static_json_parse_reviver[[:space:]]*\(' \
  2 \
  'total static JSON.parse sibling call'
require_fixed_string_count \
  'crates/lila-ir/src/lowering/call_expression.rs' \
  'try_lower_static_json_parse_reviver' \
  2 \
  'static JSON.parse sibling-call identifier use'
require_fixed_string_count \
  "$ir_lowering" \
  'try_lower_static_json_parse_reviver' \
  0 \
  'static JSON.parse specialization use outside child module'
while IFS= read -r caller; do
  case "$caller" in
    "$ir_static_json_parse_lowering"|'crates/lila-ir/src/lowering/call_expression.rs') continue ;;
  esac
  if grep -Fq 'try_lower_static_json_parse_reviver' "$caller"; then
    fail "unexpected static JSON.parse specialization use: $caller"
  fi
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
# Measured after formatting the exact extraction: 242 raw lines. The margin is
# for maintenance of static JSON parsing only, not dynamic target analysis.
check_raw_line_budget "$ir_static_json_parse_lowering" 280
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
crates/lila-ir/src/lib.rs
crates/lila-ir/src/lowering.rs'
if [ "$array_destructuring_variant_product_files" != "$expected_array_destructuring_variant_product_files" ]; then
  fail "direct ArrayDestructuringEvaluationIr variant use must stay in the reviewed four product files: $array_destructuring_variant_product_files"
fi
for product_variant_spec in \
  'crates/lila-ir/src/lowering.rs|5' \
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
  crates/lila-aot-wasm/src/module.rs \
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
  || ! grep -Fq 'compilation.compile_into(&self.globals, &mut code)?' crates/lila-aot-wasm/src/module.rs \
  || ! grep -Fq 'code.push(EmittedFunction::new(FunctionIdentity::Main, main));' crates/lila-aot-wasm/src/emit.rs \
  || ! grep -Fq 'CompiledModulePackage::append_remaining_functions;' crates/lila-aot-wasm/src/module.rs; then
  fail 'main must compile into package-owned code through its exact finalized globals'
fi
for rejected_surface in runtime_globals push_main_to append_types_to append_globals_to; do
  if grep -Fq "${rejected_surface}(" crates/lila-aot-wasm/src/module.rs; then
    fail "the finalized module package must not expose split assembly surface: ${rejected_surface}"
  fi
done
if grep -Fq 'impl FnOnce(&FinalizedModuleGlobals)' crates/lila-aot-wasm/src/module.rs; then
  fail 'the finalized module package must use the closed main compiler, not an arbitrary callback'
fi
if grep -Fq 'CompilingModulePackage' crates/lila-aot-wasm/src/module.rs; then
  fail 'main compilation must return the one compiled package, not an independently consumable code package'
fi
require_fixed_string_count \
  crates/lila-aot-wasm/src/module.rs \
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
    find crates/lila-aot-wasm/src -type f -name '*.rs' ! -path 'crates/lila-aot-wasm/src/module.rs' -print0 \
      | xargs -0 grep -Fn "module.section(&${sealed_section})" || true
  )"
  if [ -n "$sealed_section_escapes" ]; then
    fail "sealed runtime ${sealed_section} section escaped consume-once package assembly: ${sealed_section_escapes}"
  fi
done
global_section_constructor_escapes="$(
  find crates/lila-aot-wasm/src -type f -name '*.rs' \
    ! -path 'crates/lila-aot-wasm/src/module.rs' \
    ! -path "$wasm_gc_types" -print0 \
    | xargs -0 grep -Fn 'GlobalSection::new()' || true
)"
if [ -n "$global_section_constructor_escapes" ] \
  || sed '/^#\[cfg(test)\]/,$d' "$wasm_gc_types" | grep -Fq 'GlobalSection::new()'; then
  fail "production GlobalSection construction must stay in crates/lila-aot-wasm/src/module.rs: $global_section_constructor_escapes"
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
if ! grep -q '^pub(super) enum AtomicsBuiltin' "$wasm_atomics_builtins" \
  || ! grep -q '^enum AtomicsIntegerOperation' "$wasm_atomics_builtins" \
  || ! grep -q '^enum AtomicsRmwOperation' "$wasm_atomics_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_atomics_builtins"; then
  fail "$wasm_atomics_builtins must dispatch through the closed Atomics builtin/integer/RMW domains"
fi
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.emit_atomics_builtin(' \
  14 \
  'Atomics builtin delegate'
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
if grep -q 'StandardBuiltinId::' "$wasm_atomics_builtins"; then
  fail "$wasm_atomics_builtins must accept only its closed family domains, not StandardBuiltinId"
fi
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' "$wasm_atomics_builtins"; then
  fail "$wasm_atomics_builtins must keep family matches exhaustive without catch-all arms"
fi
# Measured immediately after extraction: 2,767 raw lines before formatting.
check_raw_line_budget "$wasm_atomics_builtins" 2850

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
  || ! grep -q '^    BoundFunctionInvoker,$' "$wasm_function_builtins" \
  || ! grep -q '^        match builtin {' "$wasm_function_builtins"; then
  fail "$wasm_function_builtins must dispatch through the closed FunctionBuiltin domain"
fi
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'self.emit_function_builtin(' \
  8 \
  'Function builtin delegate'
require_fixed_string_count \
  "$wasm_standard_builtins" \
  'FunctionBuiltin::BoundFunctionInvoker' \
  1 \
  'bound-function invoker delegate'
bound_function_invoker_delegate="$(
  sed -n \
    '/^            StandardBuiltinId::BoundFunctionInvoker => {$/,/^            }$/p' \
    "$wasm_standard_builtins"
)"
if [ "$(printf '%s\n' "$bound_function_invoker_delegate" | wc -l)" -ne 3 ] \
  || ! grep -Fqx \
    '                self.emit_function_builtin(FunctionBuiltin::BoundFunctionInvoker, function)?' \
    <<<"$bound_function_invoker_delegate"; then
  fail "$wasm_standard_builtins must keep BoundFunctionInvoker as one typed delegate"
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
# Measured after moving the hidden invoker body into this owner: 486 raw lines.
# The narrow margin is for maintenance of this family, not adjacent builtin
# implementations.
check_raw_line_budget "$wasm_function_builtins" 525

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

# T10's user-facing own-descriptor predicates. These three builtins have
# different input sources and observable coercion orders, but consume the same
# public [[GetOwnProperty]] protocol. Keep those decisions in one closed Rust
# domain and prevent the deleted Array/arguments/ordinary representation scans
# from returning in any wrapper.
own_descriptor_predicate_file="crates/lila-aot-wasm/src/builtins/object.rs"
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

own_descriptor_predicate_body="$(sed -n \
  '/^    fn compile_object_own_descriptor_predicate_builtin(/,/^    pub(super) fn compile_object_has_own_builtin(/p' \
  "$own_descriptor_predicate_file")"
if [ "$(grep -Fc 'match builtin {' <<<"$own_descriptor_predicate_body" || true)" -ne 3 ]; then
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
  /match builtin \{/ { matches += 1 }
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
  'compile_object_has_own_builtin|compile_object_is_builtin|OwnDescriptorPredicateBuiltin::ObjectHasOwn' \
  'compile_object_prototype_has_own_property_builtin|compile_object_prototype_lookup_builtin|OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty' \
  'compile_object_prototype_property_is_enumerable_builtin|compile_object_prototype_is_prototype_of_builtin|OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable'
do
  wrapper="${own_predicate_wrapper_spec%%|*}"
  rest="${own_predicate_wrapper_spec#*|}"
  next_wrapper="${rest%%|*}"
  variant="${rest#*|}"
  wrapper_body="$(sed -n \
    "/^    pub(super) fn ${wrapper}(/,/^    pub(super) fn ${next_wrapper}(/p" \
    "$own_descriptor_predicate_file")"
  if [ "$(grep -Fc 'self.compile_object_own_descriptor_predicate_builtin(' <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc "$variant" <<<"$wrapper_body" || true)" -ne 1 ] \
    || [ "$(grep -Fc 'self.' <<<"$wrapper_body" || true)" -ne 1 ]; then
    fail "$wrapper must be a one-call selection of $variant"
  fi
  if grep -Eq 'Instruction::|HEAP_|emit_|reserve_temp_local|StandardBuiltinId::' <<<"$wrapper_body"; then
    fail "$wrapper must not contain a representation-specific descriptor path"
  fi
done

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
# and keep the reviewed HasProperty, Delete, GetPrototypeOf, IsExtensible,
# PreventExtensions, OwnPropertyKeys and public descriptor consumers on the
# reader so no path can silently reconstruct an Object tag.
proxy_slot_reader='emit_load_live_proxy_slots('
require_fixed_string_count \
  crates/lila-aot-wasm/src/objects.rs \
  'pub(crate) fn emit_load_live_proxy_slots(' \
  1 \
  'typed live-Proxy-slot reader authority'
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs "$proxy_slot_reader" 7 'live-Proxy-slot reader definition/internal call'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/object.rs "$proxy_slot_reader" 1 'public descriptor live-Proxy-slot reader call'
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

proxy_delete_dispatch="$(sed -n \
  '/pub(crate) fn emit_object_delete_with_depth(/,/pub(crate) fn emit_delete_ordinary_by_tag(/p' \
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

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-module-boundaries: ok\n'
