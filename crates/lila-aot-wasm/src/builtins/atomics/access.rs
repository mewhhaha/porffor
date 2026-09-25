use super::*;

/// The element index accepted by ValidateAtomicAccess, before the remaining
/// argument coercions.
///
/// Those coercions run user code that can detach a non-shared backing buffer
/// or shrink a resizable one, so this carries no memory address. The only way
/// to reach the backing store is to consume it into an
/// [`AtomicsElementAddress`].
#[must_use = "a validated Atomics index must become an element address before the buffer access"]
pub(super) struct ValidatedAtomicsIndex(u32);

impl ValidatedAtomicsIndex {
    /// Wraps the index once the owner's ValidateAtomicAccess bound against
    /// the entry witness's length has rejected every out-of-range value.
    pub(super) const fn after_range_check(index_local: u32) -> Self {
        Self(index_local)
    }
}

/// The address of one element in the current backing store of an integer
/// TypedArray, used by exactly the atomic access that follows.
///
/// The field is private to this module. An address therefore exists only
/// after [`FunctionBuilder::emit_revalidate_atomic_access`] has observed the
/// buffer again after every argument coercion, or after DoWait has required a
/// SharedArrayBuffer ([`FunctionBuilder::emit_shared_atomics_element_address`]).
pub(super) struct AtomicsElementAddress(u32);

impl AtomicsElementAddress {
    pub(super) const fn local(&self) -> u32 {
        self.0
    }
}

impl<'a> FunctionBuilder<'a> {
    /// RevalidateAtomicAccess(ta, byteIndexInBuffer) for `Atomics.load`,
    /// `store`, `compareExchange` and the read-modify-write family, run after
    /// the last argument coercion and before the buffer access.
    ///
    /// A fresh witness is ValidateTypedArrayBounds: a detached or out-of-bounds
    /// view throws a TypeError. The RangeError bound covers the whole element,
    /// `accessIndex ≥ TypedArrayLength(taRecord)`, which is
    /// `byteIndexInBuffer + elementSize > [[CachedBufferByteLength]]`. It
    /// differs from the specification's `byteIndexInBuffer ≥
    /// [[CachedBufferByteLength]]` only for a length-tracking view shrunk to a
    /// partial trailing element, where the access that follows would violate
    /// GetValueFromBuffer's and SetValueInBuffer's sufficient-bytes assertion.
    /// Detaching clears the backing pointer, so it is read again here.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_revalidate_atomic_access(
        &mut self,
        view: &TypedArrayViewLocals,
        buffer_payload_local: u32,
        byte_offset_local: u32,
        bytes_per_element_local: u32,
        index: ValidatedAtomicsIndex,
        range_error_message: &str,
        address_local: u32,
        function: &mut Function,
    ) -> Result<AtomicsElementAddress, EmitError> {
        let current_length_local = self.reserve_temp_local();
        self.emit_typed_array_witness(
            view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: current_length_local,
                access: TypedArrayAccessMode::Read,
            },
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index.0));
        function.instruction(&Instruction::LocalGet(current_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            range_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(current_length_local);

        Ok(self.emit_atomics_element_address(
            buffer_payload_local,
            byte_offset_local,
            bytes_per_element_local,
            index,
            address_local,
            function,
        ))
    }

    /// DoWait's element address. DoWait has already required a
    /// SharedArrayBuffer, which can be neither detached nor shrunk, so the
    /// index validated before the value and timeout coercions still addresses
    /// the live backing store and the specification performs no revalidation.
    pub(super) fn emit_shared_atomics_element_address(
        &mut self,
        buffer_payload_local: u32,
        byte_offset_local: u32,
        bytes_per_element_local: u32,
        index: ValidatedAtomicsIndex,
        address_local: u32,
        function: &mut Function,
    ) -> AtomicsElementAddress {
        self.emit_atomics_element_address(
            buffer_payload_local,
            byte_offset_local,
            bytes_per_element_local,
            index,
            address_local,
            function,
        )
    }

    fn emit_atomics_element_address(
        &mut self,
        buffer_payload_local: u32,
        byte_offset_local: u32,
        bytes_per_element_local: u32,
        index: ValidatedAtomicsIndex,
        address_local: u32,
        function: &mut Function,
    ) -> AtomicsElementAddress {
        self.emit_load_array_buffer_data(buffer_payload_local, address_local, function);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::LocalGet(byte_offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index.0));
        function.instruction(&Instruction::LocalGet(bytes_per_element_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        AtomicsElementAddress(address_local)
    }
}
