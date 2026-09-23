use super::*;

impl FunctionBuilder<'_> {
    fn lower_width_add(&self, left: u32, right: u32, output: u32, function: &mut Function) {
        add_words(Local(left), Local(right), output, function);
        function.instruction(&Instruction::LocalGet(output));
        function.instruction(&Instruction::I64Const(WIDTH_LIMIT as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(output, Constant(WIDTH_LIMIT), function);
        function.instruction(&Instruction::End);
    }

    fn lower_width_multiply(&self, width: u32, count: u32, output: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(width));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(output, Constant(0), function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64Const(WIDTH_LIMIT as i64));
        function.instruction(&Instruction::LocalGet(width));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(output, Constant(WIDTH_LIMIT), function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(width));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(output));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_regexp_lower_widths(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) {
        let node = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        let child = self.reserve_temp_local();
        let next = self.reserve_temp_local();
        let child_kind = self.reserve_temp_local();
        let width = self.reserve_temp_local();
        let atom_width = self.reserve_temp_local();
        let child_width = self.reserve_temp_local();
        let first_capture = self.reserve_temp_local();
        let end_capture = self.reserve_temp_local();
        let minimum = self.reserve_temp_local();
        let maximum = self.reserve_temp_local();
        let flags = self.reserve_temp_local();
        let optional = self.reserve_temp_local();
        let overhead = self.reserve_temp_local();
        let reason = self.reserve_temp_local();
        let child_flags = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(compiler.node_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(compiler.node_count));
        function.instruction(&Instruction::LocalGet(compiler.node_capacity));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(node, Constant(1), function);
        self.lower_node_load(compiler, node, NodeWord::Kind, kind, function);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Root as i64));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(node, Local(compiler.node_count), function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(node));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        for (word, local) in [
            (NodeWord::Kind, kind),
            (NodeWord::First, child),
            (NodeWord::CaptureStart, first_capture),
            (NodeWord::CaptureEnd, end_capture),
            (NodeWord::Minimum, minimum),
            (NodeWord::Maximum, maximum),
            (NodeWord::Flags, flags),
        ] {
            self.lower_node_load(compiler, node, word, local, function);
        }
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Atom as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::NegativeLookahead as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(minimum));
        function.instruction(&Instruction::LocalGet(maximum));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(
            !(NODE_LAZY | NODE_ATOM_NULLABLE | NODE_OVERSIZED_BOUNDS) as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::LocalGet(first_capture));
        function.instruction(&Instruction::LocalGet(end_capture));
        function.instruction(&Instruction::I64GtU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::LocalGet(first_capture));
        function.instruction(&Instruction::LocalGet(end_capture));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(first_capture));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(end_capture));
        function.instruction(&Instruction::LocalGet(compiler.capture_count));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        set_word(atom_width, Constant(0), function);
        set_word(reason, Constant(0), function);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Atom as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(child));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(atom_width, Constant(1), function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(child));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.lower_checked_node(compiler, child, function);
        function.instruction(&Instruction::LocalGet(child));
        function.instruction(&Instruction::LocalGet(node));
        function.instruction(&Instruction::I64LeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        self.lower_node_load(compiler, child, NodeWord::Kind, child_kind, function);
        self.lower_node_load(compiler, child, NodeWord::Next, next, function);
        self.lower_node_load(compiler, child, NodeWord::Width, child_width, function);
        self.lower_node_load(compiler, child, NodeWord::Flags, child_flags, function);
        function.instruction(&Instruction::LocalGet(reason));
        function.instruction(&Instruction::LocalGet(child_flags));
        function.instruction(&Instruction::I64Const(NODE_WIDTH_REPETITION_LIMIT as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(reason));
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Sequence as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(child_kind));
        function.instruction(&Instruction::I64Const(NodeKind::Sequence as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(child_kind));
        function.instruction(&Instruction::I64Const(NodeKind::Root as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(child_kind));
        function.instruction(&Instruction::I64Const(NodeKind::Sequence as i64));
        function.instruction(&Instruction::I64Ne);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::LocalGet(next));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(overhead, Constant(2), function);
        self.lower_width_add(atom_width, overhead, atom_width, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.lower_width_add(atom_width, child_width, atom_width, function);
        function.instruction(&Instruction::LocalGet(next));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(next));
        function.instruction(&Instruction::LocalGet(child));
        function.instruction(&Instruction::I64LeU);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        set_word(child, Local(next), function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        set_word(overhead, Constant(0), function);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::Capture as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(first_capture));
        function.instruction(&Instruction::LocalGet(end_capture));
        function.instruction(&Instruction::I64Eq);
        self.lower_fail_if(compiler, CompileFailure::Corrupt, function);
        set_word(overhead, Constant(3), function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::NonCapture as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(first_capture));
        function.instruction(&Instruction::LocalGet(end_capture));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(overhead));
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(NodeKind::PositiveLookahead as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(overhead), Constant(4), overhead, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.lower_width_add(atom_width, overhead, atom_width, function);
        function.instruction(&Instruction::End);
        self.lower_store(
            compiler.nodes,
            node,
            NODE_BYTES,
            NodeWord::AtomWidth as u64,
            Local(atom_width),
            function,
        );
        self.lower_width_multiply(atom_width, minimum, width, function);
        function.instruction(&Instruction::LocalGet(maximum));
        function.instruction(&Instruction::I64Const(UNBOUNDED as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        add_words(Local(atom_width), Constant(2), optional, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(maximum));
        function.instruction(&Instruction::LocalGet(minimum));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(optional));
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(NODE_ATOM_NULLABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(atom_width));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(overhead));
        self.lower_width_multiply(overhead, optional, optional, function);
        function.instruction(&Instruction::End);
        self.lower_width_add(width, optional, width, function);
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::I64Const(NODE_OVERSIZED_BOUNDS as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(atom_width));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(width, Constant(0), function);
        function.instruction(&Instruction::Else);
        set_word(width, Constant(WIDTH_LIMIT), function);
        set_word(reason, Constant(NODE_WIDTH_REPETITION_LIMIT), function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // No emitted instruction means no assertion, capture, choice or input effect.
        function.instruction(&Instruction::LocalGet(atom_width));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(width, Constant(0), function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(width));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_word(reason, Constant(0), function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(flags));
        function.instruction(&Instruction::LocalGet(reason));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags));
        self.lower_store(
            compiler.nodes,
            node,
            NODE_BYTES,
            NodeWord::Flags as u64,
            Local(flags),
            function,
        );
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
            child_flags,
            reason,
            overhead,
            optional,
            flags,
            maximum,
            minimum,
            end_capture,
            first_capture,
            child_width,
            atom_width,
            width,
            child_kind,
            next,
            child,
            kind,
            node,
        ] {
            self.release_temp_local(local);
        }
    }
}
