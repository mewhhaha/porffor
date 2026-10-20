//! The replacer call owns four distinct roles until its one call boundary.
use super::*;

pub(super) struct JsonStringifyReplacerFunctionLocals<'v>(&'v ValueLocals);
pub(super) struct JsonStringifyReplacerReceiverLocals<'v>(&'v ValueLocals);
pub(super) struct JsonStringifyReplacerPropertyKeyLocals<'v>(&'v ValueLocals);
pub(super) struct JsonStringifyReplacerValueLocals<'v>(&'v ValueLocals);

impl<'v> JsonStringifyReplacerFunctionLocals<'v> {
    pub(super) const fn new(value: &'v ValueLocals) -> Self {
        Self(value)
    }
}
impl<'v> JsonStringifyReplacerReceiverLocals<'v> {
    pub(super) const fn new(value: &'v ValueLocals) -> Self {
        Self(value)
    }
}
impl<'v> JsonStringifyReplacerPropertyKeyLocals<'v> {
    pub(super) const fn new(value: &'v ValueLocals) -> Self {
        Self(value)
    }
}
impl<'v> JsonStringifyReplacerValueLocals<'v> {
    pub(super) const fn new(value: &'v ValueLocals) -> Self {
        Self(value)
    }
}

#[must_use = "a replacer invocation must be consumed by its call"]
pub(super) struct JsonStringifyReplacerInvocationLocals<'v> {
    replacer: JsonStringifyReplacerFunctionLocals<'v>,
    receiver: JsonStringifyReplacerReceiverLocals<'v>,
    property_key: JsonStringifyReplacerPropertyKeyLocals<'v>,
    value: JsonStringifyReplacerValueLocals<'v>,
}
impl<'v> JsonStringifyReplacerInvocationLocals<'v> {
    pub(super) const fn new(
        replacer: JsonStringifyReplacerFunctionLocals<'v>,
        receiver: JsonStringifyReplacerReceiverLocals<'v>,
        property_key: JsonStringifyReplacerPropertyKeyLocals<'v>,
        value: JsonStringifyReplacerValueLocals<'v>,
    ) -> Self {
        Self {
            replacer,
            receiver,
            property_key,
            value,
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_json_apply_replacer_with_this(
        &mut self,
        invocation: JsonStringifyReplacerInvocationLocals<'_>,
        output: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let JsonStringifyReplacerInvocationLocals {
            replacer,
            receiver,
            property_key,
            value,
        } = invocation;
        self.emit_json_call(
            replacer.0,
            receiver.0,
            &[property_key.0, value.0],
            output,
            f,
        )
    }
}
