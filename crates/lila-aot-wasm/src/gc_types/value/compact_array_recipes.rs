use super::*;
use crate::data::COMPACT_ARRAY_RECIPE_SEGMENT;
use crate::runtime_helpers::CompactArrayLiteralParameters;

impl RuntimeSchema {
    /// This Program-owned helper selects its own P module's passive recipe
    /// segment. No bytes are installed in shared linear memory. Embedded string
    /// operands separately select checked R/P-relative tables through that P.
    /// Internal bounds originate in a complete compiler recipe, never JS values.
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
