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

require_fixed_string_present() {
  file="$1"
  needle="$2"
  description="$3"
  if ! grep -Fq "$needle" "$file"; then
    fail "$file must consume $description: $needle"
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

# Shared item reader must precede every guard that consumes its projection.
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
# Optional References, conditional facts, object-literal evaluation and staged
# compound References have
# actual private owners; the expression dispatcher retains their sole entries.
for lowering_owner_entry in \
  'optional_chain:lower_optional_property_chain' \
  'conditional_flow:capture_conditional_flow_facts' \
  'object_literal:lower_object_literal' \
  'generator_compound_assignment:lower_generator_compound_assignment'
do
  lowering_owner="${lowering_owner_entry%%:*}"
  lowering_entry="${lowering_owner_entry#*:}"
  lowering_owner_path="crates/lila-ir/src/lowering/${lowering_owner}.rs"
  require_file "$lowering_owner_path"
  require_exact_line_count "$ir_lowering" "mod ${lowering_owner};" 1 'private expression-family owner attachment'
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${lowering_owner};" "$ir_lowering"; then
    fail "$ir_lowering must keep ${lowering_owner} private"
  fi
  require_fixed_string_count "$lowering_owner_path" "pub(super) fn ${lowering_entry}(" 1 'consumed expression-family entry'
  require_fixed_string_count "$ir_lowering" "fn ${lowering_entry}(" 0 'no parent expression-family implementation copy'
  check_no_inline_legacy_includes "$lowering_owner_path"
done
require_fixed_string_count "$ir_lowering" 'self.lower_generator_compound_assignment(' 1 'sole staged compound Reference dispatch consumer'
check_raw_line_budget "crates/lila-ir/src/lowering/generator_compound_assignment.rs" 120
# Suspended compound/logical assignment retains one native Identifier Reference.
# The allocator and branch consumer stay beside lowering; the backend restores
# that same reference through its private transport owner.
for reference_lowering_owner in generator_identifier_reference generator_logical_assignment; do
  reference_lowering_path="crates/lila-ir/src/lowering/${reference_lowering_owner}.rs"
  require_file "$reference_lowering_path"
  require_exact_line_count "$ir_lowering" "mod ${reference_lowering_owner};" 1 'private suspended Identifier Reference owner attachment'
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${reference_lowering_owner};" "$ir_lowering"; then
    fail "$ir_lowering must keep ${reference_lowering_owner} private"
  fi
  check_no_inline_legacy_includes "$reference_lowering_path"
done
require_regex_count crates/lila-ir/src/lowering/generator_identifier_reference.rs '^pub\(super\) struct RetainedGeneratorIdentifierReference \{' 1 'private consumed Identifier Reference lifecycle'
ir_resumable_reference="crates/lila-ir/src/lowering/async_generator_assignment.rs"
require_exact_line_count "$ir_lowering" 'mod async_generator_assignment;' 1 'private shared assignment Reference owner attachment'
require_tree_regex_count crates/lila-ir/src/lowering '^[[:space:]]*pub\(super\) fn capture_resumable_assignment_reference\(' 1 'sole checked compound/logical Reference acquisition algorithm'
require_fixed_string_count "$ir_resumable_reference" 'RetainedGeneratorIdentifierReference::capture(' 1 'shared Identifier Get-before-RHS acquisition'
require_fixed_string_count crates/lila-ir/src/lowering/generator_compound_assignment.rs 'self.capture_resumable_assignment_reference(' 1 'compound consumes the shared actual Reference owner'
require_fixed_string_count crates/lila-ir/src/lowering/generator_logical_assignment.rs 'self.capture_resumable_assignment_reference(' 1 'logical consumes the shared actual Reference owner'
# The shared read/write target now owns plain assignment's write-only capture
# as well as compound/logical GetValue: 151 measured lines, one lifecycle.
check_raw_line_budget crates/lila-ir/src/lowering/generator_identifier_reference.rs 165
check_raw_line_budget crates/lila-ir/src/lowering/generator_logical_assignment.rs 120
# Complete eager operands, plain References, object properties and eager binding
# patterns have consumed private producers. The parent keeps their dispatch.
for staged_owner_entry in \
  'generator_plain_assignment:lower_staged_generator_identifier_assignment:35' \
  'generator_eager_value:lower_staged_generator_eager_value:190' \
  'generator_object_literal:lower_staged_generator_object_literal:135' \
  'generator_pattern_initializer:lower_generator_lexical_pattern_initializer:85' \
  'generator_pattern_assignment:lower_staged_generator_pattern_assignment:45' \
  'for_lexical_environment:lower_for_lexical_environment:80' \
  'operator_values:combine_unary_value:260'
do
  staged_owner="${staged_owner_entry%%:*}"
  staged_entry_budget="${staged_owner_entry#*:}"
  staged_entry="${staged_entry_budget%%:*}"
  staged_budget="${staged_entry_budget#*:}"
  staged_owner_path="crates/lila-ir/src/lowering/${staged_owner}.rs"
  require_file "$staged_owner_path"
  require_exact_line_count "$ir_lowering" "mod ${staged_owner};" 1 'private complete staged/semantic owner attachment'
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+${staged_owner};" "$ir_lowering"; then
    fail "$ir_lowering must keep ${staged_owner} private"
  fi
  require_fixed_string_count "$staged_owner_path" "pub(super) fn ${staged_entry}(" 1 'actual staged/semantic entry owner'
  require_fixed_string_count "$ir_lowering" "fn ${staged_entry}(" 0 'no parent staged/semantic body copy'
  check_no_inline_legacy_includes "$staged_owner_path"
  check_raw_line_budget "$staged_owner_path" "$staged_budget"
done
require_fixed_string_count "$ir_lowering" 'self.lower_staged_generator_identifier_assignment(' 2 'value and discarded plain Reference dispatch'
require_fixed_string_count crates/lila-ir/src/lowering/statement/expression.rs 'self.lower_staged_generator_identifier_assignment(' 1 'ordinary statement plain Reference consumer'
require_fixed_string_count crates/lila-ir/src/lowering/generator_plain_assignment.rs 'RetainedGeneratorIdentifierTarget::capture_write_only(' 1 'plain assignment uses the shared write-only target'
require_fixed_string_count "$ir_lowering" 'self.lower_staged_generator_eager_value(' 1 'complete eager operator dispatch'
require_fixed_string_count "$ir_lowering" 'self.lower_staged_generator_object_literal(' 1 'complete staged object dispatch'
require_fixed_string_count "$ir_lowering" 'self.lower_staged_generator_pattern_assignment(' 1 'complete staged pattern assignment dispatch'
require_fixed_string_count crates/lila-ir/src/lowering/lexical_declaration.rs 'self.lower_generator_lexical_pattern_initializer(' 1 'actual lexical pattern initializer consumer'
require_fixed_string_count crates/lila-ir/src/lowering/var_declaration.rs 'self.lower_generator_var_pattern_initializer(' 1 'actual var pattern initializer consumer'
require_fixed_string_count crates/lila-ir/src/lowering/generator_pattern_initializer.rs 'pub(super) fn lower_generator_var_pattern_initializer(' 1 'shared actual var pattern initializer owner'
for operator_entry in combine_unary_value combine_relational; do
  require_fixed_string_count crates/lila-ir/src/lowering/operator_values.rs "pub(super) fn ${operator_entry}(" 1 'shared evaluated-Value operator body'
  require_fixed_string_count "$ir_lowering" "fn ${operator_entry}(" 0 'no ordinary operator body in parent'
  require_fixed_string_count "$ir_lowering" "self.${operator_entry}(" 1 'ordinary evaluated-Value operator consumer'
  require_fixed_string_count crates/lila-ir/src/lowering/generator_eager_value.rs "self.${operator_entry}(" 1 'staged evaluated-Value operator consumer'
done
check_raw_line_budget crates/lila-ir/src/lowering/object_literal.rs 580

# Checked pattern source carriers and complete optional regions own their
# constructor/state obligations beside the source planner and actual consumer.
ir_generator_source="crates/lila-ir/src/generator_value_branch_source.rs"
for pattern_source_owner_type in \
  'pattern_initializer:GeneratorPatternInitializerSource:85' \
  'pattern_assignment:GeneratorPatternAssignmentSource:85'
do
  pattern_source_owner="${pattern_source_owner_type%%:*}"
  pattern_source_type_budget="${pattern_source_owner_type#*:}"
  pattern_source_type="${pattern_source_type_budget%%:*}"
  pattern_source_budget="${pattern_source_type_budget#*:}"
  pattern_source_path="crates/lila-ir/src/generator_value_branch_source/${pattern_source_owner}.rs"
  require_file "$pattern_source_path"
  require_exact_line_count "$ir_generator_source" "mod ${pattern_source_owner};" 1 'private checked pattern source owner attachment'
  require_exact_line_count "$ir_generator_source" "pub(crate) use ${pattern_source_owner}::${pattern_source_type};" 1 'actual checked pattern source projection'
  require_regex_count "$pattern_source_path" "^pub\(crate\) struct ${pattern_source_type}<'ast>" 1 'checked pattern source carrier'
  require_fixed_string_count "$ir_generator_source" "struct ${pattern_source_type}" 0 'no parent checked pattern carrier copy'
  check_no_inline_legacy_includes "$pattern_source_path"
  check_raw_line_budget "$pattern_source_path" "$pattern_source_budget"
done
# Object-owned suspension has protocol-specific source plans and one physical
# element/Reference algorithm. Opaque operations reach the same GetV/rest/Put.
ir_object_pattern_source="crates/lila-ir/src/generator_value_branch_source/object_pattern.rs"
ir_object_pattern_lowerer="crates/lila-ir/src/lowering/generator_object_pattern.rs"
ir_pattern_target_lowerer="crates/lila-ir/src/lowering/generator_pattern_target.rs"
ir_object_pattern_elements="crates/lila-ir/src/lowering/resumable_pattern/object.rs"
ir_pattern_reference_owner="crates/lila-ir/src/lowering/pattern_target.rs"
ir_object_pattern_operation="crates/lila-ir/src/object_destructuring_operation.rs"
wasm_object_pattern="crates/lila-aot-wasm/src/control_flow/object_destructuring.rs"
for object_pattern_owner in "$ir_object_pattern_source" "$ir_object_pattern_lowerer" "$ir_object_pattern_operation" "$wasm_object_pattern"; do
  require_file "$object_pattern_owner"
  check_no_inline_legacy_includes "$object_pattern_owner"
done
require_exact_line_count "$ir_generator_source" 'mod object_pattern;' 1 'private object-pattern source attachment'
require_exact_line_count "$ir_generator_source" 'pub(crate) use object_pattern::GeneratorObjectPatternSource;' 1 'consumed checked object-pattern source'
require_regex_count "$ir_object_pattern_source" "^pub\(crate\) struct GeneratorObjectPatternSource<'ast>" 1 'sole checked object-pattern source carrier'
require_fixed_string_count "$ir_generator_source" 'struct GeneratorObjectPatternSource' 0 'no parent object-pattern source copy'
require_exact_line_count "$ir_lowering" 'mod generator_object_pattern;' 1 'private staged object-pattern attachment'
for object_pattern_entry in lower_staged_generator_object_pattern lower_staged_generator_object_pattern_with_storage_names; do
  require_fixed_string_count "$ir_object_pattern_lowerer" "pub(super) fn ${object_pattern_entry}(" 1 'actual staged object-pattern entry'
  require_fixed_string_count "$ir_lowering" "fn ${object_pattern_entry}(" 0 'no parent object-pattern lowering copy'
done
require_fixed_string_count crates/lila-ir/src/lowering/generator_pattern_assignment.rs 'self.lower_staged_generator_object_pattern(' 1 'assignment consumes the selected object-pattern owner'
require_fixed_string_count crates/lila-ir/src/lowering/generator_pattern_initializer.rs 'self.lower_staged_generator_object_pattern(' 2 'lexical and var initialization consume the same object-pattern owner'
require_fixed_string_present "$ir_object_pattern_lowerer" 'self.lower_resumable_object_pattern_elements(' 'Generator adapter consumes the shared physical Object owner'
require_tree_regex_count crates/lila-ir/src/lowering '^[[:space:]]*pub\(in crate::lowering\) fn lower_resumable_object_pattern_elements\(' 1 'sole shared Object element algorithm'
require_fixed_string_count "$ir_object_pattern_elements" 'ObjectDestructuringSourceIr::prepare(' 1 'raw acquisition and actual ToObject source factory'
require_fixed_string_count "$ir_object_pattern_elements" 'ObjectDestructuringKeyIr::prepare(' 1 'actual normalized PropertyName producer'
require_fixed_string_count "$ir_object_pattern_elements" 'ObjectDestructuringOperationIr::get_v(' 1 'actual retained GetV operation'
require_fixed_string_count "$ir_object_pattern_elements" 'ObjectDestructuringOperationIr::rest(' 2 'binding and assignment rest operations'
require_fixed_string_count "$ir_pattern_reference_owner" 'ObjectDestructuringOperationIr::put_target(' 1 'actual captured target publication'
require_fixed_string_count "$ir_pattern_reference_owner" 'DestructuringPropertyKeyIr::Computed(raw)' 1 'member Reference preserves its raw key until PutValue'
require_fixed_string_present "$ir_pattern_reference_owner" 'StatementIr::DeclarationEvaluation(value)' 'lexical initialization retains Empty completion evidence'
require_exact_line_count crates/lila-ir/src/lib.rs 'mod object_destructuring_operation;' 1 'private opaque operation owner attachment'
require_exact_line_count "$ir_object_pattern_operation" 'mod tests;' 1 'actual private constructor controls'
for object_pattern_carrier in ObjectDestructuringSourceIr ObjectDestructuringKeyIr; do
  require_regex_count "$ir_object_pattern_operation" "^pub struct ${object_pattern_carrier} \\{" 1 'sole retained operand carrier'
  object_pattern_fields="$(sed -n "/^pub struct ${object_pattern_carrier} {$/,/^}$/p" "$ir_object_pattern_operation")"
  require_text_regex_count "$object_pattern_fields" '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+' 0 'private constructor-owned operand fields'
done
require_exact_line_count "$ir_object_pattern_operation" 'pub struct ObjectDestructuringOperationIr(ObjectDestructuringOperation);' 1 'opaque closed operation payload'
require_exact_line_count "$ir_object_pattern_operation" 'enum ObjectDestructuringOperation {' 1 'private exhaustive operation domain'
require_fixed_string_count "$ir_object_pattern_operation" 'TypedExpr::spec_to_object(raw_read.clone())' 1 'real ToObject before pattern operands'
require_fixed_string_count "$ir_object_pattern_operation" 'TypedExpr::spec_to_property_key(raw_key)' 1 'real ToPropertyKey before GetV'
require_fixed_string_count "$ir_object_pattern_operation" 'row.name == binding.name || row.slot == binding.slot' 1 'exact allocated operand alias validation'
require_fixed_string_count "$ir_object_pattern_operation" 'require_distinct(&previous.binding, &key.binding)?;' 1 'rest excludes pairwise cell aliases'
require_fixed_string_count crates/lila-aot-wasm/src/operations.rs '    fn from_converted_value(value: ValueLocals) -> Self {' 1 'private normalized property-key mint'
require_regex_count crates/lila-aot-wasm/src/operations.rs '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+fn from_converted_value' 0 'no public unchecked property-key mint'
require_fixed_string_count crates/lila-aot-wasm/src/operations.rs "pub(crate) fn compile_object_destructuring_operation_keys<'operation>(" 1 'typed operation key projection owner'
require_exact_line_count crates/lila-aot-wasm/src/control_flow.rs 'mod object_destructuring;' 1 'private complete native object-destructuring attachment'
require_fixed_string_count "$wasm_object_pattern" 'pub(crate) fn compile_object_destructuring_operation_to_value(' 1 'sole incremental native operation entry'
require_fixed_string_count "$wasm_object_pattern" 'self.compile_object_destructuring_operation_keys(operation, function)?' 1 'native consumer uses the admitted key projection'
require_fixed_string_count crates/lila-aot-wasm/src/expressions.rs 'self.compile_object_destructuring_operation_to_value(operation, output, function)?' 1 'actual exhaustive expression dispatch'
require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs 'operation.visit_bindings(&mut |mode, name| {' 1 'direct lexical instantiation sees opaque pattern bindings'
ir_resumable_for_initializer="crates/lila-ir/src/lowering/resumable_for_initializer.rs"
require_file "$ir_resumable_for_initializer"
require_exact_line_count "$ir_lowering" 'mod resumable_for_initializer;' 1 'private actual common For-head initialization owner'
require_tree_regex_count crates/lila-ir/src/lowering '^[[:space:]]*pub\(super\) fn lower_resumable_generator_for_init\(' 1 'sole common For initializer algorithm'
require_fixed_string_count "$ir_resumable_for_initializer" 'lower_staged_generator_object_pattern_with_storage_names(' 1 'common For consumes the same staged Object pattern owner'
object_pattern_for_head="$(sed -n '/let storage_names =/,/if self.async_generator_entry_state()/p' "$ir_resumable_for_initializer")"
require_text_regex_count "$object_pattern_for_head" 'scoped_lexical_binding_storage_name\(' 1 'For patterns keep the analyzed scoped storage map'
for for_initializer_consumer in generator_loop async_classic_loop async_generator_loop; do
  require_fixed_string_count "crates/lila-ir/src/lowering/${for_initializer_consumer}.rs" 'self.lower_resumable_generator_for_init(source.init())' 1 'each checked protocol consumes the same actual For initializer'
done
pattern_assignment_yield_count="$(sed -n '/let source = GeneratorPatternAssignmentSource::new(/,/staged_generator_expression_yield_count(source.rhs())/p' crates/lila-ir/src/lowering_helpers.rs)"
require_text_regex_count "$pattern_assignment_yield_count" 'staged_generator_expression_yield_count\(source.rhs\(\)\)' 1 'retained pattern-assignment RHS getter has its actual eager count consumer'
check_raw_line_budget "$ir_object_pattern_source" 165
check_raw_line_budget "$ir_object_pattern_lowerer" 210
check_raw_line_budget "$ir_object_pattern_operation" 330
check_raw_line_budget "$wasm_object_pattern" 210
require_file vendor/boa_ast-0.21.1/src/expression/literal/object.rs
require_exact_line_count Cargo.toml 'boa_ast = { path = "vendor/boa_ast-0.21.1" }' 1 'workspace selects the actual corrected cover converter'
require_exact_line_count crates/lila-front/Cargo.toml 'boa_ast = { version = "0.21.1", features = ["annex-b"] }' 1 'frontend consumes the pinned patched AST'
require_fixed_string_count vendor/boa_ast-0.21.1/src/expression/literal/object.rs 'pub fn to_pattern(&self, strict: bool) -> Option<ObjectPattern>' 1 'actual vendored cover-conversion owner'
for object_pattern_control in \
  crates/lila-front/tests/object_assignment_cover_patterns.rs \
  crates/lila-ir/tests/generator_object_patterns.rs \
  crates/lila-ir/src/object_destructuring_operation/tests.rs \
  crates/lila-ir/tests/generator_pattern_assignments.rs \
  crates/lila-ir/tests/generator_pattern_initializers.rs \
  crates/lila-ir/tests/generator_throw_regions.rs \
  crates/lila-engine/tests/aot_generator_object_patterns.rs \
  crates/lila-engine/tests/fixtures/generator_object_patterns/object_patterns.js \
  crates/lila-engine/tests/fixtures/generator_object_patterns/with_references.js
do
  require_file "$object_pattern_control"
done
require_fixed_string_count crates/lila-front/tests/object_assignment_cover_patterns.rs 'fn computed_single_name_defaults_keep_inferred_and_explicit_definition_names()' 1 'actual cover conversion and NamedEvaluation control'
require_fixed_string_count crates/lila-ir/tests/generator_object_patterns.rs 'fn object_pattern_acquisition_reference_and_lazy_defaults_have_exact_source_order()' 1 'actual source ordering control'
require_fixed_string_count crates/lila-ir/tests/generator_object_patterns.rs 'fn object_pattern_member_put_keeps_raw_keys_and_lexical_for_cells_are_original()' 1 'raw member Reference and analyzed For cells control'
require_fixed_string_count crates/lila-ir/src/object_destructuring_operation/tests.rs 'fn key_factory_normalizes_once_and_rest_rejects_cell_aliases_without_rejecting_equal_keys()' 1 'meaningful private key/alias constructor control'

# Array-owned suspension retains a genuine native IteratorRecord in one exact
# original-invocation cell. Source kinds/states and whole suspension ranges are
# consumed by the opaque complete close owner, never by an expression opcode.
ir_array_pattern_source="crates/lila-ir/src/generator_value_branch_source/array_pattern.rs"
ir_array_pattern_lowerer="crates/lila-ir/src/lowering/generator_array_pattern.rs"
ir_array_pattern_operation="crates/lila-ir/src/array_destructuring_operation.rs"
ir_array_pattern_whole="crates/lila-ir/src/generator_array_destructuring.rs"
ir_async_array_pattern_whole="crates/lila-ir/src/async_array_destructuring.rs"
ir_mixed_array_pattern_whole="crates/lila-ir/src/async_generator_array_destructuring.rs"
ir_shared_pattern_target="crates/lila-ir/src/lowering/pattern_target.rs"
ir_shared_pattern_elements="crates/lila-ir/src/lowering/resumable_pattern.rs"
ir_shared_array_elements="crates/lila-ir/src/lowering/resumable_pattern/array.rs"
ir_shared_object_elements="crates/lila-ir/src/lowering/resumable_pattern/object.rs"
wasm_array_pattern="crates/lila-aot-wasm/src/control_flow/array_destructuring.rs"
wasm_resumable_array_pattern="crates/lila-aot-wasm/src/control_flow/resumable_array_destructuring.rs"
wasm_retained_array_iterator="crates/lila-aot-wasm/src/environments/retained_array_iterator.rs"
for array_pattern_owner in \
  "$ir_array_pattern_source" "$ir_array_pattern_lowerer" "$ir_pattern_target_lowerer" \
  "$ir_array_pattern_operation" "$ir_array_pattern_whole" \
  "$ir_async_array_pattern_whole" "$ir_mixed_array_pattern_whole" \
  "$ir_shared_pattern_target" "$ir_shared_pattern_elements" \
  "$ir_shared_array_elements" "$ir_shared_object_elements" \
  "$wasm_array_pattern" "$wasm_resumable_array_pattern" "$wasm_retained_array_iterator"
do
  require_file "$array_pattern_owner"
  check_no_inline_legacy_includes "$array_pattern_owner"
done
require_exact_line_count "$ir_generator_source" 'mod array_pattern;' 1 'private complete Array source owner'
require_exact_line_count "$ir_generator_source" 'pub(crate) use array_pattern::{GeneratorArrayPatternSource, GeneratorArrayPatternSourceStates};' 1 'actual source and operation tape projection'
require_regex_count "$ir_array_pattern_source" "^pub\(crate\) struct GeneratorArrayPatternSource<'ast>" 1 'sole actual ArrayPattern source carrier'
require_exact_line_count "$ir_array_pattern_source" '    pub(crate) fn operations(&self) -> &[(ArrayDestructuringOperationKindIr, u32)] {' 1 'actual ordered operation kind/state evidence'
require_fixed_string_count "$ir_generator_source" 'struct GeneratorArrayPatternSource' 0 'no parent Array source carrier copy'
for pattern_lowerer in generator_array_pattern generator_pattern_target pattern_target resumable_pattern; do
  require_exact_line_count "$ir_lowering" "mod ${pattern_lowerer};" 1 'private actual Array/shared target lowerer attachment'
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+${pattern_lowerer};" "$ir_lowering"; then
    fail "$ir_lowering must keep ${pattern_lowerer} private"
  fi
done
for array_pattern_entry in lower_staged_generator_array_pattern lower_staged_generator_array_pattern_with_storage_names; do
  require_fixed_string_count "$ir_array_pattern_lowerer" "pub(super) fn ${array_pattern_entry}(" 1 'actual staged Array entry'
  require_fixed_string_count "$ir_lowering" "fn ${array_pattern_entry}(" 0 'no parent Array lowering body'
done
require_fixed_string_present crates/lila-ir/src/lowering/generator_pattern_assignment.rs 'lower_staged_generator_array_pattern(' 'actual Array assignment producer'
require_fixed_string_present crates/lila-ir/src/lowering/generator_pattern_initializer.rs 'lower_staged_generator_array_pattern(' 'actual lexical and var Array producers'
require_fixed_string_count "$ir_resumable_for_initializer" 'lower_staged_generator_array_pattern_with_storage_names(' 1 'common For original scoped Array bindings'
for array_adapter in \
  'generator_array_pattern.rs:Generator:OrdinaryGeneratorArrayDestructuringIr' \
  'async_pattern/array.rs:Async:AsyncFunctionArrayDestructuringIr' \
  'async_generator_pattern.rs:AsyncGenerator:AsyncGeneratorArrayDestructuringIr'
do
  array_adapter_path="${array_adapter%%:*}"
  array_adapter_tail="${array_adapter#*:}"
  array_adapter_protocol="${array_adapter_tail%%:*}"
  array_adapter_carrier="${array_adapter_tail#*:}"
  array_adapter_path="crates/lila-ir/src/lowering/$array_adapter_path"
  require_fixed_string_present "$array_adapter_path" 'ArrayIteratorStorageIr::new(' 'checked original iterator storage'
  require_fixed_string_present "$array_adapter_path" "${array_adapter_carrier}::new(" 'actual protocol-specific complete carrier'
  require_fixed_string_present "$array_adapter_path" "PatternContinuation::${array_adapter_protocol}" 'closed source execution protocol'
  require_fixed_string_present "$array_adapter_path" 'self.lower_resumable_array_pattern_elements(' 'one physical Array element algorithm'
done
require_tree_regex_count crates/lila-ir/src/lowering '^[[:space:]]*pub\(in crate::lowering\) fn lower_resumable_array_pattern_elements\(' 1 'sole shared Array element algorithm'
require_exact_line_count "$ir_shared_pattern_elements" 'mod array;' 1 'private physical Array element attachment'
require_exact_line_count "$ir_shared_pattern_elements" 'mod object;' 1 'private physical Object element attachment'
for array_operation in step_value elision rest_array; do
  require_fixed_string_count "$ir_shared_array_elements" "ArrayDestructuringOperationIr::${array_operation}(" 1 'shared source-owned native Array protocol operation'
done
# Protocol adapters select defaults; Reference acquisition and Put have one
# physical body shared by both pattern shapes and all three protocols.
for pattern_target_method in retain_pattern_identifier retain_pattern_member put_pattern_target; do
  require_tree_regex_count crates/lila-ir/src/lowering "^[[:space:]]*pub\\(super\\) fn ${pattern_target_method}\\(" 1 'sole actual pattern Reference/Put owner'
  for pattern_consumer in "$ir_shared_object_elements" "$ir_shared_array_elements"; do
    require_fixed_string_present "$pattern_consumer" "self.${pattern_target_method}(" 'shared pattern Reference/Put algorithm'
  done
done
require_fixed_string_count "$ir_pattern_target_lowerer" '    fn lower_generator_pattern_default_region(' 1 'single lazy complete default region compiler'
for pattern_retention in owned_pattern_binding retain_pattern_value; do
  require_tree_regex_count crates/lila-ir/src/lowering "^pub\\(super\\) fn ${pattern_retention}\\(" 1 'sole whole Value activation retention owner'
done
require_fixed_string_count "$ir_shared_pattern_target" 'RetainedGeneratorIdentifierTarget::capture_write_only(' 1 'pattern Identifier retains original Reference before default'
require_fixed_string_present "$ir_shared_pattern_target" 'self.lower_pattern_expression(execution, access.target())?' 'Member/private original base evaluation'
require_fixed_string_count "$ir_shared_pattern_target" 'DestructuringPropertyKeyIr::Computed(raw)' 1 'Reference key remains raw until PutValue'
require_fixed_string_present "$ir_shared_pattern_target" 'StatementIr::DeclarationEvaluation(value)' 'lexical initialization Empty completion'
require_fixed_string_count "$ir_pattern_target_lowerer" 'OrdinaryGeneratorIfIr::new(' 1 'lazy defaults use complete actual branch regions'
require_fixed_string_present "$ir_shared_pattern_target" 'lower_staged_generator_object_pattern_with_storage_names(' 'nested Object actual checked owner'
require_fixed_string_present "$ir_shared_pattern_target" 'lower_staged_generator_array_pattern_with_storage_names(' 'nested Array distinct original record'
for array_carrier in array_destructuring_operation generator_array_destructuring; do
  require_exact_line_count crates/lila-ir/src/lib.rs "mod ${array_carrier};" 1 'private native Array proof attachment'
  require_exact_line_count "crates/lila-ir/src/${array_carrier}.rs" 'mod tests;' 1 'actual private damaged-allocation/source controls'
done
require_regex_count "$ir_array_pattern_operation" '^pub struct ArrayIteratorStorageIr \{' 1 'opaque real activation storage carrier'
require_exact_line_count "$ir_array_pattern_operation" 'pub struct ArrayDestructuringOperationIr(ArrayDestructuringOperation);' 1 'opaque closed Array operation'
require_exact_line_count "$ir_array_pattern_operation" 'enum ArrayDestructuringOperation {' 1 'private exhaustive Array operation payload'
require_exact_line_count "$ir_array_pattern_operation" 'pub enum ArrayDestructuringOperationKindIr {' 1 'closed source/native operation kind tape'
require_regex_count "$ir_array_pattern_whole" '^pub struct OrdinaryGeneratorArrayDestructuringIr \{' 1 'opaque complete acquisition/body/close owner'
for array_proof_owner in "$ir_array_pattern_operation" "$ir_array_pattern_whole" "$ir_async_array_pattern_whole" "$ir_mixed_array_pattern_whole"; do
  require_regex_count "$array_proof_owner" '^[[:space:]]+pub[[:space:]]+(binding|storage|raw_source|body|result)[[:space:]]*:' 0 'private constructor-owned Array fields'
  require_regex_count "$array_proof_owner" '^[[:space:]]+pub[[:space:]]+fn[[:space:]]+(new|step_value|elision|rest_array)[[:space:]]*[(]' 0 'no unchecked public native Array constructor'
done
require_fixed_string_count "$ir_array_pattern_operation" 'row.name == binding.name || row.slot == binding.slot' 1 'exact original storage allocation name/slot alias proof'
require_fixed_string_count "$ir_async_array_pattern_whole" 'row.name == binding.name || row.slot == binding.slot' 1 'shared complete pattern-cell allocation proof'
require_fixed_string_present "$ir_mixed_array_pattern_whole" 'PatternCells::new(inventory)' 'shared global pattern-cell alias census'
require_fixed_string_count "$ir_array_pattern_operation" 'StatementIr::ArrayDestructuringOperation(' 2 'operation factory emits only dedicated statements'
require_fixed_string_count "$ir_array_pattern_whole" 'if operations != states.operations()' 1 'complete native body consumes actual operation order/states'
require_fixed_string_count "$ir_array_pattern_whole" 'if points != states.suspensions()' 1 'whole source suspension tape is consumed exactly'
require_fixed_string_count "$ir_array_pattern_whole" 'state = sequence_end(' 1 'native operation positions use actual complete region progression'
require_fixed_string_count "$ir_array_pattern_whole" 'require_allocated(result, inventory)?;' 1 'complete body revalidates actual result cells'
require_fixed_string_count "$ir_array_pattern_whole" 'require_allocated(&plan.raw_binding, inventory)?;' 1 'nested raw source is in the same allocation inventory'
for array_proof_owner in "$ir_async_array_pattern_whole" "$ir_mixed_array_pattern_whole"; do
  require_fixed_string_present "$array_proof_owner" 'operations != states.operations()' 'actual ordered source operation tape'
  require_fixed_string_present "$array_proof_owner" 'operation.storage() != storage' 'foreign iterator-operation refusal'
done
require_fixed_string_count "$ir_async_array_pattern_whole" 'awaits != states.awaits()' 1 'plain Async exact source Await tape'
require_fixed_string_present "$ir_mixed_array_pattern_whole" 'validate_tape(states.suspensions(), &suspensions)' 'mixed exact source Await/Yield tape'
require_fixed_string_present "$ir_mixed_array_pattern_whole" 'AsyncGeneratorLoopRegionIr::from_array_pattern(CheckedArrayBody {' 'body region minted only by actual Array proof'
require_fixed_string_count crates/lila-ir/src/ir.rs 'ArrayDestructuringOperation(Box<crate::ArrayDestructuringOperationIr>),' 1 'sole dedicated Statement operation variant'
array_operation_expression_domain="$(braced_rust_item_source crates/lila-ir/src/ir.rs '^pub[[:space:]]+enum[[:space:]]+ExprIr[[:space:]]*[{]')"
require_text_regex_count "$array_operation_expression_domain" 'ArrayDestructuringOperation' 0 'native iterator operations cannot inhabit expressions'
for array_native_child in array_destructuring resumable_array_destructuring; do
  require_exact_line_count crates/lila-aot-wasm/src/control_flow.rs "mod ${array_native_child};" 1 'private actual native Array semantic child'
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+${array_native_child};" crates/lila-aot-wasm/src/control_flow.rs; then
    fail 'native Array semantic children must remain private'
  fi
done
require_fixed_string_count "$wasm_array_pattern" 'pub(super) fn compile_array_destructuring_operation_statement(' 1 'sole native statement-only operation entry'
for array_native_entry in compile_ordinary_generator_array_destructuring compile_async_function_array_destructuring compile_async_generator_array_destructuring; do
  require_fixed_string_count "$wasm_resumable_array_pattern" "pub(super) fn ${array_native_entry}(" 1 'actual checked protocol adapter'
  require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs "self.${array_native_entry}(plan, function)?;" 1 'actual closed native carrier dispatch'
done
require_tree_regex_count crates/lila-aot-wasm/src '^[[:space:]]*fn compile_resumable_array_destructuring\(' 1 'sole physical retained acquisition/body/close pipeline'
require_fixed_string_present "$wasm_resumable_array_pattern" 'meta.protocol().execution_kind() == plan.execution_kind()' 'exact native carrier execution admission'
require_fixed_string_present "$wasm_resumable_array_pattern" 'CheckedAsyncGeneratorEnvironmentOwner::for_array_destructuring(plan)' 'mixed environment token minted from actual opaque carrier'
for moved_array_entry in compile_array_destructure_to_locals compile_array_destructure_from_value_locals; do
  require_fixed_string_count "$wasm_array_pattern" "pub(crate) fn ${moved_array_entry}(" 1 'same ordinary complete Array compiler in actual child'
  require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs "fn ${moved_array_entry}(" 0 'no complete Array compiler copy in parent'
done
require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs 'self.compile_array_destructuring_operation_statement(operation, function)?;' 1 'actual closed native operation dispatch'
require_fixed_string_count "$wasm_array_pattern" 'self.emit_array_destructuring_rest_array(' 2 'ordinary and retained rest share one physical builder'
require_fixed_string_count "$wasm_array_pattern" '    fn emit_array_destructuring_rest_array(' 1 'sole real Array rest drain body'
require_exact_line_count crates/lila-aot-wasm/src/environments.rs 'mod retained_array_iterator;' 1 'private real activation edge owner'
require_fixed_string_count "$wasm_retained_array_iterator" 'pub(crate) fn array_iterator_storage(' 1 'native storage validates actual ordinary invocation owner'
require_fixed_string_count "$wasm_retained_array_iterator" 'InvocationFrameSchema::INVOCATION_ENVIRONMENT' 1 'record edge stays in original invocation cell'
require_fixed_string_count "$wasm_retained_array_iterator" 'self.owned_env_slot(&binding.name) != Some(binding.slot)' 1 'exact native owned storage slot proof'
require_fixed_string_present "$wasm_retained_array_iterator" 'owner.is_region()' 'mixed admission requires checked environment owner'
for iterator_edge_entry in emit_publish_retained_array_iterator emit_load_retained_array_iterator emit_retire_retained_array_iterator; do
  require_fixed_string_count "$wasm_retained_array_iterator" "pub(crate) fn ${iterator_edge_entry}(" 1 'actual native record edge lifecycle owner'
done
require_fixed_string_count "$wasm_resumable_array_pattern" 'self.emit_publish_retained_array_iterator(&storage, acquired.record(), function);' 1 'acquisition retains same genuine native record'
require_fixed_string_count "$wasm_resumable_array_pattern" 'self.emit_retire_retained_array_iterator(&storage, function);' 1 'whole close alone retires record edge'
require_fixed_string_count "$wasm_resumable_array_pattern" 'self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?;' 1 'shared original whole-completion close algorithm'
require_fixed_string_count "$wasm_array_pattern" 'self.write_generator_statement_list_binding(binding, &value, function);' 1 'protocol result writes the original invocation cell'
require_fixed_string_count crates/lila-aot-wasm/src/gc_types/layouts.rs 'DESTRUCTURING_ITERATOR_RECORD: GcRef<IteratorRecord>, Mutable, Nullable;' 1 'single actual nullable GC record edge'
check_raw_line_budget "$ir_array_pattern_source" 210
check_raw_line_budget "$ir_array_pattern_lowerer" 225
check_raw_line_budget "$ir_pattern_target_lowerer" 330
check_raw_line_budget "$ir_shared_pattern_target" 330
check_raw_line_budget "$ir_array_pattern_operation" 205
check_raw_line_budget "$ir_array_pattern_whole" 310
check_raw_line_budget "$wasm_array_pattern" 200
# The three small adapters are checked above against one physical pipeline;
# the old Generator-only raw-line budget cannot describe that shared owner.
check_raw_line_budget "$wasm_retained_array_iterator" 125
for array_pattern_control in \
  crates/lila-ir/src/array_destructuring_operation/tests.rs \
  crates/lila-ir/src/generator_array_destructuring/tests.rs \
  crates/lila-ir/tests/generator_array_patterns.rs \
  crates/lila-engine/tests/aot_generator_array_patterns.rs \
  crates/lila-engine/tests/fixtures/generator_array_patterns/array_patterns.js \
  crates/lila-engine/tests/fixtures/generator_array_patterns/with_references.js \
  crates/lila-aot-wasm/tests/generator_array_destructuring_structure.rs \
  crates/lila-aot-wasm/tests/destructuring_iterator_step_kind_structure.rs \
  crates/lila-aot-wasm/tests/destructuring_iterator_locals_ownership_structure.rs \
  crates/lila-aot-wasm/tests/sync_iterator_consumer_capability_structure.rs
do
  require_file "$array_pattern_control"
done
require_fixed_string_count crates/lila-ir/src/generator_array_destructuring/tests.rs 'fn complete_array_constructor_rejects_removed_extra_and_substituted_protocol_operations()' 1 'actual complete protocol tape damage control'
require_fixed_string_count crates/lila-ir/tests/generator_array_patterns.rs 'fn array_pattern_owns_acquisition_complete_body_and_lazy_default_states()' 1 'real source complete-close/range control'
require_fixed_string_count crates/lila-engine/tests/aot_generator_array_patterns.rs 'fn generator_array_patterns_preserve_iterator_records_close_and_original_targets()' 1 'actual strict/sloppy Wasm protocol/completion cohort'

# Parser name-scope provenance separates a source BindingIdentifier from an
# inferred display label. Analysis, lowering and fact publication share it.
ir_source_class_name="crates/lila-ir/src/class_name_source.rs"
require_file "$ir_source_class_name"
require_exact_line_count crates/lila-ir/src/lib.rs 'mod class_name_source;' 1 'private class-name source authority'
require_exact_line_count crates/lila-ir/src/lib.rs 'pub(crate) use class_name_source::SourceClassName;' 1 'consumed source class-name proof'
require_exact_line_count "$ir_source_class_name" 'pub(crate) struct SourceClassName(ClassNameSource);' 1 'opaque source binding versus label owner'
require_exact_line_count "$ir_source_class_name" 'enum ClassNameSource {' 1 'closed private class-name provenance'
require_fixed_string_count "$ir_source_class_name" 'class.name_scope().has_binding(' 1 'declaration consumes the actual parser name scope'
require_fixed_string_count "$ir_source_class_name" 'class.name_scope().is_some()' 1 'expression consumes source BindingIdentifier provenance'
for class_name_consumer in crates/lila-ir/src/analysis.rs "$ir_lowering"; do
  require_fixed_string_count "$class_name_consumer" 'SourceClassName::from_declaration(' 1 'source declaration name proof consumer'
  require_fixed_string_count "$class_name_consumer" 'SourceClassName::from_expression(' 1 'source expression name proof consumer'
done
require_fixed_string_count crates/lila-ir/src/analysis.rs 'class_name.binding_name().map(' 1 'only real class names allocate analysis environments'
require_fixed_string_count "$ir_lowering" 'class_name.binding_name().map(' 1 'only real class names allocate lowering environments'
require_fixed_string_count "$ir_lowering" 'class_name.into_label(),' 1 'metadata consumes the separate display label'
require_fixed_string_count crates/lila-ir/src/lowering/class_definition.rs 'if let (Some(class_name), Some(_)) = (&class_name, &name_binding)' 2 'class fact updates require an actual name binding'
check_no_inline_legacy_includes "$ir_source_class_name"
check_raw_line_budget "$ir_source_class_name" 85
require_file crates/lila-ir/tests/class_name_source.rs
require_file crates/lila-engine/tests/aot_class_name_source.rs
require_file crates/lila-engine/tests/fixtures/class_name_source/names.js
require_fixed_string_count crates/lila-ir/tests/class_name_source.rs 'fn inferred_class_labels_capture_outer_cells_and_explicit_names_capture_inner_cells()' 1 'actual inferred-label versus source-binding capture control'
require_fixed_string_count crates/lila-ir/tests/class_name_source.rs 'fn yielded_heritage_retains_only_actual_source_class_name_environments()' 1 'actual suspended class name environment control'
require_fixed_string_count crates/lila-ir/tests/class_name_source.rs 'fn anonymous_default_export_preserves_its_display_label_without_a_class_name_cell()' 1 'metadata-only class label control'
require_fixed_string_count crates/lila-engine/tests/aot_class_name_source.rs 'fn class_labels_preserve_outer_tdz_cells_and_explicit_inner_names_through_wasm()' 1 'actual defining class/outer TDZ Wasm cohort'
require_file crates/lila-ir/tests/class_computed_name_suspension.rs
require_file crates/lila-engine/tests/aot_class_computed_name_suspension.rs

ir_optional_source="crates/lila-ir/src/generator_value_branch_source/optional_chain.rs"
ir_optional_consumer="crates/lila-ir/src/lowering/generator_value_branch/optional_chain.rs"
require_file "$ir_optional_source"
require_file "$ir_optional_consumer"
require_exact_line_count "$ir_generator_source" 'mod optional_chain;' 1 'private complete optional source attachment'
require_exact_line_count crates/lila-ir/src/lowering/generator_value_branch.rs 'mod optional_chain;' 1 'private complete optional consumer attachment'
require_regex_count "$ir_optional_source" "^pub\(crate\) struct GeneratorOptionalChainSource<'ast>" 1 'actual complete optional source carrier'
require_fixed_string_count "$ir_generator_source" 'struct GeneratorOptionalChainSource' 0 'no parent optional source carrier copy'
for optional_region_entry in lower_generator_optional_chain lower_generator_grouped_optional_reference lower_generator_optional_delete; do
  require_fixed_string_count "$ir_optional_consumer" "pub(in crate::lowering) fn ${optional_region_entry}(" 1 'actual checked optional region entry'
  require_fixed_string_count crates/lila-ir/src/lowering/generator_value_branch.rs "fn ${optional_region_entry}(" 0 'no parent complete optional compiler copy'
done
require_fixed_string_count "$ir_optional_consumer" 'fn lower_generator_optional_chain_to(' 1 'one shared optional region destination body'
require_fixed_string_count "$ir_lowering" 'return self.lower_generator_optional_chain(source);' 1 'actual complete optional value dispatch'
ir_eager_reference_operands="crates/lila-ir/src/lowering/generator_eager_value/reference_operands.rs"
require_file "$ir_eager_reference_operands"
require_exact_line_count crates/lila-ir/src/lowering/generator_eager_value.rs 'mod reference_operands;' 1 'private shared eager Reference operand attachment'
require_fixed_string_count "$ir_eager_reference_operands" 'self.lower_generator_optional_delete(source)' 1 'actual complete optional Delete dispatch in the private Reference operand owner'
check_no_inline_legacy_includes "$ir_eager_reference_operands"
# Update/Delete/Super References keep original operand order under the shared
# protocol adapter; arithmetic/Calls remain in the bounded eager parent.
check_raw_line_budget "$ir_eager_reference_operands" 270
check_no_inline_legacy_includes "$ir_optional_source"
check_no_inline_legacy_includes "$ir_optional_consumer"
# These bounded owners now stage ordinary, async and mixed optional operands,
# including private references, grouped Calls and Delete without a second algorithm.
check_raw_line_budget "$ir_optional_source" 500
check_raw_line_budget "$ir_optional_consumer" 650
# Signature propagation and declaration bodies are genuine private owners. The
# parent retains its exhaustive declaration dispatcher and shared facts. Var
# and lexical owners now initialize mapped pattern cells across all protocols
# and preserve declaration Empty completion.
for declaration_owner_entry in \
  'signature_evidence:propagate_function_signatures:1050' \
  'var_declaration:lower_var_declarator:570' \
  'function_declaration:lower_function_declaration:225' \
  'lexical_declaration:lower_lexical_declaration:460'
do
  declaration_owner="${declaration_owner_entry%%:*}"
  declaration_entry_budget="${declaration_owner_entry#*:}"
  declaration_entry="${declaration_entry_budget%%:*}"
  declaration_budget="${declaration_entry_budget#*:}"
  declaration_owner_path="crates/lila-ir/src/lowering/${declaration_owner}.rs"
  require_file "$declaration_owner_path"
  require_exact_line_count "$ir_lowering" "mod ${declaration_owner};" 1 'private signature/declaration owner attachment'
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${declaration_owner};" "$ir_lowering"; then
    fail "$ir_lowering must keep ${declaration_owner} private"
  fi
  require_fixed_string_count "$declaration_owner_path" "pub(super) fn ${declaration_entry}(" 1 'consumed signature/declaration entry'
  require_fixed_string_count "$ir_lowering" "fn ${declaration_entry}(" 0 'no parent signature/declaration body copy'
  check_no_inline_legacy_includes "$declaration_owner_path"
  check_raw_line_budget "$declaration_owner_path" "$declaration_budget"
done
require_fixed_string_count "$ir_lowering" 'self.propagate_function_signatures();' 1 'actual signature propagation consumer'
# T12's module subsystem. `modules/` is a directory module, so the flat-file
# loop above cannot cover it: declaring `mod modules;` without the directory,
# or adding a submodule without registering it, is exactly the failure this
# catches.
ir_modules_mod="crates/lila-ir/src/modules/mod.rs"
require_file "$ir_modules_mod"
require_module_decl "$ir_lib" "modules"
for module in callable_source dynamic early graph link namespace record source; do
  require_file "crates/lila-ir/src/modules/${module}.rs"
  require_module_decl "$ir_modules_mod" "$module"
done
check_no_inline_legacy_includes "$ir_modules_mod"
check_raw_line_budget "crates/lila-ir/src/modules/callable_source.rs" 340
require_file "crates/lila-ir/src/lowering/function_environment.rs"
require_module_decl "$ir_lowering" "function_environment"
check_raw_line_budget "crates/lila-ir/src/lowering/function_environment.rs" 140
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
# The intrinsic-method gate owns the one name -> prototype-builtin catalogue and
# the only construction of its proof token, so a static method resolution that
# wants the exact builtin cannot skip the live-prototype proof.
ir_intrinsic_method_lowering="crates/lila-ir/src/lowering/intrinsic_method.rs"
require_file "$ir_intrinsic_method_lowering"
require_module_decl "$ir_lowering" "intrinsic_method"
require_fixed_string_count \
  "$ir_intrinsic_method_lowering" \
  'pub(super) fn catalogued_method(' \
  1 \
  'intrinsic-method catalogue owner'
require_fixed_string_count \
  "$ir_intrinsic_method_lowering" \
  'IntrinsicMethod { builtin }' \
  1 \
  'intrinsic-method proof construction'
check_no_inline_legacy_includes "$ir_intrinsic_method_lowering"
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
# The assignment owner also stages checked property/private/Super destinations
# across all three resumable protocols, retaining the original Reference.
check_raw_line_budget "$ir_assignment_lowering" 820
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
# T02's statement boundary owns the exhaustive Statement dispatcher. Its
# expression child owns resumable expression-statement staging; control-flow
# implementations remain in their focused owners.
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
ir_statement_expression="crates/lila-ir/src/lowering/statement/expression.rs"
require_file "$ir_statement_expression"
require_exact_line_count "$ir_statement_lowering" 'mod expression;' 1 'private expression-statement module'
require_fixed_string_count "$ir_statement_lowering" 'Statement::Expression(expression) => self.lower_expression_statement(expression),' 1 'expression-statement dispatch consumption'
require_regex_count "$ir_statement_expression" '^[[:space:]]*pub\(in crate::lowering\)[[:space:]]+fn[[:space:]]+lower_expression_statement\(' 1 'expression-statement staging owner shared only inside lowering'
require_fixed_string_count "$ir_statement_lowering" 'fn lower_expression_statement(' 0 'expression-statement body outside child'
check_no_inline_legacy_includes "$ir_statement_lowering"
check_no_inline_legacy_includes "$ir_statement_expression"
# The dispatcher selects complete protocol/resource owners; its expression
# child stages their operands and preserves source Empty completion.
check_raw_line_budget "$ir_statement_lowering" 185
check_raw_line_budget "$ir_statement_expression" 350
# T02's for-in boundary owns the complete initializer/target/body lowering
# family. Retired source-shape recognizers must stay absent. The
# eager wrapper and common initializer utilities cross only the lowering-private
# boundary; the typed head child shares the original TDZ/target preparation.
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
  widen_for_in_parameter_target \
  lower_for_in_initializer_prefix \
  prepend_statement
do
  require_regex_count \
    "$ir_for_in_lowering" \
    "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+${private_owner}[[:space:]]*[<(]" \
    1 \
    "private ${private_owner} owner"
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${private_owner}[[:space:]]*[<(]" \
    0 \
    "${private_owner} outside child module"
done
ir_for_in_head="crates/lila-ir/src/lowering/for_in/head.rs"
require_file "$ir_for_in_head"
require_exact_line_count "$ir_for_in_lowering" 'mod head;' 1 'private original ForIn head owner'
require_exact_line_count "$ir_for_in_lowering" 'pub(super) use head::ForInIterationHead;' 1 'lowering-private checked head projection'
require_fixed_string_count "$ir_for_in_head" 'pub(in crate::lowering) struct ForInIterationHead {' 1 'opaque original head shared by eager and complete protocols'
require_fixed_string_count "$ir_for_in_head" 'pub(in crate::lowering) fn prepare_for_in_iteration_head(' 1 'sole original ForIn head preparation'
for for_in_head_parent in "$ir_lowering" "$ir_for_in_lowering"; do
  require_regex_count "$for_in_head_parent" '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+prepare_for_in_iteration_head[[:space:]]*\(' 0 'no original ForIn head preparation copy outside the private head child'
done
# Target/reference selection and mapped pattern initialization for all protocols
# share the original head/TDZ preparation instead of copying it into each loop.
check_no_inline_legacy_includes "$ir_for_in_head"
check_raw_line_budget "$ir_for_in_head" 450
require_tree_regex_count crates/lila-ir/src/lowering '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+for_in_initializer_binding[[:space:]]*[<(]' 0 'retired untyped ForIn initializer helper'
for retired_for_in_recognizer in \
  for_in_known_empty_target \
  for_in_global_non_enumerable_guard_only \
  for_in_builtin_non_enumerable_assert_only \
  for_in_static_builtin_target \
  for_in_initializer_name \
  for_in_non_enumerable_guarded_assignment \
  for_in_non_enumerable_guard_name \
  for_in_not_same_value_guard_name \
  statement_is_simple_false_assignment \
  for_in_global_target \
  single_statement \
  expr_is_identifier_named \
  is_known_non_enumerable_global \
  is_known_non_enumerable_builtin_property
do
  require_tree_regex_count \
    crates/lila-ir/src \
    "^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?fn[[:space:]]+${retired_for_in_recognizer}[[:space:]]*[<(]" \
    0 \
    "retired for-in source-shape recognizer ${retired_for_in_recognizer}"
done
require_fixed_string_count "$ir_for_in_lowering" 'self.lower_for_in_head_target(&head, for_in.target(),' 1 'for-in evaluates the original target through the shared head owner'
require_fixed_string_count crates/lila-aot-wasm/src/control_flow/for_in.rs 'self.emit_create_for_in_enumerator(&source, function)?' 1 'for-in consumes actual runtime property enumeration'
# Four lowering-private methods and the opaque typed head reexport are the
# complete facade; the head algorithm itself remains in its private child.
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*fn[[:space:]]+' \
  0 \
  'private method'
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*((pub(\([^)]*\))?|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+[[:alnum:]_]+' \
  4 \
  'total function declaration'
require_regex_count \
  "$ir_for_in_lowering" \
  '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' \
  5 \
  'Rust-visible item'
for shared_parent_owner in \
  lower_for_in_of_environment \
  lower_for_head_expression_with_tdz \
  static_string_expression \
  plain_async_entry_state \
  lower_loop_body \
  lower_web_compat_loop_assignment_target \
  unwrap_parenthesized_expr
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
require_fixed_string_count 'crates/lila-ir/src/lowering/var_declaration.rs' 'pub(super) fn lower_var_declarator(' 1 'shared var declarator authority'
require_fixed_string_count "$ir_for_in_lowering" 'self.lower_var_declarator(variable)?' 1 'for-in consumes shared var declaration owner'
require_regex_count "$ir_for_in_lowering" '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+lower_var_declarator[[:space:]]*[<(]' 0 'no var declarator body copied into for-in'
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
# The source-proof admission wrapper adds one line beyond the former margin.
# Classic For now selects complete resource-head and resumable phase owners.
check_raw_line_budget "$ir_for_loop_lowering" 310
require_module_decl "$ir_lowering" "synchronous_resource_loop"
require_file "crates/lila-ir/src/lowering/synchronous_resource_loop.rs"
check_raw_line_budget "crates/lila-ir/src/lowering/synchronous_resource_loop.rs" 155
# The shared current-activation visitor stays in a private child; the parent
# retains only proven-loop lowering and its three source-admission wrappers.
require_module_decl "crates/lila-ir/src/lowering/synchronous_resource_loop.rs" "source_suspension"
require_file "crates/lila-ir/src/lowering/synchronous_resource_loop/source_suspension.rs"
check_no_inline_legacy_includes "crates/lila-ir/src/lowering/synchronous_resource_loop/source_suspension.rs"
check_raw_line_budget "crates/lila-ir/src/lowering/synchronous_resource_loop/source_suspension.rs" 180
require_file "crates/lila-ir/src/synchronous_loop_body.rs"
# The eager-body predicate also rejects complete child protocol phases.
check_raw_line_budget "crates/lila-ir/src/synchronous_loop_body.rs" 210
# T02's for-of boundary owns every specialization decision and the
# lowering-only protocol carrier. The statement dispatcher is the sole caller;
# shared loop/environment helpers and public statement/protocol IR remain in
# their existing owners.
ir_for_of_lowering="crates/lila-ir/src/lowering/for_of.rs"
ir_for_of_protocol_lowering="crates/lila-ir/src/lowering/for_of/protocol.rs"
ir_for_of_async_lowering="crates/lila-ir/src/lowering/for_of/async_function.rs"
ir_async_for_of_iterator="crates/lila-ir/src/ir/async_for_of_iterator.rs"
require_file "$ir_for_of_lowering"
require_file "$ir_for_of_protocol_lowering"
require_file "$ir_for_of_async_lowering"
require_file "$ir_async_for_of_iterator"
require_exact_line_count "$ir_for_of_lowering" 'mod async_function;' 1 'private plain-async for-of producer child'
require_exact_line_count "crates/lila-ir/src/ir.rs" 'mod async_for_of_iterator;' 1 'private completed plain-async iterator owner'
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
for private_owner in lower_for_of_head; do
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
for private_owner in lower_async_function_for_of_iterator_with_body_await lower_plain_async_for_await_with_body_await; do
  require_fixed_string_count "$ir_for_of_async_lowering" "fn ${private_owner}(" 1 "private ${private_owner} producer"
  require_fixed_string_count "$ir_for_of_lowering" "fn ${private_owner}(" 0 "${private_owner} definition outside producer leaf"
  require_fixed_string_count "$ir_lowering" "fn ${private_owner}(" 0 "${private_owner} definition in lowering parent"
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
  "$ir_async_for_of_iterator" \
  'pub struct AsyncFunctionForOfIteratorPlanIr' \
  1 \
  'completed plain-async for-of plan owner'
require_fixed_string_count "crates/lila-ir/src/ir.rs" 'pub struct AsyncFunctionForOfIteratorPlanIr' 0 'completed plan definition in IR parent'
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
# The statement-facing wrapper and the proven resource-region entry are visible
# to the private lowering parent. The protocol carrier stays in its own child.
require_regex_count \
  "$ir_for_of_lowering" \
  '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' \
  2 \
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
check_no_inline_legacy_includes "$ir_for_of_async_lowering"
check_no_inline_legacy_includes "$ir_async_for_of_iterator"
for private_owner in "$ir_for_of_async_lowering" "$ir_async_for_of_iterator"; do
  if grep -Eq '^pub(\([^)]*\))? mod ' "$private_owner"; then
    fail "$private_owner must keep implementation children private"
  fi
done
# Measured after separating the protocol witness carrier: 731 raw lines. The
# margin is for maintenance of the complete for-of lowering family, not
# unrelated lowering.
# Measured canonical ForOf head/body owner: 762 raw lines. The neutral
# resumable local-control ownership import added one line to the 761-line owner.
# Bare identifier assignment adds five actual dispatch lines to pass the already
# lowered head classification into its consuming generator proof constructor.
# Ordinary-property assignment adds 22 measured dispatch lines to consume the
# same fused Reference completion as plain assignment and the checked head plan.
# Eager generator lexical patterns add the actual semantic initializer and
# checked lexical-head dispatch. The complete current owner measures 876 lines.
check_raw_line_budget "$ir_for_of_lowering" 876
# The consumed producer measures 188 lines and the completed plan 365.
# Keep their maintenance margins local to these actual owners.
check_raw_line_budget "$ir_for_of_async_lowering" 210
check_raw_line_budget "$ir_async_for_of_iterator" 390
# The shared protocol carrier measures 129 lines after admitting the completed
# plain-async plan; its constructors retain one protocol authority.
check_raw_line_budget "$ir_for_of_protocol_lowering" 140
# T15's assignment head proof is a real private child consumed by the existing
# lowering and iterator-plan constructors. Reuse the sole summary visitor for
# entry-local escape checks rather than adding a second opcode walker.
ir_generator_for_of_plan="crates/lila-ir/src/generator_for_of_iterator.rs"
ir_generator_for_of_head="crates/lila-ir/src/generator_for_of_iterator/head.rs"
ir_resumable_sync_for_of_binding="crates/lila-ir/src/resumable_sync_for_of_binding.rs"
require_file "$ir_generator_for_of_head"
require_file "$ir_resumable_sync_for_of_binding"
require_exact_line_count "$ir_generator_for_of_plan" 'mod head;' 1 'private generator head proof module'
if grep -Eq '^pub(\([^)]*\))? mod head;' "$ir_generator_for_of_plan"; then
  fail "$ir_generator_for_of_plan must keep its head proof child private"
fi
check_no_inline_legacy_includes "$ir_generator_for_of_head"
require_fixed_string_count \
  'crates/lila-ir/src/lowering/for_of/generator.rs' \
  'GeneratorForOfAssignmentIr::identifier(' \
  1 \
  'actual generator identifier-head proof producer'
require_fixed_string_count \
  'crates/lila-ir/src/lowering/for_of/generator.rs' \
  'GeneratorForOfAssignmentIr::ordinary_property(' \
  1 \
  'actual generator ordinary-property head proof producer'
require_fixed_string_count \
  "$ir_resumable_sync_for_of_binding" \
  'pub(crate) struct ValidatedResumableSyncForOfLexicalPatternIr' \
  1 \
  'sole checked shared lexical-pattern head'
for lexical_pattern_consumer in "$ir_for_of_lowering"; do
  require_fixed_string_count \
    "$lexical_pattern_consumer" \
    'ValidatedResumableSyncForOfLexicalPatternIr::new(' \
    1 \
    'lowering-only shared lexical-pattern proof constructor'
done
# Async IR retains its own complete initialization validator; generator heads
# consume the same lowering-produced lexical-pattern proof without rebuilding it.
require_fixed_string_count "$ir_generator_for_of_head" 'pattern: ValidatedResumableSyncForOfLexicalPatternIr,' 2 'generator borrowed/owned lexical-pattern proof input'
require_fixed_string_count \
  'crates/lila-ir/src/lowering/for_of/generator.rs' \
  'GeneratorForOfLexicalPatternIr::new(' \
  1 \
  'actual generator initializer-prefix proof consumer'
require_fixed_string_count \
  "$ir_for_of_lowering" \
  'self.complete_ordinary_property_plain_assignment(' \
  1 \
  'consumed ordinary-property Reference completion in generator head'
require_fixed_string_count \
  'crates/lila-ir/src/ir.rs' \
  'pub(crate) fn statements_reference_storage(' \
  1 \
  'sole exhaustive storage-query owner'
require_fixed_string_count \
  "$ir_generator_for_of_head" \
  'statements_reference_storage(rest, &self.value_name)' \
  1 \
  'consumed assignment-head body escape check'
# Existing consumed head/shared-proof files measured after the complete lexical
# initializer and lifetime proof join; no alternate opcode visitor is added.
check_raw_line_budget "$ir_generator_for_of_head" 413
check_raw_line_budget "$ir_resumable_sync_for_of_binding" 459

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
# Measured with typed plain-async branch ranges: 190 raw lines. The margin is
# for if-statement lowering only; plan invariants remain in async_if.rs.
# Complete plain-async branches share the checked region path.
check_raw_line_budget "$ir_if_statement_lowering" 240
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

# T14's awaited while condition has its own checked IR/lowering/emission owners.
# Ordinary while and do-while remain within their existing 130-line boundary.
require_file crates/lila-ir/src/async_while.rs
require_module_decl "$ir_lib" async_while
require_exact_line_count "$ir_lib" 'pub use async_while::AsyncFunctionWhileConditionIr;' 1 'checked awaited while condition re-export'
require_file crates/lila-ir/src/lowering/awaited_while_condition.rs
require_module_decl "$ir_lowering" awaited_while_condition
require_file crates/lila-ir/src/lowering/eager_async_while_body.rs
require_module_decl "$ir_lowering" eager_async_while_body
require_fixed_string_count crates/lila-ir/src/lowering/awaited_while_condition.rs 'EagerAsyncWhileBody::new(while_loop.body())' 1 'checked eager async while body admission'
require_fixed_string_count crates/lila-ir/src/lowering/awaited_while_condition.rs 'let (body, kind) = body.lower(self);' 1 'checked eager async while body consumption'
require_fixed_string_count crates/lila-ir/src/lowering/awaited_while_condition.rs 'current_async_resume_state = None' 0 'unchecked continuation suppression in awaited while owner'
check_no_inline_legacy_includes crates/lila-ir/src/lowering/eager_async_while_body.rs
check_raw_line_budget crates/lila-ir/src/lowering/eager_async_while_body.rs 45
require_fixed_string_count crates/lila-ir/src/lowering/awaited_while_condition.rs 'pub(super) fn lower_awaited_while_condition(' 1 'awaited while condition lowering owner'
require_fixed_string_count "$ir_while_loop_lowering" 'self.lower_awaited_while_condition(while_loop, entry_state)' 1 'awaited while condition production admission'
require_fixed_string_count "$ir_lowering" 'fn lower_awaited_while_condition(' 0 'awaited while condition implementation copied into parent'
require_file crates/lila-aot-wasm/src/control_flow/async_function_while.rs
require_module_decl crates/lila-aot-wasm/src/control_flow.rs async_function_while
require_fixed_string_count crates/lila-aot-wasm/src/control_flow/async_function_while.rs 'pub(super) fn compile_async_function_while_condition(' 1 'awaited while condition emitter owner'
for awaited_while_owner in crates/lila-ir/src/async_while.rs crates/lila-ir/src/lowering/awaited_while_condition.rs crates/lila-aot-wasm/src/control_flow/async_function_while.rs; do
  check_no_inline_legacy_includes "$awaited_while_owner"
done
ir_async_while_prefix="crates/lila-ir/src/async_while/condition_prefix.rs"
ir_async_while_tests="crates/lila-ir/src/async_while/tests.rs"
require_file "$ir_async_while_prefix"
require_file "$ir_async_while_tests"
require_exact_line_count crates/lila-ir/src/async_while.rs 'mod condition_prefix;' 1 'private awaited-while prefix validator module'
require_exact_line_count crates/lila-ir/src/async_while.rs 'mod tests;' 1 'private awaited-while unit-control module'
require_fixed_string_count crates/lila-ir/src/async_while.rs 'condition_prefix::validate_condition_prefix(&condition_prefix, entry_state, ready_state)?;' 1 'mandatory awaited-while constructor validation'
require_regex_count "$ir_async_while_prefix" '^pub\(super\) fn validate_condition_prefix\(' 1 'restartable prefix validator owner'
require_regex_count "$ir_async_while_prefix" '^fn validate_value_arm\(' 1 'private value-arm validator owner'
check_no_inline_legacy_includes "$ir_async_while_prefix"
check_no_inline_legacy_includes "$ir_async_while_tests"
# Measured constructor70, prefix validator75 and existing unit controls249.
check_raw_line_budget crates/lila-ir/src/async_while.rs 90
check_raw_line_budget "$ir_async_while_prefix" 90
check_raw_line_budget "$ir_async_while_tests" 270
# Loop-context admission precedes condition lowering; its retained context
# save/restore accounts for five additional source lines.
# The eager While boundary admits checked References and routes class/complete
# child phases to the separate whole classic owner.
check_raw_line_budget crates/lila-ir/src/lowering/awaited_while_condition.rs 140
check_raw_line_budget crates/lila-aot-wasm/src/control_flow/async_function_while.rs 90
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
# The dispatcher selects the three complete protocols and CaseBlock resources.
check_raw_line_budget "$ir_switch_statement_lowering" 140
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
# The analyzed continuation token selects whole or original linear With.
check_raw_line_budget "$ir_with_statement_lowering" 160
# T02's property-access boundary owns ordinary, private and super access
# dispatch plus the primitive/exotic target-kind split. Keep that split
# exhaustive; Number now consumes the same live primitive GetV owner as the
# other primitive prototypes, preserving mutable getter effects.
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
  'ValueKind::Number => self.lower_primitive_property_key(' \
  1 \
  'Number dispatch through the shared primitive GetV owner'
require_fixed_string_count "$ir_property_access_lowering" 'IntrinsicPrototype::Number,' 1 'Number selects its actual primitive prototype'
if grep -Fq '_ => self.unsupported_expr("property access on non-object target")' "$ir_property_access_lowering"; then
  fail "$ir_property_access_lowering must exhaust ValueKind instead of hiding future variants behind a catch-all"
fi
check_no_inline_legacy_includes "$ir_property_access_lowering"
# The shared private GetValue body observes accessor effects before ordinary
# and suspended calls consume the acquired value. Syntax dispatch lives beside
# that actual owner, rather than retaining a second parent implementation.
for private_get_value_entry in lower_private_get_value lower_private_property_access; do
  require_fixed_string_count "$ir_property_access_lowering" "pub(super) fn ${private_get_value_entry}(" 1 'sole private GetValue/syntax owner'
  require_fixed_string_count "$ir_lowering" "fn ${private_get_value_entry}(" 0 'no parent private GetValue/syntax copy'
done
require_fixed_string_count "$ir_property_access_lowering" 'self.lower_private_get_value(&mut target, private_name_id)' 1 'syntax private GetValue consumer'
require_fixed_string_count crates/lila-ir/src/lowering/call_expression.rs 'self.lower_private_get_value(&mut materialized_receiver, private_name_id)' 1 'acquired private Call GetValue consumer'
# Measured after the shared private GetValue join: 267 raw lines. The margin is
# for maintenance of this consumed property-access owner only.
check_raw_line_budget "$ir_property_access_lowering" 280
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
# Measured after binding-query and observable coercion repairs: 3,112 raw lines.
# Direct identifier/property recognition remains the only implementation owner.
check_raw_line_budget "$ir_call_expression_lowering" 3112
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
# Measured after TypedArray.fill, Float16Array, Intl.Locale getter and
# likely-subtag entries, legacy accessor definers, six Instant methods
# (including toLocaleString), five ZonedDateTime methods
# (toJSON/valueOf/toLocaleString/toPlainTime/withPlainTime), three
# `Temporal.Now` members (plainDateTimeISO/plainDateISO/plainTimeISO) and
# `Instant.prototype.toZonedDateTimeISO`: 2,292 raw lines.
# This exhaustive result table must not acquire unrelated lowering.
# Nine DisplayNames/RelativeTimeFormat registrations add eight raw lines to
# the reviewed Collator/enumeration surface; optional DisplayNames.of has no
# fixed String result arm.
# Seven Segmenter entries add five formatted raw lines; containing retains
# a conservative optional-object result. Exact source census: 2,324.
# Locale.getWeekInfo adds a three-line Object result arm; getTextInfo shares
# that arm with one further exhaustive variant. HourCycles joins the Array
# result arm with one variant. Measured result table: 2,334.
# Calendar/Collation Array entries and precise TimeZones Array|Undefined add nine lines.
check_raw_line_budget "$ir_builtin_call_info_lowering" 2344
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
# Compare the actual closed enum/catalog names, not a historical variant count.
# The operation macro's one row list also generates its exhaustive enum.
for source_call_flow_variant_domain in StatementIr ExprIr SpecOperationIr; do
  case "$source_call_flow_variant_domain" in
    StatementIr|ExprIr)
      source_call_flow_domain_code="$(braced_rust_item_source crates/lila-ir/src/ir.rs "^pub enum ${source_call_flow_variant_domain}[[:space:]]*[{]")"
      expected_source_call_flow_variants="$(printf '%s\n' "$source_call_flow_domain_code" \
        | sed -nE 's/^    ([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*([(,{].*)$/\1/p' | sort -u)"
      ;;
    SpecOperationIr)
      source_call_flow_domain_code="$(braced_rust_item_source crates/lila-ir/src/operations.rs '^spec_operations![[:space:]]*[{]')"
      expected_source_call_flow_variants="$(printf '%s\n' "$source_call_flow_domain_code" \
        | sed -nE 's/^    ([A-Za-z_][A-Za-z0-9_]*)(\([^)]*\))?[[:space:]]*=>[[:space:]]*\{.*$/\1/p' | sort -u)"
      ;;
  esac
  observed_source_call_flow_variants="$({
    grep -oE "${source_call_flow_variant_domain}::[A-Za-z0-9_]+" "$ir_source_call_flow_proof" || true
  } | sed "s/^${source_call_flow_variant_domain}:://" | sort -u)"
  if [ -z "$expected_source_call_flow_variants" ] \
    || [ "$observed_source_call_flow_variants" != "$expected_source_call_flow_variants" ]; then
    fail "$ir_source_call_flow_proof must exhaust the actual ${source_call_flow_variant_domain} enum/catalog without missing or retired variants"
  fi
