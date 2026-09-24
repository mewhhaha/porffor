//! 10.5.5 Proxy `[[GetOwnProperty]]`, as `Object.getOwnPropertyDescriptor`
//! and every engine caller of it observe it.
//!
//! The trap's result object is never handed back. It is converted with 6.2.6.5
//! ToPropertyDescriptor (HasProperty then Get for each field, so accessors on
//! it run in that order), completed with 6.2.6.6, checked against the target's
//! own descriptor, and published by the 6.2.6.4 owner as a fresh object.

use super::*;
use crate::objects::{CompletedPropertyDescriptorLocals, DescriptorObjectPrototype};
use lila_ir::property_descriptor::DescriptorField;

/// The Proxy step of the enclosing `[[GetOwnProperty]]` target loop.
pub(super) struct ProxyGetOwnPropertyRequest {
    /// In: the Proxy. Out, when the handler has no trap (step 5): its
    /// `[[ProxyTarget]]`, for the loop to resolve in turn.
    pub(super) object: TaggedLocals,
    /// `P` as a JavaScript String or Symbol, the form the trap and the
    /// target's `[[GetOwnProperty]]` receive.
    pub(super) key: TaggedLocals,
    /// Set to 1 when `result` holds the answer; left 0 when forwarding.
    pub(super) handled: u32,
    /// FromPropertyDescriptor(resultDesc), or `undefined`.
    pub(super) result: TaggedLocals,
}

/// 10.5.5 step 8's `targetDesc`, read back from the object the recursive
/// `[[GetOwnProperty]]` produced. That object comes from the 6.2.6.4 owner and
/// never reaches user code, so reading its own data fields observes nothing.
struct TargetDescriptorLocals {
    found: u32,
    accessor: u32,
    value: TaggedLocals,
    writable: TaggedLocals,
    get: TaggedLocals,
    set: TaggedLocals,
    enumerable: TaggedLocals,
    configurable: TaggedLocals,
}

impl TargetDescriptorLocals {
    /// Every local of the record, in reservation order.
    fn locals(&self) -> [u32; 14] {
        [
            self.found,
            self.accessor,
            self.value.payload,
            self.value.tag,
            self.writable.payload,
            self.writable.tag,
            self.get.payload,
            self.get.tag,
            self.set.payload,
            self.set.tag,
            self.enumerable.payload,
            self.enumerable.tag,
            self.configurable.payload,
            self.configurable.tag,
        ]
    }

    fn emit_nonzero_i32(local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
    }

    fn emit_found_i32(&self, function: &mut Function) {
        Self::emit_nonzero_i32(self.found, function);
    }

    fn emit_accessor_i32(&self, function: &mut Function) {
        Self::emit_nonzero_i32(self.accessor, function);
    }

    fn emit_writable_i32(&self, function: &mut Function) {
        Self::emit_nonzero_i32(self.writable.payload, function);
    }

    fn emit_enumerable_i32(&self, function: &mut Function) {
        Self::emit_nonzero_i32(self.enumerable.payload, function);
    }

