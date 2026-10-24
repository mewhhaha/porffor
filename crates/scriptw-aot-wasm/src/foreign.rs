//! Program-owned synchronous scalar imports and their persisted authority.
use std::collections::BTreeMap;

use scriptw_ir::foreign::{
    ForeignFunctionDeclaration, ForeignFunctionId, ForeignModuleRegistry, ForeignResultType,
    ForeignScalarType,
};
use wasm_encoder::{EntityType, ImportSection, TagKind, TagType};

use crate::function_layout::{ForeignImportFunctionIndex, FunctionIndexLayout};
use crate::EmitError;

/// Canonical ABI-v1 linked requirements. Empty registries emit no section.
pub const FOREIGN_MODULES_CUSTOM_SECTION: &str = "scriptw.foreign.v1";
const _: () = assert!(
    scriptw_ir::foreign::FOREIGN_ABI_VERSION == 1,
    "AOT scalar signatures and persisted metadata must track the declared foreign ABI version"
);

/// Native import type indices cannot be confused with callable function indices.
#[derive(Clone, Copy)]
struct ForeignFunctionTypeIndex(u32);

#[derive(Clone, Copy)]
struct ForeignErrorTagTypeIndex(u32);

/// Filled only while appending the exact P type declarations.
#[derive(Default)]
pub(crate) struct ForeignImportTypes {
    functions: BTreeMap<ForeignFunctionId, ForeignFunctionTypeIndex>,
    error_tag: Option<ForeignErrorTagTypeIndex>,
}

impl ForeignImportTypes {
    pub(crate) fn register_function(&mut self, id: ForeignFunctionId, index: u32) {
        assert!(self
            .functions
            .insert(id, ForeignFunctionTypeIndex(index))
            .is_none());
    }

    pub(crate) fn register_error_tag(&mut self, index: u32) {
        assert!(self
            .error_tag
            .replace(ForeignErrorTagTypeIndex(index))
            .is_none());
    }
}

/// P currently has no other Wasm tags: ordinary JS abrupt completion uses its
/// existing Completion ABI. This index is minted together with its one import.
#[derive(Clone, Copy)]
pub(crate) struct ForeignErrorTagIndex(u32);

impl ForeignErrorTagIndex {
    pub(crate) fn catch(self, label: u32) -> wasm_encoder::Catch {
        wasm_encoder::Catch::One { tag: self.0, label }
    }
}

#[derive(Clone)]
pub(crate) struct PlannedForeignFunction {
    pub(crate) index: ForeignImportFunctionIndex,
    pub(crate) parameters: Vec<ForeignScalarType>,
    pub(crate) result: ForeignResultType,
}

/// Compilation-time declarations only; never Rust callbacks or ambient state.
#[derive(Default)]
pub(crate) struct ForeignImportPlan {
    registry: ForeignModuleRegistry,
    functions: BTreeMap<ForeignFunctionId, PlannedForeignFunction>,
    types: ForeignImportTypes,
    error_tag: Option<ForeignErrorTagIndex>,
}

impl ForeignImportPlan {
    pub(crate) fn new(
        registry: &ForeignModuleRegistry,
        types: ForeignImportTypes,
        layout: &FunctionIndexLayout,
    ) -> Result<Self, EmitError> {
        let mut functions = BTreeMap::new();
        for (id, declaration) in registry.functions() {
            if !types.functions.contains_key(&id) {
                return Err(foreign_error(
                    "declared function has no registered Wasm type",
                ));
            }
            functions.insert(
                id,
                PlannedForeignFunction {
                    index: layout.foreign_import_index(id)?,
                    parameters: declaration.parameters().to_vec(),
                    result: declaration.result(),
                },
            );
        }
        if functions.len() != types.functions.len()
            || registry.error_tag().is_some() != types.error_tag.is_some()
        {
            return Err(foreign_error("declaration and type inventories differ"));
        }
        let error_tag = registry.error_tag().map(|_| ForeignErrorTagIndex(0));
        Ok(Self {
            registry: registry.clone(),
            functions,
            types,
            error_tag,
        })
    }

    pub(crate) fn function(
        &self,
        id: ForeignFunctionId,
    ) -> Result<&PlannedForeignFunction, EmitError> {
        self.functions
            .get(&id)
            .ok_or_else(|| foreign_error("wrapper references an undeclared import"))
    }

    pub(crate) const fn error_tag(&self) -> Option<ForeignErrorTagIndex> {
        self.error_tag
    }

    pub(crate) fn emit_imports(&self, imports: &mut ImportSection) {
        for (id, declaration) in self.registry.functions() {
            imports.import(
                declaration.import_module(),
                declaration.import_name(),
                EntityType::Function(self.types.functions[&id].0),
            );
        }
        if let Some(tag) = self.registry.error_tag() {
            imports.import(
                tag.module(),
                tag.name(),
                EntityType::Tag(TagType {
                    kind: TagKind::Exception,
                    func_type_idx: self
                        .types
                        .error_tag
                        .expect("planned tag has its registered type")
                        .0,
                }),
            );
        }
    }
}

fn foreign_error(message: impl std::fmt::Display) -> EmitError {
    EmitError::unsupported(format!("foreign module ABI: {message}"))
}

