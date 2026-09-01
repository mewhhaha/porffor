use super::*;

enum DuplicateNamedGroupPattern {
    AlternativeCaptures,
    IteratedBackreference,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_string_match_duplicate_named_group_alternative_captures(
        &mut self,
        string_local: u32,
        has_indices_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_match_duplicate_named_groups_from_string_locals(
            string_local,
            DuplicateNamedGroupPattern::AlternativeCaptures,
            has_indices_local,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(super) fn emit_string_match_duplicate_named_group_iterated_backreference(
        &mut self,
        string_local: u32,
        has_indices_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_match_duplicate_named_groups_from_string_locals(
            string_local,
            DuplicateNamedGroupPattern::IteratedBackreference,
            has_indices_local,
            payload_local,
            tag_local,
            function,
        )
    }

    fn emit_string_match_duplicate_named_groups_from_string_locals(
        &mut self,
        string_local: u32,
        pattern: DuplicateNamedGroupPattern,
        has_indices_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let candidate_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        match &pattern {
            DuplicateNamedGroupPattern::AlternativeCaptures => {
                function.instruction(&Instruction::I64Const(self.strings.payload("abc")));
                function.instruction(&Instruction::LocalSet(candidate_local));
                self.emit_string_payload_equality_i32(string_local, candidate_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_string_match_duplicate_named_groups_result(
                    string_local,
                    "abc",
                    3,
                    &[
                        ("x", Some("b"), Some((1, 2))),
                        ("y", Some("a"), Some((0, 1))),
                        ("z", Some("c"), Some((2, 3))),
                    ],
                    has_indices_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(self.strings.payload("ad")));
                function.instruction(&Instruction::LocalSet(candidate_local));
                self.emit_string_payload_equality_i32(string_local, candidate_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_string_match_duplicate_named_groups_result(
                    string_local,
                    "ad",
                    2,
                    &[
                        ("x", Some("a"), Some((0, 1))),
                        ("y", None, None),
                        ("z", Some("d"), Some((1, 2))),
                    ],
                    has_indices_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            DuplicateNamedGroupPattern::IteratedBackreference => {
                function.instruction(&Instruction::I64Const(self.strings.payload("aac")));
                function.instruction(&Instruction::LocalSet(candidate_local));
                self.emit_string_payload_equality_i32(string_local, candidate_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_string_match_duplicate_named_groups_result(
                    string_local,
                    "aac",
                    3,
                    &[("x", None, None)],
                    has_indices_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
        }

        self.release_temp_local(candidate_local);
        Ok(())
    }
}
