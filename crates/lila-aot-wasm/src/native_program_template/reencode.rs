use super::*;
use plan::Plan;
use std::borrow::Cow;
use wasm_encoder::{
    reencode::{self, Reencode},
    EntityType, GlobalType, HeapType, Instruction, RefType, ValType,
};
mod names;
struct Encoder<'a> {
    plan: &'a Plan,
}
impl Encoder<'_> {
    fn reference(index: u32) -> ValType {
        ValType::Ref(RefType {
            nullable: false,
            heap_type: HeapType::Concrete(index),
        })
    }
}
impl Reencode for Encoder<'_> {
    type Error = String;
    fn function_index(&mut self, index: u32) -> Result<u32, reencode::Error<String>> {
        self.plan.function(index).ok_or_else(|| {
            reencode::Error::UserError("removed function is still referenced".into())
        })
    }
    fn global_index(&mut self, index: u32) -> Result<u32, reencode::Error<String>> {
        Ok(if index < self.plan.imported_globals {
            index
        } else {
            index + self.plan.images.len() as u32
        })
    }
    fn data_count(&mut self, _: u32) -> Result<u32, reencode::Error<String>> {
        Ok(0)
    }
    fn parse_data_section(
        &mut self,
        _: &mut wasm_encoder::DataSection,
        _: wasmparser::DataSectionReader<'_>,
    ) -> Result<(), reencode::Error<String>> {
        Ok(())
    }
    fn parse_type_section(
        &mut self,
        types: &mut wasm_encoder::TypeSection,
        reader: wasmparser::TypeSectionReader<'_>,
    ) -> Result<(), reencode::Error<String>> {
        reencode::utils::parse_type_section(self, types, reader)?;
        types.ty().array(&wasm_encoder::StorageType::I8, false);
        for factory in &self.plan.factories {
            let data = Self::reference(self.plan.data_type());
            let params = match factory.construction {
                NativeProgramArrayConstruction::Whole => vec![data],
                NativeProgramArrayConstruction::Slice => vec![ValType::I32, ValType::I32, data],
            };
            types
                .ty()
                .function(params, [Self::reference(factory.target_array_type)]);
        }
        Ok(())
    }
    fn parse_import_section(
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
        for image in &self.plan.images {
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
        Ok(())
    }
    fn parse_function_body(
        &mut self,
        code: &mut wasm_encoder::CodeSection,
        body: wasmparser::FunctionBody<'_>,
    ) -> Result<(), reencode::Error<String>> {
        let mut function = self.new_function_with_parsed_locals(&body)?;
        let mut reader = body.get_operators_reader()?;
        while !reader.eof() {
            let offset = reader.original_position();
            let op = reader.read()?;
            if self.plan.suppressed.contains(&offset) {
                continue;
            }
            if let Some(site) = self.plan.sites.get(&offset) {
                // Slice offset/length are already on the operand stack. Whole
                // construction derives the exact length from this data image.
                function.instruction(&Instruction::GlobalGet(
                    self.plan.imported_globals + site.image,
                ));
                function.instruction(&Instruction::Call(
                    self.plan.retained_functions() + site.factory,
                ));
            } else {
                function.instruction(&self.instruction(op)?);
            }
        }
        code.function(&function);
        Ok(())
    }
    fn parse_element(
        &mut self,
        section: &mut wasm_encoder::ElementSection,
        element: wasmparser::Element<'_>,
    ) -> Result<(), reencode::Error<String>> {
        if matches!(&element.kind, wasmparser::ElementKind::Declared) {
            if let wasmparser::ElementItems::Functions(reader) = &element.items {
                let mut indices = Vec::new();
                for index in (*reader).clone() {
                    if let Some(index) = self.plan.function(index?) {
                        indices.push(index)
                    }
                }
                section.declared(wasm_encoder::Elements::Functions(Cow::Owned(indices)));
                return Ok(());
            }
        }
        reencode::utils::parse_element(self, section, element)
    }
    fn parse_custom_name_subsection(
        &mut self,
        names: &mut wasm_encoder::NameSection,
        name: wasmparser::Name<'_>,
    ) -> Result<(), reencode::Error<String>> {
        names::subsection(self, names, name)
    }
}
pub(super) fn encode(bytes: &[u8], plan: &Plan) -> Result<Vec<u8>, NativeProgramTemplateError> {
    let mut encoder = Encoder { plan };
    let mut module = wasm_encoder::Module::new();
    encoder
        .parse_core_module(&mut module, wasmparser::Parser::new(0), bytes)
        .map_err(invalid)?;
    Ok(module.finish())
}
