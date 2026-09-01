use super::*;

mod decimal_format;
mod ryu;

pub(super) use decimal_format::{NumberDecimalFormat, NumberExponentialFormat};

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_number_to_string_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Number formatting appears in nearly every builtin body and its inline
        // expansion is several KB; call the shared helper instead (except while
        // compiling the helper itself). The helper returns the standard four-i64
        // tuple with the string payload in the first slot.
        if self.outline_number_to_string {
            if let Some(helper) = self.number_to_string_helper_function_index() {
                function.instruction(&Instruction::LocalGet(payload_local));
                for _ in 0..6 {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::Call(helper));
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::Drop);
                return Ok(());
            }
        }
        self.emit_ryu_number_to_string_payload(payload_local, function)
    }
}
