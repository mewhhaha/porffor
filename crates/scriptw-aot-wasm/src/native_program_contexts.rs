//! Producer-owned provenance for a native body's actual program data owner.
use std::borrow::Cow;

pub(crate) const SECTION_NAME: &str = "scriptw.native_program.contexts";
const VERSION: u32 = 1;
const RECORD_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Schema {
    pub(crate) program_type: u32,
    pub(crate) payload_type: u32,
    pub(crate) payload_field: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FunctionContext {
    Bootstrap {
        schema: Schema,
        payload_operator: Option<u32>,
    },
    Program {
        schema: Schema,
        local: u32,
        ready: u32,
    },
}

impl FunctionContext {
    pub(crate) fn schema(self) -> Schema {
        match self {
            Self::Bootstrap { schema, .. } | Self::Program { schema, .. } => schema,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ModuleContext {
    pub(crate) body: u32,
    pub(crate) context: FunctionContext,
}

pub(crate) fn section(contexts: &[ModuleContext]) -> Option<wasm_encoder::CustomSection<'static>> {
    if contexts.is_empty() {
        return None;
    }
    let mut data = Vec::with_capacity(8 + contexts.len() * RECORD_BYTES);
    data.extend_from_slice(&VERSION.to_le_bytes());
    data.extend_from_slice(
        &u32::try_from(contexts.len())
            .expect("context count fits u32")
            .to_le_bytes(),
    );
    for row in contexts {
        let schema = row.context.schema();
        let (kind, first, second, third) = match row.context {
            FunctionContext::Program { local, ready, .. } => (0, local, ready, 0),
            FunctionContext::Bootstrap {
                payload_operator: Some(operator),
                ..
            } => (1, operator, 0, 0),
            FunctionContext::Bootstrap {
                payload_operator: None,
                ..
            } => {
                panic!("a published native bootstrap owns its source payload slot")
            }
        };
        for word in [
            row.body,
            kind,
            schema.program_type,
            schema.payload_type,
            schema.payload_field,
            first,
            second,
            third,
        ] {
            data.extend_from_slice(&word.to_le_bytes());
        }
    }
    Some(wasm_encoder::CustomSection {
        name: Cow::Borrowed(SECTION_NAME),
        data: Cow::Owned(data),
    })
}

pub(crate) fn read(data: &[u8]) -> Result<Vec<ModuleContext>, &'static str> {
    if data.len() < 8 {
        return Err("truncated native program context header");
    }
    let word = |bytes: &[u8]| u32::from_le_bytes(bytes.try_into().unwrap());
    if word(&data[..4]) != VERSION {
        return Err("unknown native program context version");
    }
    let count = word(&data[4..8]) as usize;
    if count
        .checked_mul(RECORD_BYTES)
        .and_then(|bytes| bytes.checked_add(8))
        != Some(data.len())
    {
        return Err("native program context record length mismatch");
    }
    let mut result = Vec::with_capacity(count);
    let mut previous = None;
    for record in data[8..].chunks_exact(RECORD_BYTES) {
        let words = record.chunks_exact(4).map(word).collect::<Vec<_>>();
        let body = words[0];
        if previous.is_some_and(|previous| previous >= body) {
            return Err("native program context bodies are duplicated or unordered");
        }
        previous = Some(body);
        let schema = Schema {
            program_type: words[2],
            payload_type: words[3],
            payload_field: words[4],
        };
        let context = match words[1] {
            0 if words[7] == 0 => FunctionContext::Program {
                schema,
                local: words[5],
                ready: words[6],
            },
            1 if words[6] == 0 && words[7] == 0 => FunctionContext::Bootstrap {
                schema,
                payload_operator: Some(words[5]),
            },
            _ => return Err("unknown native program context kind or reserved bits"),
        };
        result.push(ModuleContext { body, context });
    }
    Ok(result)
}
