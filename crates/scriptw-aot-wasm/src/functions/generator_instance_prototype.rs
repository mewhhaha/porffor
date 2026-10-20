use super::*;
use crate::gc_types::{CompletionLocals, ValueLocals};

/// EvaluateGeneratorBody and EvaluateAsyncGeneratorBody select distinct
/// defaults from the generator function's own Realm.
#[derive(Clone, Copy)]
pub(super) enum GeneratorInstanceFamily {
    Generator,
    AsyncGenerator,
}

impl GeneratorInstanceFamily {
    const fn default_prototype(self) -> OrdinaryDefaultPrototype {
        match self {
            Self::Generator => OrdinaryDefaultPrototype::Generator,
            Self::AsyncGenerator => OrdinaryDefaultPrototype::AsyncGenerator,
        }
    }
}

impl FunctionBuilder<'_> {
    /// Read the observable prototype only after parameter initialization.
    /// The result retains the whole Throw for the activation producer to
    /// propagate after restoring the caller's execution Realm.
    pub(super) fn emit_generator_instance_prototype(
        &mut self,
        family: GeneratorInstanceFamily,
        function_object: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_get_prototype_from_constructor(
            function_object,
            family.default_prototype(),
            result,
            function,
        )
    }
}
