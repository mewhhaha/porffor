//! Optional legacy reflection is owned by actual source-body activations.
//!
//! No JavaScript binding, function name, property containing the callable or
//! depth estimate selects an activation. The ordinary entry owns its actual
//! FunctionObject, and Wasm locals save each nested activation independently.

use super::*;
use crate::gc_types::{
    ArgumentsObject, ExecutableCode, ExecutableCodeSchema, FunctionObject, FunctionObjectSchema,
    FunctionProtocolCode, GcI32Constant, GcLocal, GcOperand, NonNullable, Nullable, OrdinaryObject,
    PropertyDescriptor, PropertyDescriptorSchema, RuntimeSchema, ScalarValue,
    SourceActivationDepth, StoredValue, ValueLocals,
};

struct ReflectionProperty {
    descriptor: GcLocal<PropertyDescriptor>,
    previous_value: GcLocal<StoredValue>,
}

impl ReflectionProperty {
    fn publish(&self, value: &ValueLocals, schema: &RuntimeSchema, function: &mut Function) {
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(value, function),
            function,
        );
        schema
            .struct_type::<PropertyDescriptor>()
            .field(PropertyDescriptorSchema::VALUE)
            .write(
                &self.descriptor,
                GcOperand::reference(&stored, schema),
                schema,
                function,
            );
        stored.clear(function);
    }

    fn restore(&self, schema: &RuntimeSchema, function: &mut Function) {
        schema
            .struct_type::<PropertyDescriptor>()
            .field(PropertyDescriptorSchema::VALUE)
            .write(
                &self.descriptor,
                GcOperand::reference(&self.previous_value, schema),
                schema,
                function,
            );
    }

    fn clear(self, function: &mut Function) {
        self.previous_value.clear(function);
        self.descriptor.clear(function);
    }
}

struct LegacyReflection {
    caller: ReflectionProperty,
    arguments: ReflectionProperty,
}

/// One source-body entry and its matching exit. Strict and non-ordinary source
/// bodies still install their identity: a strict caller is an opaque barrier,
/// rather than a frame which a sloppy callee may skip to reveal an older one.
/// Native entries have a separate invocation-bound barrier; forwarding
/// intrinsics and prepared Scripts are transparent.
#[must_use = "source activations must be restored on every body exit"]
pub(crate) struct SourceActivation {
    depth: SourceActivationDepth,
    previous_function: GcLocal<FunctionObject, Nullable>,
    reflection: Option<LegacyReflection>,
}

impl SourceActivation {
    pub(crate) fn clear(self, function: &mut Function) {
        if let Some(reflection) = self.reflection {
            reflection.arguments.clear(function);
            reflection.caller.clear(function);
        }
        self.previous_function.clear(function);
        self.depth.clear(function);
    }
}

