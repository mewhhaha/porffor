//! NumberFormat owns JavaScript observations; the pinned provider owns parts.

use super::super::*;
use crate::functions::NewTargetPrototypeFallback;
use crate::objects::TaggedLocals;
use lila_intl::number_format::options::*;
use lila_intl::number_format::{NumberPartKind, RangePartSource};
use lila_intl::{NumberConfigurationWord as NfWord, NumberNumericKind, NumberPrecisionKind};

mod construction_lifecycle;
mod initialization;
mod numeric_input;
mod options;
mod pool;
pub(crate) use pool::intl_number_format_pool_strings;
mod digits;
mod primitive_locale;
mod provider_wire;
mod render;
mod resolved;
mod validation;

const NF_RECEIVER_ERROR: &str = "Intl.NumberFormat method requires a NumberFormat receiver";
const NF_RANGE_UNDEFINED: &str = "Intl.NumberFormat range endpoints must be defined";
const NF_RANGE_NAN: &str = "Intl.NumberFormat range endpoints must not be NaN";
const NF_CURRENCY_REQUIRED: &str = "Currency style requires a currency option";
const NF_UNIT_REQUIRED: &str = "Unit style requires a unit option";
const NF_INCREMENT_PRECISION: &str = "Rounding increment requires fraction precision";
const NF_INCREMENT_RANGE: &str = "Rounding increment requires equal fraction digits";
const NF_DIGIT_RANGE: &str = "Maximum digits is less than minimum digits";

#[derive(Clone, Copy)]
pub(crate) enum NfFormatMode {
    String,
    Parts,
}

struct NfOptionsLocals {
    words: [u32; lila_intl::NUMBER_CONFIGURATION_WORDS],
    style_text: u32,
}
impl NfOptionsLocals {
    fn reserve(builder: &mut FunctionBuilder<'_>) -> Self {
        Self {
            words: core::array::from_fn(|_| builder.reserve_temp_local()),
            style_text: builder.reserve_temp_local(),
        }
    }
    fn word(&self, word: NfWord) -> u32 {
        self.words[word.index()]
    }
    fn release(self, builder: &mut FunctionBuilder<'_>) {
        builder.release_temp_local(self.style_text);
        for local in self.words.into_iter().rev() {
            builder.release_temp_local(local);
        }
    }
}

impl FunctionBuilder<'_> {
    fn emit_nf_set_const(&self, destination: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::LocalSet(destination));
    }
    fn emit_nf_set_string(&mut self, destination: u32, text: &str, function: &mut Function) {
        function.instruction(&Instruction::I64Const(self.strings.payload(text)));
        function.instruction(&Instruction::LocalSet(destination));
    }
    fn emit_nf_copy(&self, source: u32, destination: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(source));
        function.instruction(&Instruction::LocalSet(destination));
    }
    fn emit_nf_if_eq(&self, local: u32, code: u64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(code as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
    }
    fn emit_nf_if_nonzero(&self, local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
    }
    fn emit_nf_range_error(
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
    fn emit_nf_type_error(
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
    fn emit_nf_record_from_receiver(
        &mut self,
        record: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload = self
            .this_payload_local
            .ok_or_else(|| EmitError::unsupported("NumberFormat method lacks receiver"))?;
        let tag = self
            .this_tag_local
            .ok_or_else(|| EmitError::unsupported("NumberFormat method lacks receiver tag"))?;
        let brand = self.reserve_temp_local();
        self.emit_nf_set_const(record, 0, function);
        self.emit_nf_if_eq(tag, ValueKind::Object.tag() as u64, function);
        self.load_i64_to_local_from_offset(
            payload,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand,
            function,
        );
        self.emit_nf_if_eq(brand, OBJECT_INTERNAL_BRAND_INTL_NUMBER_FORMAT, function);
        self.load_i64_to_local_from_offset(
            payload,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(record));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_nf_type_error(NF_RECEIVER_ERROR, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(brand);
        Ok(())
    }
    fn emit_nf_load_word(
        &self,
        record: u32,
        word: NfWord,
        destination: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            record,
            HEAP_INTL_NF_WORDS_OFFSET + word.offset(),
            destination,
            function,
        );
    }
    fn emit_nf_intrinsic_call(
        &mut self,
        builtin: StandardBuiltinId,
        receiver: Option<(u32, Option<u32>)>,
        arguments: &[(u32, u32)],
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "missing NumberFormat intrinsic dependency {builtin:?}"
                ))
            })?;
        self.emit_direct_js_call(&meta, receiver, arguments, payload, tag, function)?;
        self.emit_return_current_completion_if_throw(function);
        Ok(())
    }
}
