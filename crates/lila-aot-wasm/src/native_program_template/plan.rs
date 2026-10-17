use super::*;
use std::collections::{BTreeMap, BTreeSet};
use wasmparser::{CompositeInnerType, DataKind, Operator, Parser, Payload, StorageType, TypeRef};
mod sites;
pub(super) use sites::Site;

pub(super) struct Plan {
    pub imported_functions: u32,
    pub imported_globals: u32,
    pub types: u32,
    pub function_map: BTreeMap<u32, u32>,
    pub images: Vec<NativeProgramDataImage>,
    pub factories: Vec<NativeProgramArrayFactory>,
    pub factory_types: Vec<u32>,
    pub sites: BTreeMap<usize, Site>,
    pub suppressed: BTreeSet<usize>,
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
            factory_types: Vec::new(),
            sites: BTreeMap::new(),
            suppressed: BTreeSet::new(),
            arrays: BTreeMap::new(),
            segments: Vec::new(),
            segment_images: BTreeMap::new(),
        };
        let mut bodies = Vec::new();
        let mut referenced = super::references::References::default();
        let mut has_types = false;
        let mut has_imports = false;
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
                Payload::TableSection(reader) => referenced.tables(reader)?,
                Payload::GlobalSection(reader) => referenced.globals(reader)?,
                Payload::ExportSection(reader) => referenced.exports(reader)?,
                Payload::ElementSection(reader) => referenced.elements(reader)?,
                Payload::CodeSectionEntry(body) => bodies.push(body),
                Payload::StartSection { .. } => {
                    return Err(invalid("native P may not have a start"))
                }
                _ => {}
            }
        }
        // Native P normally has both sections. Other valid modules retain the
        // self-contained path rather than synthesizing a different section order.
        if !has_types || !has_imports {
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
        for body in &bodies {
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
        }
        let referenced = referenced.into_inner();
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
    pub fn retained_functions(&self) -> u32 {
        self.function_map.len() as u32
    }
    pub fn function(&self, old: u32) -> Option<u32> {
        if old < self.imported_functions {
            self.function_map.get(&old).copied()
        } else {
            Some(
                old - self.imported_functions
                    + self.retained_functions()
                    + self.factories.len() as u32,
            )
        }
    }
    pub fn data_type(&self) -> u32 {
        self.types
    }
    fn image(&mut self, bytes: Vec<u8>) -> u32 {
        let index = self.images.len() as u32;
        self.images.push(NativeProgramDataImage {
            name: format!("image.{index}"),
            bytes: bytes.into(),
        });
        index
    }
    fn segment(&mut self, segment: u32) -> Result<u32, NativeProgramTemplateError> {
        if let Some(image) = self.segment_images.get(&segment) {
            return Ok(*image);
        }
        let bytes = self
            .segments
            .get(segment as usize)
            .ok_or_else(|| invalid("unknown data segment"))?
            .clone();
        let image = self.image(bytes);
        self.segment_images.insert(segment, image);
        Ok(image)
    }
    fn factory(
        &mut self,
        target: u32,
        width: NativeProgramArrayWidth,
        construction: NativeProgramArrayConstruction,
    ) -> u32 {
        if let Some(index) = self.factories.iter().position(|f| {
            f.target_array_type == target && f.width == width && f.construction == construction
        }) {
            return index as u32;
        }
        let index = self.factories.len() as u32;
        self.factories.push(NativeProgramArrayFactory {
            name: format!("allocate.{index}"),
            width,
            construction,
            target_array_type: target,
        });
        index
    }
}
