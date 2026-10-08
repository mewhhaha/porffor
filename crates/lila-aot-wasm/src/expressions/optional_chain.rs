//! One native optional-chain pipeline with an explicit final Reference destination.
use super::*;
use lila_ir::{OptionalChainCallReceiverIr, OptionalChainOperationIr};

/// A Delete destination cannot publish a callee receiver or perform a final Get.
enum TerminalDestination<'a> {
    Get {
        captured_receiver: Option<&'a lila_ir::CapturedCallReceiverIr>,
    },
    DeleteProperty {
        key: &'a PropertyKeyIr,
        shorted: bool,
        strictness: lila_ir::Strictness,
    },
}
impl TerminalDestination<'_> {
    fn set_shorted_result(&self, output: &ValueLocals, function: &mut Function) {
        match self {
            Self::Get { .. } => output.set_undefined(function),
            Self::DeleteProperty { .. } => output.set_scalar(ScalarValue::Boolean(true), function),
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn compile_optional_property_chain_to_value(
        &mut self,
        target: &TypedExpr,
        chain: &[OptionalChainOperationIr],
        captured_receiver: Option<&lila_ir::CapturedCallReceiverIr>,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_optional_chain_to_destination(
            target,
            chain,
            TerminalDestination::Get { captured_receiver },
            output,
            function,
        )
    }
    pub(crate) fn compile_delete_optional_property_chain_to_value(
        &mut self,
        delete: &lila_ir::DeleteOptionalPropertyChainIr,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_optional_chain_to_destination(
            delete.target(),
            delete.prefix(),
            TerminalDestination::DeleteProperty {
                key: delete.key(),
                shorted: delete.shorted(),
                strictness: delete.strictness(),
            },
            output,
            function,
        )
    }
    fn emit_optional_chain_short(
        &mut self,
        receiver: &ValueLocals,
        output: &ValueLocals,
        has_reference: I32Local,
        terminal: &TerminalDestination<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_nullish_tagged_i32(receiver.tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        receiver.set_undefined(function);
        terminal.set_shorted_result(output, function);
        function.instruction(&Instruction::I32Const(0));
        has_reference.store(function);
        function.instruction(&Instruction::Br(1));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_optional_chain_to_destination(
        &mut self,
        target: &TypedExpr,
        chain: &[OptionalChainOperationIr],
        terminal: TerminalDestination<'_>,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let reference_receiver = schema.reserve_value_local(function);
        let has_reference = schema.reserve_i32_local(function);
        let call_this = schema.reserve_value_local(function);
        self.compile_expr_to_value(target, &receiver, function)?;
        reference_receiver.set_undefined(function);
        call_this.set_undefined(function);
        function.instruction(&Instruction::I32Const(0));
        has_reference.store(function);
        output.copy_from(&receiver, function);
        let dynamic_receiver = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            ExprIr::Undefined,
        );

        // A grouped call starts a new short-circuit segment. The receiver stays
        // rooted through key evaluation/Get, and arguments precede callability.
        self.open_frame(ControlFrameKind::Block, function);
        for operation in chain {
            match operation {
                OptionalChainOperationIr::Property { key, shorted } => {
                    if *shorted {
                        self.emit_optional_chain_short(
                            &receiver,
                            output,
                            has_reference,
                            &terminal,
                            function,
                        )?;
                    }
                    reference_receiver.copy_from(&receiver, function);
                    function.instruction(&Instruction::I32Const(1));
                    has_reference.store(function);
                    // The shared property owner evaluates a computed key before
                    // its ordinary RequireObjectCoercible check.
                    self.compile_property_read_from_locals(
                        &dynamic_receiver,
                        key,
                        &receiver,
                        output,
                        function,
                    )?;
                    self.emit_propagate_current_throw_if_needed(function);
                    receiver.copy_from(output, function);
                }
                OptionalChainOperationIr::PrivateProperty {
                    private_name_id,
                    shorted,
                } => {
                    if *shorted {
                        self.emit_optional_chain_short(
                            &receiver,
                            output,
                            has_reference,
                            &terminal,
                            function,
                        )?;
                    }
                    reference_receiver.copy_from(&receiver, function);
                    function.instruction(&Instruction::I32Const(1));
                    has_reference.store(function);
                    self.push_scope();
                    let (operand, binding) = self.retain_expression_operand(
                        "\0optional.private.receiver",
                        &receiver,
                        function,
                    );
                    let result = self.compile_private_read_to_locals(
                        &operand,
                        *private_name_id,
                        output,
                        function,
                    );
                    self.pop_scope();
                    self.release_local_binding(binding, function);
                    result?;
                    self.emit_propagate_current_throw_if_needed(function);
                    receiver.copy_from(output, function);
                }
                OptionalChainOperationIr::Call {
                    args,
                    receiver: call_receiver,
                    shorted,
                    boundary_before,
                } => {
                    if *boundary_before {
                        self.pop_control(ControlFrameKind::Block);
                        function.instruction(&Instruction::End);
                        self.open_frame(ControlFrameKind::Block, function);
                    }
                    if *shorted {
                        self.emit_optional_chain_short(
                            &receiver,
                            output,
                            has_reference,
                            &terminal,
                            function,
                        )?;
                    }
                    let arguments = self.emit_call_args_vector(args, function)?;
                    match call_receiver {
                        OptionalChainCallReceiverIr::ReferenceOrUndefined => {
                            call_this.set_undefined(function);
                            has_reference.load(function);
                            self.open_frame(ControlFrameKind::If, function);
                            call_this.copy_from(&reference_receiver, function);
                            self.pop_control(ControlFrameKind::If);
                            function.instruction(&Instruction::End);
                        }
                        OptionalChainCallReceiverIr::CurrentThis => {
                            self.compile_this_to_locals(&call_this, function)?
                        }
                    }
                    let pending = schema.reserve_completion(function);
                    pending.initialize(function);
                    self.emit_function_or_proxy_call_with_argv(
                        &receiver, &call_this, &arguments, &pending, function,
                    )?;
                    self.completion().copy_from(&pending, function);
                    self.emit_propagate_current_throw_if_needed(function);
                    output.copy_from(pending.value(), function);
                    pending.clear(function);
                    arguments.clear(function);
                    function.instruction(&Instruction::I32Const(0));
                    has_reference.store(function);
                    receiver.copy_from(output, function);
                }
            }
        }
        if let TerminalDestination::DeleteProperty {
            key,
            shorted,
            strictness,
        } = &terminal
        {
            if *shorted {
                self.emit_optional_chain_short(
                    &receiver,
                    output,
                    has_reference,
                    &terminal,
                    function,
                )?;
            }
            // Retain the real final base; the existing DeleteProperty owner
            // evaluates/coerces its key and invokes [[Delete]] without GetValue.
            self.push_scope();
            let (operand, binding) =
                self.retain_expression_operand("\0optional.delete.receiver", &receiver, function);
            let deleted = schema.reserve_i32_local(function);
            let result = self.compile_delete_property_i32(&operand, key, *strictness, function);
            self.pop_scope();
            self.release_local_binding(binding, function);
            result?;
            deleted.store(function);
            output.set_boolean(deleted, function);
            schema.release_i32_local(deleted, function);
        }
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if let TerminalDestination::Get {
            captured_receiver: Some(captured),
        } = terminal
        {
            call_this.set_undefined(function);
            has_reference.load(function);
            self.open_frame(ControlFrameKind::If, function);
            call_this.copy_from(&reference_receiver, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            let storage = self
                .lookup_binding(captured.storage_name())
                .expect("optional Reference factory declares its activation-owned receiver");
            self.write_binding_from_locals(storage, &call_this, function);
        }
        call_this.clear(function);
        schema.release_i32_local(has_reference, function);
        reference_receiver.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
