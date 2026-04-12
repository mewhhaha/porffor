use crate::{
    Context, JsResult, JsValue,
    error::JsNativeError,
    object::{internal_methods::InternalMethodPropertyContext, shape::slot::SlotAttributes},
    property::PropertyKey,
    vm::opcode::{Operation, VaryingOperand},
};
use boa_ast::scope::BindingLocatorScope;

fn binding_not_found_error(context: &Context, locator: &boa_ast::scope::BindingLocator) -> JsNativeError {
    let name = locator.name().to_std_string_escaped();
    if let BindingLocatorScope::Stack(index) = locator.scope()
        && matches!(context.environment_expect(index), crate::environments::Environment::Object(_))
        && !context.vm.frame().code_block.strict()
    {
        return JsNativeError::reference().with_message(format!("{name} is not defined"));
    }

    JsNativeError::reference().with_message(format!("{name} is not defined"))
}

fn binding_value_or_undefined_for_object_env(
    context: &mut Context,
    locator: &boa_ast::scope::BindingLocator,
) -> JsResult<Option<JsValue>> {
    match context.get_binding(locator)? {
        Some(value) => Ok(Some(value)),
        None => {
            if let BindingLocatorScope::Stack(index) = locator.scope()
                && matches!(context.environment_expect(index), crate::environments::Environment::Object(_))
                && !context.vm.frame().code_block.strict()
            {
                Ok(Some(JsValue::undefined()))
            } else {
                Ok(None)
            }
        }
    }
}

/// `GetName` implements the Opcode Operation for `Opcode::GetName`
///
/// Operation:
///  - Find a binding on the environment chain and push its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetName;

impl GetName {
    #[inline(always)]
    pub(crate) fn operation(
        (value, index): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let mut binding_locator =
            context.vm.frame().code_block.bindings[usize::from(index)].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        let result = binding_value_or_undefined_for_object_env(context, &binding_locator)?
            .ok_or_else(|| binding_not_found_error(context, &binding_locator))?;
        context.vm.set_register(value.into(), result);
        Ok(())
    }
}

impl Operation for GetName {
    const NAME: &'static str = "GetName";
    const INSTRUCTION: &'static str = "INST - GetName";
    const COST: u8 = 4;
}

/// `GetNameGlobal` implements the Opcode Operation for `Opcode::GetNameGlobal`
///
/// Operation:
///  - Find a binding in the global object and push its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameGlobal;

impl GetNameGlobal {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, index, ic_index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let mut binding_locator =
            context.vm.frame().code_block.bindings[usize::from(index)].clone();
        context.find_runtime_binding(&mut binding_locator)?;

        if binding_locator.is_global() {
            let object = context.global_object();

            let ic = &context.vm.frame().code_block().ic[usize::from(ic_index)];

            let object_borrowed = object.borrow();
            if let Some((shape, slot)) = ic.match_or_reset(object_borrowed.shape()) {
                let mut result = if slot.attributes.contains(SlotAttributes::PROTOTYPE) {
                    let prototype = shape.prototype().expect("prototype should have value");
                    let prototype = prototype.borrow();
                    prototype.properties().storage[slot.index as usize].clone()
                } else {
                    object_borrowed.properties().storage[slot.index as usize].clone()
                };

                drop(object_borrowed);
                if slot.attributes.has_get() && result.is_object() {
                    result = result.as_object().expect("should contain getter").call(
                        &object.clone().into(),
                        &[],
                        context,
                    )?;
                }
                context.vm.set_register(dst.into(), result);
                return Ok(());
            }

            drop(object_borrowed);

            let key: PropertyKey = ic.name.clone().into();

            let context = &mut InternalMethodPropertyContext::new(context);
            let Some(result) = object.__try_get__(&key, object.clone().into(), context)? else {
                let name = binding_locator.name().to_std_string_escaped();
                return Err(JsNativeError::reference()
                    .with_message(format!("{name} is not defined"))
                    .into());
            };

            // Cache the property.
            let slot = *context.slot();
            if slot.is_cachable() {
                let ic = &context.vm.frame().code_block.ic[usize::from(ic_index)];
                let object_borrowed = object.borrow();
                let shape = object_borrowed.shape();
                ic.set(shape, slot);
            }

            context.vm.set_register(dst.into(), result);
            return Ok(());
        }

