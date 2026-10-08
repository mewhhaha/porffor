//! The original Array/Object element algorithms have one physical lowering body.

use super::pattern_target::{
    owned_pattern_binding, retain_pattern_value, PatternContinuation, RetainedPatternTarget,
};
use super::*;

mod array;
mod object;

impl PatternContinuation {
    fn name(
        self,
        generator: &'static str,
        asynchronous: &'static str,
        mixed: &'static str,
    ) -> &'static str {
        match self {
            Self::Generator => generator,
            Self::Async => asynchronous,
            Self::AsyncGenerator => mixed,
        }
    }
}
impl ScriptLowerer<'_> {
    fn apply_resumable_pattern_default(
        &mut self,
        execution: PatternContinuation,
        value: &TypedExpr,
        source: &Expression,
        statements: &mut Vec<StatementIr>,
    ) -> Option<()> {
        match execution {
            PatternContinuation::Generator => {
                self.apply_generator_pattern_default(value, source, statements)
            }
            PatternContinuation::Async => {
                self.apply_async_pattern_default(value, source, statements)
            }
            PatternContinuation::AsyncGenerator => {
                self.apply_async_generator_pattern_default(value, source, statements)
            }
        }
    }
}