done
if grep -Eq '(^|[|,(])[[:space:]]*_[[:space:]]*=>' "$ir_source_call_flow_proof" \
  || grep -Eq '\{[[:space:]]*\.\.[[:space:]]*\}' "$ir_source_call_flow_proof"; then
  fail "$ir_source_call_flow_proof must not hide new IR variants behind catch-all patterns"
fi
check_no_inline_legacy_includes "$ir_source_call_flow_proof"
# The owner includes parameter-default execution and all complete protocol
# variants; the bound remains for this exhaustive proof and its focused tests.
check_raw_line_budget "$ir_source_call_flow_proof" 890
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
ir_class_callable_flow="crates/lila-ir/src/lowering/class_definition/callable_flow.rs"
require_file "$ir_class_callable_flow"
require_exact_line_count "$ir_class_definition_lowering" 'mod callable_flow;' 1 'private class callable-flow module'
require_fixed_string_count "$ir_class_definition_lowering" 'self.finalize_class_callable_flow(' 1 'class callable-flow finalization consumption'
require_regex_count "$ir_class_callable_flow" '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+finalize_class_callable_flow\(' 1 'class callable-flow finalization owner'
require_fixed_string_count "$ir_class_definition_lowering" 'fn finalize_class_callable_flow(' 0 'class flow finalization outside child'
ir_class_field_names="crates/lila-ir/src/lowering/class_definition/field_names.rs"
require_file "$ir_class_field_names"
require_exact_line_count "$ir_class_definition_lowering" 'mod field_names;' 1 'private class field-name owner attachment'
require_exact_line_count "$ir_class_definition_lowering" 'use self::field_names::class_field_initializer_name;' 1 'class planner imports its field-name owner'
require_fixed_string_count "$ir_class_field_names" 'pub(super) fn class_field_initializer_name(' 1 'sole class field initializer naming owner'
require_fixed_string_count "$ir_class_definition_lowering" 'fn class_field_initializer_name(' 0 'no parent field-name implementation copy'
require_fixed_string_count "$ir_class_definition_lowering" 'class_field_initializer_name(' 6 'actual field and accessor naming consumers'
check_no_inline_legacy_includes "$ir_class_field_names"
check_raw_line_budget "$ir_class_field_names" 24
check_no_inline_legacy_includes "$ir_class_definition_lowering"
check_no_inline_legacy_includes "$ir_class_callable_flow"
# Class planning includes separate fresh and later callable private-field facts.
check_raw_line_budget "$ir_class_definition_lowering" 1503
check_raw_line_budget "$ir_class_callable_flow" 90
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
# Measured with the private ModuleActivation protocol: 789 raw lines. The
# margin is for this function lifecycle, not unrelated lowering.
check_raw_line_budget "$ir_function_definition_lowering" 810
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
# Complete catch-pattern prefixes retain the thrown value and original catch
# environment through suspension; catch/finally lifecycle remains one owner.
check_raw_line_budget "$ir_try_statement_lowering" 400
# T02's throw-inference boundary owns the recursive statement analysis and
# carried-PutValue wrapper; its private expression leaf owns operand/key
# analysis. Only block inference crosses the outer private boundary for
# try-statement lowering; shared value/type algebra retains its owners.
ir_throw_inference_lowering="crates/lila-ir/src/lowering/throw_inference.rs"
require_file "$ir_throw_inference_lowering"
ir_throw_expression_lowering="crates/lila-ir/src/lowering/throw_inference/expression.rs"
require_file "$ir_throw_expression_lowering"
require_exact_line_count "$ir_throw_inference_lowering" 'mod expression;' 1 'private expression throw-analysis module'
require_regex_count "$ir_throw_expression_lowering" '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+infer_expr_operand_throw_info\(' 1 'private-child operand analysis boundary'
require_regex_count "$ir_throw_expression_lowering" '^[[:space:]]*fn[[:space:]]+infer_property_key_throw_info\(' 1 'private property-key analysis owner'
require_regex_count "$ir_throw_expression_lowering" '^[[:space:]]*fn[[:space:]]+merge_object_property_throw_info\(' 1 'private object-property analysis owner'
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
  infer_statement_sequence_throw_info \
  infer_statement_throw_info \
  infer_expr_throw_info
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
  infer_statement_sequence_throw_info \
  infer_statement_throw_info \
  infer_expr_throw_info \
  infer_expr_operand_throw_info \
  merge_object_property_throw_info \
  infer_property_key_throw_info
