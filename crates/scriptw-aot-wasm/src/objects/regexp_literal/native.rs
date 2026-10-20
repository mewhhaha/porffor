//! Typed Wasm kernel for the sealed, fully lowered single-literal eval proof.
use super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::gc_types::{
    CodeUnitArray, Environment, EnvironmentSchema, RealmRecord, RegExpProgram, StringConstruction,
};

impl FunctionBuilder<'_> {
    pub(crate) fn compile_native_regexp_literal_factory(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::NativeRegExpLiteralFactory);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::NativeRegExpLiteralFactoryParameters>(
                &mut function,
            );
        let schema = self.runtime_schema();
        let global = self.emit_global_environment_from(&parameters.environment, &mut function);
        let realm = schema
            .reserve_gc_local::<RealmRecord, NonNullable>(&mut function)
            .initialize(
                schema
                    .struct_type::<Environment>()
                    .field(EnvironmentSchema::DEFINING_REALM)
                    .read(&global, schema, &mut function)
                    .reference()
                    .require_non_null(&mut function),
                &mut function,
            );
        let prototype = schema.reserve_value_local(&mut function);
        // A host caller must supply the same execution Realm as its actual
        // global chain. A mismatch performs no JS work and retains ordinary P
        // preparation instead of guessing which authority should win.
        let _ = realm.load(schema, &mut function);
        let _ = parameters.realm.load(schema, &mut function);
        function.instruction(&Instruction::RefEq);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        prototype.set_scalar(ScalarValue::Undefined, &mut function);
        self.completion().set_normal(&prototype, &mut function);
        self.completion().emit(&mut function);
        function.instruction(&Instruction::I32Const(
            crate::prepared_script::PreparedScriptStatus::Unmatched.code(),
        ));
        function.instruction(&Instruction::Return);
        function.instruction(&Instruction::End);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::RegExpPrototype,
            &prototype,
            &mut function,
        );
        let source = schema.reserve_gc_local(&mut function).initialize(
            self.emit_string_from_raw_units(&parameters.source_units, &mut function),
            &mut function,
        );
        let flags = schema.reserve_gc_local(&mut function).initialize(
            self.emit_string_from_raw_units(&parameters.flags_units, &mut function),
            &mut function,
        );
        let program = schema.reserve_gc_local(&mut function).initialize(
            schema.struct_type::<RegExpProgram>().construct(
                (GcOperand::reference(&parameters.compiled, schema),),
                &mut function,
            ),
            &mut function,
        );
        let object = schema.reserve_gc_local(&mut function).initialize(
            schema.helper_reference_on_stack(schema.call_helper(
                crate::runtime_helpers::RegExpLiteralAllocateArguments::new(
                    &prototype, &source, &flags, &program, &realm,
                ),
                self.runtime_helper_base()?,
                &mut function,
            )),
            &mut function,
        );
        let value = schema.reserve_value_local(&mut function);
        value.set_reference(&object, schema, &mut function);
        self.completion().set_normal(&value, &mut function);
        value.clear(&mut function);
        object.clear(&mut function);
        program.clear(&mut function);
        flags.clear(&mut function);
        source.clear(&mut function);
        prototype.clear(&mut function);
        realm.clear(&mut function);
        global.clear(&mut function);
        parameters.release(&mut function);
        self.completion().emit(&mut function);
        function.instruction(&Instruction::I32Const(2)); // Entered.
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Raw arrays can alias a host input. Publish only our own fresh copy.
    fn emit_string_from_raw_units(
        &self,
        units: &GcLocal<CodeUnitArray>,
        function: &mut Function,
    ) -> GcStackReference<StringValue> {
        let schema = self.runtime_schema();
        let length = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(units, schema, function);
        length.store(function);
        let output = StringConstruction::allocate(
            schema,
            schema.reserve_gc_local(function),
            length,
            function,
        );
        let zero = schema.reserve_i32_local(function);
        zero.set_constant(0, function);
        output.copy_units_from(zero, units, length, schema, function);
        schema.release_i32_local(zero, function);
        schema.release_i32_local(length, function);
        output.publish(schema, function)
    }
}
