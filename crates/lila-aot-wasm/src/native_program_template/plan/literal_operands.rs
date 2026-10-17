use super::*;

impl Plan {
    pub(super) fn collect_literal_operand_sites(
        &mut self,
        body_index: usize,
        operators: &[(usize, Operator<'_>)],
        declared: &mut std::iter::Peekable<
            std::slice::Iter<'_, crate::program_literal_operands::DeclaredSite>,
        >,
    ) -> Result<(), NativeProgramTemplateError> {
        while declared
            .peek()
            .is_some_and(|site| site.body as usize == body_index)
        {
            let site = declared.next().unwrap();
            let Some((offset, Operator::I32Const { value })) =
                operators.get(site.operator as usize)
            else {
                return Err(invalid("literal operand does not name an i32.const"));
            };
            if *value != site.expected {
                return Err(invalid(
                    "literal operand does not match its checked immediate",
                ));
            }
            if self.suppressed.contains(offset) || self.sites.contains_key(offset) {
                return Err(invalid(
                    "literal operand overlaps a packed array data operand",
                ));
            }
            let index = u32::try_from(self.literal_operands.len()).map_err(invalid)?;
            let role = match site.kind {
                crate::program_literal_operands::Kind::Count => "count",
                crate::program_literal_operands::Kind::Index => "index",
            };
            self.literal_operands.push(NativeProgramLiteralOperand {
                name: format!("literal.{role}.{index}"),
                value: site.expected,
            });
            self.literal_sites.insert(*offset, index);
        }
        Ok(())
    }
}
