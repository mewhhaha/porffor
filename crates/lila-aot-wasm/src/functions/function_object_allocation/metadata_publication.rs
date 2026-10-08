//! Publishes metadata on the original completed GC function record.

use super::*;
use crate::gc_types::{
    FunctionObjectSchema, I32Local, OrdinaryObject, OrdinaryObjectSchema, ScalarValue,
};
use crate::runtime_helpers::HelperParameters;

impl FunctionBuilder<'_> {
    // Code identity, immutable context fields and ACTIVE_FUNCTION are already
    // published by the allocation owner. This call does not select a Realm.
    pub(super) fn emit_publish_function_metadata(
        &mut self,
        meta: &WasmFunctionMeta,
        realm: &GcLocal<RealmRecord>,
        callable: &GcLocal<FunctionObject>,
        materialization: FunctionPrototypeMaterialization,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let automatic = match materialization {
            FunctionPrototypeMaterialization::Automatic => true,
            FunctionPrototypeMaterialization::BootstrapSupplied => false,
        };
        let create_prototype = automatic
            && meta.host_builtin != Some(HostBuiltinId::HTMLDDA)
            && (meta.protocol().is_constructable()
                || matches!(
                    meta.protocol().execution_kind(),
                    FunctionExecutionKind::Generator | FunctionExecutionKind::AsyncGenerator
                ));
        let constructor_back_reference = match meta.protocol().execution_kind() {
            FunctionExecutionKind::Ordinary | FunctionExecutionKind::Async => true,
            FunctionExecutionKind::Generator | FunctionExecutionKind::AsyncGenerator => false,
        };
        let name = schema
            .reserve_gc_local::<StringValue, NonNullable>(function)
            .initialize(
                self.emit_interned_string_reference(meta.runtime_name(), function)?,
                function,
            );
        let parent = schema.reserve_value_local(function);
        parent.set_undefined(function);
        if create_prototype {
            let slot = match meta.protocol().execution_kind() {
                FunctionExecutionKind::Generator => NonArrayRealmIntrinsicSlot::GeneratorPrototype,
                FunctionExecutionKind::AsyncGenerator => {
                    NonArrayRealmIntrinsicSlot::AsyncGeneratorPrototype
                }
                FunctionExecutionKind::Ordinary | FunctionExecutionKind::Async => {
                    NonArrayRealmIntrinsicSlot::ObjectPrototype
                }
            };
            self.emit_load_non_array_realm_intrinsic(realm, slot, &parent, function);
        }
        let length_bits = schema.reserve_i64_local(function);
        function.instruction(&Instruction::F64Const(Ieee64::from(meta.length as f64)));
        function.instruction(&Instruction::I64ReinterpretF64);
        length_bits.store(function);
        let configurable = schema.reserve_i32_local(function);
        let caller = schema.reserve_i32_local(function);
        let prototype = schema.reserve_i32_local(function);
        let constructor = schema.reserve_i32_local(function);
        configurable.set_constant(i32::from(meta.length_name_configurable), function);
        caller.set_constant(
            i32::from(
                !meta.strict && meta.protocol() == FunctionProtocolIr::OrdinaryCallAndConstruct,
            ),
            function,
        );
        prototype.set_constant(i32::from(create_prototype), function);
        constructor.set_constant(i32::from(constructor_back_reference), function);
        schema.call_helper(
            crate::runtime_helpers::FunctionMetadataPublishArguments::new(
                callable,
                length_bits,
                &name,
                configurable,
                caller,
                prototype,
                constructor,
                &parent,
            ),
            self.runtime_helper_base()?,
            function,
        );
        schema.release_i32_local(constructor, function);
        schema.release_i32_local(prototype, function);
        schema.release_i32_local(caller, function);
        schema.release_i32_local(configurable, function);
        schema.release_i64_local(length_bits, function);
        parent.clear(function);
        name.clear(function);
        Ok(())
    }

    pub(crate) fn compile_function_metadata_publish_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self
            .begin_helper_body(crate::runtime_helpers::RuntimeHelperId::FunctionMetadataPublish);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::FunctionMetadataPublishParameters>(
                &mut function,
            );
        self.emit_function_metadata_publish_inner(&parameters, &mut function)?;
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_function_metadata_publish_inner(
        &mut self,
        parameters: &crate::runtime_helpers::FunctionMetadataPublishParameters,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let object = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::OBJECT)
                .read(&parameters.callable, schema, function)
                .reference(),
            function,
        );
        let value = schema.reserve_value_local(function);
        let zero = schema.reserve_i32_local(function);
        zero.set_constant(0, function);
        value.set_number(parameters.length_bits, function);
        self.emit_define_function_runtime_metadata_property(
            &object,
            "length",
            &value,
            zero,
            parameters.configurable,
            function,
        )?;
        value.set_reference(&parameters.name, schema, function);
        self.emit_define_function_runtime_metadata_property(
            &object,
            "name",
            &value,
            zero,
            parameters.configurable,
            function,
        )?;

        parameters.has_caller.load(function);
        self.open_frame(ControlFrameKind::If, function);
        value.set_scalar(ScalarValue::Null, function);
        self.emit_define_function_runtime_metadata_property(
            &object, "caller", &value, zero, zero, function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        parameters.create_prototype.load(function);
        self.open_frame(ControlFrameKind::If, function);
        let prototype = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(
                Some(&parameters.prototype_parent),
                function,
            )?,
            function,
        );
        let public_prototype = schema.reserve_value_local(function);
        public_prototype.set_reference(&prototype, schema, function);
        let prototype_key = self.emit_function_string_key("prototype", function)?;
        self.emit_object_append_data_property_with_flags(
            &object,
            &prototype_key,
            &public_prototype,
            true,
            false,
            false,
            function,
        )?;
        prototype_key.clear(function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&public_prototype, function),
            function,
        );
        schema
            .struct_type::<FunctionObject>()
            .field(FunctionObjectSchema::PUBLIC_PROTOTYPE_CACHE)
            .write(
                &parameters.callable,
                GcOperand::reference(&stored, schema),
                schema,
                function,
            );
        stored.clear(function);
        parameters.constructor_back_reference.load(function);
        self.open_frame(ControlFrameKind::If, function);
        value.set_reference(&parameters.callable, schema, function);
        let constructor_key = self.emit_function_string_key("constructor", function)?;
        self.emit_object_append_data_property_with_flags(
            &prototype,
            &constructor_key,
            &value,
            true,
            false,
            true,
            function,
        )?;
        constructor_key.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        public_prototype.clear(function);
        prototype.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // The Realm's thrower becomes non-extensible only after its own
        // unconfigurable metadata is complete, before bootstrap publishes it.
        parameters.configurable.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .struct_type::<OrdinaryObject>()
            .field(OrdinaryObjectSchema::EXTENSIBLE)
            .write(&object, GcOperand::boolean(false), schema, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(zero, function);
        value.clear(function);
        object.clear(function);
        Ok(())
    }

    fn emit_define_function_runtime_metadata_property(
        &mut self,
        object: &GcLocal<OrdinaryObject>,
        name: &str,
        value: &ValueLocals,
        zero: I32Local,
        configurable: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.emit_function_string_key(name, function)?;
        self.emit_object_append_data_property_with_runtime_flags(
            object,
            &key,
            value,
            zero,
            zero,
            configurable,
            function,
        )?;
        key.clear(function);
        Ok(())
    }
}
