use super::*;

#[derive(Default)]
pub(super) struct Signatures {
    types: Vec<Option<wasmparser::FuncType>>,
    functions: Vec<u32>,
    imported: u32,
    initializer: Option<u32>,
}
impl Signatures {
    pub(super) fn record_type(&mut self, ty: &wasmparser::SubType) {
        self.types.push(match &ty.composite_type.inner {
            CompositeInnerType::Func(ty) => Some(ty.clone()),
            _ => None,
        });
    }
    pub(super) fn import_function(&mut self, ty: u32) {
        self.functions.push(ty);
        self.imported += 1;
    }
    pub(super) fn define_function(&mut self, ty: u32) {
        self.functions.push(ty);
    }
    pub(super) fn record_export(
        &mut self,
        export: wasmparser::Export<'_>,
    ) -> Result<(), NativeProgramTemplateError> {
        if export.name == crate::runtime_function::PROGRAM_INITIALIZE_EXPORT
            && (export.kind != wasmparser::ExternalKind::Func
                || self.initializer.replace(export.index).is_some())
        {
            return Err(invalid(
                "native bootstrap export is duplicated or not a function",
            ));
        }
        Ok(())
    }
    fn function(&self, body: u32) -> Result<&wasmparser::FuncType, NativeProgramTemplateError> {
        self.functions
            .get((self.imported + body) as usize)
            .and_then(|ty| self.types.get(*ty as usize))
            .and_then(Option::as_ref)
            .ok_or_else(|| invalid("native program body lacks its actual function type"))
    }
    pub(super) fn initializer(
        &self,
        body: u32,
        schema: Schema,
    ) -> Result<u32, NativeProgramTemplateError> {
        let index = self.imported + body;
        let ty = self.function(body)?;
        if self.initializer != Some(index)
            || !ty.params().is_empty()
            || !matches!(ty.results(), [ValType::Ref(reference)] if !reference.is_nullable() && is_program(*reference, schema))
        {
            return Err(invalid(
                "native bootstrap witness differs from the actual [] -> P export",
            ));
        }
        Ok(index)
    }
    pub(super) fn local(
        &self,
        body: u32,
        source: &wasmparser::FunctionBody<'_>,
        local: u32,
        schema: Schema,
    ) -> Result<(), NativeProgramTemplateError> {
        let ty = self.function(body)?;
        let mut cursor = ty.params().len() as u32;
        // The producer binds an initialized owner local, never a caller parameter.
        if local < cursor {
            return Err(invalid("native program owner must be a bound local"));
        }
        for row in source.get_locals_reader().map_err(invalid)? {
            let (count, ty) = row.map_err(invalid)?;
            let end = cursor
                .checked_add(count)
                .ok_or_else(|| invalid("native local count overflow"))?;
            if local < end {
                return match ty {
                    ValType::Ref(reference) if is_program(reference, schema) => Ok(()),
                    _ => Err(invalid("native owner local differs from the actual P type")),
                };
            }
            cursor = end;
        }
        Err(invalid("native program owner local is out of range"))
    }
}
fn is_program(reference: wasmparser::RefType, schema: Schema) -> bool {
    matches!(reference.heap_type(), wasmparser::HeapType::Concrete(index) if index.as_module_index() == Some(schema.program_type))
}
