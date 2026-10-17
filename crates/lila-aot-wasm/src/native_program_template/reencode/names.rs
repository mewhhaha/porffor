use super::*;
pub(super) fn subsection(
    encoder: &mut Encoder<'_>,
    names: &mut wasm_encoder::NameSection,
    name: wasmparser::Name<'_>,
) -> Result<(), reencode::Error<String>> {
    match name {
        wasmparser::Name::Function(reader) => {
            let mut map = wasm_encoder::NameMap::new();
            for row in reader {
                let row = row?;
                if let Some(index) = encoder.plan.function(row.index) {
                    map.append(index, row.name);
                }
            }
            names.functions(&map);
            Ok(())
        }
        wasmparser::Name::Local(reader) => indirect(encoder, names, reader, false),
        wasmparser::Name::Label(reader) => indirect(encoder, names, reader, true),
        // No data segment remains in a split module. Exact raw source/data
        // provenance stays in its typed artifact and the fresh data images.
        wasmparser::Name::Data(_) => Ok(()),
        other => reencode::utils::parse_custom_name_subsection(encoder, names, other),
    }
}
fn indirect(
    encoder: &mut Encoder<'_>,
    names: &mut wasm_encoder::NameSection,
    reader: wasmparser::IndirectNameMap<'_>,
    labels: bool,
) -> Result<(), reencode::Error<String>> {
    let mut map = wasm_encoder::IndirectNameMap::new();
    for row in reader {
        let row = row?;
        let Some(index) = encoder.plan.function(row.index) else {
            continue;
        };
        let mut inner = wasm_encoder::NameMap::new();
        for entry in row.names {
            let entry = entry?;
            inner.append(entry.index, entry.name);
        }
        map.append(index, &inner);
    }
    if labels {
        names.labels(&map);
    } else {
        names.locals(&map);
    }
    Ok(())
}
