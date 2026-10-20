use super::*;
impl Encoder<'_> {
    pub(super) fn type_declarations(
        &mut self,
        types: &mut wasm_encoder::TypeSection,
        reader: wasmparser::TypeSectionReader<'_>,
    ) -> Result<(), reencode::Error<String>> {
        reencode::utils::parse_type_section(self, types, reader)?;
        types.ty().array(&wasm_encoder::StorageType::I8, false);
        for factory in &self.plan.factories {
            let data = Self::reference(self.plan.source_data_type());
            let params = match factory.construction {
                NativeProgramArrayConstruction::Whole if self.plan.contexts.is_some() => {
                    vec![ValType::I32, data]
                }
                NativeProgramArrayConstruction::Slice if self.plan.contexts.is_some() => {
                    vec![ValType::I32, ValType::I32, ValType::I32, data]
                }
                NativeProgramArrayConstruction::Whole => vec![data],
                NativeProgramArrayConstruction::Slice => vec![ValType::I32, ValType::I32, data],
            };
            types
                .ty()
                .function(params, [Self::reference(factory.target_array_type)]);
        }
        if let Some(contexts) = &self.plan.contexts {
            types.ty().function(
                [Self::reference(contexts.schema.payload_type)],
                [Self::reference(contexts.schema.program_type)],
            );
            types.ty().function(
                [ValType::I32, Self::reference(contexts.schema.payload_type)],
                [ValType::I32],
            );
        }
        Ok(())
    }
    pub(super) fn function_declarations(
        &mut self,
        functions: &mut wasm_encoder::FunctionSection,
        reader: wasmparser::FunctionSectionReader<'_>,
    ) -> Result<(), reencode::Error<String>> {
        for (body, ty) in reader.into_iter().enumerate() {
            let ty = ty?;
            let ty = if self
                .plan
                .contexts
                .as_ref()
                .is_some_and(|contexts| contexts.bootstrap_body == body as u32)
            {
                self.plan.initializer_type()
            } else {
                self.type_index(ty)?
            };
            functions.function(ty);
        }
        if self.plan.contexts.is_some() {
            functions.function(self.plan.literal_helper_type());
        }
        Ok(())
    }
    pub(super) fn import_declarations(
        &mut self,
        imports: &mut wasm_encoder::ImportSection,
        reader: wasmparser::ImportSectionReader<'_>,
    ) -> Result<(), reencode::Error<String>> {
        let mut function = 0;
        for import in reader.into_imports() {
            let import = import?;
            if matches!(
                import.ty,
                wasmparser::TypeRef::Func(_) | wasmparser::TypeRef::FuncExact(_)
            ) {
                let retain = self.plan.function_map.contains_key(&function);
                function += 1;
                if !retain {
                    continue;
                }
            }
            imports.import(import.module, import.name, self.entity_type(import.ty)?);
        }
        for (index, factory) in self.plan.factories.iter().enumerate() {
            imports.import(
                NATIVE_PROGRAM_DATA_MODULE,
                &factory.name,
                EntityType::Function(self.plan.factory_types[index]),
            );
        }
        for image in self
            .plan
            .images
            .iter()
            .filter(|_| self.plan.contexts.is_none())
        {
            imports.import(
                NATIVE_PROGRAM_DATA_MODULE,
                &image.name,
                EntityType::Global(GlobalType {
                    val_type: Self::reference(self.plan.data_type()),
                    mutable: false,
                    shared: false,
                }),
            );
        }
        for operand in self
            .plan
            .literal_operands
            .iter()
            .filter(|_| self.plan.contexts.is_none())
        {
            imports.import(
                NATIVE_PROGRAM_DATA_MODULE,
                &operand.name,
                EntityType::Global(GlobalType {
                    val_type: ValType::I32,
                    mutable: false,
                    shared: false,
                }),
            );
        }
        Ok(())
    }
}
