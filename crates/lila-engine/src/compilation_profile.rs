//! Measurements consume the original compiler stages and product Wasm policy.
use super::*;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmSectionFootprint {
    pub name: String,
    pub payload_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WasmArtifactFootprint {
    pub module_bytes: u64,
    pub imported_functions: u64,
    pub defined_functions: u64,
    pub code_body_bytes: u64,
    pub largest_code_body_bytes: u64,
    pub identical_extra_bodies: u64,
    pub identical_extra_body_bytes: u64,
    pub data_payload_bytes: u64,
    pub custom_sections: Vec<WasmSectionFootprint>,
}

impl WasmArtifactFootprint {
    /// Re-admit retained evidence with the product runtime's actual capability
    /// policy before trusting its section inventory.
    pub fn from_module_bytes(bytes: &[u8]) -> Result<Self, EngineError> {
        validate(&shared_wasm_engine()?, bytes)?;
        Self::from_validated_bytes(bytes)
    }

    fn from_validated_bytes(bytes: &[u8]) -> Result<Self, EngineError> {
        let mut result = Self {
            module_bytes: bytes.len() as u64,
            ..Self::default()
        };
        let mut bodies: BTreeMap<&[u8], u64> = BTreeMap::new();
        let parse_error = |error: wasmparser::BinaryReaderError| {
            EngineError::new(format!("cannot inspect emitted Wasm: {error}"))
        };
        for payload in WasmParser::new(0).parse_all(bytes) {
            match payload.map_err(parse_error)? {
                WasmPayload::ImportSection(reader) => {
                    for import in reader.into_imports() {
                        if matches!(
                            import.map_err(parse_error)?.ty,
                            wasmparser::TypeRef::Func(_) | wasmparser::TypeRef::FuncExact(_)
                        ) {
                            result.imported_functions += 1;
                        }
                    }
                }
                WasmPayload::CodeSectionEntry(body) => {
                    let encoded = bytes.get(body.range()).ok_or_else(|| {
                        EngineError::new("emitted code body is outside its module")
                    })?;
                    let length = encoded.len() as u64;
                    result.defined_functions += 1;
                    result.code_body_bytes += length;
                    result.largest_code_body_bytes = result.largest_code_body_bytes.max(length);
                    *bodies.entry(encoded).or_default() += 1;
                }
                WasmPayload::DataSection(reader) => {
                    for segment in reader {
                        result.data_payload_bytes +=
                            segment.map_err(parse_error)?.data.len() as u64;
                    }
                }
                WasmPayload::CustomSection(section) => {
                    result.custom_sections.push(WasmSectionFootprint {
                        name: section.name().to_owned(),
                        payload_bytes: section.data().len() as u64,
                    })
                }
                _ => {}
            }
        }
        for (body, count) in bodies {
            result.identical_extra_bodies += count - 1;
            result.identical_extra_body_bytes += (count - 1) * body.len() as u64;
        }
        Ok(result)
    }
}

#[derive(Debug)]
pub struct ScriptCompilationProfile {
    /// Parsing, early syntax errors and the actual dependency/prelude admission.
    pub preparation: Duration,
    /// Original spec IR lowering and its coded early/resolution errors.
    pub lowering: Duration,
    /// Original Wasm lowering/emission and artifact metadata publication.
    pub emission: Duration,
    /// Wasmtime validation with the same capability policy as product execution.
    pub validation: Duration,
    pub artifact: Artifact,
    pub footprint: WasmArtifactFootprint,
}

fn validate(engine: &WasmtimeEngine, bytes: &[u8]) -> Result<(), EngineError> {
    WasmtimeModule::validate(engine, bytes)
        .map_err(|error| EngineError::new(format!("emitted Wasm validation failed: {error:#}")))
}

impl Engine {
    /// Profiles a real uncached Script compilation, without executing it.
    /// Worker startup, runtime engine setup and footprint analysis are outside
    /// the four measured spans. This does not use the program artifact cache.
    pub fn profile_script_compilation(
        &self,
        source: &str,
        options: CompileOptions,
    ) -> Result<ScriptCompilationProfile, EngineError> {
        run_on_sized_stack(move || {
            let runtime = shared_wasm_engine()?;
            let started = Instant::now();
            let prepared = self.prepare_compilation(source, ParseGoal::Script, &options)?;
            let preparation = started.elapsed();
            let started = Instant::now();
            let unit = self.compile_prepared_on_current_thread(prepared)?;
            let lowering = started.elapsed();
            let started = Instant::now();
            let artifact = self.emit_wasm_on_current_thread(&unit)?;
            let emission = started.elapsed();
            let started = Instant::now();
            validate(&runtime, &artifact.bytes)?;
            let validation = started.elapsed();
            let footprint = WasmArtifactFootprint::from_validated_bytes(&artifact.bytes)?;
            Ok(ScriptCompilationProfile {
                preparation,
                lowering,
                emission,
                validation,
                artifact,
                footprint,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_uses_the_product_artifact_and_admitted_byte_inventory() {
        configure_compilation_jobs(1).unwrap();
        let engine = Engine::new(RealmBuilder::new().build());
        let source = "function twice(value) { return value + value; } twice(7);";
        let profile = engine
            .profile_script_compilation(source, CompileOptions::default())
            .unwrap();
        let unit = engine
            .compile_script(source, CompileOptions::default())
            .unwrap();
        let emitted = engine.emit_wasm(&unit).unwrap();
        assert_eq!(profile.artifact.bytes, emitted.bytes);
        assert_eq!(profile.footprint.module_bytes, emitted.bytes.len() as u64);
        assert!(profile.footprint.defined_functions > 0);
        assert!(profile.footprint.code_body_bytes >= profile.footprint.largest_code_body_bytes);
        assert_eq!(
            WasmArtifactFootprint::from_module_bytes(&emitted.bytes).unwrap(),
            profile.footprint
        );
        assert!(WasmArtifactFootprint::from_module_bytes(
            &emitted.bytes[..emitted.bytes.len() - 1]
        )
        .is_err());
        assert!(engine
            .profile_script_compilation("let = ;", CompileOptions::default())
            .is_err());
    }
}