do
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*((pub(\\([^)]*\\))?|default|const|async|unsafe|extern|\"[^\"]*\")[[:space:]]+)*fn[[:space:]]+${throw_owner}[[:space:]]*[<(]" \
    0 \
    "${throw_owner} outside throw-inference child"
done
# Four parent-private methods plus its outer block entry and three child
# operand/object/key methods form the eight-function closure. Count qualified
# declarations too, so modifiers cannot evade the closed owner inventory.
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*fn[[:space:]]+' \
  4 \
  'private throw-inference parent method'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*[<(]' \
  5 \
  'total throw-inference parent function declaration'
require_regex_count \
  "$ir_throw_inference_lowering" \
  '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' \
  1 \
  'Rust-visible item'
require_regex_count "$ir_throw_expression_lowering" '^[[:space:]]*fn[[:space:]]+' 2 'private throw-expression object/key methods'
require_regex_count "$ir_throw_expression_lowering" '^[[:space:]]*((pub(\([^)]*\))?|default|const|async|unsafe|extern|"[^"]*")[[:space:]]+)*fn[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*[<(]' 3 'total throw-expression function declarations'
require_regex_count "$ir_throw_expression_lowering" '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+' 1 'inner expression-analysis visible entry'
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
require_tree_regex_count crates/lila-ir/src/lowering '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+merge_value_infos[[:space:]]*\(' 1 'sole shared value-fact merge owner'
require_fixed_string_count crates/lila-ir/src/lowering/conditional_flow.rs 'pub(super) fn merge_value_infos(' 1 'conditional-flow owns the shared value algebra'
require_regex_count "$ir_lowering" '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+merge_value_infos[[:space:]]*\(' 0 'no parent value-fact merge copy'
require_regex_count "$ir_throw_inference_lowering" '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+merge_value_infos[[:space:]]*\(' 0 'no throw-analysis value-fact merge copy'
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
for throw_closed_owner in "$ir_throw_inference_lowering" "$ir_throw_expression_lowering"; do
require_regex_count \
  "$throw_closed_owner" \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?((unsafe|auto)[[:space:]]+)*(struct|enum|union|type|trait)[[:space:]]+(r#)?[[:alpha:]_][[:alnum:]_]*([[:space:]]|<|\{|\(|=|;|:)' \
  0 \
  'local type or trait declaration'
# This owner needs no textual runtime data. Keeping non-line-comment block
# comments and string literals out also makes the executable shape checks below
# immune to code-looking text in /* ... */ or raw/multiline strings.
throw_non_line_comment_source="$(sed '/^[[:space:]]*\/\//d' "$throw_closed_owner")"
if printf '%s\n' "$throw_non_line_comment_source" | grep -Eq '/\*|\*/|"'; then
  fail "$throw_closed_owner must express throw analysis through typed values, not block comments or string literals"
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
' "$throw_closed_owner")"
if [ -n "$throw_catch_all_arms" ]; then
  fail "$throw_closed_owner must keep every match exhaustive without catch-all arms"
fi
require_fixed_string_count \
  "$throw_closed_owner" \
  'unreachable!' \
  0 \
  'unreachable escape hatch'
require_fixed_string_count \
  "$throw_closed_owner" \
  'macro_rules!' \
  0 \
  'local macro definition'
require_regex_count \
  "$throw_closed_owner" \
  '^(    )?(::[[:space:]]*)?((r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*::[[:space:]]*)*(r#)?[[:alpha:]_][[:alnum:]_]*[[:space:]]*!' \
  0 \
  'module-or-impl-level generated helper invocation'
done
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
check_no_inline_legacy_includes "$ir_throw_expression_lowering"
# Actual closed statement owner529 includes whole Switch/Throw/Array paths;
# the exhaustive expression leaf retains its original550-line ceiling.
# Exhaustive inference includes complete loop/Switch/Array/resource runtime
# throws; its leaf also covers retained property/Super and object operands.
check_raw_line_budget "$ir_throw_inference_lowering" 660
check_raw_line_budget "$ir_throw_expression_lowering" 620
# Runtime JSON.parse owns parsing and the framed reviver walk. The retired
# static parser/protocol must not return as a separate compiler path.
for retired_json_path in \
  crates/lila-ir/src/lowering/static_json_parse.rs \
  crates/lila-aot-wasm/src/builtins/json/static_reviver.rs
do
  if [ -e "$retired_json_path" ]; then
    fail "retired static JSON compiler owner must remain absent: $retired_json_path"
  fi
done
for json_production_root in crates/lila-ir/src crates/lila-aot-wasm/src; do
  require_tree_regex_count \
    "$json_production_root" \
    'static_json_parse|static_reviver|JsonStaticParser|JsonStaticValueIr|JsonParseStaticReviver|PreparedStaticJsonParseReviver|prepare_static_json_parse_reviver|finish_static_json_parse_reviver' \
    0 \
    'retired static JSON compiler references'
done
ir_static_string_binding_facts_lowering="crates/lila-ir/src/lowering/static_string_binding_facts.rs"
require_file "$ir_static_string_binding_facts_lowering"
require_exact_line_count \
  "$ir_lowering" \
  'mod static_string_binding_facts;' \
  1 \
  'private static-string fact module declaration'
require_regex_count \
  "$ir_lowering" \
  '^(pub(\([^)]*\))?[[:space:]]+)?mod[[:space:]]+static_string_binding_facts;' \
  1 \
  'total static-string fact module declaration'
static_string_module_context="$(sed -n '/^mod statement;$/,+3p' "$ir_lowering")"
if [ "$static_string_module_context" != $'mod statement;\nmod static_literals;\nmod static_string_binding_facts;\nmod super_property_mutation;' ]; then
  fail "$ir_lowering must keep static_literals and static_string_binding_facts private between statement and super_property_mutation"
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
  1 \
  'binding-owned lowerer static-string facts'
require_exact_line_count crates/lila-ir/src/lowering/conditional_flow.rs '    static_string_bindings: StaticStringBindingFacts,' 1 'conditional-flow snapshots retain the same binding-owned facts'
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
# Reviver callback observations are consumed by the ordinary JSON.parse call
# analysis even though static parsing has been retired.
for retained_owner in known_json_parse_reviver_targets observe_json_parse_reviver_targets; do
  require_regex_count \
    "$ir_lowering" \
    "^[[:space:]]*fn[[:space:]]+${retained_owner}[[:space:]]*[<(]" \
    1 \
    "parent-owned ${retained_owner} helper"
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
  'runtime JSON.parse target discovery'
require_exact_line_count \
  "$ir_lowering" \
  '            self.observe_json_parse_reviver_targets(reviver_targets, &helper_context_id);' \
  1 \
  'runtime JSON.parse target observation'
ir_static_literals_lowering="crates/lila-ir/src/lowering/static_literals.rs"
require_file "$ir_static_literals_lowering"
require_exact_line_count "$ir_lowering" 'mod static_literals;' 1 'private static-literal compilation module'
for static_literal_entry in static_regexp_compilation_for_pattern static_regexp_compilation_for_direct_call; do
  require_regex_count "$ir_static_literals_lowering" "^[[:space:]]*pub\\(super\\)[[:space:]]+fn[[:space:]]+${static_literal_entry}[[:space:]]*\\(" 1 "consumed static-literal boundary ${static_literal_entry}"
  require_regex_count "$ir_lowering" "^[[:space:]]*(pub\\([^)]*\\)[[:space:]]+)?fn[[:space:]]+${static_literal_entry}[[:space:]]*\\(" 0 "static-literal body outside child ${static_literal_entry}"
done
require_fixed_string_count "$ir_lowering" 'static_compilation: Self::static_regexp_compilation_for_pattern(&pattern, &flags),' 1 'RegExp literal compilation consumption'
require_fixed_string_count "$ir_call_expression_lowering" 'self.static_regexp_compilation_for_direct_call(' 1 'direct RegExp call compilation consumption'
check_no_inline_legacy_includes "$ir_static_literals_lowering"
# Both unchecked call-site folds and their sole helpers are retired. The two
# consumed static RegExp compilation functions retain their private owner.
check_raw_line_budget "$ir_static_literals_lowering" 90
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
# carried from five lowering contexts into the live semantic consumers. Generic IR
# visitors that only transport the field are deliberately outside this census.
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
require_array_destructuring_producer "$ir_lowering" lower_pattern_assign_value_unscoped AssignmentEvaluation
array_assignment_scope="$(braced_rust_item_source "$ir_lowering" '^fn lower_pattern_assign_value[(]')"
require_text_regex_count "$array_assignment_scope" 'self[.]lower_pattern_assign_value_unscoped[(]pattern, value[)]' 1 'scoped assignment delegates to its sole evaluation producer'
require_array_destructuring_producer "$ir_lowering" lower_pattern_lexical_binding BindingInitialization
require_array_destructuring_producer "$ir_lowering" lower_pattern_var_binding_from_value BindingInitialization
require_array_destructuring_producer "$ir_lowering" lower_pattern_lexical_binding_from_value_with_storage_names BindingInitialization
require_fixed_string_count "$ir_lowering" 'evaluation: ArrayDestructuringEvaluationIr::' 5 'reviewed array-destructuring evaluation producers'

require_array_destructuring_consumer "$ir_ir" validate_async_function_for_of_initialization
# This exhaustive walker belongs to cfg(test) support, not the product graph.
ir_array_test_support="crates/lila-ir/src/tests/support.rs"
require_array_destructuring_consumer "$ir_array_test_support" collect_binding_storage_names
require_array_destructuring_consumer crates/lila-aot-wasm/src/control_flow.rs initialize_direct_lexical_bindings
require_array_destructuring_consumer crates/lila-aot-wasm/src/control_flow/array_destructuring.rs compile_array_destructure_to_locals
require_array_destructuring_consumer crates/lila-aot-wasm/src/planning.rs collect_hoisted_vars_statement

array_destructuring_variant_product_files="$({
  while IFS= read -r product_source_file; do
    if rust_source_without_inline_tests "$product_source_file" \
      | grep -F 'ArrayDestructuringEvaluationIr::' >/dev/null; then
      printf '%s\n' "$product_source_file"
    fi
  done < <(find crates -type f -path '*/src/*.rs' ! -path 'crates/lila-ir/src/tests/*' -print | sort)
})"
expected_array_destructuring_variant_product_files='crates/lila-aot-wasm/src/control_flow.rs
crates/lila-aot-wasm/src/control_flow/array_destructuring.rs
crates/lila-aot-wasm/src/planning.rs
crates/lila-ir/src/generator_for_of_iterator/head.rs
crates/lila-ir/src/generator_loop_control/head_bindings.rs
crates/lila-ir/src/ir.rs
crates/lila-ir/src/lowering.rs'
if [ "$array_destructuring_variant_product_files" != "$expected_array_destructuring_variant_product_files" ]; then
  fail "direct ArrayDestructuringEvaluationIr variant use must stay in the reviewed semantic proof and emission owners: $array_destructuring_variant_product_files"
fi
for product_variant_spec in \
  'crates/lila-ir/src/lowering.rs|5' \
  'crates/lila-ir/src/ir.rs|2' \
  'crates/lila-aot-wasm/src/control_flow.rs|2' \
  'crates/lila-aot-wasm/src/control_flow/array_destructuring.rs|2' \
  'crates/lila-aot-wasm/src/planning.rs|2' \
  'crates/lila-ir/src/generator_for_of_iterator/head.rs|1' \
  'crates/lila-ir/src/generator_loop_control/head_bindings.rs|2'
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
# Measured after signature/declaration extraction and the native module-source
# retrieval signature: 18,167 parent lines.
# Further implementation growth needs a reviewed owner rather than restoring the
# former 32k-line implementation store.
check_raw_line_budget "$ir_lowering" 18200

ir_module_graph_lowering="crates/lila-ir/src/lowering/module_graph.rs"
require_file "$ir_module_graph_lowering"
require_module_decl "$ir_lowering" "module_graph"
require_fixed_string_count "$ir_module_graph_lowering" 'fn lower_graph(' 1 'module graph lowering owner'
require_fixed_string_count "$ir_lowering" 'fn lower_graph(' 0 'module graph lowering outside child module'
ir_module_graph_session="crates/lila-ir/src/lowering/module_graph/session.rs"
require_file "$ir_module_graph_session"
require_exact_line_count "$ir_module_graph_lowering" 'mod session;' 1 'private retained module-graph session owner'
require_fixed_string_count "$ir_module_graph_session" 'fn lower_graph_with_admission(' 1 'sole retained-session graph admission owner'
require_fixed_string_count "$ir_module_graph_lowering" 'fn lower_graph_with_admission(' 0 'no parent retained-session admission copy'
require_fixed_string_count "$ir_module_graph_session" 'self.lower_graph_with_admission(' 2 'loaded and declared catalogs consume the same session admission owner'
require_exact_line_count "$ir_module_graph_session" '        lower_graph(' 1 'session consumes the original graph lowerer'
check_no_inline_legacy_includes "$ir_module_graph_session"
check_raw_line_budget "$ir_module_graph_session" 64
# Source-goal validation, linking and independent Script prelude orchestration.
# Complete catalog admission and its owned partition add eighteen lines.
check_raw_line_budget "$ir_module_graph_lowering" 218

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
ir_native_name="crates/lila-ir/src/builtins/native_name.rs"
require_file "$ir_native_name"
require_exact_line_count "$ir_builtins" 'mod native_name;' 1 'private builtin native-name validator module'
require_regex_count "$ir_native_name" '^pub\(super\) const fn is_catalog_native_function_name\(' 1 'closed catalog native-name validator owner'
require_fixed_string_count crates/lila-ir/src/builtins/catalog.rs 'super::native_name::is_catalog_native_function_name(name)' 1 'catalog native-name validation consumption'
require_fixed_string_count "$ir_callable_to_string" 'is_catalog_native_function_name' 0 'native-name validation outside its owner'
check_no_inline_legacy_includes "$ir_native_name"
# Existing const validation and matching unit control occupy83 raw lines.
check_raw_line_budget "$ir_native_name" 100
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
# The async disposal bridge invokes the real synchronous disposer host entry.
if [[ "$host_builtin_catalog_rows" != "24" ]]; then
  fail "$ir_host_builtin_catalog must contain the reviewed 24-row host builtin catalog (found $host_builtin_catalog_rows)"
fi
require_fixed_string_count "$ir_host_builtin_catalog" '    AsyncDisposableStackSyncDispose {' 1 'real sync-disposer host bridge in the sole catalogue'
require_fixed_string_count crates/lila-ir/src/builtins/host_surface.rs 'HostBuiltinId::AsyncDisposableStackSyncDispose,' 1 'actual disposable host surface consumes the bridge'
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
# Measured after the Float16Array and Intl.Locale identities: 1,773 raw lines.
# Metadata rows belong in their catalogs; shared machinery should shrink rather
# than regrow.
# The consumed SystemTimeZone capability accessor adds seven actual lines.
# The private native_name module declaration adds one orchestration line.
check_raw_line_budget "$ir_builtins" 1788

for module in abi arguments_protocol control_flow data emit environments expressions functions gc_types heap module modules objects operations planning; do
  require_file "crates/lila-aot-wasm/src/${module}.rs"
  require_module_decl "$wasm_lib" "$module"
done

# T15 gives async functions and plain generators one complete synchronous
# iterator owner. Typed wrappers select the closed activation protocol; the
# obligation ledger names the shared owner and both production entry points.
wasm_control_flow="crates/lila-aot-wasm/src/control_flow.rs"
wasm_async_function_for_of_iterator="crates/lila-aot-wasm/src/control_flow/async_function_for_of_iterator.rs"
wasm_resumable_sync_for_of_iterator="crates/lila-aot-wasm/src/control_flow/resumable_sync_for_of_iterator.rs"
wasm_resumable_sync_for_of_plan="crates/lila-aot-wasm/src/control_flow/resumable_sync_for_of_iterator/plan.rs"
wasm_for_await_iterator_plan="crates/lila-aot-wasm/src/control_flow/for_await_iterator_plan.rs"
wasm_emission_sites="crates/lila-aot-wasm/src/emission_sites.rs"
for private_child in async_function_for_of_iterator resumable_sync_for_of_iterator for_await_iterator_plan; do
  private_child_file="crates/lila-aot-wasm/src/control_flow/${private_child}.rs"
  require_file "$private_child_file"
  require_exact_line_count \
    "$wasm_control_flow" \
    "mod ${private_child};" \
    1 \
    "private ${private_child} child declarations"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*mod[[:space:]]+${private_child};" \
    1 \
    "private ${private_child} child declarations"
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${private_child};" "$wasm_control_flow"; then
    fail "$wasm_control_flow must keep $private_child private"
  fi
  check_no_inline_legacy_includes "$private_child_file"
done
check_no_inline_legacy_includes "$wasm_control_flow"

require_file "$wasm_resumable_sync_for_of_plan"
require_exact_line_count "$wasm_resumable_sync_for_of_iterator" 'mod plan;' 1 'private resumable plan projection module'
if grep -Eq '^pub(\([^)]*\))? mod plan;' "$wasm_resumable_sync_for_of_iterator"; then
  fail "$wasm_resumable_sync_for_of_iterator must keep its plan projection child private"
fi
check_no_inline_legacy_includes "$wasm_resumable_sync_for_of_plan"

