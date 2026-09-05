from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(text.replace(old, new, 1))


replace_once(
    "crates/lila-aot-wasm/src/environments.rs",
    '''    pub(crate) fn emit_enter_lexical_environment(
        &mut self,
        environment: &LexicalEnvironmentIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let parent_env_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(parent_env_local));
        self.emit_heap_alloc_const(
            ENV_SLOT_BASE_OFFSET + environment.bindings.len() as u64 * ENV_SLOT_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.store_i64_local_at_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        for binding in &environment.bindings {
            self.store_i64_const_at_offset(
                self.current_env_local,
                Self::env_slot_offset(binding.slot, ENV_SLOT_TAG_OFFSET),
                ENV_SLOT_UNINITIALIZED_TAG as u64,
                function,
            );
            self.store_i64_const_at_offset(
                self.current_env_local,
                Self::env_slot_offset(binding.slot, ENV_SLOT_PAYLOAD_OFFSET),
                0,
                function,
            );
        }
        self.release_temp_local(parent_env_local);

        self.begin_existing_lexical_environment_scope(environment);
        Ok(())
    }
''',
    '''    pub(crate) fn emit_enter_lexical_environment(
        &mut self,
        environment: &LexicalEnvironmentIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_allocate_lexical_environment_record(environment, function)?;
        self.begin_existing_lexical_environment_scope(environment);
        Ok(())
    }

    /// Allocate one lexical Environment Record without changing the compiler's
    /// binding view. Resumable owners use this when a fresh runtime arm and a
    /// resumed runtime arm must converge on the same compile-time scope.
    pub(crate) fn emit_allocate_lexical_environment_record(
        &mut self,
        environment: &LexicalEnvironmentIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let parent_env_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(parent_env_local));
        self.emit_heap_alloc_const(
            ENV_SLOT_BASE_OFFSET + environment.bindings.len() as u64 * ENV_SLOT_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.store_i64_local_at_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        for binding in &environment.bindings {
            self.store_i64_const_at_offset(
                self.current_env_local,
                Self::env_slot_offset(binding.slot, ENV_SLOT_TAG_OFFSET),
                ENV_SLOT_UNINITIALIZED_TAG as u64,
                function,
            );
            self.store_i64_const_at_offset(
                self.current_env_local,
                Self::env_slot_offset(binding.slot, ENV_SLOT_PAYLOAD_OFFSET),
                0,
                function,
            );
        }
        self.release_temp_local(parent_env_local);
        Ok(())
    }
''',
)

replace_once(
    "crates/lila-aot-wasm/src/control_flow.rs",
    '''    const fn resume_tag_offset(&self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_RESUME_TAG_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
        }
    }
}
''',
    '''    const fn resume_tag_offset(&self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_RESUME_TAG_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
        }
    }

    const fn lexical_environment_offset(&self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_ENV_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET,
        }
    }
}
''',
)

replace_once(
    "crates/lila-aot-wasm/src/control_flow.rs",
    '''        if body_suspends {
            // A per-iteration environment is entered at the loop head and left
            // after the body, both inside the same invocation. Split the
            // iteration and the resume would enter a second environment while
            // the first is still current, and leave only one of them.
            if lexical_environment
                .and_then(|environment| environment.iteration_environment.as_ref())
                .is_some()
            {
                return Err(EmitError::unsupported(
                    "for-await-of with a per-iteration lexical environment and a body suspension",
                ));
            }
            // `compile_async_block_contents` enters a body block's own
''',
    '''        if body_suspends {
            // `compile_async_block_contents` enters a body block's own
''',
)

replace_once(
    "crates/lila-aot-wasm/src/control_flow.rs",
    '''        let resume_state_offset = resume_layout.resume_state_offset();
        let resume_payload_offset = resume_layout.resume_payload_offset();
        let resume_tag_offset = resume_layout.resume_tag_offset();
        let state_local = self.reserve_temp_local();
''',
    '''        let resume_state_offset = resume_layout.resume_state_offset();
        let resume_payload_offset = resume_layout.resume_payload_offset();
        let resume_tag_offset = resume_layout.resume_tag_offset();
        let activation_environment_offset = resume_layout.lexical_environment_offset();
        let iteration_environment = lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref());
        let state_local = self.reserve_temp_local();
''',
)

