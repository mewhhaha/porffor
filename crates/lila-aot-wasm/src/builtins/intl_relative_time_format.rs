//! RelativeTimeFormat owns JavaScript observations; the pinned provider owns
//! locale selection and relative-time patterns.

use super::super::*;
use crate::functions::{NewTargetPrototypeFallback, OrdinaryDefaultPrototype};
use crate::objects::TaggedLocals;
use lila_intl::{
    IntlHostCallOutcome, IntlHostOp, RELATIVE_TIME_WIRE_HEADER_BYTES, RELATIVE_TIME_WIRE_VERSION,
    RelativeTimeNumberKind, RelativeTimePartKind, RelativeTimeUnit,
};

mod construction_lifecycle;
mod initialization;
mod pool;
mod provider_wire;
pub(crate) use pool::intl_relative_time_format_pool_strings;
mod rendering;
mod resolved;

const RTF_RECEIVER_ERROR: &str = "Intl.RelativeTimeFormat method called on incompatible receiver";
const RTF_INVALID_OPTION: &str = "Invalid RelativeTimeFormat option";
const RTF_NON_FINITE: &str = "Value is not a finite number";
const RTF_INVALID_UNIT: &str = "Invalid unit argument";

impl FunctionBuilder<'_> {
    fn emit_rtf_set_const(&self, destination: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::LocalSet(destination));
    }

    fn emit_rtf_set_string(&mut self, destination: u32, text: &str, function: &mut Function) {
        function.instruction(&Instruction::I64Const(self.strings.payload(text)));
        function.instruction(&Instruction::LocalSet(destination));
    }

    fn emit_rtf_copy(&self, source: u32, destination: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(source));
        function.instruction(&Instruction::LocalSet(destination));
    }

    fn emit_rtf_if_eq(&self, local: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
    }

    fn emit_rtf_range_error(
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

    fn emit_rtf_type_error(
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

    fn emit_rtf_get_option(
        &mut self,
        options: TaggedLocals,
        property: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        self.emit_rtf_set_string(key, property, function);
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

    fn emit_rtf_to_string(
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

    fn emit_rtf_string_option(
        &mut self,
        options: TaggedLocals,
        property: &str,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_rtf_get_option(options, property, value, function)?;
        self.emit_rtf_if_eq(value.tag, ValueKind::Undefined.tag() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_rtf_to_string(value, value.payload, function)?;
        self.emit_rtf_set_const(value.tag, ValueKind::String.tag() as i64, function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_rtf_select_string(
        &mut self,
        text: u32,
        allowed: &[(&str, i64)],
        destination: u32,
        function: &mut Function,
    ) {
        let expected = self.reserve_temp_local();
        self.emit_rtf_set_const(destination, 0, function);
        for &(spelling, code) in allowed {
            self.emit_rtf_set_string(expected, spelling, function);
            self.emit_string_payload_equality_i32(text, expected, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_rtf_set_const(destination, code, function);
            function.instruction(&Instruction::End);
        }
        self.release_temp_local(expected);
    }

    fn emit_rtf_choice_option(
        &mut self,
        options: TaggedLocals,
        property: &str,
        allowed: &[(&str, i64)],
        default: i64,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        self.emit_rtf_string_option(options, property, value, function)?;
        self.emit_rtf_set_const(destination, default, function);
        self.emit_rtf_if_eq(value.tag, ValueKind::Undefined.tag() as i64, function);
        function.instruction(&Instruction::Else);
        self.emit_rtf_select_string(value.payload, allowed, destination, function);
        function.instruction(&Instruction::LocalGet(destination));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_rtf_range_error(&format!("Invalid {property} option"), function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(value.tag);
        self.release_temp_local(value.payload);
        Ok(())
    }

    fn emit_rtf_options_object(
        &mut self,
        options: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_rtf_if_eq(options.tag, ValueKind::Undefined.tag() as i64, function);
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(options.payload));
        self.emit_rtf_set_const(options.tag, ValueKind::Object.tag() as i64, function);
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

    fn emit_rtf_canonical_locales(
        &mut self,
        locales: TaggedLocals,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let destination_tag = self.reserve_temp_local();
        let result = (|| {
            let meta = self
                .functions
                .get(&StandardBuiltinId::IntlGetCanonicalLocales.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported("missing Intl.getCanonicalLocales dependency")
                })?;
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
        })();
        self.release_temp_local(destination_tag);
        result
    }

    fn emit_rtf_record_from_receiver(
        &mut self,
        receiver: TaggedLocals,
        record: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload = self
            .this_payload_local
            .ok_or_else(|| EmitError::unsupported("RelativeTimeFormat method lacks receiver"))?;
        let this_tag = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("RelativeTimeFormat method lacks receiver tag")
        })?;
        let brand = self.reserve_temp_local();
        self.emit_rtf_copy(this_payload, receiver.payload, function);
        self.emit_rtf_copy(this_tag, receiver.tag, function);
        self.emit_rtf_set_const(record, 0, function);
        self.emit_rtf_if_eq(receiver.tag, ValueKind::Object.tag() as i64, function);
        self.load_i64_to_local_from_offset(
            receiver.payload,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand,
            function,
        );
        self.emit_rtf_if_eq(
            brand,
            OBJECT_INTERNAL_BRAND_INTL_RELATIVE_TIME_FORMAT as i64,
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
        self.emit_rtf_type_error(RTF_RECEIVER_ERROR, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(brand);
        Ok(())
    }

    fn emit_rtf_load_string(
        &self,
        record: u32,
        offset: u64,
        destination: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(record, offset, destination, function);
    }

    fn emit_rtf_load_word(
        &self,
        record: u32,
        offset: u64,
        destination: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(record, offset, destination, function);
    }
}
