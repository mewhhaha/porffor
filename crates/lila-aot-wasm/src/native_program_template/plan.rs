use super::*;
use std::collections::{BTreeMap, BTreeSet};
use wasmparser::{CompositeInnerType, DataKind, Operator, Parser, Payload, StorageType, TypeRef};
mod indices;
mod literal_operands;
mod sites;
pub(super) use sites::Site;

pub(super) struct Plan {
    pub imported_functions: u32,
    pub imported_globals: u32,
    pub types: u32,
    pub function_map: BTreeMap<u32, u32>,
    pub images: Vec<NativeProgramDataImage>,
    pub factories: Vec<NativeProgramArrayFactory>,
    pub literal_operands: Vec<NativeProgramLiteralOperand>,
    pub literal_sites: BTreeMap<usize, u32>,
    pub factory_types: Vec<u32>,
    pub sites: BTreeMap<usize, Site>,
    pub suppressed: BTreeSet<usize>,
    pub contexts: Option<super::contexts::OwnedContexts>,
    pub defined_functions: u32,
    pub reusable: bool,
    arrays: BTreeMap<u32, NativeProgramArrayWidth>,
    segments: Vec<Vec<u8>>,
    segment_images: BTreeMap<u32, u32>,
}
impl Plan {
    pub fn read(bytes: &[u8]) -> Result<Option<Self>, NativeProgramTemplateError> {
        let mut result = Self {
            imported_functions: 0,
            imported_globals: 0,
            types: 0,
            function_map: BTreeMap::new(),
            images: Vec::new(),
            factories: Vec::new(),
            literal_operands: Vec::new(),
            literal_sites: BTreeMap::new(),
            factory_types: Vec::new(),
            sites: BTreeMap::new(),
            suppressed: BTreeSet::new(),
            contexts: None,
            defined_functions: 0,
            reusable: true,
            arrays: BTreeMap::new(),
            segments: Vec::new(),
            segment_images: BTreeMap::new(),
        };
        let mut bodies = Vec::new();
        let mut referenced = super::references::References::default();
        let mut has_types = false;
        let mut has_imports = false;
        let mut declared_operands = None;
        let mut declared_contexts = None;
        for payload in Parser::new(0).parse_all(bytes) {
            match payload.map_err(invalid)? {
                Payload::TypeSection(reader) => {
                    has_types = true;
                    for group in reader {
                        for ty in group.map_err(invalid)?.types() {
                            if let CompositeInnerType::Array(array) = &ty.composite_type.inner {
                                if !ty.composite_type.shared {
                                    let width = match array.0.element_type {
                                        StorageType::I8 => Some(NativeProgramArrayWidth::I8),
                                        StorageType::I16 => Some(NativeProgramArrayWidth::I16),
                                        _ => None,
                                    };
                                    if let Some(width) = width {
                                        result.arrays.insert(result.types, width);
                                    }
                                }
                            }
                            result.types = result
                                .types
                                .checked_add(1)
                                .ok_or_else(|| invalid("type count overflow"))?;
                        }
                    }
                }
                Payload::ImportSection(reader) => {
                    has_imports = true;
                    for import in reader.into_imports() {
                        let import = import.map_err(invalid)?;
                        if import.module == NATIVE_PROGRAM_DATA_MODULE {
                            return Err(invalid("native data namespace already exists"));
                        }
                        match import.ty {
                            TypeRef::Func(_) | TypeRef::FuncExact(_) => {
                                result.imported_functions += 1
                            }
                            TypeRef::Global(_) => result.imported_globals += 1,
                            _ => {}
                        }
                    }
                }
                Payload::DataSection(reader) => {
                    for data in reader {
                        let data = data.map_err(invalid)?;
                        if !matches!(data.kind, DataKind::Passive) {
                            return Ok(None);
                        }
                        result.segments.push(data.data.to_vec());
                    }
                }
                Payload::TableSection(reader) => {
                    result.reusable &= reader.count() == 0;
                    referenced.tables(reader)?;
                }
                Payload::MemorySection(reader) => result.reusable &= reader.count() == 0,
                Payload::GlobalSection(reader) => {
                    result.reusable &= reader.count() == 0;
                    referenced.globals(reader)?;
                }
                Payload::FunctionSection(reader) => result.defined_functions = reader.count(),
                Payload::ExportSection(reader) => referenced.exports(reader)?,
                Payload::ElementSection(reader) => {
                    for element in reader.clone() {
                        result.reusable &= matches!(
                            element.map_err(invalid)?.kind,
                            wasmparser::ElementKind::Declared
                        );
                    }
                    referenced.elements(reader)?;
                }
                Payload::CodeSectionEntry(body) => bodies.push(body),
                Payload::CustomSection(section)
                    if section.name() == crate::native_program_contexts::SECTION_NAME =>
                {
                    if declared_contexts.is_some() {
                        return Err(invalid("duplicate native program context section"));
                    }
                    declared_contexts = Some(
                        crate::native_program_contexts::read(section.data()).map_err(invalid)?,
                    );
                }
                Payload::CustomSection(section)
                    if section.name() == crate::program_literal_operands::SECTION_NAME =>
                {
                    if declared_operands.is_some() {
                        return Err(invalid("duplicate literal operand section"));
                    }
                    declared_operands = Some(
                        crate::program_literal_operands::read(section.data()).map_err(invalid)?,
                    );
                }
                Payload::StartSection { .. } => {
                    return Err(invalid("native P may not have a start"))
                }
                _ => {}
            }
        }
        // Native P normally has both sections. Other valid modules retain the
        // self-contained path rather than synthesizing a different section order.
        if !has_types || !has_imports {
            if declared_operands.is_some() {
                return Err(invalid("literal operands require a native P module"));
            }
            return Ok(None);
        }
        // Checked artifact producers are the only callers. Development builds
        // validate their source bytes here; every release build still validates
        // the final transformed module in Wasmtime before instantiation.
        #[cfg(debug_assertions)]
        {
            let features = wasmparser::WasmFeatures::default()
                | wasmparser::WasmFeatures::THREADS
                | wasmparser::WasmFeatures::MULTI_MEMORY
                | wasmparser::WasmFeatures::REFERENCE_TYPES
                | wasmparser::WasmFeatures::FUNCTION_REFERENCES
                | wasmparser::WasmFeatures::GC
                | wasmparser::WasmFeatures::EXCEPTIONS
                | wasmparser::WasmFeatures::TAIL_CALL;
            wasmparser::Validator::new_with_features(features)
                .validate_all(bytes)
                .map_err(invalid)?;
        }
        let declared_operands = declared_operands.unwrap_or_default();
        if declared_operands
            .last()
            .is_some_and(|site| site.body as usize >= bodies.len())
        {
            return Err(invalid("literal operand body is out of range"));
        }
        let mut declared = declared_operands.iter().peekable();
        for (body_index, body) in bodies.iter().enumerate() {
            let mut reader = body.get_operators_reader().map_err(invalid)?;
            let mut operators = Vec::new();
            while !reader.eof() {
                let offset = reader.original_position();
                operators.push((offset, reader.read().map_err(invalid)?));
            }
            for (_, op) in &operators {
                referenced.operator(op);
                if matches!(
                    op,
                    Operator::MemoryInit { .. }
                        | Operator::DataDrop { .. }
                        | Operator::ArrayInitData { .. }
                ) {
                    return Ok(None);
                }
                if let Operator::ArrayNewData {
                    array_type_index, ..
                } = op
                {
                    if !result.arrays.contains_key(array_type_index) {
                        return Ok(None);
                    }
                }
            }
            result.collect_sites(&operators)?;
            result.collect_literal_operand_sites(body_index, &operators, &mut declared)?;
        }
        let referenced = referenced.into_inner();
        if let Some(contexts) = declared_contexts {
            result.contexts = super::contexts::OwnedContexts::read(
                bytes,
                contexts,
                &bodies,
                &result.sites,
                &result.literal_sites,
            )?;
        }
        let mut next = 0;
        for original in 0..result.imported_functions {
            if referenced.contains(&original) {
                result.function_map.insert(original, next);
                next += 1;
            }
        }
        result.factory_types = (0..result.factories.len())
            .map(|n| result.types + 1 + n as u32)
            .collect();
        Ok(Some(result))
    }
}
