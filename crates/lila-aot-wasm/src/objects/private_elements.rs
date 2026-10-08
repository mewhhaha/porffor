use super::*;
use crate::gc_types::{
    PrivateElement, PrivateElementKind, PrivateElementSchema, PrivateElementTable,
    PrivateEnvironment, PrivateEnvironmentSchema, PrivateName, PrivateNameTable,
};
use crate::runtime_helpers::{
    HelperParameters, PrivateElementAddArguments, PrivateElementAddParameters,
    PrivateFieldDefineArguments, PrivateFieldDefineParameters,
};

/// Which lexical private name to resolve: a compile-time identity, or the
/// scope/ordinal pair a shared helper receives as operands.
enum PrivateNameSelector {
    Constant(PrivateNameId),
    Runtime { scope: I64Local, ordinal: I32Local },
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_private_name_token_to_local(
        &mut self,
        id: PrivateNameId,
        function: &mut Function,
    ) -> Result<GcLocal<PrivateName>, EmitError> {
        let schema = self.runtime_schema();
        let environment = schema.reserve_gc_local(function).initialize(
            self.current_private_environment().load(schema, function),
            function,
        );
        let result = self.emit_private_name_token_from_environment(id, &environment, function)?;
        environment.clear(function);
        Ok(result)
    }

    /// Both source access and instance initialization resolve the same lexical
    /// private-name identity. The caller supplies its actual captured class
    /// environment, rather than an integer token or the caller's class scope.
    pub(crate) fn emit_private_name_token_from_environment(
        &mut self,
        id: PrivateNameId,
        environment: &GcLocal<PrivateEnvironment, Nullable>,
        function: &mut Function,
    ) -> Result<GcLocal<PrivateName>, EmitError> {
        self.emit_private_name_token_resolve(
            PrivateNameSelector::Constant(id),
            environment,
            function,
        )
    }

