use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn lower_width_add(
        &self,
        left: I64Local,
        right: I64Local,
        output: I64Local,
        function: &mut Function,
    ) {
        add_words(Local(left), Local(right), output, function);
        output.load(function);
        function.instruction(&Instruction::I64Const(WIDTH_LIMIT as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(output, Constant(WIDTH_LIMIT), function);
        function.instruction(&Instruction::End);
    }

    fn lower_width_multiply(
        &self,
        width: I64Local,
        count: I64Local,
        output: I64Local,
        function: &mut Function,
    ) {
        width.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(output, Constant(0), function);
        function.instruction(&Instruction::Else);
        count.load(function);
        function.instruction(&Instruction::I64Const(WIDTH_LIMIT as i64));
        width.load(function);
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(output, Constant(WIDTH_LIMIT), function);
        function.instruction(&Instruction::Else);
        width.load(function);
        count.load(function);
        function.instruction(&Instruction::I64Mul);
        output.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_regexp_lower_widths(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) {
        let node = self.runtime_schema().reserve_i64_local(function);
        let kind = self.runtime_schema().reserve_i64_local(function);
        let child = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        let child_kind = self.runtime_schema().reserve_i64_local(function);
        let width = self.runtime_schema().reserve_i64_local(function);
        let atom_width = self.runtime_schema().reserve_i64_local(function);
        let child_width = self.runtime_schema().reserve_i64_local(function);
        let child_flags = self.runtime_schema().reserve_i64_local(function);
        let pure_epsilon = self.runtime_schema().reserve_i32_local(function);
        let first_capture = self.runtime_schema().reserve_i64_local(function);
        let end_capture = self.runtime_schema().reserve_i64_local(function);
        let minimum = self.runtime_schema().reserve_i64_local(function);
        let maximum = self.runtime_schema().reserve_i64_local(function);
        let maximum_kind = self.runtime_schema().reserve_i64_local(function);
        let flags = self.runtime_schema().reserve_i64_local(function);
        let optional = self.runtime_schema().reserve_i64_local(function);
        let overhead = self.runtime_schema().reserve_i64_local(function);
        compiler.node_count.load(function);
        function.instruction(&Instruction::I64Eqz);
        compiler.node_count.load(function);
        compiler.node_capacity.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(node, Constant(1), function);
        self.lower_node_load(compiler, node, NodeWord::Kind, kind, function);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Root as i64));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(node, Local(compiler.node_count), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        node.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        for (word, local) in [
            (NodeWord::Kind, kind),
            (NodeWord::First, child),
            (NodeWord::CaptureStart, first_capture),
            (NodeWord::CaptureEnd, end_capture),
            (NodeWord::Minimum, minimum),
            (NodeWord::Maximum, maximum),
            (NodeWord::MaximumKind, maximum_kind),
            (NodeWord::Flags, flags),
        ] {
            self.lower_node_load(compiler, node, word, local, function);
        }
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Atom as i64));
        function.instruction(&Instruction::I64LtU);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::FiniteClassSet as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        minimum.load(function);
        maximum.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        flags.load(function);
        function.instruction(&Instruction::I64Const(
            !(NODE_LAZY | NODE_ATOM_NULLABLE) as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        for value in [minimum, maximum] {
            value.load(function);
            function.instruction(&Instruction::I64Const(BoundClass::Many as i64));
            function.instruction(&Instruction::I64GtU);
            self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        }
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        first_capture.load(function);
        end_capture.load(function);
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        first_capture.load(function);
        end_capture.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        first_capture.load(function);
        function.instruction(&Instruction::I64Eqz);
        end_capture.load(function);
        compiler.capture_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        self.lower_node_load(compiler, node, NodeWord::Direction, overhead, function);
        overhead.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::I32Const(0));
        pure_epsilon.store(function);
        for candidate in NodeKind::ALL {
            if candidate.composes_pure_epsilon() {
                kind.load(function);
                function.instruction(&Instruction::I64Const(candidate as i64));
                function.instruction(&Instruction::I64Eq);
                pure_epsilon.load(function);
                function.instruction(&Instruction::I32Or);
                pure_epsilon.store(function);
            }
        }
        first_capture.load(function);
        end_capture.load(function);
        function.instruction(&Instruction::I64Eq);
        pure_epsilon.load(function);
        function.instruction(&Instruction::I32And);
        pure_epsilon.store(function);
        set_word(atom_width, Constant(0), function);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Atom as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        child.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(atom_width, Constant(1), function);
        function.instruction(&Instruction::Else);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::FiniteClassSet as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        child.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        first_capture.load(function);
        end_capture.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.emit_regexp_finite_class_width(compiler, node, flags, atom_width, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        child.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.lower_checked_node(compiler, child, function);
        child.load(function);
        node.load(function);
        function.instruction(&Instruction::I64LeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_node_load(compiler, child, NodeWord::Kind, child_kind, function);
        self.lower_node_load(compiler, child, NodeWord::Next, next, function);
        self.lower_node_load(compiler, child, NodeWord::Width, child_width, function);
        self.lower_node_load(compiler, child, NodeWord::Flags, child_flags, function);
        child_flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_PURE_EPSILON as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        pure_epsilon.load(function);
        function.instruction(&Instruction::I32And);
        pure_epsilon.store(function);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Sequence as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        child_kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Sequence as i64));
        function.instruction(&Instruction::I64Eq);
        child_kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Root as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::Else);
        child_kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Sequence as i64));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        next.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(overhead, Constant(2), function);
        self.lower_width_add(atom_width, overhead, atom_width, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.lower_width_add(atom_width, child_width, atom_width, function);
        next.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        next.load(function);
        child.load(function);
        function.instruction(&Instruction::I64LeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        set_word(child, Local(next), function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        set_word(overhead, Constant(0), function);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::Capture as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        first_capture.load(function);
        end_capture.load(function);
        function.instruction(&Instruction::I64Eq);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(overhead, Constant(3), function);
        function.instruction(&Instruction::Else);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::NonCapture as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        first_capture.load(function);
        end_capture.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        overhead.store(function);
        kind.load(function);
        function.instruction(&Instruction::I64Const(NodeKind::PositiveLookahead as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(overhead), Constant(4), overhead, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.lower_width_add(atom_width, overhead, atom_width, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // Empty sequences are the base case. Only structural sequence,
        // alternation/noncapture composition may propagate this certificate;
        // nullable atoms and failable assertions never acquire it.
        pure_epsilon.load(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(atom_width, Constant(0), function);
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_PURE_EPSILON as i64));
        function.instruction(&Instruction::I64Or);
        flags.store(function);
        self.lower_store(
            compiler.nodes,
            node,
            NODE_BYTES,
            NodeWord::Flags as u64,
            Local(flags),
            function,
        );
        function.instruction(&Instruction::End);
        self.lower_store(
            compiler.nodes,
            node,
            NODE_BYTES,
            NodeWord::AtomWidth as u64,
            Local(atom_width),
            function,
        );
        self.emit_regexp_counted_body_condition(minimum, maximum, maximum_kind, flags, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(overhead, Constant(4), function);
        self.lower_width_add(atom_width, overhead, width, function);
        function.instruction(&Instruction::Else);
        self.emit_regexp_single_body_plus_condition(minimum, maximum_kind, flags, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(overhead, Constant(1), function);
        self.lower_width_add(atom_width, overhead, width, function);
        function.instruction(&Instruction::Else);
        self.lower_width_multiply(atom_width, minimum, width, function);
        maximum_kind.load(function);
        function.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Unbounded.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(atom_width), Constant(2), optional, function);
        function.instruction(&Instruction::Else);
        maximum.load(function);
        minimum.load(function);
        function.instruction(&Instruction::I64Sub);
        optional.store(function);
        flags.load(function);
        function.instruction(&Instruction::I64Const(NODE_ATOM_NULLABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::End);
        atom_width.load(function);
        function.instruction(&Instruction::I64Add);
        overhead.store(function);
        self.lower_width_multiply(overhead, optional, optional, function);
        function.instruction(&Instruction::End);
        self.lower_width_add(width, optional, width, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // No emitted instruction means no assertion, capture, choice or input effect.
        atom_width.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(width, Constant(0), function);
        function.instruction(&Instruction::End);
        self.lower_store(
            compiler.nodes,
            node,
            NODE_BYTES,
            NodeWord::Width as u64,
            Local(width),
            function,
        );
        add_words(Local(node), Constant(u64::MAX), node, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            overhead,
            optional,
            flags,
            maximum_kind,
            maximum,
            minimum,
            end_capture,
            first_capture,
            child_width,
            child_flags,
            atom_width,
            width,
            child_kind,
            next,
            child,
            kind,
            node,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.runtime_schema()
            .release_i32_local(pure_epsilon, function);
    }
}
