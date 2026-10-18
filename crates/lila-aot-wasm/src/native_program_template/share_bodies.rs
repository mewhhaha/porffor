//! Share executable bodies, never JavaScript objects or declaration identities.
use super::{invalid, plan::Plan, NativeProgramTemplateError};
use crate::native_program_contexts::FunctionContext;
use std::collections::BTreeMap;
use wasm_encoder::reencode::{self, Reencode};
mod mapping;
mod names;
#[cfg(test)]
mod tests;

/// Minted only from a checked program-owner witness, before it is consumed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Owner {
    local: u32,
    ready: u32,
}

pub(super) fn encode(
    bytes: Vec<u8>,
    checked: &Plan,
) -> Result<Vec<u8>, NativeProgramTemplateError> {
    let Some(contexts) = checked.contexts.as_ref().filter(|_| checked.reusable) else {
        return Ok(bytes);
    };
    let owners = contexts
        .bodies
        .iter()
        .filter_map(|(body, context)| {
            let FunctionContext::Program { local, ready, .. } = context else {
                return None;
            };
            Some((
                checked.function(checked.imported_functions + body).unwrap(),
                Owner {
                    local: *local,
                    ready: *ready,
                },
            ))
        })
        .collect();
    transform(bytes, &owners)
}

fn transform(
    bytes: Vec<u8>,
    owners: &BTreeMap<u32, Owner>,
) -> Result<Vec<u8>, NativeProgramTemplateError> {
    let Some(mapping) = mapping::Mapping::read(&bytes, owners)? else {
        return Ok(bytes);
    };
    let mut encoder = Encoder { mapping, body: 0 };
    let mut module = wasm_encoder::Module::new();
    encoder
        .parse_core_module(&mut module, wasmparser::Parser::new(0), &bytes)
        .map_err(invalid)?;
    Ok(module.finish())
}

struct Encoder {
    mapping: mapping::Mapping,
    body: usize,
}
impl Reencode for Encoder {
    type Error = String;
    fn function_index(&mut self, index: u32) -> Result<u32, reencode::Error<String>> {
        self.mapping
            .functions
            .get(index as usize)
            .copied()
            .ok_or_else(|| {
                reencode::Error::UserError("shared body function index out of range".into())
            })
    }
    fn parse_function_section(
        &mut self,
        section: &mut wasm_encoder::FunctionSection,
        reader: wasmparser::FunctionSectionReader<'_>,
    ) -> Result<(), reencode::Error<String>> {
        for (index, ty) in reader.into_iter().enumerate() {
            let ty = ty?;
            if self.mapping.keep[index] {
                section.function(self.type_index(ty)?);
            }
        }
        Ok(())
    }
    fn parse_function_body(
        &mut self,
        code: &mut wasm_encoder::CodeSection,
        body: wasmparser::FunctionBody<'_>,
    ) -> Result<(), reencode::Error<String>> {
        let keep = self.mapping.keep[self.body];
        self.body += 1;
        if keep {
            reencode::utils::parse_function_body(self, code, body)?;
        }
        Ok(())
    }
    fn parse_custom_name_subsection(
        &mut self,
        section: &mut wasm_encoder::NameSection,
        name: wasmparser::Name<'_>,
    ) -> Result<(), reencode::Error<String>> {
        names::subsection(self, section, name)
    }
}