    fn emit_configurable_i32(&self, function: &mut Function) {
        Self::emit_nonzero_i32(self.configurable.payload, function);
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_proxy_get_own_property_descriptor(
        &mut self,
        request: ProxyGetOwnPropertyRequest,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ProxyGetOwnPropertyRequest {
            object,
            key,
            handled,
            result,
        } = request;
        let target = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let handler = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let trap = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let trap_key = self.reserve_temp_local();

        // Steps 1-3.
        self.emit_load_live_proxy_slots(
            object.payload,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(target.payload, target.tag),
                ProxyHandlerLocals::new(handler.payload, handler.tag),
            ),
            ProxyRevocationRoute::CurrentFunctionRealm,
            function,
        )?;
        // Step 4: GetMethod(handler, "getOwnPropertyDescriptor").
        function.instruction(&Instruction::I64Const(
            self.strings.payload("getOwnPropertyDescriptor"),
        ));
        function.instruction(&Instruction::LocalSet(trap_key));
        self.emit_object_read_without_throw_propagation(
            handler.payload,
            handler.tag,
            handler.payload,
            handler.tag,
            trap_key,
            trap.payload,
            trap.tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(trap.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Step 5: return ? target.[[GetOwnProperty]](P).
        function.instruction(&Instruction::LocalGet(target.payload));
        function.instruction(&Instruction::LocalSet(object.payload));
        function.instruction(&Instruction::LocalGet(target.tag));
        function.instruction(&Instruction::LocalSet(object.tag));
        function.instruction(&Instruction::Else);
        self.emit_is_callable_i32(trap.tag, trap.payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_proxy_get_own_property_trap_result(target, handler, trap, key, result, function)?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled));
        function.instruction(&Instruction::End);

        self.release_temp_local(trap_key);
        for local in [
            trap.tag,
            trap.payload,
            handler.tag,
            handler.payload,
            target.tag,
            target.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Steps 6-16, then the caller's FromPropertyDescriptor.
    fn emit_proxy_get_own_property_trap_result(
        &mut self,
        target: TaggedLocals,
        handler: TaggedLocals,
        trap: TaggedLocals,
        key: TaggedLocals,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let trap_result = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let extensible = self.reserve_temp_local();

        // Step 6.
        self.emit_function_or_proxy_call_leave_throw_completion(
            trap.payload,
            trap.tag,
            handler.payload,
            handler.tag,
            &[(target.payload, target.tag), (key.payload, key.tag)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(trap_result.payload));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(trap_result.tag));

        // Step 7: an Object of any representation, or undefined.
        self.emit_is_heap_object_like_tag_i32(trap_result.tag, function);
        function.instruction(&Instruction::LocalGet(trap_result.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Proxy getOwnPropertyDescriptor trap result must be object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Step 8.
        let target_descriptor = self.emit_proxy_target_own_descriptor(target, key, function)?;

        function.instruction(&Instruction::LocalGet(trap_result.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Step 9. 9.a returns undefined without consulting IsExtensible.
        target_descriptor.emit_found_i32(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        target_descriptor.emit_configurable_i32(function);
        function.instruction(&Instruction::I32Eqz);
        self.emit_proxy_get_own_property_type_error_if(
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-configurable target property",
            function,
        )?;
        self.emit_object_is_extensible_i32(target.payload, target.tag, extensible, function)?;
        function.instruction(&Instruction::LocalGet(extensible));
        function.instruction(&Instruction::I64Eqz);
        self.emit_proxy_get_own_property_type_error_if(
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-extensible target",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result.payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(result.tag));
        function.instruction(&Instruction::Else);

        // Step 10 precedes the observable conversion of step 11.
        self.emit_object_is_extensible_i32(target.payload, target.tag, extensible, function)?;
        let converted = self.emit_to_property_descriptor(
            trap_result,
            "Proxy getOwnPropertyDescriptor trap result must be object or undefined",
            function,
        )?;
        // Step 12.
        let completed = self.emit_complete_property_descriptor(converted, function);
        // Steps 13-14.
        self.emit_proxy_get_own_property_compatibility(
            extensible,
            &completed,
            &target_descriptor,
            function,
        )?;
        // Step 15.
        completed.emit_configurable_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        target_descriptor.emit_found_i32(function);
        function.instruction(&Instruction::I32Eqz);
        target_descriptor.emit_configurable_i32(function);
        function.instruction(&Instruction::I32Or);
        self.emit_proxy_get_own_property_type_error_if(
            "Proxy getOwnPropertyDescriptor trap result cannot report non-configurable target property",
            function,
        )?;
        // 15.b: resultDesc has [[Writable]] exactly when it completed to data.
        completed.emit_accessor_i32(function);
        function.instruction(&Instruction::I32Eqz);
        completed.emit_writable_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        target_descriptor.emit_writable_i32(function);
        function.instruction(&Instruction::I32And);
        self.emit_proxy_get_own_property_type_error_if(
            "Proxy getOwnPropertyDescriptor trap result cannot report non-writable target property",
            function,
        )?;
        function.instruction(&Instruction::End);

        // Step 16, and Object.getOwnPropertyDescriptor's FromPropertyDescriptor
        // in the executing builtin's Realm.
        let function_realm = self.reserve_temp_local();
        let prototype = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(function_realm));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            function_realm,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_realm_intrinsic_prototype_or_global(
            function_realm,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            OBJECT_PROTOTYPE_GLOBAL_INDEX,
            prototype,
            function,
        );
        self.emit_from_property_descriptor(
            DescriptorObjectPrototype::ObjectPrototypeLocal(prototype),
            &completed.object_fields(),
            result.payload,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(result.tag));
        self.release_temp_local(prototype);
        self.release_temp_local(function_realm);
        self.release_completed_property_descriptor(completed);
        function.instruction(&Instruction::End);

        for local in target_descriptor.locals().into_iter().rev() {
            self.release_temp_local(local);
        }
        self.release_temp_local(extensible);
        self.release_temp_local(trap_result.tag);
        self.release_temp_local(trap_result.payload);
        Ok(())
    }

    /// Step 8: `? target.[[GetOwnProperty]](P)`, through this builtin, so a
    /// Proxy target runs its own trap and an exotic target its own method.
    fn emit_proxy_target_own_descriptor(
        &mut self,
        target: TaggedLocals,
        key: TaggedLocals,
        function: &mut Function,
    ) -> Result<TargetDescriptorLocals, EmitError> {
        let get_own_property_descriptor = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .expect("Object.getOwnPropertyDescriptor is compiling its own Proxy branch");
        // The record outlives this call, so it is reserved first; the scratch
        // locals below it are released before returning.
        let found = self.reserve_temp_local();
        let accessor = self.reserve_temp_local();
        let mut tagged = || TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let target_descriptor = TargetDescriptorLocals {
            found,
            accessor,
            value: tagged(),
            writable: tagged(),
            get: tagged(),
            set: tagged(),
            enumerable: tagged(),
            configurable: tagged(),
        };
        let descriptor_object = tagged();
        let field_key = self.reserve_temp_local();
        let ignored_presence = self.reserve_temp_local();

        self.emit_direct_js_call(
            &get_own_property_descriptor,
            None,
            &[(target.payload, target.tag), (key.payload, key.tag)],
            descriptor_object.payload,
            descriptor_object.tag,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(descriptor_object.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(target_descriptor.found));
        // The object is complete: it has `get` exactly when it is an accessor
        // descriptor. An absent field reads as payload 0.
        for field in DescriptorField::ALL {
            let (present, value) = match field {
                DescriptorField::Value => (ignored_presence, target_descriptor.value),
                DescriptorField::Writable => (ignored_presence, target_descriptor.writable),
                DescriptorField::Get => (target_descriptor.accessor, target_descriptor.get),
                DescriptorField::Set => (ignored_presence, target_descriptor.set),
                DescriptorField::Enumerable => (ignored_presence, target_descriptor.enumerable),
                DescriptorField::Configurable => (ignored_presence, target_descriptor.configurable),
            };
            function.instruction(&Instruction::I64Const(self.strings.payload(field.key())));
            function.instruction(&Instruction::LocalSet(field_key));
            self.emit_object_own_data_field_read(
                descriptor_object.payload,
                descriptor_object.tag,
                field_key,
                present,
                value.payload,
                value.tag,
                function,
            );
        }

        self.release_temp_local(ignored_presence);
        self.release_temp_local(field_key);
        self.release_temp_local(descriptor_object.tag);
        self.release_temp_local(descriptor_object.payload);
        Ok(target_descriptor)
    }

    /// Steps 13-14: IsCompatiblePropertyDescriptor(extensibleTarget,
    /// resultDesc, targetDesc), i.e. 10.1.6.3 ValidateAndApplyPropertyDescriptor
    /// with O undefined. `resultDesc` is complete, so every "Desc has a field"
    /// test in 10.1.6.3 is true and only the values are compared.
    fn emit_proxy_get_own_property_compatibility(
        &mut self,
        extensible: u32,
        completed: &CompletedPropertyDescriptorLocals,
        target_descriptor: &TargetDescriptorLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        const INCOMPATIBLE: &str =
            "Proxy getOwnPropertyDescriptor trap result is incompatible with target property";

        target_descriptor.emit_found_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // 10.1.6.3 step 2: a new property needs an extensible target.
        function.instruction(&Instruction::LocalGet(extensible));
        function.instruction(&Instruction::I64Eqz);
        self.emit_proxy_get_own_property_type_error_if(
            "Proxy getOwnPropertyDescriptor trap result incompatible with non-extensible target",
            function,
        )?;
        function.instruction(&Instruction::Else);
        // Step 4: only a non-configurable current property constrains Desc.
        target_descriptor.emit_configurable_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // 4.a
        completed.emit_configurable_i32(function);
        self.emit_proxy_get_own_property_type_error_if(
            "Proxy getOwnPropertyDescriptor trap result cannot report configurable for non-configurable target property",
            function,
        )?;
        // 4.b
        completed.emit_enumerable_i32(function);
        target_descriptor.emit_enumerable_i32(function);
        function.instruction(&Instruction::I32Ne);
        self.emit_proxy_get_own_property_type_error_if(INCOMPATIBLE, function)?;
        // 4.c: a complete descriptor is never generic.
        completed.emit_accessor_i32(function);
        target_descriptor.emit_accessor_i32(function);
        function.instruction(&Instruction::I32Ne);
        self.emit_proxy_get_own_property_type_error_if(INCOMPATIBLE, function)?;
        target_descriptor.emit_accessor_i32(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        // 4.d
        for (requested, current) in [
            (completed.getter(), target_descriptor.get),
            (completed.setter(), target_descriptor.set),
        ] {
            self.emit_tagged_payload_same_value_i32(
                requested.tag,
                requested.payload,
                current.tag,
                current.payload,
                function,
            )?;
            function.instruction(&Instruction::I32Eqz);
            self.emit_proxy_get_own_property_type_error_if(INCOMPATIBLE, function)?;
        }
        function.instruction(&Instruction::Else);
        // 4.e
        target_descriptor.emit_writable_i32(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        completed.emit_writable_i32(function);
        self.emit_proxy_get_own_property_type_error_if(INCOMPATIBLE, function)?;
        let requested = completed.value();
        self.emit_tagged_payload_same_value_i32(
            requested.tag,
            requested.payload,
            target_descriptor.value.tag,
            target_descriptor.value.payload,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        self.emit_proxy_get_own_property_type_error_if(INCOMPATIBLE, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Consumes an i32 condition and throws `message` from the executing
    /// builtin's Realm when it is non-zero.
    fn emit_proxy_get_own_property_type_error_if(
        &mut self,
        message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
