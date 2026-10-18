//! Retire unused adapter signatures without changing any live type's identity.
use super::{invalid, NativeProgramTemplateError};
use std::collections::BTreeSet;
use wasm_encoder::{
    reencode::{self, Reencode},
    Encode, Module, RawSection,
};
use wasmparser::{CompositeInnerType, Parser, Payload};

#[derive(Default)]
struct Uses(BTreeSet<u32>);

impl Reencode for Uses {
    type Error = String;

    fn type_index(&mut self, index: u32) -> Result<u32, reencode::Error<String>> {
        self.0.insert(index);
        Ok(index)
    }
}

pub(super) fn encode(bytes: Vec<u8>) -> Result<Vec<u8>, NativeProgramTemplateError> {
    // Use the same exhaustive adapters as the actual encoder, including type
    // references inside type definitions, locals and instruction immediates.
    // This deliberately retains references from otherwise unused definitions.
    // An unsupported/canonicalized reference fails rather than guessing a slot.
    let mut uses = Uses::default();
    uses.parse_core_module(&mut Module::new(), Parser::new(0), &bytes)
        .map_err(invalid)?;

    let mut module = Module::new();
    let mut changed = false;
    for payload in Parser::new(0).parse_all(&bytes) {
        let payload = payload.map_err(invalid)?;
        if let Payload::TypeSection(reader) = payload {
            let mut data = Vec::new();
            reader.count().encode(&mut data);
            let end = reader.range().end;
            let groups = reader
                .into_iter_with_offsets()
                .collect::<Result<Vec<_>, _>>()
                .map_err(invalid)?;
            let mut index = 0_u32;
            for (ordinal, (start, group)) in groups.iter().enumerate() {
                let end = groups.get(ordinal + 1).map_or(end, |(offset, _)| *offset);
                let retire = !group.is_explicit_rec_group()
                    && group.types().next().is_some_and(|ty| {
                        ty.is_final
                            && ty.supertype_idx.is_none()
                            && !ty.composite_type.shared
                            && ty.composite_type.descriptor_idx.is_none()
                            && ty.composite_type.describes_idx.is_none()
                            && matches!(ty.composite_type.inner, CompositeInnerType::Func(_))
                    })
                    && !uses.0.contains(&index);
                if retire {
                    // A final empty struct occupies exactly one slot but needs
                    // no Wasmtime function trampoline. Never renumber types or
                    // rewrite the bytes/boundaries of a recursive group.
                    data.extend_from_slice(&[0x5f, 0]);
                    changed = true;
                } else {
                    data.extend_from_slice(&bytes[*start..end]);
                }
                index += group.types().len() as u32;
            }
            module.section(&RawSection { id: 1, data: &data });
        } else if let Some((id, range)) = payload.as_section() {
            module.section(&RawSection {
                id,
                data: &bytes[range],
            });
        }
    }
    Ok(if changed { module.finish() } else { bytes })
}

#[cfg(test)]
#[path = "unused_function_types/tests.rs"]
mod tests;
