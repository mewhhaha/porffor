use super::*;
impl Plan {
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
    pub fn source_data_type(&self) -> u32 {
        self.contexts
            .as_ref()
            .map_or(self.data_type(), |contexts| contexts.schema.payload_type)
    }
    pub fn initializer_type(&self) -> u32 {
        self.types + 1 + self.factories.len() as u32
    }
    pub fn literal_helper_type(&self) -> u32 {
        self.initializer_type() + 1
    }
    pub fn literal_helper(&self) -> u32 {
        self.retained_functions() + self.factories.len() as u32 + self.defined_functions
    }
    pub(super) fn image(&mut self, bytes: Vec<u8>) -> u32 {
        let index = self.images.len() as u32;
        self.images.push(NativeProgramDataImage {
            name: format!("image.{index}"),
            bytes: bytes.into(),
        });
        index
    }
    pub(super) fn segment(&mut self, segment: u32) -> Result<u32, NativeProgramTemplateError> {
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
    pub(super) fn factory(
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
