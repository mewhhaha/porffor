//! Semantic call arguments. Count is always derived from the rooted vector.

use super::*;
use crate::gc_types::{
    AsyncActivation, AsyncGeneratorActivation, DirectEvalExecutionContext, Environment,
    FunctionObject, GcCallResult, GcLocal, GcNullability, GeneratorActivation, Nullable,
    PrivateEnvironment, RealmRecord, RuntimeSchema, ValueArray, ValueLocals,
};

pub(crate) struct EntryArguments<'a>(&'a GcLocal<ValueArray>);

impl<'a> EntryArguments<'a> {
    pub(crate) fn new(vector: &'a GcLocal<ValueArray>) -> Self {
        Self(vector)
    }

    pub(super) fn emit(&self, schema: &RuntimeSchema, function: &mut Function) {
        schema
            .array_type::<ValueArray>()
            .length(self.0, schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        self.0.load(schema, function);
    }
}

enum NewTargetInput<'a> {
    Call,
    Construct(&'a ValueLocals),
}

pub(crate) struct OrdinaryBodyInputs<'a> {
    this_value: &'a ValueLocals,
    new_target: NewTargetInput<'a>,
    arguments: EntryArguments<'a>,
    caller_realm: &'a GcLocal<RealmRecord>,
}

impl<'a> OrdinaryBodyInputs<'a> {
    pub(crate) fn call(
        this_value: &'a ValueLocals,
        arguments: EntryArguments<'a>,
        caller_realm: &'a GcLocal<RealmRecord>,
    ) -> Self {
        Self {
            this_value,
            new_target: NewTargetInput::Call,
            arguments,
            caller_realm,
        }
    }

    pub(crate) fn construct(
        this_value: &'a ValueLocals,
        new_target: &'a ValueLocals,
        arguments: EntryArguments<'a>,
        caller_realm: &'a GcLocal<RealmRecord>,
    ) -> Self {
        Self {
            this_value,
            new_target: NewTargetInput::Construct(new_target),
            arguments,
            caller_realm,
        }
    }

    pub(super) fn emit(
        &self,
        callable: &GcLocal<FunctionObject>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        callable.load(schema, function);
        self.this_value.emit(function);
        match &self.new_target {
            NewTargetInput::Call => {
                function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::RefNull(wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: wasm_encoder::AbstractHeapType::Eq,
                }));
            }
            NewTargetInput::Construct(value) => value.emit(function),
        }
        self.arguments.emit(schema, function);
        self.caller_realm.load(schema, function);
    }
}

macro_rules! resumable_inputs {
    ($inputs:ident, $activation:ident) => {
        pub(crate) struct $inputs<'a> {
            this_value: &'a ValueLocals,
            activation: &'a GcLocal<$activation>,
            arguments: EntryArguments<'a>,
        }
        impl<'a> $inputs<'a> {
            pub(crate) fn new(
                this_value: &'a ValueLocals,
                activation: &'a GcLocal<$activation>,
                arguments: EntryArguments<'a>,
            ) -> Self {
                Self {
                    this_value,
                    activation,
                    arguments,
                }
            }

            pub(super) fn emit(
                &self,
                callable: &GcLocal<FunctionObject>,
                schema: &RuntimeSchema,
                function: &mut Function,
            ) {
                callable.load(schema, function);
                self.this_value.emit(function);
                self.activation.load(schema, function);
                self.arguments.emit(schema, function);
            }
        }
    };
}
resumable_inputs!(GeneratorBodyInputs, GeneratorActivation);
resumable_inputs!(AsyncBodyInputs, AsyncActivation);
resumable_inputs!(AsyncGeneratorBodyInputs, AsyncGeneratorActivation);

/// Script execution has environments, rather than a FunctionObject receiver.
pub(crate) struct PreparedScriptInputs<'a> {
    lexical_environment: &'a GcLocal<Environment, Nullable>,
    this_value: &'a ValueLocals,
    new_target: &'a ValueLocals,
    variable_environment: &'a GcLocal<Environment, Nullable>,
    private_environment: &'a GcLocal<PrivateEnvironment, Nullable>,
    direct_eval_context: &'a GcLocal<DirectEvalExecutionContext, Nullable>,
}

impl<'a> PreparedScriptInputs<'a> {
    pub(crate) fn new(
        lexical_environment: &'a GcLocal<Environment, Nullable>,
        this_value: &'a ValueLocals,
        new_target: &'a ValueLocals,
        variable_environment: &'a GcLocal<Environment, Nullable>,
        private_environment: &'a GcLocal<PrivateEnvironment, Nullable>,
        direct_eval_context: &'a GcLocal<DirectEvalExecutionContext, Nullable>,
    ) -> Self {
        Self {
            lexical_environment,
            this_value,
            new_target,
            variable_environment,
            private_environment,
            direct_eval_context,
        }
    }

    fn emit(&self, schema: &RuntimeSchema, function: &mut Function) {
        self.lexical_environment.load(schema, function);
        self.this_value.emit(function);
        self.new_target.emit(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::RefNull(
            schema
                .reference_type::<ValueArray>(GcNullability::Nullable)
                .heap_type,
        ));
        self.variable_environment.load(schema, function);
        self.private_environment.load(schema, function);
        self.direct_eval_context.load(schema, function);
    }
}

macro_rules! planned_callable_call {
    ($entry:ident, $inputs:ident) => {
        impl $entry<'_> {
            pub(crate) fn emit_call(
                &self,
                callable: &GcLocal<FunctionObject>,
                inputs: $inputs<'_>,
                schema: &RuntimeSchema,
                function: &mut Function,
            ) -> GcCallResult {
                inputs.emit(callable, schema, function);
                schema.call_entry(self.0, function)
            }
        }
    };
}
planned_callable_call!(OrdinaryBodyEntry, OrdinaryBodyInputs);
planned_callable_call!(GeneratorBodyEntry, GeneratorBodyInputs);
planned_callable_call!(AsyncBodyEntry, AsyncBodyInputs);
planned_callable_call!(AsyncGeneratorBodyEntry, AsyncGeneratorBodyInputs);

impl PreparedScriptEntry<'_> {
    pub(crate) fn emit_call(
        &self,
        inputs: PreparedScriptInputs<'_>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> GcCallResult {
        inputs.emit(schema, function);
        schema.call_entry(self.0, function)
    }
}
