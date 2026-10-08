//! Shared Intl numeric observations, before the closed native provider seam.
use super::super::*;
use crate::emit::AccessorThrowRouting;
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::gc_types::*;
use crate::operations::PropertyKeyLocals;
use lila_intl::number_format::options::*;
use lila_intl::{NumberNumericKind, NumberPrecisionKind};
mod digits;
mod numeric_input;
mod options;
mod unicode_type;
pub(in crate::builtins) use numeric_input::IntlMathematicalValueLocals;

const INTL_INCREMENT_PRECISION: RuntimeErrorMessage =
    RuntimeErrorMessage::ROUNDING_INCREMENT_REQUIRES_FRACTION_PRECISION;
const INTL_INCREMENT_RANGE: RuntimeErrorMessage =
    RuntimeErrorMessage::ROUNDING_INCREMENT_REQUIRES_EQUAL_FRACTION_DIGITS;
const INTL_DIGIT_RANGE: RuntimeErrorMessage =
    RuntimeErrorMessage::MAXIMUM_DIGITS_IS_LESS_THAN_MINIMUM_DIGITS;

pub(in crate::builtins) enum IntlDigitDefaults<'a> {
    PluralRules,
    NumberFormat {
        style: &'a GcI32DomainLocal<StyleOption>,
        currency: &'a GcLocal<StringValue>,
    },
}

pub(in crate::builtins) fn set_i32(out: I32Local, value: i32, function: &mut Function) {
    function.instruction(&Instruction::I32Const(value));
    out.store(function);
}
pub(in crate::builtins) fn emit_tag_is(
    value: &ValueLocals,
    tag: WasmRuntimeValueTag,
    function: &mut Function,
) {
    value.tag().load(function);
    function.instruction(&Instruction::I32Const(tag as i32));
    function.instruction(&Instruction::I32Eq);
}
pub(in crate::builtins) fn emit_domain_is<V: GcI32Constant>(
    value: &GcI32DomainLocal<V>,
    expected: V,
    function: &mut Function,
) {
    value.load(function);
    function.instruction(&Instruction::I32Const(expected.encode()));
    function.instruction(&Instruction::I32Eq);
}

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_intl_number_adopt_completion(
        &mut self,
        pending: &CompletionLocals,
        function: &mut Function,
    ) {
        self.completion().copy_from(pending, function);
        self.emit_propagate_current_throw_if_needed(function);
    }
    pub(in crate::builtins) fn emit_intl_number_range_error(
        &mut self,
        message: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result = self.runtime_schema().reserve_completion(function);
        result.initialize(function);
        self.emit_throw_current_function_realm_range_error(message, &result, function)?;
        self.completion().copy_from(&result, function);
        result.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_intl_number_type_error(
        &mut self,
        message: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result = self.runtime_schema().reserve_completion(function);
        result.initialize(function);
        self.emit_throw_current_function_realm_type_error(message, &result, function)?;
        self.completion().copy_from(&result, function);
        result.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_intl_number_result_object(
        &mut self,
        function: &mut Function,
    ) -> Result<GcLocal<OrdinaryObject>, EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ObjectPrototype,
            &prototype,
            function,
        );
        let object = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), function)?,
            function,
        );
        prototype.clear(function);
        realm.clear(function);
        Ok(object)
    }
    pub(in crate::builtins) fn emit_intl_number_append_result_property(
        &mut self,
        object: &GcLocal<OrdinaryObject>,
        name: &str,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let text = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(name, function)?,
            function,
        );
        let key = PropertyKeyLocals::from_string(schema, &text, function);
        self.emit_object_append_data_property_with_flags(
            object, &key, value, true, true, true, function,
        )?;
        key.clear(function);
        text.clear(function);
        Ok(())
    }
}