impl FunctionBuilder<'_> {
    fn capture_legacy_reflection_property(
        &mut self,
        object: &GcLocal<OrdinaryObject>,
        name: &str,
        function: &mut Function,
    ) -> Result<ReflectionProperty, EmitError> {
        let schema = self.runtime_schema();
        let key = self.emit_function_string_key(name, function)?;
        let own = self.emit_ordinary_own_descriptor_reference(object, &key, function)?;
        // Publication fixes both immutable own descriptors before the callable
        // escapes. Their absence is a violated producer invariant, not a reason
        // to consult Function.prototype's restricted accessor.
        let descriptor = schema
            .reserve_gc_local::<PropertyDescriptor, NonNullable>(function)
            .initialize(
                own.load(schema, function).require_non_null(function),
                function,
            );
        let previous_value = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PropertyDescriptor>()
                .field(PropertyDescriptorSchema::VALUE)
                .read(&descriptor, schema, function)
                .reference(),
            function,
        );
        own.clear(function);
        key.clear(function);
        Ok(ReflectionProperty {
            descriptor,
            previous_value,
        })
    }

    pub(crate) fn emit_enter_source_activation(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .planned_entry()
            .is_some_and(|entry| entry.has_source_activation())
        {
            return Ok(());
        }
        assert!(
            self.source_activation.is_none(),
            "one owner per source body"
        );
        let schema = self.runtime_schema();
        // Admission happens before parameter initialization or reflection
        // publication. The rejected body never owns or retires its caller's
        // depth or legacy properties, and its Throw remains catchable in JS.
        let depth = schema.enter_source_activation_depth(function, |function| {
            let overflow = schema.reserve_completion(function);
            self.emit_throw_runtime_error(
                NativeErrorKind::RangeError,
                RuntimeErrorMessage::MAXIMUM_SOURCE_CALL_STACK_SIZE_EXCEEDED,
                &overflow,
                function,
            )?;
            Ok::<_, EmitError>(overflow)
        })?;
        let callable = schema.reserve_gc_local(function).initialize(
            self.body_entry_locals()
                .expect("source activation owns body-entry roles")
                .function_object()
                .expect("source activation owns an actual callable")
                .load(schema, function),
            function,
        );
        let previous_function = schema
            .reserve_gc_local::<FunctionObject, Nullable>(function)
            .initialize(schema.load_active_source_function(function), function);
        let reflection = if self.current_function_meta().is_some_and(|meta| {
            !meta.strict && meta.protocol() == FunctionProtocolIr::OrdinaryCallAndConstruct
        }) {
            let object = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<FunctionObject>()
                    .field(FunctionObjectSchema::OBJECT)
                    .read(&callable, schema, function)
                    .reference(),
                function,
            );
            let caller = self.capture_legacy_reflection_property(&object, "caller", function)?;
            let arguments =
                self.capture_legacy_reflection_property(&object, "arguments", function)?;
            object.clear(function);
            let value = schema.reserve_value_local(function);
            value.set_scalar(ScalarValue::Null, function);
            previous_function.load(schema, function).is_null(function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            let previous = schema.reserve_gc_local(function).initialize(
                previous_function
                    .load(schema, function)
                    .require_non_null(function),
                function,
            );
            let strict = schema.reserve_i32_local(function);
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::STRICT)
                .read(&previous, schema, function)
                .store(strict, function);
            let code = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<FunctionObject>()
                    .field(FunctionObjectSchema::CODE)
                    .read(&previous, schema, function)
                    .reference(),
                function,
            );
            let protocol = schema.reserve_i32_local(function);
            let kind = schema.reserve_i32_local(function);
            schema
                .struct_type::<ExecutableCode>()
                .field(ExecutableCodeSchema::PROTOCOL)
                .read(&code, schema, function)
                .store(protocol, function);
            schema
                .struct_type::<ExecutableCode>()
                .field(ExecutableCodeSchema::KIND)
                .read(&code, schema, function)
                .store(kind, function);
            strict.load(function);
            function.instruction(&Instruction::I32Eqz);
            protocol.load(function);
            function.instruction(&Instruction::I32Const(
                FunctionProtocolCode::OrdinaryCallAndConstruct.encoding(),
            ));
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::I32And);
            kind.load(function);
            function.instruction(&Instruction::I32Const(
                crate::gc_types::ExecutableCodeKind::JavaScript.encode(),
            ));
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::I32And);
            schema.release_i32_local(kind, function);
            schema.release_i32_local(protocol, function);
            code.clear(function);
            function.instruction(&Instruction::If(BlockType::Empty));
            value.set_reference(&previous, schema, function);
            function.instruction(&Instruction::End);
            schema.release_i32_local(strict, function);
            previous.clear(function);
            function.instruction(&Instruction::End);
            caller.publish(&value, schema, function);
            value.clear(function);
            Some(LegacyReflection { caller, arguments })
        } else {
            None
        };
        schema.replace_active_source_function(&callable, function);
        callable.clear(function);
        self.source_activation = Some(SourceActivation {
            depth,
            previous_function,
            reflection,
        });
        Ok(())
    }

    pub(crate) fn emit_publish_source_activation_arguments(
        &self,
        arguments: &GcLocal<ArgumentsObject>,
        function: &mut Function,
    ) {
        let Some(reflection) = self
            .source_activation
            .as_ref()
            .and_then(|owner| owner.reflection.as_ref())
        else {
            return;
        };
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        value.set_reference(arguments, schema, function);
        reflection.arguments.publish(&value, schema, function);
        value.clear(function);
    }

    /// This emits restoration, without releasing the compile-time owner: a
    /// return in one branch must not retire locals needed by another branch.
    pub(crate) fn emit_leave_source_activation(&self, function: &mut Function) {
        let Some(owner) = &self.source_activation else {
            return;
        };
        let schema = self.runtime_schema();
        if let Some(reflection) = &owner.reflection {
            reflection.arguments.restore(schema, function);
            reflection.caller.restore(schema, function);
        }
        schema.replace_active_source_function(&owner.previous_function, function);
        owner.depth.restore(function);
    }
}