        let result = binding_value_or_undefined_for_object_env(context, &binding_locator)?
            .ok_or_else(|| binding_not_found_error(context, &binding_locator))?;

        context.vm.set_register(dst.into(), result);
        Ok(())
    }
}

impl Operation for GetNameGlobal {
    const NAME: &'static str = "GetNameGlobal";
    const INSTRUCTION: &'static str = "INST - GetNameGlobal";
    const COST: u8 = 4;
}

/// `GetLocator` implements the Opcode Operation for `Opcode::GetLocator`
///
/// Operation:
///  - Find a binding on the environment and set the `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetLocator;

impl GetLocator {
    #[inline(always)]
    pub(crate) fn operation(index: VaryingOperand, context: &mut Context) -> JsResult<()> {
        let mut binding_locator =
            context.vm.frame().code_block.bindings[usize::from(index)].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        if context.vm.frame().code_block.strict()
            && matches!(binding_locator.scope(), BindingLocatorScope::GlobalObject)
            && !context.is_initialized_binding(&binding_locator)?
        {
            binding_locator.set_binding_index(Context::UNRESOLVABLE_GLOBAL_REFERENCE_MARKER);
        }

        context.vm.frame_mut().binding_stack.push(binding_locator);

        Ok(())
    }
}

impl Operation for GetLocator {
    const NAME: &'static str = "GetLocator";
    const INSTRUCTION: &'static str = "INST - GetLocator";
    const COST: u8 = 4;
}

/// `GetNameAndLocator` implements the Opcode Operation for `Opcode::GetNameAndLocator`
///
/// Operation:
///  - Find a binding on the environment chain and push its value to the stack, setting the
///    `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameAndLocator;

impl GetNameAndLocator {
    #[inline(always)]
    pub(crate) fn operation(
        (value, index): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let mut binding_locator =
            context.vm.frame().code_block.bindings[usize::from(index)].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        let result = binding_value_or_undefined_for_object_env(context, &binding_locator)?
            .ok_or_else(|| binding_not_found_error(context, &binding_locator))?;

        context.vm.frame_mut().binding_stack.push(binding_locator);
        context.vm.set_register(value.into(), result);
        Ok(())
    }
}

impl Operation for GetNameAndLocator {
    const NAME: &'static str = "GetNameAndLocator";
    const INSTRUCTION: &'static str = "INST - GetNameAndLocator";
    const COST: u8 = 4;
}

/// `GetNameOrUndefined` implements the Opcode Operation for `Opcode::GetNameOrUndefined`
///
/// Operation:
///  - Find a binding on the environment chain and push its value. If the binding does not exist push undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetNameOrUndefined;

impl GetNameOrUndefined {
    #[inline(always)]
    pub(crate) fn operation(
        (value, index): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let mut binding_locator =
            context.vm.frame().code_block.bindings[usize::from(index)].clone();

        let is_global = binding_locator.is_global();

        context.find_runtime_binding(&mut binding_locator)?;

        let result = if let Some(value) = context.get_binding(&binding_locator)? {
            value
        } else if is_global {
            JsValue::undefined()
        } else {
            let name = binding_locator.name().to_std_string_escaped();
            return Err(JsNativeError::reference()
                .with_message(format!("{name} is not defined"))
                .into());
        };

        context.vm.set_register(value.into(), result);
        Ok(())
    }
}

impl Operation for GetNameOrUndefined {
    const NAME: &'static str = "GetNameOrUndefined";
    const INSTRUCTION: &'static str = "INST - GetNameOrUndefined";
    const COST: u8 = 4;
}
