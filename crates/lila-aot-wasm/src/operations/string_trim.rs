use super::*;

const ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8: [&[u8]; 19] = [
    &[0xC2, 0xA0],       // U+00A0
    &[0xE1, 0x9A, 0x80], // U+1680
    &[0xE2, 0x80, 0x80], // U+2000
    &[0xE2, 0x80, 0x81], // U+2001
    &[0xE2, 0x80, 0x82], // U+2002
    &[0xE2, 0x80, 0x83], // U+2003
    &[0xE2, 0x80, 0x84], // U+2004
    &[0xE2, 0x80, 0x85], // U+2005
    &[0xE2, 0x80, 0x86], // U+2006
    &[0xE2, 0x80, 0x87], // U+2007
    &[0xE2, 0x80, 0x88], // U+2008
    &[0xE2, 0x80, 0x89], // U+2009
    &[0xE2, 0x80, 0x8A], // U+200A
    &[0xE2, 0x80, 0xA8], // U+2028
    &[0xE2, 0x80, 0xA9], // U+2029
    &[0xE2, 0x80, 0xAF], // U+202F
    &[0xE2, 0x81, 0x9F], // U+205F
    &[0xE3, 0x80, 0x80], // U+3000
    &[0xEF, 0xBB, 0xBF], // U+FEFF
];

/// Which boundary or boundaries the shared ECMAScript string trim owns.
///
/// `TrimString` admits exactly start, end, or start+end. Keeping the raw core
/// behind this private domain makes the former `(false, false)` state
/// unrepresentable and forces a new mode through both exhaustive scans below.
enum EcmaTrimMode {
    Start,
    End,
    Both,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_ecmascript_trim_start_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ecmascript_trim_payload_from_locals(
            string_payload_local,
            EcmaTrimMode::Start,
            function,
        )
    }

    pub(crate) fn emit_ecmascript_trim_end_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ecmascript_trim_payload_from_locals(
            string_payload_local,
            EcmaTrimMode::End,
            function,
        )
    }

    pub(crate) fn emit_ecmascript_trim_both_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_ecmascript_trim_payload_from_locals(
            string_payload_local,
            EcmaTrimMode::Both,
            function,
        )
    }

    fn emit_ecmascript_trim_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        mode: EcmaTrimMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        match &mode {
            EcmaTrimMode::Start | EcmaTrimMode::Both => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(start_local));
                function.instruction(&Instruction::LocalGet(end_local));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(start_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(byte_local));
                for bytes in ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8 {
                    Self::emit_skip_utf8_whitespace_forward(
                        function,
                        end_local,
                        start_local,
                        byte_local,
                        bytes,
                    );
                }
                self.emit_is_ascii_whitespace_i32(byte_local, function);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(start_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(start_local));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            EcmaTrimMode::End => {}
        }

        match mode {
            EcmaTrimMode::End | EcmaTrimMode::Both => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(end_local));
                function.instruction(&Instruction::LocalGet(start_local));
                function.instruction(&Instruction::I64LeU);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(end_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(index_local));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(byte_local));
                for bytes in ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8 {
                    Self::emit_skip_utf8_whitespace_backward(
                        function,
                        start_local,
                        end_local,
                        byte_local,
                        bytes,
                    );
                }
                self.emit_is_ascii_whitespace_i32(byte_local, function);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::BrIf(1));
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::LocalSet(end_local));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            EcmaTrimMode::Start => {}
        }

        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(start_local));
        self.emit_string_slice_payload_from_locals(
            string_payload_local,
            start_local,
            len_local,
            function,
        )?;

        self.release_temp_local(len_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(end_local);
        self.release_temp_local(start_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }
}
