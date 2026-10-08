//! The single materialization of an actual callable's declared entry roles.

use super::*;
use crate::gc_types::{
    AsyncActivation, AsyncActivationSchema, AsyncGeneratorActivation,
    AsyncGeneratorActivationSchema, DirectEvalExecutionContext, Environment, FunctionContext,
    FunctionContextSchema, FunctionObject, FunctionObjectSchema, GcLocal, GeneratorActivation,
    GeneratorActivationSchema, I32Local, I64Local, InvocationFrame, InvocationFrameSchema,
    NonNullable, Nullable, PrivateEnvironment, RealmRecord, RuntimeSchema, StoredValue, ValueArray,
    ValueLocals,
};

struct CallableLocals {
    function: GcLocal<FunctionObject>,
    context: GcLocal<FunctionContext>,
}

enum EntryRoleLocals {
    Ordinary {
        callable: CallableLocals,
        caller_realm: GcLocal<RealmRecord>,
    },
    Generator {
        callable: CallableLocals,
        activation: GcLocal<GeneratorActivation>,
        frame: GcLocal<InvocationFrame>,
        resume_point: I32Local,
    },
    Async {
        callable: CallableLocals,
        activation: GcLocal<AsyncActivation>,
        frame: GcLocal<InvocationFrame>,
        resume_point: I32Local,
    },
    AsyncGenerator {
        callable: CallableLocals,
        activation: GcLocal<AsyncGeneratorActivation>,
        frame: GcLocal<InvocationFrame>,
        resume_point: I32Local,
    },
    PreparedScript {
        variable_environment: GcLocal<Environment, Nullable>,
        direct_eval_context: Option<GcLocal<DirectEvalExecutionContext, Nullable>>,
    },
}

