use super::*;
mod signatures;
use crate::native_program_contexts::{FunctionContext, ModuleContext, Schema};
use std::collections::BTreeMap;
use wasmparser::{CompositeInnerType, Operator, Payload, StorageType, ValType};

/// Every rewritten source operand can obtain the metadata of its actual owner.
pub(super) struct OwnedContexts {
    pub schema: Schema,
    pub bootstrap_body: u32,
    pub payload_site: usize,
    pub bodies: BTreeMap<u32, FunctionContext>,
}

impl OwnedContexts {
    pub fn read(
        bytes: &[u8],
        rows: Vec<ModuleContext>,
        bodies: &[wasmparser::FunctionBody<'_>],
        sites: &BTreeMap<usize, super::plan::Site>,
        literal_sites: &BTreeMap<usize, u32>,
    ) -> Result<Option<Self>, NativeProgramTemplateError> {
        let initializers = rows
            .iter()
            .filter_map(|row| match row.context {
                FunctionContext::Bootstrap {
                    schema,
                    payload_operator: Some(operator),
                } => Some((row.body, schema, operator)),
                _ => None,
            })
            .collect::<Vec<_>>();
        let [(bootstrap_body, schema, payload_operator)] = initializers.as_slice() else {
            return Err(invalid(
                "native contexts require exactly one completed bootstrap",
            ));
        };
        let (bootstrap_body, schema, payload_operator) =
            (*bootstrap_body, *schema, *payload_operator);
        if schema.payload_field != crate::gc_types::ProgramInstanceLocals::native_payload_field()
            || rows
                .iter()
                .any(|row| row.context.schema() != schema || row.body as usize >= bodies.len())
        {
            return Err(invalid(
                "native program contexts have inconsistent schemas or bodies",
            ));
        }
        // Check the declared wire indices against the actual type section.
        let mut next_type = 0;
        let mut program = None;
        let mut metadata = None;
        for payload in wasmparser::Parser::new(0).parse_all(bytes) {
            if let Payload::TypeSection(reader) = payload.map_err(invalid)? {
                for group in reader {
                    for ty in group.map_err(invalid)?.types() {
                        if next_type == schema.program_type {
                            if let CompositeInnerType::Struct(ty) = &ty.composite_type.inner {
                                program = ty.fields.get(schema.payload_field as usize).copied();
                            }
                        }
                        if next_type == schema.payload_type {
                            if let CompositeInnerType::Array(ty) = &ty.composite_type.inner {
                                metadata = Some(
                                    !ty.0.mutable && matches!(ty.0.element_type, StorageType::I8),
                                );
                            }
                        }
                        next_type += 1;
                    }
                }
            }
        }
        let valid_program = program.is_some_and(|field| {
            !field.mutable
                && matches!(field.element_type,
            StorageType::Val(ValType::Ref(reference)) if !reference.is_nullable()
                && matches!(reference.heap_type(), wasmparser::HeapType::Concrete(index)
                    if index.as_module_index() == Some(schema.payload_type)))
        });
        if !valid_program || metadata != Some(true) {
            return Err(invalid(
                "native metadata context does not match the actual immutable P field",
            ));
        }
        let rows = rows
            .into_iter()
            .map(|row| (row.body, row.context))
            .collect::<BTreeMap<_, _>>();
        let signatures = signatures::Signatures::read(bytes)?;
        let initializer = signatures.initializer(bootstrap_body, schema)?;
        let mut payload_site = None;
        let mut complete = true;
        for (index, body) in bodies.iter().enumerate() {
            let mut reader = body.get_operators_reader().map_err(invalid)?;
            let mut operators = Vec::new();
            while !reader.eof() {
                let offset = reader.original_position();
                operators.push((offset, reader.read().map_err(invalid)?));
            }
            if operators.iter().any(|(_, op)| matches!(op,
                Operator::Call { function_index } | Operator::ReturnCall { function_index } | Operator::RefFunc { function_index }
                    if *function_index == initializer)) {
                return Err(invalid("native bootstrap cannot be referenced with its original parameterless type"));
            }
            if let Some(FunctionContext::Program { local, ready, .. }) = rows.get(&(index as u32)) {
                signatures.local(index as u32, body, *local, schema)?;
                if !operators.iter().take(*ready as usize).any(|(_, op)| matches!(op,
                    Operator::LocalSet { local_index } | Operator::LocalTee { local_index } if local_index == local)) {
                    return Err(invalid("native program owner is not initialized before its ready witness"));
                }
            }
            if index == bootstrap_body as usize {
                match operators.get(payload_operator as usize) {
                    Some((
                        offset,
                        Operator::ArrayNewFixed {
                            array_type_index,
                            array_size: 0,
                        },
                    )) if *array_type_index == schema.payload_type => payload_site = Some(*offset),
                    _ => {
                        return Err(invalid(
                            "native bootstrap source payload slot differs from its witness",
                        ))
                    }
                }
            }
            for (ordinal, (offset, _)) in operators.iter().enumerate() {
                if !sites.contains_key(offset) && !literal_sites.contains_key(offset) {
                    continue;
                }
                complete &= match rows.get(&(index as u32)) {
                    Some(FunctionContext::Bootstrap { .. }) => true,
                    Some(FunctionContext::Program { ready, .. }) => ordinal >= *ready as usize,
                    None => false,
                };
            }
            if let Some(FunctionContext::Program { ready, .. }) = rows.get(&(index as u32)) {
                if *ready as usize > operators.len() {
                    return Err(invalid(
                        "native program context becomes ready beyond its body",
                    ));
                }
            }
        }
        let payload_site =
            payload_site.ok_or_else(|| invalid("native bootstrap metadata site is absent"))?;
        Ok(complete.then_some(Self {
            schema,
            bootstrap_body,
            payload_site,
            bodies: rows,
        }))
    }
}
