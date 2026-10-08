//! Home objects and class captures are inputs to function allocation.

use super::*;
use crate::gc_types::{
    BuiltinClosureCapture, Environment, FunctionContext, FunctionContextSchema, FunctionObject,
    GcLocal, GcStackReference, Nullable, OrdinaryObject, PrivateElementTable, PrivateEnvironment,
    StoredValue, TemplateSource, ValueArray, ValueLocals,
};

/// These are the objects on which source definitions can install methods.
/// A scalar or a completed call result cannot become an accidental home object.
#[derive(Clone, Copy)]
pub(crate) enum FunctionHomeObject<'a> {
    Object(&'a GcLocal<OrdinaryObject>),
    Constructor(&'a GcLocal<FunctionObject>),
}

impl FunctionHomeObject<'_> {
    fn store(
        &self,
        value: &ValueLocals,
        schema: &crate::gc_types::RuntimeSchema,
        function: &mut Function,
    ) {
        match self {
            Self::Object(object) => value.set_reference(*object, schema, function),
            Self::Constructor(constructor) => value.set_reference(*constructor, schema, function),
        }
    }
}

impl FunctionBuilder<'_> {
    /// Both class objects remain private while their complete defining
    /// captures, prototype properties and constructor back edge are installed.
    pub(super) fn emit_source_class_constructor_record(
        &mut self,
        meta: &WasmFunctionMeta,
        prototype: &GcLocal<OrdinaryObject>,
        constructor_parent: &ValueLocals,
        private_environment: &GcLocal<PrivateEnvironment, Nullable>,
        field_keys: &GcLocal<ValueArray, Nullable>,
        instance_private_methods: &GcLocal<PrivateElementTable, Nullable>,
        function: &mut Function,
    ) -> Result<GcLocal<FunctionObject>, EmitError> {
        use crate::gc_types::{FunctionObjectSchema, GcOperand};
        if meta.protocol().class_kind() != ClassFunctionKind::Constructor
            || meta.standard_builtin.is_some()
            || meta.host_builtin.is_some()
        {
            return Err(EmitError::unsupported(
                "class allocation requires its planned source constructor",
            ));
        }
        let schema = self.runtime_schema();
        let realm = self.load_current_realm(function);
        let lexical_environment = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(self.current_environment().load(schema, function), function);
        let home_object = schema.reserve_value_local(function);
        home_object.set_reference(prototype, schema, function);
        let template_source = match self.inherited_template_source(meta)? {
            Some(owner) => schema
                .reserve_gc_local::<TemplateSource, Nullable>(function)
                .initialize(owner.capture(schema, function).nullable(), function),
            None => schema
                .reserve_gc_local::<TemplateSource, Nullable>(function)
                .initialize_null(schema, function),
        };
        let capture = schema
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(function)
            .initialize_null(schema, function);
        let constructor = schema.reserve_gc_local(function).initialize(
            self.emit_source_function_record_from_captures(
                meta,
                FunctionAllocationInputs {
                    realm: &realm,
                    lexical_environment: &lexical_environment,
                    private_environment,
                    home_object: &home_object,
                    field_keys,
                    instance_private_methods,
                    template_source: &template_source,
                    builtin_capture: &capture,
                    internal_prototype: constructor_parent,
                    typed_array_element_kind: None,
                },
                FunctionPrototypeMaterialization::BootstrapSupplied,
                function,
            )?,
            function,
        );
        let header = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::OBJECT)
                .read(&constructor, schema, function)
                .reference(),
            function,
        );
        let key = self.emit_function_string_key("prototype", function)?;
        self.emit_object_append_data_property_with_flags(
            &header,
            &key,
            &home_object,
            false,
            false,
            false,
            function,
        )?;
        key.clear(function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&home_object, function),
            function,
        );
        schema
            .struct_type::<FunctionObject>()
            .field(FunctionObjectSchema::PUBLIC_PROTOTYPE_CACHE)
            .write(
                &constructor,
                GcOperand::reference(&stored, schema),
                schema,
                function,
            );
        stored.clear(function);
        let constructor_value = schema.reserve_value_local(function);
        constructor_value.set_reference(&constructor, schema, function);
        let key = self.emit_function_string_key("constructor", function)?;
        self.emit_object_append_data_property_with_flags(
            prototype,
            &key,
            &constructor_value,
            true,
            false,
            true,
            function,
        )?;
        key.clear(function);
        constructor_value.clear(function);
        header.clear(function);
        capture.clear(function);
        template_source.clear(function);
        home_object.clear(function);
        lexical_environment.clear(function);
        realm.clear(function);
        Ok(constructor)
    }

    /// Static initializer bodies have the constructor as their home object.
    /// Their distinct context retains the same class lexical/private captures.
    pub(super) fn emit_class_static_execution_context(
        &mut self,
        constructor: &GcLocal<FunctionObject>,
        function: &mut Function,
    ) -> GcLocal<FunctionContext> {
        use crate::gc_types::{FunctionObjectSchema, GcOperand};
        let schema = self.runtime_schema();
        let context = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::CONTEXT)
                .read(constructor, schema, function)
                .reference(),
            function,
        );
        let row = schema.struct_type::<FunctionContext>();
        let realm = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::REALM)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        let lexical = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::LEXICAL_ENVIRONMENT)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        let private = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::PRIVATE_ENVIRONMENT)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        let field_keys = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::FIELD_KEYS)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        let template = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::TEMPLATE_SOURCE)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        let home_value = schema.reserve_value_local(function);
        home_value.set_reference(constructor, schema, function);
        let home = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&home_value, function),
            function,
        );
        let static_context = schema.reserve_gc_local(function).initialize(
            row.construct(
                (
                    GcOperand::reference(&realm, schema),
                    GcOperand::reference(&lexical, schema),
                    GcOperand::reference(&private, schema),
                    GcOperand::null(schema),
                    GcOperand::reference(&home, schema),
                    GcOperand::reference(&field_keys, schema),
                    GcOperand::reference(&template, schema),
                    GcOperand::null(schema),
                    GcOperand::null(schema),
                ),
                function,
            ),
            function,
        );
        home.clear(function);
        home_value.clear(function);
        template.clear(function);
        field_keys.clear(function);
        private.clear(function);
        lexical.clear(function);
        realm.clear(function);
        context.clear(function);
        static_context
    }

    /// The defining environment and home object are rooted before publication.
    /// Class callers supply their actual private environment and field-key list.
    pub(crate) fn emit_method_function_record(
        &mut self,
        meta: &WasmFunctionMeta,
        home: FunctionHomeObject<'_>,
        private_environment: &GcLocal<PrivateEnvironment, Nullable>,
        field_keys: &GcLocal<ValueArray, Nullable>,
        function: &mut Function,
    ) -> Result<GcStackReference<FunctionObject>, EmitError> {
        if !meta.has_home_object_execution_context() {
            return Err(EmitError::unsupported(
                "method allocation requires a planned home-object owner",
            ));
        }
        let schema = self.runtime_schema();
        let realm = self.load_current_realm(function);
        let lexical_environment = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(self.current_environment().load(schema, function), function);
        let home_object = schema.reserve_value_local(function);
        home.store(&home_object, schema, function);
        let template_source = match self.inherited_template_source(meta)? {
            Some(owner) => schema
                .reserve_gc_local::<TemplateSource, Nullable>(function)
                .initialize(owner.capture(schema, function).nullable(), function),
            None => schema
                .reserve_gc_local::<TemplateSource, Nullable>(function)
                .initialize_null(schema, function),
        };
        let internal_prototype = schema.reserve_value_local(function);
        self.emit_source_callable_internal_prototype(meta, &realm, &internal_prototype, function);
        let capture = schema
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(function)
            .initialize_null(schema, function);
        let instance_private_methods = schema
            .reserve_gc_local::<PrivateElementTable, Nullable>(function)
            .initialize_null(schema, function);
        let result = self.emit_source_function_record_from_captures(
            meta,
            FunctionAllocationInputs {
                realm: &realm,
                lexical_environment: &lexical_environment,
                private_environment,
                home_object: &home_object,
                field_keys,
                instance_private_methods: &instance_private_methods,
                template_source: &template_source,
                builtin_capture: &capture,
                internal_prototype: &internal_prototype,
                typed_array_element_kind: None,
            },
            FunctionPrototypeMaterialization::Automatic,
            function,
        )?;
        instance_private_methods.clear(function);
        capture.clear(function);
        internal_prototype.clear(function);
        template_source.clear(function);
        home_object.clear(function);
        lexical_environment.clear(function);
        realm.clear(function);
        Ok(result)
    }

    fn emit_source_callable_internal_prototype(
        &mut self,
        meta: &WasmFunctionMeta,
        realm: &GcLocal<crate::gc_types::RealmRecord>,
        output: &ValueLocals,
        function: &mut Function,
    ) {
        let slot = match meta.protocol().execution_kind() {
            FunctionExecutionKind::Ordinary => NonArrayRealmIntrinsicSlot::FunctionPrototype,
            FunctionExecutionKind::Generator => {
                NonArrayRealmIntrinsicSlot::GeneratorFunctionPrototype
            }
            FunctionExecutionKind::Async => NonArrayRealmIntrinsicSlot::AsyncFunctionPrototype,
            FunctionExecutionKind::AsyncGenerator => {
                NonArrayRealmIntrinsicSlot::AsyncGeneratorFunctionPrototype
            }
        };
        self.emit_load_non_array_realm_intrinsic(realm, slot, output, function);
    }

    /// Generated element bodies receive a distinct active function while all
    /// defining captures come from the retained class execution context.
    fn emit_class_element_function_record(
        &mut self,
        meta: &WasmFunctionMeta,
        class_context: &GcLocal<FunctionContext>,
        function: &mut Function,
    ) -> Result<GcStackReference<FunctionObject>, EmitError> {
        if meta.class_element_execution_kind == ClassElementExecutionKind::None {
            return Err(EmitError::unsupported(
                "class element call requires a planned element body",
            ));
        }
        let schema = self.runtime_schema();
        let row = schema.struct_type::<FunctionContext>();
        let realm = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::REALM)
                .read(class_context, schema, function)
                .reference(),
            function,
        );
        let lexical_environment = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::LEXICAL_ENVIRONMENT)
                .read(class_context, schema, function)
                .reference(),
            function,
        );
        let private_environment = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::PRIVATE_ENVIRONMENT)
                .read(class_context, schema, function)
                .reference(),
            function,
        );
        let stored_home = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::HOME_OBJECT)
                .read(class_context, schema, function)
                .reference(),
            function,
        );
        let home = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored_home, &home, schema, function);
        let field_keys = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::FIELD_KEYS)
                .read(class_context, schema, function)
                .reference(),
            function,
        );
        let template_source = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::TEMPLATE_SOURCE)
                .read(class_context, schema, function)
                .reference(),
            function,
        );
        let instance_private_methods = schema.reserve_gc_local(function).initialize(
            row.field(FunctionContextSchema::INSTANCE_PRIVATE_METHODS)
                .read(class_context, schema, function)
                .reference(),
            function,
        );
        let internal_prototype = schema.reserve_value_local(function);
        self.emit_source_callable_internal_prototype(meta, &realm, &internal_prototype, function);
        let capture = schema
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(function)
            .initialize_null(schema, function);
        let result = self.emit_source_function_record_from_captures(
            meta,
            FunctionAllocationInputs {
                realm: &realm,
                lexical_environment: &lexical_environment,
                private_environment: &private_environment,
                home_object: &home,
                field_keys: &field_keys,
                instance_private_methods: &instance_private_methods,
                template_source: &template_source,
                builtin_capture: &capture,
                internal_prototype: &internal_prototype,
                typed_array_element_kind: None,
            },
            FunctionPrototypeMaterialization::Automatic,
            function,
        )?;
        instance_private_methods.clear(function);
        capture.clear(function);
        internal_prototype.clear(function);
        template_source.clear(function);
        field_keys.clear(function);
        home.clear(function);
        stored_home.clear(function);
        private_environment.clear(function);
        lexical_environment.clear(function);
        realm.clear(function);
        Ok(result)
    }

    pub(crate) fn emit_direct_class_element_js_call(
        &mut self,
        meta: &WasmFunctionMeta,
        class_context: &GcLocal<FunctionContext>,
        this_value: Option<&ValueLocals>,
        arguments: &[&ValueLocals],
        result: &crate::gc_types::CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let callable = schema.reserve_gc_local(function).initialize(
            self.emit_class_element_function_record(meta, class_context, function)?,
            function,
        );
        let callee = schema.reserve_value_local(function);
        callee.set_reference(&callable, schema, function);
        let receiver = schema.reserve_value_local(function);
        match this_value {
            Some(value) => receiver.copy_from(value, function),
            None => receiver.set_undefined(function),
        }
        let list = self.emit_pre_evaluated_arg_vector(arguments, function);
        self.emit_function_or_proxy_call_with_argv(&callee, &receiver, &list, result, function)?;
        list.clear(function);
        receiver.clear(function);
        callee.clear(function);
        callable.clear(function);
        Ok(())
    }

    pub(crate) fn emit_initialize_instance_elements(
        &mut self,
        constructor_meta: &WasmFunctionMeta,
        class_context: &GcLocal<FunctionContext>,
        receiver: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        use crate::gc_types::PrivateElementTable;
        let Some(plan) = constructor_meta.class_instance_element_plan.clone() else {
            return Ok(());
        };
        let schema = self.runtime_schema();
        let private_environment = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::PRIVATE_ENVIRONMENT)
                .read(class_context, schema, function)
                .reference(),
            function,
        );
        let result = schema.reserve_completion(function);
        result.initialize(function);
        let element_result = schema.reserve_completion(function);
        let value = schema.reserve_value_local(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);

        // Constructor [[PrivateMethods]] holds completed method/accessor rows.
        // The names are identities, independent of their receiver's definitions.
        if !plan.private_method_brands.is_empty() {
            let definitions = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<FunctionContext>()
                    .field(FunctionContextSchema::INSTANCE_PRIVATE_METHODS)
                    .read(class_context, schema, function)
                    .reference()
                    .require_non_null(function),
                function,
            );
            let index = schema.reserve_i32_local(function);
            let length = schema.reserve_i32_local(function);
            function.instruction(&Instruction::I32Const(0));
            index.store(function);
            schema
                .array_type::<PrivateElementTable>()
                .length(&definitions, schema, function);
            length.store(function);
            let installed = self.open_frame(ControlFrameKind::Block, function);
            let next = self.open_frame(ControlFrameKind::Loop, function);
            index.load(function);
            length.load(function);
            function.instruction(&Instruction::I32GeU);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_branch_to_target(installed, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            let definition = schema.reserve_gc_local(function).initialize(
                schema
                    .array_type::<PrivateElementTable>()
                    .read(&definitions, index, schema, function)
                    .reference()
                    .require_non_null(function),
                function,
            );
            self.emit_private_brand_add(receiver, &definition, &element_result, function)?;
            definition.clear(function);
            self.emit_bind_metadata_abrupt_exit(&element_result, &result, exit, function);
            index.load(function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            index.store(function);
            self.emit_branch_to_target(next, function);
            self.pop_control(ControlFrameKind::Loop);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
            schema.release_i32_local(length, function);
            schema.release_i32_local(index, function);
            definitions.clear(function);
        }

        for element in plan.elements {
            let (key, initializer_id) = match element {
                ClassInstanceElementIr::Field(field) => (field.key, field.init_function_id),
                ClassInstanceElementIr::AutoAccessorBacking(accessor) => (
                    ClassFieldKeyIr::Private(accessor.backing_name.private_name_id()),
                    accessor.init_function_id,
                ),
            };
            // Each field's initializer runs before its own definition, with
            // all methods/accessors already present on the receiver.
            match initializer_id {
                Some(id) => {
                    let meta = self.functions.get(&id).cloned().ok_or_else(|| {
                        EmitError::unsupported(format!("unknown class initializer {id}"))
                    })?;
                    if meta.class_element_execution_kind
                        != ClassElementExecutionKind::InstanceFieldInitializer
                    {
                        return Err(EmitError::unsupported(
                            "instance field requires its planned initializer protocol",
                        ));
                    }
                    self.emit_direct_class_element_js_call(
                        &meta,
                        class_context,
                        Some(receiver),
                        &[],
                        &element_result,
                        function,
                    )?;
                    self.emit_bind_metadata_abrupt_exit(&element_result, &result, exit, function);
                    value.copy_from(element_result.value(), function);
                }
                None => value.set_undefined(function),
            }
            match key {
                ClassFieldKeyIr::Private(id) => {
                    self.emit_private_field_define(
                        receiver,
                        &private_environment,
                        id,
                        &value,
                        &element_result,
                        function,
                    )?;
                }
                public => {
                    let raw_key = schema.reserve_value_local(function);
                    match public {
                        ClassFieldKeyIr::Public(text) => {
                            let string = schema.reserve_gc_local(function).initialize(
                                self.emit_interned_string_reference(&text, function)?,
                                function,
                            );
                            raw_key.set_reference(&string, schema, function);
                            string.clear(function);
                        }
                        ClassFieldKeyIr::ComputedPublic(slot) => {
                            let keys = schema.reserve_gc_local(function).initialize(
                                schema
                                    .struct_type::<FunctionContext>()
                                    .field(FunctionContextSchema::FIELD_KEYS)
                                    .read(class_context, schema, function)
                                    .reference()
                                    .require_non_null(function),
                                function,
                            );
                            let index = schema.reserve_i32_local(function);
                            function.instruction(&Instruction::I32Const(slot as i32));
                            index.store(function);
                            let stored = schema.reserve_gc_local(function).initialize(
                                schema
                                    .array_type::<ValueArray>()
                                    .read(&keys, index, schema, function)
                                    .reference(),
                                function,
                            );
                            schema
                                .struct_type::<StoredValue>()
                                .read_into(&stored, &raw_key, schema, function);
                            stored.clear(function);
                            schema.release_i32_local(index, function);
                            keys.clear(function);
                        }
                        ClassFieldKeyIr::Private(_) => {
                            unreachable!("private field selected before public key path")
                        }
                    }
                    // Cached keys were converted while the class was defined.
                    // This constructor accepts only their String/Symbol form.
                    let property_key =
                        self.emit_value_to_property_key_locals(&raw_key, function)?;
                    raw_key.clear(function);
                    self.emit_define_public_class_field(
                        receiver,
                        &property_key,
                        &value,
                        &element_result,
                        function,
                    )?;
                    property_key.clear(function);
                }
            }
            self.emit_bind_metadata_abrupt_exit(&element_result, &result, exit, function);
        }
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        value.clear(function);
        element_result.clear(function);
        private_environment.clear(function);
        self.completion().copy_from(&result, function);
        result.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }
}