replace_once(
    "crates/lila-aot-wasm/src/control_flow.rs",
    '''        self.read_binding_to_locals(done_storage, done_payload_local, done_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(done_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.branch_if_to_label(break_frame.label);
        // Binding the loop variable belongs to the start of an iteration. On a
        // body resume the iteration is already under way, the value locals hold
        // nothing this invocation assigned, and the binding still holds the
        // value the body was suspended with — so rebinding would overwrite it
        // with garbage. A per-iteration environment is refused above when the
        // body can suspend, so entering one here stays a value-path-only step.
        if body_suspends {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
        }
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-await-of lexical storage must exist");
        self.write_binding_from_locals(storage, value_payload_local, value_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        if body_suspends {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // Emitted at the same control depth as before the gates above, so every
        // `break`/`continue`/`return` inside the body still resolves to the same
        // frame it did when a suspending body was refused outright.
        self.compile_statement(body, function)?;
        self.finally_stack.pop();
''',
    '''        if body_suspends && iteration_environment.is_some() {
            // A body-resume invocation already owns an in-flight iteration, so
            // `done` is known false. Reading the parent-owned slot before the
            // child environment is attached would use the wrong runtime base;
            // perform the observable done test only on the value-resume path.
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.read_binding_to_locals(done_storage, done_payload_local, done_tag_local, function)?;
            function.instruction(&Instruction::LocalGet(done_payload_local));
            function.instruction(&Instruction::I32WrapI64);
            function.branch_if_to_label(break_frame.label);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        } else {
            self.read_binding_to_locals(done_storage, done_payload_local, done_tag_local, function)?;
            function.instruction(&Instruction::LocalGet(done_payload_local));
            function.instruction(&Instruction::I32WrapI64);
            function.branch_if_to_label(break_frame.label);
        }

        // A captured `let`/`const` head owns one fresh Environment Record per
        // iteration. The value-resume invocation allocates it and publishes its
        // exact pointer in the activation. A body-resume invocation starts with
        // that pointer already restored by function entry, so both runtime arms
        // converge before the compiler attaches one binding view.
        let iteration_cleanup_frame = if body_suspends && iteration_environment.is_some() {
            Some(self.open_frame(ControlFrameKind::Block, function))
        } else {
            None
        };
        if let Some(environment) = iteration_environment {
            if body_suspends {
                function.instruction(&Instruction::LocalGet(state_local));
                function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_allocate_lexical_environment_record(environment, function)?;
                self.store_i64_local_at_offset(
                    activation_local,
                    activation_environment_offset,
                    self.current_env_local,
                    function,
                );
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.push_scope();
                self.begin_existing_lexical_environment_scope(environment);
                self.finally_stack.push(ControlTarget {
                    environment_depth: self.environment_depth,
                    ..iteration_cleanup_frame
                        .expect("resumable iteration environment needs a cleanup frame")
                });
            } else {
                self.emit_enter_lexical_environment(environment, function)?;
            }
        }

        // Binding the loop variable belongs only to the invocation that resumed
        // from `next()`. On a body resume the retained Environment Record already
        // contains the value and any mutations performed before suspension.
        if body_suspends {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-await-of lexical storage must exist");
        self.write_binding_from_locals(storage, value_payload_local, value_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        if body_suspends {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.compile_statement(body, function)?;
        if body_suspends && iteration_environment.is_some() {
            self.finally_stack.pop();
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
            self.emit_leave_lexical_environment(function);
            self.pop_scope();
            self.store_i64_local_at_offset(
                activation_local,
                activation_environment_offset,
                self.current_env_local,
                function,
            );
            // Normal fallthrough continues into the ordinary iterator-close
            // decision. Abrupt completions are routed only after the parent
            // environment is again authoritative in the activation.
            self.emit_dispatch_async_completion(function)?;
        }
        self.finally_stack.pop();
''',
)

replace_once(
    "crates/lila-aot-wasm/src/control_flow.rs",
    '''        // Pairs with the per-iteration enter at the loop head, and reads
        // `resume_is_throw_local`. Both are sound only while an iteration begins
        // and ends inside one invocation, which is why a per-iteration
        // environment and a suspending body are refused together above.
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            debug_assert!(
                !body_suspends,
                "a per-iteration environment and a body suspension must have been refused"
            );
            function.instruction(&Instruction::LocalGet(resume_is_throw_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_leave_lexical_environment(function);
            function.instruction(&Instruction::End);
        }
''',
    '''        // A non-suspending iteration still creates and retires its lexical
        // environment in one invocation. Resumable iteration environments were
        // already retired by the activation-aware cleanup immediately after the
        // body, so they must not be left a second time here.
        if iteration_environment.is_some() && !body_suspends {
            function.instruction(&Instruction::LocalGet(resume_is_throw_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_leave_lexical_environment(function);
            function.instruction(&Instruction::End);
        }
''',
)

replace_once(
    "crates/lila-aot-wasm/src/emit.rs",
    '''            if async_generator_contains_suspension(body, AsyncGeneratorSuspension::Yield)
                && (lexical_environment
                    .as_ref()
                    .and_then(|environment| environment.iteration_environment.as_ref())
                    .is_some()
                    || matches!(body.as_ref(), StatementIr::Block(block) if block.lexical_environment.is_some()))
            {
                return Some(
                    "for-await-of with a per-iteration lexical environment and a body suspension",
                );
            }
''',
    '''            if async_generator_contains_suspension(body, AsyncGeneratorSuspension::Yield)
                && matches!(body.as_ref(), StatementIr::Block(block) if block.lexical_environment.is_some())
            {
                return Some(
                    "for-await-of with a block-scoped body environment and a body suspension",
                );
            }
''',
)