    fn emit_private_name_token_resolve(
        &mut self,
        selector: PrivateNameSelector,
        environment: &GcLocal<PrivateEnvironment, Nullable>,
        function: &mut Function,
    ) -> Result<GcLocal<PrivateName>, EmitError> {
        let schema = self.runtime_schema();
        let current = schema
            .reserve_gc_local(function)
            .initialize(environment.load(schema, function), function);
        let name = schema
            .reserve_gc_local::<PrivateName, Nullable>(function)
            .initialize_null(schema, function);
        let scope = schema.reserve_i64_local(function);
        let (target_scope, ordinal) = match selector {
            PrivateNameSelector::Constant(_) => (None, schema.reserve_i32_local(function)),
            PrivateNameSelector::Runtime { scope, ordinal } => (Some(scope), ordinal),
        };
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        name.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        current.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_private_native_error(
            RuntimeErrorMessage::PRIVATE_ENVIRONMENT_IS_MISSING_ITS_DECLARED_NAME,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let owner = schema.reserve_gc_local(function).initialize(
            current.load(schema, function).require_non_null(function),
            function,
        );
        schema
            .struct_type::<PrivateEnvironment>()
            .field(PrivateEnvironmentSchema::CLASS_SCOPE)
            .read(&owner, schema, function)
            .store_i64(scope, function);
        scope.load(function);
        match (&selector, target_scope) {
            (PrivateNameSelector::Constant(id), _) => {
                function.instruction(&Instruction::I64Const(id.class_scope() as i64));
            }
            (PrivateNameSelector::Runtime { .. }, Some(target)) => target.load(function),
            (PrivateNameSelector::Runtime { .. }, None) => {
                unreachable!("runtime selector always carries its scope local")
            }
        }
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        let names = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PrivateEnvironment>()
                .field(PrivateEnvironmentSchema::NAMES)
                .read(&owner, schema, function)
                .reference(),
            function,
        );
        if let PrivateNameSelector::Constant(id) = &selector {
            function.instruction(&Instruction::I32Const(id.name_ordinal() as i32));
            ordinal.store(function);
        }
        name.replace(
            schema
                .array_type::<PrivateNameTable>()
                .read(&names, ordinal, schema, function)
                .reference()
                .nullable(),
            function,
        );
        names.clear(function);
        function.instruction(&Instruction::Else);
        current.replace(
            schema
                .struct_type::<PrivateEnvironment>()
                .field(PrivateEnvironmentSchema::PARENT)
                .read(&owner, schema, function)
                .reference(),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        owner.clear(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        let result = schema.reserve_gc_local(function).initialize(
            name.load(schema, function).require_non_null(function),
            function,
        );
        if matches!(selector, PrivateNameSelector::Constant(_)) {
            schema.release_i32_local(ordinal, function);
        }
        schema.release_i64_local(scope, function);
        name.clear(function);
        current.clear(function);
        Ok(result)
    }

    fn emit_private_native_error(
        &mut self,
        message: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let error = self.runtime_schema().reserve_completion(function);
        self.emit_throw_runtime_error(NativeErrorKind::TypeError, message, &error, function)?;
        self.completion().copy_from(&error, function);
        error.clear(function);
        self.emit_propagate_current_throw(function);
        Ok(())
    }

    pub(crate) fn emit_private_element_find(
        &mut self,
        receiver: &ValueLocals,
        name: &GcLocal<PrivateName>,
        function: &mut Function,
    ) -> GcLocal<PrivateElement, Nullable> {
        let schema = self.runtime_schema();
        let result = schema
            .reserve_gc_local::<PrivateElement, Nullable>(function)
            .initialize_null(schema, function);
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_object_header_projection(receiver, function),
            function,
        );
        let table = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<OrdinaryObject>()
                .field(OrdinaryObjectSchema::PRIVATE_ELEMENTS)
                .read(&header, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        schema
            .array_type::<PrivateElementTable>()
            .length(&table, schema, function);
        length.store(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        result.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        let candidate = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<PrivateElementTable>()
                .read(&table, index, schema, function)
                .reference(),
            function,
        );
        candidate.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let element = schema.reserve_gc_local(function).initialize(
            candidate.load(schema, function).require_non_null(function),
            function,
        );
        let stored_name = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PrivateElement>()
                .field(PrivateElementSchema::NAME)
                .read(&element, schema, function)
                .reference(),
            function,
        );
        stored_name.load(schema, function);
        name.load(schema, function);
        function.instruction(&Instruction::RefEq);
        self.open_frame(ControlFrameKind::If, function);
        result.replace(element.load(schema, function).nullable(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        stored_name.clear(function);
        element.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        candidate.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i32_local(index, function);
        schema.release_i32_local(length, function);
        table.clear(function);
        header.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result
    }

    pub(crate) fn emit_private_brand_has_i32(
        &mut self,
        receiver: &ValueLocals,
        name: &GcLocal<PrivateName>,
        result: I32Local,
        function: &mut Function,
    ) {
        let entry = self.emit_private_element_find(receiver, name, function);
        entry.load(self.runtime_schema(), function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        result.store(function);
        entry.clear(function);
    }

    /// Installs one already completed Method/Accessor/Brand row. Private-name
    /// identity lives on the row; no receiver list is retained by PrivateName.
    /// The whole PrivateElementAdd algorithm is outlined into one helper so a
    /// class with thousands of private fields emits only a call per install.
    pub(crate) fn emit_private_brand_add(
        &mut self,
        receiver: &ValueLocals,
        element: &GcLocal<PrivateElement>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        // Error construction retains the source caller's execution Realm.
        let realm = self.emit_execution_realm(function);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(
                PrivateElementAddArguments::new(receiver, element, &realm),
                base,
                function,
            )
            .store(result, function);
        realm.clear(function);
        Ok(())
    }

    pub(crate) fn compile_private_element_add_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::PrivateElementAdd);
        let parameters = self.helper_parameters::<PrivateElementAddParameters>(&mut function);
        self.push_scope();
        let schema = self.runtime_schema();
        let result = schema.reserve_completion(&mut function);
        self.emit_private_element_add_body(
            &parameters.receiver,
            &parameters.element,
            &result,
            &mut function,
        )?;
        self.completion().copy_from(&result, &mut function);
        result.clear(&mut function);
        self.pop_scope();
        self.clear_helper_execution_realm(&mut function);
        parameters.release(&mut function);
        self.completion().emit(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    // Module-private so the only caller is the helper compiler above; a second
    // caller would re-inline the find/append loops at every private field.
    fn emit_private_element_add_body(
        &mut self,
        receiver: &ValueLocals,
        element: &GcLocal<PrivateElement>,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        result.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::PRIVATE_FIELD_ACCESS_ON_WRONG_OBJECT,
            result,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let name = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PrivateElement>()
                .field(PrivateElementSchema::NAME)
                .read(element, schema, function)
                .reference(),
            function,
        );
        let previous = self.emit_private_element_find(receiver, &name, function);
        previous.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_private_element_append(receiver, element, function);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::PRIVATE_ELEMENT_ALREADY_INSTALLED_ON_OBJECT,
            result,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        previous.clear(function);
        name.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// One call per private field definition: name resolution, Field row
    /// construction and PrivateElementAdd all live in the helper body.
    pub(crate) fn emit_private_field_define(
        &mut self,
        receiver: &ValueLocals,
        private_environment: &GcLocal<PrivateEnvironment, Nullable>,
        id: PrivateNameId,
        value: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let scope = schema.reserve_i64_local(function);
        scope.set_constant(id.class_scope() as i64, function);
        let ordinal = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(id.name_ordinal() as i32));
        ordinal.store(function);
        let realm = self.emit_execution_realm(function);
        let base = self.runtime_helper_base()?;
        schema
            .call_helper(
                PrivateFieldDefineArguments::new(
                    receiver,
                    private_environment,
                    scope,
                    ordinal,
                    value,
                    &realm,
                ),
                base,
                function,
            )
            .store(result, function);
        realm.clear(function);
        schema.release_i32_local(ordinal, function);
        schema.release_i64_local(scope, function);
        Ok(())
    }

    pub(crate) fn compile_private_field_define_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::PrivateFieldDefine);
        let parameters = self.helper_parameters::<PrivateFieldDefineParameters>(&mut function);
        self.push_scope();
        let schema = self.runtime_schema();
        // A missing declared name throws through the helper's own completion.
        let name = self.emit_private_name_token_resolve(
            PrivateNameSelector::Runtime {
                scope: parameters.class_scope,
                ordinal: parameters.name_ordinal,
            },
            &parameters.private_environment,
            &mut function,
        )?;
        let stored = schema.reserve_gc_local(&mut function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&parameters.value, &mut function),
            &mut function,
        );
        let element = schema.reserve_gc_local(&mut function).initialize(
            schema.struct_type::<PrivateElement>().construct(
                (
                    crate::gc_types::GcOperand::reference(&name, schema),
                    crate::gc_types::GcOperand::constant(PrivateElementKind::Field),
                    crate::gc_types::GcOperand::nullable_reference(&stored, schema),
                    crate::gc_types::GcOperand::null(schema),
                    crate::gc_types::GcOperand::null(schema),
                    crate::gc_types::GcOperand::null(schema),
                ),
                &mut function,
            ),
            &mut function,
        );
        let result = schema.reserve_completion(&mut function);
        self.emit_private_element_add_body(&parameters.receiver, &element, &result, &mut function)?;
        self.completion().copy_from(&result, &mut function);
        result.clear(&mut function);
        element.clear(&mut function);
        stored.clear(&mut function);
        name.clear(&mut function);
        self.pop_scope();
        self.clear_helper_execution_realm(&mut function);
        parameters.release(&mut function);
        self.completion().emit(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_private_element_append(
        &self,
        receiver: &ValueLocals,
        element: &GcLocal<PrivateElement>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_object_header_projection(receiver, function),
            function,
        );
        let old = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<OrdinaryObject>()
                .field(OrdinaryObjectSchema::PRIVATE_ELEMENTS)
                .read(&header, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        let new_length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let array = schema.array_type::<PrivateElementTable>();
        array.length(&old, schema, function);
        length.store(function);
        length.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        new_length.store(function);
        new_length.load(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let next = schema.reserve_gc_local(function).initialize(
            array.filled(
                crate::gc_types::GcOperand::null(schema),
                new_length,
                function,
            ),
            function,
        );
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        let entry = schema.reserve_gc_local(function).initialize(
            array.read(&old, index, schema, function).reference(),
            function,
        );
        array.write(
            &next,
            index,
            crate::gc_types::GcOperand::reference(&entry, schema),
            schema,
            function,
        );
        entry.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        array.write(
            &next,
            length,
            crate::gc_types::GcOperand::nullable_reference(element, schema),
            schema,
            function,
        );
        schema
            .struct_type::<OrdinaryObject>()
            .field(OrdinaryObjectSchema::PRIVATE_ELEMENTS)
            .write(
                &header,
                crate::gc_types::GcOperand::reference(&next, schema),
                schema,
                function,
            );
        next.clear(function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(new_length, function);
        schema.release_i32_local(length, function);
        old.clear(function);
        header.clear(function);
    }

    pub(crate) fn compile_private_read_to_locals(
        &mut self,
        target: &TypedExpr,
        id: PrivateNameId,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let result = schema.reserve_completion(function);
        self.compile_expr_to_value(target, &receiver, function)?;
        self.emit_private_read_from_locals(&receiver, id, &result, function)?;
        self.completion().copy_from(&result, function);
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(result.value(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.clear(function);
        receiver.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }

    fn emit_private_read_from_locals(
        &mut self,
        receiver: &ValueLocals,
        id: PrivateNameId,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let name = self.emit_private_name_token_to_local(id, function)?;
        let entry = self.emit_private_element_find(receiver, &name, function);
        result.initialize(function);
        entry.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::PRIVATE_FIELD_ACCESS_ON_WRONG_OBJECT,
            result,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let element = schema.reserve_gc_local(function).initialize(
            entry.load(schema, function).require_non_null(function),
            function,
        );
        let kind = schema.reserve_i32_local(function);
        schema
            .struct_type::<PrivateElement>()
            .field(PrivateElementSchema::KIND)
            .read(&element, schema, function)
            .store(kind, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(
            crate::gc_types::GcI32Constant::encode(PrivateElementKind::Field),
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let field = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PrivateElement>()
                .field(PrivateElementSchema::FIELD_VALUE)
                .read(&element, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&field, result.value(), schema, function);
        result.set_normal(result.value(), function);
        field.clear(function);
        function.instruction(&Instruction::Else);
        kind.load(function);
        function.instruction(&Instruction::I32Const(
            crate::gc_types::GcI32Constant::encode(PrivateElementKind::Method),
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let method = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PrivateElement>()
                .field(PrivateElementSchema::METHOD)
                .read(&element, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&method, result.value(), schema, function);
        result.set_normal(result.value(), function);
        method.clear(function);
        function.instruction(&Instruction::Else);
        let getter = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PrivateElement>()
                .field(PrivateElementSchema::GETTER)
                .read(&element, schema, function)
                .reference(),
            function,
        );
        getter.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::PRIVATE_ACCESSOR_HAS_NO_GETTER,
            result,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let stored = schema.reserve_gc_local(function).initialize(
            getter.load(schema, function).require_non_null(function),
            function,
        );
        let callable = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &callable, schema, function);
        let args = self.emit_pre_evaluated_arg_vector(&[], function);
        self.emit_function_handle_call_with_argv_inner(
            &callable,
            Some(receiver),
            &args,
            result,
            PropagateCallThrow::LeaveInCompletion,
            function,
        )?;
        args.clear(function);
        callable.clear(function);
        stored.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        getter.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(kind, function);
        element.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        entry.clear(function);
        name.clear(function);
        Ok(())
    }

    pub(crate) fn compile_private_write_to_locals(
        &mut self,
        target: &TypedExpr,
        id: PrivateNameId,
        value: &TypedExpr,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let rhs = schema.reserve_value_local(function);
        let result = schema.reserve_completion(function);
        self.compile_expr_to_value(target, &receiver, function)?;
        self.compile_expr_to_value(value, &rhs, function)?;
        self.emit_private_write_from_locals(&receiver, id, &rhs, &result, function)?;
        self.completion().copy_from(&result, function);
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(&rhs, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.clear(function);
        rhs.clear(function);
        receiver.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }

    pub(crate) fn emit_private_write_from_locals(
        &mut self,
        receiver: &ValueLocals,
        id: PrivateNameId,
        value: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let name = self.emit_private_name_token_to_local(id, function)?;
        let entry = self.emit_private_element_find(receiver, &name, function);
        result.initialize(function);
        entry.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::PRIVATE_FIELD_ACCESS_ON_WRONG_OBJECT,
            result,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let element = schema.reserve_gc_local(function).initialize(
            entry.load(schema, function).require_non_null(function),
            function,
        );
        let kind = schema.reserve_i32_local(function);
        schema
            .struct_type::<PrivateElement>()
            .field(PrivateElementSchema::KIND)
            .read(&element, schema, function)
            .store(kind, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(
            crate::gc_types::GcI32Constant::encode(PrivateElementKind::Field),
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(value, function),
            function,
        );
        schema
            .struct_type::<PrivateElement>()
            .field(PrivateElementSchema::FIELD_VALUE)
            .write(
                &element,
                crate::gc_types::GcOperand::nullable_reference(&stored, schema),
                schema,
                function,
            );
        stored.clear(function);
        result.set_normal(value, function);
        function.instruction(&Instruction::Else);
        let setter = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PrivateElement>()
                .field(PrivateElementSchema::SETTER)
                .read(&element, schema, function)
                .reference(),
            function,
        );
        setter.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::PRIVATE_ELEMENT_HAS_NO_SETTER,
            result,
            function,
        )?;
        function.instruction(&Instruction::Else);
        let stored = schema.reserve_gc_local(function).initialize(
            setter.load(schema, function).require_non_null(function),
            function,
        );
        let callable = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &callable, schema, function);
        let args = self.emit_pre_evaluated_arg_vector(&[value], function);
        self.emit_function_handle_call_with_argv_inner(
            &callable,
            Some(receiver),
            &args,
            result,
            PropagateCallThrow::LeaveInCompletion,
            function,
        )?;
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        result.set_normal(value, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        args.clear(function);
        callable.clear(function);
        stored.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        setter.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(kind, function);
        element.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        entry.clear(function);
        name.clear(function);
        Ok(())
    }
}
