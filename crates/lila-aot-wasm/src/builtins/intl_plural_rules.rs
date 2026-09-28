//! Intl.PluralRules observations and receiver checks for the AOT path.

use super::super::*;
use crate::functions::{NewTargetPrototypeFallback, OrdinaryDefaultPrototype};
use crate::objects::TaggedLocals;
use lila_intl::{IntlHostCallOutcome, IntlHostOp, NumberConfigurationWord as NfWord};

mod initialization;
mod pool;
mod provider_wire;
mod resolved;
mod selection;

pub(crate) use pool::intl_plural_rules_pool_strings;

const PR_RECEIVER_ERROR: &str = "Intl.PluralRules method requires a PluralRules receiver";
const PR_INVALID_OPTION: &str = "Invalid PluralRules option";
const PR_MUST_USE_NEW: &str = "Intl.PluralRules must be called with new";
const PR_RANGE_UNDEFINED: &str = "PluralRules.selectRange arguments must not be undefined";

impl FunctionBuilder<'_> {
    fn emit_pr_set_const(&self, destination: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::LocalSet(destination));
    }

    fn emit_pr_set_string(&mut self, destination: u32, text: &str, function: &mut Function) {
        function.instruction(&Instruction::I64Const(self.strings.payload(text)));
        function.instruction(&Instruction::LocalSet(destination));
    }

    fn emit_pr_if_eq(&self, local: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
    }

    fn emit_pr_range_error(
        &mut self,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_range_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_pr_type_error(
        &mut self,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_pr_get_option(
        &mut self,
        options: TaggedLocals,
        property: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        self.emit_pr_set_string(key, property, function);
        self.emit_object_read(
            options.payload,
            options.tag,
            options.payload,
            options.tag,
            key,
            value.payload,
            value.tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.release_temp_local(key);
        Ok(())
    }

    fn emit_pr_to_string(
        &mut self,
        value: TaggedLocals,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let primitive = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::String,
            value.payload,
            value.tag,
            function,
        )?;
        self.emit_current_function_realm_primitive_to_string_local(primitive, destination, function)
    }

    fn emit_pr_choice_option(
        &mut self,
        options: TaggedLocals,
        property: &str,
        allowed: &[(&str, i64)],
        default: i64,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let text = self.reserve_temp_local();
        let expected = self.reserve_temp_local();
        self.emit_pr_get_option(options, property, value, function)?;
        self.emit_pr_set_const(destination, default, function);
        self.emit_pr_if_eq(value.tag, ValueKind::Undefined.tag() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_pr_to_string(value, text, function)?;
        self.emit_pr_set_const(destination, 0, function);
        for &(spelling, code) in allowed {
            self.emit_pr_set_string(expected, spelling, function);
            self.emit_string_payload_equality_i32(text, expected, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_pr_set_const(destination, code, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(destination));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pr_range_error(&format!("Invalid {property} option"), function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [expected, text, value.tag, value.payload] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_pr_options_object(
        &mut self,
        options: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_pr_if_eq(options.tag, ValueKind::Undefined.tag() as i64, function);
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(options.payload));
        self.emit_pr_set_const(options.tag, ValueKind::Object.tag() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_current_function_realm_object_locals(
            options.payload,
            options.tag,
            options.payload,
            options.tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_pr_canonical_locales(
        &mut self,
        locales: TaggedLocals,
        destination: u32,
        destination_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::IntlGetCanonicalLocales.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Intl.getCanonicalLocales dependency"))?;
        self.emit_direct_js_call(
            &meta,
            None,
            &[(locales.payload, locales.tag)],
            destination,
            destination_tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        Ok(())
    }

    fn emit_pr_require_record(
        &mut self,
        receiver: TaggedLocals,
        record: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload = self
            .this_payload_local
            .ok_or_else(|| EmitError::unsupported("PluralRules method lacks receiver"))?;
        let this_tag = self
            .this_tag_local
            .ok_or_else(|| EmitError::unsupported("PluralRules method lacks receiver tag"))?;
        let brand = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(this_payload));
        function.instruction(&Instruction::LocalSet(receiver.payload));
        function.instruction(&Instruction::LocalGet(this_tag));
        function.instruction(&Instruction::LocalSet(receiver.tag));
        self.emit_pr_set_const(record, 0, function);
        self.emit_pr_if_eq(receiver.tag, ValueKind::Object.tag() as i64, function);
        self.load_i64_to_local_from_offset(
            receiver.payload,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand,
            function,
        );
        self.emit_pr_if_eq(
            brand,
            OBJECT_INTERNAL_BRAND_INTL_PLURAL_RULES as i64,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver.payload,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(record));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pr_type_error(PR_RECEIVER_ERROR, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(brand);
        Ok(())
    }
}