test_path = Path("crates/lila-engine/tests/aot_async_for_of.rs")
test_text = test_path.read_text()
test_marker = '''#[test]
fn var_head_survives_multiple_body_suspensions() {
    assert_suspended_iteration_values("var");
}
'''
if test_text.count(test_marker) != 1:
    raise SystemExit("aot_async_for_of.rs: insertion marker changed")
test_addition = test_marker + r'''

#[test]
fn captured_let_head_reuses_one_cell_after_resume_and_fresh_cells_between_iterations() {
    let source = r#"
var captures = [];
async function* stream(source) {
  for await (let value of source) {
    captures.push(function () { return value; });
    yield value;
    value = value + 10;
    yield captures[captures.length - 1]();
  }
  print("captures:" + captures[0]() + ":" + captures[1]());
}
var iterator = stream([1, 2]);
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(report);
void 0;
"#;
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("captured for-await head must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    let expected = [
        "1:false",
        "11:false",
        "2:false",
        "12:false",
        "captures:11:12",
        "undefined:true",
    ]
    .into_iter()
    .map(|line| HostOutputEvent::PrintLine(line.to_string()))
    .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected);
}
'''
test_path.write_text(test_text.replace(test_marker, test_addition, 1))

replace_once(
    "README.md",
    '''Async-generator ordinary property assignments across `yield` and `yield*` now
use the shared suspended Reference path. The base and raw key survive suspension;
normal resumption performs key conversion and the strictness-aware write, while
abrupt resumption bypasses it. This is a focused backend capability, not a claim
of complete generator or Test262 conformance. See
[the suspended Reference follow-up](docs/rust-rewrite/aot-suspended-references.md).
''',
    '''Async-generator ordinary property assignments across `yield` and `yield*` now
use the shared suspended Reference path. The base and raw key survive suspension;
normal resumption performs key conversion and the strictness-aware write, while
abrupt resumption bypasses it. This is a focused backend capability, not a claim
of complete generator or Test262 conformance. See
[the suspended Reference follow-up](docs/rust-rewrite/aot-suspended-references.md).

Captured `let`/`const` heads in async-generator `for await...of` loops retain the
exact per-iteration Environment Record across body `yield` suspension. Re-entry
reattaches the activation-owned record rather than allocating a second closure
cell, and completion restores the parent before iterator-close handling. See
[the for-await environment follow-up](docs/rust-rewrite/aot-for-await-iteration-environments.md).
''',
)

Path("docs/rust-rewrite/aot-for-await-iteration-environments.md").write_text(
    '''# Captured for-await iteration environments

This follow-up closes the captured-head environment gap for async-generator
`for await...of` bodies that suspend with `yield`.

A captured `let` or `const` loop head requires a fresh declarative Environment
Record for every iteration. Before this change, the Wasm AOT dispatcher refused
that shape whenever the body could suspend because the environment was created
and destroyed inside one Wasm invocation. Splitting the body at `yield` would
otherwise allocate a second record on resume and disconnect closures from the
cell created before suspension.

The for-await emitter now applies the same ownership rule as resumable classic
loops. The invocation resuming from `await next()` first observes `done`; only an
active iteration allocates a fresh record. It publishes the exact current
environment pointer into the owning activation. A body-resume invocation starts
with that pointer already restored by function entry. Both runtime paths then
attach one compiler binding scope, so outer activation-owned slots acquire the
correct parent hop and the loop head uses the same cell captured before `yield`.

Suspension returns without cleanup. Normal and abrupt iteration completion
converge on one inner cleanup block, restore the parent pointer in the activation,
and only then continue through the existing completion and IteratorClose logic.
This keeps local `continue`, `break`, `return`, throws, and normal fallthrough from
double-unwinding or leaking the child environment into the next iteration.

The compiler/runtime split is explicit:
`emit_allocate_lexical_environment_record` creates a runtime record without
mutating compiler scope state, while `begin_existing_lexical_environment_scope`
attaches the compile-time binding view after the fresh and resumed paths converge.

## Verification

`cargo test --locked -p lila-engine --test aot_async_for_of -- --test-threads=1`
contains a captured-`let` Wasmtime regression. It proves that a closure observes a
mutation made after resumption and that the next iteration receives a distinct
cell. The normal AOT regression workflow also continues to run the complete
backend shards.

## Deliberate boundary

This batch does not remove the separate refusal for an async-generator for-await
body whose own top-level block Environment Record spans suspension, or for nested
`for await` / body-`await` state ownership. It does not change iterator
acquisition, AsyncFromSyncIterator semantics, IteratorClose precedence, or the
runtime dynamic-source policy.
'''
)