pub(crate) enum ResumableEntryLocals<'a> {
    Generator(&'a GcLocal<GeneratorActivation>),
    Async(&'a GcLocal<AsyncActivation>),
    AsyncGenerator(&'a GcLocal<AsyncGeneratorActivation>),
}

/// Owned rooted copies of the declared parameters. A body keeps this owner;
/// callers borrow semantic roles rather than rematerializing physical locals.
pub(crate) struct BodyEntryLocals {
    role: EntryRoleLocals,
    this_value: ValueLocals,
    new_target: ValueLocals,
    arguments: GcLocal<ValueArray>,
    argument_count: I64Local,
    lexical_environment: GcLocal<Environment, Nullable>,
    private_environment: GcLocal<PrivateEnvironment, Nullable>,
}

fn callable_locals(schema: &RuntimeSchema, function: &mut Function) -> CallableLocals {
    let callable = schema.parameter_gc::<FunctionObject, NonNullable>(0, function);
    let context = schema.reserve_gc_local(function).initialize(
        schema
            .struct_type::<FunctionObject>()
            .field(FunctionObjectSchema::CONTEXT)
            .read(&callable, schema, function)
            .reference(),
        function,
    );
    CallableLocals {
        function: callable,
        context,
    }
}

/// Null is an ABI encoding only for an empty vector. Check the physical count
/// once, then keep a non-null vector and a count derived from that vector.
fn normalize_arguments(
    schema: &RuntimeSchema,
    count_parameter: u32,
    vector_parameter: u32,
    function: &mut Function,
) -> (GcLocal<ValueArray>, I64Local) {
    let declared_count = schema.parameter_i64(count_parameter, function);
    let vector = schema.parameter_gc::<ValueArray, Nullable>(vector_parameter, function);
    vector.load(schema, function);
    function.instruction(&Instruction::RefIsNull);
    function.instruction(&Instruction::If(BlockType::Empty));
    declared_count.load(function);
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
    vector.replace(
        schema
            .array_type::<ValueArray>()
            .fixed([], function)
            .nullable(),
        function,
    );
    function.instruction(&Instruction::Else);
    schema
        .array_type::<ValueArray>()
        .length(&vector, schema, function);
    function.instruction(&Instruction::I64ExtendI32U);
    declared_count.load(function);
    function.instruction(&Instruction::I64Ne);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    let arguments = schema.reserve_gc_local(function).initialize(
        vector.load(schema, function).require_non_null(function),
        function,
    );
    vector.clear(function);
    let count = schema.reserve_i64_local(function);
    schema
        .array_type::<ValueArray>()
        .length(&arguments, schema, function);
    function.instruction(&Instruction::I64ExtendI32U);
    count.store(function);
    (arguments, count)
}

fn frame_new_target(
    frame: &GcLocal<InvocationFrame>,
    schema: &RuntimeSchema,
    function: &mut Function,
) -> ValueLocals {
    let stored = schema.reserve_gc_local(function).initialize(
        schema
            .struct_type::<InvocationFrame>()
            .field(InvocationFrameSchema::NEW_TARGET)
            .read(frame, schema, function)
            .reference(),
        function,
    );
    let value = schema.reserve_value_local(function);
    schema
        .struct_type::<StoredValue>()
        .read_into(&stored, &value, schema, function);
    stored.clear(function);
    value
}

impl PlannedBodyEntry<'_> {
    pub(crate) fn materialize(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> BodyEntryLocals {
        let this_value = schema.parameter_value(1, function);
        let role = match self {
            Self::Ordinary(_) => EntryRoleLocals::Ordinary {
                callable: callable_locals(schema, function),
                caller_realm: schema.parameter_gc::<RealmRecord, NonNullable>(9, function),
            },
            Self::Generator(_) => {
                let callable = callable_locals(schema, function);
                let activation =
                    schema.parameter_gc::<GeneratorActivation, NonNullable>(4, function);
                let activation_type = schema.struct_type::<GeneratorActivation>();
                let frame = schema.reserve_gc_local(function).initialize(
                    activation_type
                        .field(GeneratorActivationSchema::FRAME)
                        .read(&activation, schema, function)
                        .reference(),
                    function,
                );
                let resume_point = schema.reserve_i32_local(function);
                activation_type
                    .field(GeneratorActivationSchema::RESUME_POINT)
                    .read(&activation, schema, function)
                    .store(resume_point, function);
                EntryRoleLocals::Generator {
                    callable,
                    activation,
                    frame,
                    resume_point,
                }
            }
            Self::Async(_) => {
                let callable = callable_locals(schema, function);
                let activation = schema.parameter_gc::<AsyncActivation, NonNullable>(4, function);
                let activation_type = schema.struct_type::<AsyncActivation>();
                let frame = schema.reserve_gc_local(function).initialize(
                    activation_type
                        .field(AsyncActivationSchema::FRAME)
                        .read(&activation, schema, function)
                        .reference(),
                    function,
                );
                let resume_point = schema.reserve_i32_local(function);
                activation_type
                    .field(AsyncActivationSchema::RESUME_POINT)
                    .read(&activation, schema, function)
                    .store(resume_point, function);
                EntryRoleLocals::Async {
                    callable,
                    activation,
                    frame,
                    resume_point,
                }
            }
            Self::AsyncGenerator(_) => {
                let callable = callable_locals(schema, function);
                let activation =
                    schema.parameter_gc::<AsyncGeneratorActivation, NonNullable>(4, function);
                let activation_type = schema.struct_type::<AsyncGeneratorActivation>();
                let frame = schema.reserve_gc_local(function).initialize(
                    activation_type
                        .field(AsyncGeneratorActivationSchema::FRAME)
                        .read(&activation, schema, function)
                        .reference(),
                    function,
                );
                let resume_point = schema.reserve_i32_local(function);
                activation_type
                    .field(AsyncGeneratorActivationSchema::RESUME_POINT)
                    .read(&activation, schema, function)
                    .store(resume_point, function);
                EntryRoleLocals::AsyncGenerator {
                    callable,
                    activation,
                    frame,
                    resume_point,
                }
            }
            Self::PreparedScript(entry) => EntryRoleLocals::PreparedScript {
                variable_environment: schema.parameter_gc(9, function),
                // The shared ABI has a nullable context parameter. Only direct
                // eval owns that role; global Scripts deliberately pass null.
                // Retain the planned kind so absence never becomes a required
                // direct-eval context during body initialization.
                direct_eval_context: match entry.1 {
                    PreparedScriptKind::DirectEval(_) => Some(schema.parameter_gc(11, function)),
                    PreparedScriptKind::IndirectEval
                    | PreparedScriptKind::RealmScript
                    | PreparedScriptKind::ShadowRealmEvaluate => None,
                },
            },
        };
        let (arguments, argument_count) = match &role {
            EntryRoleLocals::Ordinary { .. } | EntryRoleLocals::PreparedScript { .. } => {
                normalize_arguments(schema, 7, 8, function)
            }
            EntryRoleLocals::Generator { .. }
            | EntryRoleLocals::Async { .. }
            | EntryRoleLocals::AsyncGenerator { .. } => normalize_arguments(schema, 5, 6, function),
        };
        let new_target = match &role {
            EntryRoleLocals::Ordinary { .. } | EntryRoleLocals::PreparedScript { .. } => {
                schema.parameter_value(4, function)
            }
            EntryRoleLocals::Generator { frame, .. }
            | EntryRoleLocals::Async { frame, .. }
            | EntryRoleLocals::AsyncGenerator { frame, .. } => {
                frame_new_target(frame, schema, function)
            }
        };
        let lexical_environment = match &role {
            EntryRoleLocals::Ordinary { callable, .. } => {
                schema.reserve_gc_local(function).initialize(
                    schema
                        .struct_type::<FunctionContext>()
                        .field(FunctionContextSchema::LEXICAL_ENVIRONMENT)
                        .read(&callable.context, schema, function)
                        .reference(),
                    function,
                )
            }
            EntryRoleLocals::Generator { frame, .. }
            | EntryRoleLocals::Async { frame, .. }
            | EntryRoleLocals::AsyncGenerator { frame, .. } => {
                schema.reserve_gc_local(function).initialize(
                    schema
                        .struct_type::<InvocationFrame>()
                        .field(InvocationFrameSchema::LEXICAL_ENVIRONMENT)
                        .read(frame, schema, function)
                        .reference(),
                    function,
                )
            }
            EntryRoleLocals::PreparedScript { .. } => schema.parameter_gc(0, function),
        };
        let private_environment = match &role {
            EntryRoleLocals::Ordinary { callable, .. }
            | EntryRoleLocals::Generator { callable, .. }
            | EntryRoleLocals::Async { callable, .. }
            | EntryRoleLocals::AsyncGenerator { callable, .. } => {
                schema.reserve_gc_local(function).initialize(
                    schema
                        .struct_type::<FunctionContext>()
                        .field(FunctionContextSchema::PRIVATE_ENVIRONMENT)
                        .read(&callable.context, schema, function)
                        .reference(),
                    function,
                )
            }
            EntryRoleLocals::PreparedScript { .. } => schema.parameter_gc(10, function),
        };
        BodyEntryLocals {
            role,
            this_value,
            new_target,
            arguments,
            argument_count,
            lexical_environment,
            private_environment,
        }
    }
}

impl BodyEntryLocals {
    pub(crate) fn caller_realm(&self) -> Option<&GcLocal<RealmRecord>> {
        match &self.role {
            EntryRoleLocals::Ordinary { caller_realm, .. } => Some(caller_realm),
            EntryRoleLocals::Generator { .. }
            | EntryRoleLocals::Async { .. }
            | EntryRoleLocals::AsyncGenerator { .. }
            | EntryRoleLocals::PreparedScript { .. } => None,
        }
    }
    pub(crate) fn this_value(&self) -> &ValueLocals {
        &self.this_value
    }
    pub(crate) fn new_target(&self) -> &ValueLocals {
        &self.new_target
    }
    pub(crate) fn arguments(&self) -> &GcLocal<ValueArray> {
        &self.arguments
    }
    pub(crate) fn argument_count(&self) -> I64Local {
        self.argument_count
    }
    pub(crate) fn lexical_environment(&self) -> &GcLocal<Environment, Nullable> {
        &self.lexical_environment
    }
    pub(crate) fn private_environment(&self) -> &GcLocal<PrivateEnvironment, Nullable> {
        &self.private_environment
    }
    fn callable(&self) -> Option<&CallableLocals> {
        match &self.role {
            EntryRoleLocals::Ordinary {
                callable: value, ..
            }
            | EntryRoleLocals::Generator {
                callable: value, ..
            }
            | EntryRoleLocals::Async {
                callable: value, ..
            }
            | EntryRoleLocals::AsyncGenerator {
                callable: value, ..
            } => Some(value),
            EntryRoleLocals::PreparedScript { .. } => None,
        }
    }
    pub(crate) fn function_object(&self) -> Option<&GcLocal<FunctionObject>> {
        self.callable().map(|callable| &callable.function)
    }
    pub(crate) fn function_context(&self) -> Option<&GcLocal<FunctionContext>> {
        self.callable().map(|callable| &callable.context)
    }
    pub(crate) fn resume_frame(&self) -> Option<&GcLocal<InvocationFrame>> {
        match &self.role {
            EntryRoleLocals::Generator { frame, .. }
            | EntryRoleLocals::Async { frame, .. }
            | EntryRoleLocals::AsyncGenerator { frame, .. } => Some(frame),
            EntryRoleLocals::Ordinary { .. } | EntryRoleLocals::PreparedScript { .. } => None,
        }
    }
    pub(crate) fn resume_point(&self) -> Option<I32Local> {
        match &self.role {
            EntryRoleLocals::Generator { resume_point, .. }
            | EntryRoleLocals::Async { resume_point, .. }
            | EntryRoleLocals::AsyncGenerator { resume_point, .. } => Some(*resume_point),
            EntryRoleLocals::Ordinary { .. } | EntryRoleLocals::PreparedScript { .. } => None,
        }
    }
    pub(crate) fn resume_activation(&self) -> Option<ResumableEntryLocals<'_>> {
        match &self.role {
            EntryRoleLocals::Generator { activation, .. } => {
                Some(ResumableEntryLocals::Generator(activation))
            }
            EntryRoleLocals::Async { activation, .. } => {
                Some(ResumableEntryLocals::Async(activation))
            }
            EntryRoleLocals::AsyncGenerator { activation, .. } => {
                Some(ResumableEntryLocals::AsyncGenerator(activation))
            }
            EntryRoleLocals::Ordinary { .. } | EntryRoleLocals::PreparedScript { .. } => None,
        }
    }
    pub(crate) fn variable_environment(&self) -> Option<&GcLocal<Environment, Nullable>> {
        match &self.role {
            EntryRoleLocals::PreparedScript {
                variable_environment,
                ..
            } => Some(variable_environment),
            EntryRoleLocals::Ordinary { .. }
            | EntryRoleLocals::Generator { .. }
            | EntryRoleLocals::Async { .. }
            | EntryRoleLocals::AsyncGenerator { .. } => None,
        }
    }
    pub(crate) fn direct_eval_context(
        &self,
    ) -> Option<&GcLocal<DirectEvalExecutionContext, Nullable>> {
        match &self.role {
            EntryRoleLocals::PreparedScript {
                direct_eval_context,
                ..
            } => direct_eval_context.as_ref(),
            EntryRoleLocals::Ordinary { .. }
            | EntryRoleLocals::Generator { .. }
            | EntryRoleLocals::Async { .. }
            | EntryRoleLocals::AsyncGenerator { .. } => None,
        }
    }
    pub(crate) fn clear(self, schema: &RuntimeSchema, function: &mut Function) {
        self.private_environment.clear(function);
        self.lexical_environment.clear(function);
        schema.release_i64_local(self.argument_count, function);
        self.arguments.clear(function);
        self.new_target.clear(function);
        self.this_value.clear(function);
        match self.role {
            EntryRoleLocals::Ordinary {
                callable,
                caller_realm,
            } => {
                caller_realm.clear(function);
                callable.clear(function);
            }
            EntryRoleLocals::Generator {
                callable,
                activation,
                frame,
                resume_point,
            } => {
                schema.release_i32_local(resume_point, function);
                frame.clear(function);
                activation.clear(function);
                callable.clear(function);
            }
            EntryRoleLocals::Async {
                callable,
                activation,
                frame,
                resume_point,
            } => {
                schema.release_i32_local(resume_point, function);
                frame.clear(function);
                activation.clear(function);
                callable.clear(function);
            }
            EntryRoleLocals::AsyncGenerator {
                callable,
                activation,
                frame,
                resume_point,
            } => {
                schema.release_i32_local(resume_point, function);
                frame.clear(function);
                activation.clear(function);
                callable.clear(function);
            }
            EntryRoleLocals::PreparedScript {
                variable_environment,
                direct_eval_context,
            } => {
                if let Some(context) = direct_eval_context {
                    context.clear(function);
                }
                variable_environment.clear(function);
            }
        }
    }
}

impl CallableLocals {
    fn clear(self, function: &mut Function) {
        self.context.clear(function);
        self.function.clear(function);
    }
}