require_exact_line_count \
  "$wasm_resumable_sync_for_of_iterator" \
  '    pub(crate) fn compile_resumable_sync_for_of_iterator(' \
  1 \
  'crate-visible shared resumable synchronous for-of owner'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+compile_resumable_sync_for_of_iterator[[:space:]]*\(' \
  1 \
  'complete shared resumable synchronous for-of owners'
require_fixed_string_count \
  "$wasm_async_function_for_of_iterator" \
  '.compile_resumable_sync_for_of_iterator(' \
  2 \
  'typed async and generator calls to the shared owner'
require_exact_line_count \
  "$wasm_emission_sites" \
  '            let _ = FunctionBuilder::compile_resumable_sync_for_of_iterator;' \
  1 \
  'shared resumable synchronous for-of obligation-ledger reference'
require_tree_regex_count \
  crates/lila-aot-wasm/src \
  'compile_resumable_sync_for_of_iterator' \
  4 \
  'shared owner, two typed wrappers and obligation-ledger sites'

for wrapper in compile_async_function_for_of_iterator compile_generator_for_of_iterator; do
  require_exact_line_count \
    "$wasm_async_function_for_of_iterator" \
    "    pub(crate) fn ${wrapper}(" \
    1 \
    "typed ${wrapper} wrapper declarations"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+${wrapper}[[:space:]]*\(" \
    1 \
    "typed ${wrapper} wrapper owners"
  require_fixed_string_count \
    "$wasm_control_flow" \
    "self.${wrapper}(iterable, plan, function)?;" \
    1 \
    "${wrapper} statement-dispatch calls"
  require_exact_line_count \
    "$wasm_emission_sites" \
    "            let _ = FunctionBuilder::${wrapper};" \
    1 \
    "${wrapper} obligation-ledger references"
  require_tree_regex_count \
    crates/lila-aot-wasm/src \
    "$wrapper" \
    3 \
    "${wrapper} owner, dispatch and obligation-ledger sites"
done
if grep -Eq 'reserve_temp_local|emit_get_iterator|emit_sync_iterator_step_value|emit_iterator_close|compile_(async|generator)_statement_sequence|finally_stack' "$wasm_async_function_for_of_iterator"; then
  fail "$wasm_async_function_for_of_iterator must remain typed delegation without an inline iterator body"
fi

# The complete typed owner retains the Iterator Record through suspension and
# keeps the pending completion intact across close and activation dispatch.
# Digest pins and raw completion/iterator-local recipes belonged to the retired ABI.
resumable_sync_for_of_owner="$(braced_rust_item_source "$wasm_resumable_sync_for_of_iterator" '^pub[(]crate[)][[:space:]]+fn[[:space:]]+compile_resumable_sync_for_of_iterator[[:space:]]*[(]')"
if ! awk '
  index($0, "self.emit_get_sync_iterator(") && !acquire { acquire = NR }
  index($0, "let loop_frame = self.open_frame") && !loop { loop = NR }
  index($0, "self.emit_sync_iterator_step_value(") && !step { step = NR }
  index($0, "self.breakable_stack.push(break_frame);") && !break_publish { break_publish = NR }
  index($0, "self.loop_stack.push(LoopTargets { continue_frame });") && !continue_publish { continue_publish = NR }
  index($0, "self.finally_stack.push(close_frame);") && !close_publish { close_publish = NR }
  index($0, "self.compile_resumable_statement_sequence(") && !body { body = NR }
  index($0, "CompletionKind::Continue.code()") && !continue_kind { continue_kind = NR }
  index($0, "self.set_completion_kind(CompletionKind::Normal, function);") && body && !continue_consume { continue_consume = NR }
  index($0, "self.loop_stack.pop();") && !continue_retire { continue_retire = NR }
  index($0, "pending.copy_from(self.completion(), function);") && !save { save = NR }
  index($0, "self.emit_sync_iterator_close(") && !iterator_close { iterator_close = NR }
  index($0, "self.completion().copy_from(&closed, function);") && !restore { restore = NR }
  index($0, "owner.emit_dispatch_completion(self, function)?;") && !dispatch { dispatch = NR }
  index($0, "self.breakable_stack.pop();") && !break_retire { break_retire = NR }
  /^[[:space:]]*iterator[.]clear[(]function[)];[[:space:]]*$/ && !release { release = NR }
  END {
    exit !(acquire && acquire < loop && loop < step && step < break_publish \
      && break_publish < continue_publish && continue_publish < close_publish \
      && close_publish < body && body < continue_kind && continue_kind < continue_consume \
      && continue_consume < continue_retire && continue_retire < save && save < iterator_close \
      && iterator_close < restore && restore < dispatch && dispatch < break_retire && break_retire < release)
  }
' <<<"$resumable_sync_for_of_owner"; then
  fail "$wasm_resumable_sync_for_of_iterator must preserve typed acquire, step, local target lifetime, whole close and dispatch order"
fi
require_file crates/lila-engine/tests/aot_generator_for_of_continuations.rs
require_file crates/lila-engine/tests/fixtures/generator_for_of_continuations/iterator-close-completion-precedence.js
require_file crates/lila-engine/tests/fixtures/generator_for_of_continuations/assignment-head-errors-close.js

# Both closed owners use the same reviewed target lifecycle. The GC parent now
# measures 378 raw lines and its consumed projection child 135. Preserve the
# parent's earlier 638-line ceiling; keep the child within its measured scope.
# The parent adds narrow checked whole-protocol dispatch, range/TDZ joins and
# committed completion routing; iterator/loop/resource algorithms stay children.
check_raw_line_budget "$wasm_control_flow" 9800
check_raw_line_budget "$wasm_async_function_for_of_iterator" 60
# The opaque paired projection and body owner measures 150 raw lines.
check_raw_line_budget "$wasm_for_await_iterator_plan" 170
check_raw_line_budget "$wasm_resumable_sync_for_of_iterator" 638
check_raw_line_budget "$wasm_resumable_sync_for_of_plan" 150

# T05's central GC schema owns raw struct/array instruction encoding.
# Typed construction, fields and complete values are implemented by its private
# layouts/value children; builtin-local encoders would bypass those types.
wasm_gc_types="crates/lila-aot-wasm/src/gc_types.rs"
wasm_gc_layouts="crates/lila-aot-wasm/src/gc_types/layouts.rs"
wasm_gc_values="crates/lila-aot-wasm/src/gc_types/value.rs"
require_file "$wasm_gc_layouts"
require_file "$wasm_gc_values"
require_exact_line_count "$wasm_gc_types" 'mod value;' 1 'private whole GC value module'
check_no_inline_legacy_includes "$wasm_gc_values"
require_exact_line_count "$wasm_gc_types" 'mod layouts;' 1 'private typed GC layout module'
check_no_inline_legacy_includes "$wasm_gc_layouts"
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
                  CallableFunctionTableSections ModuleAssemblySections \
                  ModuleGlobalSectionBuilder; do
  require_fixed_string_count "$compiled_module_package" "struct ${sole_owner}" 1 "${sole_owner} owners"
  if sed '/^#\[cfg(test)\]/,$d' crates/lila-aot-wasm/src/module.rs \
    | grep -Fq "struct ${sole_owner}"; then
    fail "$sole_owner must be owned only by $compiled_module_package"
  fi
done
gc_instruction_escapes="$(
  rg -n 'Instruction::(Struct(New|Get|Set)|Array(New|Get|Set|Fill|Copy|Len))' \
    crates/lila-aot-wasm/src --glob '*.rs' --glob '!**/gc_types/**' || true
)"
if [ -n "$gc_instruction_escapes" ]; then
  fail "raw Wasm-GC record/array instructions must stay in the central schema: $gc_instruction_escapes"
fi
require_fixed_string_count \
  "$wasm_gc_layouts" \
  'function.instruction(&Instruction::StructNew(' \
  1 \
  'typed StructNew encoder boundary'
require_fixed_string_count "$wasm_gc_layouts" 'GcStorageGet::Unpacked => Instruction::StructGet {' 1 'typed unpacked StructGet encoder boundary'
require_fixed_string_count "$wasm_gc_layouts" 'GcStorageGet::UnsignedPacked => Instruction::StructGetU {' 1 'typed packed StructGet encoder boundary'
require_fixed_string_count \
  "$wasm_gc_layouts" \
  'function.instruction(&Instruction::StructSet {' \
  1 \
  'typed mutable-field StructSet encoder boundary'
require_fixed_string_count "$compiled_module_package" 'registered: RuntimeModuleTypes::register(),' 1 'sole consumed callable/layout type registration'

# The consumed global finalizer declares each concrete root against the actual
# encoded section. Generic typed globals keep their index construction private.
if grep -R -Eq 'runtime_gc_root_global_index|fn bind_root\(' crates/lila-aot-wasm/src; then
  fail 'runtime GC roots must come from the consumed global section'
fi
gc_root_finalizer="$(braced_rust_item_source "$wasm_gc_types" '^fn finalize_globals[(]')"
gc_root_finalizer_code="$(printf '%s\n' "$gc_root_finalizer" | tr -d '[:space:]')"
gc_registered_finalizer="$(braced_rust_item_source "$wasm_gc_types" '^pub[(]crate[)] fn finalize_globals[(]')"
gc_registered_finalizer_code="$(printf '%s\n' "$gc_registered_finalizer" | tr -d '[:space:]')"
if [[ "$gc_root_finalizer_code" != *'self,mutglobals:GlobalLedger,snapshot:bool,module_guard_count:u32,)->FinalizedModuleGlobals{'* ]] \
  || [[ "$gc_root_finalizer_code" != *'snapshot.then(||snapshot::SnapshotRoots::declare(&self.layouts,&mutglobals))'* ]] \
  || [[ "$gc_root_finalizer_code" != *'FinalizedModuleGlobals{section:globals,runtime_schema,}'* ]] \
  || [[ "$gc_registered_finalizer_code" != *'globals:self.runtime.finalize_globals(globals,snapshot,module_guard_count)'* ]] \
  || ! grep -Fq 'current_realm: GcRootGlobal<RealmRecord>,' "$wasm_gc_types" \
  || ! grep -Fq 'section: GlobalLedger,' "$wasm_gc_types"; then
  fail 'both finalization modes must consume the actual global ledger through one typed-root owner and seal its matching schema'
fi
require_text_regex_count "$gc_root_finalizer" 'GcRootGlobal::declare[(]&self.layouts, &mut globals[)]' 14 'declared semantic GC roots in the sole finalizer'
require_text_regex_count "$gc_root_finalizer" 'CollectionKeyHashCounterGlobal::declare[(]&mut globals[)]' 1 'declared scalar collection identity root'
require_regex_count "$wasm_gc_types" '^[[:space:]]+fn finalize_globals[(]' 1 'sole concrete GC-root finalizer'
require_regex_count "$wasm_gc_types" '^[[:space:]]+pub[(]crate[)] fn finalize_globals[(]' 1 'sole registered type and global finalizer'
wasm_gc_snapshot="crates/lila-aot-wasm/src/gc_types/snapshot.rs"
require_file "$wasm_gc_snapshot"
require_exact_line_count "$wasm_gc_types" 'mod snapshot;' 1 'private declaration-derived snapshot projection owner'
gc_snapshot_roots="$(braced_rust_item_source "$wasm_gc_snapshot" '^pub[(]super[)] struct SnapshotRoots[[:space:]]*[{]')"
if grep -Eq '^[[:space:]]+pub([[:space:]]|[(])' <<<"$gc_snapshot_roots" \
  || ! grep -Fq 'entry: GcRootGlobal<RealmRecord>,' <<<"$gc_snapshot_roots" \
  || ! grep -Fq 'inventory: GcRootGlobal<SnapshotRealmInventory>,' <<<"$gc_snapshot_roots"; then
  fail 'opt-in snapshot roots must retain private typed Realm and inventory identities'
fi
gc_snapshot_declaration="$(braced_rust_item_source "$wasm_gc_snapshot" '^pub[(]super[)] fn declare[(]')"
for snapshot_root_invariant in \
  'let entry = GcRootGlobal::declare(layouts, globals);' \
  'let inventory = GcRootGlobal::declare(layouts, globals);' \
  'let index = globals.len();' \
  'layouts.reference(layout.layout(), GcNullability::Nullable)' \
  'mutable: false,' \
  '&ConstExpr::ref_null(reference.heap_type)'; do
  if ! grep -Fq "$snapshot_root_invariant" <<<"$gc_snapshot_declaration"; then
    fail "snapshot witnesses must derive immutable globals from their actual layout and section: $snapshot_root_invariant"
  fi
done
require_fixed_string_count "$wasm_gc_types" 'snapshot: Option<snapshot::SnapshotRoots>,' 1 'sealed optional snapshot root owner'
require_fixed_string_count "$wasm_gc_snapshot" 'let Some(roots) = &self.snapshot else {' 2 'opt-in export and Realm-retention gates'
require_fixed_string_count "$wasm_gc_snapshot" 'fn publish_snapshot_record(self, _record: GcStackReference<T>, function: &mut Function)' 1 'private typed inventory publication boundary'
require_fixed_string_count crates/lila-aot-wasm/src/functions.rs 'schema.retain_snapshot_realm(realm, function)' 1 'sole original Realm allocator retention consumer'
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
require_fixed_string_count "$compiled_module_package" 'runtime: self.registered.finalize_globals(' 1 'global-ledger finalization through the typed runtime registry'
require_fixed_string_count crates/lila-aot-wasm/src/emit/module_assembly.rs \
  'module_types.finalize_globals(globals, uses_heap, module_guard_count)' 1 'both module modes share one sealed global finalizer'
package_global_finalizer="$(braced_rust_item_source "$compiled_module_package" '^pub[(]crate[)] fn finalize_globals[(]')"
package_global_finalizer_code="$(printf '%s\n' "$package_global_finalizer" | tr -d '[:space:]')"
if [[ "$package_global_finalizer_code" != *'self,globals:ModuleGlobalSectionBuilder,snapshot_roots:bool,module_guard_count:u32,)->FinalizedModuleSections{'* ]] \
  || [[ "$package_global_finalizer_code" != *'runtime:self.registered.finalize_globals(globals.section,snapshot_roots,module_guard_count,)'* ]]; then
  fail 'global finalization must consume registered types and the actual ledger into the same sealed package, retaining snapshot and module-guard policies'
fi
require_fixed_string_count crates/lila-aot-wasm/src/emit/module_assembly.rs \
  'module_package.schema().export_snapshot_roots(&mut exports);' 1 'package-owned opt-in snapshot exports'
# Body compilation and entry roots are independent private owners, while the
# assembly owner consumes their planned bodies and the finalized module package.
wasm_emit_parent="crates/lila-aot-wasm/src/emit.rs"
wasm_body_compilation="crates/lila-aot-wasm/src/emit/body_compilation.rs"
wasm_body_entry="crates/lila-aot-wasm/src/emit/body_entry.rs"
wasm_module_assembly="crates/lila-aot-wasm/src/emit/module_assembly.rs"
for emitter_owner in body_compilation body_entry module_assembly; do
  emitter_owner_path="crates/lila-aot-wasm/src/emit/${emitter_owner}.rs"
  require_file "$emitter_owner_path"
  require_exact_line_count "$wasm_emit_parent" "mod ${emitter_owner};" 1 'private emitter owner attachment'
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${emitter_owner};" "$wasm_emit_parent"; then
    fail "$wasm_emit_parent must keep ${emitter_owner} private"
  fi
  check_no_inline_legacy_includes "$emitter_owner_path"
done
require_regex_count "$wasm_emit_parent" "^impl<'a> FunctionBuilder<'a>[[:space:]]*[{]" 1 'single parent FunctionBuilder implementation to audit'
parent_function_builder="$(braced_rust_item_source "$wasm_emit_parent" "^impl<'a> FunctionBuilder<'a>[[:space:]]*[{]")"
for body_compiler in compile_callable compile_builtin_callable compile; do
  require_fixed_string_count "$wasm_body_compilation" "pub(super) fn ${body_compiler}(" 1 'private planned-body compiler entry'
  require_text_regex_count "$parent_function_builder" "fn[[:space:]]+${body_compiler}[(]" 0 'no parent FunctionBuilder body compiler copy'
done
require_fixed_string_count "$wasm_body_compilation" 'fn compile_builtin(' 1 'private native body implementation'
require_fixed_string_count "$wasm_module_assembly" 'builder.compile_callable()?' 2 'source and prepared callable compilation consumers'
require_fixed_string_count "$wasm_module_assembly" 'builder.compile_builtin_callable()?' 2 'standard and host callable compilation consumers'
# Runtime modules compile every native body through the same private compiler;
# imported program functions have no source-shaped or placeholder stub body.
require_fixed_string_count "$wasm_module_assembly" 'StandardBuiltinId::all_functions().to_vec()' 1 'complete runtime standard-builtin inventory'
require_fixed_string_count "$wasm_module_assembly" 'HostBuiltinId::ALL.to_vec()' 1 'complete runtime host-builtin inventory'
for body_owner in "$wasm_emit_parent" "$wasm_body_compilation" "$wasm_module_assembly"; do
  require_fixed_string_count "$body_owner" 'compile_shared_stub_callable' 0 'no retired native stub body path'
done
require_fixed_string_count "$wasm_body_entry" 'pub(crate) fn begin_helper_body(' 1 'registered helper body entry owner'
require_fixed_string_count "$wasm_emit_parent" 'fn begin_helper_body(' 0 'no parent helper entry copy'
require_fixed_string_count "$wasm_body_entry" 'FunctionModuleState::RuntimeOperation(actual) if actual == helper' 1 'helper entry matches its registered body'
require_fixed_string_count "$wasm_body_compilation" 'self.clear_body_entry_roots(&mut function);' 2 'source and native root retirement consumers'
require_fixed_string_count "$wasm_module_assembly" 'pub(super) fn emit_script_module(' 1 'private physical module assembly entry'
require_fixed_string_count "$wasm_emit_parent" 'module_assembly::emit_script_module(' 2 'runtime and program assembly consumers'
require_fixed_string_count "$wasm_emit_parent" 'fn emit_script_module(' 0 'no parent assembly implementation copy'
program_module_emission="$(braced_rust_item_source "$wasm_emit_parent" '^fn emit_script[(]')"
runtime_module_emission="$(braced_rust_item_source "$wasm_emit_parent" '^pub[(]crate[)] fn emit_runtime_module[(]')"
require_text_regex_count "$program_module_emission" 'module_assembly::emit_script_module[(]' 1 'program or standalone assembly consumer'
require_text_regex_count "$program_module_emission" 'Some[(]runtime[)] => ModuleKind::Program[(]runtime[)]' 1 'heap program links its selected runtime'
require_text_regex_count "$program_module_emission" 'None => ModuleKind::Standalone' 1 'heap-free standalone module selection'
require_text_regex_count "$runtime_module_emission" 'module_assembly::emit_script_module[(]' 1 'runtime assembly consumer'
require_text_regex_count "$runtime_module_emission" 'ModuleKind::Runtime,' 1 'runtime caller selects the runtime module kind'
# The callable builder now carries typed resume-environment/resource authority
# and sparse Intl frame selection. Entry owns the per-point ancestry selection,
# retaining the original saved-chain path for uncertified linear points.
check_raw_line_budget "$wasm_emit_parent" 2320
require_module_decl "$wasm_emit_parent" async_generator_admission
check_raw_line_budget crates/lila-aot-wasm/src/emit/async_generator_admission.rs 550
check_raw_line_budget "$wasm_body_compilation" 550
check_raw_line_budget "$wasm_body_entry" 600
wasm_literal_roots="crates/lila-aot-wasm/src/emit/body_entry/literal_roots.rs"
require_file "$wasm_literal_roots"
require_exact_line_count "$wasm_body_entry" 'mod literal_roots;' 1 'private literal-root initialization owner'
for literal_root_entry in initialize_gc_literal_roots emit_pooled_string_slot compile_pooled_strings_initialize_helper; do
  require_fixed_string_count "$wasm_literal_roots" "fn ${literal_root_entry}(" 1 'literal-root algorithm owner'
  require_fixed_string_count "$wasm_body_entry" "fn ${literal_root_entry}(" 0 'no parent literal-root algorithm copy'
done
require_fixed_string_count "$wasm_literal_roots" 'pub(in crate::emit) fn initialize_gc_literal_roots(' 1 'literal-root entry stays inside emission'
require_fixed_string_count "$wasm_body_compilation" 'self.initialize_gc_literal_roots(&mut function)?;' 1 'source body consumes the canonical literal-root initializer'
require_fixed_string_count "$wasm_module_assembly" 'compile_pooled_strings_initialize_helper' 1 'runtime assembly consumes the canonical pooled-string helper'
module_assembly_code="$(tr -d '[:space:]' < "$wasm_module_assembly")"
if [[ "$module_assembly_code" != *'register_runtime_helper!(PooledStringsInitialize,compile_pooled_strings_initialize_helper);'* ]]; then
  fail 'the pooled-string body owner must compile the registered PooledStringsInitialize helper'
fi
check_no_inline_legacy_includes "$wasm_literal_roots"
check_raw_line_budget "$wasm_literal_roots" 180
check_raw_line_budget "$wasm_module_assembly" 1950
wasm_module_metadata="crates/lila-aot-wasm/src/emit/module_assembly/metadata.rs"
require_file "$wasm_module_metadata"
require_exact_line_count "$wasm_module_assembly" 'mod metadata;' 1 'private completed-function metadata owner'
for metadata_entry in append_function_attribution append_function_name_and_check_budget; do
  require_fixed_string_count "$wasm_module_metadata" "pub(super) fn ${metadata_entry}(" 1 'private completed-function metadata entry'
  require_fixed_string_count "$wasm_module_assembly" "metadata::${metadata_entry}(" 1 'assembly consumes each metadata phase once'
  require_fixed_string_count "$wasm_module_assembly" "fn ${metadata_entry}(" 0 'no parent metadata implementation copy'
done
require_fixed_string_count "$wasm_module_metadata" 'let function_sizes = function_table.summaries();' 1 'metadata derives summaries from the completed function table'
require_fixed_string_count "$wasm_module_metadata" 'module.section(&function_table.name_section());' 1 'function names use the same completed table'
require_fixed_string_count "$wasm_module_metadata" 'function_table.check_budget(budget)?;' 1 'body budgets consume the same completed table'
require_fixed_string_count "$wasm_module_assembly" 'function_table.summaries()' 0 'no second summary producer in assembly'
check_no_inline_legacy_includes "$wasm_module_metadata"
check_raw_line_budget "$wasm_module_metadata" 100
package_main_compiler="$(braced_rust_item_source "$compiled_module_package" '^pub[(]crate[)] fn compile_main[(]')"
package_main_compiler_code="$(printf '%s\n' "$package_main_compiler" | tr -d '[:space:]')"
package_assembly="$(braced_rust_item_source "$compiled_module_package" '^pub[(]crate[)] fn append_to_module[(]')"
package_assembly_code="$(printf '%s\n' "$package_assembly" | tr -d '[:space:]')"
if ! grep -Fq "Main(&'a FinalizedModuleGlobals, PromiseRejectionPolicy)" "$wasm_emit_parent" \
  || ! grep -Fq 'module_package.compile_main(MainFunctionCompilation::new(' "$wasm_module_assembly" \
  || [[ "$package_main_compiler_code" != *"&mutself,compilation:MainFunctionCompilation<'_>,)->Result<(),EmitError>"* ]] \
  || [[ "$package_main_compiler_code" != *'compilation.compile(self.runtime.globals())?;self.main=Some(main);'* ]] \
  || [[ "$package_assembly_code" != *'ifletSome(main)=main{code.push(main);}forfunctioninprogram_functions{code.push(function);}let(functions,code,function_table)=code.finish();'* ]] \
  || ! grep -Fq 'CompiledModulePackage::compile_main;' "$compiled_module_package" \
  || ! grep -Fq 'CompiledModulePackage::append_functions;' "$compiled_module_package"; then
  fail 'main must compile against its own finalized globals and stay in that package until ordered code publication'
fi
compiled_package_state="$(braced_rust_item_source "$compiled_module_package" '^pub[(]crate[)] struct CompiledModulePackage[[:space:]]*[{]')"
require_text_regex_count "$compiled_package_state" '^    main: Option<EmittedFunction>,$' 1 'privately retained compiled main'
require_text_regex_count "$compiled_package_state" '^    program_functions: Vec<EmittedFunction>,$' 1 'privately retained post-main program bodies'
require_text_regex_count "$compiled_package_state" '^[[:space:]]+pub([[:space:]]|[(])' 0 'no public replaceable module-package state'
compiled_package_code="$(tr -d '[:space:]' < "$compiled_module_package")"
if [[ "$compiled_package_code" != *"const_:fn(&mutCompiledModulePackage,MainFunctionCompilation<'_>)->Result<(),EmitError>=CompiledModulePackage::compile_main;"* ]] \
  || [[ "$compiled_package_code" != *'const_:fn(&mutCompiledModulePackage,Vec<EmittedFunction>,Vec<EmittedFunction>)=CompiledModulePackage::append_functions;'* ]]; then
  fail 'compile-time lifecycle signatures must retain main and keep runtime/program body groups separate'
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
  crates/lila-aot-wasm/src/emit/module_assembly.rs \
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
  || grep -Fq 'GlobalSection::new()' "$compiled_module_package"; then
  fail "production GlobalSection construction must stay in the canonical GC ledger encoder: $global_section_constructor_escapes"
fi
global_section_encoder="$(braced_rust_item_source "$wasm_gc_types" '^fn section_after[(]')"
require_text_regex_count "$global_section_encoder" 'GlobalSection::new[(][)]' 1 'sole raw encoder derived from the canonical global ledger'
gc_types_product_source="$(rust_source_without_inline_tests "$wasm_gc_types")"
require_text_regex_count "$gc_types_product_source" 'GlobalSection::new[(][)]' 1 'single production raw-global encoder site'
require_text_regex_count "$global_section_encoder" 'for [(]global_type, init[)] in &self[.]entries\[skip as usize[.][.]\]' 1 'defined globals preserve their actual declaration order'
require_text_regex_count "$global_section_encoder" 'section[.]global[(][*]global_type, init[)];' 1 'encoded globals use canonical type and initializer pairs'
global_section_view="$(braced_rust_item_source "$wasm_gc_types" '^pub[(]crate[)] fn defined_section[(]')"
global_section_view_code="$(printf '%s\n' "$global_section_view" | tr -d '[:space:]')"
if [[ "$global_section_view_code" != *"fndefined_section(&self,imported:u32)->implSection+'_"* ]]; then
  fail 'defined globals must expose only an opaque Section borrowed from their finalized owner'
fi
for global_view_invariant in \
  "structDefinedGlobalSection<'a>{ledger:&'aGlobalLedger,imported:u32,}" \
  "implEncodeforDefinedGlobalSection<'_>" \
  'self.ledger.section_after(self.imported).encode(sink);' \
  "implSectionforDefinedGlobalSection<'_>" \
  'DefinedGlobalSection{ledger:&self.section,imported,}'; do
  if [[ "$global_section_view_code" != *"$global_view_invariant"* ]]; then
    fail "the sealed global Section view must retain its original ledger and import boundary: $global_view_invariant"
  fi
done

for module in array atomics bigint binary_data boolean bootstrap date errors function \
              global_numeric host iterators json math number object proxy reflect \
              standard string symbol typed_array_set uri; do
  require_file "crates/lila-aot-wasm/src/builtins/${module}.rs"
  require_module_decl "$wasm_builtins_mod" "$module"
done

# Standard generator methods and Array/TypedArray iterator creation retain the
# exhaustive flat standard dispatch while their actual bodies have private owners.
for standard_owner_entry in \
  'generator:emit_generator_method_builtin:3:310' \
  'async_generator:emit_async_generator_method_builtin:3:390' \
  'array_iterator_creation:compile_array_iterator_method_builtin:6:110'
do
  standard_owner="${standard_owner_entry%%:*}"
  standard_owner_tail="${standard_owner_entry#*:}"
  standard_entry="${standard_owner_tail%%:*}"
  standard_owner_tail="${standard_owner_tail#*:}"
  standard_consumers="${standard_owner_tail%%:*}"
  standard_budget="${standard_owner_tail#*:}"
  standard_owner_path="crates/lila-aot-wasm/src/builtins/standard/${standard_owner}.rs"
  require_file "$standard_owner_path"
  require_exact_line_count "$wasm_standard_builtins" "mod ${standard_owner};" 1 'private standard helper owner attachment'
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${standard_owner};" "$wasm_standard_builtins"; then
    fail "$wasm_standard_builtins must keep ${standard_owner} private"
  fi
  require_fixed_string_count "$standard_owner_path" "pub(super) fn ${standard_entry}(" 1 'consumed standard helper body'
  require_fixed_string_count "$wasm_standard_builtins" "fn ${standard_entry}(" 0 'no standard parent body copy'
  require_fixed_string_count "$wasm_standard_builtins" "self.${standard_entry}(" "$standard_consumers" 'flat standard dispatch consumers'
  check_no_inline_legacy_includes "$standard_owner_path"
  check_raw_line_budget "$standard_owner_path" "$standard_budget"
done
require_fixed_string_count crates/lila-aot-wasm/src/builtins/standard/generator.rs 'fn emit_generator_resume_call(' 1 'private synchronous generator resume owner'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/standard/generator.rs 'self.emit_generator_resume_call(' 1 'sole synchronous generator resume consumer'
check_raw_line_budget "$wasm_standard_builtins" 3300

# Iterator.from acquisition and wrapper Call share one private consumed owner.
# The leaf measures 560 formatted lines; keep a bounded maintenance margin.
wasm_iterator_from_parent="crates/lila-aot-wasm/src/builtins/iterators.rs"
wasm_iterator_from_owner="crates/lila-aot-wasm/src/builtins/iterators/from.rs"
require_file "$wasm_iterator_from_owner"
require_exact_line_count "$wasm_iterator_from_parent" 'mod from;' 1 'private Iterator.from module declaration'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+from;' "$wasm_iterator_from_parent"; then
  fail "$wasm_iterator_from_parent must keep from private"
fi
for iterator_from_entry in \
  emit_iterator_from_builtin \
  emit_iterator_from_wrapper_next_builtin \
  emit_iterator_from_wrapper_return_builtin
do
  require_fixed_string_count "$wasm_iterator_from_owner" "pub(in crate::builtins) fn ${iterator_from_entry}(" 1 "Iterator.from leaf entry $iterator_from_entry"
  require_fixed_string_count crates/lila-aot-wasm/src/builtins/standard.rs "self.${iterator_from_entry}(function)?" 1 "live Iterator.from dispatch $iterator_from_entry"
done
check_no_inline_legacy_includes "$wasm_iterator_from_owner"
check_raw_line_budget "$wasm_iterator_from_owner" 600

wasm_builtin_bootstrap="crates/lila-aot-wasm/src/builtins/bootstrap.rs"
if ! grep -q 'match builtin\.intrinsic_installer()' "$wasm_builtin_bootstrap"; then
  fail "$wasm_builtin_bootstrap must dispatch through the catalog installer class"
fi
wasm_realm_initialization="crates/lila-aot-wasm/src/builtins/bootstrap/realm_initialization.rs"
require_file "$wasm_realm_initialization"
require_exact_line_count "$wasm_builtin_bootstrap" 'mod realm_initialization;' 1 'private shared Realm initialization helper owner'
require_fixed_string_count "$wasm_builtin_bootstrap" 'fn emit_initialize_realm_intrinsics_inner(' 1 'one ordered Realm initialization algorithm'
require_fixed_string_count "$wasm_realm_initialization" 'self.emit_initialize_realm_intrinsics_inner(' 1 'only the shared helper emits Realm initialization'
require_fixed_string_count "$wasm_builtin_bootstrap" 'self.emit_initialize_realm_intrinsics(' 2 'entry and created Realm consume the same helper'
require_fixed_string_count "$wasm_module_assembly" 'compile_realm_initialize_intrinsics_helper' 1 'registered Realm initialization body'
check_raw_line_budget "$wasm_realm_initialization" 130


# T02's Object, Proxy, Math, Symbol, BigInt, Boolean, Number, Function, Atomics,
# global numeric, URI, Error and JSON
# builtin body boundaries. The exhaustive StandardBuiltinId dispatch remains in
# standard.rs, but family bodies are one-line delegates so unrelated builtin
# work no longer collides with ~11k lines of Object descriptor/prototype
# implementation, the Proxy lifecycle, the Math emitter family, Symbol's
# registry/prototype implementation or BigInt's constructor, fixed-width and
# prototype implementation, Boolean's constructor and prototype receiver logic,
# Number's constructor, predicates and prototype methods, Function's constructor,
# four prototype methods and the typed callable dispatcher, the Error intrinsic
# family, the Atomics integer/wait family, or JSON's parse/stringify/raw-JSON
# wrappers. The two coercing global numeric predicates
# and the six global URI and Annex-B codec wrappers likewise stay out of the
# shared dispatcher.
check_no_inline_legacy_includes "$wasm_standard_builtins"
# Measured after the Atomics extraction: 30,567 raw lines before formatting.
# This margin is dispatch-maintenance headroom; substantive bodies belong in
# family modules.
check_raw_line_budget "$wasm_standard_builtins" 30800

# TypedArray.set owns brand/write admission and alias-aware ordered byte copies.
# The complete GC algorithm measures 695 lines, including private snapshots and
# whole completion cleanup; its three consumed family entries stay bounded.
wasm_typed_array_set="crates/lila-aot-wasm/src/builtins/typed_array_set.rs"
check_raw_line_budget "$wasm_typed_array_set" 730
for typed_set_entry in emit_typed_array_set_builtin emit_native_typed_array_byte_access emit_typed_array_set_byte_copy; do
  require_regex_count "$wasm_typed_array_set" "^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+${typed_set_entry}[[:space:]]*\(" 1 'consumed TypedArray Set/byte owner'
  require_regex_count "$wasm_standard_builtins" "^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+${typed_set_entry}[[:space:]]*\(" 0 'no dispatcher copy of TypedArray Set/bytes'
done
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_typed_array_set_builtin(function)?;' 1 'TypedArray Set fixed dispatch'
require_file crates/lila-engine/tests/aot_typed_array_byte_copy.rs
require_file crates/lila-engine/tests/aot_gc_typed_array_immutable_properties.rs

# Fixed Atomics entries consume the private operation and access proof owners.
# Async waiting has its own private GC queue/result lifecycle, not scalar locals.
wasm_atomics_builtins="crates/lila-aot-wasm/src/builtins/atomics.rs"
wasm_atomics_wait_async="crates/lila-aot-wasm/src/builtins/atomics/wait_async_result.rs"
check_no_inline_legacy_includes "$wasm_atomics_builtins"
require_file "$wasm_atomics_wait_async"
require_exact_line_count "$wasm_atomics_builtins" 'mod wait_async_result;' 1 'private Atomics async-wait owner'
for atomic_proof in AtomicsIntegerOperation PendingAtomicAccess RevalidatedAtomicAccess; do
  require_regex_count "$wasm_atomics_builtins" "^(enum|struct)[[:space:]]+${atomic_proof}[[:space:]]*\{" 1 'private Atomics operation/access authority'
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+(enum|struct)[[:space:]]+${atomic_proof}" "$wasm_atomics_builtins"; then
    fail "$atomic_proof must stay private to its complete Atomics owner"
  fi
done
if grep -Eq 'AtomicsIntegerOperation|PendingAtomicAccess|RevalidatedAtomicAccess' "$wasm_standard_builtins"; then
  fail 'the Standard dispatcher must use fixed Atomics entries, not private access policy'
fi
for atomics_builtin in add and compare_exchange exchange is_lock_free load notify or pause store sub wait wait_async xor; do
  require_regex_count "$wasm_atomics_builtins" "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_atomics_${atomics_builtin}_builtin[[:space:]]*\(" 1 'fixed Atomics entry'
  require_fixed_string_count "$wasm_standard_builtins" "self.emit_atomics_${atomics_builtin}_builtin(function)?" 1 "fixed Atomics ${atomics_builtin} route"
done
require_fixed_string_count "$wasm_atomics_builtins" 'pub(super) const ATOMICS_PUBLICATION_ORDER: [StandardBuiltinId; 14]' 1 'ordered Atomics publication surface'
require_fixed_string_count "$wasm_builtin_bootstrap" 'for builtin in ATOMICS_PUBLICATION_ORDER' 1 'sole shared Realm Atomics publication loop'
for atomics_wait_hook in emit_drain_atomics_wait_async_timeouts emit_poll_atomics_wait_async_timeouts; do
  require_regex_count "$wasm_atomics_wait_async" "^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+${atomics_wait_hook}[[:space:]]*\(" 1 'consumed async-wait checkpoint owner'
  require_regex_count "$wasm_atomics_builtins" "^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+${atomics_wait_hook}[[:space:]]*\(" 0 'no parent async-wait checkpoint copy'
done
require_fixed_string_count crates/lila-aot-wasm/src/emit/body_compilation.rs 'self.emit_drain_atomics_wait_async_timeouts(' 1 'Main event-loop actual async-wait drain consumer'
# Complete Generator, Async and AsyncGenerator phases consume one physical
# classic-loop lifecycle; the prior linear split remains a separate source owner.
wasm_generator_loop="crates/lila-aot-wasm/src/control_flow/generator_loop.rs"
require_file "$wasm_generator_loop"
require_exact_line_count crates/lila-aot-wasm/src/control_flow.rs 'mod generator_loop;' 1 'private ordinary generator loop owner attachment'
require_exact_line_count crates/lila-ir/src/lib.rs 'mod generator_loop_control;' 1 'checked ordinary generator phase domain attachment'
for generator_phase_entry in compile_ordinary_generator_loop compile_ordinary_generator_if; do
  require_fixed_string_count "$wasm_generator_loop" "pub(super) fn ${generator_phase_entry}(" 1 'actual checked ordinary phase compiler'
  require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs "fn ${generator_phase_entry}(" 0 'no ordinary phase implementation in parent'
done
require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs 'self.compile_ordinary_generator_loop(plan,' 2 'ordinary and labelled loop dispatch consumers'
require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs 'self.compile_ordinary_generator_if(plan,' 1 'ordinary conditional dispatch consumer'
require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs 'fn compile_plain_generator_loop(' 0 'retired single-yield ordinary loop implementation'
require_file crates/lila-engine/tests/aot_generator_classic_loops.rs
require_file crates/lila-engine/tests/fixtures/generator_classic_loops/phases_and_branches.js
require_file crates/lila-engine/tests/fixtures/generator_classic_loops/completions_and_environments.js
check_no_inline_legacy_includes "$wasm_generator_loop"
for shared_loop_entry in compile_resumable_generator_loop compile_resumable_generator_loop_phases; do
  require_tree_regex_count crates/lila-aot-wasm/src "^[[:space:]]*fn[[:space:]]+${shared_loop_entry}[[:space:]]*\\(" 1 'sole physical complete classic-loop lifecycle'
done
for complete_loop_protocol in Generator Async AsyncGenerator; do
  require_fixed_string_count "$wasm_generator_loop" "ResumableRegionProtocolIr::${complete_loop_protocol} => FunctionExecutionKind::${complete_loop_protocol}," 1 'closed checked loop protocol projection'
done
require_fixed_string_count "$wasm_generator_loop" 'Some(CheckedAsyncGeneratorEnvironmentOwner::for_loop(plan));' 1 'only the actual opaque loop mints region environment authority'
# Its bounded facade now owns three-protocol regions, source-certified scope
# tokens, resource-head lifetime and retained terminal operand callbacks.
check_raw_line_budget "$wasm_generator_loop" 730
# Ordinary Switch source ranges, one CaseBlock and recursive Empty completion
# have consumed private owners. Runtime V stays in the original invocation's
# real cells, and the generic sequence/Yield dispatch uses that same context.
ir_generator_switch_source="crates/lila-ir/src/generator_switch_source.rs"
ir_generator_switch="crates/lila-ir/src/generator_switch.rs"
ir_generator_switch_lowerer="crates/lila-ir/src/lowering/generator_switch.rs"
ir_resumable_switch_lowerer="crates/lila-ir/src/lowering/resumable_switch.rs"
wasm_generator_switch="crates/lila-aot-wasm/src/control_flow/generator_switch.rs"
wasm_statement_list_value="crates/lila-aot-wasm/src/control_flow/statement_completion.rs"
wasm_resumable_sequence="crates/lila-aot-wasm/src/control_flow/resumable_sequence.rs"
for generator_switch_owner in \
  "$ir_generator_switch_source" "$ir_generator_switch" \
  "$ir_generator_switch_lowerer" "$ir_resumable_switch_lowerer" "$wasm_generator_switch" \
  "$wasm_statement_list_value" "$wasm_resumable_sequence"
do
  require_file "$generator_switch_owner"
  check_no_inline_legacy_includes "$generator_switch_owner"
done
require_exact_line_count "$ir_lib" 'mod generator_switch;' 1 'opaque checked Switch domain attachment'
require_exact_line_count "$ir_lowering" 'mod generator_switch;' 1 'private source Switch lowerer attachment'
require_exact_line_count "$ir_lowering" 'mod resumable_switch;' 1 'private shared complete Switch algorithm attachment'
require_exact_line_count crates/lila-ir/src/lowering_helpers.rs 'mod generator_switch_source;' 1 'private actual Switch source-plan attachment'
require_exact_line_count "$wasm_control_flow" 'mod generator_switch;' 1 'private native Switch owner attachment'
for switch_private_parent in "$ir_lib" "$ir_lowering" "$wasm_control_flow"; do
  if grep -Eq '^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+generator_switch;' "$switch_private_parent"; then
    fail "$switch_private_parent must keep its generator Switch module private"
  fi
done
require_fixed_string_count "$ir_generator_switch_source" 'pub(crate) struct CheckedGeneratorSwitchSource<' 1 'checked actual Switch source owner'
require_fixed_string_count "$ir_generator_switch_source" 'pub(crate) struct CheckedEmptyStatementCompletionSource<' 1 'actual recursive Empty source proof'
require_fixed_string_count "$ir_generator_switch_lowerer" 'pub(super) fn lower_ordinary_generator_switch(' 1 'sole ordinary Switch producer'
require_fixed_string_count crates/lila-ir/src/lowering/switch_statement.rs 'self.lower_ordinary_generator_switch(switch)' 1 'actual ordinary Switch source dispatch'
require_fixed_string_count "$ir_lowering" 'fn lower_ordinary_generator_switch(' 0 'no parent Switch producer copy'
require_fixed_string_count "$ir_generator_switch_lowerer" 'CheckedGeneratorSwitchSource::new(' 1 'lowering consumes the actual checked source plan'
require_fixed_string_count "$ir_generator_switch_source" 'CheckedGeneratorSwitchSource::new(' 1 'function source planner consumes the same checked owner'
require_fixed_string_count "$ir_generator_switch_lowerer" 'self.lower_resumable_switch::<super::resumable_switch::OrdinarySwitch>(switch, states)' 1 'ordinary adapter consumes the sealed shared Switch owner'
require_fixed_string_count "$ir_resumable_switch_lowerer" 'pub(super) trait ResumableSwitchProtocol: sealed::Sealed {' 1 'closed source-to-region Switch protocol'
require_exact_line_count "$ir_resumable_switch_lowerer" 'mod sealed {' 1 'private Switch protocol sealing authority'
require_fixed_string_count "$ir_resumable_switch_lowerer" 'pub(super) fn lower_resumable_switch<P: ResumableSwitchProtocol>(' 1 'sole shared complete Switch lowering algorithm'
require_tree_regex_count crates/lila-ir/src/lowering '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+lower_resumable_switch[[:space:]]*<' 1 'one complete Switch lowering owner at any visibility'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+resumable_switch;' "$ir_lowering" \
  || grep -Eq '^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+sealed' "$ir_resumable_switch_lowerer"; then
  fail 'the shared Switch algorithm and its protocol sealing authority must remain private modules'
fi
require_fixed_string_count "$ir_resumable_switch_lowerer" 'OrdinaryGeneratorSwitchIr::new(' 1 'shared owner consumes the actual opaque ordinary Switch constructor'
require_regex_count "$ir_generator_switch" '^pub struct (OrdinaryGeneratorSwitchIr|OrdinaryGeneratorSwitchCaseIr|EmptyStatementCompletionIr) \{' 3 'closed opaque Switch and original Empty item carriers'
require_regex_count "$ir_generator_switch" '^[[:space:]]*pub[[:space:]]+fn[[:space:]]+new\(' 0 'no unchecked public Switch or Empty constructor'
require_regex_count "$ir_generator_switch" '^[[:space:]]*pub[[:space:]]+(discriminant|cases|value_binding|statement)[[:space:]]*:' 0 'no public carrier fields'
ordinary_switch_protocol_code="$(braced_rust_item_source "$ir_resumable_switch_lowerer" '^impl ResumableSwitchProtocol for OrdinarySwitch')"
require_text_regex_count "$ordinary_switch_protocol_code" 'l\.ordinary_generator_region_depth \+= 1;' 1 'actual nested ordinary region entry in the sealed adapter'
require_text_regex_count "$ordinary_switch_protocol_code" 'l\.ordinary_generator_region_depth -= 1;' 1 'balanced ordinary region exit in the sealed adapter'
require_text_regex_count "$ordinary_switch_protocol_code" 'l\.ordinary_generator_switch_depth \+= 1;' 1 'actual ordinary Switch completion scope entry'
require_text_regex_count "$ordinary_switch_protocol_code" 'l\.ordinary_generator_switch_depth -= 1;' 1 'balanced ordinary Switch completion scope exit'
require_fixed_string_count "$ir_lowering" 'ordinary_generator_loop_depth' 0 'retired loop-only region context spelling'
require_fixed_string_count crates/lila-ir/src/lowering/statement.rs 'CheckedEmptyStatementCompletionSource::from_statement(statement)' 1 'actual Var/Empty/Debugger completion proof'
require_fixed_string_count "$ir_lowering" 'CheckedEmptyStatementCompletionSource::from_declaration(declaration)' 1 'actual declaration completion proof'
require_fixed_string_count "$ir_lowering" 'if self.ordinary_generator_switch_depth > 0' 1 'declaration Empty wrapping consumes actual enclosing complete contexts'
require_fixed_string_count crates/lila-ir/src/lowering/statement.rs 'if self.ordinary_generator_switch_depth > 0' 1 'statement Empty wrapping consumes actual enclosing complete contexts'
require_fixed_string_count "$wasm_generator_switch" 'pub(super) fn compile_ordinary_generator_switch(' 1 'sole native checked Switch compiler'
require_fixed_string_count "$wasm_control_flow" 'fn compile_ordinary_generator_switch(' 0 'no native Switch implementation copy in parent'
require_fixed_string_count "$wasm_control_flow" 'self.compile_ordinary_generator_switch(plan,' 2 'ordinary and labelled native Switch dispatch'
require_fixed_string_count "$wasm_generator_switch" 'self.begin_resumable_switch_statement_list_value(plan, break_target)?;' 1 'whole checked Switch constructs its actual completion context'
require_fixed_string_count "$wasm_generator_switch" 'self.end_generator_statement_list_value();' 1 'actual enclosing completion context restoration'
require_fixed_string_count "$wasm_generator_switch" 'builder.compile_switch_case_match(&value, selector.value(), function);' 1 'native selector strict equality consumes the terminal value inside its retained operand scope'
require_fixed_string_count "$wasm_generator_switch" 'self.compile_resumable_operand_region_then(' 1 'selector terminal match remains inside operand suppression and retained scope'
for shared_switch_entry in compile_resumable_generator_switch compile_resumable_generator_switch_selection_and_bodies; do
  require_tree_regex_count crates/lila-aot-wasm/src "^[[:space:]]*fn[[:space:]]+${shared_switch_entry}[[:space:]]*\\(" 1 'sole physical complete Switch lifecycle'
done
require_fixed_string_count "$wasm_statement_list_value" 'pub(crate) struct GeneratorStatementListValueContext {' 1 'sole native persistent statement value owner'
require_fixed_string_count "$wasm_statement_list_value" 'pub(super) fn compile_resumable_operand_region_then(' 1 'one retained terminal operand scope owner'
require_tree_regex_count crates/lila-aot-wasm/src '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+compile_resumable_operand_region_then[[:space:]]*\(' 1 'no duplicate terminal operand scope lifecycle'
operand_region_scope_code="$(braced_rust_item_source "$wasm_statement_list_value" '^pub[(]super[)] fn compile_resumable_operand_region_then')"
require_text_regex_count "$operand_region_scope_code" '\.and_then\(\|\(\)\| finish\(self, function\)\);' 1 'terminal value is consumed before exact scope restoration'
for operand_scope_restore in \
  'self.binding_scopes = previous_scopes;' \
  'self.statement_list_value_context = previous_context;' \
  'self.environment_depth = previous_environment_depth;'
do
  require_text_regex_count "$operand_region_scope_code" "$(printf '%s\n' "$operand_scope_restore" | sed 's/\./\\./g')" 1 'prior operand scope and completion context restore on success and Err'
done
require_fixed_string_count crates/lila-aot-wasm/src/emit.rs 'pub(crate) statement_list_value_context:' 1 'actual callable builder owns the completion scope'
require_fixed_string_count crates/lila-aot-wasm/src/emit.rs 'statement_list_value_context: None,' 1 'fresh callable has no Switch completion scope'
require_fixed_string_count "$wasm_statement_list_value" 'InvocationFrameSchema::INVOCATION_ENVIRONMENT' 1 'retained values consume original invocation cells'
require_fixed_string_count "$wasm_resumable_sequence" 'self.emit_generator_statement_list_entry(entry_state, function)?;' 1 'actual resumable sequence consumes persistent entry value'
require_fixed_string_count "$wasm_control_flow" 'self.compile_empty_statement_completion(item.statement(), function)?;' 1 'actual recursive Empty completion native dispatch'
require_fixed_string_count "$wasm_control_flow" 'self.emit_retire_generator_statement_list_values(Some(target), function);' 1 'leaving native branches retire private completion cells'
require_fixed_string_count "$wasm_control_flow" 'self.emit_retire_generator_statement_list_values(None, function);' 1 'committed native invocation exit retires private completion cells'
require_fixed_string_count "$wasm_control_flow" '!self.generator_statement_list_branch_retires(target)' 1 'conditional branch cannot bypass leaving-scope retirement'
require_fixed_string_count crates/lila-aot-wasm/src/generator_delegation.rs 'if !self.has_generator_statement_list_value() {' 1 'delegated completion respects the actual active complete StatementList value owner'
require_fixed_string_count crates/lila-ir/src/generator_loop_control/head_bindings.rs 'visit_head_statement_binding(item.statement(), visit);' 1 'actual head bindings delegate the original Empty item'
check_raw_line_budget "$ir_generator_switch_source" 225
check_raw_line_budget "$ir_generator_switch_lowerer" 240
# Ordinary/async/mixed typed adapters, shared selectors/CaseBlock lowering and
# whole CaseBlock resource registration live in this single bounded owner.
check_raw_line_budget "$ir_resumable_switch_lowerer" 800
check_raw_line_budget "$ir_generator_switch" 325
# Actual complete Array/Class prefix state and suspension joins measure956.
check_raw_line_budget crates/lila-ir/src/generator_loop_control.rs 980
# The Switch facade shares selection/fallthrough/CaseBlock cleanup across all
# protocols and whole resource lifetimes. Completion owns their retained V and
# the terminal operand callback, including exact restoration on compilation Err.
check_raw_line_budget "$wasm_generator_switch" 420
check_raw_line_budget "$wasm_statement_list_value" 600
check_raw_line_budget "$wasm_resumable_sequence" 110
require_module_decl "$wasm_control_flow" "async_suspension"
require_file crates/lila-aot-wasm/src/control_flow/async_suspension.rs
check_no_inline_legacy_includes crates/lila-aot-wasm/src/control_flow/async_suspension.rs
check_raw_line_budget crates/lila-aot-wasm/src/control_flow/async_suspension.rs 60
require_file crates/lila-ir/tests/generator_switch_regions.rs
require_file crates/lila-engine/tests/aot_generator_switch_regions.rs
require_file crates/lila-engine/tests/fixtures/generator_switch_regions/switches.js
# Constructor admission, classic For environment creation and activation
# retention share the real declaration-binding visitor, including patterns.
ir_generator_head_bindings="crates/lila-ir/src/generator_loop_control/head_bindings.rs"
require_file "$ir_generator_head_bindings"
require_exact_line_count crates/lila-ir/src/generator_loop_control.rs 'mod head_bindings;' 1 'private actual For head binding owner'
require_exact_line_count crates/lila-ir/src/generator_loop_control.rs 'pub(crate) use head_bindings::visit_lexical_head_statement_bindings;' 1 'shared actual head-binding projection'
require_fixed_string_count "$ir_generator_head_bindings" 'pub(crate) fn visit_lexical_head_statement_bindings(' 1 'sole declaration-binding visitor entry'
require_fixed_string_count crates/lila-ir/src/generator_loop_control.rs 'visit_lexical_head_statement_bindings(' 1 'checked loop constructor consumes actual bindings'
require_fixed_string_count crates/lila-ir/src/lowering/for_lexical_environment.rs 'visit_lexical_head_statement_bindings(' 1 'actual For environment consumes the same visitor'
require_fixed_string_count crates/lila-ir/src/lowering/resumable_for_initializer.rs 'Self::visit_for_head_lexical_bindings(' 1 'shared complete activation retains the same original head bindings'
for classic_for_consumer in for_loop generator_loop; do
  require_fixed_string_count "crates/lila-ir/src/lowering/${classic_for_consumer}.rs" 'self.lower_for_lexical_environment(' 1 'shared analyzed classic For environment consumer'
done
check_no_inline_legacy_includes "$ir_generator_head_bindings"
# Binding patterns, complete resource heads and Empty items share this original
# cell projection; it never visits a child scope's CaseBlock declarations.
check_raw_line_budget "$ir_generator_head_bindings" 140
wasm_identifier_reference_parent="crates/lila-aot-wasm/src/environments/environment_reference.rs"
wasm_identifier_reference_capture="crates/lila-aot-wasm/src/environments/environment_reference/captured_identifier_reference.rs"
wasm_global_identifier_read="crates/lila-aot-wasm/src/environments/environment_reference/global_identifier_read.rs"
require_file "$wasm_global_identifier_read"
require_exact_line_count "$wasm_identifier_reference_parent" 'mod global_identifier_read;' 1 'private complete global Get helper owner'
require_fixed_string_count "$wasm_global_identifier_read" 'pub(crate) fn emit_global_identifier_read(' 1 'sole consumed global Get facade'
require_fixed_string_count crates/lila-aot-wasm/src/environments.rs 'fn emit_global_identifier_read(' 0 'no parent inline global Get copy'
require_fixed_string_count "$wasm_global_identifier_read" '.emit_resolve_environment_identifier_from_record(' 1 'shared real ResolveBinding owner'
require_fixed_string_count "$wasm_global_identifier_read" 'self.emit_environment_identifier_get(' 1 'shared real GetBindingValue owner'
require_fixed_string_count "$wasm_global_identifier_read" 'self.release_environment_identifier_reference(' 1 'same owned Reference released'
check_no_inline_legacy_includes "$wasm_global_identifier_read"
check_raw_line_budget "$wasm_global_identifier_read" 180
wasm_identifier_reference_consumer="crates/lila-aot-wasm/src/expressions/environment_identifier.rs"
require_file "$wasm_identifier_reference_capture"
require_exact_line_count "$wasm_identifier_reference_parent" 'mod captured_identifier_reference;' 1 'private native Reference transport attachment'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+captured_identifier_reference;' "$wasm_identifier_reference_parent"; then
  fail 'native Reference transport must remain private to its Environment Reference owner'
fi
for reference_transport_entry in emit_capture_identifier_reference emit_take_captured_identifier_reference emit_release_captured_identifier_reference; do
  require_fixed_string_count "$wasm_identifier_reference_capture" "pub(crate) fn ${reference_transport_entry}(" 1 'consumed native Reference transport entry'
  require_fixed_string_count "$wasm_identifier_reference_consumer" "self.${reference_transport_entry}(" 1 'closed Environment Identifier transport dispatch'
  require_fixed_string_count "$wasm_identifier_reference_parent" "fn ${reference_transport_entry}(" 0 'no parent native Reference transport copy'
done
require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs 'self.emit_retire_abandoned_identifier_references(function);' 4 'committed Return/caught Throw and whole async-generator completion dispatch retire original References'
require_fixed_string_count crates/lila-aot-wasm/src/control_flow.rs 'self.emit_retire_abandoned_identifier_references_if_throw(function);' 1 'actual uncaught Throw retirement at completion exit'
require_file crates/lila-engine/tests/aot_generator_identifier_reference.rs
require_file crates/lila-engine/tests/fixtures/generator_staged_operands/identifier_references.js
check_no_inline_legacy_includes "$wasm_identifier_reference_capture"
# The same field10 record now admits Generator/Async and checked mixed owners,
# retaining it through normal suspension and retiring only committed abrupt paths.
check_raw_line_budget "$wasm_identifier_reference_capture" 590
require_fixed_string_count crates/lila-aot-wasm/src/builtins/promise.rs 'self.emit_poll_atomics_wait_async_timeouts(' 1 'Promise checkpoint actual async-wait poll consumer'
# Current formatted owners: 1,187 operation lines and 367 queue/result lines.
check_raw_line_budget "$wasm_atomics_builtins" 1250
check_raw_line_budget "$wasm_atomics_wait_async" 400
require_file crates/lila-engine/tests/aot_gc_binary_data_entries.rs
require_fixed_string_count crates/lila-engine/tests/aot_gc_binary_data_entries.rs 'fn gc_atomics_keep_whole_abrupts_finite_waits_and_called_realms()' 1 'finite Atomics GC semantic control'

# Boolean construction consumes GetPrototypeFromConstructor; the private closed
# prototype policy is shared only by the two fixed ToString/ValueOf wrappers.
wasm_boolean_builtins="crates/lila-aot-wasm/src/builtins/boolean.rs"
check_no_inline_legacy_includes "$wasm_boolean_builtins"
require_regex_count "$wasm_boolean_builtins" '^enum BooleanPrototypeOperation \{' 1 'private Boolean prototype domain'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+BooleanPrototypeOperation' "$wasm_boolean_builtins"   || grep -Eq 'BooleanPrototypeOperation|emit_boolean_prototype_builtin\(' "$wasm_standard_builtins"; then
  fail 'Boolean prototype policy must stay inside its native owner'
fi
for boolean_entry in constructor prototype_to_string prototype_value_of; do
  require_regex_count "$wasm_boolean_builtins" "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_boolean_${boolean_entry}_builtin[[:space:]]*\(" 1 'fixed Boolean entry'
  require_fixed_string_count "$wasm_standard_builtins" "self.emit_boolean_${boolean_entry}_builtin(function)?" 1 'fixed Boolean dispatch'
done
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
# The private sumPrecise owner keeps the exact arithmetic and iterator lifecycle
# together; its state, limb operation and completed witness cannot escape into
# the family dispatcher.
wasm_math_sum_precise="crates/lila-aot-wasm/src/builtins/math/sum_precise.rs"
require_file "$wasm_math_sum_precise"
check_no_inline_legacy_includes "$wasm_math_sum_precise"
require_exact_line_count "$wasm_math_builtins" 'mod sum_precise;' 1 'private Math.sumPrecise owner'
require_fixed_string_count "$wasm_math_builtins" 'self.emit_runtime_math_sum_precise(' 1 'Math.sumPrecise semantic consumer'
require_fixed_string_count "$wasm_math_sum_precise" 'pub(super) fn emit_runtime_math_sum_precise(' 1 'Math.sumPrecise semantic entry'
require_fixed_string_count "$wasm_math_sum_precise" 'pub(super) fn' 1 'sole parent-visible Math.sumPrecise entry'
require_fixed_string_count "$wasm_math_sum_precise" 'StandardBuiltinId::' 0 'Math.sumPrecise family-domain isolation'
if grep -Eq '^pub|pub\(crate\)' "$wasm_math_sum_precise"; then
  fail "$wasm_math_sum_precise must keep its arithmetic domains and lifecycle private"
fi
for sum_precise_policy in \
  MATH_SUM_PRECISE \
  MathSumPreciseState \
  MathSumPreciseLimbOperation \
  MathSumPreciseAccumulator \
  CompletedMathSumPreciseReduction; do
  sum_precise_policy_files="$(grep -RFl --include='*.rs' "$sum_precise_policy" crates/lila-aot-wasm/src || true)"
  if [ "$sum_precise_policy_files" != "$wasm_math_sum_precise" ]; then
    fail "$sum_precise_policy must remain owned only by $wasm_math_sum_precise"
  fi
done
# Measured after the complete sumPrecise lifecycle extraction: 1,625 parent
# lines and 1,034 child lines. The existing parent budget is unchanged; the
# child margin permits narrow maintenance of the exact reduction owner.
check_raw_line_budget "$wasm_math_builtins" 2430
check_raw_line_budget "$wasm_math_sum_precise" 1050

wasm_number_builtins="crates/lila-aot-wasm/src/builtins/number.rs"
check_no_inline_legacy_includes "$wasm_number_builtins"
for number_domain in NumberPredicate NumberPrototypeOperation; do
  require_regex_count "$wasm_number_builtins" "^enum ${number_domain} \{" 1 'private closed Number policy owner'
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+${number_domain}" "$wasm_number_builtins"; then
    fail 'Number policy domains must remain private to their actual native owner'
  fi
done
if grep -Eq 'NumberPredicate|NumberPrototypeOperation|emit_number_predicate\(|emit_number_prototype_builtin\(' "$wasm_standard_builtins"; then
  fail 'the Standard dispatcher must use fixed Number entries'
fi
for number_internal_entry in emit_number_predicate emit_number_prototype_builtin; do
  require_fixed_string_count "$wasm_number_builtins" "fn ${number_internal_entry}(" 1 'private Number algorithm owner'
done
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
require_regex_count "$wasm_function_builtins" '^enum FunctionBuiltin \{' 1 'private closed Function builtin family'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+FunctionBuiltin' "$wasm_function_builtins"; then
  fail 'FunctionBuiltin must remain private'
fi
if grep -Eq 'FunctionBuiltin|emit_function_builtin\(' "$wasm_standard_builtins"; then
  fail 'the Standard dispatcher must use fixed Function entries'
fi
require_fixed_string_count "$wasm_function_builtins" 'fn emit_function_builtin(' 1 'private Function algorithm owner'
# Bound invocation uses the typed callable dispatch/record owner. Its real
# call, construct, new.target and root fixtures below remain required.
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
wasm_date_constructor="crates/lila-aot-wasm/src/builtins/date/constructor.rs"
wasm_date_components="crates/lila-aot-wasm/src/builtins/date/components.rs"
wasm_date_local_string="crates/lila-aot-wasm/src/builtins/date/local_string.rs"
for date_owner in "$wasm_date_constructor" "$wasm_date_components" "$wasm_date_local_string"
do
  require_file "$date_owner"
  check_no_inline_legacy_includes "$date_owner"
done
check_no_inline_legacy_includes "$wasm_date_builtins"
for date_child in constructor components zone
do
  require_exact_line_count "$wasm_date_builtins" "mod $date_child;" 1 'private Date child declaration'
done
for date_domain in DateTimeBasis DateComponentGetter DateComponentSetter
do
  require_fixed_string_count "$wasm_date_components" "enum $date_domain {" 1 'closed Date operation owner'
done
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' "$wasm_date_components" "$wasm_date_constructor"; then
  fail 'Date component and construction decisions must keep exhaustive closed domains'
fi
require_fixed_string_count "$wasm_date_components" 'fn emit_date_component_getter(' 1 'Date getter emitter'
require_fixed_string_count "$wasm_date_components" 'fn emit_date_component_setter(' 1 'Date setter emitter'
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_date_component_getter(' 18 'all local/UTC/legacy getter delegates'
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_date_component_setter(' 15 'all local/UTC/legacy setter delegates'
require_fixed_string_count "$wasm_date_constructor" 'fn emit_date_constructor(' 1 'Date construction owner'
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_date_constructor(function)?;' 1 'Date constructor semantic delegate'
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_date_utc(function)?;' 1 'Date.UTC semantic delegate'
require_fixed_string_count "$wasm_standard_builtins" 'self.emit_date_set_time(function)?;' 1 'Date setTime semantic delegate'
require_fixed_string_count "$wasm_date_builtins" 'fn emit_date_store_clip(' 1 'sole typed Date setter storage owner'
require_fixed_string_count "$wasm_date_constructor" 'self.emit_date_store_clip(' 1 'setTime actual storage caller'
require_fixed_string_count "$wasm_date_components" 'self.emit_date_store_clip(' 1 'component actual storage caller'
require_fixed_string_count "$wasm_date_constructor" 'schema.struct_type::<DateObject>().construct(' 1 'sole concrete GC Date allocation owner'
date_allocation_body="$(braced_rust_item_source "$wasm_date_constructor" '^fn[[:space:]]+emit_date_alloc_with_new_target[[:space:]]*[(]')"
require_text_regex_count "$date_allocation_body" 'clip: &CompletedDateClipLocals,' 1 'sole Date allocator requires completed TimeClip'
date_number_result_body="$(sed -n '/fn emit_date_clip_number_result(/,$p' "$wasm_date_constructor")"
require_text_regex_count "$date_number_result_body" 'clip: &CompletedDateClipLocals,' 1 'Date Number result borrows completed TimeClip'
require_fixed_string_count "$wasm_date_builtins" 'clip: &CompletedDateClipLocals' 1 'Date writer requires completed TimeClip'
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
require_regex_count \
  "$wasm_date_local_string" \
  '^[[:space:]]*fn[[:space:]]+emit_date_time_value_from_source[[:space:]]*[(]' \
  1 \
  'sole private Date time-value source dispatcher'
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
  'self.emit_date_current_time_payload(bits, function)?;' \
  1 \
  'unchanged Date.now semantic delegate'
require_fixed_string_count \
  "$wasm_date_constructor" \
  'self.emit_date_current_time_payload(bits, function)?;' \
  1 \
  'Date constructor current-time semantic delegate'
require_regex_count \
  "$wasm_date_local_string" \
  '^enum[[:space:]]+DateLocalStringFormat[[:space:]]*\{' \
  1 \
  'private Date local-string format owner'
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
require_fixed_string_count "$wasm_date_constructor" 'self.emit_date_function_call(function)?;' 1 \
  'Date function-call local-string delegate'
for date_local_string_call in \
  'self.emit_date_to_date_string(function)?;' \
  'self.emit_date_to_time_string(function)?;' \
  'self.emit_date_to_string(function)?;'
do
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "$date_local_string_call" \
    1 \
    'Date local-string semantic delegate'
done
for date_locale_format in Date Time DateAndTime
do
  require_fixed_string_count \
    "$wasm_standard_builtins" \
    "self.emit_date_to_locale_string(DateLocaleFormat::$date_locale_format, function)?;" \
    1 \
    "$date_locale_format Date locale-method delegate"
done
# GC construction now retains typed prototype and completion owners (317 lines).
# Keep the existing parent/component/string bounds and a narrow constructor margin.
check_raw_line_budget "$wasm_date_builtins" 1650
check_raw_line_budget "$wasm_date_constructor" 350
check_raw_line_budget "$wasm_date_components" 390
check_raw_line_budget "$wasm_date_local_string" 480

wasm_bigint_builtins="crates/lila-aot-wasm/src/builtins/bigint.rs"
wasm_bigint_radix_formatting="crates/lila-aot-wasm/src/builtins/bigint/radix_formatting.rs"
wasm_numeric_operations="crates/lila-aot-wasm/src/operations.rs"
wasm_emit="crates/lila-aot-wasm/src/emit.rs"
wasm_host_builtins="crates/lila-aot-wasm/src/builtins/host.rs"
wasm_host_detach_array_buffer="crates/lila-aot-wasm/src/builtins/host/detach_array_buffer.rs"
require_file "$wasm_host_detach_array_buffer"
require_exact_line_count "$wasm_host_builtins" 'mod detach_array_buffer;' 1 \
  'private host ArrayBuffer detach module declaration'
require_fixed_string_count "$wasm_host_detach_array_buffer" \
  'fn compile_host_detach_array_buffer_builtin(' 1 'host ArrayBuffer detach owner'
require_fixed_string_count "$wasm_host_builtins" \
  'fn compile_host_detach_array_buffer_builtin(' 0 'no parent host ArrayBuffer detach copy'
check_raw_line_budget "$wasm_host_detach_array_buffer" 50
check_no_inline_legacy_includes "$wasm_host_builtins"
# Created globals use the shared completed Realm bootstrap; the retired private
# WeakRef installer cannot provide a weak facility through ordinary GC fields.
require_file crates/lila-aot-wasm/src/builtins/weak_unavailable.rs
require_module_decl crates/lila-aot-wasm/src/builtins/mod.rs weak_unavailable
check_no_inline_legacy_includes "$wasm_bigint_builtins"
check_no_inline_legacy_includes "$wasm_bigint_radix_formatting"
for bigint_domain in BigIntFixedWidthOperation BigIntPrototypeOperation; do
  require_regex_count "$wasm_bigint_builtins" "^enum ${bigint_domain} \{" 1 'private closed BigInt policy owner'
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+${bigint_domain}" "$wasm_bigint_builtins"; then
    fail 'BigInt native policy domains must remain private'
  fi
done
if grep -Eq 'BigIntFixedWidthOperation|BigIntPrototypeOperation|emit_bigint_builtin\(' "$wasm_standard_builtins"; then
  fail 'the Standard dispatcher must use fixed BigInt entries'
fi
require_exact_line_count "$wasm_bigint_builtins" 'mod radix_formatting;' 1 'private BigInt radix owner'
require_regex_count "$wasm_bigint_radix_formatting" '^struct PreparedBigIntRadix\(I64Local\);$' 1 'private completed scalar radix'
require_fixed_string_count "$wasm_bigint_radix_formatting" 'pub(super) fn emit_bigint_radix_string_result(' 1 'radix semantic wrapper'
if grep -Eq 'PreparedBigIntRadix|radix_formatting::' "$wasm_bigint_builtins"; then
  fail 'BigInt parent must consume the radix wrapper without constructing its private proof'
fi

# Numeric helpers carry whole input and the actual nullable caller Environment
# through the same registry row used by both helper bodies and call adapters.
require_file crates/lila-aot-wasm/src/runtime_helpers.rs
for numeric_helper_arguments in ValueToNumberArguments ValueToNumericArguments; do
  require_fixed_string_count crates/lila-aot-wasm/src/runtime_helpers.rs "$numeric_helper_arguments" 1 'registered typed numeric helper argument owner'
done

# BigInt callable publication uses the shared retained-Realm function factory.
# The former per-error raw header slots are not part of the GC representation.

# Measured after the prepared-radix lifecycle extraction: 807 parent lines and
# 94 child lines. The narrow margins are for maintenance of each owner.
# Measured after closing the six fixed entries: 855 raw lines. The narrow
# margin is for maintenance of this family, not adjacent builtin work.
check_raw_line_budget "$wasm_bigint_builtins" 875
check_raw_line_budget "$wasm_bigint_radix_formatting" 120
# The parseFloat decimal-prefix grammar and its compiler entry share a private
# owner; only the existing product compiler entry is crate-visible.
wasm_host_parse_float="crates/lila-aot-wasm/src/builtins/host/parse_float.rs"
require_file "$wasm_host_parse_float"
check_no_inline_legacy_includes "$wasm_host_parse_float"
require_exact_line_count "$wasm_host_builtins" 'mod parse_float;' 1 'private parseFloat owner'
require_fixed_string_count "$wasm_host_parse_float" 'pub(crate) fn compile_host_parse_float_builtin(' 1 'parseFloat product compiler entry'
require_fixed_string_count "$wasm_host_parse_float" 'pub(crate) fn' 1 'sole crate-visible parseFloat entry'
require_fixed_string_count "$wasm_host_parse_float" 'fn emit_host_decimal_digits(' 1 'private parseFloat digit grammar owner'
require_fixed_string_count "$wasm_host_builtins" 'fn emit_host_decimal_digits(' 0 'parseFloat grammar isolation'
require_fixed_string_count "$wasm_host_builtins" 'compile_host_parse_float_builtin' 0 'parseFloat compiler ownership'
# Measured after the complete parseFloat extraction: 8,634 parent lines and
# 402 child lines. Existing host and WeakRef budgets are unchanged.
check_raw_line_budget "$wasm_host_builtins" 9000
check_raw_line_budget "$wasm_host_parse_float" 430

wasm_intl_locale="crates/lila-aot-wasm/src/builtins/intl.rs"
wasm_intl_locale_construction="crates/lila-aot-wasm/src/builtins/intl/construction_lifecycle.rs"
require_file "$wasm_intl_locale"
require_file "$wasm_intl_locale_construction"
check_no_inline_legacy_includes "$wasm_intl_locale"
check_no_inline_legacy_includes "$wasm_intl_locale_construction"
require_exact_line_count "$wasm_intl_locale" 'mod construction_lifecycle;' 1 'private Intl.Locale lifecycle attachment'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+construction_lifecycle;' "$wasm_intl_locale"; then
  fail "$wasm_intl_locale must keep construction_lifecycle private"
fi
# The reserved header and initialized branded record are consuming GC states.
# Their tuple fields remain private to the child; native entry wrappers do not
# acquire a constructor-table classification or a raw heap address.
require_exact_line_count "$wasm_intl_locale_construction" \
  'pub(super) struct ReservedIntlLocaleObjectLocal(GcLocal<OrdinaryObject>);' 1 'reserved Locale GC header proof'
require_exact_line_count "$wasm_intl_locale_construction" \
  'pub(super) struct InitializedIntlLocaleObjectLocal(GcLocal<IntlLocaleObject>);' 1 'initialized Locale GC record proof'
if grep -Eq '^#\[derive\([^]]*(Clone|Copy)' "$wasm_intl_locale_construction"; then
  fail "$wasm_intl_locale_construction must not clone or copy consuming lifecycle states"
fi
for lifecycle_transition in emit_reserve_intl_locale_object emit_initialize_intl_locale_object emit_publish_intl_locale_object; do
  require_regex_count "$wasm_intl_locale_construction" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${lifecycle_transition}[[:space:]]*\(" 1 'Locale lifecycle transition owner'
  require_fixed_string_count "$wasm_intl_locale" ".${lifecycle_transition}(" 1 'Locale constructor lifecycle consumer'
done
require_fixed_string_count "$wasm_intl_locale_construction" \
  'reserved: ReservedIntlLocaleObjectLocal,' 1 'consuming Locale initialization input'
require_fixed_string_count "$wasm_intl_locale_construction" \
  'initialized: InitializedIntlLocaleObjectLocal,' 1 'consuming Locale publication input'
require_exact_line_count "$wasm_intl_locale" 'enum LocaleStringSlot {' 1 'private Locale string-slot domain'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+LocaleStringSlot' "$wasm_intl_locale"; then
  fail "$wasm_intl_locale must keep LocaleStringSlot private"
fi
require_regex_count "$wasm_intl_locale" \
  '^[[:space:]]*fn[[:space:]]+emit_intl_locale_string_getter[[:space:]]*\(' 1 'private typed Locale string getter'
for intl_locale_entry in language_getter:Language script_getter:Script region_getter:Region base_name_getter:BaseName to_string:Tag; do
  intl_locale_method="${intl_locale_entry%%:*}"
  intl_locale_slot="${intl_locale_entry#*:}"
  require_regex_count "$wasm_intl_locale" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_intl_locale_${intl_locale_method}_builtin[[:space:]]*\(" 1 'fixed Locale string entry'
  require_fixed_string_count "$wasm_intl_locale" \
    "self.emit_intl_locale_string_getter(LocaleStringSlot::${intl_locale_slot}," 1 'closed Locale string-slot selection'
  require_fixed_string_count "$wasm_standard_builtins" \
    "self.emit_intl_locale_${intl_locale_method}_builtin(function)?;" 1 'fixed standard Locale string route'
done
if grep -Eq 'LocaleStringSlot|emit_intl_locale_string_getter\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Locale string entries"
fi
# MAIN129's actual GC owners measure 655 parent and 88 lifecycle raw lines.
# These caps retain the parent/child boundary with a narrow maintenance margin.
check_raw_line_budget "$wasm_intl_locale" 700
check_raw_line_budget "$wasm_intl_locale_construction" 100

wasm_intl_date_time_format="crates/lila-aot-wasm/src/builtins/intl_datetimeformat.rs"
wasm_intl_date_time_format_construction="crates/lila-aot-wasm/src/builtins/intl_datetimeformat/construction_lifecycle.rs"
wasm_intl_date_time_format_initialization="crates/lila-aot-wasm/src/builtins/intl_datetimeformat/initialization.rs"
wasm_intl_date_time_format_time_zone="crates/lila-aot-wasm/src/builtins/intl_datetimeformat/time_zone.rs"
require_file "$wasm_intl_date_time_format"
check_no_inline_legacy_includes "$wasm_intl_date_time_format"
# Pattern data and partition algorithms belong to lila-intl. The AOT shell
# owns observations, typed GC inputs and current-Realm result records.
for intl_dtf_leaf in construction_lifecycle initialization provider_input provider_render provider_wire time_zone zoned_locale; do
  intl_dtf_leaf_path="crates/lila-aot-wasm/src/builtins/intl_datetimeformat/${intl_dtf_leaf}.rs"
  require_file "$intl_dtf_leaf_path"
  require_exact_line_count "$wasm_intl_date_time_format" "mod ${intl_dtf_leaf};" 1 'private DateTimeFormat child attachment'
  check_no_inline_legacy_includes "$intl_dtf_leaf_path"
done
require_module_decl "crates/lila-intl/src/lib.rs" "datetime"
require_module_decl "crates/lila-intl/src/lib.rs" "datetime_protocol"
require_module_decl "crates/lila-intl/src/provider.rs" "datetime"
require_module_decl "crates/lila-engine/src/lib.rs" "intl_datetime_host"
require_tree_regex_count crates/lila-aot-wasm/src \
  '(struct[[:space:]]+DtfComponentLocals|enum[[:space:]]+DtfRangePattern|enum[[:space:]]+IntlDtfRelevantExtensionKey|const[[:space:]]+INTL_DTF_(MONTHS|WEEKDAYS)_)' 0 \
  'retired DateTimeFormat pattern and locale authority in the Wasm emitter'
intl_time_zone_domain="crates/lila-intl/src/time_zone.rs"
require_file "$intl_time_zone_domain"
require_exact_line_count "$intl_time_zone_domain" 'pub enum TimeZoneNameStyle {' 1 'shared time-zone-name style domain'
require_tree_regex_count crates/lila-aot-wasm/src 'enum[[:space:]]+TimeZoneNameStyle' 0 'duplicate Wasm time-zone-name style domain'
require_fixed_string_count "$wasm_intl_date_time_format_initialization" \
  'pub(super) zone_name: GcI32DomainLocal<Option<TimeZoneNameStyle>>,' 1 'typed DateTimeFormat time-zone-name option'
require_exact_line_count "$intl_time_zone_domain" 'pub struct FixedTimeZoneOffset(i32);' 1 'validated private fixed-offset representation'
require_exact_line_count "$wasm_intl_date_time_format_time_zone" \
  'pub(super) struct ResolvedDtfTimeZone {' 1 'family-owned completed DateTimeFormat zone'
if grep -Eq '^pub(\(crate\)|\(in [^)]*\))?[[:space:]]+struct[[:space:]]+ResolvedDtfTimeZone' "$wasm_intl_date_time_format_time_zone"; then
  fail "$wasm_intl_date_time_format_time_zone must keep ResolvedDtfTimeZone inside the DateTimeFormat family"
fi
require_fixed_string_count "$wasm_intl_date_time_format_time_zone" \
  'pub(super) kind: GcI32DomainLocal<TimeZoneKind>,' 1 'closed named/fixed DateTimeFormat zone domain'
require_tree_regex_count crates/lila-aot-wasm/src \
  '(struct[[:space:]]+(TzOffsetMinutes|IntlDtfNamedZone)|const[[:space:]]+INTL_DTF_NAMED_ZONES)' 0 \
  'retired constant-offset named-zone authority'
require_exact_line_count "$wasm_intl_date_time_format" 'enum DtfReceiverOperation {' 1 'private DateTimeFormat receiver domain'
require_regex_count "$wasm_intl_date_time_format" \
  '^[[:space:]]*fn[[:space:]]+emit_dtf_record_from_receiver[[:space:]]*\(' 1 'private branded GC DateTimeFormat receiver admission'
# Initialization consumes the reserved GC header. Its sibling needs the
# private qualified import and family-visible header field; it publishes the
# completed IntlDateTimeFormatObject directly through whole Completion.
require_exact_line_count "$wasm_intl_date_time_format_construction" \
  'pub(super) struct ReservedIntlDateTimeFormatObjectLocal(pub(super) GcLocal<OrdinaryObject>);' 1 'family-owned DateTimeFormat reserved GC header'
require_exact_line_count "$wasm_intl_date_time_format_initialization" \
  'use construction_lifecycle::ReservedIntlDateTimeFormatObjectLocal;' 1 'private initialization lifecycle import'
if grep -Eq '^[[:space:]]*pub(\([^)]*\))?[[:space:]]+use.*construction_lifecycle' \
  "$wasm_intl_date_time_format" "$wasm_intl_date_time_format_initialization"; then
  fail 'DateTimeFormat reserved lifecycle state must not be re-exported'
fi
if grep -Eq '^#\[derive\([^]]*(Clone|Copy)' "$wasm_intl_date_time_format_construction"; then
  fail "$wasm_intl_date_time_format_construction must not clone or copy its reserved header"
fi
for lifecycle_transition in emit_reserve_intl_date_time_format_object emit_reserve_intrinsic_date_time_format_object; do
  require_regex_count "$wasm_intl_date_time_format_construction" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${lifecycle_transition}[[:space:]]*\(" 1 'DateTimeFormat header reservation owner'
  require_fixed_string_count "$wasm_intl_date_time_format_initialization" ".${lifecycle_transition}(" 1 'DateTimeFormat purpose-specific reservation'
done
require_regex_count "$wasm_intl_date_time_format_initialization" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_dtf_complete_initialization[[:space:]]*\(' 1 'consuming DateTimeFormat initialization owner'
require_regex_count "$wasm_intl_date_time_format_initialization" \
  '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_intl_create_date_time_format[[:space:]]*\(' 1 'whole Completion DateTimeFormat creation entry'
require_fixed_string_count "$wasm_intl_date_time_format_initialization" \
  'schema.struct_type::<IntlDateTimeFormatObject>().construct(' 1 'branded GC DateTimeFormat record publication'
# After the announced formatting overlay: parent 510, initialization 895,
# lifecycle 84 raw lines. Typed initialization now owns the complete GC recipe.
check_raw_line_budget "$wasm_intl_date_time_format" 550
check_raw_line_budget "$wasm_intl_date_time_format_initialization" 925
check_raw_line_budget "$wasm_intl_date_time_format_construction" 100
require_module_decl "$wasm_date_builtins" "locale_string"
check_raw_line_budget "crates/lila-aot-wasm/src/builtins/date/locale_string.rs" 115
# Observable reservation/order/Realm behavior stays covered by real Engine
# and CLI controls; this source gate does not execute those controls.
for intl_witness in \
  crates/lila-engine/tests/aot_intl_locale_constructor.rs \
  crates/lila-engine/tests/aot_intl_locale_likely_subtags.rs \
  crates/lila-engine/tests/aot_intl_datetime_provider.rs \
  crates/lila-engine/tests/aot_intl_datetime_range_endpoints.rs \
  crates/lila-cli/tests/cli/intl.rs \
  crates/lila-cli/tests/fixtures/wasm_intl_locale_construction_order.js \
  crates/lila-cli/tests/fixtures/wasm_intl_date_time_format_construction_order.js; do
  require_file "$intl_witness"
done
for intl_cli_witness in \
  run_wasm_intl_locale_construction_order_fixture_succeeds \
  run_wasm_intl_date_time_format_construction_order_fixture_succeeds \
  run_wasm_intl_canonical_locale_tag_roles_fixture_succeeds; do
  require_regex_count crates/lila-cli/tests/cli/intl.rs \
    "^fn ${intl_cli_witness}[[:space:]]*\(" 1 'real CLI Intl ownership/order witness'
done

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
# The whole UTF-16 codec implementation now lives here (865 formatted lines),
# including validation and completion cleanup rather than six raw wrappers.
check_raw_line_budget "$wasm_uri_builtins" 900
require_file crates/lila-engine/tests/aot_gc_uri_entries.rs

wasm_error_builtins="crates/lila-aot-wasm/src/builtins/errors.rs"
require_file "$wasm_error_builtins"
require_regex_count "$wasm_error_builtins" '^enum ErrorBuiltin[[:space:]]*\{' 1 'private closed Error-family domain'
require_regex_count "$wasm_error_builtins" '^[[:space:]]*fn emit_error_builtin\(' 1 'private Error-family compiler'
error_dispatch_body="$(braced_rust_item_source "$wasm_error_builtins" '^fn[[:space:]]+emit_error_builtin[[:space:]]*[(]')"
require_text_regex_count "$error_dispatch_body" '^[[:space:]]*match builtin[[:space:]]*\{' 1 'exhaustive Error-family dispatch'
if grep -Eq '^[[:space:]]*_ =>' <<<"$error_dispatch_body"; then
  fail 'Error-family operation and constructor-kind dispatch must remain exhaustive'
fi
if grep -Eq 'ErrorBuiltin|NativeErrorKind|self\.emit_error_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must use fixed Error-family operations instead of the raw policy"
fi
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
  require_regex_count "$wasm_error_builtins" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${error_wrapper}[[:space:]]*\(" \
    1 "fixed Error-family entry $error_wrapper"
  require_fixed_string_count "$wasm_standard_builtins" "self.${error_wrapper}(function)?" 1 "standard call to fixed Error-family entry $error_wrapper"
done
for error_child in aggregate_error_preparation constructor promise_any prototype_to_string runtime_error; do
  error_child_source="crates/lila-aot-wasm/src/builtins/errors/${error_child}.rs"
  require_file "$error_child_source"
  require_exact_line_count "$wasm_error_builtins" "mod ${error_child};" 1 'private Error algorithm child'
  check_no_inline_legacy_includes "$error_child_source"
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${error_child};" "$wasm_error_builtins"; then
    fail "$wasm_error_builtins must keep ${error_child} private"
  fi
done
check_no_inline_legacy_includes "$wasm_error_builtins"
wasm_aggregate_error_preparation="crates/lila-aot-wasm/src/builtins/errors/aggregate_error_preparation.rs"
wasm_error_constructor="crates/lila-aot-wasm/src/builtins/errors/constructor.rs"
wasm_promise_any_error="crates/lila-aot-wasm/src/builtins/errors/promise_any.rs"
require_tree_regex_count crates/lila-aot-wasm/src 'aggregate_error_preparation::' 0 'AggregateError preparation imports or re-exports'
require_regex_count "$wasm_aggregate_error_preparation" \
  '^pub\(super\) struct PreparedAggregateErrorLocal[(]' 1 'opaque prepared AggregateError owner'
require_fixed_string_count "$wasm_error_builtins" 'PreparedAggregateErrorLocal' 0 'prepared AggregateError parent names'
for aggregate_error_preparation_method in emit_prepare_aggregate_error_instance emit_finish_aggregate_error_instance emit_aggregate_error_iterable_to_list; do
  require_regex_count "$wasm_aggregate_error_preparation" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${aggregate_error_preparation_method}[[:space:]]*\(" \
    1 "$aggregate_error_preparation_method private-child owner"
done
# The prepared object is consumed only after message/cause preparation and the
# actual shared IteratorToList walk. Promise.any has its own retained-Realm path.
aggregate_error_finish_body="$(braced_rust_item_source "$wasm_aggregate_error_preparation" '^pub[(]super[)][[:space:]]+fn[[:space:]]+emit_finish_aggregate_error_instance[[:space:]]*[(]')"
require_text_regex_count "$aggregate_error_finish_body" '^[[:space:]]*prepared: PreparedAggregateErrorLocal,' 1 'consuming AggregateError completion boundary'
require_regex_count "$wasm_error_constructor" '^pub\(super\) struct PreparedNativeErrorInstance[(]' 1 'opaque unpublished native Error owner'
require_regex_count "$wasm_error_constructor" '^[[:space:]]*pub\(super\)[[:space:]]+fn publish\(' 1 'consuming native Error publication owner'
if grep -Eq 'struct PreparedAggregateErrorLocal[(][[:space:]]*pub' "$wasm_aggregate_error_preparation" \
  || grep -Eq 'struct PreparedNativeErrorInstance[(][[:space:]]*pub' "$wasm_error_constructor"; then
  fail 'prepared Error publication authorities must keep their tuple fields private'
fi
require_regex_count "$wasm_promise_any_error" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn emit_promise_any_aggregate_error\(' 1 'retained-Realm Promise.any AggregateError owner'
# Raw scalar fields and recursive proof/projection counts do not constrain the
# concrete GC record or consuming publication path.
# Actual MAIN129 owner sizes: Error parent 346, preparation 73, constructor 210.
check_raw_line_budget "$wasm_error_builtins" 400
check_raw_line_budget "$wasm_aggregate_error_preparation" 100
check_raw_line_budget "$wasm_error_constructor" 250

wasm_promise_builtins="crates/lila-aot-wasm/src/builtins/promise.rs"
require_file "$wasm_promise_builtins"
check_no_inline_legacy_includes "$wasm_promise_builtins"
for promise_child in \
  promise_internal_function_materialization \
  promise_try_callback_type_error \
  promise_prototype_receiver_type_error \
  promise_prototype_then_invocation \
  promise_settlement_record_allocation \
  promise_with_resolvers_result_allocation
do
  promise_child_source="crates/lila-aot-wasm/src/builtins/promise/${promise_child}.rs"
  require_file "$promise_child_source"
  check_no_inline_legacy_includes "$promise_child_source"
  require_exact_line_count "$wasm_promise_builtins" "mod ${promise_child};" 1 'private Promise lifecycle child'
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${promise_child};" "$wasm_promise_builtins"; then
    fail "$wasm_promise_builtins must keep ${promise_child} private"
  fi
  if grep -Eq "^pub([^[:space:]]*[[:space:]]+)?use[[:space:]]+.*${promise_child}" "$wasm_promise_builtins"; then
    fail "$wasm_promise_builtins must not re-export ${promise_child} lifecycle authorities"
  fi
done
wasm_promise_internal_function_materialization="crates/lila-aot-wasm/src/builtins/promise/promise_internal_function_materialization.rs"
wasm_promise_try_callback_type_error="crates/lila-aot-wasm/src/builtins/promise/promise_try_callback_type_error.rs"
wasm_promise_prototype_receiver_type_error="crates/lila-aot-wasm/src/builtins/promise/promise_prototype_receiver_type_error.rs"
wasm_promise_prototype_then_invocation="crates/lila-aot-wasm/src/builtins/promise/promise_prototype_then_invocation.rs"
wasm_promise_settlement_record_allocation="crates/lila-aot-wasm/src/builtins/promise/promise_settlement_record_allocation.rs"
wasm_promise_with_resolvers_result_allocation="crates/lila-aot-wasm/src/builtins/promise/promise_with_resolvers_result_allocation.rs"
require_regex_count "$wasm_promise_builtins" \
  '^use self::promise_internal_function_materialization::' 1 'private Promise materialization authority import'
require_regex_count "$wasm_promise_internal_function_materialization" \
  '^pub\(crate\) struct PromiseInternalFunctionMaterializationContext[(]' 1 'opaque Promise materialization context owner'
if grep -Eq 'struct PromiseInternalFunctionMaterializationContext[(][[:space:]]*pub' "$wasm_promise_internal_function_materialization"; then
  fail 'Promise materialization must keep its retained Realm context private'
fi
require_regex_count "$wasm_promise_internal_function_materialization" \
  "^pub\(super\) enum PromiseInternalFunction<'a>" 1 'closed typed Promise capture admission owner'
require_regex_count "$wasm_promise_internal_function_materialization" \
  '^[[:space:]]*fn publication\(' 1 'private exhaustive Promise publication selector'
promise_capture_selection="$(braced_rust_item_source "$wasm_promise_internal_function_materialization" '^fn[[:space:]]+publication[[:space:]]*[(]')"
if grep -Eq '^[[:space:]]*_ =>' <<<"$promise_capture_selection"; then
  fail 'Promise internal-function capture publication must remain exhaustive'
fi
for promise_materialization_factory in \
  emit_promise_internal_function_materialization_context_from_realm \
  emit_current_function_promise_internal_function_materialization_context \
  release_promise_internal_function_materialization_context
do
  require_regex_count "$wasm_promise_internal_function_materialization" \
    "^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+${promise_materialization_factory}[[:space:]]*\(" \
    1 "Promise materialization owner $promise_materialization_factory"
done
require_regex_count "$wasm_promise_internal_function_materialization" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn emit_promise_internal_function_value\(' 1 'typed Promise internal-function publication owner'
promise_publication_body="$(braced_rust_item_source "$wasm_promise_internal_function_materialization" '^pub[(]super[)][[:space:]]+fn[[:space:]]+emit_promise_internal_function_value[[:space:]]*[(]')"
require_text_regex_count "$promise_publication_body" \
  "^[[:space:]]*entry: PromiseInternalFunction<'_>," 1 'closed typed Promise capture admission boundary'
require_text_regex_count "$promise_publication_body" \
  '^[[:space:]]*context: &PromiseInternalFunctionMaterializationContext,' 1 'retained Realm materialization boundary'
if grep -q 'materialization_context.realm_local' crates/lila-aot-wasm/src/builtins/promise/promise_resolve_realm_context.rs; then
  fail 'PromiseResolve must load the materialization Realm through the child-owned capability'
fi

# Opaque authorities carry complete values and are consumed by their fixed
# child operations. No scalar layout, construction/projection or recursive-use
# census is required to describe those typed lifecycle boundaries.
for promise_opaque_owner in \
  'promise_try_callback_type_error PromiseTryCallbackTypeErrorPrototypeLocal' \
  'promise_prototype_receiver_type_error PromisePrototypeReceiverTypeErrorPrototypeLocal' \
  'promise_prototype_then_invocation ValidatedPromisePrototypeThenInvocationLocals' \
  'promise_settlement_record_allocation PromiseSettlementRecordAllocationContext' \
  'promise_with_resolvers_result_allocation PromiseWithResolversResultAllocationContext'
do
  set -- $promise_opaque_owner
  promise_opaque_source="crates/lila-aot-wasm/src/builtins/promise/${1}.rs"
  require_regex_count "$promise_opaque_source" \
    "^pub\(super\) struct ${2}[[:space:]]*[({]" 1 'opaque Promise lifecycle authority owner'
  require_tree_regex_count crates/lila-aot-wasm/src "${1}::" 0 'Promise lifecycle authority imports or re-exports'
  require_fixed_string_count "$wasm_promise_builtins" "$2" 0 'Promise lifecycle authority parent names'
  if grep -Eq '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+[[:alnum:]_]+:' "$promise_opaque_source" \
    || grep -Eq "struct ${2}[(][[:space:]]*pub" "$promise_opaque_source"; then
    fail "$promise_opaque_source must keep lifecycle authority fields private"
  fi
done
require_regex_count "$wasm_promise_prototype_receiver_type_error" \
  '^enum PromisePrototypeReceiverError[[:space:]]*\{' 1 'private closed Promise receiver-error domain'
if grep -Eq 'PromisePrototypeReceiverError|emit_throw_promise_prototype_receiver_error\(' "$wasm_promise_builtins"; then
  fail 'Promise receiver errors must consume fixed child operations'
fi
for promise_lifecycle_method_owner in \
  'promise_try_callback_type_error emit_load_promise_try_callback_type_error_prototype' \
  'promise_try_callback_type_error emit_throw_promise_try_non_callable_callback' \
  'promise_prototype_receiver_type_error emit_load_promise_prototype_receiver_type_error_prototype' \
  'promise_prototype_receiver_type_error emit_throw_promise_then_incompatible_receiver_error' \
  'promise_prototype_receiver_type_error emit_throw_promise_finally_non_object_receiver_error' \
  'promise_prototype_then_invocation emit_validate_promise_prototype_then_invocation' \
  'promise_prototype_then_invocation emit_call_validated_promise_prototype_then_invocation' \
  'promise_settlement_record_allocation emit_self_backed_promise_settlement_record_allocation_context' \
  'promise_settlement_record_allocation emit_alloc_promise_settlement_record' \
  'promise_with_resolvers_result_allocation emit_current_function_promise_with_resolvers_result_allocation_context' \
  'promise_with_resolvers_result_allocation emit_alloc_promise_with_resolvers_result'
do
  set -- $promise_lifecycle_method_owner
  promise_lifecycle_source="crates/lila-aot-wasm/src/builtins/promise/${1}.rs"
  require_regex_count "$promise_lifecycle_source" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${2}[[:space:]]*\(" \
    1 'fixed Promise lifecycle child operation'
  require_regex_count "$wasm_promise_builtins" \
    "^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+${2}[[:space:]]*\(" \
    0 'no parent Promise lifecycle operation copy'
done
promise_then_call_body="$(braced_rust_item_source "$wasm_promise_prototype_then_invocation" '^pub[(]super[)][[:space:]]+fn[[:space:]]+emit_call_validated_promise_prototype_then_invocation[[:space:]]*[(]')"
require_text_regex_count "$promise_then_call_body" \
  '^[[:space:]]*invocation: ValidatedPromisePrototypeThenInvocationLocals,' 1 'observed then Reference consumed by its call'
for promise_allocation_boundary in \
  'promise_settlement_record_allocation emit_alloc_promise_settlement_record PromiseSettlementRecordAllocationContext' \
  'promise_with_resolvers_result_allocation emit_alloc_promise_with_resolvers_result PromiseWithResolversResultAllocationContext'
do
  set -- $promise_allocation_boundary
  promise_allocation_source="crates/lila-aot-wasm/src/builtins/promise/${1}.rs"
  promise_allocation_body="$(braced_rust_item_source "$promise_allocation_source" "^pub[(]super[)][[:space:]]+fn[[:space:]]+${2}[[:space:]]*[(]")"
  require_text_regex_count "$promise_allocation_body" "^[[:space:]]*context: ${3}," 1 'selected Promise prototype consumed by allocation'
done
# Actual MAIN129 sizes are 4,628/280/40/85/61/65/70 lines respectively.
check_raw_line_budget "$wasm_promise_builtins" 4800
check_raw_line_budget "$wasm_promise_internal_function_materialization" 320
check_raw_line_budget "$wasm_promise_try_callback_type_error" 80
check_raw_line_budget "$wasm_promise_prototype_receiver_type_error" 120
check_raw_line_budget "$wasm_promise_prototype_then_invocation" 80
check_raw_line_budget "$wasm_promise_settlement_record_allocation" 100
check_raw_line_budget "$wasm_promise_with_resolvers_result_allocation" 110

wasm_json_builtins="crates/lila-aot-wasm/src/builtins/json.rs"
wasm_json_parse_frame_state="crates/lila-aot-wasm/src/builtins/json/parse_frame_state.rs"
wasm_json_parse="crates/lila-aot-wasm/src/builtins/json/parse.rs"
wasm_json_reviver="crates/lila-aot-wasm/src/builtins/json/reviver.rs"
wasm_json_stringify="crates/lila-aot-wasm/src/builtins/json/stringify.rs"
require_file "$wasm_json_builtins"
check_no_inline_legacy_includes "$wasm_json_builtins"
for json_child in grammar parse parse_frame_state quote reviver stringify; do
  json_child_source="crates/lila-aot-wasm/src/builtins/json/${json_child}.rs"
  require_file "$json_child_source"
  check_no_inline_legacy_includes "$json_child_source"
  require_exact_line_count "$wasm_json_builtins" "mod ${json_child};" 1 'private JSON algorithm child'
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${json_child};" "$wasm_json_builtins"; then
    fail "$wasm_json_builtins must keep ${json_child} private"
  fi
done
require_tree_regex_count crates/lila-aot-wasm/src 'parse_frame_state::' 0 'JSON frame-owner imports or re-exports'
require_regex_count "$wasm_json_builtins" '^enum JsonBuiltin[[:space:]]*\{' 1 'private closed JSON operation domain'
require_regex_count "$wasm_json_builtins" '^[[:space:]]*fn emit_json_builtin\(' 1 'private exhaustive JSON dispatcher'
json_dispatch_body="$(braced_rust_item_source "$wasm_json_builtins" '^fn[[:space:]]+emit_json_builtin[[:space:]]*[(]')"
require_text_regex_count "$json_dispatch_body" '^[[:space:]]*match operation[[:space:]]*\{' 1 'exhaustive JSON operation dispatch'
if grep -Eq '^[[:space:]]*_ =>' <<<"$json_dispatch_body"; then
  fail 'JSON operation dispatch must remain exhaustive'
fi
if grep -Eq 'JsonBuiltin|emit_json_builtin\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must consume four fixed JSON operations"
fi
for json_builtin_wrapper in emit_json_parse_builtin emit_json_stringify_builtin emit_json_raw_json_builtin emit_json_is_raw_json_builtin; do
  require_regex_count "$wasm_json_builtins" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${json_builtin_wrapper}[[:space:]]*\(" \
    1 'fixed JSON entry owner'
  require_fixed_string_count "$wasm_standard_builtins" "self.${json_builtin_wrapper}(function)?" 1 'standard JSON route'
done
# Closed domains and concrete rooted records replace the scalar validated-state
# carrier and linear stack. Preserve each private semantic operation owner.
for json_state_domain in JsonParseFrameState JsonReviverFrameState JsonReviverPropertyRole; do
  require_regex_count "$wasm_json_builtins" "^json_domain!\(${json_state_domain}[[:space:]]*\{" 1 'closed JSON frame domain'
done
for json_frame_method in emit_json_parse_frame emit_json_parse_state emit_json_parse_record emit_json_parse_child; do
  require_regex_count "$wasm_json_parse_frame_state" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${json_frame_method}[[:space:]]*\(" \
    1 'private JSON frame lifecycle owner'
done
require_regex_count "$wasm_json_parse" '^[[:space:]]*fn emit_json_parse_text\(' 1 'private runtime JSON parser owner'
require_regex_count "$wasm_json_parse" '^[[:space:]]*fn emit_json_complete_container\(' 1 'completed JSON container publication owner'
require_regex_count "$wasm_json_reviver" '^[[:space:]]*pub\(super\)[[:space:]]+fn emit_json_revive\(' 1 'rooted iterative JSON reviver owner'
for json_entry_owner in 'parse emit_json_parse_entry' 'parse emit_json_raw_entry' 'stringify emit_json_stringify_entry'; do
  set -- $json_entry_owner
  json_entry_source="crates/lila-aot-wasm/src/builtins/json/${1}.rs"
  require_regex_count "$json_entry_source" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${2}[[:space:]]*\(" \
    1 'private JSON semantic entry owner'
  require_fixed_string_count "$wasm_json_builtins" "self.${2}(" 1 'fixed JSON entry consumption'
done
# Parent/frame/parse/reviver/stringify MAIN129 sizes: 324/157/494/494/898.
check_raw_line_budget "$wasm_json_builtins" 360
check_raw_line_budget "$wasm_json_parse_frame_state" 200
check_raw_line_budget "$wasm_json_parse" 550
check_raw_line_budget "$wasm_json_reviver" 550
check_raw_line_budget "$wasm_json_stringify" 950

# Real finite semantic witnesses retain their live registration and fixtures.
# These controls exercise the compiler/Engine/CLI rather than count old ABI
# fields, proof projections or recursive emitter calls.
for semantic_control in \
  'crates/lila-engine/tests/aot_aggregate_error_iterator.rs aggregate_error_preserves_prefix_order_and_never_closes_protocol_abrupts' \
  'crates/lila-engine/tests/aot_aggregate_error_constructor_realm.rs prototype_getters_preserve_objects_and_abrupt_completions' \
  'crates/lila-engine/tests/aot_promise_combinator_intrinsics.rs any_uses_the_canonical_aggregate_error_for_empty_and_rejected_inputs' \
  'crates/lila-engine/tests/aot_promise_combinator_intrinsics.rs borrowed_foreign_combinators_separate_method_and_capability_realms' \
  'crates/lila-engine/tests/aot_json_canonical_reviver.rs source_context_uses_same_value_and_final_duplicate_source_without_static_values' \
  'crates/lila-engine/tests/aot_json_reviver_definitions.rs reviver_propagates_proxy_definition_throws_through_catch_and_finally' \
  'crates/lila-engine/tests/aot_json_stringify_preparation.rs gc_serialization_retains_context_path_lists_callbacks_and_abrupt_values' \
  'crates/lila-cli/tests/cli/functions.rs run_wasm_backend_publishes_created_realm_promise_foundation' \
  'crates/lila-cli/tests/cli/functions.rs run_wasm_backend_preserves_created_realm_promise_internal_callbacks' \
  'crates/lila-cli/tests/cli/language_numerics.rs run_wasm_backend_succeeds_for_json_parse_dynamic_reviver_frame_fixture' \
  'crates/lila-cli/tests/cli/language_numerics.rs run_wasm_backend_preserves_json_stringify_replacer_invocation_roles'
do
  set -- $semantic_control
  require_file "$1"
  require_regex_count "$1" "^fn[[:space:]]+${2}[[:space:]]*\(" 1 'finite Error/Promise/JSON semantic test owner'
  semantic_control_attributes="$(rust_item_attributes_source "$1" "^fn[[:space:]]+${2}[[:space:]]*[(]")"
  require_text_regex_count "$semantic_control_attributes" '^#\[test\]$' 1 'live Error/Promise/JSON semantic test registration'
  if grep -Eq '^#\[(cfg|cfg_attr|ignore)([^[:alnum:]_]|$)' <<<"$semantic_control_attributes"; then
    fail "$1 ${2} semantic witness must remain enabled"
  fi
done
for semantic_fixture_binding in \
  'crates/lila-engine/tests/aot_aggregate_error_iterator.rs aggregate_error_preserves_prefix_order_and_never_closes_protocol_abrupts fixtures/aggregate_error_iterator/ordering_and_abrupt.js' \
  'crates/lila-engine/tests/aot_json_canonical_reviver.rs source_context_uses_same_value_and_final_duplicate_source_without_static_values fixtures/json_canonical_reviver/source_context_mutation.js' \
  'crates/lila-engine/tests/aot_json_stringify_preparation.rs gc_serialization_retains_context_path_lists_callbacks_and_abrupt_values fixtures/json_stringify_preparation/gc_roots_and_abrupt.js' \
  'crates/lila-cli/tests/cli/functions.rs run_wasm_backend_publishes_created_realm_promise_foundation wasm_promise_created_realm.js' \
  'crates/lila-cli/tests/cli/functions.rs run_wasm_backend_preserves_created_realm_promise_internal_callbacks wasm_promise_internal_callback_realm.js' \
  'crates/lila-cli/tests/cli/language_numerics.rs run_wasm_backend_succeeds_for_json_parse_dynamic_reviver_frame_fixture wasm_json_parse_dynamic_reviver_frame.js' \
  'crates/lila-cli/tests/cli/language_numerics.rs run_wasm_backend_preserves_json_stringify_replacer_invocation_roles wasm_json_stringify_replacer_invocation_roles.js'
do
  set -- $semantic_fixture_binding
  semantic_witness_body="$(braced_rust_item_source "$1" "^fn[[:space:]]+${2}[[:space:]]*[(]")"
  if ! grep -Fq "\"${3}\"" <<<"$semantic_witness_body"; then
    fail "$1 ${2} must consume its actual semantic fixture ${3}"
  fi
done
for semantic_fixture in \
  crates/lila-engine/tests/fixtures/aggregate_error_iterator/ordering_and_abrupt.js \
  crates/lila-engine/tests/fixtures/aggregate_error_iterator/realms_and_protocol.js \
  crates/lila-engine/tests/fixtures/json_canonical_reviver/source_context_mutation.js \
  crates/lila-engine/tests/fixtures/json_stringify_preparation/gc_roots_and_abrupt.js \
  crates/lila-cli/tests/fixtures/wasm_promise_internal_callback_realm.js \
  crates/lila-cli/tests/fixtures/wasm_promise_created_realm.js \
  crates/lila-cli/tests/fixtures/wasm_json_parse_dynamic_reviver_frame.js \
  crates/lila-cli/tests/fixtures/wasm_json_stringify_replacer_invocation_roles.js
do
  require_file "$semantic_fixture"
done
# T02's strong Map get-or-insert owner. Its two crate-visible semantic entry
# points remain product-callable, but only the private child may construct the
# raw value-source policy or call the shared parameterized emitter. WeakMap's
# selected-runtime rejection is owned by weak_unavailable, not this algorithm.
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
if grep -Eq 'MapGetOrInsertValueSource|emit_collection_get_or_insert\(|map_get_or_insert::' "$wasm_collections_builtins"; then
  fail "$wasm_collections_builtins must not name, construct, project or import the private Map get-or-insert policy"
fi
require_fixed_string_count \
  "$wasm_map_get_or_insert" \
  'MapGetOrInsertValueSource' \
  8 \
  'Map get-or-insert value-source owner lines'
require_fixed_string_count \
  "$wasm_map_get_or_insert" \
  'MapGetOrInsertValueSource::' \
  6 \
  'Map get-or-insert qualified value-source uses'
require_fixed_string_count \
  "$wasm_map_get_or_insert" \
  'emit_collection_get_or_insert(' \
  3 \
  'private Map get-or-insert emitter definition and calls'
for semantic_get_or_insert in \
  emit_map_prototype_get_or_insert \
  emit_map_prototype_get_or_insert_computed
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
# GC record schemas own storage; parent files own admission, constructors and
# accessors, and method leaves consume their typed records and completed proofs.
# ZonedDateTime constructors and records are owned by temporal.rs, included below.
for module in temporal temporal_plain_date temporal_plain_date_methods \
              temporal_plain_time temporal_plain_time_methods \
              temporal_plain_date_time temporal_plain_date_time_methods \
              temporal_zoned_date_time_methods; do
  require_file "crates/lila-aot-wasm/src/builtins/${module}.rs"
  require_module_decl "$wasm_builtins_mod" "$module"
  check_no_inline_legacy_includes "crates/lila-aot-wasm/src/builtins/${module}.rs"
done

wasm_temporal_plain_date="crates/lila-aot-wasm/src/builtins/temporal_plain_date.rs"
wasm_temporal_exact="crates/lila-aot-wasm/src/builtins/temporal_zone_provider/exact.rs"
require_file "$wasm_temporal_exact"
require_exact_line_count "$wasm_temporal_exact" \
  'pub(crate) struct TemporalCalendarSlotLocals {' 1 'completed Temporal GC calendar slot owner'
temporal_calendar_slot_fields="$(sed -n '/^pub(crate) struct TemporalCalendarSlotLocals {$/,/^}$/p' "$wasm_temporal_exact")"
if grep -Eq '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+' <<<"$temporal_calendar_slot_fields"; then
  fail "$wasm_temporal_exact must keep calendar-slot fields private"
fi
require_regex_count "$wasm_temporal_exact" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+emit_temporal_calendar_slot_from_identifier[[:space:]]*\(' 1 'Temporal calendar admission producer'
if grep -Eq 'TemporalCalendarCarrier|emit_temporal_calendar_slot_fast_path\(' "$wasm_temporal_plain_date"; then
  fail "$wasm_temporal_plain_date must consume completed GC calendar slots rather than a raw calendar carrier"
fi
wasm_temporal_plain_month_day="crates/lila-aot-wasm/src/builtins/temporal_plain_month_day.rs"
require_file "$wasm_temporal_plain_month_day"
require_exact_line_count "$wasm_temporal_plain_month_day" \
  'struct TemporalParsedMonthDayYear {' 1 'private parsed MonthDay source-year proof'
require_regex_count "$wasm_temporal_plain_month_day" \
  '^[[:space:]]*fn[[:space:]]+emit_temporal_parse_month_day_string[[:space:]]*\(' 1 'private GC MonthDay parser'
require_regex_count "$wasm_temporal_plain_month_day" \
  '^[[:space:]]*fn[[:space:]]+emit_temporal_month_day_string_reference_year[[:space:]]*\(' 1 'private completed MonthDay reference conversion'
if grep -Eq 'pub(\([^)]*\))?[[:space:]]+(struct[[:space:]]+TemporalParsedMonthDayYear|fn[[:space:]]+emit_temporal_parse_month_day_string|fn[[:space:]]+emit_temporal_month_day_string_reference_year)' "$wasm_temporal_plain_month_day"; then
  fail "$wasm_temporal_plain_month_day must keep parsed-year admission and reference conversion private"
fi
require_fixed_string_count "$wasm_temporal_plain_month_day" \
  'parsed: TemporalParsedMonthDayYear,' 1 'consuming parsed MonthDay reference conversion input'
require_fixed_string_count "$wasm_temporal_plain_month_day" \
  'self.emit_temporal_parse_month_day_string(' 1 'MonthDay string parser consumer'
require_fixed_string_count "$wasm_temporal_plain_month_day" \
  'self.emit_temporal_month_day_string_reference_year(' 1 'MonthDay source-year policy consumer'

wasm_temporal_duration="crates/lila-aot-wasm/src/builtins/temporal_duration.rs"
wasm_temporal_duration_fields="crates/lila-aot-wasm/src/builtins/temporal_duration/fields.rs"
wasm_temporal_plain_date_time="crates/lila-aot-wasm/src/builtins/temporal_plain_date_time.rs"
wasm_gc_layouts="crates/lila-aot-wasm/src/gc_types/layouts.rs"
require_file "$wasm_temporal_duration"
require_file "$wasm_temporal_duration_fields"
require_file "$wasm_gc_layouts"
require_module_decl "$wasm_temporal_duration" fields
require_exact_line_count "$wasm_temporal_duration_fields" \
  'pub(crate) struct TemporalDurationFields([I64Local; 10]);' 1 'private-representation Duration Number-bits proof'
require_exact_line_count "$wasm_temporal_duration" \
  'pub(super) enum TemporalDurationField {' 1 'closed Duration accessor selector'
require_exact_line_count "$wasm_temporal_plain_date_time" \
  'pub(crate) enum TemporalPlainDateTimeField {' 1 'closed PlainDateTime accessor selector'
# GC schema names, rather than manual byte offsets, join allocation and reads.
for temporal_gc_record in TemporalDurationObject TemporalPlainDateTimeObject TemporalInstantObject; do
  require_regex_count "$wasm_gc_layouts" \
    "^[[:space:]]*struct ${temporal_gc_record} => ${temporal_gc_record}Schema \{" 1 'central Temporal GC record schema'
done
require_fixed_string_count "$wasm_temporal_duration" \
  'schema.struct_type::<TemporalDurationObject>().construct(' 1 'GC Duration allocator'
require_fixed_string_count "$wasm_temporal_duration" \
  'self.emit_temporal_record_from_receiver::<TemporalDurationObject>(' 1 'GC Duration receiver admission'
require_fixed_string_count "$wasm_temporal_plain_date_time" \
  'self.emit_temporal_record_from_receiver::<TemporalPlainDateTimeObject>(' 2 'GC PlainDateTime receiver entries'
if grep -Eq 'TEMPORAL_(DURATION|PLAIN_DATE_TIME)_FIELD_OFFSETS' "$wasm_temporal_duration" "$wasm_temporal_plain_date_time"; then
  fail 'Temporal Duration/PlainDateTime records must not restore manual field-offset tables'
fi
wasm_temporal_instant="crates/lila-aot-wasm/src/builtins/temporal_instant.rs"
wasm_temporal_epoch="crates/lila-aot-wasm/src/builtins/temporal/epoch.rs"
require_file "$wasm_temporal_instant"
require_file "$wasm_temporal_epoch"
require_module_decl crates/lila-aot-wasm/src/builtins/temporal.rs epoch
require_exact_line_count "$wasm_temporal_epoch" \
  'pub(in crate::builtins) struct TemporalEpochNanoseconds {' 1 'completed Instant epoch proof owner'
temporal_epoch_fields="$(sed -n '/^pub(in crate::builtins) struct TemporalEpochNanoseconds {$/,/^}$/p' "$wasm_temporal_epoch")"
if grep -Eq '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+' <<<"$temporal_epoch_fields"; then
  fail "$wasm_temporal_epoch must keep the validated epoch representation private"
fi
require_regex_count "$wasm_temporal_epoch" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+emit_alloc_temporal_instant[[:space:]]*\(' 1 'completed epoch Instant allocator'
require_fixed_string_count "$wasm_temporal_epoch" \
  'epoch: &TemporalEpochNanoseconds,' 1 'Instant allocator completed-proof input'
for instant_diagnostic in \
  TEMPORAL_INSTANT_FROMEPOCHMILLISECONDS_REQUIRES_AN_INTEGRAL_NUMBER \
  TEMPORAL_INSTANT_DOES_NOT_SUPPORT_IMPLICIT_CONVERSION_USE_COMPARE_OR_EQUALS; do
  require_fixed_string_count "$wasm_temporal_instant" "RuntimeErrorMessage::${instant_diagnostic}" 1 'typed Instant diagnostic consumer'
done
# These real Engine controls retain record values, ordered duration conversion,
# negative epochs, UTF-16 syntax and provider/Realm joins across the GC change.
require_file crates/lila-engine/tests/aot_gc_temporal_entries.rs
require_file crates/lila-engine/tests/aot_temporal_created_realm.rs
for temporal_gc_fixture in records_duration_order instant_epoch_utf16 zoned_transitions_realms; do
  require_file "crates/lila-engine/tests/fixtures/temporal_gc/${temporal_gc_fixture}.js"
  require_fixed_string_count crates/lila-engine/tests/aot_gc_temporal_entries.rs \
    "fixtures/temporal_gc/${temporal_gc_fixture}.js" 1 'real Engine Temporal GC fixture attachment'
done

wasm_string_trim="crates/lila-aot-wasm/src/operations/string_trim.rs"
wasm_string_operations="crates/lila-aot-wasm/src/operations.rs"
require_file "$wasm_string_trim"
require_module_decl "$wasm_string_operations" string_trim
require_exact_line_count "$wasm_string_trim" 'enum EcmaTrimMode {' 1 'private closed UTF-16 trim boundary domain'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+enum[[:space:]]+EcmaTrimMode' "$wasm_string_trim"; then
  fail "$wasm_string_trim must keep its trim boundary selector private"
fi
for trim_entry in start:Start end:End both:Both; do
  trim_method="${trim_entry%%:*}"
  trim_mode="${trim_entry#*:}"
  require_regex_count "$wasm_string_trim" \
    "^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_ecmascript_trim_${trim_method}_payload_from_locals[[:space:]]*\(" 1 'fixed GC trim entry'
  require_fixed_string_count "$wasm_string_trim" \
    "self.emit_ecmascript_trim_payload_from_locals(string, EcmaTrimMode::${trim_mode}," 1 'closed trim mode selection'
done
require_regex_count "$wasm_string_operations" \
  '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_gc_string_trim_range[[:space:]]*\(' 1 'shared GC UTF-16 trim range owner'
require_regex_count "$wasm_string_operations" \
  '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_ecmascript_whitespace_i32[[:space:]]*\(' 1 'shared ECMAScript whitespace policy owner'
require_fixed_string_count "$wasm_string_trim" \
  'self.emit_gc_string_trim_range(string, start, end, function);' 1 'typed GC trim range consumer'
if grep -Eq 'ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8' "$wasm_string_trim" "$wasm_builtins_mod"; then
  fail 'String trimming must use the shared UTF-16 whitespace owner rather than a byte-pattern table'
fi
require_file crates/lila-cli/tests/fixtures/wasm_string_annexb_substr_trim_core.js
require_regex_count crates/lila-cli/tests/cli/string.rs \
  '^fn run_wasm_backend_succeeds_for_string_annexb_substr_trim_fixture[[:space:]]*\(' 1 'real CLI trim semantics witness'
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

wasm_objects_property_read="$(braced_rust_item_source crates/lila-aot-wasm/src/objects.rs '^pub[(]crate[)][[:space:]]+fn[[:space:]]+compile_property_read_from_locals[[:space:]]*[(]')"
require_text_regex_count "$wasm_objects_property_read" 'receiver: &ValueLocals,' 1 'raw whole receiver property-read input'
require_text_regex_count "$wasm_objects_property_read" 'output: &ValueLocals,' 1 'whole property-read result'
require_text_regex_count "$wasm_objects_property_read" 'self\.emit_dynamic_property_read_with_key_locals[(]' 1 'sole dynamic property-read consumer'
wasm_dynamic_property_read="$(braced_rust_item_source crates/lila-aot-wasm/src/objects.rs '^pub[(]crate[)][[:space:]]+fn[[:space:]]+emit_dynamic_property_read_with_key_locals[[:space:]]*[(]')"
require_text_regex_count "$wasm_dynamic_property_read" 'crate::runtime_helpers::ObjectReadArguments::new[(]' 1 'registered whole property-read helper boundary'
require_text_regex_count "$wasm_dynamic_property_read" 'receiver: &ValueLocals,' 1 'retained whole Get receiver'
require_text_regex_count "$wasm_dynamic_property_read" 'result: &CompletionLocals,' 1 'complete Get completion publication'
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
  static_generator_declaration_values_by_name \
  static_generator_call_overrides \
  static_object_iterator_literal_values \
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
# Generator calls and assignments retain their actual values through the
# non-property Call and assignment owners checked above, never cached yields.
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
test262_worker_controls="crates/lila-test262/tests/differential_worker_execution.rs"
require_file crates/lila-test262/src/differential/worker.rs
require_file crates/lila-test262/src/differential/worker_process.rs
require_file crates/lila-test262/tests/differential_oracle_compile_boundary_structure.rs
require_file "$test262_worker_controls"
require_exact_line_count "$test262_differential" 'mod worker;' 1 'feature-only differential worker attachment'
worker_module_attributes="$(rust_item_attributes_source "$test262_differential" '^mod[[:space:]]+worker;')"
require_text_regex_count "$worker_module_attributes" '^#\[cfg\(feature = "spec-exec-oracle"\)\]$' 1 'worker module explicit oracle compile gate'
# The existing compile-boundary test checks every gated declaration and the
# feature-off refusal. The executable controls are bound to the required worker.
require_exact_line_count "$test262_worker_controls" '#![cfg(feature = "spec-exec-oracle")]' 1 'feature-only paired worker controls'
require_fixed_string_count "$test262_worker_controls" 'env!("CARGO_BIN_EXE_lila-differential-worker")' 2 'actual Cargo worker selection and image identity controls'
require_fixed_string_count "$test262_worker_controls" 'fn module_loader_context_sources(' 1 'feature-only module-loader fixture owner'
require_exact_line_count crates/lila-test262/src/lib.rs 'fn skip_template_source(bytes: &[u8], mut idx: usize) -> usize {' 0 'obsolete template-source scanner'
require_exact_line_count crates/lila-test262/src/lib.rs '    fn values_mut(&mut self) -> Option<&mut Vec<T>> {' 1 'test-only wire-list mutation entry'

wasm_temporal_zoned_date_time_methods="crates/lila-aot-wasm/src/builtins/temporal_zoned_date_time_methods.rs"
wasm_temporal_zoned_arithmetic="crates/lila-aot-wasm/src/builtins/temporal_zoned_arithmetic.rs"
require_file "$wasm_temporal_zoned_arithmetic"
for direction_domain in TemporalZonedArithmeticOperation TemporalZonedDifferenceOperation; do
  require_fixed_string_count \
    "$wasm_temporal_zoned_arithmetic" \
    "pub(super) enum $direction_domain" \
    1 \
    "shared ZonedDateTime direction authority $direction_domain"
  if grep -Eq "enum[[:space:]]+$direction_domain" "$wasm_temporal_zoned_date_time_methods"; then
    fail "$direction_domain must be owned by the shared ZonedDateTime arithmetic authority"
  fi
  if grep -Eq "$direction_domain" "$wasm_standard_builtins" "$wasm_builtins_mod"; then
    fail "$direction_domain must not escape its ZonedDateTime authority and method entries"
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
wasm_module_source_intrinsic='crates/lila-aot-wasm/src/intrinsics/abstract_module_source.rs'
wasm_module_source_host='crates/lila-aot-wasm/src/builtins/host/abstract_module_source.rs'
require_file "$wasm_module_source_intrinsic"
require_file "$wasm_module_source_host"
require_exact_line_count "$wasm_intrinsics_mod" 'mod abstract_module_source;' 1 'private native module-source intrinsic owner'
require_exact_line_count 'crates/lila-aot-wasm/src/builtins/host.rs' 'mod abstract_module_source;' 1 'private Test262 intrinsic retrieval owner'
require_fixed_string_count "$wasm_module_source_intrinsic" 'pub(crate) fn emit_initialize_abstract_module_source_intrinsic(' 1 'actual GC native intrinsic bootstrap owner'
require_fixed_string_count 'crates/lila-aot-wasm/src/builtins/bootstrap.rs' 'self.emit_initialize_abstract_module_source_intrinsic(' 1 'consumed Realm bootstrap entry'
require_fixed_string_count "$wasm_module_source_host" 'pub(crate) fn compile_host_get_abstract_module_source_builtin(' 1 'actual defining-Realm intrinsic retrieval owner'
require_fixed_string_count 'crates/lila-aot-wasm/src/emit/body_compilation.rs' 'self.compile_host_get_abstract_module_source_builtin(' 1 'consumed native host body entry'
check_no_inline_legacy_includes "$wasm_module_source_intrinsic"
check_no_inline_legacy_includes "$wasm_module_source_host"
check_raw_line_budget "$wasm_module_source_intrinsic" 75
check_raw_line_budget "$wasm_module_source_host" 45
check_no_inline_legacy_includes "$wasm_intrinsics_mod"

require_pub_use "$wasm_lib" '^pub use emit::emit;' 'the Wasm emit entry point'
# 180 against a CODE-ONLY count, measured 101 at batch 6 (118 raw). Unlike the
# `lila-ir` budget above this one was never near its limit, so the switch to a
# code-only count did not need a matching adjustment.
# Two actual declaration/body owners, function_entry and emitted_function,
# add their consumed module declarations to the crate orchestration surface.
check_orchestration_surface "$wasm_lib" 182
check_no_inline_legacy_includes "$wasm_lib"
check_no_inline_legacy_includes "$wasm_builtins_mod"

# T02's RegExp range-search owner. The parent matcher may request the semantic
# mismatch operation, but only the private child may select encoded range
# bounds or read their raw offsets.
wasm_regexp_builtins="crates/lila-aot-wasm/src/builtins/regexp.rs"
wasm_regexp_range_search="crates/lila-aot-wasm/src/builtins/regexp/range_search.rs"
wasm_regexp_counted="crates/lila-aot-wasm/src/builtins/regexp/counted.rs"
wasm_regexp_workspace="crates/lila-aot-wasm/src/builtins/regexp/matcher_workspace.rs"
for regexp_state_owner in "$wasm_regexp_counted" "$wasm_regexp_workspace"; do
  require_file "$regexp_state_owner"
  check_no_inline_legacy_includes "$regexp_state_owner"
done
require_exact_line_count "$wasm_regexp_builtins" 'mod counted;' 1 'private counted-repeat owner attachment'
require_exact_line_count "$wasm_regexp_builtins" 'mod matcher_workspace;' 1 'private matcher workspace attachment'
require_fixed_string_count "$wasm_regexp_counted" 'pub(super) fn emit_regexp_counted_dispatch(' 1 'consumed counted-repeat dispatcher'
require_fixed_string_count "$wasm_regexp_builtins" 'fn emit_regexp_counted_dispatch(' 0 'no parent counted-repeat dispatcher copy'
require_fixed_string_count "$wasm_regexp_builtins" 'self.emit_regexp_counted_dispatch(' 1 'actual native matcher counted-repeat dispatch'
require_fixed_string_count "$wasm_regexp_builtins" 'MatcherWorkspace::allocate(' 1 'actual matcher workspace allocation'
require_fixed_string_count "$wasm_regexp_builtins" 'workspace.choices().push_snapshot(' 2 'ordinary and progress choices consume the same linked full-state owner'
require_fixed_string_count "$wasm_regexp_counted" 'workspace.choices().push_snapshot(' 1 'counted fallback consumes the same linked full-state owner'
require_fixed_string_count "$wasm_regexp_builtins" 'workspace.choices().find_assertion(' 1 'assertion rollback searches logical entries'
require_fixed_string_count "$wasm_regexp_builtins" 'workspace.choices().find_progress(' 1 'progress searches logical entries'
for obsolete_choice_access in 'workspace.ensure_frame(' 'workspace.save_state(' 'choice_depth' 'frame_width'; do
  require_fixed_string_count "$wasm_regexp_builtins" "$obsolete_choice_access" 0 'no fixed-depth choice indexing in the matcher'
  require_fixed_string_count "$wasm_regexp_counted" "$obsolete_choice_access" 0 'no fixed-depth choice indexing in counted dispatch'
done
wasm_regexp_compiler="crates/lila-aot-wasm/src/builtins/regexp/compiler.rs"
wasm_regexp_program="crates/lila-aot-wasm/src/builtins/regexp/program.rs"
for regexp_descriptor_owner in \
  'compiler:repeat_bounds:emit_regexp_compile_repeat_bounds:300' \
  'program:repeat_bounds:emit_validate_regexp_repeat_bounds:370' \
  'program:counted_regions:emit_validate_regexp_counted_regions:575'
do
  regexp_descriptor_parent="${regexp_descriptor_owner%%:*}"
  regexp_descriptor_tail="${regexp_descriptor_owner#*:}"
  regexp_descriptor_child="${regexp_descriptor_tail%%:*}"
  regexp_descriptor_tail="${regexp_descriptor_tail#*:}"
  regexp_descriptor_entry="${regexp_descriptor_tail%%:*}"
  regexp_descriptor_budget="${regexp_descriptor_tail#*:}"
  regexp_descriptor_parent_path="crates/lila-aot-wasm/src/builtins/regexp/${regexp_descriptor_parent}.rs"
  regexp_descriptor_child_path="crates/lila-aot-wasm/src/builtins/regexp/${regexp_descriptor_parent}/${regexp_descriptor_child}.rs"
  require_file "$regexp_descriptor_child_path"
  require_exact_line_count "$regexp_descriptor_parent_path" "mod ${regexp_descriptor_child};" 1 'private exact RegExp descriptor owner attachment'
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+${regexp_descriptor_child};" "$regexp_descriptor_parent_path"; then
    fail "$regexp_descriptor_parent_path must keep ${regexp_descriptor_child} private"
  fi
  require_fixed_string_count "$regexp_descriptor_child_path" "pub(super) fn ${regexp_descriptor_entry}(" 1 'consumed exact RegExp descriptor entry'
  require_fixed_string_count "$regexp_descriptor_parent_path" "self.${regexp_descriptor_entry}(" 1 'actual descriptor publication/admission consumer'
  require_fixed_string_count "$regexp_descriptor_parent_path" "fn ${regexp_descriptor_entry}(" 0 'no parent exact descriptor implementation copy'
  check_no_inline_legacy_includes "$regexp_descriptor_child_path"
  check_raw_line_budget "$regexp_descriptor_child_path" "$regexp_descriptor_budget"
done
check_raw_line_budget "$wasm_regexp_compiler" 1900
check_raw_line_budget "$wasm_regexp_program" 720
wasm_regexp_natural_counter="crates/lila-aot-wasm/src/builtins/regexp/counted/natural_counter.rs"
wasm_regexp_required_empty="crates/lila-aot-wasm/src/builtins/regexp/counted/required_empty.rs"
wasm_regexp_required_run="crates/lila-aot-wasm/src/builtins/regexp/counted/required_choice_run.rs"
wasm_regexp_independent_iterations="crates/lila-aot-wasm/src/builtins/regexp/counted/independent_iterations.rs"
wasm_regexp_choices="crates/lila-aot-wasm/src/builtins/regexp/matcher_workspace/choice_entries.rs"
wasm_regexp_run_playback="crates/lila-aot-wasm/src/builtins/regexp/matcher_workspace/choice_entries/required_run.rs"
for regexp_counter_child in natural_counter completed_exact_child input_only_assertion required_empty independent_iterations; do
  regexp_counter_path="crates/lila-aot-wasm/src/builtins/regexp/counted/${regexp_counter_child}.rs"
  require_file "$regexp_counter_path"
  require_exact_line_count "$wasm_regexp_counted" "mod ${regexp_counter_child};" 1 'private native counted-repeat child attachment'
  if grep -Eq "^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+${regexp_counter_child};" "$wasm_regexp_counted"; then
    fail "$wasm_regexp_counted must keep ${regexp_counter_child} private"
  fi
  check_no_inline_legacy_includes "$regexp_counter_path"
done
require_exact_line_count "$wasm_regexp_counted" 'pub(super) mod required_choice_run;' 1 'Run proof is visible only to its same-matcher physical consumer'
require_file "$wasm_regexp_required_run"
check_no_inline_legacy_includes "$wasm_regexp_required_run"
require_regex_count "$wasm_regexp_natural_counter" '^pub\(super\) struct MinimumCounter;' 1 'private minimum counter role'
require_regex_count "$wasm_regexp_natural_counter" '^pub\(super\) struct MaximumCounter;' 1 'private maximum counter role'
require_fixed_string_count "$wasm_regexp_counted" 'NaturalCounterLocals::reserve(' 2 'actual minimum/maximum counter allocation'
# Private scoped proofs retain the actual admitted pair and workspace. Begin
# translates only independent iterations; End preserves ordered empty replay
# or publishes the exact proven template, retaining the finite optional gap.
for regexp_proof in \
  'required_empty:ProvenRequiredEmptyReplay:RequiredEmptyReplayLocals' \
  'required_choice_run:ProvenRequiredChoiceRun:RequiredChoiceRunLocals' \
  'independent_iterations:ProvenIndependentIterations:IndependentIterationLocals'
do
  regexp_proof_child="${regexp_proof%%:*}"
  regexp_proof_tail="${regexp_proof#*:}"
  regexp_proof_type="${regexp_proof_tail%%:*}"
  regexp_proof_locals="${regexp_proof_tail#*:}"
  regexp_proof_path="crates/lila-aot-wasm/src/builtins/regexp/counted/${regexp_proof_child}.rs"
  require_fixed_string_count "$regexp_proof_path" 'struct ProofScope;' 1 'private scoped acceleration authority'
  regexp_proof_fields="$(sed -n "/struct ${regexp_proof_type}</,/^}/p" "$regexp_proof_path")"
  require_text_regex_count "$regexp_proof_fields" '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+' 0 'no caller-manufactured acceleration proof'
  require_fixed_string_present "$regexp_proof_path" "fn emit_if_proven<'body>(" 'actual pair/workspace proof constructor'
  require_fixed_string_count "$wasm_regexp_counted" "${regexp_proof_locals}::emit_if_proven(" 1 'actual counted dispatcher consumes its proof'
done
require_fixed_string_count "$wasm_regexp_counted" 'proof.finish_required_batch(' 2 'checked End consumes empty replay or ordered Run publication'
require_fixed_string_count "$wasm_regexp_counted" 'proof.translate_required_count(builder, f)' 1 'checked Begin consumes independent-iteration translation'
require_fixed_string_count "$wasm_regexp_counted" 'workspace.choices().observe_repeat_end(self, p.state, f);' 1 'actual End observes the authorized live row before its counter semantics'
for regexp_batch_proof in "$wasm_regexp_required_empty" "$wasm_regexp_required_run"; do
  require_fixed_string_present "$regexp_batch_proof" 'self.pair.maximum.subtract_minimum(' 'finite maximum preserves the exact optional gap'
  require_fixed_string_present "$regexp_batch_proof" 'self.pair.minimum.clear(' 'completed mandatory batch retires its minimum'
done
require_fixed_string_count "$wasm_regexp_required_run" '.append_required_run(builder, &self, f)?;' 1 'only a same-workspace proven template publishes a Run'
for bounded_counter_operation in greater_than_bounded subtract_bounded retain_bounded; do
  require_tree_regex_count crates/lila-aot-wasm/src/builtins/regexp/counted "^[[:space:]]*pub\\(in crate::builtins::regexp::counted\\) fn ${bounded_counter_operation}\\(" 1 'sole canonical source-sized bounded counter operation'
  require_fixed_string_present "$wasm_regexp_independent_iterations" ".${bounded_counter_operation}(" 'independent translation consumes full-limb arithmetic'
done
require_fixed_string_present "$wasm_regexp_independent_iterations" 'self.pair.maximum.subtract_minimum(' 'independent translation preserves max-minus-min'
require_exact_line_count "$wasm_regexp_workspace" 'mod choice_entries;' 1 'private linked choice arena attachment'
require_fixed_string_count "$wasm_regexp_workspace" 'pub(super) fn choices(&self)' 1 'sole same-workspace choice authority'
require_fixed_string_count "$wasm_regexp_workspace" 'fn ensure_choice_bytes(' 1 'exclusive transient-tail growth owner'
require_exact_line_count "$wasm_regexp_choices" 'mod required_run;' 1 'private physical Run playback attachment'
for regexp_run_child in natural logical exhaustion path tree; do
  regexp_run_child_path="crates/lila-aot-wasm/src/builtins/regexp/matcher_workspace/choice_entries/required_run/${regexp_run_child}.rs"
  require_file "$regexp_run_child_path"
  require_exact_line_count "$wasm_regexp_run_playback" "mod ${regexp_run_child};" 1 'private actual Run semantic owner'
  check_no_inline_legacy_includes "$regexp_run_child_path"
done
require_tree_regex_count crates/lila-aot-wasm/src/builtins/regexp/matcher_workspace '^[[:space:]]*pub\(in crate::builtins::regexp\) fn append_required_run\(' 1 'sole physical Run publication owner'
require_fixed_string_present "$wasm_regexp_run_playback" 'core::ptr::eq(self.workspace, proof.workspace())' 'proof belongs to the original arena'
require_fixed_string_present "$wasm_regexp_run_playback" '(proof.template_bytes(), layout.template_bytes)' 'Run copies a measured immutable tree rather than unfolding repetitions'
require_fixed_string_present "$wasm_regexp_run_playback" 'layout.initialize_tree(self.workspace, builder, f);' 'copied child paths use checked relative playback state'
for regexp_tree_reader in with_template_snapshots compare_template_tree; do
  require_tree_regex_count crates/lila-aot-wasm/src/builtins/regexp/matcher_workspace "^[[:space:]]*pub\\(in crate::builtins::regexp\\) fn ${regexp_tree_reader}\\(" 1 'sole bounded same-workspace tree reader'
  require_fixed_string_present "$wasm_regexp_required_run" ".${regexp_tree_reader}(" 'proof consumes the actual physical tree'
done
require_fixed_string_present "$wasm_regexp_builtins" 'entry.restore_required_run(' 'native failure resumes the selected virtual continuation'
require_fixed_string_present "$wasm_regexp_builtins" 'entry.restore_repeats(builder, function);' 'logical assertion rollback restores the complete counter slab'
for regexp_choice_owner in "$wasm_regexp_choices" "$wasm_regexp_run_playback"; do
  check_no_inline_legacy_includes "$regexp_choice_owner"
done
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
# GC program/input roots and typed scalar locals bring the matcher to 3,960 lines.
# Range-search remains its separate 120-line owner with a narrow margin.
check_raw_line_budget "$wasm_regexp_builtins" 4050
check_raw_line_budget "$wasm_regexp_range_search" 145

# String protocol acquisition and RegExp flag selection have private children.
# Fixed Standard entries consume those owners; the dispatcher cannot select a
# private protocol or recreate a public-property lookup.
wasm_string_builtins="crates/lila-aot-wasm/src/builtins/string.rs"
wasm_string_symbol_method="crates/lila-aot-wasm/src/builtins/string/symbol_method.rs"
wasm_regexp_protocol="crates/lila-aot-wasm/src/builtins/string/regexp_protocol.rs"
wasm_regexp_substitution="crates/lila-aot-wasm/src/builtins/string/regexp_substitution.rs"
wasm_string_literal_replacement_scope="crates/lila-aot-wasm/src/builtins/string/string_literal_replacement_scope.rs"
for string_child in symbol_method regexp_protocol regexp_substitution string_literal_replacement_scope; do
  string_child_source="crates/lila-aot-wasm/src/builtins/string/${string_child}.rs"
  require_file "$string_child_source"
  require_exact_line_count "$wasm_string_builtins" "mod ${string_child};" 1 'private String algorithm child'
  check_no_inline_legacy_includes "$string_child_source"
  if grep -Eq "^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+${string_child};" "$wasm_string_builtins"; then
    fail "$wasm_string_builtins must keep ${string_child} private"
  fi
done
check_no_inline_legacy_includes "$wasm_string_builtins"
require_regex_count "$wasm_string_symbol_method" '^enum NativeStringProtocol \{' 1 'private closed String protocol domain'
require_fixed_string_count "$wasm_string_symbol_method" 'fn emit_native_string_protocol(' 1 'sole String protocol owner'
require_fixed_string_count "$wasm_string_symbol_method" 'pub(super) struct NullableStringSymbolMethod {' 1 'retained acquired String method owner'
for string_protocol_non_owner in "$wasm_string_builtins" "$wasm_standard_builtins"; do
  if grep -Eq 'NativeStringProtocol|emit_native_string_protocol\(' "$string_protocol_non_owner"; then
    fail "$string_protocol_non_owner must consume fixed String protocol entries"
  fi
done
for string_symbol_hook_entry in match match_all replace replace_all search split; do
  require_regex_count "$wasm_string_symbol_method" \
    "^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_string_${string_symbol_hook_entry}_builtin[[:space:]]*\(" \
    1 "fixed String protocol entry ${string_symbol_hook_entry}"
  require_fixed_string_count "$wasm_standard_builtins" \
    "self.emit_string_${string_symbol_hook_entry}_builtin(function)?;" 1 \
    "fixed String protocol route ${string_symbol_hook_entry}"
done
require_fixed_string_count "$wasm_regexp_protocol" 'fn emit_native_regexp_flag_getter(' 1 'sole RegExp flag-getter owner'
if grep -Eq 'NativeRegExpFlag|emit_native_regexp_flag_getter\(' "$wasm_standard_builtins"; then
  fail "$wasm_standard_builtins must consume fixed RegExp flag-getter entries"
fi
for regexp_flag_getter_entry in has_indices global ignore_case multiline dot_all unicode unicode_sets sticky; do
  require_regex_count "$wasm_regexp_protocol" \
    "^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_regexp_prototype_${regexp_flag_getter_entry}_getter_builtin[[:space:]]*\(" \
    1 "fixed RegExp flag-getter entry ${regexp_flag_getter_entry}"
  require_fixed_string_count "$wasm_standard_builtins" \
    "self.emit_regexp_prototype_${regexp_flag_getter_entry}_getter_builtin(function)?;" 1 \
    "fixed RegExp flag-getter route ${regexp_flag_getter_entry}"
done

# Completed capture lists and named capture conversion belong to GetSubstitution.
# Both RegExp replacement and literal String replacement consume this one owner.
for capture_owner in SubstitutionCaptureList SubstitutionCaptures NamedSubstitutionCaptures; do
  require_regex_count "$wasm_regexp_substitution" \
    "^pub\(super\) struct ${capture_owner}[({]" 1 'opaque completed substitution input owner'
done
require_regex_count "$wasm_regexp_substitution" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_regexp_get_substitution[[:space:]]*\(' \
  1 'sole semantic GetSubstitution surface'
for substitution_consumer in "$wasm_regexp_protocol" "$wasm_string_literal_replacement_scope"; do
  require_fixed_string_count "$substitution_consumer" 'emit_regexp_get_substitution(' 1 'actual GetSubstitution consumer'
done
require_fixed_string_count "$wasm_string_builtins" 'fn emit_regexp_get_substitution(' 0 'no parent GetSubstitution copy'
check_raw_line_budget "$wasm_regexp_substitution" 520

# The private literal-replacement scope owns first/all selection. The observed
# String protocol child consumes only its two semantic wrappers.
require_regex_count "$wasm_string_literal_replacement_scope" '^enum StringLiteralReplacementScope \{' 1 'private replacement scope domain'
for replacement_non_owner in "$wasm_string_builtins" "$wasm_string_symbol_method" "$wasm_standard_builtins"; do
  if grep -Eq 'StringLiteralReplacementScope|emit_string_replace_literal_from_string_locals\(' "$replacement_non_owner"; then
    fail "$replacement_non_owner must consume fixed literal-replacement wrappers"
  fi
done
for semantic_replacement in emit_string_replace_literal_first_occurrence_from_string_locals emit_string_replace_literal_all_occurrences_from_string_locals; do
  require_regex_count "$wasm_string_literal_replacement_scope" \
    "^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+${semantic_replacement}[[:space:]]*\(" \
    1 'literal-replacement semantic wrapper'
  require_fixed_string_count "$wasm_string_symbol_method" "${semantic_replacement}(" 1 'observed String protocol replacement consumer'
done
check_raw_line_budget "$wasm_string_builtins" 19980
# Whole GC strings/captures and completion cleanup add typed lifecycle code;
# the private literal-replacement owner is 564 lines with a narrow margin.
check_raw_line_budget "$wasm_string_literal_replacement_scope" 600

# T20's Number-to-32-bit residue arithmetic remains in one typed numeric owner.
# GC cutover moved its consumers into private host, String and indexed-property
# children; a frozen call-count/file inventory is not the operation contract.
wasm_uint32_authority="crates/lila-aot-wasm/src/operations.rs"
uint32_modulus='Instruction::F64Const(Ieee64::from(4_294_967_296.0))'
uint32_modulus_files="$(grep -RFl --include='*.rs' "$uint32_modulus" crates/lila-aot-wasm/src || true)"
if [ "$uint32_modulus_files" != "$wasm_uint32_authority" ]; then
  fail "the exact modulo-2^32 implementation must exist only in $wasm_uint32_authority (found: ${uint32_modulus_files:-none})"
fi
require_regex_count "$wasm_uint32_authority" '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_to_uint32_i64_from_number_payload[[:space:]]*\(' 1 'sole typed ToUint32 kernel'
require_file crates/lila-engine/tests/aot_gc_numeric_values.rs
require_file crates/lila-engine/tests/aot_gc_binary_data_entries.rs

# T20's variadic Math extremum walk. The call ABI already owns an arbitrary
# argc/argv domain, so min/max must consume the runtime vector rather than grow
# another reviewed-looking finite prefix. The private enum owns the paired
# identity/reduction decisions; the loop owns every argument conversion.
math_extremum_file="crates/lila-aot-wasm/src/builtins/math.rs"
require_fixed_string_count "$math_extremum_file" 'enum MathExtremum {' 1 'closed Math extremum domain'
require_regex_count "$math_extremum_file" '^[[:space:]]*const fn identity\(&self\) -> f64 \{' 1 'private Math extremum identity owner'
require_regex_count "$math_extremum_file" '^[[:space:]]*fn emit_combine\(' 1 'private Math extremum reduction owner'
require_fixed_string_count "$math_extremum_file" 'emit_math_extremum_builtin(' 3 'Math extremum body and two fixed consumers'
require_fixed_string_count "$math_extremum_file" 'fn emit_math_coerce_number(' 1 'sole whole-completion Math argument coercion owner'
# Argument traversal consumes the actual nonnull ValueArray from BodyEntryLocals.
# Exact scalar opcode/count recipes are obsolete; the retained CLI arity fixture
# below checks the complete variadic behavior and abrupt conversion ordering.
if grep -Eq 'argv_param_local\(|argc_param_local\(|emit_builtin_arg_to_locals\(' "$math_extremum_file"; then
  fail 'Math must consume complete rooted argument vectors instead of scalar ABI accessors'
fi
for math_extremum_entry in min max; do
  require_fixed_string_count "$math_extremum_file" "fn emit_math_${math_extremum_entry}_builtin(" 1 'fixed Math extremum entry'
  require_fixed_string_count "$wasm_standard_builtins" "self.emit_math_${math_extremum_entry}_builtin(function)?" 1 'fixed Standard Math extremum route'
done
require_fixed_string_count \
  crates/lila-cli/tests/cli/language_numerics.rs \
  'fn run_wasm_backend_succeeds_for_math_extremum_argument_reduction()' \
  1 \
  'Math extremum variadic CLI regression'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_math_min_max_arity.js ]; then
  fail 'Math extremum variadic fixture must remain present'
fi

# Passive linear-record layout inventories are retired with the atomic GC model.
# Actual record schemas and their consumed field codecs live in gc_types/.

# Object.getOwnPropertyNames uses two fixed all-string Array operations.
# Enumerable selection belongs to the shared Object keys/entries/values path,
# which observes descriptors lazily after collecting own keys.
array_named_key_owner="crates/lila-aot-wasm/src/builtins/array.rs"
array_named_key_caller="crates/lila-aot-wasm/src/builtins/object.rs"
for retired_array_key_policy in \
  ArrayNamedStringKeySelection \
  emit_array_named_string_props_count \
  emit_array_named_string_props_write_keys \
  emit_array_enumerable_named_string_props_count \
  emit_array_enumerable_named_string_props_write_keys
do
  require_fixed_string_count "$array_named_key_owner" "$retired_array_key_policy" 0 'retired eager Array key selection'
  require_fixed_string_count "$array_named_key_caller" "$retired_array_key_policy" 0 'retired eager Object key selection'
done
require_fixed_string_count "$array_named_key_caller" 'fn emit_native_object_own_keys_builtin(' 1 'actual native own-key algorithm owner'
require_fixed_string_count "$array_named_key_caller" 'self.emit_native_object_own_keys_builtin(NativeOwnKeyKind::String' 1 'fixed own-name entry'
require_fixed_string_count "$array_named_key_caller" 'self.emit_native_object_own_keys_builtin(NativeOwnKeyKind::Symbol' 1 'fixed own-symbol entry'

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
  require_fixed_string_count "$array_sort_owner" "pub(crate) fn ${array_sort_wrapper}(" 1 "fixed Array sort entry $array_sort_wrapper"
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
require_fixed_string_count "$array_find_owner" 'self.compile_array_find_with_kind(' 8 'fixed Array and TypedArray find entry calls'
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
# Whole descriptor acquisition and validated publication use the shared actual
# descriptor kernel; the old Arguments-specialized carriers are retired.
require_fixed_string_count "$object_define_property_file" 'self.emit_to_property_descriptor(' 1 'whole descriptor acquisition caller'
require_fixed_string_count "$object_define_property_file" 'self.emit_object_define_entry_validated(' 1 'validated descriptor publication caller'
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

# 10.5.5 Proxy [[GetOwnProperty]] is the descriptor owner's private child: the
# parent declares it privately and calls its one entry, and the child owns the
# live-slot read, the trap call, the recursive targetDesc and the conversion.
object_get_own_descriptor_proxy_file="crates/lila-aot-wasm/src/builtins/object/get_own_property_descriptor/proxy.rs"
require_file "$object_get_own_descriptor_proxy_file"
check_no_inline_legacy_includes "$object_get_own_descriptor_proxy_file"
require_exact_line_count "$object_get_own_descriptor_file" 'mod proxy;' 1 'private Proxy [[GetOwnProperty]] module declaration'
require_fixed_string_count "$object_get_own_descriptor_file" 'builder.emit_proxy_get_own_property_descriptor(' 1 'Proxy [[GetOwnProperty]] step call'
require_fixed_string_count "$object_get_own_descriptor_proxy_file" 'pub(super) fn emit_proxy_get_own_property_descriptor(' 1 'Proxy [[GetOwnProperty]] step entry'
require_fixed_string_count "$object_get_own_descriptor_proxy_file" 'self.emit_to_property_descriptor(' 1 'Proxy trap-result ToPropertyDescriptor call'
require_fixed_string_count "$object_get_own_descriptor_proxy_file" 'self.emit_complete_property_descriptor(' 1 'Proxy trap-result CompletePropertyDescriptor call'
require_fixed_string_count "$object_get_own_descriptor_proxy_file" 'self.emit_native_descriptor_object(' 1 'Proxy result FromPropertyDescriptor call'
if grep -Eq 'emit_function_or_proxy_call|emit_to_property_descriptor|getOwnPropertyDescriptor trap' "$object_get_own_descriptor_file"; then
  fail "$object_get_own_descriptor_file must leave the Proxy trap step to its private child"
fi
check_raw_line_budget "$object_get_own_descriptor_proxy_file" 560

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
require_regex_count "$own_descriptor_predicate_file" '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_.*_builtin[[:space:]]*\(' 3 'own-descriptor predicate semantic wrapper visibility'
own_descriptor_predicate_body="$(braced_rust_item_source "$own_descriptor_predicate_file" '^fn[[:space:]]+compile_object_own_descriptor_predicate_builtin[[:space:]]*[(]')"
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' <<<"$own_descriptor_predicate_body"; then
  fail 'own-descriptor predicate compiler must keep its closed source/order/projection decisions exhaustive'
fi
require_text_regex_count "$own_descriptor_predicate_body" 'self\.emit_proxy_target_own_descriptor[(]' 1 'sole completed GetOwnProperty descriptor consumer'
require_text_regex_count "$own_descriptor_predicate_body" 'descriptor\.emit_enumerable_i32[(]' 1 'non-observing completed enumerable projection'
# The live two source arms fuse the prototype forms while preserving each
# required coercion order. No canonical public-property relookup is involved.
if ! awk '
  /OwnDescriptorPredicateBuiltin::ObjectHasOwn =>/ { object_arm = NR }
  /emit_value_to_object_locals/ && object_arm && !prototype_arm && !object_conversion { object_conversion = NR }
  /emit_value_to_property_key_locals/ && object_arm && !prototype_arm && !object_key { object_key = NR }
  /OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty/ && object_key && !prototype_arm { prototype_arm = NR }
  /emit_value_to_property_key_locals/ && prototype_arm && !prototype_key { prototype_key = NR }
  /emit_value_to_object_locals/ && prototype_arm && !prototype_conversion { prototype_conversion = NR }
  END { exit !(object_arm && object_conversion && object_key && prototype_arm && prototype_key && prototype_conversion \
    && object_arm < object_conversion && object_conversion < object_key && object_key < prototype_arm \
    && prototype_arm < prototype_key && prototype_key < prototype_conversion) }
' <<<"$own_descriptor_predicate_body"; then
  fail 'own-descriptor predicates must preserve object-first and prototype key-first conversion order'
fi
for raw_own_predicate_scan in HEAP_ PROXY_HANDLER_ emit_object_own_property_present emit_known_array_index_from_property_key load_i64_to_local_from_offset; do
  if grep -Fq "$raw_own_predicate_scan" <<<"$own_descriptor_predicate_body"; then
    fail "own-descriptor predicate compiler must not regain raw storage scans through $raw_own_predicate_scan"
  fi
done

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

# T10's Object keys/entries/values policy has one private owner. The parent and
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
require_fixed_string_count \
  "$enumerable_own_properties_file" \
  'compile_object_enumerable_own_properties_builtin(' \
  4 \
  'private enumerable-own-properties compiler definition and wrapper calls'
require_regex_count \
  "$enumerable_own_properties_file" \
  '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+compile_object_(keys|entries|values)_builtin[[:space:]]*\(' \
  3 \
  'enumerable-own-properties semantic wrapper visibility'
enumerable_own_properties_body="$(sed -n \
  '/^    fn compile_object_enumerable_own_properties_builtin(/,/^    pub(in crate::builtins) fn compile_object_keys_builtin(/p' \
  "$enumerable_own_properties_file")"
# The shared emitter exhaustively selects diagnostics and result shape. The
# complete result-array owner, not the enum, carries the consuming publication.
if grep -Eq '_ =>' <<<"$enumerable_own_properties_body"; then
  fail 'enumerable-own-properties must exhaustively match its closed operation domain'
fi
require_regex_count "$enumerable_own_properties_file" '^pub\(super\) struct NativeObjectResultArray \{' 1 'private completed native Object result-array owner'
result_array_attributes="$(rust_item_attributes_source "$enumerable_own_properties_file" '^pub[(]super[)][[:space:]]+struct[[:space:]]+NativeObjectResultArray[[:space:]]*[{]')"
if grep -Eq '#\[derive\([^]]*(Clone|Copy)' <<<"$result_array_attributes"; then
  fail 'NativeObjectResultArray must retain its consuming publication capability'
fi
require_regex_count "$enumerable_own_properties_file" '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+finish[[:space:]]*\(' 1 'consuming Object result-array publisher'
result_array_finish="$(braced_rust_item_source "$enumerable_own_properties_file" '^pub[(]super[)][[:space:]]+fn[[:space:]]+finish[[:space:]]*[(]')"
require_text_regex_count "$result_array_finish" '^[[:space:]]*self,$' 1 'result-array finish consumes its rooted owner'
require_text_regex_count "$enumerable_own_properties_body" 'self\.emit_object_own_property_keys[(]' 1 'completed Object own-key snapshot'
require_text_regex_count "$enumerable_own_properties_body" 'self\.emit_proxy_target_own_descriptor[(]' 1 'lazy completed Object descriptor observation'

for enumerable_own_properties_wrapper_spec in \
  'compile_object_keys_builtin|compile_object_entries_builtin|EnumerableOwnProperties::Keys' \
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
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' <<<"$integrity_test_body"; then
  fail 'integrity-test compiler must keep its closed policy exhaustive'
fi
require_text_regex_count "$integrity_test_body" 'self\.emit_object_is_extensible[(]' 1 'TestIntegrityLevel complete extensibility observation'
require_text_regex_count "$integrity_test_body" 'self\.emit_proxy_target_own_descriptor[(]' 1 'TestIntegrityLevel completed descriptor observation'

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
if grep -Eq '^[[:space:]]*_ =>|unreachable!\(' <<<"$prototype_lookup_body"; then
  fail 'prototype-lookup compiler must keep its closed getter/setter selection exhaustive'
fi
require_text_regex_count "$prototype_lookup_body" 'self\.emit_proxy_target_own_descriptor[(]' 1 'completed prototype own-descriptor observation'
require_text_regex_count "$prototype_lookup_body" 'self\.emit_object_get_prototype_of[(]' 1 'complete prototype traversal authority'

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

# HasProperty's public wrapper and private helper body consume the registered
# typed input/key/Environment and complete result, without a parallel tagged ABI.
require_fixed_string_count crates/lila-aot-wasm/src/objects/has_property.rs 'pub(crate) fn emit_object_has_property_i32(' 1 'actual whole HasProperty entry'
require_fixed_string_count crates/lila-aot-wasm/src/objects/has_property.rs 'pub(crate) fn compile_object_has_property_helper(' 1 'actual shared HasProperty body'
require_fixed_string_count crates/lila-aot-wasm/src/objects/has_property.rs 'fn emit_has_property_dispatch(' 1 'private HasProperty dispatch'
require_fixed_string_count \
  crates/lila-engine/src/lib.rs \
  'fn wasm_backend_has_property_dispatches_every_live_exotic_branch()' \
  1 \
  'complete HasProperty runtime regression'

# Recursive target descriptor acquisition has one consumed completed owner.
# Its real native GPD call runs before nonobserving private stored-field reads.
wasm_proxy_target_descriptor="crates/lila-aot-wasm/src/objects/proxy_target_descriptor.rs"
require_file "$wasm_proxy_target_descriptor"
require_exact_line_count crates/lila-aot-wasm/src/objects.rs 'mod proxy_target_descriptor;' 1 'private completed target descriptor module'
check_no_inline_legacy_includes "$wasm_proxy_target_descriptor"
require_fixed_string_count "$wasm_proxy_target_descriptor" 'pub(crate) struct CompletedProxyTargetDescriptor {' 1 'opaque completed descriptor owner'
for completed_descriptor_entry in emit_proxy_target_own_descriptor emit_proxy_target_descriptor_fact; do
  require_fixed_string_count "$wasm_proxy_target_descriptor" "fn ${completed_descriptor_entry}(" 1 'completed target descriptor producer'
done
# Meaningful Get/Set/Reflect compiler fixtures remain attached below.
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
require_regex_count crates/lila-aot-wasm/src/functions/argument_vectors.rs '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_builtin_arg_is_present_i32[[:space:]]*\(' 1 'actual argument-vector optional-presence authority'
require_tree_regex_count crates/lila-aot-wasm/src '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_builtin_arg_is_present_i32[[:space:]]*\(' 1 'sole builtin argument-presence owner'
# Reflect's two consumed routes cover receiver defaults and Construct newTarget.
require_fixed_string_count crates/lila-aot-wasm/src/builtins/reflect.rs "$builtin_arg_presence" 2 'Reflect optional-argument presence consumers'
require_active_wasm_cli_rust_test \
  "$proxy_set_cli" \
  run_wasm_backend_distinguishes_omitted_reflect_optional_arguments \
  'Reflect optional-argument presence CLI regression'
reflect_optional_presence_cli_test="$(braced_rust_item_source "$proxy_set_cli" '^fn[[:space:]]+run_wasm_backend_distinguishes_omitted_reflect_optional_arguments[[:space:]]*[(]')"
require_text_regex_count "$reflect_optional_presence_cli_test" 'fixture_path\("wasm_reflect_optional_argument_presence\.js"\)' 1 'Reflect optional-argument presence fixture wiring'
if [ ! -f crates/lila-cli/tests/fixtures/wasm_reflect_optional_argument_presence.js ]; then
  fail 'Reflect optional-argument presence fixture must remain present'
fi
# All six property entries share the one complete key conversion branch.
require_fixed_string_count crates/lila-aot-wasm/src/builtins/reflect.rs 'emit_value_to_property_key_locals(' 1 'shared Reflect complete PropertyKey authority'
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

# Descriptor values/getters/setters are read as complete values through the
# completed target descriptor owner. Raw-zero and offset projections are gone.

# One live Proxy-slot reader roots its concrete record and exposes opaque whole
# target/handler values to the operation-specific revocation and Call routes.
require_fixed_string_count crates/lila-aot-wasm/src/objects.rs 'pub(crate) fn emit_load_live_proxy_slots(' 1 'live Proxy-slot reader authority'
require_tree_regex_count crates/lila-aot-wasm/src '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_load_live_proxy_slots[[:space:]]*\(' 1 'sole live Proxy-slot reader'
# Reflect entries delegate to their shared whole-value property algorithms;
# neither handler-offset whitelists nor duplicated inline trap recipes apply.

# Native optional Get and terminal Delete consume one rooted prefix pipeline.
# Only its private destination changes the final operation or shorted result.
wasm_expressions="crates/lila-aot-wasm/src/expressions.rs"
wasm_optional_chain="crates/lila-aot-wasm/src/expressions/optional_chain.rs"
require_file "$wasm_optional_chain"
require_exact_line_count "$wasm_expressions" 'mod optional_chain;' 1 'private native optional pipeline attachment'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+optional_chain;' "$wasm_expressions"; then
  fail "$wasm_expressions must keep its optional pipeline private"
fi
require_regex_count "$wasm_optional_chain" "^enum TerminalDestination<'a>" 1 'private native optional destination domain'
require_fixed_string_count "$wasm_optional_chain" 'fn compile_optional_chain_to_destination(' 1 'one rooted native optional prefix body'
require_fixed_string_count "$wasm_optional_chain" 'self.compile_optional_chain_to_destination(' 2 'Get and Delete consume the same prefix pipeline'
for native_optional_entry in compile_optional_property_chain_to_value compile_delete_optional_property_chain_to_value; do
  require_fixed_string_count "$wasm_optional_chain" "pub(crate) fn ${native_optional_entry}(" 1 'actual native optional destination entry'
  require_fixed_string_count "$wasm_expressions" "fn ${native_optional_entry}(" 0 'no parent optional pipeline copy'
done
require_fixed_string_count "$wasm_expressions" 'compile_optional_property_chain_to_value(' 2 'actual optional Value and captured Call dispatch'
require_fixed_string_count "$wasm_expressions" 'self.compile_delete_optional_property_chain_to_value(' 1 'actual terminal optional Delete dispatch'
require_fixed_string_count "$wasm_optional_chain" 'self.compile_delete_property_i32(' 1 'terminal optional Delete uses the existing whole property algorithm'
check_no_inline_legacy_includes "$wasm_optional_chain"
check_raw_line_budget "$wasm_optional_chain" 300
check_raw_line_budget "$wasm_expressions" 1980
wasm_regexp_expressions="crates/lila-aot-wasm/src/expressions/regexp_program.rs"
require_file "$wasm_regexp_expressions"
require_exact_line_count "$wasm_expressions" 'mod regexp_program;' 1 'private RegExp expression compiler owner'
for regexp_expression_entry in emit_validated_regexp_program emit_regexp_program_slots emit_runtime_regexp_program_slots emit_regexp_compiler_failures compile_regexp_program_candidate_hook emit_gc_string_contains_ascii_byte_i32 emit_regexp_program_with_compatible_flags compile_regexp_literal_to_value; do
  require_fixed_string_count "$wasm_regexp_expressions" "fn ${regexp_expression_entry}(" 1 'RegExp expression algorithm owner'
  require_fixed_string_count "$wasm_expressions" "fn ${regexp_expression_entry}(" 0 'no parent RegExp expression algorithm copy'
done
require_fixed_string_count "$wasm_regexp_expressions" 'pub(super) fn compile_regexp_literal_to_value(' 1 'RegExp literal entry stays inside expressions'
require_fixed_string_count "$wasm_expressions" 'self.compile_regexp_literal_to_value(' 1 'RegExp expression dispatch consumes its private owner'
check_no_inline_legacy_includes "$wasm_regexp_expressions"
check_raw_line_budget "$wasm_regexp_expressions" 510

# T02 gives T09's private-element environment, storage and access lifecycle one
# private backend owner. Keep the complete family together: widening only some
# methods back into objects.rs would recreate the shared ownership surface this
# boundary removes.
wasm_objects="crates/lila-aot-wasm/src/objects.rs"
wasm_private_elements="crates/lila-aot-wasm/src/objects/private_elements.rs"
require_file "$wasm_private_elements"
require_exact_line_count "$wasm_objects" 'mod private_elements;' 1 'private-elements module declaration'
require_regex_count \
  "$wasm_objects" \
  '^(pub(\([^)]*\))?[[:space:]]+)?mod[[:space:]]+private_elements;' \
  1 \
  'private-elements module declarations'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+private_elements;' "$wasm_objects"; then
  fail "$wasm_objects must keep private_elements private"
fi

# Token lookup, table operations and language reads/writes stay in the private
# GC owner; raw carrier variants and the old method count are not contracts.
for private_element_entry in emit_private_name_token_to_local emit_private_element_find emit_private_brand_has_i32 emit_private_brand_add emit_private_field_define compile_private_read_to_locals compile_private_write_to_locals; do
  require_fixed_string_count "$wasm_private_elements" "fn ${private_element_entry}(" 1 'actual private-element algorithm owner'
  require_fixed_string_count "$wasm_objects" "fn ${private_element_entry}(" 0 'no parent private-element algorithm copy'
done
require_fixed_string_count "$wasm_private_elements" 'fn compile_private_field_define_helper(' 1 'complete private-field definition helper owner'
require_fixed_string_count "$wasm_private_elements" 'PrivateFieldDefineArguments::new(' 1 'private-field definition uses its typed helper call'
require_fixed_string_count "$wasm_objects" 'fn compile_private_field_define_helper(' 0 'no parent private-field helper copy'

check_no_inline_legacy_includes "$wasm_private_elements"
# The current whole-GC object owner is 4,178 lines. Literal property definition
# is one 235-line private body consumed by ordinary and resumed construction.
wasm_object_literal_property="crates/lila-aot-wasm/src/objects/object_literal_property.rs"
require_file "$wasm_object_literal_property"
require_exact_line_count "$wasm_objects" 'mod object_literal_property;' 1 'private shared native literal property owner'
if grep -Eq '^pub(\([^)]*\))?[[:space:]]+mod[[:space:]]+object_literal_property;' "$wasm_objects"; then
  fail "$wasm_objects must keep literal property definitions private"
fi
require_fixed_string_count "$wasm_object_literal_property" 'pub(super) fn emit_object_literal_property(' 1 'sole complete native literal property body'
require_fixed_string_count "$wasm_objects" 'fn emit_object_literal_property(' 0 'no parent literal property body copy'
require_fixed_string_count "$wasm_objects" 'self.emit_object_literal_property(' 1 'ordinary literal consumes the shared property body'
require_fixed_string_count "$wasm_object_literal_property" 'self.emit_object_literal_property(' 1 'retained object consumes the same property body'
require_fixed_string_count "$wasm_object_literal_property" 'pub(crate) fn compile_object_property_definition_payload(' 1 'actual checked retained-object definition entry'
require_fixed_string_count "$wasm_expressions" 'self.compile_object_property_definition_payload(' 1 'actual retained-object expression dispatch'
check_no_inline_legacy_includes "$wasm_object_literal_property"
check_raw_line_budget "$wasm_objects" 4250
check_raw_line_budget "$wasm_object_literal_property" 250
check_raw_line_budget "$wasm_private_elements" 1050

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
# The captured ParameterMap result is a nullable BindingCell reference. Its
# typed read/write owner preserves alias identity independently of descriptors.
for arguments_mapping_entry in emit_arguments_index_mapping emit_arguments_parameter_map_read emit_arguments_parameter_map_write; do
  require_fixed_string_count "$wasm_arguments_index_mapping" "fn ${arguments_mapping_entry}(" 1 'actual Arguments binding-cell owner'
  require_fixed_string_count "$wasm_functions" "fn ${arguments_mapping_entry}(" 0 'no parent Arguments mapping copy'
done

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

# Reserved storage cannot be used as the completed Array prototype. The GC
# slot/value witnesses and consuming transition are private to this child.
for created_array_transition in reserve_realm_array_prototype_local emit_initialize_realm_array_prototype emit_define_realm_array_prototype_data_with_flags emit_bind_realm_array_constructor_prototype release_realm_array_prototype_local; do
  require_fixed_string_count "$wasm_created_realm_array_prototype" "fn ${created_array_transition}(" 1 'actual created-Realm Array lifecycle owner'
  require_fixed_string_count "$wasm_functions" "fn ${created_array_transition}(" 0 'no parent Array prototype lifecycle copy'
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
# The exhaustive slot selector supplies a typed non-Array intrinsic or the
# separately typed Array root. No fixed variant count or byte offset is needed.
require_fixed_string_count "$wasm_required_resolved_realm_ordinary_prototype" 'pub(super) struct ResolvedRealmOrdinaryPrototypeLocal(ValueLocals);' 1 'completed whole prototype witness'
require_fixed_string_count "$wasm_required_resolved_realm_ordinary_prototype" 'const fn slot(self) -> Option<NonArrayRealmIntrinsicSlot>' 1 'closed intrinsic selector owner'
for required_prototype_entry in emit_get_prototype_from_constructor emit_required_function_realm_ordinary_prototype_completion emit_required_new_target_realm_ordinary_prototype; do
  require_fixed_string_count "$wasm_required_resolved_realm_ordinary_prototype" "fn ${required_prototype_entry}(" 1 'actual constructor/prototype completion owner'
  require_fixed_string_count "$wasm_functions" "fn ${required_prototype_entry}(" 0 'no parent prototype algorithm copy'
done

check_no_inline_legacy_includes "$wasm_required_resolved_realm_ordinary_prototype"
# Rooted resolved-Realm/prototype completion now measures 249 formatted lines.
check_raw_line_budget "$wasm_required_resolved_realm_ordinary_prototype" 270

# The active function Realm's Array prototype is a one-shot proof: one private
# child constructs it, and five consumers install it with an Array payload.
# Iterator-toArray and Proxy dispatch retain their original consumers; Object
# owns two key-array consumers and Reflect owns one.
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
# The current function Realm selects a rooted concrete Array prototype; the
# actual allocator consumes that witness before publishing the result header.
for current_array_entry in emit_load_current_function_realm_array_prototype emit_alloc_array_with_current_function_realm_prototype; do
  require_fixed_string_count "$wasm_current_function_realm_array_prototype" "fn ${current_array_entry}(" 1 'current function Realm Array owner'
  require_fixed_string_count "$wasm_functions" "fn ${current_array_entry}(" 0 'no parent current-Realm Array lifecycle copy'
done
# Called-Realm result semantics are exercised by the maintained finite Array,
# Object, Reflect and Intl Engine/CLI cohorts, rather than allocation call counts.

check_no_inline_legacy_includes "$wasm_current_function_realm_array_prototype"
check_raw_line_budget "$wasm_current_function_realm_array_prototype" 110

# GetFunctionRealm's rooted result lifecycle has one private owner. The parent
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

# Each resolved Realm result stays rooted until its consuming operation clears it.

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

# Every catch/finally clause seeds its own statement-list completion after
# preserving the incoming completion. Ordinary try clauses also seed undefined
# for TryStatement's UpdateEmpty, since empty blocks preserve the list value.
statement_completion_owner="crates/lila-aot-wasm/src/control_flow/statement_completion.rs"
require_file "$statement_completion_owner"
require_regex_count "$statement_completion_owner" \
  '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_statement_result\(' 1 'sole complete statement-result initialization owner'
require_fixed_string_count "$statement_completion_owner" 'self.completion().value().set_undefined(function);' 1 'canonical empty statement value'
require_fixed_string_count "$statement_completion_owner" 'self.set_completion_kind(CompletionKind::Normal, function);' 1 'normal empty statement completion'
try_clause_seed_pattern='^[[:space:]]*self\.emit_statement_result\(function\);[[:space:]]*$'
for try_clause_seed_owner in \
  'compile_try_catch emit_generator_state_in_range 2' \
  'compile_generator_try_catch compile_generator_try_finally 1' \
  'compile_generator_try_finally compile_generator_try_catch_finally 1' \
  'compile_generator_try_catch_finally compile_async_try_catch 2' \
  'compile_async_try_catch compile_async_try_catch_finally 1' \
  'compile_async_try_catch_finally compile_async_try_finally 2' \
  'compile_async_try_finally compile_try_finally 1' \
  'compile_try_finally compile_async_disposable_scope 2' \
  'compile_try_catch_finally compile_while 3'
do
  set -- $try_clause_seed_owner
  try_clause_seed_owner_source="$(sed -n "/fn ${1}(/,/fn ${2}(/p" "$wasm_control_flow")"
  require_text_regex_count \
    "$try_clause_seed_owner_source" \
    "$try_clause_seed_pattern" \
    "$3" \
    "$1 empty-completion clause-entry seeds"
done

require_file crates/lila-aot-wasm/src/builtins/intl_supported_values.rs
require_file crates/lila-aot-wasm/src/data/intl_supported_values.rs
require_fixed_string_count crates/lila-aot-wasm/src/builtins/mod.rs 'mod intl_supported_values;' 1 'actual supportedValuesOf emitter registration'
require_fixed_string_count crates/lila-aot-wasm/src/data.rs 'mod intl_supported_values;' 1 'checked static catalogue owner registration'
wasm_normalization_data="crates/lila-aot-wasm/src/data/normalization.rs"
require_file "$wasm_normalization_data"
require_exact_line_count crates/lila-aot-wasm/src/data.rs 'mod normalization;' 1 'private Unicode normalization table owner'
require_fixed_string_count "$wasm_normalization_data" "pub(super) fn tables() -> &'static NormalizationTables" 1 'shared normalization table authority'
require_fixed_string_count crates/lila-aot-wasm/src/data.rs 'normalization::tables()' 1 'data construction consumes the normalization authority'
check_no_inline_legacy_includes "$wasm_normalization_data"
check_raw_line_budget "$wasm_normalization_data" 170
require_fixed_string_count crates/lila-aot-wasm/src/builtins/standard.rs 'self.emit_intl_supported_values_of(function)?;' 1 'actual supportedValuesOf dispatch'
require_tree_regex_count crates/lila-aot-wasm/src '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_intl_supported_values_of[[:space:]]*\(' 1 'supportedValuesOf sole emitter owner'
require_fixed_string_count crates/lila-aot-wasm/src/emit.rs 'let intl_selection = lila_intl::IntlDataSelection::new(intl_profile.clone());' 1 'one selected Intl owner outside builtin retries'
require_fixed_string_count crates/lila-aot-wasm/src/lib.rs 'pub use emit::emit_with_intl_profile;' 1 'actual selected Intl compiler entry'
require_fixed_string_count crates/lila-aot-wasm/src/emit/module_assembly.rs 'string_pool.check_intl_supported_values()?' 1 'consumed checked selected catalogue admission'
require_fixed_string_count crates/lila-aot-wasm/src/emit/module_assembly.rs 'selected.identity().artifact_identity()' 1 'sole selected artifact identity owner'
require_fixed_string_count crates/lila-aot-wasm/src/emit/module_assembly.rs 'for (name, data) in selected.component_sections()' 1 'same selected physical frame inventory consumed by export and emission'
require_fixed_string_count crates/lila-aot-wasm/src/emit/module_assembly.rs 'if let Some(services) = selected.service_selection()' 1 'artifact carries its checked public service authority'
require_fixed_string_count crates/lila-aot-wasm/src/data/intl_supported_values.rs 'pub(crate) enum CompiledSupportedValuesTable {' 1 'closed per-key selected primitive catalogue owner'
require_exact_line_count crates/lila-aot-wasm/src/data/intl_supported_values.rs '    Available(Arc<SupportedValuesList>),' 1 'available keys retain their admitted catalogue'
require_exact_line_count crates/lila-aot-wasm/src/data/intl_supported_values.rs '    Unavailable,' 1 'omitted key data has explicit unavailable authority'
require_fixed_string_count crates/lila-aot-wasm/src/data/intl_supported_values.rs 'pub(crate) fn source(&self) -> Option<Arc<SupportedValuesList>>' 1 'catalogue access preserves physical absence'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/string/unicode_operations.rs 'b.strings.intl_default_locale()?' 1 'selected Unicode case DefaultLocale consumer'
require_tree_regex_count crates/lila-aot-wasm/src/builtins 'embedded_intl_data_identity' 0 'no default identity escape from selected native consumers'

# Real filtered List data has one crate-private catalogue owner and a private
# exact-row exporter. Both production and admission consume this same closure;
# Duration resolves its retained List association through the selected owner.
intl_list_image="crates/lila-intl/src/list_image.rs"
intl_list_projection="crates/lila-intl/src/list_image/projection.rs"
intl_list_projection_export="crates/lila-intl/src/list_image/projection/export.rs"
require_file "$intl_list_projection"
require_file "$intl_list_projection_export"
require_exact_line_count "$intl_list_image" 'pub(crate) mod projection;' 1 'crate-private actual List projection owner attachment'
require_exact_line_count "$intl_list_projection" 'mod export;' 1 'private exact List row exporter attachment'
for list_projection_entry in produce admit; do
  require_fixed_string_count "$intl_list_projection" "pub(super) fn ${list_projection_entry}(" 1 'actual projected List production/admission owner'
  require_fixed_string_count "$intl_list_image" "projection::${list_projection_entry}(" 1 'actual selected List projection consumer'
done
require_fixed_string_count "$intl_list_projection_export" 'pub(super) fn export_rows(' 1 'actual exact List row export entry'
require_fixed_string_count "$intl_list_projection" 'export::export_rows(' 1 'physical selected List rows consume the exporter'
require_fixed_string_count crates/lila-intl/src/list_format/profiles.rs 'use crate::list_image::projection::ListCatalogue;' 1 'List admission consumes the projected catalogue'
require_fixed_string_count crates/lila-intl/src/list_format/profiles.rs 'pub(crate) fn resolve_duration_locale(' 1 'selected private Duration List association owner'
require_fixed_string_count crates/lila-intl/src/duration_format/profiles.rs 'lists.resolve_duration_locale(' 1 'Duration consumes the selected List association'
check_no_inline_legacy_includes "$intl_list_projection"
check_no_inline_legacy_includes "$intl_list_projection_export"
check_raw_line_budget "$intl_list_image" 250
check_raw_line_budget "$intl_list_projection" 435
check_raw_line_budget "$intl_list_projection_export" 60
check_raw_line_budget crates/lila-intl/src/list_format/profiles.rs 300
check_raw_line_budget crates/lila-intl/src/duration_format/profiles.rs 375

# RelativeTime's native projection owns actual complete selected rows. Byte
# admission recomputes the producer closure before minting public profiles;
# the complete Number foundation remains the numeric operation owner.
intl_relative_image="crates/lila-intl/src/relative_time_image.rs"
intl_relative_projection="crates/lila-intl/src/relative_time_image/projection.rs"
intl_selection="crates/lila-intl/src/selection.rs"
require_file "$intl_relative_projection"
require_exact_line_count "$intl_relative_image" 'pub(crate) mod projection;' 1 'private actual RelativeTime projection attachment'
for relative_projection_entry in produce admit; do
  require_fixed_string_count "$intl_relative_projection" "pub(super) fn ${relative_projection_entry}" 1 'actual selected RelativeTime production/admission owner'
  require_fixed_string_count "$intl_relative_image" "projection::${relative_projection_entry}(" 1 'consumed RelativeTime projection entry'
done
require_fixed_string_count "$intl_relative_projection" 'if payload != expected.as_slice()' 1 'reader requires the exact pinned projection derivation'
require_fixed_string_count "$intl_relative_projection" 'pub(crate) struct RelativeCatalogue {' 1 'private producer-minted selected RelativeTime catalogue'
require_fixed_string_count crates/lila-intl/src/relative_time_format/profiles.rs 'catalogue: &crate::relative_time_image::projection::RelativeCatalogue,' 1 'selected profiles require actual admitted row provenance'
require_fixed_string_count "$intl_relative_image" 'RelativeProfiles::from_projected_json(source, &catalogue, &numbers.profiles())' 1 'image publishes selected templates with the actual Number owner'
check_no_inline_legacy_includes "$intl_relative_projection"
# Exact native admission also validates its retained private Number domain.
check_raw_line_budget "$intl_relative_projection" 335
require_file crates/lila-intl/src/relative_time_image/projection/tests.rs

# DisplayNames projection removes unselected locale/currency data and
# unreachable typed pools. Exact rederivation mints the private native catalogue.
intl_display_image="crates/lila-intl/src/display_names_image.rs"
intl_display_projection="crates/lila-intl/src/display_names_image/projection.rs"
intl_display_profiles="crates/lila-intl/src/display_names/profiles.rs"
require_file "$intl_display_projection"
require_exact_line_count "$intl_display_image" 'mod projection;' 1 'private selected DisplayNames physical producer/admission'
require_exact_line_count "$intl_display_image" 'pub(crate) use projection::DisplayNamesCatalogue;' 1 'consumed private borrowed native catalogue'
for display_projection_entry in produce admit; do
  require_regex_count "$intl_display_projection" "^[[:space:]]*pub\\(super\\) fn ${display_projection_entry}[<(]" 1 'real selected DisplayNames factory/admission body, distinct from the currency producer'
  require_fixed_string_count "$intl_display_image" "projection::${display_projection_entry}(" 1 'actual image consumes selected DisplayNames body'
done
require_fixed_string_count "$intl_display_projection" "pub(crate) struct DisplayNamesCatalogue<'a>" 1 'only exact admission mints a borrowed complete-row catalogue'
require_fixed_string_count "$intl_display_projection" 'if payload != expected.as_slice()' 1 'actual reader requires complete source rederivation'
require_fixed_string_count "$intl_display_projection" 'raw["name_pool"] = serde_json::Value::Array(selected_pools);' 1 'locale producer physically publishes reachable typed pools'
require_fixed_string_count "$intl_display_projection" 'raw["name_pool"] = serde_json::Value::Array(pools);' 1 'currency producer physically publishes the filtered remapped pools'
require_fixed_string_count "$intl_display_image" 'projection::produce_data(' 1 'actual paired currency projection consumer'
require_fixed_string_count "$intl_display_image" 'pub(crate) fn currency_codes(&self)' 1 'same admitted currency domain is consumed by provider admission'
require_fixed_string_count "$intl_display_projection" 'source_pool_indices,' 1 'source association order is bound by descriptor'
require_fixed_string_count "$intl_display_profiles" "catalogue: &crate::display_names_image::DisplayNamesCatalogue<'_>," 1 'actual native decoder consumes private admitted catalogue'
require_fixed_string_count "$intl_display_profiles" 'serde_json::from_slice(catalogue.bytes())' 1 'selected native operation tables consume actual projected bytes'
check_no_inline_legacy_includes "$intl_display_projection"
# Currency and locale production share this catalogue; retain private authority
# checks instead of the old locale-only line ceilings.
intl_display_catalogue_fields="$(sed -n "/^pub(crate) struct DisplayNamesCatalogue<'a> {$/,/^}$/p" "$intl_display_projection")"
require_text_regex_count "$intl_display_catalogue_fields" '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+' 0 'no caller-manufactured sparse currency catalogue'
require_fixed_string_present "$intl_display_profiles" 'owner.currency_codes()' 'native sparse Currency admission consumes the same closed domain'
require_file crates/lila-intl/src/display_names_image/projection/tests.rs
require_fixed_string_count crates/lila-intl/src/display_names_image/projection/tests.rs 'fn selected_rows_keep_complete_reachable_pools_and_the_real_fallback_catalogue()' 1 'complete six-domain/three-width/fallback/pool closure control'
require_fixed_string_count crates/lila-intl/src/display_names_image/projection/tests.rs 'fn self_consistent_changed_rows_pools_and_provenance_cannot_claim_the_projection()' 1 'actual rederived payload damage refusal control'

# One checked Custom identity composes actual producers for eight independent
# locale selectors, data dimensions and a closed public service/frame owner.
require_exact_line_count "$intl_selection" '    CustomProjection(CustomIntlProfile),' 1 'closed combined compilation selection'
require_exact_line_count "$intl_selection" 'pub struct CustomIntlProfile {' 1 'sole combined Custom input owner'
intl_custom_fields="$(sed -n '/^pub struct CustomIntlProfile {$/,/^}$/p' "$intl_selection")"
require_text_regex_count "$intl_custom_fields" '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+' 0 'private validated combined input fields'
intl_custom_constructor="$(sed -n '/^impl CustomIntlProfile {$/,/^pub enum InvalidCustomIntlProfile {/p' "$intl_selection")"
require_text_regex_count "$intl_custom_constructor" '^[[:space:]]+pub fn new\(' 1 'single consumed combined Custom constructor'
require_fixed_string_count "$intl_selection" 'if list_locales.is_none()' 1 'no-filter admission checks the actual List field'
for intl_locale_field in relative_time display_names duration number date_time collator segmenter; do
  require_fixed_string_count "$intl_selection" "&& ${intl_locale_field}_locales.is_none()" 1 'no-filter admission checks every independent locale field'
  require_fixed_string_count "$intl_selection" "pub fn ${intl_locale_field}_locales(&self) -> Option<&[LocaleId]>" 1 'actual checked component locale getter'
  require_fixed_string_count crates/lila-engine/src/lib.rs "hash_optional_program_cache_locales(&mut hash, selection.${intl_locale_field}_locales())" 1 'cache consumes each independent locale selector'
done
for intl_locale_factory in ListDataImage RelativeTimeDataImage DisplayNamesDataImage DurationDataImage NumberProfilesDataImage DateTimeDataImage CollatorDataImage SegmenterDataImage; do
  require_fixed_string_present "$intl_selection" "${intl_locale_factory}::for_custom_projection(" 'selected bundle consumes the actual component producer'
done
require_fixed_string_count crates/lila-engine/src/lib.rs 'b"lila-program-wasm-cache-key-v18"' 1 'current component/data/service cache framing domain'
require_fixed_string_count crates/lila-engine/src/lib.rs 'fn hash_optional_program_cache_locales(' 1 'presence/count/length-framed component locale owner'
require_fixed_string_count crates/lila-engine/src/lib.rs 'hash_optional_program_cache_locales(' 9 'one cache field owner consumed by all eight independent filters'
require_fixed_string_count crates/lila-engine/src/lib.rs '.map(CustomListProfile::requested_locales)' 1 'cache consumes the actual List filter'
for intl_data_dimension in currency_codes date_time_calendars numbering_systems named_time_zones service_selection; do
  require_fixed_string_count crates/lila-engine/src/lib.rs "match selection.${intl_data_dimension}()" 1 'cache independently frames each checked data/service dimension'
done
require_fixed_string_count crates/lila-engine/src/lib.rs 'hash.update(services.wire().to_le_bytes());' 1 'cache binds the actual public service set'
require_exact_line_count crates/lila-cli/src/main.rs 'enum IntlLocaleProjectionComponent {' 1 'private closed CLI component input authority'
require_fixed_string_count crates/lila-cli/src/main.rs 'fn parse(argument: &str) -> Option<Self>' 1 'sole open-string CLI component boundary'
require_fixed_string_count crates/lila-cli/src/main.rs 'IntlLocaleProjectionComponent::parse(&args[index])' 1 'actual parser consumes the checked component token'
for cli_projection_route in 'List:list' 'RelativeTime:relative_time' 'DisplayNames:display_names' 'Duration:duration' 'Number:number' 'DateTime:date_time' 'Collator:collator' 'Segmenter:segmenter'; do
  cli_projection_component="${cli_projection_route%%:*}"
  cli_projection_field="${cli_projection_route#*:}"
  require_fixed_string_count crates/lila-cli/src/main.rs "IntlLocaleProjectionComponent::${cli_projection_component} => &mut ${cli_projection_field}_locales," 1 'exhaustive checked component chooses its own real field'
done
for cli_projection_flag in '--intl-list-locales:List' '--intl-relative-time-locales:RelativeTime' '--intl-displaynames-locales:DisplayNames' '--intl-duration-locales:Duration' '--intl-number-locales:Number' '--intl-datetime-locales:DateTime' '--intl-collator-locales:Collator' '--intl-segmenter-locales:Segmenter'; do
  cli_projection_spelling="${cli_projection_flag%%:*}"
  cli_projection_token="${cli_projection_flag#*:}"
  require_fixed_string_count crates/lila-cli/src/main.rs "\"${cli_projection_spelling}\" => Some(Self::${cli_projection_token})," 1 'actual locale flag spelling mints its closed token'
done
for cli_projection_argument in lists relative_times display_names durations numbers date_times collators segmenters; do
  require_fixed_string_present crates/lila-cli/src/main.rs "${cli_projection_argument}.as_deref()" 'CLI transports its checked locale field to the combined constructor'
done
require_fixed_string_present crates/lila-cli/src/main.rs 'CustomIntlProfile::from_manifest_json(json)' 'CLI consumes the same strict dimension/service manifest owner'
intl_service_owner="crates/lila-intl/src/service_selection.rs"
intl_bundle_export="crates/lila-intl/src/selection/export.rs"
intl_bundle_manifest="crates/lila-intl/src/selection/manifest.rs"
intl_selected_provider="crates/lila-intl/src/provider.rs"
for intl_service_path in "$intl_service_owner" "$intl_bundle_export" "$intl_bundle_manifest"; do
  require_file "$intl_service_path"
  check_no_inline_legacy_includes "$intl_service_path"
done
require_exact_line_count crates/lila-intl/src/lib.rs 'mod service_selection;' 1 'private closed service/dependency owner attachment'
require_exact_line_count "$intl_selection" 'mod export;' 1 'private actual sparse bundle framing owner'
require_exact_line_count "$intl_selection" 'mod manifest;' 1 'private checked dimension/service manifest owner'
require_exact_line_count "$intl_service_owner" 'pub struct IntlDataComponentSet(u16);' 1 'private closed component-set bits'
require_exact_line_count "$intl_service_owner" 'pub struct CheckedIntlServiceSelection {' 1 'single checked service-to-frame authority'
intl_service_fields="$(sed -n '/^pub struct CheckedIntlServiceSelection {$/,/^}$/p' "$intl_service_owner")"
require_text_regex_count "$intl_service_fields" '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+' 0 'no raw caller-branded service closure'
require_fixed_string_count "$intl_service_owner" 'let required: &[C] = match service {' 1 'every closed service derives its actual dependency frames'
require_fixed_string_count "$intl_service_owner" 'if bits == 0 || bits & !allowed != 0' 1 'wire admission refuses empty or unknown services'
require_fixed_string_count "$intl_selection" 'if checked.contains_component(IntlDataComponent::$kind)' 1 'only retained component factories execute'
require_fixed_string_count "$intl_selection" 'EmbeddedIntlProvider::with_selected_data_images(' 1 'bundle publishes through the checked native foundation gate'
require_fixed_string_count "$intl_selection" 'pub fn component_images(&self) -> Vec<(IntlDataComponent, Arc<[u8]>)>' 1 'typed physical frames are shared by export, emission and footprint accounting'
require_fixed_string_count "$intl_selection" '.map(|(component, bytes)| (component.section_name(), bytes))' 1 'section emission projects only the actual physical frame inventory'
require_fixed_string_count "$intl_selected_provider" 'pub fn with_selected_data_images(' 1 'sole sparse native publication gate'
require_fixed_string_count "$intl_selected_provider" 'services.contains_component(*component) != digest.is_some()' 1 'admission requires the exact retained foundation set'
require_fixed_string_count "$intl_selected_provider" 'if services.requested() == IntlServiceSet::ALL' 1 'complete selection retains original full-image admission'
require_fixed_string_count "$intl_selected_provider" 'let crate::IntlDataProfile::Custom(id) = locale.profile() else {' 1 'partial service/frame sets require an actual Custom foundation'
require_fixed_string_count "$intl_selected_provider" 'if !self.services.permits(service)' 1 'retained dependencies do not authorize absent public services'
require_fixed_string_present "$intl_selected_provider" 'data.ok_or(crate::UnavailableIntlService(service))' 'missing service data fails explicitly without embedded fallback'
for intl_public_service in Locale DateTimeFormat NumberFormat PluralRules ListFormat Collator DisplayNames DurationFormat Segmenter; do
  require_fixed_string_present "$intl_selected_provider" "self.service_data(IntlService::${intl_public_service}," 'actual public operation consumes its requested service authority'
done
intl_relative_public_operation="$(braced_rust_item_source "$intl_selected_provider" '^impl IntlOperationProvider<ResolveRelativeTimeLocale>')"
require_text_regex_count "$intl_relative_public_operation" 'self\.service_data\(' 1 'public RelativeTime resolution consumes service admission'
require_text_regex_count "$intl_relative_public_operation" 'IntlService::RelativeTimeFormat,' 2 'public RelativeTime admission and its retained Number foundation use the same service token'
require_fixed_string_count "$intl_bundle_export" 'if count != checked.components().len()' 1 'export-v2 reader requires exactly the derived frame count'
require_fixed_string_present "$intl_bundle_export" 'CheckedIntlServiceSelection::from_wire(wire)' 'sparse export re-admits the closed service owner'
require_fixed_string_count "$intl_bundle_export" '.eq(checked.components().iter())' 1 'export re-admission requires canonical exact component identities'
require_fixed_string_count "$intl_bundle_export" 'Self::from_component_sections(selection, frames)?' 1 'wire admission uses the actual native owner decoder'
require_fixed_string_present "$intl_bundle_manifest" 'if version.schema_version == 6' 'strict service manifest owns its current version'
require_fixed_string_present "$intl_selection" 'fn validate_service_dimensions(' 'dimensions cannot decorate omitted frames'
for intl_dimension_setter in with_currency_codes with_date_time_calendars with_numbering_systems with_named_time_zones with_services; do
  require_fixed_string_present "$intl_selection" "pub fn ${intl_dimension_setter}(" 'SDK consumes checked data/service selection'
  require_fixed_string_present "$intl_bundle_manifest" "$intl_dimension_setter" 'manifest consumes the same typed dimension owner'
done
for intl_dimensional_factory in \
  'NumberProfilesDataImage::for_custom_data_projection(' \
  'NumberProfilesDataImage::for_custom_numbering_projection(' \
  'DisplayNamesDataImage::for_custom_data_projection(' \
  'DateTimeDataImage::for_custom_data_projection(' \
  'DateTimeDataImage::for_custom_numbering_projection(' \
  'NamedTimeZoneDataImage::for_custom_projection('
do
  require_fixed_string_present "$intl_selection" "$intl_dimensional_factory" 'checked selector reaches a real physical native producer'
done
for intl_paired_domain in \
  'numbers.currency_codes() != display.currency_codes()' \
  'numbers.numbering_system_selection() != date_time.numbering_system_selection()' \
  'date_time.named_time_zone_selection() != named.named_time_zone_selection()' \
  'names.named_time_zone_selection() != named.named_time_zone_selection()'
do
  require_fixed_string_present "$intl_selected_provider" "$intl_paired_domain" 'native admission binds shared selected domains to their retained owners'
done
for intl_rederived_projection in number_image datetime_image; do
  intl_rederived_path="crates/lila-intl/src/${intl_rederived_projection}/projection.rs"
  require_file "$intl_rederived_path"
  require_fixed_string_count "$intl_rederived_path" 'if payload != expected.as_slice()' 1 'projected wire bytes require complete pinned source rederivation'
done
require_tree_regex_count crates/lila-engine/src 'CustomListProjection' 0 'no obsolete List-only product profile'
require_tree_regex_count crates/lila-cli/src 'CustomListProjection' 0 'no obsolete List-only CLI profile'
require_fixed_string_count scripts/generate-intl-display-relative-identity.py "rtf |= production_sources(repository, 'crates/lila-intl/src/relative_time_image')" 1 'RelativeTime recipe binds its actual projection child'
require_fixed_string_count scripts/generate-intl-display-relative-identity.py "dn |= production_sources(repository, 'crates/lila-intl/src/display_names_image')" 1 'DisplayNames recipe binds its actual consumed projection child'
for projection_recipe in scripts/generate-intl-list-identity.py scripts/generate-intl-display-relative-identity.py; do
  require_fixed_string_count "$projection_recipe" 'crates/lila-intl/src/selection.rs' 1 'actual combined SDK selection in each component source recipe'
done
for combined_projection_control in \
  crates/lila-engine/tests/aot_intl_compilation_profile.rs \
  crates/lila-cli/tests/cli/intl.rs \
  crates/lila-engine/tests/fixtures/intl_compilation_profile/projected_list.js \
  crates/lila-engine/tests/fixtures/intl_compilation_profile/projected_list_relative_time.js \
  crates/lila-engine/tests/fixtures/intl_compilation_profile/projected_display_names.js
do
  require_file "$combined_projection_control"
done
require_fixed_string_count crates/lila-engine/src/lib.rs 'fn component_projection_cache_keys_bind_each_filter_presence_and_order_independently()' 1 'real artifact cache tuple control'
require_fixed_string_count crates/lila-cli/src/main.rs 'fn relative_and_combined_projection_arguments_are_checked_before_source_loading()' 1 'actual CLI admission and ordering control'
require_fixed_string_count crates/lila-engine/tests/aot_intl_compilation_profile.rs 'fn independent_relative_and_combined_projections_emit_exact_rows_and_remint_selected_wire_owners()' 1 'actual emitted-frame and wire-owner control'
require_fixed_string_count crates/lila-cli/tests/cli/intl.rs 'fn combined_projection_cli_build_matches_sdk_and_runs_both_selected_services()' 1 'actual CLI/SDK artifact equality and operation control'
require_fixed_string_count crates/lila-engine/src/lib.rs 'fn display_names_cache_filters_are_independent_of_other_components_and_input_order()' 1 'actual independent third cache-field control'
require_fixed_string_count crates/lila-cli/src/main.rs 'fn display_names_projection_arguments_are_checked_before_source_loading()' 1 'actual three-component CLI admission ordering control'
require_fixed_string_count crates/lila-engine/tests/aot_intl_compilation_profile.rs 'fn display_names_and_three_component_projections_emit_selected_tables_and_run_real_consumers()' 1 'actual physical frames/foreign/wire/script/module consumer control'
require_fixed_string_count crates/lila-cli/tests/cli/intl.rs 'fn display_names_cli_projection_matches_sdk_and_composes_all_three_filters()' 1 'actual CLI/SDK three-component producer parity control'


# Exact exporter data has one narrow generated expansion boundary. The
# handwritten Intl kernel keeps the workspace ban on unsafe operations.
require_exact_line_count Cargo.toml 'unsafe_code = "forbid"' 1 'handwritten workspace unsafe ban'
require_exact_line_count crates/lila-intl/Cargo.toml 'workspace = true' 1 'Intl workspace lint inheritance'
collator_data=crates/lila-intl-collator-data
require_exact_line_count "$collator_data/Cargo.toml" 'unsafe_code = "deny"' 1 'data crate default unsafe denial'
require_exact_line_count "$collator_data/src/lib.rs" '#![deny(unsafe_code)]' 1 'data root unsafe denial'
require_exact_line_count "$collator_data/src/lib.rs" '#[allow(unsafe_code)]' 1 'private generated expansion allowance'
require_exact_line_count "$collator_data/src/lib.rs" 'mod baked;' 1 'private baked provider owner'
require_exact_line_count "$collator_data/src/lib.rs" 'pub use baked::PinnedCollatorProvider;' 1 'sole authored provider export'
require_exact_line_count "$collator_data/src/baked.rs" 'pub struct PinnedCollatorProvider;' 1 'immutable pinned provider'
require_fixed_string_count "$collator_data/src/baked.rs" 'make_provider!(PinnedCollatorProvider);' 1 'exact exporter provider marker'
require_regex_count "$collator_data/src/baked.rs" '^impl_(collation|normalizer)_[a-z_0-9]+!' 9 'exact consumed data marker expansions'
require_tree_regex_count "$collator_data/src" '(^|[^[:alnum:]_])unsafe[[:space:]]+(fn|impl|trait|extern|\{)' 0 'handwritten unsafe operations'

# Source ownership checks; runtime conformance is a separate batch gate.
wasm_collator=crates/lila-aot-wasm/src/builtins/intl_collator.rs
wasm_collator_compare=crates/lila-aot-wasm/src/builtins/intl_collator/compare.rs
require_file "$wasm_collator"
for collator_child in construction options compare resolved pool; do
  collator_child_path="crates/lila-aot-wasm/src/builtins/intl_collator/${collator_child}.rs"
  require_file "$collator_child_path"
  require_exact_line_count "$wasm_collator" "mod ${collator_child};" 1 'private Collator child attachment'
  check_no_inline_legacy_includes "$collator_child_path"
done
require_exact_line_count "$wasm_collator" "enum CollatorProviderRequest<'a> {" 1 'private closed Collator provider request domain'
require_exact_line_count "$wasm_collator" 'struct CollatorProviderResponse {' 1 'private Collator GC response owner'
require_exact_line_count "$wasm_collator_compare" \
  'pub(super) struct CompletedCollatorCompareStringsLocals {' 1 'completed ordered Collator strings owner'
collator_completed_fields="$(sed -n '/^pub(super) struct CompletedCollatorCompareStringsLocals {$/,/^}$/p' "$wasm_collator_compare")"
if grep -Eq '^[[:space:]]+pub(\([^)]*\))?[[:space:]]+' <<<"$collator_completed_fields"; then
  fail "$wasm_collator_compare must keep completed comparison string fields private"
fi
require_fixed_string_count "$wasm_collator_compare" \
  'BuiltinClosurePayload::IntlCollator(&record)' 1 'rooted Collator record in the cached bound closure'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/intl_collator/resolved.rs 'self.emit_array_from_argument_list(&list, function)?;' 1 'Collator completed list Array publisher'
require_tree_regex_count crates/lila-aot-wasm/src '^[[:space:]]*pub\(in crate::builtins\)[[:space:]]+fn[[:space:]]+emit_intl_is_unicode_type_i32[[:space:]]*\(' 1 'shared real Unicode-type validator owner'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/intl_number.rs 'mod unicode_type;' 1 'shared Unicode-type child'
for collator_method in emit_intl_collator_constructor emit_intl_collator_supported_locales_of emit_intl_collator_resolved_options emit_intl_collator_compare_getter emit_intl_collator_bound_compare; do
  require_tree_regex_count crates/lila-aot-wasm/src "^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+${collator_method}[[:space:]]*\(" 1 'actual Collator emitter owner'
  require_fixed_string_count "$wasm_standard_builtins" "self.${collator_method}(function)?;" 1 'fixed standard Collator entry'
done
# localeCompare is a String operation: its whole receiver/arguments enter the
# same Collator owner through string.rs, rather than a sixth Standard delegate.
require_regex_count "$wasm_collator_compare" \
  '^[[:space:]]*pub\(crate\)[[:space:]]+fn[[:space:]]+emit_intrinsic_string_locale_compare[[:space:]]*\(' 1 'String localeCompare Collator owner'
require_fixed_string_count crates/lila-aot-wasm/src/builtins/string.rs \
  'b.emit_intrinsic_string_locale_compare(' 1 'whole-value String localeCompare consumer'
require_fixed_string_count "$wasm_standard_builtins" \
  'self.emit_string_prototype_locale_compare_builtin(function)?' 1 'fixed standard String localeCompare entry'
require_file crates/lila-engine/tests/aot_intl_collator.rs
require_file crates/lila-engine/tests/fixtures/intl_collator/ordered_argument_conversion.js
require_file crates/lila-engine/tests/fixtures/intl_collator/brand_and_cache.js
require_file crates/lila-engine/tests/fixtures/intl_collator/called_function_realms.js

# The DurationFormat family has private production owners; its real GC record
# is declared by the central schema rather than a passive linear layout.
wasm_duration_format=crates/lila-aot-wasm/src/builtins/intl_durationformat.rs
require_file "$wasm_duration_format"
require_exact_line_count crates/lila-aot-wasm/src/builtins/mod.rs 'mod intl_durationformat;' 1 'private DurationFormat module'
if grep -Eq '^(pub(\([^)]*\))?[[:space:]]+)mod[[:space:]]+intl_durationformat;' crates/lila-aot-wasm/src/builtins/mod.rs; then
  fail 'DurationFormat family must remain a private module'
fi
for duration_format_child in construction inputs options pool render resolved temporal; do
  require_exact_line_count "$wasm_duration_format" "mod $duration_format_child;" 1 "private DurationFormat $duration_format_child child"
  require_file "crates/lila-aot-wasm/src/builtins/intl_durationformat/$duration_format_child.rs"
  check_no_inline_legacy_includes "crates/lila-aot-wasm/src/builtins/intl_durationformat/$duration_format_child.rs"
done
check_no_inline_legacy_includes "$wasm_duration_format"

# Locale information lists have a closed selector and one framed GC reader.
wasm_locale_information_list=crates/lila-aot-wasm/src/builtins/intl/locale_information_list.rs
for locale_list_child in calendars collations time_zones locale_information_list; do
  locale_list_path="crates/lila-aot-wasm/src/builtins/intl/${locale_list_child}.rs"
  require_file "$locale_list_path"
  require_exact_line_count crates/lila-aot-wasm/src/builtins/intl.rs "mod ${locale_list_child};" 1 'private Locale list child attachment'
  check_no_inline_legacy_includes "$locale_list_path"
  check_raw_line_budget "$locale_list_path" 400
done
require_exact_line_count "$wasm_locale_information_list" \
  'pub(super) enum LocaleInformationKind {' 1 'closed family-owned Locale list selector'
require_regex_count "$wasm_locale_information_list" \
  '^[[:space:]]*pub\(super\)[[:space:]]+fn[[:space:]]+emit_intl_locale_information_array[[:space:]]*\(' 1 'checked GC Locale list reader and Array publisher'
require_regex_count "$wasm_locale_information_list" \
  '^[[:space:]]*fn[[:space:]]+emit_validate_locale_information_name[[:space:]]*\(' 1 'private completed Locale list name validation'
require_fixed_string_count "$wasm_locale_information_list" \
  'IntlByteArrayReader::new(&response, schema, function)' 1 'GC ByteArray Locale response reader'
require_fixed_string_count "$wasm_locale_information_list" \
  'lila_intl::LOCALE_INFORMATION_WIRE_VERSION as i64' 1 'Locale response protocol version admission'
require_fixed_string_count "$wasm_locale_information_list" \
  'kind.operation().code()' 1 'closed Locale response operation admission'
require_fixed_string_count "$wasm_locale_information_list" \
  'self.emit_array_from_argument_list(&values, function)?' 1 'completed GC Locale List Array publication'
for locale_list_entry in calendars:Calendars collations:Collations time_zones:TimeZones; do
  locale_list_child="${locale_list_entry%%:*}"
  locale_list_kind="${locale_list_entry#*:}"
  locale_list_path="crates/lila-aot-wasm/src/builtins/intl/${locale_list_child}.rs"
  require_fixed_string_count "$locale_list_path" \
    'self.emit_intl_locale_information_array(' 1 'thin Locale information consumer'
  require_fixed_string_count "$locale_list_path" \
    "LocaleInformationKind::${locale_list_kind}," 1 'closed Locale information kind selection'
done
require_file crates/lila-intl/src/locale_information_wire.rs
require_module_decl crates/lila-intl/src/lib.rs locale_information_wire
require_file crates/lila-engine/tests/aot_intl_locale_information_lists.rs

# Chinese/Dangi retained data and emitted year models have private owners.
require_module_decl crates/lila-aot-wasm/src/data.rs temporal_east_asian_years
require_file crates/lila-aot-wasm/src/data/temporal_east_asian_years.rs
check_no_inline_legacy_includes crates/lila-aot-wasm/src/data/temporal_east_asian_years.rs
require_module_decl crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic.rs east_asian
require_file crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic/east_asian.rs
check_no_inline_legacy_includes crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic/east_asian.rs
for east_asian_child in proleptic month_day; do
  require_module_decl crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic/east_asian.rs "$east_asian_child"
  require_file "crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic/east_asian/${east_asian_child}.rs"
  check_no_inline_legacy_includes "crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic/east_asian/${east_asian_child}.rs"
done

# Calendar arithmetic and differences have consumed private owner modules.
require_module_decl crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic.rs hebrew
require_file crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic/hebrew.rs
check_no_inline_legacy_includes crates/lila-aot-wasm/src/builtins/temporal_calendar_arithmetic/hebrew.rs
for calendar_difference_parent in temporal_plain_date_methods temporal_plain_year_month_methods; do
  calendar_difference_source="crates/lila-aot-wasm/src/builtins/${calendar_difference_parent}.rs"
  calendar_difference_leaf="crates/lila-aot-wasm/src/builtins/${calendar_difference_parent}/difference.rs"
  require_module_decl "$calendar_difference_source" difference
  require_file "$calendar_difference_leaf"
  check_no_inline_legacy_includes "$calendar_difference_leaf"
done

# String hooks consume one private observed-method owner.
require_module_decl crates/lila-aot-wasm/src/builtins/string.rs symbol_method
require_file crates/lila-aot-wasm/src/builtins/string/symbol_method.rs
check_no_inline_legacy_includes crates/lila-aot-wasm/src/builtins/string/symbol_method.rs

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-module-boundaries: ok\n'
