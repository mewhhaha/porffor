use super::*;
use crate::data::COMPACT_ARRAY_RECIPE_SEGMENT;
use crate::runtime_helpers::CompactArrayLiteralParameters;

impl RuntimeSchema {
    /// The internal helper parameters originate in a checked compiler recipe.
    /// Its private data segment cannot be selected by JavaScript operands.
    pub(crate) fn compact_array_recipe_data(
        &self,
        parameters: &CompactArrayLiteralParameters,
        function: &mut Function,
    ) -> GcStackReference<ByteArray> {
        parameters.offset.load(function);
        parameters.bytes.load(function);
        function.instruction(&Instruction::ArrayNewData {
            array_type_index: self.array_type::<ByteArray>().type_index().raw(),
            array_data_index: COMPACT_ARRAY_RECIPE_SEGMENT,
        });
        GcStackReference::new()
    }
}
