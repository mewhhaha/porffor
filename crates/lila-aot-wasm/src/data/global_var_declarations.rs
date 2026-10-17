//! Ordered declaration names in the actual P's immutable metadata image.
use super::*;
use lila_ir::PreparedScriptUnit;

/// Only collection of a complete declaration plan can mint these word rows.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GlobalVarDeclarationNames(ProgramMetadataSpan);
impl GlobalVarDeclarationNames {
    pub(crate) const fn span(self) -> ProgramMetadataSpan {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GlobalVarDeclarationTables {
    pub(crate) lexical_checks: GlobalVarDeclarationNames,
    pub(crate) variable_bindings: GlobalVarDeclarationNames,
}

impl StringPool {
    pub(super) fn collect_global_var_declarations(
        &mut self,
        unit: &PreparedScriptUnit,
    ) -> Result<(), EmitError> {
        if !unit.has_global_variable_environment() {
            return Ok(());
        }
        if self.global_var_declarations.contains_key(&unit.id) {
            return Err(EmitError::unsupported(
                "duplicate global declaration plan owner",
            ));
        }
        let names = &unit.declarations.var_names;
        for name in names {
            self.intern_string(name);
        }
        let lexical_checks = self.append_global_var_names(names.iter().map(String::as_str))?;
        let function_names = unit
            .declarations
            .functions_in_reverse_order
            .iter()
            .map(|declaration| declaration.name.as_str())
            .collect::<BTreeSet<_>>();
        let variables = names
            .iter()
            .filter(|name| !function_names.contains(name.as_str()))
            .map(String::as_str)
            .collect::<Vec<_>>();
        let variable_bindings = if variables.len() == names.len() {
            lexical_checks
        } else {
            self.append_global_var_names(variables.into_iter())?
        };
        self.global_var_declarations.insert(
            unit.id,
            GlobalVarDeclarationTables {
                lexical_checks,
                variable_bindings,
            },
        );
        Ok(())
    }

    fn append_global_var_names<'a>(
        &mut self,
        names: impl Iterator<Item = &'a str>,
    ) -> Result<GlobalVarDeclarationNames, EmitError> {
        self.align_bytes(8);
        let address = u32::try_from(self.bytes.len())
            .ok()
            .and_then(|offset| STATIC_DATA_OFFSET.checked_add(offset))
            .ok_or_else(|| {
                EmitError::unsupported("global declaration table exceeds its P image")
            })?;
        let mut length = 0_u32;
        for name in names {
            length = length
                .checked_add(8)
                .ok_or_else(|| EmitError::unsupported("global declaration table is too large"))?;
            let operand = PooledLiteralOperand::for_pool(self, self.pooled_string_index(name)?)?;
            self.bytes.extend_from_slice(&operand.word().to_le_bytes());
        }
        ProgramMetadataSpan::for_pool(self, address, length).map(GlobalVarDeclarationNames)
    }

    pub(crate) fn global_var_declaration_tables(
        &self,
        unit: &PreparedScriptUnit,
    ) -> Result<GlobalVarDeclarationTables, EmitError> {
        self.global_var_declarations
            .get(&unit.id)
            .copied()
            .ok_or_else(|| {
                EmitError::unsupported("global declaration body has no collected name plan")
            })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/global_var_declaration_names.rs"]
mod tests;