/// Read canonical linked declarations from P and check them against its actual
/// Wasm imports/types. No source file or compiler-process callback registry is
/// consulted. The caller still admits the complete Wasm body/runtime pairing.
///
/// An absent section permits only Scriptw's existing host/runtime namespaces.
/// Unexpected functions, duplicate sections, undeclared tags, unsupported
/// scalar signatures and metadata/import disagreement fail closed.
pub fn foreign_registry_from_module(bytes: &[u8]) -> Result<ForeignModuleRegistry, EmitError> {
    use wasmparser::{CompositeInnerType, Encoding, Parser, Payload, TypeRef};
    let mut metadata = None;
    let mut types = Vec::new();
    let mut functions = Vec::new();
    let mut tags = Vec::new();
    let mut reached_foreign_functions = false;
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.map_err(foreign_error)? {
            Payload::Version {
                encoding: Encoding::Module,
                ..
            } => {}
            Payload::Version { .. } => {
                return Err(foreign_error("expected a core Wasm program module"))
            }
            Payload::TypeSection(groups) => {
                for group in groups {
                    for subtype in group.map_err(foreign_error)?.into_types() {
                        types.push(match subtype.composite_type.inner {
                            CompositeInnerType::Func(signature) => Some(signature),
                            CompositeInnerType::Array(_)
                            | CompositeInnerType::Struct(_)
                            | CompositeInnerType::Cont(_) => None,
                        });
                    }
                }
            }
            Payload::ImportSection(imports) => {
                for import in imports.into_imports() {
                    let import = import.map_err(foreign_error)?;
                    match import.ty {
                        TypeRef::Func(index) | TypeRef::FuncExact(index) => {
                            if import.module != crate::module::HOST_IMPORT_MODULE
                                && import.module != crate::RUNTIME_IMPORT_NAMESPACE
                            {
                                reached_foreign_functions = true;
                                functions.push((
                                    import.module.to_owned(),
                                    import.name.to_owned(),
                                    index,
                                ));
                            } else if reached_foreign_functions {
                                return Err(foreign_error(
                                    "common H/R function imports must precede P foreign imports",
                                ));
                            }
                        }
                        TypeRef::Tag(tag) => tags.push((
                            import.module.to_owned(),
                            import.name.to_owned(),
                            tag.func_type_idx,
                        )),
                        TypeRef::Memory(_) | TypeRef::Table(_) | TypeRef::Global(_) => {
                            if import.module != crate::module::HOST_IMPORT_MODULE
                                && import.module != crate::RUNTIME_IMPORT_NAMESPACE
                            {
                                return Err(foreign_error("v1 permits only declared scalar functions and the optional error tag"));
                            }
                        }
                    }
                }
            }
            Payload::CustomSection(section) if section.name() == FOREIGN_MODULES_CUSTOM_SECTION => {
                if metadata.replace(section.data()).is_some() {
                    return Err(foreign_error("duplicate linked requirements section"));
                }
            }
            _ => {}
        }
    }
    let registry = match metadata {
        Some(wire) => {
            let registry =
                ForeignModuleRegistry::from_normalized_json(wire).map_err(foreign_error)?;
            if registry.is_empty() {
                return Err(foreign_error(
                    "empty linked requirements must omit their section",
                ));
            }
            registry
        }
        None => ForeignModuleRegistry::default(),
    };
    if functions.len() != registry.functions().count() {
        return Err(foreign_error(
            "actual foreign import count differs from linked requirements",
        ));
    }
    for ((module, name, type_index), (_, declaration)) in functions.iter().zip(registry.functions())
    {
        if module != declaration.import_module() || name != declaration.import_name() {
            return Err(foreign_error(
                "actual foreign import order/name differs from linked requirements",
            ));
        }
        let signature = types
            .get(*type_index as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| foreign_error("foreign import does not name a function type"))?;
        require_scalar_signature(signature, declaration)?;
    }
    match (registry.error_tag(), tags.as_slice()) {
        (None, []) => {}
        (Some(declared), [(module, name, index)]) => {
            let signature = types
                .get(*index as usize)
                .and_then(Option::as_ref)
                .ok_or_else(|| foreign_error("foreign error tag does not name a function type"))?;
            if module != declared.module()
                || name != declared.name()
                || signature.params() != [wasmparser::ValType::I32]
                || !signature.results().is_empty()
            {
                return Err(foreign_error(
                    "foreign error tag import differs from its i32-code ABI",
                ));
            }
        }
        _ => {
            return Err(foreign_error(
                "actual error tag inventory differs from linked requirements",
            ))
        }
    }
    Ok(registry)
}

fn require_scalar_signature(
    actual: &wasmparser::FuncType,
    declaration: &ForeignFunctionDeclaration,
) -> Result<(), EmitError> {
    let parameters = declaration
        .parameters()
        .iter()
        .map(|parameter| match parameter {
            ForeignScalarType::Number => wasmparser::ValType::F64,
        })
        .collect::<Vec<_>>();
    let results = match declaration.result() {
        ForeignResultType::Number => vec![wasmparser::ValType::F64],
        ForeignResultType::Void => Vec::new(),
    };
    if actual.params() != parameters || actual.results() != results {
        return Err(foreign_error(
            "foreign function signature differs from linked requirements",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
