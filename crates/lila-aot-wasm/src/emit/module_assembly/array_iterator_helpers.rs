//! Compile each registered array-pattern kernel once into R.

use super::*;

pub(super) fn compile(
    schema: &RuntimeSchema,
    strings: &StringPool,
    functions: &FunctionMetaRegistry,
    uses_heap: bool,
    bootstrap: &RuntimeBootstrapPlan,
    base: Option<crate::runtime_helpers::RuntimeHelperFunctionBase>,
) -> Result<BTreeMap<RuntimeHelperId, Function>, EmitError> {
    let mut bodies = BTreeMap::new();
    macro_rules! register {
        ($helper:ident, $compile:ident) => {{
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                schema,
                RuntimeHelperId::$helper,
                strings,
                functions,
                uses_heap,
                bootstrap.clone(),
                base,
            );
            bodies.insert(RuntimeHelperId::$helper, builder.$compile()?);
        }};
    }
    register!(
        ArrayIteratorStepValue,
        compile_array_iterator_step_value_helper
    );
    register!(ArrayIteratorElision, compile_array_iterator_elision_helper);
    register!(ArrayIteratorClose, compile_array_iterator_close_helper);
    Ok(bodies)
}
