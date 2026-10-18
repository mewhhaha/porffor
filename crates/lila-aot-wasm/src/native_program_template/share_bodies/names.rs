use super::*;
pub(super) fn subsection(
    encoder: &mut Encoder,
    section: &mut wasm_encoder::NameSection,
    name: wasmparser::Name<'_>,
) -> Result<(), reencode::Error<String>> {
    match name {
        wasmparser::Name::Function(reader) => {
            let mut map = wasm_encoder::NameMap::new();
            for row in reader {
                let row = row?;
                if encoder.mapping.retained(row.index) {
                    let index = encoder.function_index(row.index)?;
                    if row.name.starts_with("js::") {
                        map.append(index, &format!("js::native_function_{index}"));
                    } else {
                        map.append(index, row.name);
                    }
                }
            }
            section.functions(&map);
            Ok(())
        }
        wasmparser::Name::Local(reader) => indirect(encoder, section, reader, false),
        wasmparser::Name::Label(reader) => indirect(encoder, section, reader, true),
        other => reencode::utils::parse_custom_name_subsection(encoder, section, other),
    }
}
fn indirect(
    encoder: &mut Encoder,
    section: &mut wasm_encoder::NameSection,
    reader: wasmparser::IndirectNameMap<'_>,
    labels: bool,
) -> Result<(), reencode::Error<String>> {
    let mut map = wasm_encoder::IndirectNameMap::new();
    for row in reader {
        let row = row?;
        if !encoder.mapping.retained(row.index) {
            continue;
        }
        let mut inner = wasm_encoder::NameMap::new();
        for entry in row.names {
            let entry = entry?;
            inner.append(entry.index, entry.name);
        }
        map.append(encoder.function_index(row.index)?, &inner);
    }
    if labels {
        section.labels(&map);
    } else {
        section.locals(&map);
    }
    Ok(())
}
