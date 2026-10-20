"""Keep source ownership controls effective when validation passes are fused."""
import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location(
    'native_templates', ROOT / 'scripts/check-native-program-templates.py')
controls = importlib.util.module_from_spec(spec)
spec.loader.exec_module(controls)


class NativeTemplateSourceControls(unittest.TestCase):
    def test_primitive_proof_wasm_and_per_call_operands_cannot_be_bypassed(self):
        proof = 'primitive_eval_requires_exact_scalar_IR_and_inert_preparation'
        execution = 'primitive_eval_executes_real_stateless_wasm_with_current_operands_and_budget'
        mutations = [
            ('crates/scriptw-ir/src/runtime_direct_eval/primitive_literal.rs',
             'self.inert_preparation()?', 'None?', proof),
            ('crates/scriptw-ir/src/runtime_direct_eval/primitive_literal.rs',
             'expression.value_info() != ValueInfo::new(kind)', 'false', proof),
            (controls.A + 'module/compiled_module_package.rs',
             'Instruction::LocalGet(1)', 'Instruction::I64Const(0)', execution),
            (controls.E + 'wasm_native_direct_eval_compiler/primitive.rs',
             'match proof.value()', 'match RuntimeDirectEvalPrimitive::Null', execution),
            (controls.E + 'wasm_native_direct_eval_compiler/primitive.rs',
             'factory.call(&mut *store, &arguments, &mut output)?;', '', execution),
        ]
        for path, before, after, expected in mutations:
            with self.subTest(path=path, before=before):
                sources = dict(self.sources)
                self.assertEqual(sources[path].count(before), 1)
                sources[path] = sources[path].replace(before, after)
                self.assertIn(expected, controls.check(sources))

    def test_bulk_reads_keep_the_actual_payload_span_and_fresh_typed_arrays(self):
        mutations = [
            ('/bindings/allocation/bulk.rs', 'image.copy_i8_range_to_slice', 'image.copy_to_i8_slice',
             'fresh_source_arrays_use_only_the_received_P_payload_image_bounds'),
            ('/bindings/allocation/bulk.rs', 'width.bytes()', '1',
             'fresh_source_arrays_use_only_the_received_P_payload_image_bounds'),
            ('/bindings/allocation/bulk.rs', 'ArrayRef::new_from_i8_slice', 'reuse_array',
             'fresh_allocations_exact_bits_and_bounds'),
        ]
        for suffix, before, after, expected in mutations:
            with self.subTest(suffix=suffix, before=before):
                path = controls.N + suffix
                sources = dict(self.sources)
                self.assertEqual(sources[path].count(before), 1)
                sources[path] = sources[path].replace(before, after)
                self.assertIn(expected, controls.check(sources))

    def test_regexp_literal_proof_allocation_and_real_wasm_cannot_be_removed(self):
        mutations = [
            (controls.A + 'objects/regexp_literal/native.rs',
             'Instruction::RefEq', 'Instruction::Nop',
             'regexp_eval_checks_actual_realm_and_copies_raw_units_before_string_publication'),
            ('crates/scriptw-ir/src/runtime_direct_eval/regexp_literal.rs',
             'ValidatedRegExpProgram::from_program(program).ok()?', 'None?',
             'regexp_eval_requires_single_lowered_literal_and_validated_program'),
            (controls.A + 'objects/regexp_literal.rs',
             'writable: true', 'writable: false',
             'regexp_literal_installs_exact_lastindex_on_only_its_fresh_header'),
            (controls.E + 'wasm_native_direct_eval_compiler/regexp_literal.rs',
             'factory.call(&mut *store, &operands, &mut output)?;', '',
             'regexp_eval_executes_exact_R_wasm_with_fresh_raw_operands_and_original_budget'),
        ]
        for path, before, after, expected in mutations:
            with self.subTest(path=path):
                sources = dict(self.sources)
                self.assertEqual(sources[path].count(before), 1)
                sources[path] = sources[path].replace(before, after)
                self.assertIn(expected, controls.check(sources))

    @classmethod
    def setUpClass(cls):
        cls.sources = {name: (ROOT / name).read_text() for name in controls.FILES}

    def test_current_fused_pass_preserves_the_ownership_controls(self):
        self.assertEqual(controls.check(self.sources), [])

    def test_missing_field_local_or_plan_wiring_cannot_pass(self):
        owner = 'payload_owner_has_actual_typed_initialized_local_and_bootstrap_witness'
        wiring = 'ownership_facts_share_the_actual_planning_pass'
        mutations = [
            ('/contexts/facts.rs', '!field.mutable', 'field.mutable', owner),
            ('/contexts.rs', 'self.signatures.local(index, body, *local, self.schema)?;', '', owner),
            ('/plan.rs', 'context_facts.record_type(result.types, ty);', '', wiring),
            ('/plan.rs', 'contexts.body(', 'unvalidated_body(', wiring),
        ]
        for suffix, before, after, expected in mutations:
            with self.subTest(suffix=suffix, before=before):
                path = controls.T + suffix
                sources = dict(self.sources)
                self.assertEqual(sources[path].count(before), 1)
                sources[path] = sources[path].replace(before, after)
                self.assertIn(expected, controls.check(sources))

    def test_empty_eval_cannot_bypass_syntax_instantiation_or_real_wasm(self):
        proof = 'empty_eval_requires_actual_syntax_and_lowered_no_effects_proof'
        execution = 'empty_eval_executes_typed_stateless_wasm_with_original_authority_and_deadline'
        mutations = [
            ('crates/scriptw-ir/src/runtime_direct_eval.rs',
             'block.statement_list().statements().is_empty()', 'true', proof),
            ('crates/scriptw-ir/src/runtime_direct_eval.rs',
             'StatementListItem::Declaration(_) => false',
             'StatementListItem::Declaration(_) => true', proof),
            ('crates/scriptw-ir/src/runtime_direct_eval/empty/metadata.rs', 'binding.slot == 0', 'true',
             'empty_eval_metadata_has_only_dead_intrinsics_and_exact_hidden_caller_capture'),
            ('crates/scriptw-ir/src/runtime_direct_eval/empty.rs', '!self.empty_syntax', 'false', proof),
            ('crates/scriptw-ir/src/runtime_direct_eval/empty.rs',
             'unit.declarations != RuntimeGlobalDeclarationPlan::default()', 'false', proof),
            (controls.E + 'wasm_native_direct_eval_compiler/empty.rs',
             'factory.call(&mut *store, &[], &mut output)?;', '', execution),
        ]
        for path, before, after, expected in mutations:
            with self.subTest(path=path, before=before):
                sources = dict(self.sources)
                self.assertEqual(sources[path].count(before), 1)
                sources[path] = sources[path].replace(before, after)
                self.assertIn(expected, controls.check(sources))


if __name__ == '__main__':
    unittest.main()
